// Constants chosen from the reference FXAA implementation.
const FXAA_SPAN_MAX: f32 = 8.0;
const FXAA_REDUCE_MIN: f32 = 1.0 / 128.0;
const FXAA_REDUCE_MUL: f32 = 1.0 / 8.0;

 fn fxaa(pos: vec2<f32>, color: vec3<f32>) -> vec3<f32> {
   let pixel_size = 1.0 / vec2<f32>(textureDimensions(prev_img, 0));

    // Luma of the current pixel and its diagonals.
    let luma_m = rgb_to_luma(color);
    let luma_nw = rgb_to_luma(image(pos + vec2<f32>(-pixel_size.x, -pixel_size.y)));
    let luma_ne = rgb_to_luma(image(pos + vec2<f32>(pixel_size.x, -pixel_size.y)));
    let luma_sw = rgb_to_luma(image(pos + vec2<f32>(-pixel_size.x, pixel_size.y)));
    let luma_se = rgb_to_luma(image(pos + vec2<f32>(pixel_size.x, pixel_size.y)));

    let luma_min = min(luma_m, min(min(luma_nw, luma_ne), min(luma_sw, luma_se)));
    let luma_max = max(luma_m, max(max(luma_nw, luma_ne), max(luma_sw, luma_se)));

    // Early out if no edge is detected.
    if luma_max - luma_min < max(FXAA_REDUCE_MIN, luma_max * 0.03125) {
        return color;
    }

    // Estimate edge direction.
    var dir = vec2<f32>(
        (luma_sw + luma_se) - (luma_nw + luma_ne),
        (luma_nw + luma_sw) - (luma_ne + luma_se),
    );

    let dir_reduce =
        max((luma_nw + luma_ne + luma_sw + luma_se) * (0.25 * FXAA_REDUCE_MUL), FXAA_REDUCE_MIN);
    let inv_dir_min = 1.0 / (min(abs(dir.x), abs(dir.y)) + dir_reduce);
    dir = clamp(dir * inv_dir_min, vec2<f32>(-FXAA_SPAN_MAX, -FXAA_SPAN_MAX), vec2<f32>(FXAA_SPAN_MAX, FXAA_SPAN_MAX));
    dir *= pixel_size;

    // Blend samples along the edge.
    let rgb_a = 0.5 * (image(pos + dir * (1.0 / 3.0 - 0.5)) + image(pos + dir * (2.0 / 3.0 - 0.5)));
    let rgb_b = rgb_a * 0.5 + 0.25 * (image(pos + dir * -0.5) + image(pos + dir * 0.5));
    let luma_b = rgb_to_luma(rgb_b);

    if luma_b < luma_min || luma_b > luma_max {
        return rgb_a;
    }

    return rgb_b;
}
