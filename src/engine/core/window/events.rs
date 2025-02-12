use std::time::Duration;

use super::handler::{Event, WindowEventHandler};
use crossbeam::channel::Sender;
use winit::{event_loop::EventLoop, platform::pump_events::EventLoopExtPumpEvents};

pub struct WindowEventLoop {
    events: EventLoop<()>,
    handler: WindowEventHandler,
}

impl WindowEventLoop {
    pub fn new(events: EventLoop<()>, sender: Sender<Event>) -> Self {
        WindowEventLoop {
            events,
            handler: WindowEventHandler::new(sender),
        }
    }

    pub fn start(&mut self) {
        loop {
            self.events
                .pump_app_events(Some(Duration::from_millis(100)), &mut self.handler);

            if self.handler.engine.unwrap().exited() {
                break;
            }
        }
    }

    pub fn handler_mut(&mut self) -> &mut WindowEventHandler {
        &mut self.handler
    }
}
