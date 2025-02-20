use crossbeam::channel::Sender;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
};

use crate::engine::{core::engine::Engine, renderer::size::Size};

pub enum Event {
    WindowEvent(WindowEvent),
    DeviceEvent(DeviceEvent),
}

pub struct WindowEventHandler {
    sender: Sender<Event>,

    pub engine: Option<&'static Engine<'static>>,
}

impl WindowEventHandler {
    pub fn new(sender: Sender<Event>) -> Self {
        Self {
            sender,
            engine: None,
        }
    }

    pub fn set_engine(&mut self, engine: &'static Engine<'static>) {
        self.engine = Some(engine)
    }
}

impl ApplicationHandler for WindowEventHandler {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        self.sender.send(Event::DeviceEvent(event)).unwrap();
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // Handle closes and resizes immediately
        match event {
            WindowEvent::CloseRequested => {
                self.engine.unwrap().exit();
            }
            WindowEvent::Resized(size) => {
                self.engine.unwrap().renderer().resize(Size {
                    width: size.width,
                    height: size.height,
                    pixels_per_point: 1.0,
                });
            }
            _ => {}
        }

        self.sender.send(Event::WindowEvent(event)).unwrap();
    }
}
