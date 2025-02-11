use std::collections::VecDeque;

use crossbeam::{atomic::AtomicCell, channel::Receiver};
use input::InputHandler;
use scene::Scene;
use winit::event::WindowEvent;

use crate::engine::core::engine::Engine;

pub mod input;
pub mod scene;
pub mod ui;

enum Change {
    Push(Box<dyn Scene>),
    Pop,
}

pub struct Game {
    engine: &'static Engine<'static>,
    scenes: Vec<Box<dyn Scene>>,
    handler: &'static AtomicCell<InputHandler>,
    changes: VecDeque<Change>,
    events: Receiver<WindowEvent>,
}

impl Game {
    pub fn new(
        engine: &'static Engine<'static>,
        handler: &'static AtomicCell<InputHandler>,
        mut scene: Box<dyn Scene>,
        events: Receiver<WindowEvent>,
    ) -> Self {
        let mut game = Self {
            engine,
            scenes: Vec::with_capacity(16),
            handler,
            changes: VecDeque::with_capacity(16),
            events,
        };

        scene.on_load(&mut game);

        game.scenes.push(scene);

        game
    }

    pub fn engine(&self) -> &Engine {
        self.engine
    }

    pub fn exited(&self) -> bool {
        self.engine.exited()
    }

    pub fn push_scene(&mut self, scene: Box<dyn Scene>) {
        self.changes.push_back(Change::Push(scene));
    }

    pub fn pop_scene(&mut self) {
        self.changes.push_back(Change::Pop);
    }

    pub fn render(&mut self) {
        if let Some(mut scene) = self.scenes.pop() {
            scene.render(self);

            self.scenes.push(scene);
        }

        let has_changes = !self.changes.is_empty();

        while let Some(scene) = self.changes.pop_front() {
            match scene {
                Change::Push(mut scene) => {
                    scene.on_load(self);
                    self.scenes.push(scene);
                }
                Change::Pop => {
                    if let Some(mut scene) = self.scenes.pop() {
                        scene.on_unload(self);
                    }
                }
            };

            // Clear events on scene change
            while self.events.try_recv().is_ok() {}
        }

        if has_changes {
            if let Some(mut scene) = self.scenes.pop() {
                scene.on_current(self);

                self.scenes.push(scene);
            }
        }
    }

    pub fn set_handler(&self, handler: InputHandler) {
        self.handler.store(handler);
    }
}
