struct PostProcessingOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn full_screen_quat(@builtin(vertex_index) vertex_index: u32) -> PostProcessingOutput {
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
    output.tex_coords = uv;
    return output;
}
// UVCoord Struktur nicht mehr benötigt, da wir jetzt VertexOutput verwenden
@group(0) @binding(0)
var prev_img: texture_2d<f32>;
@group(0) @binding(1)
var prev_img_s: sampler;

@fragment
fn apply_effects(in: PostProcessingOutput) -> @location(0) vec4<f32> {
    let color = textureSample(prev_img, prev_img_s, in.tex_coords);

    return vec4<f32>(1.0 - color.x, 1.0 - color.y, 1.0 - color.z, color.w);
}
