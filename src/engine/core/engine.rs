use crate::engine::renderer::{backend::Backend, camera::Camera, Renderer};
use crossbeam::channel::Receiver;
use std::sync::atomic::{AtomicBool, Ordering};
use wgpu::Device;

use super::window::{handler::Event, window::Window};

pub struct Engine<'a> {
    renderer: Renderer<'a>,
    exited: &'static AtomicBool,
}

impl<'a> Engine<'a> {
    pub fn new(backend: Backend<'a>) -> Self {
        let renderer = Renderer::new(backend);

        Self {
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

    pub fn device(&self) -> &Device {
        self.renderer.backend().device()
    }

    pub fn window(&self) -> &Window {
        self.renderer().window()
    }

    pub fn events(&self) -> &Receiver<Event> {
        self.renderer().window().events()
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
