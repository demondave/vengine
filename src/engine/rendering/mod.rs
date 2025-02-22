pub mod backend;
pub mod camera;
pub mod configuration;
pub mod frame;
pub mod pass;
pub mod pipeline;
pub mod size;
pub mod texture;

use super::core::window::window::Window;
use backend::Backend;
use camera::Camera;
use configuration::Configuration;
use crossbeam::atomic::AtomicCell;
use frame::Frame;
use size::Size;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use texture::Texture;
use wgpu::SurfaceTexture;

pub struct Renderer<'a, C: Configuration> {
    current_size: AtomicCell<Size>,
    new_size: AtomicCell<Size>,
    resized: AtomicBool,
    configuration: C,
    camera: Camera,
    depth_texture: Mutex<Texture>,
    backend: Backend<'a>,
}

impl<'a, C: Configuration> Renderer<'a, C> {
    pub fn new(mut configuration: C, backend: Backend<'a>) -> Self {
        let size = Size {
            width: backend.window().dimension().0,
            height: backend.window().dimension().1,
            pixels_per_point: backend.window().window().scale_factor() as f32,
        };

        let camera = Camera::new(
            size.width as f32 / size.height as f32,
            backend.device(),
            backend.queue().clone(),
        );

        configuration.initialize(&backend, &camera);

        let lock = backend.surface_configuration().lock().unwrap();

        let depth_texture =
            Texture::create_depth_texture(backend.device(), &lock, "engine::depth_texture");

        drop(lock);

        Self {
            backend,
            current_size: AtomicCell::new(size),
            new_size: AtomicCell::new(size),
            camera,
            resized: AtomicBool::new(false),
            depth_texture: Mutex::new(depth_texture),
            configuration,
        }
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn size(&self) -> Size {
        self.current_size.load()
    }

    pub fn depth_texture(&self) -> &Mutex<Texture> {
        &self.depth_texture
    }

    pub fn start_frame(&self) -> Frame<C> {
        let output: SurfaceTexture;

        self.handle_resize();

        loop {
            match self.backend().surface().get_current_texture() {
                Ok(o) => {
                    output = o;
                    break;
                }
                Err(wgpu::SurfaceError::Outdated) => {
                    self.reconfigure_surface();
                }
                Err(e) => {
                    panic!("{}", e)
                }
            };
        }

        Frame::new(self, output)
    }

    pub fn finish_frame(&self, frame: Frame<C>) {
        let output = frame.finish();

        output.present();
    }

    pub fn resize(&self, size: Size) {
        if size.width > 0 && size.height > 0 {
            self.new_size.store(size);
            self.resized.store(true, Ordering::Relaxed);
        }
    }

    pub fn window(&self) -> &Window {
        self.backend.window()
    }

    pub fn configuration(&self) -> &C {
        &self.configuration
    }

    pub fn handle_resize(&self) {
        if self.resized.load(Ordering::Relaxed) {
            self.reconfigure_surface();
            self.resized.store(false, Ordering::Relaxed);
        }
    }

    pub fn reconfigure_surface(&self) {
        let size = self.new_size.load();

        let mut surface_lock = self.backend().surface_configuration().lock().unwrap();
        surface_lock.width = size.width;
        surface_lock.height = size.height;

        self.backend()
            .surface()
            .configure(self.backend().device(), &surface_lock);

        let mut texture_lock = self.depth_texture.lock().unwrap();

        *texture_lock = Texture::create_depth_texture(
            self.backend().device(),
            &surface_lock,
            "vengine::depth_texture",
        );

        self.camera
            .set_aspect(size.width as f32 / size.height as f32);

        self.current_size.store(size);
    }
}
