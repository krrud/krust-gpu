#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    pub diffuse: [f32; 4],
    pub specular: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub refract: f32,
    pub ior: f32,
}

impl Material {
    pub fn new(
        diffuse: [f32; 4],
        specular: f32,
        roughness: f32,
        metallic: f32,
        refract: f32,
        ior: f32,
    ) -> Self {
        Material {
            diffuse,
            specular,
            roughness,
            metallic,
            refract,
            ior,
        }
    }
}