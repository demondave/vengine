use crate::engine::physics::simulation::Simulation;
use crate::engine::voxel::object::{Object, Properties};
use cgmath::{Matrix4, Point3, Quaternion, SquareMatrix, Vector3};
use colorgrad::preset::turbo;
use engine::{
    core::{engine::Engine, window::Window},
    renderer::backend::Backend,
    voxel::chunk::{Chunk, CHUNK_SIZE},
};
use input::EventHandler;
use rand::Rng;
use rapier3d::dynamics::{RigidBodyBuilder, RigidBodyHandle};
use rapier3d::geometry::ColliderBuilder;
use stats::{Ranking, Stats};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use util::gradient_to_palette;

pub mod engine;
pub mod input;
pub mod stats;
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
    engine.camera().set_eye(Point3::new(-25.0, 20.0, -25.0));
    engine.camera().set_look_at(Point3::new(0.5, 20.0, -0.5));

    let mut rng = rand::rng();

    let mut simulation = Simulation::new(nalgebra::Vector3::new(0.0, -9.81, 0.0));

    let mut base = Object::new(
        engine.device().clone(),
        Matrix4::identity(),
        Properties::default(),
    );

    let mut chunk = Chunk::empty();

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            chunk.set(x, 0, z, true, 30);
        }
    }

    base.add_chunk(Vector3::new(0, 0, 0), chunk, true);

    let base_rigid_body = RigidBodyBuilder::fixed()
        .translation(nalgebra::Vector3::new(
            CHUNK_SIZE as f32 / 2.0,
            0.0,
            CHUNK_SIZE as f32 / 2.0,
        ))
        .build();

    let base_handle = simulation.add_rigid_body(base_rigid_body);

    let base_collider =
        ColliderBuilder::cuboid(CHUNK_SIZE as f32 / 2.0, 0.5, CHUNK_SIZE as f32 / 2.0);

    simulation.add_collider(base_collider, Some(base_handle));

    /*
    let positions = vec![
        vec![0.0, 32.0, 0.0],
        vec![-1.0, 16.0, -1.0],
        vec![CHUNK_SIZE as f32 - 1.0, 32.0, 0.0],
        vec![CHUNK_SIZE as f32, 16.0, -1.0],
        vec![0.0, 32.0, CHUNK_SIZE as f32 - 1.0],
        vec![-1.0, 16.0, CHUNK_SIZE as f32 ],
        vec![CHUNK_SIZE as f32 - 1.0, 32.0, CHUNK_SIZE as f32 - 1.0],
        vec![CHUNK_SIZE as f32, 16.0, CHUNK_SIZE as f32 ],
    ];
    */

    let mut positions = vec![];

    for _ in 0..1000 {
        let x = rng.random_range(0..=31) as f32;
        let y = rng.random_range(32..=128) as f32;
        let z = rng.random_range(0..=31) as f32;
        positions.push(nalgebra::Vector3::new(x, y, z));
    }

    let mut cubes: Vec<(Object, RigidBodyHandle)> = vec![];

    for i in positions {
        let mut cube = Object::new(
            engine.device().clone(),
            Matrix4::from_translation(Vector3::new(i[0], i[1], i[2])),
            Properties::default(),
        );

        let mut chunk = Chunk::empty();

        chunk.set(0, 0, 0, true, rng.random_range(0..=127));

        cube.add_chunk(Vector3::new(0, 0, 0), chunk, true);

        let cube_rigid_body = RigidBodyBuilder::dynamic()
            .translation(nalgebra::Vector3::new(i[0], i[1], i[2]))
            .build();

        let cube_handle = simulation.add_rigid_body(cube_rigid_body);

        let cube_collider = ColliderBuilder::cuboid(0.5, 0.5, 0.5);

        simulation.add_collider(cube_collider, Some(cube_handle));

        cubes.push((cube, cube_handle));
    }

    //let mut terrain = Terrain::new(12, engine.device().clone());

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

    let mut last = Instant::now();

    while !engine.exited() {
        let start = Instant::now();

        let mut pass = engine.renderer().start_render_pass().unwrap();

        //terrain.render(engine, &mut pass);

        pass.render_object(&base);

        if last.elapsed().as_secs_f64() >= 1.0 / 60.0 {
            last = Instant::now();

            simulation.step();
        }

        for (cube, handle) in cubes.iter_mut() {
            let pos = simulation.rigid_body_set()[*handle].position();

            let transform = Matrix4::from_translation(Vector3::new(
                pos.translation.x,
                pos.translation.y,
                pos.translation.z,
            )) * Matrix4::from(Quaternion::new(
                pos.rotation.w,
                pos.rotation.i,
                pos.rotation.j,
                pos.rotation.k,
            ));

            cube.set_transform(transform);
            pass.render_object(cube);
        }

        engine.renderer().finish_render_pass(pass);

        let end = Instant::now();

        stats.push_metric("fps", 1.0 / ((end - start).as_secs_f64()));
        stats.push_metric("frame_time", (end - start).as_secs_f64() * 1000.0);

        stats.print();
    }
}
