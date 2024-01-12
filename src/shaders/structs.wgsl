const MAX_SIZE: i32 = 1280 * 720;

struct CameraUniform {
    origin: vec4<f32>,
    focus: vec4<f32>,
    up: vec4<f32>,
    fovy: f32,
    aspect: f32,
    znear: f32,
    zfars: f32,
};

struct CameraMatrixUniform {
    view_proj: mat4x4<f32>,
    origin: vec3<f32>,
};

struct Sphere {
    center: vec3<f32>,
    radius: f32,
};

struct Scene {
    objects: array<Sphere>,
};

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>
}

struct RayBuffer {
    data: array<Ray, MAX_SIZE>,
    size: vec2<u32>
}