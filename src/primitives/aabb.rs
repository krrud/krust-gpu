use crate::primitives::triangle::{Triangle, TriangleCPU};


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

    pub fn empty() -> Self {
        AABB {
            min: [f32::INFINITY; 4],
            max: [f32::NEG_INFINITY; 4],
        }
    }

    pub fn union(&self, other: &AABB) -> AABB {
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

    pub fn surface_area(&self) -> f32 {
        let x = self.max[0] - self.min[0];
        let y = self.max[1] - self.min[1];
        let z = self.max[2] - self.min[2];
        2.0 * (x * y + y * z + z * x)
    }

    pub fn intersection(&self, other: &AABB) -> Option<AABB> {
        let min = [
            self.min[0].max(other.min[0]),
            self.min[1].max(other.min[1]),
            self.min[2].max(other.min[2]),
            0.0,
        ];
    
        let max = [
            self.max[0].min(other.max[0]),
            self.max[1].min(other.max[1]),
            self.max[2].min(other.max[2]),
            0.0,
        ];
    
        if min[0] <= max[0] && min[1] <= max[1] && min[2] <= max[2] {
            Some(AABB {min, max})
        } else {
            None
        }
    }

    pub fn bounding_box_for_slice(primitives: &[TriangleCPU], start: usize, end: usize) -> Self {
        if start >= end {
            return AABB::empty();
        }
    
        let mut bbox = primitives[start].bounding_box(0.0, 0.0);
    
        for i in start + 1..end {
            bbox = bbox.union(&primitives[i].bounding_box(0.0, 0.0));
        }
    
        bbox
    }

    pub fn extend(&mut self, other: &AABB) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(other.min[i]);
            self.max[i] = self.max[i].max(other.max[i]);
        }
    }

    pub fn contract(&mut self, other: &AABB) {
        for i in 0..3 {
            self.min[i] = self.min[i].max(other.min[i]);
            self.max[i] = self.max[i].min(other.max[i]);
        }
    }
    
}