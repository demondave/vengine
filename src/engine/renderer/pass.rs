use cgmath::Matrix;
use wgpu::{CommandEncoder, RenderPass, SurfaceTexture};

use crate::engine::voxel::object::Object;

use super::texture::Texture;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstant {
    transform: [f32; 4 * 4],
    offset: [i32; 3],
}

pub struct Pass<'a> {
    encoder: *mut CommandEncoder,
    pass: RenderPass<'a>,
    depth: Texture,
    output: SurfaceTexture,
}

impl<'a> Pass<'a> {
    pub fn new(
        pass: RenderPass<'a>,
        encoder: *mut CommandEncoder,
        depth: Texture,
        output: SurfaceTexture,
    ) -> Pass<'a> {
        Pass {
            encoder,
            pass,
            depth,
            output,
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

    pub fn into_inner(self) -> (Box<CommandEncoder>, RenderPass<'a>, Texture, SurfaceTexture) {
        (
            unsafe { Box::from_raw(self.encoder) },
            self.pass,
            self.depth,
            self.output,
        )
    }
}
