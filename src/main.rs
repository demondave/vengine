use std::time::Instant;

use cgmath::{Point3, Vector3, Zero};
use colorgrad::{preset::turbo, Gradient};
use engine::{
    object::{chunk::Chunk, Object},
    Engine,
};
use input::Input;
use wgpu::{Backends, Instance, InstanceDescriptor};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub mod engine;
pub mod input;

pub fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window: &'static Window =
        Box::leak(Box::new(WindowBuilder::new().build(&event_loop).unwrap()));

    let size = window.inner_size();

    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::PRIMARY,
        ..Default::default()
    });

    let surface = instance.create_surface(window).unwrap();

    let engine = pollster::block_on(Engine::new(surface, instance, size.width, size.height));

    let engine: &'static Engine = Box::leak(Box::new(engine));

    std::thread::spawn(move || {
        start(engine);
    });

    let input = Input::new(event_loop, window, engine);
    input.run();
}

fn start(engine: &Engine) {
    engine.palette().set_palette(palette(&turbo()));

    engine.camera().set_eye(Point3::new(-3.0, 0.5, -3f32));
    engine
        .camera()
        .set_look_at(Point3::new(0.5f32, 0.5f32, -0.5f32));

    let mut object = Object::new(engine.device(), Vector3::zero());

    let mut chunk = Chunk::empty(Vector3::new(1f32, 0f32, 0f32));
    chunk.set(0, 0, 0, true);
    chunk.set(0, 1, 0, true);
    chunk.remesh();
    chunk.allocate(engine.device());

    object.add_chunk(Vector3::zero(), chunk);

    engine.add(object);

    loop {
        let start = Instant::now();
        engine.render().unwrap();
        let end = Instant::now();
        println!("{:.3}", (end - start).as_secs_f64() * 1000.0)
    }
}

fn palette(gradient: &impl Gradient) -> [[f32; 4]; 128] {
    let mut buffer = [[0f32; 4]; 128];

    let diff = 1f32 / 128f32;
    let mut n = 0f32;

    for idx in 0..128 {
        buffer[idx].copy_from_slice(&gradient.at(n).to_linear_rgba());
        n += diff;
    }

    buffer
}
