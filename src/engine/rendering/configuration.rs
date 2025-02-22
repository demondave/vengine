use super::{backend::Backend, camera::Camera};

pub trait Configuration {
    fn initialize(&mut self, backend: &Backend<'_>, camera: &Camera);
}
