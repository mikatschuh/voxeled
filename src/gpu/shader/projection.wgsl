struct CameraUniform {
    view_proj: mat4x4<f32>
};
struct ChunkMetadata {
    pos_lod: vec4<i32>,
}

@group(0) @binding(0) var tex_array: texture_2d_array<f32>;
@group(0) @binding(1) var smp: sampler;

@group(1) @binding(0) var<uniform> camera: CameraUniform;

var<push_constant> chunk_metadata: ChunkMetadata;

struct VertexInput {
    @location(0) kind: u32
}
struct InstanceInput {
    @location(1) kind: u32
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @interpolate(flat) @location(1) texture_index: u32, // contains the texture index
    @interpolate(flat) @location(2) orientation: u32,
}

@vertex fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var vertex_position: vec3<f32>;

    let orientation = instance.kind >> 29u;
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
    var out: VertexOutput;

    if model.kind == 0u {
        out.tex_coords = vec2(1.0, 0.0);
    } else if model.kind == 1u {
        out.tex_coords = vec2(0.0, 0.0);
    } else if model.kind == 2u {
        out.tex_coords = vec2(0.0, 1.0);
    } else {
        out.tex_coords = vec2(1.0, 1.0);
    }

    out.orientation = orientation;
    out.texture_index = instance.kind & 16383;
    let lod = u32(chunk_metadata.pos_lod.w);
    out.clip_position = camera.view_proj * vec4<f32>(
        (vec3<f32>(
            f32(chunk_metadata.pos_lod.x),
            f32(chunk_metadata.pos_lod.y),
            f32(chunk_metadata.pos_lod.z)
        ) * 32
        + vec3<f32>(
            f32((instance.kind >> 24) & 31u),
            f32((instance.kind >> 19) & 31u),
            f32((instance.kind >> 14) & 31u)
        ) + vertex_position) * f32(1u << lod),
        1.0
    );
    return out;
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(tex_array, smp, in.tex_coords, in.texture_index);

    if color.a == 0.0 {
        discard;
    }
    var shading: f32;
    if in.orientation == 0u {
        shading = 0.8;
    } else if in.orientation == 1u {
        shading = 0.3;
    } else if in.orientation == 2u {
        shading = 0.5;
    } else if in.orientation == 3u {
        shading = 0.5;
    } else if in.orientation == 4u {
        shading = 0.5;
    } else if in.orientation == 5u {
        shading = 0.5;
    }

    /*var tint: vec3<f32>;
    if in.lod_level == 0 {
        tint = vec3(2., 0.5, 0.5);
    } else if in.lod_level == 1 {
        tint = vec3(0.5, 2., 0.5);
    } else {
        tint = vec3(0.5, 0.5, 2.);
    }*/

    return vec4<f32>(shading * color.rgb, color.a);
}
