// INTERSECTIONS
fn hit_sphere(sphere: Sphere, ray: Ray) -> HitRec {
    let oc: vec3<f32> = ray.origin - sphere.center;
    let a: f32 = dot(ray.direction, ray.direction);
    let b: f32 = 2.0 * dot(oc, ray.direction);
    let c: f32 = dot(oc, oc) - sphere.radius * sphere.radius;
    let discriminant: f32 = b * b - 4.0 * a * c;

    if discriminant >= 0.0 {
        let t: f32 = (-b - sqrt(discriminant)) / (2.0 * a);
        if t > 0.0 {
            let p: vec3<f32> = point_at(ray, t);
            let normal: vec3<f32> = (p - sphere.center) / sphere.radius;
            let frontface = dot(ray.direction, normal) < 0.0;
            return HitRec(t, p, normal, sphere.material, frontface);
        }    
    }

    return NULL_HIT;
}

fn hit_triangle(triangle: Triangle, ray: Ray) -> HitRec {
    let a = vertexBuffer.data[triangle.indices.x].xyz;
    let b = vertexBuffer.data[triangle.indices.y].xyz;
    let c = vertexBuffer.data[triangle.indices.z].xyz;
    let e1: vec3<f32> = b - a;
    let e2: vec3<f32> = c - a;
    let p: vec3<f32> = cross(ray.direction, e2);
    let det: f32 = dot(e1, p);

    if (det > -EPSILON && det < EPSILON) {
        return NULL_HIT;
    }

    let inv_det = 1.0 / det;
    let s: vec3<f32> = ray.origin - a;
    let u: f32 = dot(s, p) * inv_det;

    if (u < 0.0 || u > 1.0) {
        return NULL_HIT;
    }

    let q: vec3<f32> = cross(s, e1);
    let v: f32 = dot(ray.direction, q) * inv_det;

    if (v < 0.0 || u + v > 1.0) {
        return NULL_HIT;
    }

    let t = dot(e2, q) * inv_det;
    
    if (t > EPSILON) {
        let p: vec3<f32> = point_at(ray, t);
        let na = normalBuffer.data[triangle.indices.x].xyz;
        let nb = normalBuffer.data[triangle.indices.y].xyz;
        let nc = normalBuffer.data[triangle.indices.z].xyz;
        var normal: vec3<f32> = normalize((1.0 - u - v) * na + u * nb + v * nc);
        let frontface = dot(ray.direction, normal) < 0.0;
        if (!frontface) {
            normal = -normal;
        }
        return HitRec(t, p, normal, materialBuffer.data[triangle.material], frontface);
    }
    else {
        return NULL_HIT;
    }
}

fn hit_aabb(ray: Ray, box: AABB) -> bool {
    let t1: vec3<f32> = (box.min.xyz - ray.origin.xyz) / ray.direction.xyz;
    let t2: vec3<f32> = (box.max.xyz - ray.origin.xyz) / ray.direction.xyz;

    let tmin: vec3<f32> = min(t1, t2);
    let tmax: vec3<f32> = max(t1, t2);

    if tmax.x < max(tmin.y, tmin.z) || tmax.y < max(tmin.x, tmin.z) || tmax.z < max(tmin.x, tmin.y) {
        return false;
    }

    return tmax.x >= 0.0 && tmax.y >= 0.0 && tmax.z >= 0.0;
}

fn distance_to_aabb(ray: Ray, box: AABB) -> f32 {
    let t1: vec3<f32> = (box.min.xyz - ray.origin.xyz) / ray.direction.xyz;
    let t2: vec3<f32> = (box.max.xyz - ray.origin.xyz) / ray.direction.xyz;

    let tmin: vec3<f32> = min(t1, t2);
    let tmax: vec3<f32> = max(t1, t2);

    let tmin_final: f32 = max(max(tmin.x, tmin.y), tmin.z);
    let tmax_final: f32 = min(min(tmax.x, tmax.y), tmax.z);

    if (tmax_final < 0.0) {
        return -1.0;
    }

    return max(0.0, tmin_final);
}

fn hit_bvh(ray: Ray) -> HitRec {
    var rec: HitRec = NULL_HIT;
    var stack: array<i32, 128>;
    var stack_top: i32 = 0;
    stack[stack_top] = bvhBuffer.root;

    while (stack_top >= 0) {
        let node_index: i32 = stack[stack_top];
        stack_top -= 1;

        let node = bvhBuffer.nodes[node_index];

        if (!hit_aabb(ray, node.aabb)) {
            continue;
        }

        while (node.triangle < 0) {
            // Branch node
            let left_node = bvhBuffer.nodes[node.left];
            let right_node = bvhBuffer.nodes[node.right];
            let left_t = distance_to_aabb(ray, left_node.aabb);
            let right_t = distance_to_aabb(ray, right_node.aabb);

            // Both children are behind the ray
            if (left_t < 0.0 && right_t < 0.0) {
                break;
            }

            // Prioritize closest
            if (left_t >= 0.0 && right_t >= 0.0) {
                if (left_t < right_t) {
                    stack_top += 1;
                    stack[stack_top] = node.right;
                    stack_top += 1;
                    stack[stack_top] = node.left;
                } else {
                    stack_top += 1;
                    stack[stack_top] = node.left;
                    stack_top += 1;
                    stack[stack_top] = node.right;
                }
            } else {
                // Only add
                if (left_t >= 0.0) {
                    stack_top += 1;
                    stack[stack_top] = node.left;
                } else {
                    stack_top += 1;
                    stack[stack_top] = node.right;
                }
            }

            break;
        }

        if (node.triangle >= 0) {
            // Leaf node
            let hit: HitRec = hit_triangle(triangleBuffer.data[node.triangle], ray);
            if (hit.t > 0.0 && (rec.t < 0.0 || hit.t < rec.t)) {
                rec = hit;
            }
        }
    }
    return rec;
}


// CAMERA
fn point_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + ray.direction * t;
}

fn get_offset_ray(ray: Ray, pixelSize: vec2<f32>, focusDistance: f32, rng: vec2<f32>) -> Ray {
    var aaOffset = vec3<f32>((rng.x - 0.5) * pixelSize.x, (rng.y - 0.5) * pixelSize.y, 0.0);
    var aaDirection = normalize(ray.direction + aaOffset);
    let dof = random_in_unit_disk(rng) * scene.camera.aperture;
    let origin = ray.origin + dof;
    let direction = normalize((aaDirection * focusDistance - dof));

    return Ray(origin, direction);
}

fn get_strat_offset_ray(ray: Ray, pixelSize: vec2<f32>, focusDistance: f32, rng: vec2<f32>, count: u32) -> Ray {
    let gridSize = 4.0;
    let wrappedCount = count % u32(gridSize * gridSize);
    let gridPos = vec2<f32>(f32(wrappedCount % u32(gridSize)), f32(wrappedCount / u32(gridSize)));
    let stratifiedRng = (gridPos + rng) / gridSize;

    var aaOffset = vec3<f32>((stratifiedRng.x - 0.5) * pixelSize.x, (stratifiedRng.y - 0.5) * pixelSize.y, 0.0);
    var aaDirection = normalize(ray.direction + aaOffset);
    var dof = random_in_unit_disk(rng) * scene.camera.aperture;
    let origin = ray.origin + dof;
    let direction = normalize((aaDirection * focusDistance - dof));

    return Ray(origin, direction);
}

// RANDOM
fn hash_f32(n: f32) -> f32 {
    let a =  0.61803398875;
    return fract(sin(n * a) * 43758.5453);
}

fn hash_u32(seed: u32) -> f32 {
    var s: u32 = seed;
    s ^= s >> 15u;
    s *= 0x2c1b3c6du;
    s ^= s >> 12u;
    s *= 0x297a4d43u;
    s ^= s >> 15u;

    return f32(s) / 4294967295.0;
}

fn random_unit_vector(rng: vec2<f32>) -> vec3<f32> {
    let r1: f32 = hash_f32(rng.x);
    let r2: f32 = hash_f32(rng.y);

    let theta: f32 = 2.0 * 3.14159265359 * r1; 
    let phi: f32 = acos(1.0 - 2.0 * r2); 

    let x: f32 = sin(phi) * cos(theta);
    let y: f32 = sin(phi) * sin(theta);
    let z: f32 = cos(phi);

    return normalize(vec3<f32>(x, y, z));
}

fn random_in_unit_disk(rng: vec2<f32>) -> vec3<f32> {
    let theta = 2.0 * PI * hash_f32(rng.x);
    let r = sqrt(hash_f32(rng.y));
    let x = r * cos(theta);
    let y = r * sin(theta);
    return vec3<f32>(x, y, 0.0);
}

fn random_on_quad(p: vec3<f32>, n: vec3<f32>, u: vec3<f32>, v: vec3<f32>, rng: vec2<f32>) -> vec3<f32> {
    return p + rng.x * u + rng.y * v;
}


// LIGHTING AND MATERIALS
fn schlick(cosine: f32, ior: f32) -> f32 {
    var f0 = (1.0 - ior) / (1.0 + ior);
    f0 = f0 * f0;
    return f0 + (1.0 - f0) * pow(1.0 - cosine, 5.0);
}

// fn ambient_occlusion(rec: HitRec, samples: i32, rng: vec2<f32>) -> vec4<f32> {
//     var occlusion = 0.0;
//     for (var i = 0; i < samples; i = i + 1) {
//         let sampleDir = cosine_weighted_hemisphere(rec, rng);
//         let sampleRay = Ray(rec.p, sampleDir);
//         let sampleRec = hit_bvh(sampleRay);
//         if (sampleRec.t > 0.0) {
//             occlusion += 1.0;
//         }
//     }
//     occlusion = occlusion / f32(samples);
//     occlusion = 1.0 - occlusion;
//     return vec4<f32>(occlusion, occlusion, occlusion, 1.0);
// }

fn cosine_weighted_hemisphere(rec: HitRec, rng: vec2<f32>) -> CosineDiffuse {
    let phi = TWO_PI * rng.x;
    let cosTheta = sqrt(1.0 - rng.y);
    let sinTheta = sqrt(rng.y);

    let localDir = vec3<f32>(cos(phi) * sinTheta, cosTheta, sin(phi) * sinTheta);

    let worldUp = vec3<f32>(0.0, 1.0, 0.0);
    let worldNormal = normalize(rec.normal.xyz);
    let worldTangent = normalize(cross(worldUp, worldNormal));
    let worldBitangent = normalize(cross(worldNormal, worldTangent));

    let direction = normalize(worldTangent * localDir.x + worldNormal * localDir.y + worldBitangent * localDir.z);
    let pdf = abs(dot(direction, rec.normal)) / PI;

    return CosineDiffuse(direction, pdf);
}

fn sample_quad_light(light: QuadLight, rec: HitRec, rng: vec2<f32>) -> LightSample {
    let lightSample = random_on_quad(light.position.xyz, light.normal.xyz, light.u.xyz, light.v.xyz, rng);
    let lightDist = length(lightSample - rec.p);
    let lightSize = length(cross(light.u.xyz, light.v.xyz)); 
    let toLight = normalize(lightSample - rec.p);
    let lightCos = dot(rec.normal.xyz, toLight);
    let lightRay = Ray(rec.p, toLight);
    let lightRec = hit_bvh(lightRay);
    var lightShadow = 0.0;
    if (lightRec.t > 0.0 && lightRec.t < lightDist){
        return LightSample(vec4<f32>(0.0, 0.0, 0.0, 0.0), toLight);
    }
    let lightPdf = lightDist * lightDist / (lightSize * abs(lightCos));
    let lightWeight = max(dot(rec.normal.xyz, toLight), 0.0) / lightPdf;
    let lightColor = vec4<f32>(light.color * light.intensity / (lightDist * lightDist), 0.0);
    return LightSample(lightColor * lightWeight, toLight);
}

fn sample_sky(ray: Ray, intensity: f32) -> vec4<f32> {
    let uv = vec2<f32>(atan2(ray.direction.z, ray.direction.x) / (PI * 2.0) + 0.5, -asin(ray.direction.y) / PI + 0.5);
    let color = textureSampleLevel(t_sky, s_sky, uv, 0.0);
    return color * intensity;
}

fn is_nan(v: vec4<f32>) -> bool {
    if (v.x != v.x || v.y != v.y || v.z != v.z || v.w != v.w) {
        return true;
    }
    return false;
}