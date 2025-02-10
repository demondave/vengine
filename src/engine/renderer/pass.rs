use crate::engine::voxel::{chunk_mesh::ChunkMesh, object::Object};
use cgmath::{Array, Matrix, Matrix4, Vector3};
use wgpu::{CommandEncoder, RenderPass, SurfaceTexture};

use super::{renderer::Renderer, texture::Texture};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstant {
    transform: [f32; 4 * 4],
    offset: [i32; 3],
}

pub struct Pass<'a> {
    _renderer: &'a Renderer<'a>,
    encoder: *mut CommandEncoder,
    pass: RenderPass<'a>,
    depth: Texture,
    output: SurfaceTexture,
}

impl<'a> Pass<'a> {
    pub fn new(
        renderer: &'a Renderer,
        pass: RenderPass<'a>,
        encoder: *mut CommandEncoder,
        depth: Texture,
        output: SurfaceTexture,
    ) -> Pass<'a> {
        Pass {
            _renderer: renderer,
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

                /*
                let position = object.transform().w.truncate()
                    + offset.cast::<f32>().unwrap() * CHUNK_SIZE as f32 * VOXEL_SIZE;

                let visible = chunk.visible(
                    &self.renderer.camera().get_eye().to_vec(),
                    position,
                    Vector3::zero(),
                );

                let offsets = chunk.offsets();

                for idx in 0..6 {
                    if !visible[idx] {
                        continue;
                    }

                    let start = offsets[idx] as u32;
                    let end = *(offsets
                        .get(idx + 1)
                        .unwrap_or(&(chunk.quads().unwrap().len() as u16)))
                        as u32;

                    // Set instance buffer
                    self.pass.set_vertex_buffer(1, buffer.slice(..));

                    // Draw chunk
                    self.pass.draw(0..4, start..end);
                }
                */

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

    pub fn into_inner(self) -> (Box<CommandEncoder>, RenderPass<'a>, Texture, SurfaceTexture) {
        (
            unsafe { Box::from_raw(self.encoder) },
            self.pass,
            self.depth,
            self.output,
        )
    }
}
