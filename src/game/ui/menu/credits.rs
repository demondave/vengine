use egui::{Align2, Area, Button, Color32, Frame, RichText};
use winit::window::CursorGrabMode;

use crate::game::{input::InputHandler, scene::Scene, Game};

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
        game.engine.renderer().handle_resize();

        let frame = game.engine.start_frame();

        let mut ui_pass = frame.start_ui_render_pass();

        ui_pass.render_ui(|ui| {
            Area::new("main_menu_area".into())
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui, |ui| {
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

        frame.finish_ui_render_pass(ui_pass);

        game.engine.finish_frame(frame);
    }
}
