use crate::engine::core::window::WindowBuilder;
use crate::engine::physics::simulation::Simulation;
use crate::engine::ui::egui::EguiRenderer;
use crate::engine::voxel::chunk::VOXEL_SIZE;
use crate::engine::voxel::object::{Object, Properties};
use crate::engine::voxel::terrain::Terrain;
use cgmath::{Matrix4, Point3, Quaternion, Vector3};
use colorgrad::preset::turbo;
use egui::{Align2, Area, Color32, FontFamily, Frame, RichText};
use egui_wgpu::ScreenDescriptor;
use engine::{
    core::{engine::Engine, window::Window},
    renderer::backend::Backend,
    voxel::chunk::Chunk,
};
use input::EventHandler;
use rand::Rng;
use rapier3d::dynamics::{RigidBodyBuilder, RigidBodyHandle};
use rapier3d::geometry::ColliderBuilder;
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
    engine.camera().set_eye(Point3::new(-25.0, 64.0, -25.0));
    engine.camera().set_look_at(Point3::new(0.5, 64.0, -0.5));

    let mut simulation = Simulation::new(nalgebra::Vector3::new(0.0, -9.81, 0.0));

    // Falling cubes
    let mut positions = vec![];
    let mut rng = rand::rng();

    for _ in 0..5000 {
        let x = rng.random_range(-128..=127) as f32;
        let y = rng.random_range(64..=128) as f32;
        let z = rng.random_range(-128..=127) as f32;
        positions.push(nalgebra::Vector3::new(x, y, z));
    }

    for y in (0..64).step_by(2) {
        positions.push(nalgebra::Vector3::new(44.0, y as f32 + 55.0, 12.0));
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
            .translation(nalgebra::Vector3::new(
                i[0] + VOXEL_SIZE / 2.0,
                i[1] + VOXEL_SIZE / 2.0,
                i[2] + VOXEL_SIZE / 2.0,
            ))
            .build();

        let cube_handle = simulation.add_rigid_body(cube_rigid_body);

        let cube_collider =
            ColliderBuilder::cuboid(VOXEL_SIZE / 2.0, VOXEL_SIZE / 2.0, VOXEL_SIZE / 2.0);

        simulation.add_collider(cube_collider, Some(cube_handle));

        cubes.push((cube, cube_handle));
    }

    let mut terrain = Terrain::new(12, engine.device().clone());

    // Render object
    let mut stats = Stats::default();

    stats.add_metric(
        "physics".to_string(),
        "physics".to_string(),
        "ms".to_string(),
        Ranking::High,
    );

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

    // ui renderer
    let mut egui_renderer = EguiRenderer::new(
        engine.window().window(),
        engine.device(),
        *engine.renderer().backend().surface_format(),
        None,
        1,
    );

    let mut last = Instant::now();

    while !engine.exited() {
        let start = Instant::now();

        let output = engine
            .renderer()
            .backend()
            .surface()
            .get_current_texture()
            .unwrap();

        let mut engine_pass = engine.renderer().start_render_pass(&output).unwrap();
        let mut ui_pass =
            egui_renderer.start_render_pass(engine.window().window(), &output, engine.device());

        terrain.render(engine, &mut engine_pass, &mut simulation);

        if last.elapsed().as_secs_f64() >= 1.0 / 60.0 {
            last = Instant::now();

            simulation.step();

            stats.push_metric("physics", last.elapsed().as_secs_f64() * 1000.0)
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
            engine_pass.render_object(cube);
        }

        ui_pass.render_ui(|ui| {
            Area::new("stats_display".into())
                .anchor(Align2::LEFT_TOP, [10.0, 10.0])
                .show(ui, |ui| {
                    Frame::new().fill(Color32::BLACK).show(ui, |ui| {
                        for line in stats.get() {
                            ui.label(
                                RichText::new(line)
                                    .color(Color32::WHITE)
                                    .size(12.0)
                                    .family(FontFamily::Monospace),
                            );
                        }
                    });
                });
        });

        let engine_encoder = engine.renderer().finish_render_pass(engine_pass);
        let ui_encoder = egui_renderer.finish_render_pass(
            ui_pass,
            engine.device(),
            engine.renderer().backend().queue(),
            &ScreenDescriptor {
                size_in_pixels: [engine.window().dimension().0, engine.window().dimension().1],
                pixels_per_point: engine.window().window().scale_factor() as f32,
            },
        );

        engine
            .renderer()
            .backend()
            .queue()
            .submit([engine_encoder.finish(), ui_encoder.finish()]);

        output.present();

        let end = Instant::now();

        stats.push_metric("fps", 1.0 / ((end - start).as_secs_f64()));
        stats.push_metric("frame_time", (end - start).as_secs_f64() * 1000.0);
    }
}
