fn ggx_distribution(n_dot_h: f32, roughness: f32) -> f32 {
    // GGX distribution function
    let a2 = roughness * roughness;
    let d = ((n_dot_h * a2 - n_dot_h) * n_dot_h + 1.0); 
    return a2 / (PI * d * d + EPSILON);
}


fn ggx_sample(n: vec3<f32>, roughness: f32, rng: vec2<f32>) -> vec3<f32> {
	// Bitangent and tangent vectors
	let b = get_perpendicular(n);
	let t = cross(b, n);

	// Sampling
	let a2: f32 = (roughness * roughness) - 1.0;
	let cos_theta: f32 = sqrt(max(0.0, (1.0 - rng.x) / ((a2 * rng.x) + 1.0)));
	let sin_theta: f32 = sqrt(max(0.0, 1.0 - cos_theta * cos_theta));
	let phi: f32 = rng.y * PI * 2.0;

	return t * (sin_theta * cos(phi)) + b * (sin_theta * sin(phi)) + n * cos_theta;
}

fn schlick_masking(n_dot_l: f32, n_dot_v: f32, roughness: f32) -> f32 {
    let k = roughness * roughness / 2.0;
    let g1 = n_dot_l / (n_dot_l * (1.0 - k) + k + EPSILON);
    let g2 = n_dot_v / (n_dot_v * (1.0 - k) + k + EPSILON);
    return g1 * g2;
}

fn schlick_fresnel(f0: vec3<f32>, l_dot_h: f32) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0, 1.0, 1.0) - f0) * pow(1.0 - l_dot_h, 5.0);
}

fn ggx_indirect(
    n: vec3<f32>,
    v: vec3<f32>,
    roughness: f32,
    f0: vec3<f32>,
    rng: vec2<f32>,
) -> GGX {
    // Sample half vector
    let r = roughness * roughness + EPSILON;
    let h = ggx_sample(n, r, rng);
    let l = normalize(2.0 * dot(v, h) * h - v);

    // Compute terms
    let n_dot_l = saturate(dot(n, l));
    let n_dot_v = saturate(dot(n, v));
    let n_dot_h = saturate(dot(n, h));
    let l_dot_h = saturate(dot(l, h));

    // Compute weight
    let d = ggx_distribution(n_dot_h, r);
    let f = schlick_fresnel(f0, l_dot_h);
    let g = schlick_masking(n_dot_l, n_dot_v, r);
    let term = f * d * g / (4.0 * n_dot_l * n_dot_v + 0.01);
    let prob = d * n_dot_h / (4.0 * l_dot_h + 0.01);
    let weight = n_dot_l * term / prob;
    
    return GGX(h, vec4<f32>(weight, 1.0));
}   

fn get_perpendicular(n: vec3<f32>) -> vec3<f32> {
    // Find a vector perpendicular to n
    let b = vec3<f32>(0.0, 1.0, 0.0);
    let t = cross(b, n);
    return normalize(t);
}