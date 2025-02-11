use crate::{
    engine::voxel::object::Object,
    game::{input::InputHandler, scene::Scene, ui::level::custom::CustomLevel, Game},
    io::load_voxels,
};
use cgmath::{Matrix4, SquareMatrix};
use egui::{Align2, Area, Button, Color32, Frame, RichText, TextEdit};
use std::{path::PathBuf, str::FromStr};
use winit::window::CursorGrabMode;

#[derive(Default)]
pub struct CustomMenu {
    buffer: String,
}

impl CustomMenu {
    pub fn new() -> Self {
        CustomMenu {
            buffer: String::new(),
        }
    }
}

impl Scene for CustomMenu {
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

        ui_pass.render_ui(|ctx| {
            Area::new("seed_menu_area".into())
                .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    Frame::new().fill(Color32::BLACK).show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add(TextEdit::singleline(&mut self.buffer).hint_text("Path"));

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
                                        RichText::new("Load")
                                            .color(Color32::WHITE)
                                            .size(32.0)
                                            .italics(),
                                    ))
                                    .clicked()
                                {
                                    let path = PathBuf::from_str(&self.buffer).unwrap();
                                    if path.exists() {
                                        let voxels = load_voxels(path);
                                        let object = Object::from_voxels(
                                            game.engine().device().clone(),
                                            Matrix4::identity(),
                                            voxels,
                                        );

                                        game.pop_scene();
                                        game.push_scene(Box::new(CustomLevel::new(object)));
                                    } else {
                                        println!("'{}' doesn't exist", path.display());
                                    }
                                }
                            });
                        });
                    });
                });
        });

        frame.finish_ui_render_pass(ui_pass);

        game.engine.finish_frame(frame);
    }
}
