// Global constants
const PI: f32 = 3.1415926535897932384626433832795;
const TWO_PI: f32 = 2.0 * PI;
const EPSILON: f32 = 1e-5;

// Scene object types in place of an enum
const TRIANGLE_TYPE: u32 = 0u;
const SPHERE_TYPE: u32 = 1u;
const PLANE_TYPE: u32 = 2u;
const QUADLIGHT_TYPE: u32 = 3u;

// Nulls
const NULL_MATERIAL = Material(vec4<f32>(0.0, 0.0, 0.0, 0.0), 0.0, 0.0, 0.0, 0.0, 1.5);
const NULL_HIT = HitRec(-1.0, vec3<f32>(0.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 0.0), NULL_MATERIAL, true);

// Sizes
const MAX_BVH_SIZE: u32 = 1024;


struct SceneObject {
    objectType: u32,
    index: u32,
};

struct QuadLight {
    position: vec4<f32>,
    normal: vec4<f32>,
    u: vec4<f32>,
    v: vec4<f32>,
    color: vec3<f32>,
    intensity: f32,
};

struct QuadLightBuffer {
    data: array<QuadLight>,
};

struct Triangle {
    indices: vec3<u32>,
    material: u32,
};

struct TriangleBuffer {
    data: array<Triangle>,
}

struct MaterialBuffer {
    data: array<Material>,
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
    // sss: SSSData,
}

struct RenderConfig {
    seed: vec4<u32>,
    size: vec2<u32>,
    pixel_size: vec2<f32>,
    max_depth: u32,
    samples: u32,
    num_objects: u32,
    count: u32,
    sky_intensity: f32,
    sky_color: vec4<f32>,
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
    frontface: bool,
}

struct GGX {
    direction: vec3<f32>,
    weight: vec4<f32>,
} 

struct PixelBuffer {
    data: array<vec4<f32>>
}

struct VertexBuffer {
    data: array<vec4<f32>>
}

struct NormalBuffer {
    data: array<vec3<f32>>
}

struct AABB {
    min: vec4<f32>,
    max: vec4<f32>,
}

struct BVHNode {
    aabb: AABB,
    left: i32,
    right: i32,
    triangle: i32,
}

struct BVHBuffer {
    root: i32,
    nodes: array<BVHNode>,
}

struct LightSample {
    color: vec4<f32>,
    dir: vec3<f32>,
}

struct WithSky {
    color: vec4<f32>,
    sky: vec4<f32>,
}

struct CosineDiffuse {
    dir: vec3<f32>,
    pdf: f32,
}

struct SSSData {
    scatter_coeff: vec4<f32>,
    absorption_coeff: vec4<f32>,
    scale: f32,
    anisotropy: f32,
}
