pub mod camera;
pub mod object;
pub mod palette;
pub mod pipelines;
pub mod texture;

use camera::Camera;
use cgmath::Point3;
use object::chunk::Chunk;
use palette::Palette;
use pipelines::voxels::voxel_pipeline;
use std::sync::{Arc, Mutex};
use wgpu::{util::DeviceExt, Buffer, Device, Instance, Queue, RenderPipeline, Surface};

pub struct Engine<'a> {
    size: (u32, u32),
    device: Arc<Device>,
    queue: Arc<Queue>,
    config: wgpu::SurfaceConfiguration,
    camera: Camera,
    voxel_pipeline: RenderPipeline,
    quad: Buffer,
    chunks: Mutex<Vec<Chunk>>,
    depth_texture: texture::Texture,
    palette: Palette,
    surface: Surface<'a>,
}

impl<'a> Engine<'a> {
    pub async fn new(surface: Surface<'a>, instance: Instance, width: u32, height: u32) -> Self {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let queue = Arc::new(queue);

        // Camera related
        let camera = Camera::new(
            Point3::new(0.0, 5.0, 2.0),
            config.width as f32 / config.height as f32,
            &device,
            queue.clone(),
        );

        // Quad
        let quad = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
        let palette = Palette::new(&device, queue.clone());

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "engine::depth_texture");

        Self {
            voxel_pipeline: voxel_pipeline(&device, &camera, &palette, surface_format),
            surface,
            config,
            size: (width, height),
            device: Arc::new(device),
            queue,
            camera,
            quad,
            chunks: Mutex::new(Vec::new()),
            depth_texture,
            palette,
        }
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("vengine::render_encoder"),
            });

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
                view: &self.depth_texture.view,
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
        // Palette
        render_pass.set_bind_group(1, self.palette.bind_group(), &[]);

        // Voxel rendering
        let lock = self.chunks.lock().unwrap();

        for chunk in lock.iter() {
            if let Some(buffer) = chunk.buffer() {
                // Quad
                render_pass.set_vertex_buffer(0, self.quad.slice(..));

                // Instances
                render_pass.set_vertex_buffer(1, buffer.slice(..));

                render_pass.draw(0..4, 0..chunk.quads().len() as u32);
            }
        }

        drop(render_pass);

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.size = (width, height);
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.camera
                .set_aspect(self.config.width as f32 / self.config.height as f32);
            // NEW!
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn add(&self, chunk: Chunk) {
        let mut lock = self.chunks.lock().unwrap();
        lock.push(chunk);
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }
}
