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
    height_cache: HashMap<(i32, i32), usize>, // Cache heightmap results
}

impl Terrain {
    pub fn new(distance: i32, device: Arc<Device>) -> Terrain {
        Terrain {
            distance,
            chunks: HashMap::with_capacity((distance * 2 * distance * 2) as usize),
            device,
            height_cache: HashMap::new(),
        }
    }

    fn get_cached_height(&mut self, x: i32, z: i32) -> usize {
        if let Some(&height) = self.height_cache.get(&(x, z)) {
            height
        } else {
            let height = heightmap(x, z);
            self.height_cache.insert((x, z), height);
            height
        }
    }

    fn generate_chunk(&mut self, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Option<ChunkEx> {
        let min_x = chunk_x * CHUNK_SIZE as i32;
        let min_z = chunk_z * CHUNK_SIZE as i32;
        let min_y = chunk_y * CHUNK_SIZE as i32;

        // Quick height check to see if we need this chunk at all
        let corner_heights = [
            self.get_cached_height(min_x, min_z),
            self.get_cached_height(min_x + CHUNK_SIZE as i32 - 1, min_z),
            self.get_cached_height(min_x, min_z + CHUNK_SIZE as i32 - 1),
            self.get_cached_height(min_x + CHUNK_SIZE as i32 - 1, min_z + CHUNK_SIZE as i32 - 1),
        ];

        let min_height = *corner_heights.iter().min().unwrap() as i32;
        let max_height = *corner_heights.iter().max().unwrap() as i32;

        // Skip chunk if it's completely above or below the terrain
        if max_height < min_y || min_height >= min_y + CHUNK_SIZE as i32 {
            return None;
        }

        let mut chunk = Chunk::empty();
        let mut has_blocks = false;

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let height = self.get_cached_height(min_x + x as i32, min_z + z as i32);
                let absolute_height = height as i32;

                if absolute_height >= min_y && absolute_height < min_y + CHUNK_SIZE as i32 {
                    has_blocks = true;
                    let local_height = (absolute_height - min_y) as usize;
                    for y in 0..=local_height {
                        chunk.set(x, y, z, true);
                    }
                } else if absolute_height >= min_y + CHUNK_SIZE as i32 {
                    has_blocks = true;
                    for y in 0..CHUNK_SIZE {
                        chunk.set(x, y, z, true);
                    }
                }
            }
        }

        if has_blocks {
            let mut chunk_ex = ChunkEx::new(chunk);
            chunk_ex.remesh();
            chunk_ex.allocate(&self.device);
            Some(chunk_ex)
        } else {
            None
        }
    }

    pub fn render(&mut self, engine: &Engine, pass: &mut Pass) {
        let eye = engine.camera().get_eye();

        let eye_x = eye.x as i32 / CHUNK_SIZE as i32;
        let eye_y = eye.y as i32 / CHUNK_SIZE as i32;
        let eye_z = eye.z as i32 / CHUNK_SIZE as i32;

        for y in (eye_y - self.distance)..(eye_y + self.distance) {
            for z in (eye_z - self.distance)..(eye_z + self.distance) {
                for x in (eye_x - self.distance)..(eye_x + self.distance) {
                    let chunk_pos = Vector3::new(x, y, z);

                    match self.chunks.get(&chunk_pos) {
                        Some(chunk) => {
                            pass.render_chunk(Matrix4::identity(), chunk_pos, chunk);
                        }
                        None => {
                            if let Some(chunk) = self.generate_chunk(x, y, z) {
                                pass.render_chunk(Matrix4::identity(), chunk_pos, &chunk);
                                self.chunks.insert(chunk_pos, chunk);
                            }
                        }
                    }
                }
            }
        }
    }
}
pub const MAX_HEIGHT_CHUNKS: usize = 8; // 256 blocks total height (8 * 32)
fn heightmap(x: i32, z: i32) -> usize {
    let seed = 69;
    let perlin = Perlin::new(seed);
    const SCALE: f64 = 0.01;
    const HEIGHT_MULTIPLIER: f64 = 25.0;
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
    height.clamp(0.0, (MAX_HEIGHT_CHUNKS * CHUNK_SIZE - 1) as f64) as usize
}
