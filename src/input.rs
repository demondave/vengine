use cgmath::{Deg, EuclideanSpace, InnerSpace, Matrix3, Point3};
use winit::{
    event::{DeviceId, Event, KeyEvent, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::engine::Engine;

pub struct Input {
    engine: &'static Engine<'static>,
    event_loop: Option<EventLoop<()>>,
    wdw_id: WindowId,
}

impl Input {
    pub fn new(event_loop: EventLoop<()>, window: &Window, engine: &'static Engine) -> Self {
        Self {
            event_loop: Some(event_loop),
            wdw_id: window.id(),
            engine,
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
                if window_id != self.wdw_id {
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

                    WindowEvent::Resized(size) => {}
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

                let direction = (look_at - eye).normalize() * 0.025;

                self.engine.camera().set_eye_no_update(eye + direction);
                self.engine
                    .camera()
                    .set_look_at_no_update(look_at + direction);
                self.engine.camera().update();
            }
            KeyCode::KeyA => {}
            KeyCode::KeyS => {
                let eye = self.engine.camera().get_eye();
                let look_at = self.engine.camera().get_look_at();

                let direction = (eye - look_at).normalize() * 0.025;

                self.engine.camera().set_eye_no_update(eye + direction);
                self.engine
                    .camera()
                    .set_look_at_no_update(look_at + direction);
                self.engine.camera().update();
            }
            KeyCode::KeyD => {}
            // Arrow-Keys
            KeyCode::ArrowUp => {
                let rotation: Matrix3<f32> = Matrix3::from_angle_x(Deg(-1.0));

                let eye = self.engine.camera().get_eye();
                let look_at =
                    (rotation * (self.engine.camera().get_look_at() - eye)) + eye.to_vec();

                self.engine.camera().set_look_at(Point3::from_vec(look_at));
            }
            KeyCode::ArrowDown => {
                let rotation: Matrix3<f32> = Matrix3::from_angle_x(Deg(1.0));

                let eye = self.engine.camera().get_eye();
                let look_at =
                    (rotation * (self.engine.camera().get_look_at() - eye)) + eye.to_vec();

                self.engine.camera().set_look_at(Point3::from_vec(look_at));
            }
            KeyCode::ArrowLeft => {
                let rotation: Matrix3<f32> = Matrix3::from_angle_y(Deg(1.0));

                let eye = self.engine.camera().get_eye();
                let look_at =
                    (rotation * (self.engine.camera().get_look_at() - eye)) + eye.to_vec();

                self.engine.camera().set_look_at(Point3::from_vec(look_at));
            }
            KeyCode::ArrowRight => {
                let rotation: Matrix3<f32> = Matrix3::from_angle_y(Deg(-1.0));

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
