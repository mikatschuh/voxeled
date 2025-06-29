struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0) // 1.
var<uniform> camera: CameraUniform;

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
    @location(1) kind: u32,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

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
    if instance.kind == 0u {                            // 0
        if model.kind == 0u {
            vertex_position = vec3(0.0, 0.0, 1.0);
        } else if model.kind == 1u {
            vertex_position = vec3(0.0, 0.0, 0.0);
        } else if model.kind == 2u {
            vertex_position = vec3(0.0, 1.0, 0.0);
        } else { // model.kind == 3u
            vertex_position = vec3(0.0, 1.0, 1.0);
        }
    } else if instance.kind == 1u {                     // 1
        if model.kind == 0u {
            vertex_position = vec3(1.0, 0.0, 0.0);
        } else if model.kind == 1u {
            vertex_position = vec3(1.0, 0.0, 1.0);
        } else if model.kind == 2u {
            vertex_position = vec3(1.0, 1.0, 1.0);
        } else { // model.kind == 3u
            vertex_position = vec3(1.0, 1.0, 0.0);
        }
    } else if instance.kind == 2u {                     // 2
        if model.kind == 0u {
            vertex_position = vec3(0.0, 0.0, 1.0);
        } else if model.kind == 1u {
            vertex_position = vec3(1.0, 0.0, 1.0);
        } else if model.kind == 2u {
            vertex_position = vec3(1.0, 0.0, 0.0);
        } else { // model.kind == 3u
            vertex_position = vec3(0.0, 0.0, 0.0);
        }
    } else if instance.kind == 3u {                     // 3
        if model.kind == 0u {
            vertex_position = vec3(0.0, 1.0, 0.0);
        } else if model.kind == 1u {
            vertex_position = vec3(1.0, 1.0, 0.0);
        } else if model.kind == 2u {
            vertex_position = vec3(1.0, 1.0, 1.0);
        } else { // model.kind == 3u
            vertex_position = vec3(0.0, 1.0, 1.0);
        }
    } else if instance.kind == 4u {                     // 4
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
    out.kind = instance.kind;
    out.clip_position = camera.view_proj * vec4<f32>(
        vec3<f32>(f32(instance.position.x), f32(instance.position.y), f32(instance.position.z)) + vertex_position,
        1.0
    );
    return out;
}
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    if color.a == 0.0 {
        discard;
    }
    return color;
}
