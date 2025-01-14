use std::sync::Arc;

use crossbeam::atomic::AtomicCell;
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue};

pub struct Palette {
    palette_buffer: Buffer,
    palette_bind_group_layout: BindGroupLayout,
    palette_bind_group: BindGroup,
    palette: AtomicCell<[[f32; 4]; 128]>,
    queue: Arc<Queue>,
}

impl Palette {
    pub fn new(device: &Device, queue: Arc<Queue>) -> Palette {
        let palette_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vengine::voxel_palette"),
            contents: bytemuck::cast_slice(&[[1f32, 0f32, 0f32, 1f32]; 128]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let palette_bind_group_layout =
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
                label: Some("vengine::voxel_palette_bind_group_layout"),
            });

        let palette_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &palette_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: palette_buffer.as_entire_binding(),
            }],
            label: Some("vengine::voxel_palette_bind_group"),
        });

        Palette {
            palette_buffer,
            palette_bind_group_layout,
            palette_bind_group,
            palette: AtomicCell::new([[1f32, 0f32, 0f32, 1f32]; 128]),
            queue,
        }
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.palette_bind_group
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.palette_bind_group_layout
    }

    pub fn get_palette(&self) -> [[f32; 4]; 128] {
        self.palette.load()
    }

    pub fn set_palette(&self, palette: [[f32; 4]; 128]) {
        self.queue
            .write_buffer(&self.palette_buffer, 0, bytemuck::cast_slice(&palette));
        self.palette.store(palette);
    }
}
