use cgmath::{Matrix4, SquareMatrix, Vector3};
use chunk::Chunk;
use wgpu::Device;

pub mod chunk;
pub mod quad;

pub struct Object {
    transform: Matrix4<f32>,
    chunks: Vec<(Vector3<i32>, Chunk)>,
}

impl Object {
    pub fn new(device: &Device, position: Vector3<f32>) -> Object {
        Object {
            transform: Matrix4::identity(),

            chunks: Vec::new(),
        }
    }

    pub fn get_transform(&self) -> &Matrix4<f32> {
        &self.transform
    }

    pub fn set_transform(&mut self, transform: Matrix4<f32>) {
        self.transform = transform;
    }

    pub fn add_chunk(&mut self, position: Vector3<i32>, chunk: Chunk) {
        self.chunks.push((position, chunk));
    }

    pub fn chunks(&self) -> &[(Vector3<i32>, Chunk)] {
        &self.chunks
    }
}
