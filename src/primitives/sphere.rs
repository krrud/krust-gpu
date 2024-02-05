use crate::primitives::material::Material;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
    center: [f32; 3],
    radius: f32,
    material: Material,
}

impl Sphere {
    pub fn new(center: [f32; 3], radius: f32, material: Material) -> Self {
        Sphere {center, radius, material}
    }
}