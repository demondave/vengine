use std::marker::PhantomData;

use crate::{
    engine::renderer::frame::ui_pass::UiPass,
    game::{input::InputHandler, scene::Scene, ui::level::SeededLevel, Game},
};
use egui::{Align2, Area, Button, Color32, Frame, RichText, TextEdit};
use winit::window::CursorGrabMode;

#[derive(Default)]
pub struct SeedMenu<T: Scene + SeededLevel + 'static> {
    buffer: String,
    t: PhantomData<T>,
}

impl<T: Scene + SeededLevel> SeedMenu<T> {
    pub fn new() -> Self {
        SeedMenu {
            buffer: String::new(),
            t: PhantomData,
        }
    }
}

impl<T: Scene + SeededLevel> Scene for SeedMenu<T> {
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
            Area::new("seed_menu_area".into())
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    Frame::new().fill(Color32::BLACK).show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add(TextEdit::singleline(&mut self.buffer).hint_text("Seed"));

                            ui.add_space(12.5);

                            ui.horizontal_centered(|ui| {
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

                                ui.add_space(25.0);

                                if ui
                                    .add(Button::new(
                                        RichText::new("Generate")
                                            .color(Color32::WHITE)
                                            .size(32.0)
                                            .italics(),
                                    ))
                                    .clicked()
                                {
                                    if let Ok(seed) = self.buffer.parse::<u32>() {
                                        game.pop_scene();
                                        game.push_scene(Box::new(T::with_seed(seed)))
                                    }
                                }
                            });
                        });
                    });
                });
        });

        frame.finish_render_pass::<UiPass>(ui_pass);

        game.engine.renderer().finish_frame(frame);
    }
}
