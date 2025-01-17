use std::sync::Mutex;

use cgmath::Point3;
use crossbeam::atomic::AtomicCell;
use wgpu::{util::DeviceExt, BindGroup, Buffer, RenderPipeline};

use crate::engine::voxel::object::Object;

use super::{
    backend::Backend, camera::Camera, palette::Palette, pipeline::voxels::voxel_pipeline,
    texture::Texture,
};

pub struct Renderer<'a> {
    // Backend
    backend: Backend<'a>,
    // Size (in Pixels)
    size: AtomicCell<(u32, u32)>,
    // Voxel pipeline
    voxel_pipeline: RenderPipeline,
    // Camera
    camera: Camera,
    // Object transform
    object_transform_buffer: Buffer,
    object_transform_bindgroup: BindGroup,
    // Chunk offset
    chunk_offset_buffer: Buffer,
    chunk_offset_bindgroup: BindGroup,
    // Palette
    palette: Palette,
    // Depth texture
    depth_texture: Mutex<Texture>,
    // Quad
    quad: Buffer,
}

impl<'a> Renderer<'a> {
    pub fn new(backend: Backend<'a>, size: (u32, u32)) -> Self {
        // Object transform
        let object_transform_buffer =
            backend
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("vengine::object_transform_buffer"),
                    contents: bytemuck::cast_slice(&[0f32; 4 * 4]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let object_transform_bindgroup_layout =
            backend
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("vengine::object_transform_bindgroup_layout"),
                });

        let object_transform_bindgroup =
            backend
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &object_transform_bindgroup_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: object_transform_buffer.as_entire_binding(),
                    }],
                    label: Some("vengine::chunk_offset_bind_group"),
                });

        // Chunk offset
        let chunk_offset_buffer =
            backend
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("vengine::chunk_offset_buffer"),
                    contents: bytemuck::cast_slice(&[0f32; 3]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let chunk_offset_bindgroup_layout =
            backend
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("vengine::chunk_offset_bind_group_layout"),
                });

        let chunk_offset_bindgroup =
            backend
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &chunk_offset_bindgroup_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: chunk_offset_buffer.as_entire_binding(),
                    }],
                    label: Some("vengine::chunk_offset_bind_group"),
                });

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
            &object_transform_bindgroup_layout,
            &chunk_offset_bindgroup_layout,
        );

        Self {
            backend,
            size: AtomicCell::new(size),
            camera,
            chunk_offset_bindgroup,
            chunk_offset_buffer,
            object_transform_bindgroup,
            object_transform_buffer,
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
        // Camera
        render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
        // Object transform
        render_pass.set_bind_group(1, &self.object_transform_bindgroup, &[]);
        // Chunk offset
        render_pass.set_bind_group(2, &self.chunk_offset_bindgroup, &[]);
        // Palette
        render_pass.set_bind_group(3, self.palette.bind_group(), &[]);

        let mut copy_encoder =
            self.backend()
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("vengine::copy_encoder"),
                });

        for (_, chunk) in object.chunks() {
            if let Some(buffer) = chunk.buffer() {
                copy_encoder.copy_buffer_to_buffer(
                    buffer,
                    0,
                    &self.chunk_offset_buffer,
                    0,
                    size_of::<[f32; 3]>() as u64,
                );

                // Quad
                render_pass.set_vertex_buffer(0, self.quad.slice(..));

                // Instances
                render_pass.set_vertex_buffer(1, buffer.slice(size_of::<[f32; 3]>() as u64..));

                render_pass.draw(0..4, 0..chunk.quads().len() as u32);
            }
        }

        self.backend().queue().submit(Some(copy_encoder.finish()));

        drop(render_pass);

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
