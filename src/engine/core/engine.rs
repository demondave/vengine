use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::engine::renderer::{backend::Backend, camera::Camera, frame::Frame, renderer::Renderer};
use crate::engine::ui::renderer::UiRenderer;
use crossbeam::channel::Receiver;
use wgpu::{Device, SurfaceTexture};

use super::window::{handler::Event, window::Window};

pub struct Engine<'a> {
    renderer: Renderer<'a>,
    ui_renderer: UiRenderer,
    exited: &'static AtomicBool,
    // Window must be dropped at last
    window: &'static Window,
}

impl<'a> Engine<'a> {
    pub fn new(window: &'static Window, backend: Backend<'a>) -> Self {
        let renderer = Renderer::new(backend, window.dimension());

        let ui_renderer = UiRenderer::new(window.window(), renderer.backend(), 1);

        Self {
            window,
            renderer,
            ui_renderer,
            exited: Box::leak(Box::new(AtomicBool::new(false))),
        }
    }

    pub fn renderer(&self) -> &Renderer<'a> {
        &self.renderer
    }

    pub fn ui_renderer(&self) -> &UiRenderer {
        &self.ui_renderer
    }

    pub fn start_frame(&self) -> Frame {
        let output: SurfaceTexture;

        self.renderer().reconfigure_surface();

        loop {
            match self.renderer().backend().surface().get_current_texture() {
                Ok(o) => {
                    output = o;
                    break;
                }
                Err(wgpu::SurfaceError::Outdated) => {
                    self.renderer().reconfigure_surface();
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

    pub fn camera(&self) -> &Camera {
        self.renderer.camera()
    }

    pub fn device(&self) -> &Arc<Device> {
        self.renderer.backend().device()
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    pub fn events(&self) -> &Receiver<Event> {
        self.window().events()
    }

    pub fn exited(&self) -> bool {
        self.exited.load(Ordering::Relaxed)
    }

    pub fn exited_ref(&self) -> &'static AtomicBool {
        self.exited
    }

    pub fn exit(&self) {
        self.exited.store(true, Ordering::Relaxed);
    }
}
