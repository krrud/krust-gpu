// Indirect diffuse compute shader
@group(0) @binding(7) var outputTex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let ix = global_ix.xy;
    let rayIdx = ix.y * scene.config.size.x + ix.x;
    let ray = rays.data[rayIdx];
    let pixelColor = sample_direct_diffuse(
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

fn sample_direct_diffuse(ray: Ray, maxDepth: u32, spp: u32, pixelSize: vec2<f32>, globalIdx: u32) -> vec4<f32> {
    var outColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focusDistance: f32 = length(ray.origin);

    for (var sampleIdx = 0u; sampleIdx < spp; sampleIdx = sampleIdx + 1u) {
        let seed = sampleIdx * globalIdx + sampleIdx + 999u * globalIdx;
        let rng = vec2<f32>(hash_u32(seed * scene.config.seed.x), hash_u32(seed * scene.config.seed.y));

        var color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        var ray = get_strat_offset_ray(ray, pixelSize, focusDistance, rng, scene.config.count);
        var rayActive = true;

        for (var depth = maxDepth; depth > 0u; depth = depth - 1u) {
            let rec = hit_bvh(ray);
            var isMetallic = rec.material.metallic > rng.x;
            if (rec.t > 0.0 && !isMetallic) { 
                let lightSample = sample_quad_light(quadLightBuffer.data[0], rec, rng);             
                color *= rec.material.diffuse * lightSample.color;  
                ray = Ray(rec.p, cosine_weighted_hemisphere(rec, rng));   
            } else if (rayActive) {
                if (depth == maxDepth) {
                    color *= 0.0;
                }
                rayActive = false;
            }            
        }
        outColor += color;
    }  
    return outColor / vec4<f32>(f32(spp));
}

