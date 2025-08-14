struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> orientation: u32;

struct VertexInput {
    @location(0) kind: u32
}
struct InstanceInput {
    @location(2) position: vec3<i32>,
    @location(3) kind: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) texture_index: u32, // contains the texture index
    @location(2) height: f32,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.texture_index = instance.kind;

    if model.kind == 0u {
        out.tex_coords = vec2(0.0, 0.0);
    } else if model.kind == 1u {
        out.tex_coords = vec2(0.0, 1.0);
    } else if model.kind == 2u {
        out.tex_coords = vec2(1.0, 1.0);
    } else {
        out.tex_coords = vec2(1.0, 0.0);
    }

    var vertex_position: vec3<f32>;
    if orientation == 0u {                            // 0
        if model.kind == 0u {
            vertex_position = vec3(0.0, 0.0, 1.0);
        } else if model.kind == 1u {
            vertex_position = vec3(0.0, 0.0, 0.0);
        } else if model.kind == 2u {
            vertex_position = vec3(0.0, 1.0, 0.0);
        } else { // model.kind == 3u
            vertex_position = vec3(0.0, 1.0, 1.0);
        }
    } else if orientation == 1u {                     // 1
        if model.kind == 0u {
            vertex_position = vec3(1.0, 0.0, 0.0);
        } else if model.kind == 1u {
            vertex_position = vec3(1.0, 0.0, 1.0);
        } else if model.kind == 2u {
            vertex_position = vec3(1.0, 1.0, 1.0);
        } else { // model.kind == 3u
            vertex_position = vec3(1.0, 1.0, 0.0);
        }
    } else if orientation == 2u {                     // 2
        if model.kind == 0u {
            vertex_position = vec3(0.0, 0.0, 1.0);
        } else if model.kind == 1u {
            vertex_position = vec3(1.0, 0.0, 1.0);
        } else if model.kind == 2u {
            vertex_position = vec3(1.0, 0.0, 0.0);
        } else { // model.kind == 3u
            vertex_position = vec3(0.0, 0.0, 0.0);
        }
    } else if orientation == 3u {                     // 3
        if model.kind == 0u {
            vertex_position = vec3(0.0, 1.0, 0.0);
        } else if model.kind == 1u {
            vertex_position = vec3(1.0, 1.0, 0.0);
        } else if model.kind == 2u {
            vertex_position = vec3(1.0, 1.0, 1.0);
        } else { // model.kind == 3u
            vertex_position = vec3(0.0, 1.0, 1.0);
        }
    } else if orientation == 4u {                     // 4
        if model.kind == 0u {
            vertex_position = vec3(0.0, 0.0, 0.0);
        } else if model.kind == 1u {
            vertex_position = vec3(1.0, 0.0, 0.0);
        } else if model.kind == 2u {
            vertex_position = vec3(1.0, 1.0, 0.0);
        } else { // model.kind == 3u
            vertex_position = vec3(0.0, 1.0, 0.0);
        }
    } else {                                            // 5
        if model.kind == 0u {
            vertex_position = vec3(1.0, 0.0, 1.0);
        } else if model.kind == 1u {
            vertex_position = vec3(0.0, 0.0, 1.0);
        } else if model.kind == 2u {
            vertex_position = vec3(0.0, 1.0, 1.0);
        } else { // model.kind == 3u
            vertex_position = vec3(1.0, 1.0, 1.0);
        }
    }
    out.height = f32(instance.position.y);
    out.clip_position = camera.view_proj * vec4<f32>(
        vec3<f32>(f32(instance.position.x), f32(instance.position.y), f32(instance.position.z)) + vertex_position,
        1.0
    );
    return out;
}
@group(0) @binding(0)
var tex_array: texture_2d_array<f32>;
@group(0) @binding(1)
var smp: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(tex_array, smp, in.tex_coords, in.texture_index);

    if color.a == 0.0 {
        discard;
    }
    var shading: f32;
    if orientation == 0 {
        shading = 1.0;
    } else if orientation == 1 {
        shading = 0.6;
    } else if orientation == 2 {
        shading = 0.8;
    } else if orientation == 3 {
        shading = 0.8;
    } else if orientation == 4 {
        shading = 0.8;
    } else if orientation == 5 {
        shading = 0.8;
    }
    return vec4<f32>(shading * color.rgb * min(1.0 / (abs(in.height) * 0.001), 1.0), color.a);
}
