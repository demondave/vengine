use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, Device,
};

use super::{chunk::Chunk, quad::Quad};

pub struct ChunkMesh {
    chunk: Chunk,
    quads: Option<Vec<Quad>>,
    /// On-Device quad buffer
    buffer: Option<Buffer>,
    /// Indices for face starts, (Left, Right, Up, Down, Front, Back)
    offsets: [u16; 6],
}

impl ChunkMesh {
    pub fn new(chunk: Chunk) -> Self {
        Self {
            chunk,
            quads: None,
            buffer: None,
            offsets: [0u16; 6],
        }
    }

    pub fn chunk(&self) -> &Chunk {
        &self.chunk
    }

    pub fn offsets(&self) -> &[u16; 6] {
        &self.offsets
    }

    pub fn chunk_mut(&mut self) -> &mut Chunk {
        &mut self.chunk
    }

    pub fn quads(&self) -> Option<&[Quad]> {
        self.quads.as_deref()
    }

    pub fn remesh(&mut self) {
        let mut quads = Vec::new();
        self.chunk.remesh(&mut self.offsets, &mut quads);

        self.quads = Some(quads);
    }

    pub fn allocate(&mut self, device: &Device) -> bool {
        if let Some(quads) = &self.quads {
            self.buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(quads),
                usage: wgpu::BufferUsages::VERTEX,
            }));

            return true;
        }

        false
    }

    pub fn into_chunk(self) -> Chunk {
        self.chunk
    }

    pub fn deallocate(&self) {
        if let Some(buffer) = &self.buffer {
            buffer.destroy();
        }
    }

    pub fn buffer(&self) -> &Option<Buffer> {
        &self.buffer
    }
}
