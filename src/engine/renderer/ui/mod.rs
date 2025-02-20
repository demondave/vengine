use egui::{Context, ViewportId};
use egui_winit::State;
use std::sync::{Mutex, MutexGuard};
use winit::event::WindowEvent;

use crate::engine::core::window::window::Window;

use super::backend::Backend;

pub struct UiRenderer {
    state: Mutex<State>,
    context: Context,
    rendererer: Mutex<egui_wgpu::Renderer>,
}

impl UiRenderer {
    pub fn new(window: &Window, backend: &Backend, msaa_samples: u32) -> Self {
        let context = Context::default();
        let state = State::new(
            context.clone(),
            ViewportId::ROOT,
            window.window(),
            Some(window.window().scale_factor() as f32),
            None,
            Some(2048),
        );

        let rendererer = egui_wgpu::Renderer::new(
            backend.device(),
            *backend.surface_format(),
            None,
            msaa_samples,
            true,
        );

        UiRenderer {
            state: Mutex::new(state),
            context,
            rendererer: Mutex::new(rendererer),
        }
    }

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
