use crate::engine::core::window::WindowBuilder;
use cgmath::Point3;
use colorgrad::preset::turbo;
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
use stats::{Ranking, Stats};
use std::sync::Arc;
use std::time::Instant;
use util::gradient_to_palette;

pub mod engine;
pub mod input;
pub mod stats;
pub mod util;

pub fn main() {
    env_logger::init();

    let (window_fut, run_fn) = WindowBuilder::new()
        .size(1000, 1000)
        .cursor_visible(false)
        .cursor_lock(true)
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
    let mut stats = Stats::default();

    stats.add_metric(
        "fps".to_string(),
        "FPS".to_string(),
        "FPS".to_string(),
        Ranking::Low,
    );
    stats.add_metric(
        "frame_time".to_string(),
        "Frame time".to_string(),
        "ms".to_string(),
        Ranking::High,
    );

    while !engine.exited() {
        let start = Instant::now();

        let mut pass = engine.renderer().start_render_pass().unwrap();

        terrain.render(engine, &mut pass);

        engine.renderer().finish_render_pass(pass);

        let end = Instant::now();

        stats.push_metric("fps", 1.0 / ((end - start).as_secs_f64()));
        stats.push_metric("frame_time", (end - start).as_secs_f64() * 1000.0);

        stats.print();
    }
}
