use cgmath::Point3;
use colorgrad::preset::{plasma, turbo};
use engine::{
    core::{engine::Engine, window::Window},
    renderer::backend::Backend,
    voxel::{
        chunk::{Chunk, CHUNK_SIZE},
        object::ChunkEx,
        terrain::Terrain,
    },
};
use input::EventHandler;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use util::gradient_to_palette;

pub mod engine;
pub mod input;
pub mod util;

pub fn main() {
    env_logger::init();

    let window = Window::new(1000, 1000);

    let window = Arc::new(window);

    let tmp = window.clone();

    std::thread::spawn(move || {
        init(tmp);
    });

    // We need to spawn a new thread because the event loop needs to be run in the main loop
    window.start_event_loop();
}

fn init(window: Arc<Window>) {
    std::thread::sleep(Duration::from_millis(100));

    let backend = pollster::block_on(Backend::new(&window));

    let engine: &'static Engine = Box::leak(Box::new(Engine::new(window.clone(), backend)));

    std::thread::spawn(|| {
        let mut event_handler = EventHandler::new(engine);
        event_handler.handle();
    });

    std::thread::spawn(|| {
        setup(engine);
    });
}

fn setup(engine: &'static Engine) {
    // Setup texture palette
    engine
        .renderer()
        .palette()
        .set_palette(gradient_to_palette(&turbo()));

    // Setup camera
    engine.camera().set_eye(Point3::new(-3.0, 0.5, -3f32));
    engine
        .camera()
        .set_look_at(Point3::new(0.5f32, 0.5f32, -0.5f32));

    let mut chunk = Chunk::empty();

    for y in 0..CHUNK_SIZE {
        for x in 0..CHUNK_SIZE {
            chunk.set(x, 0, y, true, 0);
        }
    }

    let mut chunk = ChunkEx::new(chunk);
    chunk.remesh();
    chunk.allocate(engine.device());

    let mut terrain = Terrain::new(12, engine.device().clone());

    // Render object
    while !engine.exited() {
        let start = Instant::now();

        let mut pass = engine.renderer().start_render_pass().unwrap();

        terrain.render(engine, &mut pass);

        engine.renderer().finish_render_pass(pass);

        let end = Instant::now();

        println!(
            "{:.3}ms -> {:.0} FPS",
            (end - start).as_secs_f64() * 1000.0,
            1.0 / ((end - start).as_secs_f64())
        );
    }
}
