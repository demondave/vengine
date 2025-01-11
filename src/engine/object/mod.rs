use chunk::Chunk;

pub mod chunk;
pub mod quad;

pub struct Object {
    pub chunk: Chunk,
}

impl Object {
    pub fn empty() -> Object {
        Object {
            chunk: Chunk::empty(),
        }
    }
}
