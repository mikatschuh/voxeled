fn rand(seed: f32) -> f32 {
    return fract(sin(seed) * 43758.5453123);
}
fn random3(x: vec3<f32>, y: f32) -> vec3<f32> {
    return vec3<f32>(
        fract(sin(dot(x, vec3<f32>(-127.1, 311.7, 74.7) * y)) * 43758.5453123),
        fract(sin(dot(x, vec3<f32>(269.5, -183.3, -246.1) * y)) * 43758.5453123),
        fract(sin(dot(x, vec3<f32>(-113.5, 271.9, 124.6) * y)) * 43758.5453123)
    );
}
