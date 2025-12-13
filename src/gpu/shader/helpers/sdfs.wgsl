struct PhysicalObject {
    pos: vec3<f32>,
    scale: vec3<f32>,
    rot: vec3<f32>
};

fn square(pos: vec2<f32>, size: vec2<f32>, p: vec2<f32>) -> f32 {
    let d = abs(p - pos) - size * 0.5;

    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

fn sphere(pos: vec3<f32>, r: f32, p: vec3<f32>) -> f32 { return length(pos - p) - r; }

fn cube(pos: vec3<f32>, size: vec3<f32>, p: vec3<f32>) -> f32 {
    let d = abs(p - pos) - size * 0.5;

    return length(max(d, vec3<f32>(0.0))) + min(max(max(d.x, d.y), d.z), 0.0);
}

fn fractal_cube(pos: vec3<f32>, diameter: f32, layers: u32, p: vec3<f32>) -> f32 {
    var distance_to_shape = cube(pos, vec3<f32>(diameter), p);

    var size = diameter;
    var third_size = diameter / 3.0;
    var mod_pos = (p - pos) % size;

    var n = u32(0);
    while n < layers {
        distance_to_shape = max(
            distance_to_shape,
            -min(
                cube(pos, vec3<f32>(size, third_size, third_size) + 0.001 * 2.0, mod_pos),
                min(
                    cube(pos, vec3<f32>(third_size, size, third_size) + 0.001 * 2.0, mod_pos),
                    cube(pos, vec3<f32>(third_size, third_size, size) + 0.001 * 2.0, mod_pos)
                )
            )
        );
        size /= 3.0;
        third_size /= 3.0;
        mod_pos /= 3.0;

        n++;
    }
    return distance_to_shape;
}
