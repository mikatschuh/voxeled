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
    // Sample the color at the current UV coordinate
    let color = textureSample(prev_img, prev_img_s, in.tex_coords);
    
    // Create a retro CRT-like effect with scan lines and vignette
    
    // Calculate scan lines (horizontal lines)
    let scan_line = sin(in.tex_coords.y * 120.0) * 0.08 + 0.92;
    
    // Add some color distortion/aberration (RGB shift)
    let r = textureSample(prev_img, prev_img_s, in.tex_coords + vec2<f32>(0.005, 0.0)).r;
    let g = color.g;
    let b = textureSample(prev_img, prev_img_s, in.tex_coords - vec2<f32>(0.005, 0.0)).b;
    
    // Create vignette effect (darker at the edges)
    let center = vec2<f32>(0.5, 0.5);
    let dist = distance(in.tex_coords, center);
    let vignette = smoothstep(0.5, 0.2, dist) * 0.85 + 0.15;
    
    // Enhance contrast slightly
    let contrast = 1.1;
    let mid = 0.5;
    let contrast_color = vec3<f32>(
        (r - mid) * contrast + mid, 
        (g - mid) * contrast + mid, 
        (b - mid) * contrast + mid
    );
    
    // Combine all effects
    let final_color = contrast_color * scan_line * vignette;
    
    // Add a subtle blue-green tint to give it a retro computing feel
    let tint = vec3<f32>(0.95, 1.05, 1.05);
    
    return vec4<f32>(final_color * tint, color.a);
}
