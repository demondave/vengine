use std::fmt::Debug;

use super::chunk::direction::Direction;

#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Quad {
    low: u32,
    color: u32,
}

impl Quad {
    pub fn new(direction: Direction, x: usize, y: usize, z: usize, color: [u8; 4]) -> Self {
        let low = (x as u32) | ((y as u32) << 6) | ((z as u32) << 12) | ((direction as u32) << 18);

        Self {
            low,
            color: u32::from_be_bytes(color),
        }
    }

    pub fn x(&self) -> u32 {
        self.low & 0b00000000000000000000000000111111
    }

    pub fn y(&self) -> u32 {
        (self.low & 0b00000000000000000000111111000000) >> 6
    }

    pub fn z(&self) -> u32 {
        (self.low & 0b00000000000000111111000000000000) >> 12
    }

    pub fn direction(&self) -> Direction {
        match (self.low & 0b00000000000111000000000000000000) >> 18 {
            0 => Direction::Left,
            1 => Direction::Right,
            2 => Direction::Up,
            3 => Direction::Down,
            4 => Direction::Front,
            5 => Direction::Back,
            _ => panic!("invalid direction"),
        }
    }

    pub fn color(&self) -> [u8; 4] {
        self.color.to_le_bytes()
    }

    pub fn set_texture_id(&mut self, id: u8) {
        // Erst die alten Bits löschen
        self.low &= !(0b01111111 << 21);
        // Dann die neuen Bits setzen
        self.low |= ((0b01111111 & id) as u32) << 21;
    }
}

impl Debug for Quad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Point")
            .field("x", &self.x())
            .field("y", &self.y())
            .field("z", &self.z())
            .field("direction", &self.direction())
            .field("texture_id", &self.color())
            .finish()
    }
}

#[test]
fn test_quad() {
    let directions = &[
        Direction::Left,
        Direction::Right,
        Direction::Up,
        Direction::Down,
        Direction::Front,
        Direction::Back,
    ];

    for d in directions {
        for z in 0..32 {
            for y in 0..32 {
                for x in 0..32 {
                    let quad = Quad::new(*d, x, y, z, [0u8; 4]);

                    assert_eq!(quad.x(), x as u32);
                    assert_eq!(quad.y(), y as u32);
                    assert_eq!(quad.z(), z as u32);
                    assert_eq!(quad.color(), [0u8; 4]);
                    assert_eq!(quad.direction(), *d);
                }
            }
        }
    }
}
