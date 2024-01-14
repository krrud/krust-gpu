use crate::primitives::sphere::Sphere;
use crate::primitives::camera::CameraUniform;
use wgpu::util::DeviceExt;


#[repr(C)]
#[derive(Clone, Debug)]
pub struct Scene {
    pub config: RenderConfig,
    pub camera: CameraUniform,
    pub objects: Vec<Sphere>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            config: RenderConfig::default(),
            camera: CameraUniform::new(),
            objects: Vec::new(),
        }
    }

    pub fn from(mut config: RenderConfig, camera: CameraUniform, objects: Vec<Sphere>) -> Self {
        config.num_objects = objects.len() as u32;
        Scene {
            config,
            camera,
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
        let config_bytes = bytemuck::bytes_of(&self.config);
        let camera_bytes = bytemuck::bytes_of(&self.camera);
        let data_bytes = bytemuck::cast_slice(&self.objects);
        let buffer_contents = [config_bytes, camera_bytes, data_bytes].concat();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Buffer"),
            contents: &buffer_contents,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn update_buffer(&self, buffer: &wgpu::Buffer, queue: &wgpu::Queue) {
        let config_bytes = bytemuck::bytes_of(&self.config);
        let camera_bytes = bytemuck::bytes_of(&self.camera);
        let data_bytes = bytemuck::cast_slice(&self.objects);
        let buffer_contents = [config_bytes, camera_bytes, data_bytes].concat();
        queue.write_buffer(buffer, 0, &buffer_contents);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderConfig {
    pub size: [u32; 2],
    pub pixel_size: [f32; 2],
    pub max_depth: u32,
    pub samples: u32,
    pub num_objects: u32,
    _padding: [u32; 1],
}

impl RenderConfig {
    pub fn default() -> Self {
        Self {
            size: [960, 540],
            pixel_size: [1.0 / 960.0, 1.0 / 540.0],
            max_depth: 16,
            samples: 16,
            num_objects: 0,
            _padding: [0; 1],
        }
    }

    pub fn new(size: [u32; 2], max_depth: u32, samples: u32) -> Self {
        Self {
            size,
            pixel_size: [1.0 / size[0] as f32, 1.0 / size[1] as f32],
            max_depth,
            samples,
            num_objects: 0,
            _padding: [0; 1],
        }
    }
}


