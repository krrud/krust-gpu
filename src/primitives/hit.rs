use cgmath::{Point3, Vector3};


#[repr(C)]
pub struct HitRec {
    point: Point3<f32>,
    normal: Vector3<f32>,
    t: f32,
    front_face: bool,
}

impl HitRec {
    pub fn new(point: Point3<f32>, normal: Vector3<f32>, t: f32, front_face: bool) -> Self {
        HitRec { point, normal, t, front_face }
    }
}