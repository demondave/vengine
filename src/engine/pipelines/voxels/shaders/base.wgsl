// Camera
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// Palette
struct Palette {
    palette: array<vec4<f32>,128>,
};
@group(1) @binding(0)
var<uniform> palette: Palette;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct InstanceInput {
    @location(1) instance: u32,
};




@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    var position_x: u32 = instance.instance & 63;
    var position_y: u32 = (instance.instance >> 6) & 63;
    var position_z: u32 = (instance.instance >> 12) & 63;
    var direction: u32 = (instance.instance >> 18) & 7;
    var texture_id: u32 = (instance.instance >> 21) & 63;

    var position: vec3<f32> = model.position;

    switch direction {
        // Up
        case 0u: {
            position.y += 1.0;
        }
        // Down
        case 1u: {
            position = vec3(position.x, position.y, -position.z - 1.0);
        }
        // Left
        case 2u: {
            position = vec3(position.y + 1.0, -position.x + 1.0, position.z);
        }
        // Right
        case 3u: {
            position = vec3(position.y, position.x, position.z);
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

    position += vec3(f32(position_x), f32(position_y), f32(position_z));

    out.color = palette.palette[texture_id];

    out.clip_position = camera.view_proj * vec4<f32>(position, 1.0);

    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}