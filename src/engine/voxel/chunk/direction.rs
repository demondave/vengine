use cgmath::Vector3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    /// Left (X+)
    Left = 0,
    /// Right (X-)
    Right = 1,
    /// Up (Y+)
    Up = 2,
    /// Down (Y-)
    Down = 3,
    /// Front (Z+)
    Front = 4,
    /// Back (Z-)
    Back = 5,
}

impl Direction {
    pub fn unit_vector(&self) -> Vector3<f32> {
        match self {
            Direction::Left => Vector3::new(1f32, 0f32, 0f32),
            Direction::Right => Vector3::new(-1f32, 0f32, 0f32),
            Direction::Up => Vector3::new(0f32, 1f32, 0f32),
            Direction::Down => Vector3::new(0f32, -1f32, 0f32),
            Direction::Front => Vector3::new(0f32, 0f32, 1f32),
            Direction::Back => Vector3::new(0f32, 0f32, -1f32),
        }
    }
}
