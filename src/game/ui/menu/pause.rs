use egui::{Align2, Area, Button, Color32, Frame, RichText};
use winit::window::CursorGrabMode;

use crate::game::{input::InputHandler, scene::Scene, Game};

#[derive(Default)]
pub struct PauseMenu {}

impl PauseMenu {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scene for PauseMenu {
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
        let frame = game.engine.start_frame();

        let mut ui_pass = frame.start_ui_render_pass();

        ui_pass.render_ui(|ctx| {
            Area::new("pause_menu_area".into())
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    Frame::new().fill(Color32::BLACK).show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            if ui
                                .add(Button::new(
                                    RichText::new("Resume")
                                        .color(Color32::WHITE)
                                        .size(32.0)
                                        .italics(),
                                ))
                                .clicked()
                            {
                                game.pop_scene();
                            }

                            ui.add_space(12.5);

                            if ui
                                .add(Button::new(
                                    RichText::new("Back to Main Menu")
                                        .color(Color32::WHITE)
                                        .size(32.0)
                                        .italics(),
                                ))
                                .clicked()
                            {
                                game.pop_scene();
                                game.pop_scene();
                            }

                            ui.add_space(12.5);

                            if ui
                                .add(Button::new(
                                    RichText::new("Exit")
                                        .color(Color32::WHITE)
                                        .size(32.0)
                                        .italics(),
                                ))
                                .clicked()
                            {
                                game.engine().exit();
                            }

                            ui.add_space(12.5);
                        });
                    });
                });
        });

        frame.finish_ui_render_pass(ui_pass);

        game.engine.finish_frame(frame);
    }
}
