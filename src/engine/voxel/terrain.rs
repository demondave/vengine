use crate::engine::core::engine::Engine;
use crate::engine::physics::simulation::Simulation;
use crate::engine::renderer::frame::voxel_pass::VoxelPass;
use crate::engine::voxel::chunk::{Chunk, CHUNK_SIZE, VOXEL_SIZE};
use crate::engine::voxel::chunk_mesh::ChunkMesh;
use ahash::{HashMap, HashMapExt};
use cgmath::{Matrix4, SquareMatrix, Vector3};
use colorgrad::Gradient;
use nalgebra::DMatrix;
use noise::{NoiseFn, Perlin};
use rapier3d::dynamics::RigidBodyBuilder;
use rapier3d::geometry::ColliderBuilder;
use std::sync::Arc;
use wgpu::Device;

pub const MAX_STACKED_CHUNKS: usize = 8;

pub struct Terrain {
    seed: u32,
    distance: u32,
    gradient: Box<dyn Gradient>,
    device: Arc<Device>,
    chunks: HashMap<Vector3<i32>, ChunkMesh>,
    height_cache: HashMap<(i32, i32), usize>,
    height_bounds_cache: HashMap<(i32, i32), (i32, i32)>,
}

impl Terrain {
    pub fn new(
        seed: u32,
        distance: u32,
        gradient: Box<dyn Gradient>,
        device: Arc<Device>,
    ) -> Terrain {
        let capacity = (distance * 2).pow(2) as usize;

        Terrain {
            seed,
            distance,
            gradient,
            device,
            chunks: HashMap::with_capacity(capacity),
            height_cache: HashMap::new(),
            height_bounds_cache: HashMap::with_capacity(capacity),
        }
    }

    fn get_cached_height(&mut self, x: i32, z: i32) -> usize {
        *self
            .height_cache
            .entry((x, z))
            .or_insert_with(|| heightmap(self.seed, x, z))
    }

    fn generate_chunk(&mut self, chunk_x: i32, chunk_y: i32, chunk_z: i32) -> Option<ChunkMesh> {
        let min_x = chunk_x * CHUNK_SIZE as i32;
        let min_z = chunk_z * CHUNK_SIZE as i32;
        let min_y = chunk_y * CHUNK_SIZE as i32;

        let bounds_key = (chunk_x, chunk_z);

        let (min_height, max_height) =
            if let Some(&bounds) = self.height_bounds_cache.get(&bounds_key) {
                bounds
            } else {
                let mut min_height = i32::MAX;
                let mut max_height = i32::MIN;

                for dx in (0..CHUNK_SIZE as i32).step_by(4) {
                    for dz in (0..CHUNK_SIZE as i32).step_by(4) {
                        let h = self.get_cached_height(min_x + dx, min_z + dz) as i32;

                        min_height = min_height.min(h);
                        max_height = max_height.max(h);
                    }
                }

                self.height_bounds_cache
                    .insert(bounds_key, (min_height, max_height));

                (min_height, max_height)
            };

        if max_height < min_y || min_height >= min_y + CHUNK_SIZE as i32 {
            return None;
        }

        let mut chunk = Chunk::empty();
        let mut has_voxels = false;

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let height = self.get_cached_height(min_x + x as i32, min_z + z as i32) as i32;

                if height >= min_y && height < min_y + CHUNK_SIZE as i32 {
                    has_voxels = true;

                    let local_height = (height - min_y) as usize;

                    for y in 0..=local_height {
                        chunk.set(
                            x,
                            y,
                            z,
                            true,
                            self.gradient.at((y % 128) as f32 / 128.0).to_rgba8(),
                        );
                    }
                } else if height >= min_y + CHUNK_SIZE as i32 {
                    has_voxels = true;

                    for y in 0..CHUNK_SIZE {
                        chunk.set(
                            x,
                            y,
                            z,
                            true,
                            self.gradient.at((y % 128) as f32 / 128.0).to_rgba8(),
                        );
                    }
                }
            }
        }

        if has_voxels {
            let mut chunk_mesh = ChunkMesh::new(chunk);

            chunk_mesh.remesh();
            chunk_mesh.allocate(&self.device);

            Some(chunk_mesh)
        } else {
            None
        }
    }

    pub fn render(&mut self, engine: &Engine, pass: &mut VoxelPass, simulation: &mut Simulation) {
        let eye = engine.camera().get_eye();

        let eye_x = eye.x as i32 / CHUNK_SIZE as i32;
        let eye_y = eye.y as i32 / CHUNK_SIZE as i32;
        let eye_z = eye.z as i32 / CHUNK_SIZE as i32;

        let distance = self.distance as i32;

        for x in (eye_x - distance)..(eye_x + distance) {
            for y in (eye_y - distance)..(eye_y + distance) {
                for z in (eye_z - distance)..(eye_z + distance) {
                    let chunk_pos = Vector3::new(x, y, z);

                    match self.chunks.get(&chunk_pos) {
                        Some(chunk) => {
                            pass.render_chunk(Matrix4::identity(), chunk_pos, chunk);
                        }
                        None => {
                            if let Some(chunk) = self.generate_chunk(x, y, z) {
                                let rigid_body = RigidBodyBuilder::fixed()
                                    .translation(nalgebra::Vector3::new(
                                        chunk_pos.x as f32 * CHUNK_SIZE as f32
                                            + CHUNK_SIZE as f32 / 2.0,
                                        VOXEL_SIZE / 2.0,
                                        chunk_pos.z as f32 * CHUNK_SIZE as f32
                                            + CHUNK_SIZE as f32 / 2.0,
                                    ))
                                    .build();

                                let handle = simulation.add_rigid_body(rigid_body);

                                let mut heights = DMatrix::<f32>::zeros(CHUNK_SIZE, CHUNK_SIZE);

                                for x in 0..CHUNK_SIZE {
                                    for z in 0..CHUNK_SIZE {
                                        let y = self.get_cached_height(
                                            chunk_pos.x * CHUNK_SIZE as i32 + x as i32,
                                            chunk_pos.z * CHUNK_SIZE as i32 + z as i32,
                                        );

                                        heights[(z, x)] = y as f32;
                                    }
                                }

                                let collider = ColliderBuilder::heightfield(
                                    heights,
                                    nalgebra::Vector3::new(
                                        CHUNK_SIZE as f32,
                                        1.0,
                                        CHUNK_SIZE as f32,
                                    ),
                                );

                                simulation.add_collider(collider, Some(handle));

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

fn heightmap(seed: u32, x: i32, z: i32) -> usize {
    let perlin = Perlin::new(seed);

    const SCALE: f64 = 0.01;
    const HEIGHT_MULTIPLIER: f64 = 25.0;
    const OCTAVES: u32 = 4;
    const PERSISTENCE: f64 = 0.5;

    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut height = 0.0;

    for _ in 0..OCTAVES {
        height +=
            perlin.get([x as f64 * SCALE * frequency, z as f64 * SCALE * frequency]) * amplitude;

        amplitude *= PERSISTENCE;
        frequency *= 2.0;
    }

    let height = (height + 1.0) * HEIGHT_MULTIPLIER;

    height.clamp(0.0, (MAX_STACKED_CHUNKS * CHUNK_SIZE - 1) as f64) as usize
}
