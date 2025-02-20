use egui::{Align2, Area, Button, Color32, Frame, RichText};
use winit::window::CursorGrabMode;

use crate::{
    engine::renderer::frame::ui_pass::UiPass,
    game::{input::InputHandler, scene::Scene, Game},
};

#[derive(Default)]
pub struct Credits {}

impl Scene for Credits {
    fn on_load(&mut self, game: &mut Game) {
        game.engine()
            .window()
            .window()
            .set_cursor_grab(CursorGrabMode::None)
            .unwrap();

        game.engine().window().window().set_cursor_visible(true);

        game.set_handler(InputHandler::Gui);
    }

    fn render(&mut self, game: &mut Game) {
        let frame = game.engine.renderer().start_frame();

        let mut ui_pass = frame.start_render_pass::<UiPass>();

        ui_pass.render_ui(|ctx| {
            Area::new("main_menu_area".into())
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    Frame::new().fill(Color32::BLACK).show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new("David Maul")
                                    .color(Color32::WHITE)
                                    .size(32.0)
                                    .italics(),
                            );

                            ui.add_space(12.5);

                            ui.label(
                                RichText::new("Leon Bohnwagner")
                                    .color(Color32::WHITE)
                                    .size(32.0)
                                    .italics(),
                            );

                            ui.add_space(12.5);

                            ui.label(
                                RichText::new("Ruben Otto")
                                    .color(Color32::WHITE)
                                    .size(32.0)
                                    .italics(),
                            );

                            ui.add_space(12.5);

                            ui.label(
                                RichText::new("Jonas Gärtner")
                                    .color(Color32::WHITE)
                                    .size(32.0)
                                    .italics(),
                            );

                            ui.add_space(12.5);

                            ui.label(
                                RichText::new("IN RUST WE TRUST")
                                    .color(Color32::GRAY)
                                    .size(16.0)
                                    .italics(),
                            );

                            ui.add_space(12.5);

                            if ui
                                .add(Button::new(
                                    RichText::new("Back")
                                        .color(Color32::WHITE)
                                        .size(32.0)
                                        .italics(),
                                ))
                                .clicked()
                            {
                                game.pop_scene();
                            }
                        });
                    });
                });
        });

        frame.finish_render_pass(ui_pass);

        game.engine.renderer().finish_frame(frame);
    }
}
