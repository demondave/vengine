use super::{backend::Backend, camera::Camera};

pub mod ui;
pub mod voxel;

pub trait Pipeline {
    fn initialize(backend: &Backend<'_>, camera: &Camera) -> Self;
}

pub trait GetPipeline<T> {
    fn get_pipeline(&self) -> &T;
}
