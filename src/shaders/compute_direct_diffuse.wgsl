// Indirect diffuse compute shader
@group(0) @binding(10) var output_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_idx: vec3<u32>) {
    let idx = global_idx.xy;
    let ray_idx = idx.y * scene.config.size.x + idx.x;
    let ray = rays.data[ray_idx];
    let pixel_color = sample_direct_diffuse(
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

fn sample_direct_diffuse(ray: Ray, max_depth: u32, spp: u32, pixel_size: vec2<f32>, global_idx: u32) -> vec4<f32> {
    var pixel_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focus_distance: f32 = length(ray.origin);
    let inv_spp = 1.0 / f32(spp);

    for (var sample_idx = 0u; sample_idx < spp; sample_idx = sample_idx + 1u) {
        let seed = sample_idx * global_idx + sample_idx + 999u * global_idx;
        let rng = vec2<f32>(hash_u32(seed * scene.config.seed.x), hash_u32(seed * scene.config.seed.y));

        var color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        var ray = get_strat_offset_ray(ray, pixel_size, focus_distance, rng, scene.config.count);

        for (var depth = max_depth; depth > 0u; depth = depth - 1u) {
            let rec = hit_bvh(ray);
            var is_metal = rec.material.metallic > rng.x;
            if (rec.t > 0.0 && !is_metal) { 
                let light_sample = sample_quad_light(rec, rng);    
                let cosine_sample: CosineDiffuse = cosine_weighted_hemisphere(rec, rng);         
                color *= rec.material.diffuse * light_sample.color * cosine_sample.pdf;  
                ray = Ray(rec.p, cosine_sample.dir);   
            } else {
                if (depth == max_depth) {
                    color *= 0.0;
                }
                break;
            }            
        }
        pixel_color += color * inv_spp; 
    }  
    return pixel_color;
}

