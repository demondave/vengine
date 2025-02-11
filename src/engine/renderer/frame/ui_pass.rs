use egui::Context;
use wgpu::{CommandEncoder, RenderPass};

pub struct UiPass {
    encoder: CommandEncoder,
    pass: RenderPass<'static>,
    context: Context,
    dimensions: (u32, u32),
}

impl UiPass {
    pub(super) fn new(
        pass: RenderPass<'static>,
        encoder: CommandEncoder,
        context: Context,
        dimensions: (u32, u32),
    ) -> Self {
        Self {
            encoder,
            pass,
            context,
            dimensions,
        }
    }

    pub fn render_ui<F: FnOnce(&Context)>(&mut self, f: F) {
        f(&self.context);
    }

    pub fn into_inner(self) -> (CommandEncoder, RenderPass<'static>, (u32, u32)) {
        (self.encoder, self.pass, self.dimensions)
    }
}
