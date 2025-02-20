use super::Frame;

pub trait RenderPass {
    fn start(frame: &Frame) -> Self;

    fn finish(self, frame: &Frame);
}
