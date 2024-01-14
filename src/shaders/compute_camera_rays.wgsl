// Camera ray generation shader
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<storage, read_write> rays: RayBuffer;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_ix: vec3<u32>) {
    // Get fragment coordinate
    let ix = global_ix.xy;
    let coord = vec2<f32>(f32(ix.x), f32(ix.y));
    let pixelDim = vec2<f32>(f32(rays.size.x), f32(rays.size.y)); 

    // Generate ray
    let uv = vec2<f32>(coord) / pixelDim;
    let ray = create_primary_ray(uv);

    // Write to buffer
    let index = ix.y * rays.size.x + ix.x;
    rays.data[index] = ray;
}


fn create_primary_ray(uv: vec2<f32>) -> Ray {
    let forward = normalize(camera.focus.xyz - camera.origin.xyz);
    let right = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), forward));
    let up = cross(forward, right);
    let fov = radians(camera.fovy);

    let csc = vec2<f32>(
        (uv.x * 2.0 - 1.0) * camera.aspect * tan(fov / 2.0),
        (1.0 - uv.y * 2.0) * tan(fov / 2.0)
    );

    let direction = normalize(csc.x * right + csc.y * up + forward);

    return Ray(camera.origin.xyz, direction);
}