use std::fmt::Debug;

use super::chunk::Direction;

#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Quad(pub u32);

impl Quad {
    pub fn new(direction: Direction, x: usize, y: usize, z: usize, texture: usize) -> Self {
        Self(
            (x as u32)
                | ((y as u32) << 6)
                | ((z as u32) << 12)
                | ((direction as u32) << 18)
                | ((texture as u32) << 21),
        )
    }

    pub fn x(&self) -> u32 {
        self.0 & 0b00000000000000000000000000111111
    }

    pub fn y(&self) -> u32 {
        (self.0 & 0b00000000000000000000111111000000) >> 6
    }

    pub fn z(&self) -> u32 {
        (self.0 & 0b00000000000000111111000000000000) >> 12
    }

    pub fn direction(&self) -> Direction {
        match (self.0 & 0b00000000000111000000000000000000) >> 18 {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Right,
            4 => Direction::Front,
            5 => Direction::Back,
            _ => panic!("invalid direction"),
        }
    }

    pub fn texture_id(&self) -> u32 {
        (self.0 & 0b00001111111000000000000000000000) >> 21
    }
}

impl Debug for Quad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x())
            .field("y", &self.y())
            .field("z", &self.z())
            .field("direction", &self.direction())
            .field("texture_id", &self.texture_id())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quad() {
        let directions = &[
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
            Direction::Front,
            Direction::Back,
        ];

        for d in directions {
            for z in 0..32 {
                for y in 0..32 {
                    for x in 0..32 {
                        for t in 0..128 {
                            let quad = Quad::new(*d, x, y, z, t);

                            assert_eq!(quad.x(), x as u32);
                            assert_eq!(quad.y(), y as u32);
                            assert_eq!(quad.z(), z as u32);
                            assert_eq!(quad.texture_id(), t as u32);
                            assert_eq!(quad.direction(), *d);
                        }
                    }
                }
            }
        }
    }
}
