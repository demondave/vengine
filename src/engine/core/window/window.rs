use crate::engine::core::window::application::WindowApplication;
use crate::engine::core::window::event::UserEvent;
use crossbeam::channel::{unbounded, Receiver};
use std::future::Future;
use std::mem::MaybeUninit;
use std::sync::Arc;
use winit::dpi::PhysicalSize;
use winit::window::{CursorGrabMode, WindowAttributes};
use winit::{
    event::Event,
    event_loop::{EventLoop, EventLoopProxy},
    window::WindowId,
};

pub struct WindowBuilder {
    pub(super) window_attributes: WindowAttributes,
    pub(super) cursor_visibility: bool,
    pub(super) cursor_lock: bool,
}

impl WindowBuilder {
    pub fn new() -> WindowBuilder {
        WindowBuilder {
            window_attributes: WindowAttributes::default(),
            cursor_visibility: true,
            cursor_lock: false,
        }
    }

    pub fn size(mut self, width: u32, height: u32) -> WindowBuilder {
        self.window_attributes = self
            .window_attributes
            .with_inner_size(PhysicalSize::new(width, height));
        self
    }

    pub fn cursor_visible(mut self, cursor_visible: bool) -> WindowBuilder {
        self.cursor_visibility = cursor_visible;
        self
    }

    pub fn cursor_lock(mut self, cursor_lock: bool) -> WindowBuilder {
        self.cursor_lock = cursor_lock;
        self
    }

    pub fn set_resizable(mut self, resizable: bool) -> WindowBuilder {
        self.window_attributes.resizable = resizable;
        self
    }

    pub fn build(self) -> (impl Future<Output = Window>, impl FnOnce()) {
        let (window_sender, window_receiver) = unbounded();

        let ret_fut = async move { window_receiver.recv().unwrap() };
        let ret_fn = move || {
            let event_loop: EventLoop<UserEvent> = EventLoop::with_user_event().build().unwrap();

            let mut app = WindowApplication {
                window_builder: Some(self),
                event_loop_proxy: Some(event_loop.create_proxy()),
                init_sender: window_sender,
                event_sender: MaybeUninit::uninit(),
            };

            event_loop.run_app(&mut app).unwrap();
        };

        (ret_fut, ret_fn)
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Window {
    pub(super) window: Arc<winit::window::Window>,
    pub(super) event_proxy: EventLoopProxy<UserEvent>,

    pub(super) event_receiver: Receiver<Event<UserEvent>>,
}

impl Window {
    pub fn window(&self) -> &Arc<winit::window::Window> {
        &self.window
    }

    pub fn dimension(&self) -> (u32, u32) {
        let size = self.window().inner_size();
        (size.width, size.height)
    }

    pub fn events(&self) -> &Receiver<Event<UserEvent>> {
        &self.event_receiver
    }

    pub fn id(&self) -> WindowId {
        self.window().id()
    }

    pub fn exit(&self) {
        self.event_proxy.send_event(UserEvent::Exit).unwrap();
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
