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

        let axis = choose_axis(primitives, start, end);
        let object_span = end - start;
        let mut left = -1;
        let mut right = -1;
        let mut triangle = -1;
        let mut aabb;

        if object_span == 1 {
            aabb = primitives[start].bounding_box(0.0, 0.0);
            triangle = start as i32;
        } else {
            primitives[start..end].sort_by(|a, b| box_compare(a, b, axis));
            let mid = start + object_span / 2;
            left = BVHNode::new(nodes, primitives, start, mid);
            right = BVHNode::new(nodes, primitives, mid, end);
            triangle = -1;
            aabb = nodes[left as usize].aabb.surrounding_box(&nodes[right as usize].aabb);
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

    for i in start..end {
        let centroid = primitives[i].centroid();
        for axis in 0..3 {
            centroid_sums[axis] += centroid[axis];
            centroid_sums_squared[axis] += centroid[axis].powi(2);
        }
    }

    let num_primitives = end - start;

    let mut max_variance = 0.0;
    let mut max_variance_axis = 0;

    for axis in 0..3 {
        let mean = centroid_sums[axis] / num_primitives as f32;
        let variance = centroid_sums_squared[axis] / num_primitives as f32 - mean.powi(2);
        
        if variance > max_variance {
            max_variance = variance;
            max_variance_axis = axis;
        }
    }

    max_variance_axis
}