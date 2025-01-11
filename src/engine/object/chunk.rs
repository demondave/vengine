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
    voxels: Box<[u32; 32 * 32]>,
    quads: Vec<Quad>,
    buffer: Option<Buffer>,
}

impl Chunk {
    pub fn empty() -> Chunk {
        Chunk {
            voxels: Box::new([0u32; 32 * 32]),
            quads: Vec::with_capacity(2 ^ 16),
            buffer: None,
        }
    }

    pub fn amogus(&mut self) {
        self.quads.push(Quad::new(Direction::Up, 0, 1, 1, 120));
        self.quads.push(Quad::new(Direction::Down, 0, 1, 1, 69));
        self.quads.push(Quad::new(Direction::Left, 0, 1, 1, 29));
        self.quads.push(Quad::new(Direction::Right, 0, 1, 1, 79));
        self.quads.push(Quad::new(Direction::Front, 0, 1, 1, 45));
        self.quads.push(Quad::new(Direction::Back, 0, 1, 1, 45));
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
        let mut previous = [0u32; 32];
        let mut current = [0u32; 32];

        // Left
        for n in 0..32 {
            self.slice(Axis::X, n, &mut current);
            // "Vertical"
            for a in 0..32 {
                // "Horizontal"
                for b in 0..32 {
                    if previous[a] & (2147483648 >> b) == 0 && current[a] & (2147483648 >> b) != 0 {
                        self.quads
                            .push(Quad::new(Direction::Left, n, 31 - a, b, 69));
                    }
                }
            }

            previous.copy_from_slice(&current);
        }

        previous = [0u32; 32];

        // Right
        for n in 0..32 {
            self.slice(Axis::X, 31 - n, &mut current);
            // "Vertical"
            for a in 0..32 {
                // "Horizontal"
                for b in 0..32 {
                    if previous[a] & (2147483648 >> b) == 0 && current[a] & (2147483648 >> b) != 0 {
                        self.quads
                            .push(Quad::new(Direction::Right, n, 31 - a, b, 69));
                    }
                }
            }

            previous.copy_from_slice(&current);
        }

        previous = [0u32; 32];

        // Up
        for n in 0..32 {
            self.slice(Axis::Y, n, &mut current);
            // "Vertical"
            for a in 0..32 {
                // "Horizontal"
                for b in 0..32 {
                    if previous[a] & (2147483648 >> b) == 0 && current[a] & (2147483648 >> b) != 0 {
                        self.quads.push(Quad::new(Direction::Up, b, n, 31 - a, 69));
                    }
                }
            }

            previous.copy_from_slice(&current);
        }

        previous = [0u32; 32];

        // Down
        for n in 0..32 {
            self.slice(Axis::Y, 31 - n, &mut current);
            // "Vertical"
            for a in 0..32 {
                // "Horizontal"
                for b in 0..32 {
                    if previous[a] & (2147483648 >> b) == 0 && current[a] & (2147483648 >> b) != 0 {
                        self.quads
                            .push(Quad::new(Direction::Down, b, 31 - n, 31 - a, 69));
                    }
                }
            }

            previous.copy_from_slice(&current);
        }

        previous = [0u32; 32];

        // Front
        for n in 0..32 {
            self.slice(Axis::Z, n, &mut current);
            // "Vertical"
            for a in 0..32 {
                // "Horizontal"
                for b in 0..32 {
                    if previous[a] & (2147483648 >> b) == 0 && current[a] & (2147483648 >> b) != 0 {
                        self.quads
                            .push(Quad::new(Direction::Front, b, 31 - a, n, 69));
                    }
                }
            }

            previous.copy_from_slice(&current);
        }

        previous = [0u32; 32];

        // Back
        for n in 0..32 {
            self.slice(Axis::Z, 31 - n, &mut current);
            // "Vertical"
            for a in 0..32 {
                // "Horizontal"
                for b in 0..32 {
                    if previous[a] & (2147483648 >> b) == 0 && current[a] & (2147483648 >> b) != 0 {
                        self.quads
                            .push(Quad::new(Direction::Back, b, 31 - a, 31 - n, 69));
                    }
                }
            }

            previous.copy_from_slice(&current);
        }
    }

    pub fn quads(&self) -> &[Quad] {
        &self.quads
    }

    pub fn allocate(&mut self, device: &Device) -> bool {
        if !self.quads.is_empty() {
            self.buffer = Some(device.create_buffer_init(&BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.quads),
                usage: wgpu::BufferUsages::VERTEX,
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

#[test]
fn test_set_get() {
    for z in 0..32 {
        for y in 0..32 {
            for x in 0..32 {
                let mut chunk = Chunk::empty();

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
        let mut chunk = Chunk::empty();

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
        let mut chunk = Chunk::empty();

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
        let mut chunk = Chunk::empty();

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
