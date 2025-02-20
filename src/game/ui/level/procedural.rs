use crate::{
    engine::{
        physics::simulation::Simulation,
        renderer::frame::{ui_pass::UiPass, voxel_pass::VoxelPass},
        voxel::terrain::Terrain,
    },
    game::{input::InputHandler, scene::Scene, ui::menu::pause::PauseMenu, Game},
    stats::{Ranking, Stats},
    TERRAIN_RENDER_DISTANCE,
};
use cgmath::Point3;
use colorgrad::Gradient;
use egui::{Align2, Area, Color32, FontFamily, Frame, RichText};
use noise::{NoiseFn, Perlin};
use std::{mem::MaybeUninit, time::Instant};
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use super::SeededLevel;

pub struct ProceduralLevel {
    terrain: Terrain,
    simulation: Simulation,
    stats: Stats,
    last: Instant,
    seed: u32,
}

impl SeededLevel for ProceduralLevel {
    fn with_seed(seed: u32) -> Self {
        let level = MaybeUninit::<ProceduralLevel>::uninit();

        let mut level = unsafe { level.assume_init() };

        level.seed = seed;

        level
    }
}

impl Scene for ProceduralLevel {
    fn on_current(&mut self, game: &mut Game) {
        game.set_handler(InputHandler::Game);
        game.engine().window().set_grab(true);
        game.engine().window().window().set_cursor_visible(false);
    }

    fn on_load(&mut self, game: &mut Game) {
        let seed = self.seed;

        // Setup camera
        game.engine()
            .camera()
            .set_eye(Point3::new(-25.0, 64.0, -25.0));
        game.engine()
            .camera()
            .set_look_at(Point3::new(0.5, 64.0, -0.5));

        let simulation = Simulation::new(nalgebra::Vector3::new(0.0, -9.81, 0.0));

        pub struct NaturalGradient {
            pub noise: Perlin,
        }

        pub fn natural(seed: u32) -> NaturalGradient {
            NaturalGradient {
                noise: Perlin::new(seed),
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

        let terrain = Terrain::new(
            seed,
            TERRAIN_RENDER_DISTANCE,
            Box::new(natural(seed)),
            game.engine(),
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
                ProceduralLevel {
                    seed,
                    terrain,
                    simulation,
                    stats,
                    last,
                },
            );
        }

        game.set_handler(InputHandler::Game);
        game.engine().window().set_grab(true);
        game.engine().window().window().set_cursor_visible(false);
    }

    fn render(&mut self, game: &mut Game) {
        // Handle events
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

        let start = Instant::now();

        let eye = game.engine().camera().get_eye();

        let frame = game.engine().renderer().start_frame();

        let mut scene_pass = frame.start_render_pass::<VoxelPass>();

        let mut ui_pass = frame.start_render_pass::<UiPass>();

        self.terrain
            .render(game.engine(), &mut scene_pass, &mut self.simulation);

        if self.last.elapsed().as_secs_f64() >= 1.0 / 60.0 {
            self.last = Instant::now();

            self.simulation.step();

            self.stats
                .push_metric("physics", self.last.elapsed().as_secs_f64() * 1000.0)
        }

        ui_pass.render_ui(|ctx| {
            Area::new("stats_display".into())
                .anchor(Align2::LEFT_TOP, [10.0, 10.0])
                .show(ctx, |ui| {
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
                .show(ctx, |ui| {
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

        frame.finish_render_pass(scene_pass);
        frame.finish_render_pass(ui_pass);

        game.engine().renderer().finish_frame(frame);

        let end = Instant::now();

        self.stats
            .push_metric("fps", 1.0 / ((end - start).as_secs_f64()));
        self.stats
            .push_metric("frame_time", (end - start).as_secs_f64() * 1000.0);
    }
}
