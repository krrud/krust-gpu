use std::cmp::Ordering;
use wgpu::util::DeviceExt;

use crate::primitives::aabb::AABB;
use crate::primitives::triangle::Triangle;


#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BVHNode {
    aabb: AABB,
    left: i32,
    right: i32,
    triangle: i32,
    _padding: i32, 
}

impl BVHNode {
    fn new(nodes: &mut Vec<BVHNode>, primitives: &mut [Triangle], start: usize, end: usize) -> i32 {
        if start >= end {
            panic!("BVHNode::new called with invalid range");
        }

        let object_span = end - start;
        let mut left = -1;
        let mut right = -1;
        let mut triangle = -1;
        let mut aabb;

        if object_span == 1 {
            aabb = primitives[start].bounding_box(0.0, 0.0);
            triangle = start as i32;
        } else {            
            // let (axis, mut split) = choose_axis_sah(primitives, start, end);
            // primitives[start..end].sort_by(|a, b| box_compare(a, b, axis));
            // if split <= start || split >= end {
            //     split = start + object_span / 2;
            // }
            let axis = choose_axis(primitives, start, end);
            primitives[start..end].sort_by(|a, b| box_compare(a, b, axis));
            let split = start + object_span / 2;
            left = BVHNode::new(nodes, primitives, start, split);
            right = BVHNode::new(nodes, primitives, split, end);
            triangle = -1;
            aabb = nodes[left as usize].aabb.union(&nodes[right as usize].aabb);
        }

        nodes.push(BVHNode {
            aabb,
            left,
            right,
            triangle,
            _padding: 0 as i32,
        });

        (nodes.len() - 1) as i32
    }
}


#[repr(C)]
#[derive(Debug, Clone)]
pub struct BVH {
    root: i32,
    _padding: [i32; 3],
    nodes: Vec<BVHNode>,
}

impl BVH {
    pub fn new(primitives: &mut [Triangle], seed: u64) -> BVH {
        let mut nodes = Vec::new();
        let root = BVHNode::new(&mut nodes, primitives, 0, primitives.len());
        BVH { root, _padding: [0; 3], nodes }
    }

    pub fn to_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let root_bytes = bytemuck::bytes_of(&self.root);
        let padding_bytes = bytemuck::bytes_of(&self._padding);
        let node_bytes = bytemuck::cast_slice(&self.nodes);
        let bytes: Vec<u8> = [&root_bytes, &padding_bytes, node_bytes].concat();

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("BVH Buffer"),
            contents: &bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn nodes(&self) -> &Vec<BVHNode> {
        &self.nodes
    }

    pub fn root(&self) -> i32 {
        self.root
    }
}

fn box_compare(a: &Triangle, b: &Triangle, axis: usize) -> Ordering {
    a.bounding_box(0.0, 0.0).min[axis].partial_cmp(&b.bounding_box(0.0, 0.0).min[axis]).unwrap_or(Ordering::Equal)
}

fn choose_axis(primitives: &[Triangle], start: usize, end: usize) -> usize {
    let mut centroid_sums = [0.0; 3];
    let mut centroid_sums_squared = [0.0; 3];
    let mut surface_area_sums = [0.0; 3];

    // Compute the sums of centroid coordinates and surface areas along each axis
    for i in start..end {
        let centroid = primitives[i].centroid();
        let bbox = primitives[i].bounding_box(0.0, 0.0); 
        let bbox_surface_area = bbox.surface_area(); 

        for axis in 0..3 {
            centroid_sums[axis] += centroid[axis];
            centroid_sums_squared[axis] += centroid[axis].powi(2);
            surface_area_sums[axis] += bbox_surface_area;
        }
    }

    let num_primitives = end - start;

    // Compute the variance of centroid coordinates and surface areas along each axis
    let mut max_combined_variance = 0.0;
    let mut max_combined_variance_axis = 0;

    for axis in 0..3 {
        let centroid_mean = centroid_sums[axis] / num_primitives as f32;
        let centroid_variance = centroid_sums_squared[axis] / num_primitives as f32 - centroid_mean.powi(2);
        
        let surface_area_mean = surface_area_sums[axis] / num_primitives as f32;
        let mut surface_area_variance_sum = 0.0;
        
        // Compute the sum of squared differences from the mean for surface areas
        for i in start..end {
            let bbox = primitives[i].bounding_box(0.0, 0.0);
            let bbox_surface_area = bbox.surface_area();
            let difference = bbox_surface_area - surface_area_mean;
            surface_area_variance_sum += difference * difference;
        }
        let surface_area_variance = surface_area_variance_sum / num_primitives as f32;

        // Combine centroid and surface area variances into a single metric
        let combined_variance = centroid_variance + surface_area_variance;
        
        if combined_variance > max_combined_variance {
            max_combined_variance = combined_variance;
            max_combined_variance_axis = axis;
        }
    }

    max_combined_variance_axis
}

fn choose_axis_sah(primitives: &[Triangle], start: usize, end: usize) -> (usize, usize) {
    let mut best_axis = 0;
    let mut best_cost = f32::INFINITY;
    let mut best_index = 0;

    for axis in 0..3 {
        let mut local_primitives = primitives[start..end].to_vec();
        local_primitives.sort_by(|a, b| box_compare(a, b, axis));

        let mut left_box = AABB::empty();
        let mut right_box = local_primitives.iter().fold(AABB::empty(), |acc, prim| acc.union(&prim.bounding_box(0.0, 0.0)));

        for i in 1..local_primitives.len() {
            left_box = left_box.union(&local_primitives[i - 1].bounding_box(0.0, 0.0));
            if let Some(intersection) = right_box.intersection(&local_primitives[i - 1].bounding_box(0.0, 0.0)) {
                right_box = intersection;

                let left_cost = left_box.surface_area() * i as f32;
                let right_cost = right_box.surface_area() * (local_primitives.len() - i) as f32;
                let cost = left_cost + right_cost;

                if cost < best_cost {
                    best_cost = cost;
                    best_axis = axis;
                    best_index = i - 1;
                }
            }
        }
    }

    (best_axis, best_index)
}
