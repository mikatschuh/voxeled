fn rotate(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> { // ai
    let u = q.xyz;       // Vektoranteil des Quaternions
    let s = q.w;         // Skalaranteil des Quaternions
    let v_cross_u = cross(u, v);  // Kreuzprodukt von u und v
    let term1 = 2.0 * dot(u, v) * u;  // Parallelprojektion auf u
    let term2 = (s * s - dot(u, u)) * v;  // Skaliertes v
    let term3 = 2.0 * s * v_cross_u;  // Rotationseffekt
    return term1 + term2 + term3;     // Ergebnis
}
fn rotation_matrix(axis: vec3<f32>, angle: f32) -> mat3x3<f32> { // ai
    let normalized_axis = normalize(axis);
    let x = normalized_axis.x;
    let y = normalized_axis.y;
    let z = normalized_axis.z;

    let cos_theta = cos(angle);
    let sin_theta = sin(angle);
    let one_minus_cos = 1.0 - cos_theta;

    return mat3x3<f32>(
        vec3<f32>(
            cos_theta + x * x * one_minus_cos,
            x * y * one_minus_cos - z * sin_theta,
            x * z * one_minus_cos + y * sin_theta
        ),
        vec3<f32>(
            y * x * one_minus_cos + z * sin_theta,
            cos_theta + y * y * one_minus_cos,
            y * z * one_minus_cos - x * sin_theta
        ),
        vec3<f32>(
            z * x * one_minus_cos - y * sin_theta,
            z * y * one_minus_cos + x * sin_theta,
            cos_theta + z * z * one_minus_cos
        )
    );
}
fn smooth_min(a: f32, b: f32, k: f32) -> f32 { // ai
    let h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0); // Interpolationswert basierend auf dem Unterschied und der Schärfe
    return mix(b, a, h); // Glattes Mischen der beiden Werte
}
fn reflect(direction: vec3<f32>, normal: vec3<f32>) -> vec3<f32> { // ai
    // Reflexionsformel
    return direction - 2.0 * dot(direction, normal) * normal;
}
fn reflect_glossy(direction: vec3<f32>, normal: vec3<f32>, spread: f32, seed: f32) -> vec3<f32> { // ai and human
    let dot_product = dot(direction, normal);
    // reflection formula
    return normalize(direction - 2.0 * dot_product * normal + random3(vec3<f32>(dot_product), seed) * spread);
}

fn apply_noise(base_color: vec3<f32>, seed: vec3<f32>) -> vec3<f32> {

    return base_color + rand(length(seed)) * 0.001 * seed;
}
