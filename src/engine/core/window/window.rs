use crossbeam::channel::{unbounded, Receiver};
use std::sync::Arc;
use winit::window::{CursorGrabMode, WindowAttributes};
use winit::{event_loop::EventLoop, window::WindowId};

use super::events::WindowEventLoop;
use super::handler::Event;

pub struct Window {
    window: Arc<winit::window::Window>,
    receiver: Receiver<Event>,
}

impl Window {
    pub fn new(attributes: WindowAttributes) -> (Window, WindowEventLoop) {
        let events = EventLoop::new().unwrap();

        #[allow(deprecated)]
        let window = events.create_window(attributes).unwrap();

        let (sender, receiver) = unbounded();

        (
            Window {
                window: Arc::new(window),
                receiver,
            },
            WindowEventLoop::new(events, sender),
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
        &self.receiver
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
