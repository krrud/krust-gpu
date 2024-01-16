use wgpu::util::DeviceExt;
use cgmath::{Vector3, prelude::*};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadLight {
    pub position: [f32; 4],
    pub normal: [f32; 4],
    pub u: [f32; 4],
    pub v: [f32; 4],
    pub color: [f32; 3],
    pub intensity: f32,
}

impl QuadLight {
    pub fn new(position: Vector3<f32>, aim: Vector3<f32>, size: [f32; 2], color: [f32; 3], intensity: f32) -> Self {
        let normal = (position - aim).normalize();
        let u = Vector3::new(0.0, 1.0, 0.0).cross(normal).normalize() * size[0];
        let v = normal.cross(u).normalize() * size[1];
        Self {
            position: [position.x, position.y, position.z, 0.0],
            normal: [normal.x, normal.y, normal.z, 0.0],
            u: [u.x, u.y, u.z, 0.0],
            v: [v.x, v.y, v.z, 0.0],
            color,
            intensity,
        }
    }

    pub fn to_buffer(quad_lights: &[QuadLight], device: &wgpu::Device) -> wgpu::Buffer {
        let bytes: &[u8] = bytemuck::cast_slice(quad_lights);
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Light Buffer"),
            contents: bytemuck::cast_slice(&bytes),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }
}

