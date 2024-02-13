// Scene traversal shader
@group(0) @binding(8) var outputTex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let ix = global_ix.xy;
    let rayIdx = ix.y * scene.config.size.x + ix.x;
    let ray = rays.data[rayIdx];
    let pixelColor = sample_scene(
        ray, 
        scene.config.max_depth, 
        scene.config.samples,
        scene.config.pixel_size, 
        rayIdx
        );

    // TODO: investigate coord system mismatch
    let flippedIdx = vec2<i32>(i32(scene.config.size.x) - i32(ix.x) - 1, i32(ix.y));
    textureStore(outputTex, flippedIdx, pixelColor);
}

fn sample_scene(ray: Ray, maxDepth: u32, spp: u32, pixelSize: vec2<f32>, globalIdx: u32) -> vec4<f32> {
    var outColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focusDistance: f32 = length(ray.origin);

    for (var sampleIdx = 0u; sampleIdx < spp; sampleIdx = sampleIdx + 1u) {
        let seed = sampleIdx * globalIdx + sampleIdx + 999u * globalIdx;
        let rng = vec2<f32>(hash_u32(seed * scene.config.seed.x), hash_u32(seed * scene.config.seed.y));

        var compositeColor = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        var directSpecColor = compositeColor;
        var indirectSpecColor = compositeColor;
        var indirectDiffColor = compositeColor;
        var directDiffColor = compositeColor;
        var skyColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);

        var directSpecRay = get_strat_offset_ray(ray, pixelSize, focusDistance, rng, scene.config.count);
        var indirectSpecRay = directSpecRay;
        var directDiffRay = directSpecRay;
        var indirectDiffRay = directSpecRay;

        var directSpecActive = true;
        var indirectSpecActive = true;
        var directDiffActive = true;
        var indirectDiffActive = true; 
        var primaryHitMetal = false;   

        for (var depth = maxDepth; depth > 0u; depth = depth - 1u) {

            // Rays be bouncing
            let directSpecRec = hit_bvh(directSpecRay);
            let indirectSpecRec = hit_bvh(indirectSpecRay);
            let directDiffRec = hit_bvh(directDiffRay);
            let indirectDiffRec = hit_bvh(indirectDiffRay);

            // Direct specular
            if (directSpecRec.t > 0.0) {
                let isMetallic = directSpecRec.material.metallic > rng.x;
                var specularColor = vec4<f32>(1.0, 1.0, 1.0, 1.0) * directSpecRec.material.specular;
                var f0: vec3<f32>;
                if (isMetallic) {                     
                    specularColor *= directSpecRec.material.diffuse;
                    f0 = vec3<f32>(0.8, 0.8, 0.8);
                } else {                     
                    f0 = vec3<f32>(pow((1.0 - directSpecRec.material.ior) / (1.0 + directSpecRec.material.ior), 2.0));
                }

                let lightSample = sample_quad_light(quadLightBuffer.data[0], directDiffRec, rng);      
                let ggxDirect = ggx_direct(
                    directSpecRec.normal, 
                    -directSpecRay.direction, 
                    lightSample.dir, 
                    directSpecRec.material.roughness, 
                    f0
                );
                directSpecColor = specularColor * ggxDirect.weight * lightSample.color;
                directSpecRay = Ray(directSpecRec.p, lightSample.dir);                
            } else if (directSpecActive) {
                if (depth == maxDepth) {
                    directSpecColor *= 0.0;
                }
                directSpecActive = false; 
            }

            // Indirect specular
            if (indirectSpecRec.t > 0.0) {
                var specularColor = vec4<f32>(1.0, 1.0, 1.0, 1.0) * indirectSpecRec.material.specular;
                var f0: vec3<f32>;
                if (depth == maxDepth) {
                    primaryHitMetal = indirectSpecRec.material.metallic > rng.x;
                } else {
                    specularColor *= indirectSpecRec.material.diffuse;
                }
                if (primaryHitMetal) {                     
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
                indirectSpecColor *= specularColor * ggxIndirect.weight;
                indirectSpecRay = Ray(indirectSpecRec.p, reflect(indirectSpecRay.direction, ggxIndirect.direction));
            } else if (indirectSpecActive) {
                let skySample = sample_sky(indirectSpecRay, scene.config.sky_intensity);
                if (depth == maxDepth) {
                    skyColor += skySample;
                    indirectSpecColor *= 0.0;
                } else {
                    indirectSpecColor *= skySample;     
                }                              
                indirectSpecActive = false; 
            }
                
            // Direct diffuse 
            var isMetallic = directDiffRec.material.metallic > rng.x;
            if (directDiffRec.t > 0.0 && !isMetallic) { 
                let lightSample = sample_quad_light(quadLightBuffer.data[0], directDiffRec, rng);             
                directDiffColor *= directDiffRec.material.diffuse * lightSample.color;  
                directDiffRay = Ray(directDiffRec.p, cosine_weighted_hemisphere(directDiffRec, rng).dir);   
            } else if (directDiffActive) {
                if (depth == maxDepth) {
                    directDiffColor *= 0.0;
                }
                directDiffActive = false;
            }

            // Indirect diffuse
            isMetallic = indirectDiffRec.material.metallic > rng.x;
            if (indirectDiffRec.t > 0.0 && !isMetallic && indirectDiffActive) {
                indirectDiffColor *= indirectDiffRec.material.diffuse;
                indirectDiffRay = Ray(indirectDiffRec.p, cosine_weighted_hemisphere(indirectDiffRec, rng).dir);                   
            } else if (indirectDiffActive) {
                if (depth == maxDepth) {
                    indirectDiffColor *= 0.0;
                } else {
                    indirectDiffColor *= sample_sky(indirectDiffRay, scene.config.sky_intensity);
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

fn sample_scene_alt(ray: Ray, maxDepth: u32, spp: u32, pixelSize: vec2<f32>, globalIdx: u32) -> vec4<f32> {
    var outColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focusDistance: f32 = length(ray.origin);

    for (var sampleIdx = 0u; sampleIdx < spp; sampleIdx = sampleIdx + 1u) {
        let seed = sampleIdx * globalIdx + sampleIdx + 999u * globalIdx;
        let rng = vec2<f32>(hash_u32(seed * scene.config.seed.x), hash_u32(seed * scene.config.seed.y));

        var compositeColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        var rayColor = compositeColor;
        var skyColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);

        var ray = get_strat_offset_ray(ray, pixelSize, focusDistance, rng, scene.config.count);
        var rayActive = true;
        var primaryHitMetal = false;   

        for (var depth = maxDepth; depth > 0u; depth = depth - 1u) {
            // Rays be bouncing
            let rec = hit_bvh(ray);

            if (rec.t > 0.0) {
                let isMetallic = rec.material.metallic > rng.x;
                var color = vec4<f32>(1.0, 1.0, 1.0, 1.0) * rec.material.specular;
                var f0: vec3<f32>;
                if (isMetallic) {                     
                    color *= rec.material.diffuse;
                    f0 = vec3<f32>(0.8, 0.8, 0.8);
                } else {                     
                    f0 = vec3<f32>(pow((1.0 - rec.material.ior) / (1.0 + rec.material.ior), 2.0));
                }

                let lightSample = sample_quad_light(quadLightBuffer.data[0], rec, rng);      
                let ggx = ggx_direct(
                    rec.normal, 
                    -ray.direction, 
                    lightSample.dir, 
                    rec.material.roughness, 
                    f0
                );
                rayColor = color * ggx.weight * lightSample.color;
                ray = Ray(rec.p, lightSample.dir);                


                // Direct diffuse
                if (!isMetallic) { 
                    let lightSample = sample_quad_light(quadLightBuffer.data[0], rec, rng);             
                    rayColor += rec.material.diffuse * lightSample.color;  
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
                if (!isMetallic) {
                    rayColor += rec.material.diffuse;
                    ray = Ray(rec.p, cosine_weighted_hemisphere(rec, rng).dir);                   
                }
            } else if (rayActive) {
                if (depth == maxDepth) {
                    rayColor *= 0.0;
                }
                rayActive = false; 
            }

            if (!rayActive) {
                let skySample = sample_sky(ray, scene.config.sky_intensity);
                if (depth == maxDepth) {
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

