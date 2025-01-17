use cgmath::{Point3, Vector3, Zero};
use colorgrad::preset::turbo;
use engine::{
    core::{engine::Engine, window::Window},
    renderer::backend::Backend,
    voxel::{chunk::Chunk, object::Object},
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
        let event_handler = EventHandler::new(engine);
        event_handler.handle();
    });

    std::thread::spawn(|| {
        run(engine);
    });
}

fn run(engine: &'static Engine) {
    engine
        .renderer()
        .palette()
        .set_palette(gradient_to_palette(&turbo()));

    engine.camera().set_eye(Point3::new(-3.0, 0.5, -3f32));
    engine
        .camera()
        .set_look_at(Point3::new(0.5f32, 0.5f32, -0.5f32));

    let mut object = Object::new(engine.device(), Vector3::zero());

    let mut chunk = Chunk::empty(Vector3::new(1f32, 0f32, 0f32));
    chunk.set(0, 0, 0, true);
    chunk.set(0, 1, 0, true);
    chunk.remesh();
    chunk.allocate(engine.device());

    object.add_chunk(Vector3::zero(), chunk);

    while !engine.exited() {
        let start = Instant::now();
        engine.renderer().render(&object).unwrap();
        let end = Instant::now();
        println!("{:.3}", (end - start).as_secs_f64() * 1000.0)
    }
}
