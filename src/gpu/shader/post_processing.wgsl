struct PostProcessingOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex fn full_screen_quat(@builtin(vertex_index) vertex_index: u32) -> PostProcessingOutput {
    var uv_coords: array<vec2<f32>, 6>;
    uv_coords[0] = vec2<f32>(0.0, 0.0);
    uv_coords[1] = vec2<f32>(1.0, 0.0);
    uv_coords[2] = vec2<f32>(0.0, 1.0);
    uv_coords[3] = vec2<f32>(0.0, 1.0);
    uv_coords[4] = vec2<f32>(1.0, 0.0);
    uv_coords[5] = vec2<f32>(1.0, 1.0);

    let uv = uv_coords[vertex_index];
    var output: PostProcessingOutput;
    output.position = vec4<f32>(2.0 * uv.x - 1.0, 2.0 * uv.y - 1.0, 0.0, 1.0);
    // output.tex_coords = uv;
    output.tex_coords = vec2<f32>(uv.x, 1.0 - uv.y);
    return output;
}

@group(0) @binding(0) var prev_img: texture_2d<f32>;
@group(0) @binding(1) var prev_img_s: sampler;

@group(1) @binding(0) var depth_img: texture_depth_2d;
@group(1) @binding(1) var depth_img_s: sampler;

@fragment fn post_processing(in: PostProcessingOutput) -> @location(0) vec4<f32> {
    let pos = in.tex_coords;
    let depth_sample = textureSample(depth_img, depth_img_s, pos);
    let depth = linearize_depth(depth_sample, 0.1, 10000.0);
    let color = textureSample(prev_img, prev_img_s, pos).rgb;

    return vec4<f32>(apply_effects(pos, color, depth), 1.0);
}

fn linearize_depth(depth: f32, near: f32, far: f32) -> f32 {
    // WGPU uses 0..1 NDC depth (D3D/Vulkan), so no remap to [-1,1].
    return (near * far) / (far - depth * (far - near)); // view-space distance
}
