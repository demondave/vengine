use core::f32;
use std::{
    collections::HashMap,
    thread::sleep,
    time::{Duration, Instant},
};

use crate::engine::core::engine::Engine;
use cgmath::{InnerSpace, Vector2, Vector3, Zero};
use winit::{
    event::{Event, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

const MOVEMENT_SPEED: f32 = 0.05;
const X_SENSITIVITY: f32 = -0.00075;
const Y_SENSITIVITY: f32 = 0.00075;
const TICKS: f64 = 64.0;

pub struct EventHandler {
    engine: &'static Engine<'static>,
    cursor_position: Vector2<f32>,
    keymap: HashMap<KeyCode, bool>,
}

impl EventHandler {
    pub fn new(engine: &'static Engine) -> Self {
        let keys = &[
            KeyCode::KeyW,
            KeyCode::KeyA,
            KeyCode::KeyS,
            KeyCode::KeyD,
            KeyCode::Space,
            KeyCode::ShiftLeft,
        ];

        Self {
            engine,
            cursor_position: Vector2::new(f32::NAN, f32::NAN),
            keymap: HashMap::from_iter(keys.iter().map(|k| (*k, false))),
        }
    }

    pub fn handle(&mut self) {
        let events = self.engine.events();

        let id = self.engine.window().id();

        let duration = Duration::from_secs_f64(1.0 / TICKS);

        loop {
            let start = Instant::now();

            if self.engine.exited() {
                break;
            }

            while let Ok(event) = events.try_recv() {
                if let Event::WindowEvent { window_id, event } = event {
                    if window_id == id {
                        self.handle_window_event(event);
                    }
                }
            }

            let mut offset: Vector3<f32> = Vector3::zero();

            if *self.keymap.get(&KeyCode::KeyW).unwrap() {
                offset.z += 1.0;
            }

            if *self.keymap.get(&KeyCode::KeyS).unwrap() {
                offset.z -= 1.0;
            }

            if *self.keymap.get(&KeyCode::KeyA).unwrap() {
                offset.x -= 1.0;
            }

            if *self.keymap.get(&KeyCode::KeyD).unwrap() {
                offset.x += 1.0;
            }

            if *self.keymap.get(&KeyCode::Space).unwrap() {
                offset.y += 1.0;
            }

            if *self.keymap.get(&KeyCode::ShiftLeft).unwrap() {
                offset.y -= 1.0;
            }

            let eye = self.engine.camera().get_eye();
            let look_at = self.engine.camera().get_look_at();

            let direction = (look_at - eye).normalize();

            let right = direction.cross(self.engine.camera().up()).normalize();

            self.engine.camera().set_eye_no_update(
                eye + (direction * offset.z * MOVEMENT_SPEED)
                    + (right * offset.x * MOVEMENT_SPEED)
                    + (self.engine.camera().up() * offset.y * MOVEMENT_SPEED),
            );
            self.engine.camera().set_look_at_no_update(
                look_at
                    + (direction * offset.z * MOVEMENT_SPEED)
                    + (right * offset.x * MOVEMENT_SPEED)
                    + (self.engine.camera().up() * offset.y * MOVEMENT_SPEED),
            );
            self.engine.camera().update();

            let elapsed = start.elapsed();

            if elapsed < duration {
                sleep(duration - elapsed);
            }
        }
    }

    pub fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.engine.exit();
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                let new = Vector2::new(position.x as f32, position.y as f32);

                if self.cursor_position.x.is_nan() || self.cursor_position.y.is_nan() {
                    self.cursor_position = new;
                }

                let diff = self.cursor_position - new;

                self.engine.camera().add_yaw(diff.x * X_SENSITIVITY);

                self.engine.camera().add_pitch(diff.y * Y_SENSITIVITY);

                self.engine.camera().update_target();
                self.engine.camera().update();

                self.cursor_position = new;
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if let Some(state) = self.keymap.get_mut(&code) {
                        *state = event.state.is_pressed();
                    }
                }
            }

            WindowEvent::Resized(_size) => {
                // TODO
                // self.engine.renderer().resize(size.width, size.height);
            }

            _ => {}
        }
    }
}
