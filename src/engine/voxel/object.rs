use std::{collections::hash_map::Iter, sync::Arc};

use ahash::{HashMap, HashMapExt};
use cgmath::{Matrix, Matrix4, SquareMatrix, Vector3};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, Device,
};

use super::{chunk::Chunk, quad::Quad};

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
    // GPU Transform Buffer
    transform_buffer: Buffer,
    // Chunks with additional information
    chunks: HashMap<Vector3<i32>, ChunkEx>,
    // Object properties, eg. if the object is axis aligned or static
    properties: Properties,
    // Device
    device: Arc<Device>,
}

impl Object {
    pub fn new(device: Arc<Device>, transform: Matrix4<f32>, properties: Properties) -> Object {
        let slice = unsafe { std::slice::from_raw_parts(transform.as_ptr(), 4 * 4) };

        let transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(slice),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
        });

        Object {
            transform: Matrix4::identity(),
            chunks: HashMap::new(),
            properties,
            transform_buffer,
            device,
        }
    }

    pub fn transform_buffer(&self) -> &Buffer {
        &self.transform_buffer
    }

    pub fn voxelize_from_mesh() -> Object {
        todo!()
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

        chunk.quads.extend_from_slice(&[Quad::default(); 3]);

        let tmp = [offset.x, offset.y, offset.z];
        let dst: &mut [u8] = bytemuck::cast_slice_mut(&mut chunk.quads[0..3]);
        dst.copy_from_slice(bytemuck::cast_slice(&tmp));

        chunk.quads.truncate(3);

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
