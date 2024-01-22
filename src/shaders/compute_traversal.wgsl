// Scene traversal shader
@group(0) @binding(0) var<storage, read> scene: Scene;
@group(0) @binding(1) var<storage, read> rays: RayBuffer;
@group(0) @binding(2) var outputTex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(3) var t_sky: texture_2d<f32>;
@group(0) @binding(4) var s_sky: sampler;
@group(0) @binding(5) var<storage, read> sphereBuffer: SphereBuffer;
@group(0) @binding(6) var<storage, read> triangleBuffer: TriangleBuffer;
@group(0) @binding(7) var<storage, read> quadLightBuffer: QuadLightBuffer;


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

    let test = quadLightBuffer.data[0];
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
            let rec = hit_scene(currentRay);

            if (rec.t > 0.0) {
                // Handle material interaction
                var diffuseWeight = saturate(1.0 - rec.material.metallic);
                let specularWeight = rec.material.specular / (rec.material.specular + diffuseWeight);
                diffuseWeight = diffuseWeight * (1.0 - specularWeight);
                let isMetallic = rec.material.metallic > rng.x;
                let isSpecular = specularWeight > rng.y || isMetallic;

                // Light info for direct lighting
                let light = quadLightBuffer.data[0];// Position of the light source
                let lightDist = length(light.position.xyz - rec.p);
                let lightColor = vec4<f32>(light.color * light.intensity * light.intensity / (lightDist * lightDist), 0.0);
                var lightShadow = 0.0;
                let lightSize = length(cross(light.u.xyz, light.v.xyz)); 
                let lightSample = random_on_quad(light.position.xyz, light.normal.xyz, light.u.xyz, light.v.xyz, rng);
                let lightDir = normalize(lightSample - rec.p);
                let lightCos = dot(rec.normal.xyz, lightDir);
                let lightRay = Ray(rec.p, lightDir);
                let lightRec = hit_scene(lightRay);
                if (lightRec.t <= 0.0 || lightRec.t >= lightDist) {
                    lightShadow = lightShadow + 1.0;
                }
                let lightWeight = saturate(lightCos) * lightShadow;


                if (isSpecular) {
                    // Sepcular -- GGX
                    let directWeight = 0.5;
                    let isDirect = rng.x < directWeight;

                    // Indirect
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

                    if (isDirect) {
                        // Direct specular
                        let ggx_direct = ggx_direct(
                            rec.normal, 
                            -currentRay.direction, 
                            lightDir, 
                            rec.material.roughness, 
                            f0
                        );

                        localColor = localColor / specularWeight * ggx_direct.weight * lightColor * lightWeight / directWeight;
                        currentRay = Ray(rec.p, lightDir);

                    } else {
                        // Indirect specular
                        let ggx_indirect = ggx_indirect(
                            rec.normal, 
                            -currentRay.direction, 
                            rec.material.roughness, 
                            f0,
                            rng
                            );
    
                        localColor = localColor / specularWeight * ggx_indirect.weight / directWeight;
                        currentRay = Ray(rec.p, reflect(currentRay.direction, ggx_indirect.direction));  
                    }
                } else {
                    // Diffuse

                    // Direct lighting                        
                    let directColor = localColor * rec.material.diffuse * lightColor * lightWeight;
                                                              
                    // Indirect lighting
                    var offset = rec.p.xy;
                    let threshold = 0.10;
                    if (abs(1.0 - rec.normal.y) < threshold) {
                        offset = rec.p.xz;
                    };
                    let indirectColor = localColor / diffuseWeight * rec.material.diffuse;
                    localColor = directColor + indirectColor;
                    currentRay = Ray(rec.p, rec.normal + random_unit_vector(offset * rng)); 
                    
                }
            } else {
                // Sky
                let uv = vec2<f32>(atan2(currentRay.direction.z, currentRay.direction.x) / (PI * 2.0) + 0.5, -asin(currentRay.direction.y) / PI + 0.5);
                let skyColor = textureSampleLevel(t_sky, s_sky, uv, 0.0);
                localColor = localColor * skyColor * max(0.001, scene.config.sky_intensity);
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

fn hit_scene(currentRay: Ray) -> HitRec {
    var i = 0u;
    var rec = NULL_HIT;

    while (i < scene.config.num_objects) {
        // Find closest hit
        var hit = rec;
        if (scene.objects[i].objectType == SPHERE_TYPE) {
            // We hit a sphere
            hit = hit_sphere(sphereBuffer.data[i], currentRay);
        } 
        // else if (scene.objects[i].objectType == TRIANGLE_TYPE) {
        //     // We hit a triangle
        //     hit = hit_triangle(triangleBuffer.data[0], currentRay);
        // }
        if (hit.t > 0.0 && (rec.t < 0.0 || hit.t < rec.t)) {
            // We hit a thing and it's closer than the last thing
            rec = hit;
        }
        i = i + 1u;
    }
    return rec;
}

