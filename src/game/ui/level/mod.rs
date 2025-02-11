pub mod custom;
pub mod physics;
pub mod procedural;

pub trait SeededLevel {
    fn with_seed(seed: u32) -> Self;
}
