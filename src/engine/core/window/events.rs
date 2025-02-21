use std::time::Duration;

use super::handler::{Event, WindowEventHandler};
use crossbeam::channel::Sender;
use winit::{event_loop::EventLoop, platform::pump_events::EventLoopExtPumpEvents};

pub struct WindowEventLoop {
    events: EventLoop<()>,
    handler: WindowEventHandler,
}

impl WindowEventLoop {
    pub fn new(
        events: EventLoop<()>,
        events_sender: Sender<Event>,
        engine_events_sender: Sender<Event>,
    ) -> Self {
        WindowEventLoop {
            events,
            handler: WindowEventHandler::new(events_sender, engine_events_sender),
        }
    }

    pub fn pump(&mut self, timeout: Option<Duration>) {
        self.events.pump_app_events(timeout, &mut self.handler);
    }
}
