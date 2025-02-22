// Camera
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct PushConstant {
    transform: mat4x4<f32>,
    offset: vec3<i32>
}

var<push_constant> pc: PushConstant;


struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct InstanceInput {
    @location(1) low: u32, 
    @location(2) color: u32, 
};

const CHUNK_SIZE: f32 = 32.0;
const VOXEL_SIZE: f32 = 1.0;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    var position_x: u32 = instance.low & 63u;
    var position_y: u32 = (instance.low >> 6u) & 63u;
    var position_z: u32 = (instance.low >> 12u) & 63u;
    var direction: u32 = (instance.low >> 18u) & 7u;

    var position: vec3<f32> = model.position;

    switch direction {
        // Left
        case 0u: {
            position = vec3(position.y + 1.0, -position.x + 1.0, position.z);
        }
         // Right
        case 1u: {
            position = vec3(position.y, position.x, position.z);
        }
        // Up
        case 2u: {
            position.y += 1.0;
        }
        // Down
        case 3u: {
            position = vec3(position.x, position.y, -position.z - 1.0);
        }
        // Front
        case 4u: {
            position = vec3(position.x, position.z + 1.0, position.y - 1.0);
        }
        // Back
        case 5u: {
            position = vec3(position.x, -position.z, -position.y);
        }
        default: {}
    }

    position += vec3(f32(position_x), f32(position_y), f32(position_z)) + (vec3(f32(pc.offset.x), f32(pc.offset.y), f32(pc.offset.z)) * CHUNK_SIZE * VOXEL_SIZE);

    let pos4 = pc.transform * vec4<f32>(position, 1.0);
    position = (pos4.xyz / pos4.w);

    out.color = unpack_color(instance.color);

    // Apply "shading"
    switch direction {
        // Left
        case 0u: {
            out.color = darken_color(out.color, 0.3);
        }
        // Right
        case 1u: {
            out.color = darken_color(out.color, 0.325);
        }
        // Up
        case 2u: {
            out.color = darken_color(out.color, 0.15);
        }
        // Down
        case 3u: {
            out.color = darken_color(out.color, 0.4);
        } 
        // Front
        case 4u: {
            out.color = darken_color(out.color, 0.35);
        }
        // Back
        case 5u: {
            out.color = darken_color(out.color, 0.375);
        }
        default: {}
    }

    out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);

    return out;
}

fn unpack_color(color: u32) -> vec4<f32> {
    let r = f32((color >> 24u) & 0xFFu) / 255.0;
    let g = f32((color >> 16u) & 0xFFu) / 255.0;
    let b = f32((color >> 8u) & 0xFFu) / 255.0;
    let a = f32(color & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

fn darken_color(color: vec4<f32>, factor: f32) -> vec4<f32> {
    return vec4<f32>(color.rgb * (1.0 - factor), color.a);
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}