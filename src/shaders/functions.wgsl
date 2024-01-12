fn look_at(eye: vec3<f32>, center: vec3<f32>, up: vec3<f32>) -> mat4x4<f32> {
    let f = normalize(center - eye);
    let r = normalize(cross(up, f));
    let u = cross(f, r);

    let m = mat4x4<f32>(
        vec4<f32>(r.x, u.x, f.x, 0.0),
        vec4<f32>(r.y, u.y, f.y, 0.0),
        vec4<f32>(r.z, u.z, f.z, 0.0),
        vec4<f32>(-dot(r, eye), -dot(u, eye), -dot(f, eye), 1.0)
    );

    return m;
}

fn perspective(fovy: f32, aspect: f32, znear: f32, zfar: f32) -> mat4x4<f32> {
    let tanHalfFovy = tan(fovy / 2.0);

    let m = mat4x4<f32>(
        vec4<f32>(1.0 / (aspect * tanHalfFovy), 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0 / tanHalfFovy, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, -(zfar + znear) / (zfar - znear), -1.0),
        vec4<f32>(0.0, 0.0, -2.0 * zfar * znear / (zfar - znear), 0.0)
    );

    return m;
}