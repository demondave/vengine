use crossbeam::channel::Sender;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
};

pub enum Event {
    WindowEvent(WindowEvent),
    DeviceEvent(DeviceEvent),
}

pub struct WindowEventHandler {
    events_sender: Sender<Event>,
    engine_events_sender: Sender<Event>,
}

impl WindowEventHandler {
    pub fn new(events_sender: Sender<Event>, engine_events_sender: Sender<Event>) -> Self {
        Self {
            events_sender,
            engine_events_sender,
        }
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
        self.events_sender.send(Event::DeviceEvent(event)).unwrap();
    }

    fn window_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.engine_events_sender
                    .send(Event::WindowEvent(event.clone()))
                    .unwrap();
            }
            WindowEvent::Resized(_) => {
                self.engine_events_sender
                    .send(Event::WindowEvent(event.clone()))
                    .unwrap();
            }
            _ => {}
        }

        self.events_sender.send(Event::WindowEvent(event)).unwrap();
    }
}
