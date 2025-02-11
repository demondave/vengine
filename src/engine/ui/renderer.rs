use crate::engine::renderer::backend::Backend;
use egui::{Context, ViewportId};
use egui_wgpu::Renderer;
use egui_winit::State;
use std::sync::{Mutex, MutexGuard};
use winit::window::Window;

pub struct UiRenderer {
    state: Mutex<State>,
    context: Context,
    renderer: Mutex<Renderer>,
}

impl UiRenderer {
    pub fn new(window: &Window, backend: &Backend, msaa_samples: u32) -> Self {
        let context = Context::default();
        let state = State::new(
            context.clone(),
            ViewportId::ROOT,
            window,
            Some(window.scale_factor() as f32),
            None,
            Some(2048),
        );

        let renderer = Renderer::new(
            backend.device(),
            *backend.surface_format(),
            None,
            msaa_samples,
            true,
        );

        UiRenderer {
            state: Mutex::new(state),
            context,
            renderer: Mutex::new(renderer),
        }
    }

    pub fn state(&self) -> MutexGuard<State> {
        self.state.lock().unwrap()
    }

    pub fn renderer(&self) -> MutexGuard<Renderer> {
        self.renderer.lock().unwrap()
    }

    pub fn context(&self) -> &Context {
        &self.context
    }
}
