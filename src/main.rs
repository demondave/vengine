use cgmath::{Matrix4, Point3, SquareMatrix};
use colorgrad::preset::plasma;
use engine::{
    core::{engine::Engine, window::Window},
    renderer::backend::Backend,
    voxel::object::{Object, Properties},
};
use input::EventHandler;
use obj::{load_obj, Obj, Vertex};
use std::{
    fs::File,
    io::BufReader,
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
        .set_palette(gradient_to_palette(&plasma()));

    // Setup camera
    engine.camera().set_eye(Point3::new(-3.0, 0.5, -3f32));
    engine
        .camera()
        .set_look_at(Point3::new(0.5f32, 0.5f32, -0.5f32));

    // Create a static and axis aligned voxel object
    let mut properties = Properties::default();
    properties.set_is_static(true);
    properties.set_is_axis_aligned(true);

    // https://groups.csail.mit.edu/graphics/classes/6.837/F03/models/
    let input = BufReader::new(File::open("teapot.obj").unwrap());
    let obj: Obj<Vertex, u32> = load_obj(input).unwrap();

    let scale = 32.0;

    fn scale_vec(vector: [f32; 3], scale: f32) -> [f32; 3] {
        [vector[0] * scale, vector[1] * scale, vector[2] * scale]
    }

    let triangles = obj
        .indices
        .chunks(3)
        .map(|c| {
            [
                scale_vec(obj.vertices[c[0] as usize].position, scale),
                scale_vec(obj.vertices[c[1] as usize].position, scale),
                scale_vec(obj.vertices[c[2] as usize].position, scale),
            ]
        })
        .collect::<Vec<[[f32; 3]; 3]>>();

    println!("Loaded");

    let object = Object::voxelize_from_mesh(
        engine.device().clone(),
        Matrix4::identity(),
        properties,
        &triangles,
    );

    let mut count = 0;

    for (_, value) in object.chunks() {
        count += value.chunk().count();
    }

    println!("{} Voxels", count);

    println!("Voxelized");

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
