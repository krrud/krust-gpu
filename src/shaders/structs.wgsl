const PI: f32 = 3.1415926535897932384626433832795;
const EPSILON: f32 = 1e-6;

struct CameraUniform {
    origin: vec4<f32>,
    focus: vec4<f32>,
    aperture: f32,
    fovy: f32,
    aspect: f32,
};

struct CameraMatrixUniform {
    view_proj: mat4x4<f32>,
    origin: vec3<f32>,
};

struct Sphere {
    center: vec3<f32>,
    radius: f32,
    material: Material,
};

struct Scene {
    config: RenderConfig,
    camera: CameraUniform,
    objects: array<Sphere>,
}

struct RenderConfig {
    size: vec2<u32>,
    pixel_size: vec2<f32>,
    max_depth: u32,
    samples: u32,
    num_objects: u32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>
}

struct RayBuffer {
    size: vec2<u32>,
    data: array<Ray>,
}

struct Material {
    diffuse: vec4<f32>,
    specular: f32,
    roughness: f32,
    metallic: f32,
    refract: f32,
    ior: f32,
}

struct HitRec {
    t: f32,
    p: vec3<f32>,
    normal: vec3<f32>,
    material: Material,
}

struct GGX {
    direction: vec3<f32>,
    weight: vec4<f32>,
} 

