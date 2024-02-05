
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AABB {
    pub min: [f32; 4],
    pub max: [f32; 4],
}

impl AABB {
    pub fn new(min: [f32; 4], max: [f32; 4]) -> AABB {
        AABB {min, max}
    }   

    pub fn default() -> AABB {
        AABB {
            min: [0.0, 0.0, 0.0, 0.0],
            max: [0.0, 0.0, 0.0, 0.0],
        }
    }

    pub fn surrounding_box(&self, other: &AABB) -> AABB {
        let min = [
            self.min[0].min(other.min[0]),
            self.min[1].min(other.min[1]),
            self.min[2].min(other.min[2]),
            0.0,
        ];
        let max = [
            self.max[0].max(other.max[0]),
            self.max[1].max(other.max[1]),
            self.max[2].max(other.max[2]),
            0.0,
        ];
        AABB {min, max}
    }
}