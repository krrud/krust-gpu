fn ggx_distribution(n_dot_h: f32, roughness: f32) -> f32 {
    // GGX distribution function
    let a2 = roughness * roughness;
    let d = ((n_dot_h * a2 - n_dot_h) * n_dot_h + 1.0); 
    var denom = PI * d * d ;
    if (denom == 0.0) {
        denom = EPSILON;
    }
    return a2 / (denom);
}


fn ggx_sample(n: vec3<f32>, roughness: f32, rng: vec2<f32>) -> vec3<f32> {
	// Bitangent and tangent vectors
	let b = get_perpendicular(n);
	let t = cross(b, n);

	// Sampling
	let a2: f32 = (roughness * roughness) - 1.0;
	let cos_theta: f32 = min(1.0, sqrt(max(0.0, (1.0 - rng.x) / ((a2 * rng.x) + 1.0))));
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
    var base = 1.0 - l_dot_h;
    if base < 0.0 { 
        base= 0.0; 
    };
    return f0 + (vec3<f32>(1.0, 1.0, 1.0) - f0) * pow(1.0 - l_dot_h, 5.0);
}

fn ggx_indirect(
    n: vec3<f32>,
    v: vec3<f32>,
    roughness: f32,
    f0: vec3<f32>,
    rng: vec2<f32>,
) -> GGX {
    // Half vector and ggx sample
    let r = roughness * roughness + EPSILON;
    let h = ggx_sample(n, r, rng);
    let l = normalize(2.0 * dot(v, h) * h - v);

    // Compute terms
    let ndl = saturate(dot(n, l));
    let ndv = saturate(dot(n, v));
    let ndh = saturate(dot(n, h));
    let ldh = saturate(dot(l, h));

    // Compute weight
    let d = ggx_distribution(ndh, r);
    let f = schlick_fresnel(f0, ldh);
    let g = schlick_masking(ndl, ndv, r);
    let term = f * d * g / (4.0 * ndl * ndv + EPSILON);
    let prob = d * ndh / (4.0 * ldh + EPSILON);
    let weight = ndl * term / prob;
    
    return GGX(h, vec4<f32>(weight, 1.0));
}  

fn ggx_direct(n : vec3<f32>, v : vec3<f32>, l: vec3<f32>, roughness : f32, f0: vec3<f32>) -> GGX
{
    // Half vector and dots
    let r = roughness * roughness + EPSILON;
    let h : vec3<f32> = normalize(v + l);
    let ndl : f32 = saturate(dot(n, l));
    let ndh : f32 = saturate(dot(n, h));
    let ldh : f32 = saturate(dot(l, h));
    let ndv : f32 = saturate(dot(n, v));

    // Compute terms
    let d : f32 = ggx_distribution(ndh, r);
    let g : f32 = schlick_masking(ndl, ndv, r);
    let f : vec3<f32> = schlick_fresnel(f0, ldh);

    // Compute weight
    let weight : vec3<f32> = d * g * f / (4.0 * ndv * ndl + EPSILON);

    return GGX(h, vec4<f32>(weight, 1.0));
}

fn get_perpendicular(n: vec3<f32>) -> vec3<f32> {
    // Find a vector perpendicular to n
    var b = vec3<f32>(1.0, 0.0, 0.0);
    var t = cross(b, n);
    if (length(t) == 0.0) {
        b = vec3<f32>(0.0, 1.0, 0.0);
        t = cross(b, n);
    }
    
    return normalize(t);
}