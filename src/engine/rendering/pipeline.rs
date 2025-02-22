use super::{backend::Backend, camera::Camera};

pub trait Pipeline {
    fn initialize(backend: &Backend<'_>, camera: &Camera) -> Self;
}

pub trait GetPipeline<T> {
    fn get_pipeline(&self) -> &T;
}
