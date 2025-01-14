use bytemuck::cast_slice;
use cgmath::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, Device,
};

use super::quad::Quad;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
    Front = 4,
    Back = 5,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
}

// The chunk coordinates are left handed
pub struct Chunk {
    // Z * Y * X
    position: Vector3<f32>,
    voxels: Box<[u32; 32 * 32]>,
    quads: Vec<Quad>,
    buffer: Option<Buffer>,
}

impl Chunk {
    pub fn empty(position: Vector3<f32>) -> Chunk {
        let mut quads = Vec::with_capacity(2 ^ 16);
        quads.extend_from_slice(&[Quad::default(); 3]);

        let mut chunk = Chunk {
            position,
            voxels: Box::new([0u32; 32 * 32]),
            quads,
            buffer: None,
        };

        chunk.set_position(position);

        chunk
    }

    pub fn get_position(&self) -> Vector3<f32> {
        self.position
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        let tmp = [position.x, position.y, position.z];
        let dst: &mut [u8] = bytemuck::cast_slice_mut(&mut self.quads[0..3]);
        dst.copy_from_slice(cast_slice(&tmp));
    }

    /// Sets voxel state inside a chunk
    /// The voxel coordinate system is left handed
    pub fn set(&mut self, x: usize, y: usize, z: usize, state: bool) {
        if state {
            self.voxels[(z * 32) + (31 - y)] |= 2147483648 >> x;
        } else {
            self.voxels[(z * 32) + (31 - y)] &= u32::MAX ^ (2147483648 >> x);
        }
    }

    /// Gets a voxel state inside a chunk
    /// The voxel coordinate system is left handed
    pub fn get(&self, x: usize, y: usize, z: usize) -> bool {
        self.voxels[(z * 32) + (31 - y)] & (2147483648 >> x) != 0
    }

    pub fn remesh(&mut self) {
        self.quads.truncate(3);

        let mut buffer = [[0u32; 32]; 34];

        // X-Axis
        for n in 0..32 {
            self.slice(Axis::X, n, &mut buffer[n + 1]);
        }

        for n in 1..33 {
            // "Vertical"
            for a in 0..32 {
                // "Horizontal"
                for b in 0..32 {
                    let right = &buffer[n - 1];
                    let mid = &buffer[n];
                    let left = &buffer[n + 1];

                    if left[a] & (2147483648 >> b) == 0 && mid[a] & (2147483648 >> b) != 0 {
                        self.quads
                            .push(Quad::new(Direction::Left, n - 1, 31 - a, b, 69));
                    }
                    if mid[a] & (2147483648 >> b) != 0 && right[a] & (2147483648 >> b) == 0 {
                        self.quads
                            .push(Quad::new(Direction::Right, n - 1, 31 - a, b, 69));
                    }
                }
            }
        }

        // Y-Axis
        for n in 0..32 {
            self.slice(Axis::Y, n, &mut buffer[n + 1]);
        }

        for n in 1..33 {
            // "Vertical"
            for a in 0..32 {
                // "Horizontal"
                for b in 0..32 {
                    let up = &buffer[n + 1];
                    let mid = &buffer[n];
                    let down = &buffer[n - 1];

                    if up[a] & (2147483648 >> b) == 0 && mid[a] & (2147483648 >> b) != 0 {
                        self.quads
                            .push(Quad::new(Direction::Up, b, n - 1, 31 - a, 69));
                    }
                    if mid[a] & (2147483648 >> b) != 0 && down[a] & (2147483648 >> b) == 0 {
                        self.quads
                            .push(Quad::new(Direction::Down, b, n - 1, 31 - a, 69));
                    }
                }
            }
        }

        // Z-Axis
        for n in 0..32 {
            self.slice(Axis::Z, n, &mut buffer[n + 1]);
        }

        for n in 1..33 {
            // "Vertical"
            for a in 0..32 {
                // "Horizontal"
                for b in 0..32 {
                    let front = &buffer[n - 1];
                    let mid = &buffer[n];
                    let back = &buffer[n + 1];

                    if front[a] & (2147483648 >> b) == 0 && mid[a] & (2147483648 >> b) != 0 {
                        self.quads
                            .push(Quad::new(Direction::Front, b, 31 - a, n - 1, 69));
                    }
                    if mid[a] & (2147483648 >> b) != 0 && back[a] & (2147483648 >> b) == 0 {
                        self.quads
                            .push(Quad::new(Direction::Back, b, 31 - a, n - 1, 69));
                    }
                }
            }
        }
    }

    pub fn quads(&self) -> &[Quad] {
        &self.quads[3.min(self.quads.len())..]
    }

    pub fn allocate(&mut self, device: &Device) -> bool {
        if !self.quads.is_empty() {
            self.buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.quads),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_SRC,
            }));

            return true;
        }

        false
    }

    pub fn deallocate(&self) {
        if let Some(buffer) = &self.buffer {
            buffer.destroy();
        }
    }

    pub fn buffer(&self) -> &Option<Buffer> {
        &self.buffer
    }

    fn slice(&self, axis: Axis, n: usize, buffer: &mut [u32; 32]) {
        match axis {
            Axis::X => {
                for y in 0..32 {
                    for z in 0..32 {
                        buffer[y] |= ((self.voxels[z * 32 + y] << n) & 2147483648) >> z;
                    }
                }
            }
            Axis::Y => {
                for z in 0..32 {
                    buffer[31 - z] = self.voxels[z * 32 + (31 - n)]
                }
            }
            Axis::Z => {
                for y in 0..32 {
                    buffer[y] = self.voxels[(n * 32) + y];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_get() {
        for z in 0..32 {
            for y in 0..32 {
                for x in 0..32 {
                    let mut chunk = Chunk::empty(Vector3::new(0f32, 0f32, 0f32));

                    chunk.set(x, y, z, true);
                    assert!(chunk.get(x, y, z));
                }
            }
        }
    }

    #[test]
    fn test_slice() {
        let mut target = [u32::MAX; 32];

        target[30] ^= 1 << 30;

        let mut buffer = [0u32; 32];

        // X-Axis
        for n in 0..32 {
            let mut chunk = Chunk::empty(Vector3::new(0f32, 0f32, 0f32));

            for y in 0..32 {
                for z in 0..32 {
                    chunk.set(n, z, y, true);
                }
            }

            chunk.set(n, 1, 1, false);

            chunk.slice(Axis::X, n, &mut buffer);

            assert_eq!(buffer, target);
        }

        // Y-Axis
        for n in 0..32 {
            let mut chunk = Chunk::empty(Vector3::new(0f32, 0f32, 0f32));

            for x in 0..32 {
                for z in 0..32 {
                    chunk.set(x, n, z, true);
                }
            }

            chunk.set(1, n, 1, false);

            chunk.slice(Axis::Y, n, &mut buffer);

            assert_eq!(buffer, target);
        }

        // Z-Axis
        for n in 0..32 {
            let mut chunk = Chunk::empty(Vector3::new(0f32, 0f32, 0f32));

            for y in 0..32 {
                for x in 0..32 {
                    chunk.set(x, y, n, true);
                }
            }

            chunk.set(1, 1, n, false);

            chunk.slice(Axis::Z, n, &mut buffer);

            assert_eq!(buffer, target);
        }
    }
}
