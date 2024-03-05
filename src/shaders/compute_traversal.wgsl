// Scene traversal shader
@group(0) @binding(8) var outputTex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let ix = global_ix.xy;
    let ray_idx = ix.y * scene.config.size.x + ix.x;
    let ray = rays.data[ray_idx];
    let pixel_color = sample_scene(
        ray, 
        scene.config.max_depth, 
        scene.config.samples,
        scene.config.pixel_size, 
        ray_idx
        );

    // TODO: investigate coord system mismatch
    let flippedIdx = vec2<i32>(i32(scene.config.size.x) - i32(ix.x) - 1, i32(ix.y));
    textureStore(outputTex, flippedIdx, pixel_color);
}

fn sample_scene(ray: Ray, max_depth: u32, spp: u32, pixel_size: vec2<f32>, global_idx: u32) -> vec4<f32> {
    var outColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focus_distance: f32 = length(ray.origin);

    for (var sample_idx = 0u; sample_idx < spp; sample_idx = sample_idx + 1u) {
        let seed = sample_idx * global_idx + sample_idx + 999u * global_idx;
        let rng = vec2<f32>(hash_u32(seed * scene.config.seed.x), hash_u32(seed * scene.config.seed.y));

        var compositeColor = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        var directSpecColor = compositeColor;
        var indirectSpecColor = compositeColor;
        var indirectDiffColor = compositeColor;
        var directDiffColor = compositeColor;
        var skyColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);

        var directSpecRay = get_strat_offset_ray(ray, pixel_size, focus_distance, rng, scene.config.count);
        var indirectSpecRay = directSpecRay;
        var directDiffRay = directSpecRay;
        var indirectDiffRay = directSpecRay;

        var directSpecActive = true;
        var indirectSpecActive = true;
        var directDiffActive = true;
        var indirectDiffActive = true; 
        var primary_hit_metal = false;   

        for (var depth = max_depth; depth > 0u; depth = depth - 1u) {

            // Rays be bouncing
            let directSpecRec = hit_bvh(directSpecRay);
            let indirectSpecRec = hit_bvh(indirectSpecRay);
            let directDiffRec = hit_bvh(directDiffRay);
            let indirectDiffRec = hit_bvh(indirectDiffRay);

            // Direct specular
            if (directSpecRec.t > 0.0) {
                let is_metal = directSpecRec.material.metallic > rng.x;
                var specular_color = vec4<f32>(1.0, 1.0, 1.0, 1.0) * directSpecRec.material.specular;
                var f0: vec3<f32>;
                if (is_metal) {                     
                    specular_color *= directSpecRec.material.diffuse;
                    f0 = vec3<f32>(0.8, 0.8, 0.8);
                } else {                     
                    f0 = vec3<f32>(pow((1.0 - directSpecRec.material.ior) / (1.0 + directSpecRec.material.ior), 2.0));
                }

                let light_sample = sample_quad_light(directDiffRec, rng);      
                let ggxDirect = ggx_direct(
                    directSpecRec.normal, 
                    -directSpecRay.direction, 
                    light_sample.dir, 
                    directSpecRec.material.roughness, 
                    f0
                );
                directSpecColor = specular_color * ggxDirect.weight * light_sample.color;
                directSpecRay = Ray(directSpecRec.p, light_sample.dir);                
            } else if (directSpecActive) {
                if (depth == max_depth) {
                    directSpecColor *= 0.0;
                }
                directSpecActive = false; 
            }

            // Indirect specular
            if (indirectSpecRec.t > 0.0) {
                var specular_color = vec4<f32>(1.0, 1.0, 1.0, 1.0) * indirectSpecRec.material.specular;
                var f0: vec3<f32>;
                if (depth == max_depth) {
                    primary_hit_metal = indirectSpecRec.material.metallic > rng.x;
                } else {
                    specular_color *= indirectSpecRec.material.diffuse;
                }
                if (primary_hit_metal) {                     
                    f0 = vec3<f32>(0.8, 0.8, 0.8);
                } else {                     
                    f0 = vec3<f32>(pow((1.0 - indirectSpecRec.material.ior) / (1.0 + indirectSpecRec.material.ior), 2.0));
                }
                let ggxIndirect = ggx_indirect(
                    indirectSpecRec.normal, 
                    -indirectSpecRay.direction, 
                    indirectSpecRec.material.roughness, 
                    f0,
                    rng
                );    
                indirectSpecColor *= specular_color * ggxIndirect.weight;
                indirectSpecRay = Ray(indirectSpecRec.p, reflect(indirectSpecRay.direction, ggxIndirect.direction));
            } else if (indirectSpecActive) {
                let skySample = sample_sky(indirectSpecRay.direction, scene.config.sky_intensity);
                if (depth == max_depth) {
                    skyColor += skySample;
                    indirectSpecColor *= 0.0;
                } else {
                    indirectSpecColor *= skySample;     
                }                              
                indirectSpecActive = false; 
            }
                
            // Direct diffuse 
            var is_metal = directDiffRec.material.metallic > rng.x;
            if (directDiffRec.t > 0.0 && !is_metal) { 
                let light_sample = sample_quad_light(directDiffRec, rng);             
                directDiffColor *= directDiffRec.material.diffuse * light_sample.color;  
                directDiffRay = Ray(directDiffRec.p, cosine_weighted_hemisphere(directDiffRec, rng).dir);   
            } else if (directDiffActive) {
                if (depth == max_depth) {
                    directDiffColor *= 0.0;
                }
                directDiffActive = false;
            }

            // Indirect diffuse
            is_metal = indirectDiffRec.material.metallic > rng.x;
            if (indirectDiffRec.t > 0.0 && !is_metal && indirectDiffActive) {
                indirectDiffColor *= indirectDiffRec.material.diffuse;
                indirectDiffRay = Ray(indirectDiffRec.p, cosine_weighted_hemisphere(indirectDiffRec, rng).dir);                   
            } else if (indirectDiffActive) {
                if (depth == max_depth) {
                    indirectDiffColor *= 0.0;
                } else {
                    indirectDiffColor *= sample_sky(indirectDiffRay.direction, scene.config.sky_intensity);
                } 
                indirectDiffActive = false;
            }
            compositeColor = directSpecColor + indirectSpecColor + directDiffColor + indirectDiffColor + skyColor;                
        }
        outColor += compositeColor;
    }
    outColor /= vec4<f32>(f32(spp));

    return outColor;
}

fn sample_scene_alt(ray: Ray, max_depth: u32, spp: u32, pixel_size: vec2<f32>, global_idx: u32) -> vec4<f32> {
    var outColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focus_distance: f32 = length(ray.origin);

    for (var sample_idx = 0u; sample_idx < spp; sample_idx = sample_idx + 1u) {
        let seed = sample_idx * global_idx + sample_idx + 999u * global_idx;
        let rng = vec2<f32>(hash_u32(seed * scene.config.seed.x), hash_u32(seed * scene.config.seed.y));

        var compositeColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        var rayColor = compositeColor;
        var skyColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);

        var ray = get_strat_offset_ray(ray, pixel_size, focus_distance, rng, scene.config.count);
        var rayActive = true;
        var primary_hit_metal = false;   

        for (var depth = max_depth; depth > 0u; depth = depth - 1u) {
            // Rays be bouncing
            let rec = hit_bvh(ray);

            if (rec.t > 0.0) {
                let is_metal = rec.material.metallic > rng.x;
                var color = vec4<f32>(1.0, 1.0, 1.0, 1.0) * rec.material.specular;
                var f0: vec3<f32>;
                if (is_metal) {                     
                    color *= rec.material.diffuse;
                    f0 = vec3<f32>(0.8, 0.8, 0.8);
                } else {                     
                    f0 = vec3<f32>(pow((1.0 - rec.material.ior) / (1.0 + rec.material.ior), 2.0));
                }

                let light_sample = sample_quad_light(rec, rng);      
                let ggx = ggx_direct(
                    rec.normal, 
                    -ray.direction, 
                    light_sample.dir, 
                    rec.material.roughness, 
                    f0
                );
                rayColor = color * ggx.weight * light_sample.color;
                ray = Ray(rec.p, light_sample.dir);                


                // Direct diffuse
                if (!is_metal) { 
                    let light_sample = sample_quad_light(rec, rng);             
                    rayColor += rec.material.diffuse * light_sample.color;  
                    ray = Ray(rec.p, cosine_weighted_hemisphere(rec, rng).dir);   
                }

                // Indirect lighting
                let ggxIndirect = ggx_indirect(
                    rec.normal, 
                    -ray.direction, 
                    rec.material.roughness, 
                    f0,
                    rng
                );    
                rayColor += color * ggxIndirect.weight;
                ray = Ray(rec.p, reflect(ray.direction, ggxIndirect.direction));

                // Indirect diffuse
                if (!is_metal) {
                    rayColor += rec.material.diffuse;
                    ray = Ray(rec.p, cosine_weighted_hemisphere(rec, rng).dir);                   
                }
            } else if (rayActive) {
                if (depth == max_depth) {
                    rayColor *= 0.0;
                }
                rayActive = false; 
            }

            if (!rayActive) {
                let skySample = sample_sky(ray.direction, scene.config.sky_intensity);
                if (depth == max_depth) {
                    skyColor += skySample;
                    rayColor *= 0.0;
                } else {
                    rayColor *= skySample;     
                }                              
            }
                
            compositeColor = rayColor;                
        }
        outColor += compositeColor;
    }
    outColor /= vec4<f32>(f32(spp));

    return outColor;
}

