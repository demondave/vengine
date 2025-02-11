use crate::engine::core::engine::Engine;
use egui_wgpu::ScreenDescriptor;
use std::sync::Mutex;
use ui_pass::UiPass;
use voxel_pass::VoxelPass;
use wgpu::{CommandBuffer, CommandEncoder, StoreOp, SurfaceTexture};

pub mod ui_pass;
pub mod voxel_pass;

pub struct Frame<'a> {
    engine: &'a Engine<'a>,
    output: SurfaceTexture,
    encoders: Mutex<Vec<CommandEncoder>>,
    dimensions: (u32, u32),
}

impl<'a> Frame<'a> {
    pub fn new(engine: &'a Engine, output: SurfaceTexture) -> Self {
        let dimensions = engine.renderer().dimensions();

        Self {
            engine,
            output,
            encoders: Mutex::new(Vec::with_capacity(32)),
            dimensions,
        }
    }

    pub fn start_voxel_render_pass(&self) -> Result<VoxelPass, wgpu::SurfaceError> {
        let view = self
            .output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .engine
            .renderer()
            .backend()
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("vengine::render_scene_encoder"),
            });

        let depth_view = self
            .engine
            .renderer()
            .depth_texture
            .lock()
            .unwrap()
            .view
            .clone();

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.engine.renderer().voxel_pipeline);
        pass.set_bind_group(0, self.engine.renderer().camera().bind_group(), &[]);

        // Quad buffer (bleibt für alle Chunks gleich)
        pass.set_vertex_buffer(0, self.engine.renderer().quad.slice(..));

        Ok(VoxelPass::new(pass.forget_lifetime(), encoder))
    }

    pub fn start_ui_render_pass(&self) -> UiPass {
        let mut ui_state = self.engine.ui_renderer().state();
        let raw_input = ui_state.take_egui_input(self.engine.window().window());

        ui_state.egui_ctx().begin_pass(raw_input);

        let view = self
            .output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .engine
            .renderer()
            .backend()
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("vengine::render_ui_encoder"),
            });

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
            self.dimensions.0 as f32,
            self.dimensions.1 as f32,
            0.0,
            1.0,
        );

        UiPass::new(
            pass.forget_lifetime(),
            encoder,
            self.engine.ui_renderer().context().clone(),
            self.dimensions,
        )
    }

    pub fn finish_voxel_render_pass(&self, pass: VoxelPass) {
        let (encoder, pass) = pass.into_inner();

        drop(pass);

        let mut lock = self.encoders.lock().unwrap();

        lock.push(encoder);
    }

    pub fn finish_ui_render_pass(&self, pass: UiPass) {
        let mut ui_renderer = self.engine.ui_renderer().renderer();

        let (mut encoder, pass, dimensions) = pass.into_inner();

        let full_output = self.engine.ui_renderer().state().egui_ctx().end_pass();

        let tris = self
            .engine
            .ui_renderer()
            .state()
            .egui_ctx()
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            ui_renderer.update_texture(
                self.engine.renderer().backend().device(),
                self.engine.renderer().backend().queue(),
                *id,
                image_delta,
            );
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [dimensions.0, dimensions.1],
            pixels_per_point: self.engine.window().window().scale_factor() as f32,
        };

        ui_renderer.update_buffers(
            self.engine.renderer().backend().device(),
            self.engine.renderer().backend().queue(),
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        ui_renderer.render(&mut pass.forget_lifetime(), &tris, &screen_descriptor);
        for x in &full_output.textures_delta.free {
            ui_renderer.free_texture(x)
        }

        let mut lock = self.encoders.lock().unwrap();

        lock.push(encoder);
    }

    pub fn finish(self) -> SurfaceTexture {
        let buffers = self
            .encoders
            .into_inner()
            .unwrap()
            .into_iter()
            .map(|e| e.finish())
            .collect::<Vec<CommandBuffer>>();

        self.engine.renderer().backend().queue().submit(buffers);

        self.output
    }
}
