use crate::engine::rendering::{
    backend::Backend, camera::Camera, configuration::Configuration, size::Size, Renderer,
};
use crossbeam::channel::Receiver;
use std::sync::atomic::{AtomicBool, Ordering};
use wgpu::Device;
use winit::event::WindowEvent;

use super::window::{handler::Event, window::Window};

pub struct Engine<'a, C: Configuration> {
    renderer: Renderer<'a, C>,
    exited: AtomicBool,
}

impl<'a, C: Configuration> Engine<'a, C> {
    pub fn new(configuration: C, backend: Backend<'a>) -> Self {
        let renderer = Renderer::new(configuration, backend);

        Self {
            renderer,
            exited: AtomicBool::new(false),
        }
    }

    pub fn handle_engine_events(&self) {
        while let Ok(event) = self.window().engine_events().try_recv() {
            if let Event::WindowEvent(event) = event {
                match event {
                    WindowEvent::Resized(size) => {
                        self.renderer().resize(Size {
                            width: size.width,
                            height: size.height,
                            pixels_per_point: self.window().window().scale_factor() as f32,
                        });
                    }
                    WindowEvent::CloseRequested => {
                        self.exited.store(true, Ordering::Relaxed);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn renderer(&self) -> &Renderer<'a, C> {
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

    pub fn exit(&self) {
        self.exited.store(true, Ordering::Relaxed);
    }
}
