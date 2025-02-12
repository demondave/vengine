use crossbeam::{
    atomic::AtomicCell,
    channel::{unbounded, Receiver},
};
use engine::{
    core::{engine::Engine, window::window::Window},
    renderer::backend::Backend,
};
use game::{
    input::{EventHandler, InputHandler},
    ui::menu::main::MainMenu,
    Game,
};
use winit::{event::WindowEvent, window::WindowAttributes};

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

    let (window, mut events) = Window::new(WindowAttributes::default());

    let window: &'static Window = Box::leak(Box::new(window));

    let backend = pollster::block_on(Backend::new(window));

    let engine: &'static Engine = Box::leak(Box::new(Engine::new(window, backend)));

    events.handler_mut().set_engine(engine);

    std::thread::spawn(move || {
        init(engine);
    });

    events.start();
}

fn init(engine: &'static Engine) {
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
