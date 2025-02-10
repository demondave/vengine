use egui::Context;
use wgpu::{CommandEncoder, RenderPass};

pub struct Pass {
    encoder: CommandEncoder,
    pass: RenderPass<'static>,

    context: Context,

    static_uis: bool,
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
            static_uis: false,
        }
    }

    pub fn render_ui<F: FnOnce(&Context)>(&mut self, f: F) {
        f(&self.context);
    }

    pub fn render_static_uis(&mut self) {
        self.static_uis = true;
    }

    pub(super) fn finish(self) -> (CommandEncoder, RenderPass<'static>, bool) {
        (self.encoder, self.pass, self.static_uis)
    }
}
