use super::{
    chunk::{Chunk, CHUNK_SIZE},
    chunk_mesh::ChunkMesh,
};
use ahash::{HashMap, HashMapExt};
use cgmath::{Matrix4, Vector3};
use std::{
    collections::{hash_map::Iter, HashSet},
    sync::Arc,
};
use wgpu::Device;

pub struct Object {
    // Object Transform
    transform: Matrix4<f32>,
    // Chunks with additional information
    chunks: HashMap<Vector3<i32>, ChunkMesh>,
    // Device
    device: Arc<Device>,
}

impl Object {
    pub fn new(device: Arc<Device>, transform: Matrix4<f32>) -> Object {
        Object {
            transform,
            chunks: HashMap::new(),
            device,
        }
    }

    pub fn count(&self) -> usize {
        let mut count = 0;
        for chunk in self.chunks.values() {
            count += chunk.chunk().count()
        }

        count
    }

    pub fn transform(&self) -> &Matrix4<f32> {
        &self.transform
    }

    pub fn from_voxels(
        device: Arc<Device>,
        transform: Matrix4<f32>,
        mut voxels: Vec<([i32; 3], [u8; 4])>,
    ) -> Object {
        let mut chunks: HashSet<Vector3<i32>> = HashSet::new();

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut min_z = i32::MAX;

        for (voxel, _) in &voxels {
            min_x = min_x.min(voxel[0]);
            min_y = min_y.min(voxel[1]);
            min_z = min_z.min(voxel[2]);
        }

        min_x = min_x.abs();
        min_y = min_y.abs();
        min_z = min_z.abs();

        for (voxel, _) in &mut voxels {
            voxel[0] = voxel[0].saturating_add(min_x);
            voxel[1] = voxel[1].saturating_add(min_y);
            voxel[2] = voxel[2].saturating_add(min_z);

            let pos = Vector3::new(
                voxel[0] / CHUNK_SIZE as i32,
                voxel[1] / CHUNK_SIZE as i32,
                voxel[2] / CHUNK_SIZE as i32,
            );

            chunks.insert(pos);
        }

        let chunks = chunks.into_iter().collect::<Vec<Vector3<i32>>>();

        let mut chunks = HashMap::from_iter(
            chunks
                .into_iter()
                .map(|c| (c, ChunkMesh::new(Chunk::empty()))),
        );

        for (voxel, c) in &voxels {
            let chunk = chunks
                .get_mut(&Vector3::new(
                    voxel[0] / CHUNK_SIZE as i32,
                    voxel[1] / CHUNK_SIZE as i32,
                    voxel[2] / CHUNK_SIZE as i32,
                ))
                .unwrap();

            let local_x = voxel[0] % CHUNK_SIZE as i32;
            let local_y = voxel[1] % CHUNK_SIZE as i32;
            let local_z = voxel[2] % CHUNK_SIZE as i32;

            chunk.chunk_mut().set(
                local_x as usize,
                local_y as usize,
                local_z as usize,
                true,
                *c,
            );
        }

        for chunk in chunks.values_mut() {
            chunk.remesh();
            chunk.allocate(&device);
        }

        Object {
            transform,
            chunks,

            device,
        }
    }

    pub fn get_transform(&self) -> &Matrix4<f32> {
        &self.transform
    }

    pub fn set_transform(&mut self, transform: Matrix4<f32>) {
        self.transform = transform;
    }

    pub fn add_chunk(&mut self, offset: Vector3<i32>, chunk: Chunk, allocate: bool) {
        let mut chunk = ChunkMesh::new(chunk);

        if allocate {
            chunk.remesh();
            chunk.allocate(&self.device);
        }

        self.chunks.insert(offset, chunk);
    }

    pub fn remove_chunk(&mut self, position: &Vector3<i32>) -> Option<Chunk> {
        self.chunks.remove(position).map(|c| c.into_chunk())
    }

    pub fn get_chunk(&self, position: &Vector3<i32>) -> Option<&ChunkMesh> {
        self.chunks.get(position)
    }

    pub fn get_chunk_mut(&mut self, position: Vector3<i32>) -> Option<&mut ChunkMesh> {
        self.chunks.get_mut(&position)
    }

    pub fn chunks(&self) -> Iter<Vector3<i32>, ChunkMesh> {
        self.chunks.iter()
    }
}
