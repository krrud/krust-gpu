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
    let ray = generateRay(uv);

    // Write to buffer
    let index = ix.y * rays.size.x + ix.x;
    rays.data[index] = ray;
}


fn generateRay(uv: vec2<f32>) -> Ray {
    let forward = normalize(camera.focus.xyz - camera.origin.xyz);
    let right = normalize(cross(camera.up.xyz, forward));
    let up = cross(forward, right);

    let fovy = radians(camera.fovy);
    let aspect = camera.aspect;

    let csc = vec2<f32>(
        (uv.x * 2.0 - 1.0) * aspect * tan(fovy / 2.0),
        (1.0 - uv.y * 2.0) * tan(fovy / 2.0)
    );

    let direction = normalize(csc.x * right + csc.y * up + forward);

    return Ray(camera.origin.xyz, direction);
}