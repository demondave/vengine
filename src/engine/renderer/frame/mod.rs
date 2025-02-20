use pass::RenderPass;
use std::sync::Mutex;
use wgpu::{CommandBuffer, CommandEncoder, SurfaceTexture};

use super::{size::Size, Renderer};

pub mod pass;
pub mod ui_pass;
pub mod voxel_pass;

pub struct Frame<'a> {
    renderer: &'a Renderer<'a>,
    output: SurfaceTexture,
    encoders: Mutex<Vec<CommandEncoder>>,
    size: Size,
}

impl<'a> Frame<'a> {
    pub fn new(renderer: &'a Renderer, output: SurfaceTexture) -> Self {
        let size = renderer.size();

        Self {
            renderer,
            output,
            encoders: Mutex::new(Vec::with_capacity(32)),
            size,
        }
    }

    pub fn start_render_pass<T: RenderPass>(&self) -> T {
        T::start(self)
    }
    pub fn finish_render_pass<T: RenderPass>(&self, pass: T) {
        pass.finish(self);
    }

    pub fn renderer(&self) -> &Renderer {
        self.renderer
    }

    pub fn output(&self) -> &SurfaceTexture {
        &self.output
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn push_encoder(&self, encoder: CommandEncoder) {
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

        self.renderer.backend().queue().submit(buffers);

        self.output
    }
}
