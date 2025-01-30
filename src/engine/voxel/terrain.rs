use std::sync::Arc;

use ahash::{HashMap, HashMapExt};
use cgmath::{Matrix4, SquareMatrix, Vector3};
use noise::{NoiseFn, Perlin};
use wgpu::Device;

use crate::engine::{core::engine::Engine, renderer::pass::Pass};

use super::{
    chunk::{Chunk, CHUNK_SIZE},
    object::ChunkEx,
};

pub struct Terrain {
    distance: i32,
    chunks: HashMap<Vector3<i32>, ChunkEx>,
    device: Arc<Device>,
}

impl Terrain {
    pub fn new(distance: i32, device: Arc<Device>) -> Terrain {
        Terrain {
            distance,
            chunks: HashMap::with_capacity((distance * 2 * distance * 2) as usize),
            device,
        }
    }

    pub fn render(&mut self, engine: &Engine, pass: &mut Pass) {
        let eye = engine.camera().get_eye();

        let eye_x = eye.x as i32 / CHUNK_SIZE as i32;
        let eye_z = eye.z as i32 / CHUNK_SIZE as i32;

        for z in (eye_z - self.distance)..(eye_z + self.distance) {
            for x in (eye_x - self.distance)..(eye_x + self.distance) {
                match self.chunks.get(&Vector3::new(x, 0, z)) {
                    Some(chunk) => {
                        pass.render_chunk(Matrix4::identity(), Vector3::new(x, 0, z), chunk);
                    }
                    None => {
                        let min_x = x * CHUNK_SIZE as i32;
                        let min_z = z * CHUNK_SIZE as i32;

                        let mut chunk = Chunk::empty();

                        for z in 0..CHUNK_SIZE {
                            for x in 0..CHUNK_SIZE {
                                let n = heightmap(min_x + x as i32, min_z + z as i32);

                                chunk.set(x, n, z, true);

                                for n in 0..n {
                                    chunk.set(x, n, z, true);
                                }
                            }
                        }

                        let mut chunk = ChunkEx::new(chunk);
                        chunk.remesh();
                        chunk.allocate(&self.device);

                        pass.render_chunk(Matrix4::identity(), Vector3::new(x, 0, z), &chunk);

                        self.chunks.insert(Vector3::new(x, 0, z), chunk);
                    }
                }
            }
        }
    }
}

fn heightmap(x: i32, z: i32) -> usize {
    let seed = 69;
    let perlin = Perlin::new(seed);
    const SCALE: f64 = 0.01;
    const HEIGHT_MULTIPLIER: f64 = 8.0;
    const OCTAVES: u32 = 4;
    const PERSISTENCE: f64 = 0.5;

    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut height = 0.0;

    // Sum multiple octaves of noise
    for _ in 0..OCTAVES {
        height +=
            perlin.get([x as f64 * SCALE * frequency, z as f64 * SCALE * frequency]) * amplitude;

        amplitude *= PERSISTENCE;
        frequency *= 2.0;
    }

    // Normalize and scale
    let height = (height + 1.0) * HEIGHT_MULTIPLIER;
    height.clamp(0f64, CHUNK_SIZE as f64 - 1.0) as usize
}
