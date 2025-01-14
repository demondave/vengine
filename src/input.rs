use std::sync::atomic::{AtomicBool, Ordering};

use ahash::AHashMap;
use cgmath::{Deg, EuclideanSpace, InnerSpace, Matrix3, Point3, Vector3};
use winit::{
    event::{DeviceId, Event, KeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::engine::Engine;

const MOVEMENT_SPEED: f32 = 0.05;
const ROTATION_SPEED: f32 = 1.0;

pub struct Input {
    engine: &'static Engine<'static>,
    event_loop: Option<EventLoop<()>>,
    window_id: WindowId,
    keys: AHashMap<KeyCode, AtomicBool>,
}

impl Input {
    pub fn new(event_loop: EventLoop<()>, window: &Window, engine: &'static Engine) -> Self {
        Self {
            event_loop: Some(event_loop),
            window_id: window.id(),
            engine,
            keys: AHashMap::new(),
        }
    }

    pub fn listen(&mut self, keys: &[KeyCode]) {
        for key in keys {
            self.keys.insert(*key, AtomicBool::new(false));
        }
    }

    pub fn is_pressed(&self, key: &KeyCode) -> bool {
        match self.keys.get(key) {
            Some(s) => s.load(Ordering::Relaxed),
            None => {
                panic!("key wasn't registered")
            }
        }
    }

    pub fn run(mut self) {
        let event_loop = self.event_loop.take().unwrap();

        event_loop
            .run(move |event, control_flow| {
                self.handle(event, control_flow);
            })
            .unwrap();
    }

    fn handle(&mut self, event: Event<()>, control_flow: &EventLoopWindowTarget<()>) {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                if window_id != self.window_id {
                    return;
                }

                match event {
                    WindowEvent::CloseRequested => {
                        control_flow.exit();
                    }
                    WindowEvent::KeyboardInput {
                        device_id,
                        event,
                        is_synthetic,
                    } => {
                        self.handle_keyboard_event(device_id, event, is_synthetic);
                    }

                    WindowEvent::Resized(size) => {
                        //self.engine.resize(size.width, size.height);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn handle_cursor_moved(&mut self, device_id: &DeviceId, event: &KeyEvent, is_synthetic: &bool) {
    }

    fn handle_keyboard_event(
        &mut self,
        device_id: &DeviceId,
        event: &KeyEvent,
        is_synthetic: &bool,
    ) {
        let key = match event.physical_key {
            PhysicalKey::Code(c) => c,
            PhysicalKey::Unidentified(_) => {
                return;
            }
        };

        let state = event.state.is_pressed();

        match key {
            // WASD
            KeyCode::KeyW => {
                let eye = self.engine.camera().get_eye();
                let look_at = self.engine.camera().get_look_at();

                let direction = (look_at - eye).normalize() * MOVEMENT_SPEED;

                self.engine.camera().set_eye_no_update(eye + direction);
                self.engine
                    .camera()
                    .set_look_at_no_update(look_at + direction);
                self.engine.camera().update();
            }
            KeyCode::KeyA => {
                let eye = self.engine.camera().get_eye();
                let look_at = self.engine.camera().get_look_at();

                let view_dir = (look_at - eye).normalize();

                let right = view_dir.cross(self.engine.camera().up()).normalize() * MOVEMENT_SPEED;

                self.engine.camera().set_eye_no_update(eye - right);
                self.engine.camera().set_look_at_no_update(look_at - right);
                self.engine.camera().update();
            }
            KeyCode::KeyS => {
                let eye = self.engine.camera().get_eye();
                let look_at = self.engine.camera().get_look_at();

                let direction = (eye - look_at).normalize() * MOVEMENT_SPEED;

                self.engine.camera().set_eye_no_update(eye + direction);
                self.engine
                    .camera()
                    .set_look_at_no_update(look_at + direction);
                self.engine.camera().update();
            }
            KeyCode::KeyD => {
                let eye = self.engine.camera().get_eye();
                let look_at = self.engine.camera().get_look_at();

                let view_dir = (look_at - eye).normalize();

                let right = view_dir.cross(self.engine.camera().up()).normalize() * -MOVEMENT_SPEED;

                self.engine.camera().set_eye_no_update(eye - right);
                self.engine.camera().set_look_at_no_update(look_at - right);
                self.engine.camera().update();
            }
            // Space and Shift
            KeyCode::Space => {
                let eye = self.engine.camera().get_eye();
                let look_at = self.engine.camera().get_look_at();
                self.engine
                    .camera()
                    .set_eye_no_update(eye + Vector3::unit_y() * MOVEMENT_SPEED);
                self.engine
                    .camera()
                    .set_look_at_no_update(look_at + Vector3::unit_y() * MOVEMENT_SPEED);
                self.engine.camera().update();
            }
            KeyCode::ShiftLeft => {
                let eye = self.engine.camera().get_eye();
                let look_at = self.engine.camera().get_look_at();
                self.engine
                    .camera()
                    .set_eye_no_update(eye + Vector3::unit_y() * -MOVEMENT_SPEED);
                self.engine
                    .camera()
                    .set_look_at_no_update(look_at + Vector3::unit_y() * -MOVEMENT_SPEED);
                self.engine.camera().update();
            }
            // Arrow-Keys
            KeyCode::ArrowUp => {
                let eye = self.engine.camera().get_eye();
                let look_at = self.engine.camera().get_look_at();

                let view_dir = (look_at - eye).normalize();

                let right = view_dir.cross(self.engine.camera().up()).normalize();

                let rotation = Matrix3::from_axis_angle(right, Deg(ROTATION_SPEED));

                let relative = look_at - eye;

                let look_at = (rotation * relative) + eye.to_vec();

                self.engine.camera().set_look_at(Point3::from_vec(look_at));
            }
            KeyCode::ArrowDown => {
                let eye = self.engine.camera().get_eye();
                let look_at = self.engine.camera().get_look_at();

                let view_dir = (look_at - eye).normalize();

                let right = view_dir.cross(self.engine.camera().up()).normalize();

                let rotation = Matrix3::from_axis_angle(right, Deg(-ROTATION_SPEED));

                let relative = look_at - eye;

                let look_at = (rotation * relative) + eye.to_vec();

                self.engine.camera().set_look_at(Point3::from_vec(look_at));
            }
            KeyCode::ArrowLeft => {
                let rotation: Matrix3<f32> = Matrix3::from_angle_y(Deg(ROTATION_SPEED));

                let eye = self.engine.camera().get_eye();
                let look_at =
                    (rotation * (self.engine.camera().get_look_at() - eye)) + eye.to_vec();

                self.engine.camera().set_look_at(Point3::from_vec(look_at));
            }
            KeyCode::ArrowRight => {
                let rotation: Matrix3<f32> = Matrix3::from_angle_y(Deg(-ROTATION_SPEED));

                let eye = self.engine.camera().get_eye();
                let look_at =
                    (rotation * (self.engine.camera().get_look_at() - eye)) + eye.to_vec();

                self.engine.camera().set_look_at(Point3::from_vec(look_at));
            }
            _ => {
                return;
            }
        }
    }
}
