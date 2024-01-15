use crate::primitives::material::Material;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    a: [f32; 4],
    b: [f32; 4],
    c: [f32; 4],
    material: Material,
    _padding: [u32; 3],
}

impl Triangle {
    pub fn new(a: [f32; 3], b: [f32; 3], c: [f32; 3], material: Material) -> Self {
        Self {
            a: [a[0], a[1], a[2], 0.0],
            b: [b[0], b[1], b[2], 0.0],
            c: [c[0], c[1], c[2], 0.0],
            material,
            _padding: [0; 3],
        }
    }
}