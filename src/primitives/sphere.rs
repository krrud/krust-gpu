// use crate::primitives::hit::HitRec;
// use crate::primitives::ray::Ray;


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sphere {
    center: [f32; 3],
    radius: f32,
}

impl Sphere {
    pub fn new(center: [f32; 3], radius: f32) -> Self {
        Sphere { center, radius }
    }

    // Assuming Ray and HitRec are defined elsewhere and imported correctly
    // pub fn hit(&self, ray: &Ray) -> Option<HitRec> {
    //     Some(HitRec::new(
    //             Point3::new(0.0, 0.0, 0.0),
    //             Vector3::new(0.0, 0.0, 0.0),
    //             0.0,
    //             false,
    //     ))
    // }
}