// INTERSECTIONS
//
//
//
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
            return HitRec(t, p, normal, sphere.material);
        }    
    }

    let nullMaterial = Material(vec4<f32>(0.0, 0.0, 0.0, 0.0), 0.0, 0.0, 0.0, 0.0, 1.5);
    let nullHit = HitRec(-1.0, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0), nullMaterial);
    return nullHit;
}


// RAY AND CAMERA
//
//
//
fn point_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + ray.direction * t;
}


// MATERIALS
//
//
//
fn schlick(cosine: f32, ref_idx: f32) -> f32 {
    var r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
    r0 = r0 * r0;
    return r0 + (1.0 - r0) * pow(1.0 - cosine, 5.0);
}


// RANDOM
// 
// 
//
fn hash_simple(n: f32) -> f32 {
    let a = 12.9898;
    return fract(sin(n * a) * 43758.5453);
}

fn hash(s: u32) -> f32 {
    var seed: u32 = (s ^ 61u) ^ (s >> 16u);
    seed *= 9u;
    seed = seed ^ (seed >> 4u);
    seed *= 0x27d4eb2du;
    seed = seed ^ (seed >> 15u);

    return f32(seed) / 4294967295.0;
}

fn random_unit_vector(rng: vec2<f32>) -> vec3<f32> {
    let r1: f32 = hash_simple(rng.x);
    let r2: f32 = hash_simple(rng.y);

    let theta: f32 = 2.0 * 3.14159265359 * r1; 
    let phi: f32 = acos(1.0 - 2.0 * r2); 

    let x: f32 = sin(phi) * cos(theta);
    let y: f32 = sin(phi) * sin(theta);
    let z: f32 = cos(phi);

    return normalize(vec3<f32>(x, y, z));
}

fn random_in_unit_disk(rng: vec2<f32>) -> vec3<f32> {
    loop {
        var p = vec3<f32>(hash_simple(rng.x) * 2.0 - 1.0, hash_simple(rng.y) * 2.0 - 1.0, 0.0);
        if (dot(p.xy, p.xy) < 1.0) {
            return p;
        }
    }
    return vec3<f32>(0.0, 0.0, 0.0);
}

// fn random_in_unit_disk(rng: vec2<f32>) -> vec3<f32> {
//     let r1: f32 = hash_simple(rng.x);
//     let r2: f32 = hash_simple(rng.y);
    
//     let theta: f32 = 2.0 * 3.14159265359 * r2;
//     let x: f32 = r1 * cos(theta);
//     let y: f32 = r1 * sin(theta);
//     return vec3<f32>(x, y, 0.0);
// }