use std::cmp::Ordering;
use wgpu::util::DeviceExt;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

use crate::primitives::aabb::AABB;
use crate::primitives::triangle::TriangleCPU;


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
    fn new(nodes: &mut Vec<BVHNode>, primitives: &mut [TriangleCPU], start: usize, end: usize) -> i32 {
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
            let split = best_divide(primitives, start, end);
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
    pub fn new(primitives: &mut [TriangleCPU], seed: u64) -> BVH {
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

    pub fn reorder_nodes(&mut self) {
        let mut ordered_nodes = Vec::with_capacity(self.nodes.len());
        let new_root = self.reorder_nodes_recursive(self.root, &mut ordered_nodes);
        
        // Update the root index
        self.root = new_root;
        
        // Replace the original nodes with the reordered nodes
        self.nodes = ordered_nodes;
    }

    fn reorder_nodes_recursive(&self, node_index: i32, ordered_nodes: &mut Vec<BVHNode>) -> i32 {
        if node_index == -1 {
            return -1;
        }

        let node = self.nodes[node_index as usize];
        let new_index = ordered_nodes.len() as i32;
        ordered_nodes.push(node);

        // Recursively reorder left and right child nodes
        let new_left = self.reorder_nodes_recursive(node.left, ordered_nodes);
        let new_right = self.reorder_nodes_recursive(node.right, ordered_nodes);

        // Update the left and right indices of the current node
        let mut new_node = ordered_nodes[new_index as usize];
        new_node.left = new_left;
        new_node.right = new_right;
        ordered_nodes[new_index as usize] = new_node;

        new_index
    }
}

fn box_compare(a: &TriangleCPU, b: &TriangleCPU, axis: usize) -> Ordering {
    a.bounding_box(0.0, 0.0).min[axis].partial_cmp(&b.bounding_box(0.0, 0.0).min[axis]).unwrap_or(Ordering::Equal)
}

fn best_divide(primitives: &mut [TriangleCPU], start: usize, end: usize) -> usize {
    
    let num_primitives = end - start;
    let centroid_sums = Mutex::new([0.0; 3]);
    let centroid_sums_squared = Mutex::new([0.0; 3]);
    let surface_area_sums = Mutex::new([0.0; 3]);

    // Compute the sums of centroid coordinates and surface areas along each axis
    primitives[start..end].par_iter().for_each(|primitive| {
        let centroid = primitive.centroid();
        let bbox_surface_area = primitive.bbox_surface_area();
        
        for axis in 0..3 {
            centroid_sums.lock().unwrap()[axis] += centroid[axis];
            centroid_sums_squared.lock().unwrap()[axis] += centroid[axis].powi(2);
            surface_area_sums.lock().unwrap()[axis] += bbox_surface_area;
        }
    });

    // Compute variance
    let mut max_combined_variance = 0.0;
    let mut best_axis = 0;

    for axis in 0..3 {
        let centroid_mean = centroid_sums.lock().unwrap()[axis] / num_primitives as f32;
        let centroid_variance = centroid_sums_squared.lock().unwrap()[axis] / num_primitives as f32 - centroid_mean.powi(2);
        let surface_area_mean = surface_area_sums.lock().unwrap()[axis] / num_primitives as f32;
        let mut surface_area_variance_sum = 0.0;
        
        // Sum of differences^2 from the mean for surface areas
        for i in start..end {
            let bbox = primitives[i].bounding_box(0.0, 0.0);
            let bbox_surface_area = primitives[i].bbox_surface_area();
            let difference = bbox_surface_area - surface_area_mean;
            surface_area_variance_sum += difference * difference;
        }
        let surface_area_variance = surface_area_variance_sum / num_primitives as f32;

        // Combine variances
        let combined_variance = centroid_variance + surface_area_variance;
        
        if combined_variance > max_combined_variance {
            max_combined_variance = combined_variance;
            best_axis = axis;
        }
    }

    primitives[start..end].par_sort_by(|a, b| box_compare(a, b, best_axis));
    let best_index = sah_split(primitives, start, end);

    best_index
}


fn sah_middle_out(
    primitives: &mut [TriangleCPU],
    start: usize,
    end: usize,
) -> usize {

    let range = end - start;
    let mut split = range / 2;
    let mut best_cost = f32::INFINITY;
    let mut best_index = split;

    // Go forward from middle
    let mut i = split;
    while i > 0 {
        let left = AABB::bounding_box_for_slice(&primitives[start..start + i], 0, i).surface_area();
        let right = AABB::bounding_box_for_slice(&primitives[start + i..end], 0, range - i).surface_area();
        let cost = left * i as f32 + right * (range - i) as f32;

        if cost < best_cost {
            best_cost = cost;
            best_index = i;
        } else {
            break;
        }

        i -= 1;
    }

    // Go backward from middle
    let mut i = split + 1;
    while i < range {
        let left = AABB::bounding_box_for_slice(&primitives[start..start + i], 0, i).surface_area();
        let right = AABB::bounding_box_for_slice(&primitives[start + i..end], 0, range - i).surface_area();
        let cost = left * i as f32 + right * (range - i) as f32;

        if cost < best_cost {
            best_cost = cost;
            best_index = i;
        } else {
            break;
        }

        i += 1;
    }

    start + best_index
}


fn sah_split(
    primitives: &mut [TriangleCPU],
    start: usize,
    end: usize,
) -> usize {

    let range = end - start;
    let bucket_count = 12;
    let bucket_size = range / bucket_count;
    let mut best_cost = f32::INFINITY;
    let mut best_index = 0;

    let indices: Vec<_> = (0..bucket_count)
        .into_par_iter()
        .map(|i| {
            let bucket_start = start + i * bucket_size;
            let bucket_end = if i < bucket_count - 1 {
                start + (i + 1) * bucket_size
            } else {
                end
            };

            if bucket_start < bucket_end {
                let left = AABB::bounding_box_for_slice(&primitives[start..bucket_end], 0, bucket_end - start).surface_area();
                let right = AABB::bounding_box_for_slice(&primitives[bucket_end..end], 0, end - bucket_end).surface_area();
                let cost = left * (bucket_end - start) as f32 + right * (end - bucket_end) as f32;

                (cost, (bucket_start + bucket_end) / 2)
            } else {
                (f32::INFINITY, 0)
            }
        })
        .collect();

    for (cost, index) in indices {
        if cost < best_cost {
            best_cost = cost;
            best_index = index;
        }
    }

    if (best_index <= start || best_index >= end) {
        best_index = (end - start) / 2 + start;
    }

    best_index
}


