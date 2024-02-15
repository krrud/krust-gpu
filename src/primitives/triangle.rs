use crate::primitives::material::Material;
use crate::primitives::aabb::AABB;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    indices: [u32; 3],
    material: u32,
}


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TriangleCPU {
    a_index: u32,
    b_index: u32,
    c_index: u32,
    a: [f32; 4],
    b: [f32; 4],
    c: [f32; 4],
    material: u32,
    centroid: [f32; 3],
    bbox: AABB,
    bbox_surface_area: f32,
    surface_area: f32,
}

impl TriangleCPU {
    pub fn new(
        a_index: u32,
        b_index: u32,
        c_index: u32,
        a: [f32; 3], 
        b: [f32; 3], 
        c: [f32; 3], 
        material: u32,
    ) -> Self {
            let centroid =         [
                (a[0] + b[0] + c[0]) / 3.0,
                (a[1] + b[1] + c[1]) / 3.0,
                (a[2] + b[2] + c[2]) / 3.0,
            ];
            let surface_area = 0.5 * ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs();
            let bbox = AABB::new(
                [a[0].min(b[0]).min(c[0]), a[1].min(b[1]).min(c[1]), a[2].min(b[2]).min(c[2]), 0.0],
                [a[0].max(b[0]).max(c[0]), a[1].max(b[1]).max(c[1]), a[2].max(b[2]).max(c[2]), 0.0],
            );
            let bbox_surface_area = bbox.surface_area();
        Self {
            a_index,
            b_index,
            c_index,
            a: [a[0], a[1], a[2], 0.0],
            b: [b[0], b[1], b[2], 0.0],
            c: [c[0], c[1], c[2], 0.0],
            material,
            centroid,
            bbox,
            bbox_surface_area,
            surface_area,            
        }
    }

    pub fn bounding_box(&self, _t0: f32, _t1: f32) -> AABB {
        self.bbox
    }

    pub fn centroid(&self) -> [f32; 3] {
        self.centroid
    }

    pub fn surface_area(&self) -> f32 {
        self.surface_area
    }

    pub fn bbox_surface_area(&self) -> f32 {
        self.bbox_surface_area
    }
    
    pub fn to_buffer_vec(triangles: &Vec<TriangleCPU>) -> Vec<Triangle> {
        let mut buffer_vec = Vec::with_capacity(triangles.len());
        for triangle in triangles {
            buffer_vec.push(Triangle {
                indices: [triangle.a_index, triangle.b_index, triangle.c_index],
                material: triangle.material,
            });
        }
        buffer_vec
    }
}