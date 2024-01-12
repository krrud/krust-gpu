use crate::primitives::sphere::Sphere;
use wgpu::util::DeviceExt;


#[repr(C)]
#[derive(Clone, Debug)]
pub struct Scene {
    objects: Vec<Sphere>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: Vec::new(),
        }
    }

    pub fn from(objects: Vec<Sphere>) -> Self {
        Scene {
            objects,
        }
    }

    pub fn add(&mut self, object: Sphere) {
        self.objects.push(object);
    }

    pub fn objects(&self) -> &[Sphere] {
        &self.objects
    }

    pub fn to_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let data_bytes = bytemuck::cast_slice(&self.objects);
        let padding_bytes = bytemuck::bytes_of(&[0u8; 4]);
        let buffer_contents = [data_bytes, padding_bytes].concat();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Buffer"),
            contents: &buffer_contents,
            usage: wgpu::BufferUsages::STORAGE,
        })
    }
}