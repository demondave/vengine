use crate::{
    engine::{
        renderer::frame::{ui_pass::UiPass, voxel_pass::VoxelPass},
        voxel::object::Object,
    },
    game::{input::InputHandler, scene::Scene, ui::menu::pause::PauseMenu, Game},
    stats::{Ranking, Stats},
};
use egui::{Align2, Area, Color32, FontFamily, Frame, RichText};
use std::time::Instant;
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

pub struct CustomLevel {
    object: Object,
    stats: Stats,
}

impl CustomLevel {
    pub fn new(object: Object) -> Self {
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

        Self { object, stats }
    }
}

impl Scene for CustomLevel {
    fn on_current(&mut self, game: &mut Game) {
        game.set_handler(InputHandler::Game);
        game.engine().window().set_grab(true);
        game.engine().window().window().set_cursor_visible(false);
    }

    fn on_load(&mut self, game: &mut Game) {
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

        let start = Instant::now();

        let eye = game.engine().camera().get_eye();

        let frame = game.engine().renderer().start_frame();

        let mut scene_pass = frame.start_render_pass::<VoxelPass>();

        let mut ui_pass = frame.start_render_pass::<UiPass>();

        scene_pass.render_object(&self.object);

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
