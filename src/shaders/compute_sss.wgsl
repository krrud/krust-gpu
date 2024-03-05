// Indirect diffuse compute shader
@group(0) @binding(10) var output_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_idx: vec3<u32>) {
    let idx = global_idx.xy;
    let ray_idx = idx.y * scene.config.size.x + idx.x;
    let ray = rays.data[ray_idx];
    let pixel_color = sample_sss(
        ray, 
        scene.config.max_depth, 
        scene.config.samples,
        scene.config.pixel_size, 
        ray_idx
        );

    // TODO: investigate coord system mismatch
    let flipped_idx = vec2<i32>(i32(scene.config.size.x) - i32(idx.x) - 1, i32(idx.y));
    textureStore(output_tex, flipped_idx, pixel_color);
}

fn sample_sss(ray: Ray, max_depth: u32, spp: u32, pixel_size: vec2<f32>, global_idx: u32) -> vec4<f32> {
    var pixel_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focus_distance: f32 = length(ray.origin);
    let inv_spp = 1.0 / f32(spp);

    for (var sample_idx = 0u; sample_idx < spp; sample_idx = sample_idx + 1u) {
        let seed = sample_idx * global_idx + sample_idx + 999u * global_idx;
        let rng = vec2<f32>(hash_u32(seed * scene.config.seed.x), hash_u32(seed * scene.config.seed.y));

        var color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        var sample_ray = get_strat_offset_ray(ray, pixel_size, focus_distance, rng, scene.config.count);

        let rec = hit_bvh(sample_ray);
        var is_metal = rec.material.metallic > rng.x;
        if (rec.t > 0.0) { 
            color *= randomwalk_sss(rec, rng);                
        } else {
            color *= 0.0;
        }    

        pixel_color += color * inv_spp; 
    }  
    return pixel_color;
}

fn randomwalk_sss(rec: HitRec, rng: vec2<f32>) -> vec4<f32> {
    var hit = rec.p;
    let num_steps: u32 = 128u;
    let scale = 1.0;

    // Skin
    let absorption_coeff = vec3<f32>(0.04, 0.07, 0.1) * scale;
    let scattering_coeff = vec3<f32>(2.1, 1.9, 1.2) * scale;
    let g = 0.0;

    var total_light: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);

    let light_sample = sample_quad_light(rec, rng);
    var shadow = 0.0;
    if (light_sample.color.x + light_sample.color.y + light_sample.color.z > 0.0) {
        shadow = 1.0;
    }

    let max_sss_depth = 0.5;
    var sss_depth = 0.0;
    for (var i: u32 = 0u; i < num_steps; i = i + 1u) {
        let step_direction = randomwalk_step(hit, rec.normal, rng, g);
        let step_length = -log(1.0 - rng.x) / (absorption_coeff.x + scattering_coeff.x);
        hit += step_direction * step_length;
        sss_depth += step_length;
        
        if (sss_depth > max_sss_depth) {
            break;
        }

        // Calculate light attenuation and scattering
        let sky_contribution = sample_sky(step_direction, scene.config.sky_intensity).xyz;
        let light = quad_light_buffer.data[0];
        let to_light = light.position.xyz - hit;
        let light_distance = length(to_light);
        let light_dir = to_light / light_distance;
        let ndotl = max(dot(step_direction, light.normal.xyz), 0.0);
        let falloff = 1.0 / (light_distance * light_distance);
        var light_contribution = light.color.xyz * light.intensity * falloff * ndotl;

        let phase = phase_dwivedi(g, dot(rec.normal, -step_direction));

        let attenuation = exp(-absorption_coeff * step_length); 
        let scattered_light = (light_contribution + sky_contribution) * scattering_coeff * phase;
        
        total_light += attenuation * scattered_light * rec.material.diffuse.xyz;
    }
    
    return vec4<f32>(total_light / f32(num_steps), 1.0);
}

// Dwivedi phase function approximation
fn phase_dwivedi(g: f32, cos_theta: f32) -> f32 {
    let k = (1.0 - g * g) / (1.0 + g);
    let denom = 1.0 + k * cos_theta;
    return k / (4.0 * PI * denom * denom);
}

// Physically based random step direction considering anisotropy
fn randomwalk_step(hit: vec3<f32>, normal: vec3<f32>, rng: vec2<f32>, g: f32) -> vec3<f32> {
    let r = 2.0 * rng.x - 1.0;
    var cos_theta = r;
    if (g != 0.0) {
        cos_theta = 1.0 / (2.0 * g) * (1 + g*g - pow((1 - g*g) / (1 + g*r), 2.0));
    };
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);

    let phi = TWO_PI * rng.y;
    let x = sin_theta * cos(phi);
    let y = sin_theta * sin(phi);
    let z = cos_theta;
    let step_direction = vec3<f32>(x, y, z);
    return normalize(step_direction);
}


// fn dipole_sss(rec: HitRec, rng: vec2<f32>) -> vec4<f32> {
//     let sss_radius: f32 = 1.0;
//     let sigma_a: vec3<f32> = vec3<f32>(0.9) / sss_radius;
//     let sigma_s: vec3<f32> = vec3<f32>(0.4) / sss_radius;

//     let sigma_t = sigma_a + sigma_s;
//     let mu_t = dot(sigma_t, vec3<f32>(0.299, 0.587, 0.114));
//     let mu_s = dot(sigma_s, vec3<f32>(0.299, 0.587, 0.114));
//     let albedo = mu_s / mu_t;
//     let g = 0.0;

//     let sigma_prime_s = sigma_s * (1.0 - g);
//     let single_scattering = (1.0 - albedo) / (4.0 * PI);

//     let sky_direction = random_unit_vector(rng);
//     let sky_sample = sample_sky(sky_direction, scene.config.sky_intensity);

//     let light_sample = sample_quad_light(rec, rng);

//     let sss_color = sigma_a * (single_scattering + ((sky_sample.xyz + light_sample.color.xyz) * rec.material.diffuse.xyz));

//     return vec4<f32>(sss_color, 1.0);
// }


// fn dipole_sss(rec: HitRec, rng: vec2<f32>) -> vec4<f32> {
//     let sigma_a: vec3<f32> = vec3<f32>(0.65);
//     let sigma_s: vec3<f32> = vec3<f32>(0.014, 0.007, 0.004);

//     let sigma_t = sigma_a + sigma_s;
//     let mu_t = dot(sigma_t, vec3<f32>(0.299, 0.587, 0.114));
//     let mu_s = dot(sigma_s, vec3<f32>(0.299, 0.587, 0.114));
//     let mu_a = dot(sigma_a, vec3<f32>(0.299, 0.587, 0.114));
//     let albedo = mu_s / mu_t;
//     let g = 0.0;

//     let sigma_prime_s = sigma_s * (1.0 - g);
//     let single_scattering = (1.0 - albedo) / (4.0 * PI);

//     let sky_direction = random_unit_vector(rng);
//     let sky_sample = sample_sky(sky_direction, scene.config.sky_intensity);

//     let light_sample = sample_quad_light(rec, rng);

//     let sss_color = sigma_a * (single_scattering + ((sky_sample.xyz + light_sample.color.xyz) * rec.material.diffuse.xyz));

//     return vec4<f32>(sss_color, 1.0);
// }



