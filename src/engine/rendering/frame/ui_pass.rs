use super::{pass::RenderPass, Frame};
use crate::engine::rendering::{
    configuration::Configuration,
    pipeline::{ui::UiPipeline, GetPipeline},
};
use egui::Context;
use egui_wgpu::ScreenDescriptor;
use wgpu::{CommandEncoder, StoreOp};

pub struct UiPass {
    encoder: CommandEncoder,
    pass: wgpu::RenderPass<'static>,
    context: Context,
}

impl RenderPass for UiPass {
    type RequiredPipeline = UiPipeline;

    fn start<C: Configuration + GetPipeline<UiPipeline>>(frame: &Frame<C>) -> Self {
        let mut ui_state = frame.renderer().configuration().get_pipeline().state();
        let raw_input = ui_state.take_egui_input(frame.renderer().window().window());
        ui_state.egui_ctx().begin_pass(raw_input);

        let view = frame
            .output()
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = frame.renderer().backend().device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("vengine::render_ui_encoder"),
            },
        );

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            label: None,
            occlusion_query_set: None,
        });

        pass.set_viewport(
            0.0,
            0.0,
            frame.size().width as f32,
            frame.size().height as f32,
            0.0,
            1.0,
        );

        UiPass {
            pass: pass.forget_lifetime(),
            encoder,
            context: frame
                .renderer()
                .configuration()
                .get_pipeline()
                .context()
                .clone(),
        }
    }

    fn finish<C: Configuration + GetPipeline<UiPipeline>>(mut self, frame: &Frame<C>) {
        let mut ui_renderer = frame.renderer().configuration().get_pipeline().renderer();

        let full_output = frame
            .renderer()
            .configuration()
            .get_pipeline()
            .state()
            .egui_ctx()
            .end_pass();

        let tris = frame
            .renderer()
            .configuration()
            .get_pipeline()
            .state()
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            ui_renderer.update_texture(
                frame.renderer().backend().device(),
                frame.renderer().backend().queue(),
                *id,
                image_delta,
            );
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [frame.size().width, frame.size().height],
            pixels_per_point: frame.size().pixels_per_point,
        };

        ui_renderer.update_buffers(
            frame.renderer().backend().device(),
            frame.renderer().backend().queue(),
            &mut self.encoder,
            &tris,
            &screen_descriptor,
        );

        ui_renderer.render(&mut self.pass.forget_lifetime(), &tris, &screen_descriptor);
        for x in &full_output.textures_delta.free {
            ui_renderer.free_texture(x)
        }

        frame.push_encoder(self.encoder);
    }
}

impl UiPass {
    pub fn render_ui<F: FnOnce(&Context)>(&mut self, f: F) {
        f(&self.context);
    }
}
