// Scene traversal shader
@group(0) @binding(0) var<storage, read> scene: Scene;
@group(0) @binding(1) var<storage, read> rays: RayBuffer;
@group(0) @binding(2) var outputTex: texture_storage_2d<rgba8unorm, write>;


@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let ix = global_ix.xy;
    let ray = rays.data[ix.y * scene.config.size.x + ix.x];
    let pixelColor = sample_scene(
        ray, 
        scene.config.max_depth, 
        scene.config.samples, 
        scene.config.pixel_size, 
        ix.x * ix.y
        );

    textureStore(outputTex, vec2<i32>(ix), pixelColor);
}

fn sample_scene(ray: Ray, maxDepth: u32, spp: u32, pixelSize: vec2<f32>, globalIdx: u32) -> vec4<f32> {
    var outColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focusDistance: f32 = length(ray.origin);

    for (var sampleIdx = 0; sampleIdx < i32(spp); sampleIdx = sampleIdx + 1) {
        let rng = vec2<f32>(hash(u32(sampleIdx) * globalIdx), hash(u32(sampleIdx + 99) * globalIdx));
        var localColor = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        var currentRay = get_offset_ray(ray, pixelSize, focusDistance, rng);

        for (var depth = i32(maxDepth); depth > 0; depth = depth - 1) {
            var i = 0;
            let nullMaterial = Material(vec4<f32>(0.0, 0.0, 0.0, 0.0), 0.0, 0.0, 0.0, 0.0, 1.5);
            var rec = HitRec(-1.0, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0), nullMaterial);

            while (i < i32(scene.config.num_objects)) {
                let hit = hit_sphere(scene.objects[i], currentRay);
                if (hit.t > 0.0 && (rec.t < 0.0 || hit.t < rec.t)) {
                    rec = hit;
                }
                i = i + 1;
            }

            if (rec.t > 0.0) {
                // We hit a thing
                var diffuseWeight = saturate(1.0 - rec.material.metallic);
                let specularWeight = rec.material.specular / (rec.material.specular + diffuseWeight);
                diffuseWeight = diffuseWeight * (1.0 - specularWeight);
                let isMetallic = rec.material.metallic > hash_simple(rec.p.x);
                let isSpecular = specularWeight > hash_simple(rec.p.x) || isMetallic;

                if (isSpecular) {
                    let specularColor = vec4<f32>(1.0, 1.0, 1.0, 1.0) * rec.material.specular;
                    var f0: vec3<f32>;
                    if (isMetallic) {
                        // Conductive
                        localColor = localColor * rec.material.diffuse;
                        f0 = vec3<f32>(0.8, 0.8, 0.8);
                    } else {
                        // Dielectric
                        localColor = localColor * specularColor;
                        f0 = vec3<f32>(0.04, 0.04, 0.04);
                    }

                    // Sample GGX distribution
                    let ggx = ggx_indirect(
                        rec.normal, 
                        -currentRay.direction, 
                        rec.material.roughness, 
                        f0,
                        rng
                        );

                    // Set ray direction and accumulate color    
                    currentRay = Ray(rec.p, ggx.direction);
                    localColor = localColor / specularWeight * ggx.weight;

                } else {
                    // Diffuse
                    localColor = localColor / diffuseWeight * rec.material.diffuse;
                    currentRay = Ray(rec.p, rec.normal + random_unit_vector(rec.p.xy * rng));
                }

            } else {
                // Sky
                let unit_direction = normalize(currentRay.direction);
                let t = 2.0 * max(0.0, unit_direction.y);
                let skyColor = mix(vec4<f32>(0.8, 0.8, 0.8, 1.0), vec4<f32>(0.5, 0.7, 1.0, 1.0), t);
                localColor = localColor * skyColor;
                break;
            }
        }
        outColor = outColor + localColor;
    }
    outColor = outColor / vec4<f32>(f32(spp));

    return outColor;
}

fn get_offset_ray(ray: Ray, pixelSize: vec2<f32>, focusDistance: f32, rng: vec2<f32>) -> Ray {
    // Anti-aliasing and dof offset
    var aaOffset = vec3<f32>((rng.x - 0.5) * pixelSize.x, (rng.y - 0.5) * pixelSize.y, 0.0);
    var aaDirection = normalize(ray.direction + aaOffset);
    let dof = random_in_unit_disk(rng) * scene.camera.aperture;
    let origin = ray.origin + dof;
    let direction = normalize((aaDirection * focusDistance - dof));

    return Ray(origin, direction);
}
