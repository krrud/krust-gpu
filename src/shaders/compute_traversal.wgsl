struct HitRec {
    t: f32,
    p: vec3<f32>,
    normal: vec3<f32>,
}

// Scene traversal shader
@group(0) @binding(0) var<storage, read> scene: Scene;
@group(0) @binding(1) var<storage, read> rays: RayBuffer;
@group(0) @binding(2) var outputTex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    let ix = global_ix.xy;
    let ray = rays.data[ix.y * u32(rays.size.x) + ix.x];
    let pixelColor = trace_scene(ray, 5);
    textureStore(outputTex, vec2<i32>(ix), pixelColor);
}

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
            return HitRec(t, p, normal);
        }    
    }
    return HitRec(-1.0, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0));
}


fn trace_scene(ray: Ray, maxDepth: i32) -> vec4<f32> {
    if maxDepth == 0 {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }

    var i: i32 = 0;
    var rec = HitRec(-1.0, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0));

    while (i < 2) {
        let hit = hit_sphere(scene.objects[i], ray);
        if (hit.t > 0.0 && (rec.t < 0.0 || hit.t < rec.t)) {
            rec = hit;
        }
        i = i + 1;
    }

    if (rec.t > 0.0) {
        return vec4<f32>(rec.normal * 0.5 + 0.5, 1.0);
    }

    // Sky
    let unit_direction = normalize(ray.direction);
    let t = 0.5 * (unit_direction.y + 1.0);
    return (1.0 - t) * vec4<f32>(1.0, 1.0, 1.0, 1.0) + t * vec4<f32>(0.5, 0.7, 1.0, 1.0);
}


fn point_at(ray: Ray, t: f32) -> vec3<f32> {
    return ray.origin + ray.direction * t;
}