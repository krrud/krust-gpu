
use wgpu::util::DeviceExt;
use wgpu::Extent3d;
use bytemuck::{Pod, Zeroable};
use cgmath::prelude::*;

const MAX_SIZE: usize = 1280 * 720;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    _padding: [f32; 2],
}

impl Ray {
    pub fn default() -> Self {
        let o  = cgmath::Vector3::new(2.0, 2.0, 2.0);
        let d = (cgmath::Vector3::new(0.0, 0.0, 0.0) - o).normalize();

        Self {
            origin: [o.x, o.y, o.z],
            direction: [d.x, d.y, d.z],
            _padding: [0f32; 2] 
        }
    } 

    pub fn new(origin: [f32; 3], direction: [f32; 3]) -> Self {
        Self {
            origin,
            direction,
            _padding: [0f32; 2] 
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct RayBuffer {
    pub data: Vec<Ray>,
    pub size: [u32; 2],
}

impl RayBuffer {
    pub fn new(size: [u32; 2]) -> Self {
        let data = vec![Ray::default(); MAX_SIZE];
        Self { data, size }
    }

    pub fn to_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let data_bytes = bytemuck::cast_slice(&self.data);
        let size_bytes = bytemuck::bytes_of(&self.size);  
        let padding_bytes = bytemuck::bytes_of(&[0u8; 6]); 
        let buffer_contents = [data_bytes, size_bytes, padding_bytes].concat();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ray Buffer"),
            contents: &buffer_contents,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn update_buffer(&self, buffer: &wgpu::Buffer, queue: &wgpu::Queue) {
        let data_bytes = bytemuck::cast_slice(&self.data);
        let size_bytes = bytemuck::bytes_of(&self.size);  
        let padding_size = (4 - (self.size.len() * std::mem::size_of::<u32>() % 4)) % 4;
        let padding_bytes = vec![0u8; padding_size];
        let buffer_contents = [data_bytes, size_bytes, &padding_bytes[..]].concat();
        queue.write_buffer(buffer, 0, &buffer_contents);
    }
}
