// Indirect diffuse compute shader
@group(0) @binding(10) var outputTex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(11) var skyTex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let ix = global_ix.xy;
    let rayIdx = ix.y * scene.config.size.x + ix.x;
    let ray = rays.data[rayIdx];
    let sample = sample_indirect_specular(
        ray, 
        scene.config.max_depth, 
        scene.config.samples,
        scene.config.pixel_size, 
        rayIdx
        );
    let testMtl = materialBuffer.data[0];

    // TODO: investigate coord system mismatch
    let flippedIdx = vec2<i32>(i32(scene.config.size.x) - i32(ix.x) - 1, i32(ix.y));
    textureStore(outputTex, flippedIdx, sample.color);
    textureStore(skyTex, flippedIdx, sample.sky);
}

fn sample_indirect_specular(ray: Ray, maxDepth: u32, spp: u32, pixelSize: vec2<f32>, globalIdx: u32) -> WithSky {
    var outColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    var skyColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focusDistance: f32 = length(ray.origin);
    let invSpp = 1.0 / f32(spp);

    for (var sampleIdx = 0u; sampleIdx < spp; sampleIdx = sampleIdx + 1u) {
        let seed: u32 = sampleIdx * globalIdx + sampleIdx + 999u * globalIdx;
        let rng = vec2<f32>(hash_u32(seed * scene.config.seed.x), hash_u32(seed * scene.config.seed.y));

        var color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        var ray: Ray = get_strat_offset_ray(ray, pixelSize, focusDistance, rng, scene.config.count);
        var primaryHitMetal = false;

        for (var depth = maxDepth; depth > 0u; depth = depth - 1u) {
            let rec = hit_bvh(ray);
            if (rec.t > 0.0) {
                var specularColor = vec4<f32>(1.0, 1.0, 1.0, 1.0) * rec.material.specular;
                var f0: vec3<f32>;
                if (depth == maxDepth) {
                    let isMetallic = rec.material.metallic > rng.x;
                    primaryHitMetal = isMetallic;
                    if (isMetallic) {
                       specularColor *= rec.material.diffuse; 
                    }                    
                } else {
                    specularColor *= rec.material.diffuse;
                }
                if (primaryHitMetal) {                     
                    f0 = vec3<f32>(0.6, 0.6,  0.6);
                } else {                     
                    f0 = vec3<f32>(pow((1.0 - rec.material.ior) / (1.0 + rec.material.ior), 2.0));
                }
                let ggxIndirect = ggx_indirect(
                    rec.normal, 
                    -ray.direction, 
                    rec.material.roughness, 
                    f0,
                    rng
                );    
                color *= specularColor * ggxIndirect.weight;
                if (scene.config.sky_intensity < EPSILON || !rec.frontface) { // TODO: hacky wacky
                    color*= 0.0;
                }
                ray = Ray(rec.p, reflect(ray.direction, ggxIndirect.direction));             
            } else {
                let skySample = sample_sky(ray, scene.config.sky_intensity);
                if (depth == maxDepth) {
                    skyColor += skySample;
                    color *= 0.0;
                } else {
                    color *= skySample;     
                }
                break;
            }       
        }
        outColor += color * invSpp;
    }
    return WithSky(outColor, skyColor);
}

