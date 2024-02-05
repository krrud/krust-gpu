use crate::primitives::material::Material;
use crate::primitives::aabb::AABB;


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    a: [f32; 4],
    b: [f32; 4],
    c: [f32; 4],
    material: Material,
}

impl Triangle {
    pub fn new(a: [f32; 3], b: [f32; 3], c: [f32; 3], material: Material) -> Self {
        Self {
            a: [a[0], a[1], a[2], 0.0],
            b: [b[0], b[1], b[2], 0.0],
            c: [c[0], c[1], c[2], 0.0],
            material,
        }
    }

    pub fn bounding_box(&self, _t0: f32, _t1: f32) -> AABB {
        let min = [
            self.a[0].min(self.b[0]).min(self.c[0]),
            self.a[1].min(self.b[1]).min(self.c[1]),
            self.a[2].min(self.b[2]).min(self.c[2]),
            0.0,
        ];
        let max = [
            self.a[0].max(self.b[0]).max(self.c[0]),
            self.a[1].max(self.b[1]).max(self.c[1]),
            self.a[2].max(self.b[2]).max(self.c[2]),
            0.0,
        ];
        AABB::new(min, max)
    }

    pub fn centroid(&self) -> [f32; 3] {
        [
            (self.a[0] + self.b[0] + self.c[0]) / 3.0,
            (self.a[1] + self.b[1] + self.c[1]) / 3.0,
            (self.a[2] + self.b[2] + self.c[2]) / 3.0,
        ]
    }
 
}