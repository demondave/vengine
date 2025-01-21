use std::sync::Mutex;

use cgmath::{Matrix, Point3};
use crossbeam::atomic::AtomicCell;
use wgpu::{util::DeviceExt, Buffer, RenderPipeline};

use crate::engine::voxel::object::Object;

use super::{
    backend::Backend, camera::Camera, palette::Palette, pipeline::voxels::voxel_pipeline,
    texture::Texture,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PushConstant {
    transform: [f32; 4 * 4],
    offset: [i32; 3],
}

pub struct Renderer<'a> {
    // Backend
    backend: Backend<'a>,
    // Size (in Pixels)
    size: AtomicCell<(u32, u32)>,
    // Voxel pipeline
    voxel_pipeline: RenderPipeline,
    // Camera
    camera: Camera,
    // Palette
    palette: Palette,
    // Depth texture
    depth_texture: Mutex<Texture>,
    // Quad
    quad: Buffer,
}

impl<'a> Renderer<'a> {
    pub fn new(backend: Backend<'a>, size: (u32, u32)) -> Self {
        // Camera related
        let camera = Camera::new(
            Point3::new(0.0, 5.0, 2.0),
            size.0 as f32 / size.1 as f32,
            backend.device(),
            backend.queue().clone(),
        );

        // Quad
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

        // Palette
        let palette = Palette::new(backend.device(), backend.queue().clone());

        let lock = backend.surface_configuration().lock().unwrap();

        let depth_texture =
            Texture::create_depth_texture(backend.device(), &lock, "engine::depth_texture");

        drop(lock);

        let voxel_pipeline = voxel_pipeline(
            backend.device(),
            &camera,
            &palette,
            *backend.surface_format(),
        );

        Self {
            backend,
            size: AtomicCell::new(size),
            camera,

            palette,
            depth_texture: Mutex::new(depth_texture),
            quad,
            voxel_pipeline,
        }
    }

    pub fn render(&self, object: &Object) -> Result<(), wgpu::SurfaceError> {
        let output = self.backend().surface().get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.backend()
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("vengine::render_encoder"),
                });

        let lock = self.depth_texture.lock().unwrap();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    view: &lock.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.voxel_pipeline);
            render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
            render_pass.set_bind_group(1, self.palette.bind_group(), &[]);

            // Quad buffer (bleibt für alle Chunks gleich)
            render_pass.set_vertex_buffer(0, self.quad.slice(..));

            let mut pc = PushConstant {
                transform: [0f32; 4 * 4],
                offset: [0i32; 3],
            };

            let tmp = unsafe { std::slice::from_raw_parts(object.transform().as_ptr(), 4 * 4) };

            pc.transform[..].copy_from_slice(tmp);

            for (offset, chunk) in object.chunks() {
                if let Some(buffer) = chunk.buffer() {
                    pc.offset = [offset.x, offset.y, offset.z];
                    render_pass.set_push_constants(
                        wgpu::ShaderStages::VERTEX,
                        0,
                        bytemuck::cast_slice(&[pc]),
                    );

                    // Set instance buffer
                    render_pass.set_vertex_buffer(1, buffer.slice(..));

                    // Draw chunk
                    render_pass.draw(0..4, 0..chunk.quads().len() as u32);
                }
            }
        } // render_pass wird hier dropped

        self.backend()
            .queue()
            .submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn resize(&self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.size.store((width, height));
            let mut surface_lock = self.backend().surface_configuration().lock().unwrap();
            surface_lock.width = width;
            surface_lock.height = height;

            self.backend()
                .surface()
                .configure(self.backend().device(), &surface_lock);
            self.camera.set_aspect(width as f32 / height as f32);

            let mut texture_lock = self.depth_texture.lock().unwrap();

            *texture_lock = Texture::create_depth_texture(
                self.backend().device(),
                &surface_lock,
                "engine::depth_texture",
            );
        }
    }
}
