use crate::{
    engine::{
        physics::simulation::Simulation,
        voxel::{
            chunk::{Chunk, VOXEL_SIZE},
            object::Object,
            terrain::Terrain,
        },
    },
    game::{input::InputHandler, scene::Scene, ui::menu::pause::PauseMenu, Game},
    stats::{Ranking, Stats},
    TERRAIN_RENDER_DISTANCE,
};
use cgmath::{Matrix4, Point3, Quaternion, Vector3};
use colorgrad::Gradient;
use egui::{Align2, Area, Color32, FontFamily, Frame, RichText};
use noise::{NoiseFn, Perlin};
use rand::Rng;
use rapier3d::prelude::{ColliderBuilder, RigidBodyBuilder, RigidBodyHandle};
use std::{mem::MaybeUninit, time::Instant};
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use super::SeededLevel;

pub struct PhysicsLevel {
    terrain: Terrain,
    simulation: Simulation,
    stats: Stats,
    last: Instant,
    cubes: Vec<(Object, RigidBodyHandle)>,
    seed: u32,
}

impl SeededLevel for PhysicsLevel {
    fn with_seed(seed: u32) -> Self {
        let level = MaybeUninit::<PhysicsLevel>::uninit();

        let mut level = unsafe { level.assume_init() };

        level.seed = seed;

        level
    }
}

impl Scene for PhysicsLevel {
    fn on_current(&mut self, game: &mut Game) {
        game.set_handler(InputHandler::Game);
        game.engine().window().set_grab(true);
        game.engine().window().window().set_cursor_visible(false);
    }

    fn on_load(&mut self, game: &mut Game) {
        // Setup camera
        game.engine()
            .camera()
            .set_eye(Point3::new(-25.0, 64.0, -25.0));
        game.engine()
            .camera()
            .set_look_at(Point3::new(0.5, 64.0, -0.5));

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
                game.engine().device().clone(),
                Matrix4::from_translation(Vector3::new(i[0], i[1], i[2])),
            );

            let mut chunk = Chunk::empty();

            chunk.set(
                0,
                0,
                0,
                true,
                [
                    rng.random_range(0..=255),
                    rng.random_range(0..=255),
                    rng.random_range(0..=255),
                    255,
                ],
            );

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

        pub struct NaturalGradient {
            pub noise: Perlin,
        }

        pub fn natural() -> NaturalGradient {
            NaturalGradient {
                noise: Perlin::new(1234),
            }
        }

        impl Gradient for NaturalGradient {
            fn at(&self, t: f32) -> colorgrad::Color {
                let t = t.clamp(0.0, 1.0);
                let base_height = t * 256.0;
                let noise = self.noise.get([base_height as f64 * 0.1, 0.0]) as f32 * 4.0;

                let height = t * 256.0 + noise;

                if height <= 32.0 {
                    let water_t = height / 32.0;
                    colorgrad::Color::new(0.0, 0.2 + (water_t * 0.4), 0.5 + (water_t * 0.5), 1.0)
                } else if height <= 35.0 {
                    colorgrad::Color::new(0.94, 0.87, 0.73, 1.0)
                } else if height <= 90.0 {
                    let grass_t = (height - 32.0) / 58.0;
                    colorgrad::Color::new(0.2 + (grass_t * 0.1), 0.5 - (grass_t * 0.1), 0.1, 1.0)
                } else if height <= 140.0 {
                    let mountain_t = (height - 90.0) / 50.0;
                    colorgrad::Color::new(
                        0.5 + (mountain_t * 0.1),
                        0.4 + (mountain_t * 0.1),
                        0.3 + (mountain_t * 0.2),
                        1.0,
                    )
                } else {
                    let snow_t = (height - 140.0) / 116.0;
                    let white = 0.9 + (snow_t * 0.1);
                    colorgrad::Color::new(white, white, white + 0.05, 1.0)
                }
            }
        }

        let seed: u32 = rng.random();
        let terrain = Terrain::new(
            seed,
            TERRAIN_RENDER_DISTANCE,
            Box::new(natural()),
            game.engine().device().clone(),
        );

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

        let last = Instant::now();

        unsafe {
            std::ptr::write(
                self,
                PhysicsLevel {
                    seed,
                    terrain,
                    simulation,
                    stats,
                    last,
                    cubes,
                },
            );
        }

        game.set_handler(InputHandler::Game);
        game.engine().window().set_grab(true);
        game.engine().window().window().set_cursor_visible(false);
    }

    fn render(&mut self, game: &mut Game) {
        while let Ok(event) = game.events.try_recv() {
            if let WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } = event
            {
                if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                    if !event.state.is_pressed() {
                        game.push_scene(Box::new(PauseMenu::new()));
                    }
                    return;
                }
            }
        }

        // Handle resizes befor rendering
        game.engine().renderer().handle_resize();

        let start = Instant::now();

        let eye = game.engine().camera().get_eye();

        let frame = game.engine().start_frame();

        let mut scene_pass = frame.start_voxel_render_pass().unwrap();

        let mut ui_pass = frame.start_ui_render_pass();

        self.terrain
            .render(game.engine(), &mut scene_pass, &mut self.simulation);

        if self.last.elapsed().as_secs_f64() >= 1.0 / 60.0 {
            self.last = Instant::now();

            self.simulation.step();

            self.stats
                .push_metric("physics", self.last.elapsed().as_secs_f64() * 1000.0)
        }

        for (cube, handle) in self.cubes.iter_mut() {
            let pos = self.simulation.rigid_body_set()[*handle].position();

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
            scene_pass.render_object(cube);
        }

        ui_pass.render_ui(|ui| {
            Area::new("stats_display".into())
                .anchor(Align2::LEFT_TOP, [10.0, 10.0])
                .show(ui, |ui| {
                    Frame::new().fill(Color32::BLACK).show(ui, |ui| {
                        for line in self.stats.get() {
                            ui.label(
                                RichText::new(line)
                                    .color(Color32::WHITE)
                                    .size(12.0)
                                    .family(FontFamily::Monospace),
                            );
                        }
                    });
                });
            Area::new("coordinates_display".into())
                .anchor(Align2::RIGHT_TOP, [-10.0, 10.0])
                .show(ui, |ui| {
                    Frame::new().fill(Color32::BLACK).show(ui, |ui| {
                        let labels = ["X", "Y", "Z"];
                        let coords = [eye.x, eye.y, eye.z];

                        for (label, coord) in labels.iter().zip(coords.iter()) {
                            ui.label(
                                RichText::new(format!("{}: {:>9.2}", label, coord))
                                    .color(Color32::WHITE)
                                    .size(12.0)
                                    .family(FontFamily::Monospace),
                            );
                        }
                    });
                });
        });

        frame.finish_voxel_render_pass(scene_pass);
        frame.finish_ui_render_pass(ui_pass);

        game.engine().finish_frame(frame);

        let end = Instant::now();

        self.stats
            .push_metric("fps", 1.0 / ((end - start).as_secs_f64()));
        self.stats
            .push_metric("frame_time", (end - start).as_secs_f64() * 1000.0);
    }
}
