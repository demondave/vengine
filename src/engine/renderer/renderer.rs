use super::{backend::Backend, camera::Camera, pipeline::voxels::voxel_pipeline, texture::Texture};
use cgmath::Point3;
use crossbeam::atomic::AtomicCell;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use wgpu::{util::DeviceExt, Buffer, RenderPipeline};

pub struct Renderer<'a> {
    // Backend
    backend: Backend<'a>,
    // Current size (in Pixels)
    current_size: AtomicCell<(u32, u32)>,
    // New size (in Pixels)
    new_size: AtomicCell<(u32, u32)>,
    // Flag if the surface has been resized
    resized: AtomicBool,
    // Voxel pipeline
    pub voxel_pipeline: RenderPipeline,
    // Camera
    camera: Camera,
    // Depth texture
    pub depth_texture: Mutex<Texture>,
    // Quad
    pub quad: Buffer,
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

        let lock = backend.surface_configuration().lock().unwrap();

        let depth_texture =
            Texture::create_depth_texture(backend.device(), &lock, "engine::depth_texture");

        drop(lock);

        let voxel_pipeline = voxel_pipeline(backend.device(), &camera, *backend.surface_format());

        Self {
            backend,
            current_size: AtomicCell::new(size),
            new_size: AtomicCell::new((0, 0)),
            camera,
            resized: AtomicBool::new(false),
            depth_texture: Mutex::new(depth_texture),
            quad,
            voxel_pipeline,
        }
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn dimensions(&self) -> (u32, u32) {
        self.current_size.load()
    }

    pub fn resize(&self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.new_size.store((width, height));
            self.resized.store(true, Ordering::Relaxed);
        }
    }

    pub fn reconfigure_surface(&self) {
        if self.resized.load(Ordering::Relaxed) {
            let (width, height) = self.new_size.load();
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
            self.current_size.store((width, height));
            self.resized.store(false, Ordering::Relaxed);
        }
    }
}
