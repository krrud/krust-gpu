// Indirect diffuse compute shader
@group(0) @binding(10) var outputTex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let ix = global_ix.xy;
    let rayIdx = ix.y * scene.config.size.x + ix.x;
    let ray = rays.data[rayIdx];
    let pixelColor = sample_direct_specular(
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

fn sample_direct_specular(ray: Ray, maxDepth: u32, spp: u32, pixelSize: vec2<f32>, globalIdx: u32) -> vec4<f32> {
    var outColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focusDistance: f32 = length(ray.origin);
    let invSpp = 1.0 / f32(spp);

    for (var sampleIdx = 0u; sampleIdx < spp; sampleIdx = sampleIdx + 1u) {
        let seed = sampleIdx * globalIdx + sampleIdx + 999u * globalIdx;
        let rng = vec2<f32>(hash_u32(seed * scene.config.seed.x), hash_u32(seed * scene.config.seed.y));

        var color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        var ray = get_strat_offset_ray(ray, pixelSize, focusDistance, rng, scene.config.count);

        for (var depth = maxDepth; depth > 0u; depth = depth - 1u) {
            let rec = hit_bvh(ray);
            if (rec.t > 0.0) {
                let isMetallic = rec.material.metallic > rng.x;
                var specularColor = vec4<f32>(1.0, 1.0, 1.0, 1.0) * rec.material.specular;
                var f0: vec3<f32>;
                if (isMetallic) {                     
                    specularColor *= rec.material.diffuse;
                    f0 = vec3<f32>(0.8, 0.8, 0.8);
                } else {                     
                    f0 = vec3<f32>(pow((1.0 - rec.material.ior) / (1.0 + rec.material.ior), 2.0));
                }

                let lightSample = sample_quad_light(quadLightBuffer.data[0], rec, rng);      
                let ggxDirect = ggx_direct(
                    rec.normal, 
                    -ray.direction, 
                    lightSample.dir, 
                    rec.material.roughness, 
                    f0
                );
                color = specularColor * ggxDirect.weight * lightSample.color;
                ray = Ray(rec.p, lightSample.dir);                
            } else {
                if (depth == maxDepth) {
                    color *= 0.0;
                }
                break;
            }         
        }
        outColor += color * invSpp;
    }   
    return outColor;
}

