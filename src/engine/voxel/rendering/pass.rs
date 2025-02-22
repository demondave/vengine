use super::pipeline::VoxelPipeline;
use crate::engine::{
    rendering::{
        configuration::Configuration, frame::Frame, pass::RenderPass, pipeline::GetPipeline,
    },
    voxel::{chunk_mesh::ChunkMesh, object::Object},
};
use cgmath::{Array, Matrix, Matrix4, Vector3};
use wgpu::CommandEncoder;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstant {
    transform: [f32; 4 * 4],
    offset: [i32; 3],
}

pub struct VoxelPass {
    encoder: CommandEncoder,
    pass: wgpu::RenderPass<'static>,
}

impl VoxelPass {
    pub fn render_object(&mut self, object: &Object) {
        let mut pc = PushConstant {
            transform: [0f32; 4 * 4],
            offset: [0i32; 3],
        };

        let tmp = unsafe { std::slice::from_raw_parts(object.transform().as_ptr(), 4 * 4) };

        pc.transform[..].copy_from_slice(tmp);

        for (offset, chunk) in object.chunks() {
            if let Some(buffer) = chunk.buffer() {
                pc.offset = [offset.x, offset.y, offset.z];
                self.pass.set_push_constants(
                    wgpu::ShaderStages::VERTEX,
                    0,
                    bytemuck::cast_slice(&[pc]),
                );

                // Set instance buffer
                self.pass.set_vertex_buffer(1, buffer.slice(..));

                // Draw chunk
                self.pass.draw(0..4, 0..chunk.quads().unwrap().len() as u32);
            }
        }
    }

    pub fn render_chunk(
        &mut self,
        transform: Matrix4<f32>,
        offset: Vector3<i32>,
        chunk: &ChunkMesh,
    ) {
        let mut pc = PushConstant {
            transform: [0f32; 4 * 4],
            offset: [0i32; 3],
        };

        let tmp = unsafe { std::slice::from_raw_parts(transform.as_ptr(), 4 * 4) };
        pc.transform[..].copy_from_slice(tmp);

        let tmp = unsafe { std::slice::from_raw_parts(offset.as_ptr(), 3) };
        pc.offset[..].copy_from_slice(tmp);

        if let Some(buffer) = chunk.buffer() {
            self.pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                bytemuck::cast_slice(&[pc]),
            );

            // Set instance buffer
            self.pass.set_vertex_buffer(1, buffer.slice(..));

            // Draw chunk
            self.pass.draw(0..4, 0..chunk.quads().unwrap().len() as u32);
        }
    }
}

impl RenderPass for VoxelPass {
    type RequiredPipeline = VoxelPipeline;

    fn start<C: Configuration + GetPipeline<VoxelPipeline>>(frame: &Frame<C>) -> Self {
        let view = frame
            .output()
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = frame.renderer().backend().device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("vengine::render_scene_encoder"),
            },
        );

        let depth_view = frame
            .renderer()
            .depth_texture()
            .lock()
            .unwrap()
            .view
            .clone();

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        pass.set_pipeline(frame.renderer().configuration().get_pipeline().pipeline());
        pass.set_bind_group(0, frame.renderer().camera().bind_group(), &[]);

        pass.set_vertex_buffer(
            0,
            frame
                .renderer()
                .configuration()
                .get_pipeline()
                .quad()
                .slice(..),
        );

        Self {
            pass: pass.forget_lifetime(),
            encoder,
        }
    }

    fn finish<C: Configuration + GetPipeline<VoxelPipeline>>(self, frame: &Frame<C>) {
        frame.push_encoder(self.encoder);
    }
}
