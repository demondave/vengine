use crate::engine::core::window::event::UserEvent;
use crate::engine::core::window::WindowBuilder;
use crate::Window;
use crossbeam::channel::{unbounded, Sender};
use std::mem::MaybeUninit;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, Event, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{CursorGrabMode, WindowId};

pub(super) struct WindowApplication {
    // will be `None` after the window init event was received
    pub(super) window_builder: Option<WindowBuilder>,
    // will be `None` after the window init event was received
    pub(super) event_loop_proxy: Option<EventLoopProxy<UserEvent>>,

    // a window struct will be sent once when window is initialized
    pub(super) init_sender: Sender<Window>,

    // will be set after the window init event was received
    pub(super) event_sender: MaybeUninit<Sender<Event<UserEvent>>>,
}

macro_rules! forward_application_handler_events {
    ($(fn $name:ident (&mut self, $($arg:ident: $ty:ty),*) => $event:expr)*) => {
        $(
            fn $name (&mut self, $($arg: $ty),*) {
                unsafe { self.event_sender.assume_init_ref().send($event).unwrap() };
            }
        )*
    }
}

impl ApplicationHandler<UserEvent> for WindowApplication {
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init {
            let window_builder = self.window_builder.take().unwrap();

            let window = event_loop
                .create_window(window_builder.window_attributes)
                .unwrap();

            window.set_cursor_visible(window_builder.cursor_visibility);

            if window_builder.cursor_lock {
                window
                    .set_cursor_grab(CursorGrabMode::Confined)
                    .or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked))
                    .unwrap();
            }

            let (sender, receiver) = unbounded();

            self.event_sender = MaybeUninit::new(sender);

            self.init_sender
                .send(Window {
                    window: Arc::new(window),
                    event_proxy: self.event_loop_proxy.take().unwrap(),
                    event_receiver: receiver,
                })
                .unwrap();
        }

        unsafe {
            self.event_sender
                .assume_init_ref()
                .send(Event::NewEvents(cause))
                .unwrap()
        };
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Exit => event_loop.exit(),
        }
        unsafe {
            self.event_sender
                .assume_init_ref()
                .send(Event::UserEvent(event))
                .unwrap()
        }
    }

    forward_application_handler_events! {
        fn resumed(&mut self, _event_loop: &ActiveEventLoop) => Event::Resumed
        fn window_event(&mut self, _event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) => Event::WindowEvent { window_id, event }
        fn suspended(&mut self, _event_loop: &ActiveEventLoop) => Event::Suspended
        fn device_event(&mut self, _event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) => Event::DeviceEvent { device_id, event }
        fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) => Event::AboutToWait
        fn exiting(&mut self, _event_loop: &ActiveEventLoop) => Event::LoopExiting
        fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) => Event::MemoryWarning
    }
}
