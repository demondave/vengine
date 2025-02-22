use crate::engine::rendering::{configuration::Configuration, pipeline::GetPipeline};

use super::Frame;

pub trait RenderPass {
    type RequiredPipeline;

    fn start<C: Configuration + GetPipeline<Self::RequiredPipeline>>(frame: &Frame<C>) -> Self;

    fn finish<C: Configuration + GetPipeline<Self::RequiredPipeline>>(self, frame: &Frame<C>);
}
