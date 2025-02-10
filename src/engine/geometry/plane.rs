use cgmath::{InnerSpace, Vector3};

pub struct Plane {
    point: Vector3<f32>,
    normal: Vector3<f32>,
}

impl Plane {
    pub fn new(point: Vector3<f32>, normal: Vector3<f32>) -> Self {
        let normal = normal.normalize();
        Self { point, normal }
    }

    pub fn distance(&self, point: &Vector3<f32>) -> f32 {
        (point - self.point).dot(self.normal)
    }

    pub fn side(&self, point: &Vector3<f32>) -> bool {
        if self.distance(point) >= 0.0 {
            return true;
        }

        false
    }
}
