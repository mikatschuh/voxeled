const distance_fog: f32 = 60.0;
const COLOR_SHIFT: f32 = 0.001;
const SKY_COLOR: vec3<f32> = vec3<f32>(0.2, 0.5, 0.7);

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

@group(0) @binding(0)
var prev_img: texture_2d<f32>;
@group(0) @binding(1)
var prev_img_s: sampler;

@group(1) @binding(0)
var depth_img: texture_depth_2d;
@group(1) @binding(1)
var depth_img_s: sampler;

@fragment
fn post_processing(in: PostProcessingOutput) -> @location(0) vec4<f32> {
    let pos = in.tex_coords;
    let depth = textureSample(depth_img, depth_img_s, pos);
    let color = textureSample(prev_img, prev_img_s, pos).rgb;

    return vec4<f32>(apply_effects(pos, color, depth), 1.0);
}

fn apply_effects(pos: vec2<f32>, color: vec3<f32>, depth: f32) -> vec3<f32> {
    if depth == 1.0 { return SKY_COLOR; }
    // FXAA implementation
    let pixelSize = 1.0 / vec2<f32>(textureDimensions(prev_img, 0));

    // Sample neighboring pixels
    var center = color;
    let lumaCenter = rgb_to_luma(center);

    let lumaUp = rgb_to_luma(textureSample(prev_img, prev_img_s, pos + vec2<f32>(0.0, -pixelSize.y)).rgb);
    let lumaDown = rgb_to_luma(textureSample(prev_img, prev_img_s, pos + vec2<f32>(0.0, pixelSize.y)).rgb);
    let lumaLeft = rgb_to_luma(textureSample(prev_img, prev_img_s, pos + vec2<f32>(-pixelSize.x, 0.0)).rgb);
    let lumaRight = rgb_to_luma(textureSample(prev_img, prev_img_s, pos + vec2<f32>(pixelSize.x, 0.0)).rgb);

    // Detect local contrast - is there an edge here?
    let lumaMin = min(lumaCenter, min(min(lumaUp, lumaDown), min(lumaLeft, lumaRight)));
    let lumaMax = max(lumaCenter, max(max(lumaUp, lumaDown), max(lumaLeft, lumaRight)));
    let lumaRange = lumaMax - lumaMin;

    // Early exit if not an edge
    if !(lumaRange < max(0.0312, lumaMax * 0.125)) {
        // Find the direction of the edge
        let horzLuma = lumaLeft + lumaRight;
        let vertLuma = lumaUp + lumaDown;

        let isHorizontal = horzLuma > vertLuma;

        // Get additional samples to refine edge detection
        // WGSL nutzt select() statt ternärer Operatoren
        let samplingDirection = vec2<f32>(
            select(1.0, 0.0, isHorizontal), // x = 0 wenn horizontal, sonst 1
            select(0.0, 1.0, isHorizontal)  // y = 1 wenn horizontal, sonst 0
        );

        // Sampling step distance depends on the edge length
        let stepLength = 0.5;
        let oppositeLuma1 = rgb_to_luma(image(pos + samplingDirection * pixelSize * stepLength));
        let oppositeLuma2 = rgb_to_luma(image(pos - samplingDirection * pixelSize * stepLength));

        // Blend between original and anti-aliased sample based on edge significance
        let blendFactor = 0.6; // How strong the anti-aliasing effect is
        let edgeStrength = abs(oppositeLuma1 + oppositeLuma2 - 2.0 * lumaCenter) / lumaRange;

        // Apply simple anti-aliasing - blend with neighbors based on edge strength
        let sampleOffset = samplingDirection * pixelSize * blendFactor;
        let sample1 = image(pos + sampleOffset);
        let sample2 = image(pos - sampleOffset);

        let blendWeight = clamp(edgeStrength, 0.0, 0.5);
        center = vec3<f32>(mix(center, (sample1 + sample2) * 0.5, blendWeight));
    }

    // Now apply our CRT/retro effects to the anti-aliased image

    // Create a retro CRT-like effect with scan lines and vignette

    // Calculate scan lines (horizontal lines)
    let scan_line = sin(pos.y * 120.0) * 0.08 + 0.92;

    // Add some color distortion/aberration (RGB shift)
    let r = textureSample(prev_img, prev_img_s, pos + vec2<f32>(COLOR_SHIFT, 0.0)).r;
    let g = color.g;
    let b = textureSample(prev_img, prev_img_s, pos - vec2<f32>(COLOR_SHIFT, 0.0)).b;

    // Create vignette effect (darker at the edges)
    let screenCenter = vec2<f32>(0.5, 0.5); // Mittelpunkt des Bildschirms für Vignette
    let dist = distance(pos, screenCenter);
    let vignette = smoothstep(0.5, 0.2, dist - 0.25) * 0.85 + 0.15;

    // Enhance contrast slightly
    let contrast = 1.1;
    let mid = 0.5;
    let contrast_color = vec3<f32>(
        (r - mid) * contrast + mid,
        (g - mid) * contrast + mid,
        (b - mid) * contrast + mid
    );

    // Combine all effects
    let final_color = contrast_color  * vignette;

    // Add a subtle blue-green tint to give it a retro computing feel
    let tint = vec3<f32>(0.95, 1.05, 1.05);
    let linearized_depth = linearize_depth(depth, 0.001, 1000.0) * distance_fog;
    // 1 = x^2 + y^2 => 1 - x^2 = y^2 => -x^2 = y^2 - 1 => x^2 = 1-y^2 => x = √(1-y^2)
    let fog = min(linearized_depth, 1.0); // sqrt(1.0 - linearized_depth * linearized_depth);

    return mix(final_color * tint, SKY_COLOR, 1.0 - fog);
}
// Helpers for FXAA
fn rgb_to_luma(rgb: vec3<f32>) -> f32 {
    // Convert RGB to brightness using standard coefficients
    return dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
}
fn image(pos: vec2<f32>) -> vec3<f32> {
    return textureSample(prev_img, prev_img_s, pos).rgb;
}

fn color(prev: f32) -> vec3<f32> {
    // Farbverlauf basierend auf Sinuswellen
    let r = 0.5 + 0.5 * cos(6.2831 * prev + 0.0);
    let g = 0.5 + 0.5 * cos(6.2831 * prev + 2.0);
    let b = 0.5 + 0.5 * cos(6.2831 * prev + 4.0);

    return vec3<f32>(r, g, b);
}
fn linearize_depth(depth: f32, near: f32, far: f32) -> f32 {
    let z = depth * 2.0 - 1.0; // Depth von [0,1] nach [-1,1] für NDC transformieren
    return (far + near - z * (far - near)) / (near * far); // Invertierte Berechnung
}
