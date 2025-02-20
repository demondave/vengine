use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::engine::renderer::{backend::Backend, camera::Camera, Renderer};
use crossbeam::channel::Receiver;
use wgpu::Device;

use super::window::{handler::Event, window::Window};

pub struct Engine<'a> {
    renderer: Renderer<'a>,
    exited: &'static AtomicBool,
    // Window must be dropped at last
    window: &'static Window,
}

impl<'a> Engine<'a> {
    pub fn new(window: &'static Window, backend: Backend<'a>) -> Self {
        let renderer = Renderer::new(backend, window);

        Self {
            window,
            renderer,
            exited: Box::leak(Box::new(AtomicBool::new(false))),
        }
    }

    pub fn renderer(&self) -> &Renderer<'a> {
        &self.renderer
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
