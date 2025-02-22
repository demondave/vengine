use super::Pipeline;
use crate::engine::{
    rendering::{backend::Backend, camera::Camera, texture::Texture},
    voxel::quad::Quad,
};
use wgpu::{util::DeviceExt, Buffer, RenderPipeline};

pub struct VoxelPipeline {
    pipeline: RenderPipeline,
    quad: Buffer,
}

impl VoxelPipeline {
    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    pub fn quad(&self) -> &Buffer {
        &self.quad
    }
}

impl Pipeline for VoxelPipeline {
    fn initialize(backend: &Backend<'_>, camera: &Camera) -> Self {
        let render_pipeline_layout =
            backend
                .device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("vengine::voxel_pipeline_layout"),
                    bind_group_layouts: &[camera.bind_group_layout()],
                    push_constant_ranges: &[wgpu::PushConstantRange {
                        stages: wgpu::ShaderStages::VERTEX,
                        range: 0..(size_of::<[f32; 4 * 4]>() + size_of::<[i32; 3]>()) as u32,
                    }],
                });

        let shader = backend
            .device()
            .create_shader_module(wgpu::include_wgsl!("shaders/base.wgsl"));

        let pipeline = backend
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("vengine::voxel_pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[vertex_desc(), instance_desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: *backend.surface_format(),
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        let quad = backend
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vengine::voxel_quad"),
                contents: bytemuck::cast_slice(&[
                    [0.0f32, 0.0f32, -1.0f32],
                    [0.0f32, 0.0f32, 0.0f32],
                    [1.0f32, 0.0f32, -1.0f32],
                    [1.0f32, 0.0f32, 0.0f32],
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        Self { pipeline, quad }
    }
}

fn vertex_desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            shader_location: 0,
            format: wgpu::VertexFormat::Float32x3,
        }],
    }
}

pub fn instance_desc() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Quad>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 1,
                format: wgpu::VertexFormat::Uint32,
            },
            wgpu::VertexAttribute {
                offset: 4,
                shader_location: 2,
                format: wgpu::VertexFormat::Uint32,
            },
        ],
    }
}
