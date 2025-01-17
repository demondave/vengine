use crossbeam::channel::{unbounded, Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use winit::{
    dpi::PhysicalSize,
    event::Event,
    event_loop::{EventLoop, EventLoopProxy},
    window::{WindowBuilder, WindowId},
};

use crate::engine::util::ptr::as_mut_ptr;

pub struct Window {
    width: u32,
    height: u32,
    window: Option<Arc<winit::window::Window>>,
    sender: Option<Sender<Event<()>>>,
    receiver: Receiver<Event<()>>,
    exit: AtomicBool,
    proxy: Option<EventLoopProxy<()>>,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Self {
        let (sender, receiver) = unbounded::<Event<()>>();

        std::thread::spawn(move || {});

        Self {
            width,
            height,
            window: None,
            sender: Some(sender),
            receiver,
            exit: AtomicBool::new(false),
            proxy: None,
        }
    }

    pub fn start_event_loop(&self) {
        let event_loop = EventLoop::new().unwrap();

        // Window erstellen und über den Channel senden
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(self.width, self.height))
            .build(&event_loop)
            .unwrap();

        // A little unsafe trick
        let sender;

        unsafe {
            *as_mut_ptr(&self.window) = Some(Arc::new(window));

            *as_mut_ptr(&self.proxy) = Some(event_loop.create_proxy());

            let ptr = as_mut_ptr(&self.sender);

            sender = (*ptr).take().unwrap();
        }

        event_loop
            .run(move |event, control_flow| {
                if self.exit.load(Ordering::Relaxed) {
                    control_flow.exit();
                }

                sender.send(event).unwrap();
            })
            .unwrap();
    }

    pub fn window(&self) -> &Arc<winit::window::Window> {
        self.window.as_ref().unwrap()
    }

    pub fn dimension(&self) -> (u32, u32) {
        let size = self.window().inner_size();
        (size.width, size.height)
    }

    pub fn events(&self) -> &Receiver<Event<()>> {
        &self.receiver
    }

    pub fn id(&self) -> WindowId {
        self.window().id()
    }

    pub fn exit(&self) {
        self.exit.store(true, Ordering::Relaxed);
        self.proxy.as_ref().unwrap().send_event(()).unwrap();
    }
}
