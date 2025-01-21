use core::f32;
use std::{
    collections::HashMap,
    thread::sleep,
    time::{Duration, Instant},
};

use crate::engine::core::engine::Engine;
use cgmath::{Deg, InnerSpace, Matrix3, Vector3, Zero};
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

const MOVEMENT_SPEED: f32 = 0.05;
const MOVEMENT_CONTROL_MULTIPLIER: f32 = 4.0;
const X_SENSITIVITY: f32 = -0.01;
const Y_SENSITIVITY: f32 = -0.01;
const TICKS: f64 = 64.0;

pub struct EventHandler {
    engine: &'static Engine<'static>,
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
            KeyCode::ControlLeft,
        ];

        Self {
            engine,
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
                match event {
                    Event::WindowEvent { window_id, event } => {
                        if window_id == id {
                            self.handle_window_event(event);
                        }
                    }
                    Event::DeviceEvent {
                        device_id: _,
                        event,
                    } => {
                        self.handle_device_event(event);
                    }
                    _ => {}
                }
            }

            let mut offset: Vector3<f32> = Vector3::zero();

            let multiplier = match self.keymap.get(&KeyCode::ControlLeft).unwrap() {
                true => MOVEMENT_CONTROL_MULTIPLIER,
                false => 1.0,
            };

            if *self.keymap.get(&KeyCode::KeyW).unwrap() {
                offset.z += 1.0 * multiplier;
            }

            if *self.keymap.get(&KeyCode::KeyS).unwrap() {
                offset.z -= 1.0 * multiplier;
            }

            if *self.keymap.get(&KeyCode::KeyA).unwrap() {
                offset.x -= 1.0 * multiplier;
            }

            if *self.keymap.get(&KeyCode::KeyD).unwrap() {
                offset.x += 1.0 * multiplier;
            }

            if *self.keymap.get(&KeyCode::Space).unwrap() {
                offset.y += 1.0 * multiplier;
            }

            if *self.keymap.get(&KeyCode::ShiftLeft).unwrap() {
                offset.y -= 1.0 * multiplier;
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

    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion {
            delta: (delta_x, delta_y),
        } = event
        {
            let eye = self.engine.camera().get_eye();
            let look_at = self.engine.camera().get_look_at();

            let mut relative = look_at - eye;

            let rotation = Matrix3::from_angle_y(Deg(delta_x as f32 * X_SENSITIVITY));

            relative = rotation * relative;

            self.engine.camera().set_look_at_no_update(eye + relative);

            let eye = self.engine.camera().get_eye();
            let look_at = self.engine.camera().get_look_at();

            let mut relative = (look_at - eye).normalize();

            let right = relative.cross(Vector3::unit_y()).normalize();

            let rotation = Matrix3::from_axis_angle(right, Deg(delta_y as f32 * Y_SENSITIVITY));

            relative = rotation * relative;

            self.engine.camera().set_look_at_no_update(eye + relative);

            self.engine.camera().update();
        };
    }

    pub fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.engine.exit();
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
