use std::{collections::hash_map::Iter, sync::Arc};

use ahash::{HashMap, HashMapExt};
use cgmath::{Matrix4, Vector3};
use dda_voxelize::DdaVoxelizer;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, Device,
};

use super::{
    chunk::{Chunk, CHUNK_SIZE},
    quad::Quad,
};

#[derive(Clone, Copy, Default)]
pub struct Properties(u8);

impl Properties {
    pub fn set_is_static(&mut self, value: bool) {
        if value {
            self.0 |= 1 << 0;
        } else {
            self.0 &= !(1 << 0);
        }
    }

    pub fn is_static(&self) -> bool {
        (self.0 & (1u8 << 0)) != 0
    }

    pub fn set_is_axis_aligned(&mut self, value: bool) {
        if value {
            self.0 |= 1 << 1;
        } else {
            self.0 &= !(1 << 1);
        }
    }

    pub fn is_axis_aligned(&self) -> bool {
        (self.0 & (1u8 << 1)) != 0
    }
}

pub struct ChunkEx {
    chunk: Chunk,
    quads: Vec<Quad>,
    buffer: Option<Buffer>,
}

impl ChunkEx {
    pub fn quads(&self) -> &[Quad] {
        &self.quads
    }

    pub fn allocate(&mut self, device: &Device) -> bool {
        if !self.quads.is_empty() {
            self.buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.quads),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
            }));

            return true;
        }

        false
    }

    pub fn deallocate(&self) {
        if let Some(buffer) = &self.buffer {
            buffer.destroy();
        }
    }

    pub fn buffer(&self) -> &Option<Buffer> {
        &self.buffer
    }
}

pub struct Object {
    // Object Transform
    transform: Matrix4<f32>,
    // Chunks with additional information
    chunks: HashMap<Vector3<i32>, ChunkEx>,
    // Object properties, eg. if the object is axis aligned or static
    properties: Properties,
    // Device
    device: Arc<Device>,
}

impl Object {
    pub fn new(device: Arc<Device>, transform: Matrix4<f32>, properties: Properties) -> Object {
        Object {
            transform,
            chunks: HashMap::new(),
            properties,
            device,
        }
    }

    pub fn transform(&self) -> &Matrix4<f32> {
        &self.transform
    }

    pub fn voxelize_from_mesh(
        device: Arc<Device>,
        transform: Matrix4<f32>,
        properties: Properties,
        triangles: &[[[f32; 3]; 3]],
    ) -> Object {
        let mut voxelizer = DdaVoxelizer::new();

        for triangle in triangles {
            voxelizer.add_triangle(triangle, &|_, [_, _, _], __| [255u8, 255u8, 255u8]);
        }

        let voxels = voxelizer.finalize();

        let mut chunks: HashMap<Vector3<i32>, ChunkEx> = HashMap::new();

        for (voxel, _) in voxels {
            let pos = Vector3::new(
                voxel[0] / CHUNK_SIZE as i32,
                voxel[1] / CHUNK_SIZE as i32,
                voxel[2] / CHUNK_SIZE as i32,
            );

            let chunk = match chunks.get_mut(&pos) {
                Some(r) => r,
                None => {
                    chunks.insert(
                        pos,
                        ChunkEx {
                            chunk: Chunk::empty(),
                            quads: Vec::new(),
                            buffer: None,
                        },
                    );

                    chunks.get_mut(&pos).unwrap()
                }
            };

            // Berechne lokale Koordinaten im Chunk
            let local_x = voxel[0] - (pos.x * CHUNK_SIZE as i32);
            let local_y = voxel[1] - (pos.y * CHUNK_SIZE as i32);
            let local_z = voxel[2] - (pos.z * CHUNK_SIZE as i32);

            chunk
                .chunk
                .set(local_x as usize, local_y as usize, local_z as usize, true);
        }

        for chunk in chunks.values_mut() {
            chunk.chunk.remesh(&mut chunk.quads);
            chunk.allocate(&device);
        }

        Object {
            transform,
            chunks,
            properties,
            device,
        }
    }

    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    pub fn properties_mut(&mut self) -> &mut Properties {
        &mut self.properties
    }

    pub fn get_transform(&self) -> &Matrix4<f32> {
        &self.transform
    }

    pub fn set_transform(&mut self, transform: Matrix4<f32>) {
        self.transform = transform;
    }

    pub fn add_chunk(&mut self, offset: Vector3<i32>, chunk: Chunk, allocate: bool) {
        let mut chunk = ChunkEx {
            chunk,
            quads: Vec::with_capacity(2 ^ 16),
            buffer: None,
        };

        if allocate {
            chunk.chunk.remesh(&mut chunk.quads);
            chunk.allocate(&self.device);
        }

        self.chunks.insert(offset, chunk);
    }

    pub fn remove_chunk(&mut self, position: &Vector3<i32>) -> Option<Chunk> {
        match self.chunks.remove(position) {
            Some(c) => Some(c.chunk),
            None => None,
        }
    }

    pub fn get_chunk(&self, position: &Vector3<i32>) -> Option<&ChunkEx> {
        self.chunks.get(position)
    }

    pub fn get_chunk_mut(&mut self, position: Vector3<i32>) -> Option<&mut ChunkEx> {
        self.chunks.get_mut(&position)
    }

    pub fn chunks(&self) -> Iter<Vector3<i32>, ChunkEx> {
        self.chunks.iter()
    }
}
