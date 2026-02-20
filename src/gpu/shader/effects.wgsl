const VIEW_DISTANCE: f32 = RENDER_DISTANCE * 32.0;
const SKY_COLOR: vec3<f32> = vec3(0.005, 0.006, 0.008); // Deep cave void
const FOG_COLOR_LOW: vec3<f32> = vec3(0.02, 0.025, 0.03);
const FOG_COLOR_HIGH: vec3<f32> = vec3(0.05, 0.055, 0.06);

const FOG_START: f32 = 0.03;
const FOG_END: f32 = 0.85;
const FOG_DENSITY: f32 = 100.0;
const FOG_MAX: f32 = 0.85;

fn apply_effects(pos: vec2<f32>, color: vec3<f32>, depth: f32) -> vec3<f32> {
    let anti_aliased = fxaa(pos, color);

    // Create vignette effect (darker at the edges)
    let screenCenter = vec2<f32>(0.5, 0.5); // Mittelpunkt des Bildschirms für Vignette
    let center_dst = distance(pos, screenCenter);
    let vignette = smoothstep(0.5, 0.2, center_dst - 0.25) * 0.85 + 0.15;

    // Combine all effects
    let final_color = anti_aliased * sqrt(vignette);

    // depth is view-space distance, shape against render distance.
    let view_dist = max(VIEW_DISTANCE, 1.0);
    let fog_start = view_dist * FOG_START;
    let fog_end = view_dist * FOG_END;
    let clamped_depth = min(depth, view_dist);
    let fog_t = clamp((clamped_depth - fog_start) / (fog_end - fog_start), 0.0, 1.0);
    let fog = min(1.0 - exp2(-fog_t * fog_t * FOG_DENSITY), FOG_MAX);
    let fog_color = mix(FOG_COLOR_LOW, FOG_COLOR_HIGH, clamp(pos.y * 0.8 + 0.1, 0.0, 1.0));

    let base = mix(final_color, fog_color, fog);
    let over = clamp((depth - view_dist) / (view_dist * 0.05), 0.0, 1.0);
    return mix(base, SKY_COLOR, over);
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
