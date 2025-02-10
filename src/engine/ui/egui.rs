use crate::engine::ui::pass::Pass;
use egui::{Context, ViewportId};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, StoreOp, SurfaceTexture, TextureFormat};
use winit::window::Window;

pub struct EguiRenderer {
    state: State,

    renderer: Renderer,
}

impl EguiRenderer {
    pub fn new(
        window: &Window,
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
    ) -> EguiRenderer {
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

        EguiRenderer { state, renderer }
    }

    pub fn start_render_pass(
        &mut self,
        window: &Window,
        surface_texture: &SurfaceTexture,
        device: &Device,
    ) -> Pass {
        let raw_input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(raw_input);

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
            self.state.egui_ctx().clone(),
        )
    }

    pub fn finish_render_pass(
        &mut self,
        pass: Pass,
        device: &Device,
        queue: &Queue,
        screen_descriptor: &ScreenDescriptor,
    ) -> CommandEncoder {
        let (mut encoder, rpass) = pass.finish();

        let full_output = self.state.egui_ctx().end_pass();

        let tris = self
            .state
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(device, queue, *id, image_delta);
        }
        self.renderer
            .update_buffers(device, queue, &mut encoder, &tris, screen_descriptor);

        self.renderer
            .render(&mut rpass.forget_lifetime(), &tris, screen_descriptor);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }

        encoder
    }
}
