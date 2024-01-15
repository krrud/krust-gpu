// Global constants
const PI: f32 = 3.1415926535897932384626433832795;
const EPSILON: f32 = 1e-6;

// Scene object types in place of an enum
const TRIANGLE_TYPE: u32 = 0u;
const SPHERE_TYPE: u32 = 1u;
const PLANE_TYPE: u32 = 2u;
const QUADLIGHT_TYPE: u32 = 3u;

// Nulls
const NULL_MATERIAL = Material(vec4<f32>(0.0, 0.0, 0.0, 0.0), 0.0, 0.0, 0.0, 0.0, 1.5);
const NULL_HIT = HitRec(-1.0, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0), NULL_MATERIAL);

struct SceneObject {
    objectType: u32,
    index: u32,
};

struct QuadLight {
    center: vec3<f32>,
    normal: vec3<f32>,
    u: vec3<f32>,
    v: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
};

struct Triangle {
    a: vec4<f32>,
    b: vec4<f32>,
    c: vec4<f32>,
    material: Material,
};

struct TriangleBuffer {
    data: array<Triangle>,
}

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

struct SphereBuffer {
    data: array<Sphere>,
};

struct Scene {
    config: RenderConfig,
    camera: CameraUniform,
    objects: array<SceneObject>,
}

struct RenderConfig {
    seed: vec4<u32>,
    size: vec2<u32>,
    pixel_size: vec2<f32>,
    max_depth: u32,
    samples: u32,
    num_objects: u32,
    count: u32,
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

struct PixelBuffer {
    data: array<vec4<f32>>
}

