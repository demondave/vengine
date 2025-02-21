use crossbeam::channel::{unbounded, Receiver};
use std::sync::Arc;
use winit::window::{CursorGrabMode, WindowAttributes};
use winit::{event_loop::EventLoop, window::WindowId};

use super::events::WindowEventLoop;
use super::handler::Event;

pub struct Window {
    window: Arc<winit::window::Window>,
    events_receiver: Receiver<Event>,
    engine_events_receiver: Receiver<Event>,
}

impl Window {
    pub fn new(attributes: WindowAttributes) -> (Window, WindowEventLoop) {
        let events = EventLoop::new().unwrap();

        #[allow(deprecated)]
        let window = events.create_window(attributes).unwrap();

        let (events_sender, events_receiver) = unbounded();
        let (engine_events_sender, engine_events_receiver) = unbounded();

        (
            Window {
                window: Arc::new(window),
                events_receiver,
                engine_events_receiver,
            },
            WindowEventLoop::new(events, events_sender, engine_events_sender),
        )
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn dimension(&self) -> (u32, u32) {
        let size = self.window().inner_size();
        (size.width, size.height)
    }

    pub fn events(&self) -> &Receiver<Event> {
        &self.events_receiver
    }

    pub fn engine_events(&self) -> &Receiver<Event> {
        &self.engine_events_receiver
    }

    pub fn id(&self) -> WindowId {
        self.window().id()
    }

    pub fn set_grab(&self, grab: bool) {
        if grab {
            self.window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Locked))
                .unwrap();
        } else {
            self.window.set_cursor_grab(CursorGrabMode::None).unwrap();
        }
    }
}
