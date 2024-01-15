// Scene traversal shader
@group(0) @binding(0) var<storage, read> scene: Scene;
@group(0) @binding(1) var<storage, read> rays: RayBuffer;
@group(0) @binding(2) var outputTex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var t_sky: texture_2d<f32>;
@group(0) @binding(4) var s_sky: sampler;
@group(0) @binding(5) var<storage, read> sphereBuffer: SphereBuffer;
@group(0) @binding(6) var<storage, read> triangleBuffer: TriangleBuffer;


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

    textureStore(outputTex, vec2<i32>(ix), pixelColor);
}

fn sample_scene(ray: Ray, maxDepth: u32, spp: u32, pixelSize: vec2<f32>, globalIdx: u32) -> vec4<f32> {
    var outColor = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let focusDistance: f32 = length(ray.origin);

    for (var sampleIdx = 0u; sampleIdx < spp; sampleIdx = sampleIdx + 1u) {
        // Instantiate random number generator -- pixel color -- current ray
        let seed = sampleIdx * globalIdx + sampleIdx + 999u * globalIdx;
        let rng = vec2<f32>(hash_u32(seed * scene.config.seed.x), hash_u32(seed * scene.config.seed.y));
        var localColor = vec4<f32>(1.0, 1.0, 1.0, 1.0);
        var currentRay = get_offset_ray(ray, pixelSize, focusDistance, rng);

        for (var depth = maxDepth; depth > 0u; depth = depth - 1u) {
            // Rays be bouncing
            var i = 0u;
            var rec = NULL_HIT;

            while (i < scene.config.num_objects) {
                // Find closest hit
                var hit = rec;
                if (scene.objects[i].objectType == SPHERE_TYPE) {
                    // We hit a sphere
                    hit = hit_sphere(sphereBuffer.data[i], currentRay);
                } 
                else if (scene.objects[i].objectType == TRIANGLE_TYPE) {
                    // We hit a triangle
                    hit = hit_triangle(triangleBuffer.data[0], currentRay);
                }
                if (hit.t > 0.0 && (rec.t < 0.0 || hit.t < rec.t)) {
                    // We hit a thing and it's closer than the last thing
                    rec = hit;
                }
                i = i + 1u;
            }

            if (rec.t > 0.0) {
                // Handle material interaction
                var diffuseWeight = saturate(1.0 - rec.material.metallic);
                let specularWeight = rec.material.specular / (rec.material.specular + diffuseWeight);
                diffuseWeight = diffuseWeight * (1.0 - specularWeight);
                let isMetallic = rec.material.metallic > rng.x;
                let isSpecular = specularWeight > rng.y || isMetallic;

                if (isSpecular) {
                    // Sepcular -- GGX
                    let specularColor = vec4<f32>(1.0, 1.0, 1.0, 1.0) * rec.material.specular;
                    var f0: vec3<f32>;
                    if (isMetallic) {
                        // Conductive                        
                        localColor = localColor * rec.material.diffuse;
                        f0 = vec3<f32>(0.9, 0.9, 0.9);
                    } else {
                        // Dielectric
                        localColor = localColor * specularColor;
                        f0 = vec3<f32>(0.04, 0.04, 0.04);
                    }

                    // Sample GGX distribution
                    let ggx = ggx_indirect(
                        rec.normal, 
                        normalize(-currentRay.direction), 
                        rec.material.roughness, 
                        f0,
                        rng
                        );

                    // Set ray direction and accumulate color    
                    currentRay = Ray(rec.p, reflect(currentRay.direction, ggx.direction));
                    localColor = localColor / specularWeight * ggx.weight;

                } else {
                    // Diffuse -- cosine weighted
                    var offset = rec.p.xy;
                    let threshold = 0.10;
                    if (abs(1.0 - rec.normal.y) < threshold) {
                        offset = rec.p.xz;
                    };
                    localColor = localColor / diffuseWeight * rec.material.diffuse;
                    currentRay = Ray(rec.p, rec.normal + random_unit_vector(offset * rng));
                }

            } else {
                // Sky

                // let unit_direction = normalize(currentRay.direction);
                // let t = 2.0 * max(0.0, unit_direction.y);
                // let skyColor = mix(vec4<f32>(0.8, 0.8, 0.8, 1.0), vec4<f32>(0.5, 0.7, 1.0, 1.0), t);
                // localColor = localColor * skyColor;

                let uv = vec2<f32>(atan2(currentRay.direction.z, currentRay.direction.x) / (PI * 2.0) + 0.5, -asin(currentRay.direction.y) / PI + 0.5);
                let skyColor = textureSampleLevel(t_sky, s_sky, uv, 0.0);
                localColor = localColor * skyColor * 0.5;
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

