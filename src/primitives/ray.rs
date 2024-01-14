use wgpu::util::DeviceExt;
use wgpu::Extent3d;
use bytemuck::{Pod, Zeroable};
use cgmath::prelude::*;


const MAX_SIZE: usize = 1920 * 1080;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
    _padding: [f32; 2],
}

impl Ray {
    pub fn default() -> Self {
        let o  = cgmath::Vector3::new(0.0, 0.0, 0.0);
        let d = (cgmath::Vector3::new(0.0, 0.0, 1.0) - o).normalize();

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
    pub size: [u32; 2],
    _padding: [u32; 2],
    pub data: Vec<Ray>,
}

impl RayBuffer {
    pub fn new(size: [u32; 2]) -> Self {
        let data = vec![Ray::default(); MAX_SIZE];
        Self { size, _padding: [0u32; 2], data }
    }

    pub fn to_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let data_bytes = bytemuck::cast_slice(&self.data);
        let padding_bytes = bytemuck::bytes_of(&self._padding);
        let size_bytes = bytemuck::bytes_of(&self.size); 
        let buffer_contents = [size_bytes, padding_bytes, data_bytes].concat();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ray Buffer"),
            contents: &buffer_contents,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn update_buffer(&self, buffer: &wgpu::Buffer, queue: &wgpu::Queue) {
        let size_bytes = bytemuck::bytes_of(&self.size);
        let padding_bytes = bytemuck::bytes_of(&self._padding);
        let data_bytes = bytemuck::cast_slice(&self.data);
        let buffer_contents = [size_bytes, padding_bytes, data_bytes].concat();
        queue.write_buffer(buffer, 0, &buffer_contents);
    }
}


// #[repr(C)]
// #[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// struct CameraUniform {
//     view_proj: [[f32; 4]; 4],
//     origin: [f32; 3],
//     _padding: [f32; 1],
// }

// impl CameraUniform {
//     fn new() -> Self {
//         use cgmath::SquareMatrix;
//         Self {
//             view_proj: cgmath::Matrix4::identity().into(),
//             origin: [0.0; 3],
//             _padding: [0.0; 1],
//         }
//     }

//     fn update_view_proj(&mut self, camera: &Camera) {
//         self.view_proj = (OPENGL_TO_WGPU_MATRIX * camera.build_view_projection_matrix()).into();
//         self.origin = camera.origin.into();
//     }
// }