pub mod backend;
pub mod camera;
pub mod frame;
pub mod pipeline;
#[allow(clippy::module_inception)]
pub mod size;
pub mod texture;
pub mod ui;
pub mod voxel;

use super::core::window::window::Window;
use backend::Backend;
use camera::Camera;
use crossbeam::atomic::AtomicCell;
use frame::Frame;
use size::Size;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use texture::Texture;
use ui::UiRenderer;
use voxel::VoxelRenderer;
use wgpu::SurfaceTexture;

pub struct Renderer<'a> {
    current_size: AtomicCell<Size>,
    new_size: AtomicCell<Size>,
    resized: AtomicBool,
    voxel_renderer: VoxelRenderer,
    ui_renderer: UiRenderer,
    camera: Camera,
    depth_texture: Mutex<Texture>,
    backend: Backend<'a>,
}

impl<'a> Renderer<'a> {
    pub fn new(backend: Backend<'a>) -> Self {
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

        let voxel_renderer = VoxelRenderer::new(&backend, &camera);

        let ui_renderer = UiRenderer::new(backend.window(), &backend, 1);

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
            ui_renderer,
            voxel_renderer,
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

    pub fn start_frame(&self) -> Frame {
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

    pub fn finish_frame(&self, frame: Frame) {
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

    pub fn voxel_renderer(&self) -> &VoxelRenderer {
        &self.voxel_renderer
    }

    pub fn ui_renderer(&self) -> &UiRenderer {
        &self.ui_renderer
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
