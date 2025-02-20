use super::{backend::Backend, camera::Camera, pipeline::voxels::voxel_pipeline};
use wgpu::{util::DeviceExt, Buffer, RenderPipeline};

pub struct VoxelRenderer {
    quad: Buffer,
    voxel_pipeline: RenderPipeline,
}

impl VoxelRenderer {
    pub fn new(backend: &Backend, camera: &Camera) -> Self {
        let quad = backend
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vengine::voxel_quad"),
                contents: bytemuck::cast_slice(&[
                    [0.0f32, 0.0f32, -1.0f32],
                    [0.0f32, 0.0f32, 0.0f32],
                    [1.0f32, 0.0f32, -1.0f32],
                    [1.0f32, 0.0f32, 0.0f32],
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let voxel_pipeline = voxel_pipeline(backend.device(), camera, *backend.surface_format());

        Self {
            quad,
            voxel_pipeline,
        }
    }

    pub fn voxel_pipeline(&self) -> &RenderPipeline {
        &self.voxel_pipeline
    }

    pub fn quad(&self) -> &Buffer {
        &self.quad
    }
}
