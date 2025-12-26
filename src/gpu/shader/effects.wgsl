const distance_fog: f32 = 0.8;
const SKY_COLOR: vec3<f32> = vec3(0.0); //vec3(0.2, 0.5, 0.7);

fn apply_effects(pos: vec2<f32>, color: vec3<f32>, depth: f32) -> vec3<f32> {
    if depth == 1.0 { return SKY_COLOR; }

    let anti_aliased = fxaa(pos, color);

    // Create vignette effect (darker at the edges)
    let screenCenter = vec2<f32>(0.5, 0.5); // Mittelpunkt des Bildschirms für Vignette
    let dist = distance(pos, screenCenter);
    let vignette = smoothstep(0.5, 0.2, dist - 0.25) * 0.85 + 0.15;

    // Combine all effects
    let final_color = anti_aliased * sqrt(vignette);

    // let linearized_depth = linearize_depth(depth, 0.1, 10_000.0);
    let fog = depth * distance_fog;

    return mix(final_color, SKY_COLOR, fog);
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
    let z = depth * 2.0 - 1.0; // depth from [0,1] into clip space [-1,1]
    let view_z = (2.0 * near) / (far + near - z * (far - near)); // view-space distance
    return clamp((view_z - near) / (far - near), 0.0, 1.0); // 0 at near, 1 at far
}
