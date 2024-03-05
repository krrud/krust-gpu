// Indirect diffuse compute shader
@group(0) @binding(10) var output_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(11) var sky_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let idx = global_ix.xy;
    let ray_idx = idx.y * scene.config.size.x + idx.x;
    let ray = rays.data[ray_idx];
    let sample = sample_indirect_specular(
        ray, 
        scene.config.max_depth, 
        scene.config.samples,
        scene.config.pixel_size, 
        ray_idx
        );

    // TODO: investigate coord system mismatch
    let flipped_idx = vec2<i32>(i32(scene.config.size.x) - i32(idx.x) - 1, i32(idx.y));
    textureStore(output_tex, flipped_idx, sample.color);
    textureStore(sky_tex, flipped_idx, sample.sky);
}

fn sample_indirect_specular(ray: Ray, max_depth: u32, spp: u32, pixel_size: vec2<f32>, global_idx: u32) -> WithSky {
    var pixel_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    var sky_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focus_distance: f32 = length(ray.origin);
    let inv_spp = 1.0 / f32(spp);

    for (var sample_idx = 0u; sample_idx < spp; sample_idx = sample_idx + 1u) {
        let seed: u32 = sample_idx * global_idx + sample_idx + 999u * global_idx;
        let rng = vec2<f32>(hash_u32(seed * scene.config.seed.x), hash_u32(seed * scene.config.seed.y));

        var color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        var ray: Ray = get_strat_offset_ray(ray, pixel_size, focus_distance, rng, scene.config.count);
        var primary_hit_metal = false;

        for (var depth = max_depth; depth > 0u; depth = depth - 1u) {
            let rec = hit_bvh(ray);
            if (rec.t > 0.0) {
                var specular_color = vec4<f32>(1.0, 1.0, 1.0, 1.0) * rec.material.specular;
                var f0: vec3<f32>;
                if (depth == max_depth) {
                    let is_metal = rec.material.metallic > rng.x;
                    primary_hit_metal = is_metal;
                    if (is_metal) {
                       specular_color *= rec.material.diffuse; 
                    }                    
                } else {
                    specular_color *= rec.material.diffuse;
                }
                if (primary_hit_metal) {                     
                    f0 = vec3<f32>(0.6, 0.6,  0.6);
                } else {                     
                    f0 = vec3<f32>(pow((1.0 - rec.material.ior) / (1.0 + rec.material.ior), 2.0));
                }
                let ggx = ggx_indirect(
                    rec.normal, 
                    -ray.direction, 
                    rec.material.roughness, 
                    f0,
                    rng
                );    
                color *= specular_color * ggx.weight;
                if (scene.config.sky_intensity < EPSILON || !rec.frontface) { // TODO: hacky wacky
                    color*= 0.0;
                }
                ray = Ray(rec.p, reflect(ray.direction, ggx.direction));             
            } else {
                let sky_sample = sample_sky(ray.direction, scene.config.sky_intensity);
                if (depth == max_depth) {
                    sky_color += sky_sample;
                    color *= 0.0;
                } else {
                    color *= sky_sample;     
                }
                break;
            }       
        }
        pixel_color += color * inv_spp;
    }
    return WithSky(pixel_color, sky_color);
}

