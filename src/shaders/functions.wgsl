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

    return NULL_HIT;
}

fn hit_triangle(triangle: Triangle, ray: Ray) -> HitRec {
    let e1: vec3<f32> = triangle.b.xyz - triangle.a.xyz;
    let e2: vec3<f32> = triangle.c.xyz - triangle.a.xyz;
    let p: vec3<f32> = cross(ray.direction, e2);
    let det: f32 = dot(e1, p);

    if (det > -EPSILON && det < EPSILON) {
        return NULL_HIT;
    }

    let inv_det = 1.0 / det;
    let s: vec3<f32> = ray.origin - triangle.a.xyz;
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
        let normal: vec3<f32> = normalize(cross(e1, e2));
        return HitRec(t, p, normal, triangle.material);
    }
    else {
        return NULL_HIT;
    }


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

// fn rand(rnd: vec3<f32>) -> f32 {
//   const C = vec3(60493  * 9377,
//                  11279  * 2539 * 23,
//                  7919   * 631  * 5 * 3);

//   rnd = (rnd * C) ^ (rnd.yzx >> vec3(4u));
//   return f32(rnd.x ^ rnd.y) / f32(0xffffffff);
// }

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
    loop {
        var p = vec3<f32>(hash_f32(rng.x) * 2.0 - 1.0, hash_f32(rng.y) * 2.0 - 1.0, 0.0);
        if (dot(p.xy, p.xy) < 1.0) {
            return p;
        }
    }
    return vec3<f32>(0.0, 0.0, 0.0);
}

fn random_on_quad(p: vec3<f32>, n: vec3<f32>, u: vec3<f32>, v: vec3<f32>, rng: vec2<f32>) -> vec3<f32> {
    // sample a random point on a quad
    return p + rng.x * u + rng.y * v;
}