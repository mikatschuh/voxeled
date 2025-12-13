const distance_fog: f32 = 10.0;
const COLOR_SHIFT: f32 = 0.0;
const SKY_COLOR: vec3<f32> = vec3<f32>(0.2, 0.5, 0.7);

fn apply_effects(pos: vec2<f32>, color: vec3<f32>, depth: f32) -> vec3<f32> {
    if depth == 1.0 { return SKY_COLOR; }

    let anti_aliased = fxaa(pos, color);

    // Create vignette effect (darker at the edges)
    let screenCenter = vec2<f32>(0.5, 0.5); // Mittelpunkt des Bildschirms für Vignette
    let dist = distance(pos, screenCenter);
    let vignette = smoothstep(0.5, 0.2, dist - 0.25) * 0.85 + 0.15;

    // Combine all effects
    let final_color = anti_aliased * sqrt(vignette);

    let linearized_depth = (1.0 - linearize_depth(depth, 0.1, 1000.0)) * distance_fog;
    // 1 = x^2 + y^2 => 1 - x^2 = y^2 => -x^2 = y^2 - 1 => x^2 = 1-y^2 => x = √(1-y^2)
    let fog = min(linearized_depth, 1.0);

    return mix(final_color, vec3<f32>(0.0), fog);
}

fn just_fog(pos: vec2<f32>, color: vec3<f32>, depth: f32) -> vec3<f32> {
    let linearized_depth = linearize_depth(depth, 0.001, 1000.0) * distance_fog;
    let fog = min(linearized_depth, 1.0); // sqrt(1.0 - linearized_depth * linearized_depth);
    return mix(color, SKY_COLOR, 1.0 - fog);
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
