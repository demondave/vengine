pub mod performance;
pub mod rendering;

use std::sync::{Mutex, MutexGuard};

use egui::{Context, ViewportId};
use egui_winit::State;
use winit::event::WindowEvent;

use crate::engine::{
    core::window::window::Window,
    rendering::{backend::Backend, camera::Camera},
};

use super::rendering::pipeline::Pipeline;

pub struct UiPipeline {
    state: Mutex<State>,
    context: Context,
    rendererer: Mutex<egui_wgpu::Renderer>,
}

impl UiPipeline {
    pub fn state(&self) -> MutexGuard<State> {
        self.state.lock().unwrap()
    }

    pub fn renderer(&self) -> MutexGuard<egui_wgpu::Renderer> {
        self.rendererer.lock().unwrap()
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn handle_window_event(&self, window: &Window, event: &WindowEvent) {
        let _ = self.state().on_window_event(window.window(), event);
    }
}

impl Pipeline for UiPipeline {
    fn initialize(backend: &Backend<'_>, _camera: &Camera) -> Self {
        let context = Context::default();
        let state = State::new(
            context.clone(),
            ViewportId::ROOT,
            backend.window().window(),
            Some(backend.window().window().scale_factor() as f32),
            None,
            Some(2048),
        );

        let rendererer =
            egui_wgpu::Renderer::new(backend.device(), *backend.surface_format(), None, 1, true);

        UiPipeline {
            state: Mutex::new(state),
            context,
            rendererer: Mutex::new(rendererer),
        }
    }
}
