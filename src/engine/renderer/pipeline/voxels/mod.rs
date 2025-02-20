use wgpu::{Device, RenderPipeline, TextureFormat};

use crate::engine::{
    renderer::{camera::Camera, texture::Texture},
    voxel::quad::Quad,
};

pub fn voxel_pipeline(device: &Device, camera: &Camera, format: TextureFormat) -> RenderPipeline {
    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("vengine::voxel_pipeline_layout"),
        bind_group_layouts: &[camera.bind_group_layout()],
        push_constant_ranges: &[wgpu::PushConstantRange {
            stages: wgpu::ShaderStages::VERTEX,
            range: 0..(size_of::<[f32; 4 * 4]>() + size_of::<[i32; 3]>()) as u32,
        }],
    });

    let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/base.wgsl"));

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("vengine::voxel_pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),               // 1.
            buffers: &[vertex_desc(), instance_desc()], // 2.
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            // 3.
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                // 4.
                format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip, // 1.
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw, // 2.
            cull_mode: Some(wgpu::Face::Back),
            // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less, // 1.
            stencil: wgpu::StencilState::default(),     // 2.
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,                         // 2.
            mask: !0,                         // 3.
            alpha_to_coverage_enabled: false, // 4.
        },
        multiview: None, // 5.
        cache: None,     // 6.
    })
}

pub fn vertex_desc() -> wgpu::VertexBufferLayout<'static> {
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
                offset: 4, // 4 bytes offset für den zweiten u32 Wert
                shader_location: 2,
                format: wgpu::VertexFormat::Uint32,
            },
        ],
    }
}
