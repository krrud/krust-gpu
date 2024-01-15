pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

pub struct QuadLight {
    pub color: [f32; 4],
    pub intensity: f32,
    pub transform: Transform,
    pub size: [f32; 2],
}

impl QuadLight {
    pub fn new(color: [f32; 4], intensity: f32, transform: Transform, size: [f32; 2]) -> Self {
        Self {
            color,
            intensity,
            transform,
            size,
        }
    }
}