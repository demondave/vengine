use egui::{Align2, Area, Button, Color32, Frame, RichText};
use winit::window::CursorGrabMode;

use crate::{
    engine::renderer::frame::ui_pass::UiPass,
    game::{
        input::InputHandler,
        scene::Scene,
        ui::level::{physics::PhysicsLevel, procedural::ProceduralLevel},
        Game,
    },
};

use super::{credits::Credits, custom::CustomMenu, seed::SeedMenu};

#[derive(Default)]
pub struct MainMenu {}

impl MainMenu {
    pub fn new() -> Self {
        MainMenu {}
    }
}

impl Scene for MainMenu {
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
                            if ui
                                .add(Button::new(
                                    RichText::new("World Generation")
                                        .color(Color32::WHITE)
                                        .size(32.0)
                                        .italics(),
                                ))
                                .clicked()
                            {
                                game.push_scene(Box::new(SeedMenu::<ProceduralLevel>::new()));
                            }

                            ui.add_space(12.5);

                            if ui
                                .add(Button::new(
                                    RichText::new("Physics")
                                        .color(Color32::WHITE)
                                        .size(32.0)
                                        .italics(),
                                ))
                                .clicked()
                            {
                                game.push_scene(Box::new(SeedMenu::<PhysicsLevel>::new()));
                            }

                            ui.add_space(12.5);

                            if ui
                                .add(Button::new(
                                    RichText::new("Custom")
                                        .color(Color32::WHITE)
                                        .size(32.0)
                                        .italics(),
                                ))
                                .clicked()
                            {
                                game.push_scene(Box::new(CustomMenu::new()));
                            }

                            ui.add_space(12.5);

                            if ui
                                .add(Button::new(
                                    RichText::new("Credits")
                                        .color(Color32::WHITE)
                                        .size(32.0)
                                        .italics(),
                                ))
                                .clicked()
                            {
                                game.push_scene(Box::new(Credits::default()));
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
                        });
                    });
                });
        });

        frame.finish_render_pass::<UiPass>(ui_pass);

        game.engine.renderer().finish_frame(frame);
    }
}
