use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crossbeam::channel::Receiver;
use wgpu::Device;
use winit::event::Event;

use super::window::{UserEvent, Window};
use crate::engine::renderer::{backend::Backend, camera::Camera, frame::Frame, renderer::Renderer};
use crate::engine::ui::renderer::UiRenderer;

pub struct Engine<'a> {
    renderer: Renderer<'a>,
    ui_renderer: UiRenderer,
    exited: AtomicBool,
    // Window must be dropped at last
    window: Arc<Window>,
}

impl<'a> Engine<'a> {
    pub fn new(window: Arc<Window>, backend: Backend<'a>) -> Self {
        let renderer = Renderer::new(backend, window.dimension());

        let ui_renderer = UiRenderer::new(window.window(), renderer.backend(), 1);

        Self {
            window,
            renderer,
            ui_renderer,
            exited: AtomicBool::new(false),
        }
    }

    pub fn renderer(&self) -> &Renderer<'a> {
        &self.renderer
    }

    pub fn ui_renderer(&self) -> &UiRenderer {
        &self.ui_renderer
    }

    pub fn start_frame(&self) -> Frame {
        let output = self
            .renderer()
            .backend()
            .surface()
            .get_current_texture()
            .unwrap();

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
        &self.window
    }

    pub fn events(&self) -> &Receiver<Event<UserEvent>> {
        self.window().events()
    }

    pub fn exited(&self) -> bool {
        self.exited.load(Ordering::Relaxed)
    }

    pub fn exit(&self) {
        self.exited.store(true, Ordering::Relaxed);
        // Exit the window
        self.window().exit();
    }
}
