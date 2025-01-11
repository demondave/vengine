use cgmath::{Point3, Vector3};
use crossbeam::atomic::AtomicCell;
use std::sync::Arc;
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    eye: AtomicCell<Point3<f32>>,
    target: AtomicCell<Point3<f32>>,
    up: Vector3<f32>,
    aspect: AtomicCell<f32>,
    fovy: f32,
    znear: f32,
    zfar: f32,
    camera_uniform: AtomicCell<CameraUniform>,
    camera_bind_group: BindGroup,
    camera_buffer: Buffer,
    camera_bind_group_layout: BindGroupLayout,
    queue: Arc<Queue>,
}

impl Camera {
    pub fn new(pos: Point3<f32>, aspect: f32, device: &Device, queue: Arc<Queue>) -> Self {
        let camera_uniform = CameraUniform::new();

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vengine::camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("vengine::camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("vengine::camera_bind_group"),
        });

        let camera = Camera {
            eye: AtomicCell::new(pos),
            target: AtomicCell::new((0.0, 0.0, 0.0).into()),
            up: cgmath::Vector3::unit_y(),
            aspect: AtomicCell::new(aspect),
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
            camera_uniform: AtomicCell::new(camera_uniform),
            camera_bind_group,
            camera_buffer,
            camera_bind_group_layout,
            queue,
        };

        camera.camera_uniform.store(CameraUniform {
            view_proj: camera.build_view_projection_matrix().into(),
        });

        camera
    }

    pub fn update(&self) {
        self.camera_uniform.store(CameraUniform {
            view_proj: self.build_view_projection_matrix().into(),
        });

        let tmp = self.camera_uniform.load();

        self.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[tmp]));
    }

    pub fn set_aspect(&self, aspect: f32) {
        self.aspect.store(aspect);
    }

    pub fn get_look_at(&self) -> Point3<f32> {
        self.target.load()
    }

    pub fn set_look_at(&self, n: Point3<f32>) {
        self.target.store(n);
        self.update();
    }

    pub fn set_look_at_no_update(&self, n: Point3<f32>) {
        self.target.store(n);
        self.update();
    }

    pub fn get_eye(&self) -> Point3<f32> {
        self.eye.load()
    }

    pub fn set_eye(&self, n: Point3<f32>) {
        self.eye.store(n);
        self.update();
    }

    pub fn set_eye_no_update(&self, n: Point3<f32>) {
        self.eye.store(n);
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.camera_bind_group
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.camera_bind_group_layout
    }

    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye.load(), self.target.load(), self.up);
        // 2.
        let proj = cgmath::perspective(
            cgmath::Deg(self.fovy),
            self.aspect.load(),
            self.znear,
            self.zfar,
        );

        // 3.
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
