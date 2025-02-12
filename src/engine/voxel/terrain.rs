use crate::engine::core::engine::Engine;
use crate::engine::physics::simulation::Simulation;
use crate::engine::renderer::frame::voxel_pass::VoxelPass;
use crate::engine::voxel::chunk::{Chunk, CHUNK_SIZE, VOXEL_SIZE};
use crate::engine::voxel::chunk_mesh::ChunkMesh;
use ahash::{HashMap, HashMapExt, HashSet, HashSetExt};
use cgmath::{EuclideanSpace, Matrix4, MetricSpace, SquareMatrix, Vector3};
use colorgrad::Gradient;
use crossbeam::channel::{unbounded, Receiver, Sender};
use nalgebra::DMatrix;
use noise::{NoiseFn, Perlin};
use rapier3d::dynamics::{RigidBodyBuilder, RigidBodyHandle};
use rapier3d::geometry::ColliderBuilder;
use std::sync::Arc;
use std::thread;
use wgpu::Device;

pub const MAX_STACKED_CHUNKS: usize = 8;

#[allow(clippy::type_complexity)]
pub struct Terrain {
    distance: u32,
    eye_sender: Sender<Vector3<f32>>,
    chunk_receiver: Receiver<(DMatrix<f32>, Arc<(Vector3<i32>, ChunkMesh)>)>,
    unload_sender: Sender<Vector3<i32>>,
    chunks: HashMap<Vector3<i32>, (RigidBodyHandle, Arc<(Vector3<i32>, ChunkMesh)>)>,
}

impl Terrain {
    pub fn new(
        seed: u32,
        distance: u32,
        gradient: Box<dyn Gradient + Send + Sync>,
        engine: &Engine,
    ) -> Terrain {
        let capacity = (distance * 2).pow(2) as usize;

        let (eye_sender, eye_receiver) = unbounded();
        let (chunk_sender, chunk_receiver) = unbounded();
        let (unload_sender, unload_receiver) = unbounded();

        let mut generator = Generator {
            device: engine.device().clone(),
            seed,
            distance,
            gradient,
            chunks: HashSet::with_capacity(capacity),
            height_cache: HashMap::new(),
            height_bounds_cache: HashMap::with_capacity(capacity),
            eye_receiver,
            chunk_sender,
            unload_receiver,
        };

        thread::spawn(move || loop {
            while let Ok(eye) = generator.eye_receiver.recv() {
                generator.generate(eye);

                while let Ok(chunk_pos) = generator.unload_receiver.try_recv() {
                    generator.unload_chunk(chunk_pos);
                }
            }
        });

        Terrain {
            distance,
            eye_sender,
            chunk_receiver,
            unload_sender,
            chunks: HashMap::with_capacity(capacity),
        }
    }

    pub fn render(&mut self, engine: &Engine, pass: &mut VoxelPass, simulation: &mut Simulation) {
        let eye = engine.camera().get_eye();

        self.eye_sender.send(eye.to_vec()).unwrap();

        while let Ok(data) = self.chunk_receiver.try_recv() {
            let heights = data.0;
            let chunk = data.1;

            let rigid_body = RigidBodyBuilder::fixed()
                .translation(nalgebra::Vector3::new(
                    chunk.0.x as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0,
                    VOXEL_SIZE / 2.0,
                    chunk.0.z as f32 * CHUNK_SIZE as f32 + CHUNK_SIZE as f32 / 2.0,
                ))
                .build();

            let handle = simulation.add_rigid_body(rigid_body);

            let collider = ColliderBuilder::heightfield(
                heights,
                nalgebra::Vector3::new(CHUNK_SIZE as f32, 1.0, CHUNK_SIZE as f32),
            );

            simulation.add_collider(collider, Some(handle));

            self.chunks.insert(chunk.0, (handle, chunk));
        }

        let unload: Vec<Vector3<i32>> = self
            .chunks
            .iter()
            .filter_map(|(chunk_pos, _)| {
                if (eye.to_vec() / CHUNK_SIZE as f32).distance(chunk_pos.map(|x| x as f32))
                    > self.distance as f32 * 1.5
                {
                    Some(*chunk_pos)
                } else {
                    None
                }
            })
            .collect();

        for chunk_pos in unload {
            if let Some(chunk) = self.chunks.get(&chunk_pos) {
                simulation.remove_rigid_body(chunk.0);

                self.unload_sender.send(chunk_pos).unwrap();
                self.chunks.remove(&chunk_pos);
            }
        }

        for (chunk_pos, chunk) in &self.chunks {
            pass.render_chunk(Matrix4::identity(), *chunk_pos, &chunk.1 .1);
        }
    }
}

#[allow(clippy::type_complexity)]
struct Generator {
    seed: u32,
    distance: u32,
    gradient: Box<dyn Gradient + Send + Sync>,
    chunks: HashSet<Vector3<i32>>,
    height_cache: HashMap<(i32, i32), usize>,
    height_bounds_cache: HashMap<(i32, i32), (i32, i32)>,
    eye_receiver: Receiver<Vector3<f32>>,
    chunk_sender: Sender<(DMatrix<f32>, Arc<(Vector3<i32>, ChunkMesh)>)>,
    unload_receiver: Receiver<Vector3<i32>>,
    device: Arc<Device>,
}

impl Generator {
    pub fn generate(&mut self, eye: Vector3<f32>) {
        let eye_x = eye.x as i32 / CHUNK_SIZE as i32;
        let eye_y = eye.y as i32 / CHUNK_SIZE as i32;
        let eye_z = eye.z as i32 / CHUNK_SIZE as i32;

        let distance = self.distance as i32;

        for x in (eye_x - distance)..(eye_x + distance) {
            for y in (eye_y - distance)..(eye_y + distance) {
                for z in (eye_z - distance)..(eye_z + distance) {
                    let chunk_pos = Vector3::new(x, y, z);

                    if !self.chunks.contains(&chunk_pos) {
                        if let Some(chunk) = self.generate_chunk(chunk_pos) {
                            let _ = self.chunk_sender.send(chunk);
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::type_complexity)]
    fn generate_chunk(
        &mut self,
        chunk_pos: Vector3<i32>,
    ) -> Option<(DMatrix<f32>, Arc<(Vector3<i32>, ChunkMesh)>)> {
        let min_x = chunk_pos.x * CHUNK_SIZE as i32;
        let min_y = chunk_pos.y * CHUNK_SIZE as i32;
        let min_z = chunk_pos.z * CHUNK_SIZE as i32;

        let bounds_key = (chunk_pos.x, chunk_pos.z);

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
        let perlin = Perlin::new(self.seed);
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let height = self.get_cached_height(min_x + x as i32, min_z + z as i32) as i32;

                const NOISE_INTENSITY: f64 = 3.0;
                if height >= min_y && height < min_y + CHUNK_SIZE as i32 {
                    has_voxels = true;

                    let local_height = (height - min_y) as usize;

                    for y in 0..=local_height {
                        let noise_y = calculate_noise(
                            x as i32,
                            y as i32,
                            z as i32,
                            min_x,
                            min_y,
                            min_z,
                            NOISE_INTENSITY,
                            &perlin,
                        );
                        chunk.set(
                            x,
                            y,
                            z,
                            true,
                            self.gradient.at((noise_y / 256.0) as f32).to_rgba8(),
                        );
                    }
                } else if height >= min_y + CHUNK_SIZE as i32 {
                    has_voxels = true;
                    for y in 0..CHUNK_SIZE {
                        let noise_y = calculate_noise(
                            x as i32,
                            y as i32,
                            z as i32,
                            min_x,
                            min_y,
                            min_z,
                            NOISE_INTENSITY,
                            &perlin,
                        );
                        chunk.set(
                            x,
                            y,
                            z,
                            true,
                            self.gradient.at((noise_y / 256.0) as f32).to_rgba8(),
                        );
                    }
                }
            }
        }

        if has_voxels {
            let mut chunk_mesh = ChunkMesh::new(chunk);
            chunk_mesh.remesh();
            chunk_mesh.allocate(&self.device);

            let chunk = Arc::new((chunk_pos, chunk_mesh));
            self.chunks.insert(chunk_pos);

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

            Some((heights, chunk))
        } else {
            None
        }
    }

    fn get_cached_height(&mut self, x: i32, z: i32) -> usize {
        *self
            .height_cache
            .entry((x, z))
            .or_insert_with(|| heightmap(self.seed, x, z))
    }

    fn unload_chunk(&mut self, chunk_pos: Vector3<i32>) {
        self.chunks.remove(&chunk_pos);

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                self.height_cache.remove(&(
                    chunk_pos.x * CHUNK_SIZE as i32 + x as i32,
                    chunk_pos.z * CHUNK_SIZE as i32 + z as i32,
                ));
            }
        }

        self.height_bounds_cache.remove(&(chunk_pos.x, chunk_pos.z));
    }
}

fn heightmap(seed: u32, x: i32, z: i32) -> usize {
    let perlin = Perlin::new(seed);

    const SCALE: f64 = 0.001;
    const HEIGHT_MULTIPLIER: f64 = 100.0;
    const OCTAVES: u32 = 4;
    const PERSISTENCE: f64 = 0.5;
    const DETAIL_SCALE: f64 = 2.0;

    let mut amplitude = 1.0;
    let mut frequency = 3.0;
    let mut height = 0.0;

    let flatness = perlin.get([x as f64 * 0.001, z as f64 * 0.001]);

    for _ in 0..OCTAVES {
        height +=
            perlin.get([x as f64 * SCALE * frequency, z as f64 * SCALE * frequency]) * amplitude;

        let weirdness = perlin.get([x as f64 * SCALE * frequency, z as f64 * SCALE * frequency]);

        let pv = 1.0 - (3.0 * weirdness.abs() - 2.0).abs();

        height += pv * amplitude * (flatness * 1.2);

        amplitude *= PERSISTENCE;
        frequency *= 2.0;
    }

    let continental = perlin.get([x as f64 * 0.0001, z as f64 * 0.0001]);
    let detail = perlin.get([x as f64 * 0.09, z as f64 * 0.09]);

    let height =
        ((height + 1.0) * (HEIGHT_MULTIPLIER * continental) + (detail * DETAIL_SCALE)) * flatness;

    height.clamp(0.0, (MAX_STACKED_CHUNKS * CHUNK_SIZE - 1) as f64) as usize
}

#[allow(clippy::too_many_arguments)]
fn calculate_noise(
    x: i32,
    y: i32,
    z: i32,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    noise_intensity: f64,
    perlin: &Perlin,
) -> f64 {
    let noise_y = min_y as f64 + y as f64;
    let height_difference = noise_y / 30.0;

    if noise_y >= 32.0 + noise_intensity * height_difference {
        noise_y
            + perlin.get([
                (min_x + x) as f64 * 0.1,
                (min_y + y) as f64 * 0.1,
                (min_z + z) as f64 * 0.1,
            ]) * noise_intensity
                * height_difference
    } else {
        noise_y
    }
}
