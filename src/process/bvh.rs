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
    let box_a = a.bounding_box(0.0, 0.0);
    let box_b = b.bounding_box(0.0, 0.0);
    box_a.min[axis].partial_cmp(&box_b.min[axis]).unwrap_or(Ordering::Equal)
}

fn choose_axis(primitives: &[Triangle], start: usize, end: usize) -> usize {
    let centroid_positions: Vec<_> = (start..end).map(|i| primitives[i].centroid()).collect();
    let mut variances = [0.0; 3];
    for axis in 0..3 {
        let mean = centroid_positions.iter().map(|p| p[axis]).sum::<f32>() / (end - start) as f32;
        variances[axis] = centroid_positions.iter().map(|p| (p[axis] - mean).powi(2)).sum::<f32>();
    }
    variances.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap().0
}