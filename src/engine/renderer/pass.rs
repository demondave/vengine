use cgmath::{Array, Matrix, Matrix4, Vector3};
use wgpu::{CommandEncoder, RenderPass};

use crate::engine::voxel::object::{ChunkEx, Object};

use super::texture::Texture;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstant {
    transform: [f32; 4 * 4],
    offset: [i32; 3],
}

pub struct Pass<'a> {
    encoder: CommandEncoder,
    pass: RenderPass<'a>,
    depth: Texture,
}

impl<'a> Pass<'a> {
    pub fn new(pass: RenderPass<'a>, encoder: CommandEncoder, depth: Texture) -> Pass<'a> {
        Pass {
            encoder,
            pass,
            depth,
        }
    }

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
                self.pass.draw(0..4, 0..chunk.quads().len() as u32);
            }
        }
    }

    pub fn render_chunk(&mut self, transform: Matrix4<f32>, offset: Vector3<i32>, chunk: &ChunkEx) {
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
            self.pass.draw(0..4, 0..chunk.quads().len() as u32);
        }
    }

    pub fn into_inner(self) -> (CommandEncoder, RenderPass<'a>, Texture) {
        (self.encoder, self.pass, self.depth)
    }
}
