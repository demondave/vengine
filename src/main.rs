use cgmath::{Matrix4, Point3, SquareMatrix, Vector3};
use colorgrad::preset::turbo;
use engine::{
    core::{engine::Engine, window::Window},
    renderer::backend::Backend,
    voxel::{
        chunk::Chunk,
        object::{Object, Properties},
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

    // Create a static and axis aligned voxel object
    let mut properties = Properties::default();
    properties.set_is_static(true);
    properties.set_is_axis_aligned(true);

    let mut object = Object::new(engine.device().clone(), Matrix4::identity(), properties);

    let mut chunk = Chunk::empty();
    for n in 0..31 {
        chunk.set(0, n, 0, true);
    }

    object.add_chunk(Vector3::new(0, 0, 0), chunk, true);

    let mut chunk = Chunk::empty();
    chunk.set(0, 0, 0, true);

    object.add_chunk(Vector3::new(0, 1, 0), chunk, true);

    // Render object
    while !engine.exited() {
        let start = Instant::now();
        engine.renderer().render(&object).unwrap();
        let end = Instant::now();

        println!(
            "{:.3}ms -> {:.0} FPS",
            (end - start).as_secs_f64() * 1000.0,
            1.0 / ((end - start).as_secs_f64())
        );
    }
}
