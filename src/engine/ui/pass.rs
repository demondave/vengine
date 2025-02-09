use egui::Context;
use wgpu::{CommandEncoder, RenderPass};

pub struct Pass {
    encoder: CommandEncoder,
    pass: RenderPass<'static>,

    context: Context,
}

impl Pass {
    pub(super) fn new(
        pass: RenderPass<'static>,
        encoder: CommandEncoder,
        context: Context,
    ) -> Self {
        Self {
            encoder,
            pass,
            context,
        }
    }

    pub fn render_ui<F: FnOnce(&Context)>(&mut self, f: F) {
        f(&self.context);
    }

    pub(super) fn finish(self) -> (CommandEncoder, RenderPass<'static>) {
        (self.encoder, self.pass)
    }
}
