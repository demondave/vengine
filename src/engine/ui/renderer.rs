use crate::engine::ui::pass::Pass;
use ahash::{HashMap, HashMapExt};
use egui::{Context, ViewportId};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use std::mem;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use wgpu::{CommandEncoder, Device, Queue, StoreOp, SurfaceTexture, TextureFormat};
use winit::event::WindowEvent;
use winit::window::Window;

pub struct UiRenderer<'a> {
    state: Mutex<State>,
    renderer: Mutex<Renderer>,

    static_ui_count: AtomicU64,
    #[allow(clippy::type_complexity)]
    static_uis: Arc<Mutex<HashMap<u64, Box<dyn Fn(&Context) + Send + Sync + 'a>>>>,
}

impl<'a> UiRenderer<'a> {
    pub fn new(
        window: &Window,
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
    ) -> UiRenderer<'a> {
        let context = Context::default();
        let state = State::new(
            context.clone(),
            ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2048),
        );

        let renderer = Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_samples,
            true,
        );

        UiRenderer {
            state: Mutex::new(state),
            renderer: Mutex::new(renderer),
            static_ui_count: AtomicU64::new(0),
            static_uis: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn context(&self) -> Context {
        self.state.lock().unwrap().egui_ctx().clone()
    }

    pub fn handle_input(&self, window: &Window, event: &WindowEvent) {
        let _ = self.state.lock().unwrap().on_window_event(window, event);
    }

    pub fn start_render_pass(
        &self,
        window: &Window,
        surface_texture: &SurfaceTexture,
        device: &Device,
    ) -> Pass {
        let mut state = self.state.lock().unwrap();

        let raw_input = state.take_egui_input(window);
        state.egui_ctx().begin_pass(raw_input);

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("ui_encoder"),
        });

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            label: Some("ui render pass"),
            occlusion_query_set: None,
        });

        Pass::new(
            render_pass.forget_lifetime(),
            encoder,
            state.egui_ctx().clone(),
        )
    }

    pub fn finish_render_pass(
        &self,
        pass: Pass,
        device: &Device,
        queue: &Queue,
        screen_descriptor: &ScreenDescriptor,
    ) -> CommandEncoder {
        let state = self.state.lock().unwrap();
        let mut renderer = self.renderer.lock().unwrap();

        let (mut encoder, rpass, render_static) = pass.finish();

        // render static uis if requested via the pass
        if render_static {
            let ctx = state.egui_ctx();
            for ui in self.static_uis.lock().unwrap().values() {
                ui(ctx)
            }
        }

        let full_output = state.egui_ctx().end_pass();

        let tris = state
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            renderer.update_texture(device, queue, *id, image_delta);
        }
        renderer.update_buffers(device, queue, &mut encoder, &tris, screen_descriptor);

        renderer.render(&mut rpass.forget_lifetime(), &tris, screen_descriptor);
        for x in &full_output.textures_delta.free {
            renderer.free_texture(x)
        }

        encoder
    }

    #[must_use]
    pub fn add_static_ui<F: Fn(&Context) + Send + Sync + 'a>(&self, f: F) -> StaticUiGuard<'a> {
        let id = self.static_ui_count.fetch_add(1, Ordering::Relaxed);

        self.static_uis.lock().unwrap().insert(id, Box::new(f));

        StaticUiGuard {
            id,
            uis: self.static_uis.clone(),
        }
    }
}

/// Created by [`UiRenderer::add_static_ui`]. When dropped, the ui element will be removed.
pub struct StaticUiGuard<'a> {
    id: u64,
    #[allow(clippy::type_complexity)]
    uis: Arc<Mutex<HashMap<u64, Box<dyn Fn(&Context) + Send + Sync + 'a>>>>,
}

impl StaticUiGuard<'_> {
    /// Do not remove the ui when this [`StaticUiGuard`] is dropped.
    pub fn forget(self) {
        mem::forget(self)
    }
}

impl Drop for StaticUiGuard<'_> {
    fn drop(&mut self) {
        self.uis.lock().unwrap().remove(&self.id);
    }
}
