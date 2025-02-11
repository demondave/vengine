use crate::engine::core::window::WindowBuilder;
use crossbeam::{
    atomic::AtomicCell,
    channel::{unbounded, Receiver},
};
use engine::{
    core::{engine::Engine, window::Window},
    renderer::backend::Backend,
};
use game::{
    input::{EventHandler, InputHandler},
    ui::menu::main::MainMenu,
    Game,
};
use std::sync::Arc;
use winit::event::WindowEvent;

pub mod engine;
pub mod game;
pub mod io;
pub mod stats;

#[cfg(not(feature = "profiling"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "profiling")]
#[global_allocator]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> =
    tracy_client::ProfiledAllocator::new(std::alloc::System, 100);

pub const TERRAIN_RENDER_DISTANCE: u32 = 12;

pub fn main() {
    env_logger::init();

    #[cfg(feature = "profiling")]
    {
        use tracy_client::Client;
        Client::start();
    }

    let (window_fut, run_fn) = WindowBuilder::new()
        .size(1000, 1000)
        .cursor_visible(false)
        .cursor_lock(true)
        .title(format!(
            "{} v{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .build();

    std::thread::spawn(move || {
        let window = pollster::block_on(window_fut);
        init(Arc::new(window));
    });

    // We need to spawn a new thread because the event loop needs to be run in the main loop
    run_fn()
}

fn init(window: Arc<Window>) {
    let backend = pollster::block_on(Backend::new(&window));

    let engine: &'static Engine = Box::leak(Box::new(Engine::new(window, backend)));

    let handler: &'static AtomicCell<InputHandler> =
        Box::leak(Box::new(AtomicCell::new(InputHandler::Gui)));

    let (tmp, receiver) = unbounded();

    std::thread::spawn(move || {
        let mut event_handler = EventHandler::new(engine, handler, tmp);
        event_handler.handle();
    });

    start(engine, handler, receiver);
}

fn start(
    engine: &'static Engine,
    handler: &'static AtomicCell<InputHandler>,
    receiver: Receiver<WindowEvent>,
) {
    let mut game = Game::new(engine, handler, Box::new(MainMenu::new()), receiver);

    while !game.exited() {
        game.render();
    }
}
