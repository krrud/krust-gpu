use crate::primitives::sphere::Sphere;
use crate::primitives::triangle::Triangle;
use crate::primitives::camera::CameraUniform;
use wgpu::util::DeviceExt;
use rand::random;


#[repr(C)]
#[derive(Clone, Debug)]
pub struct Scene {
    pub config: RenderConfig,
    pub camera: CameraUniform,
    pub objects: Vec<SceneObject>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            config: RenderConfig::default(),
            camera: CameraUniform::new(),
            objects: Vec::new(),
        }
    }

    pub fn from(mut config: RenderConfig, camera: CameraUniform, objects: Vec<SceneObject>) -> Self {
        config.num_objects = objects.len() as u32;
        Scene {
            config,
            camera,
            objects,
        }
    }

    pub fn add(&mut self, object: SceneObject) {
        self.objects.push(object);
    }

    pub fn objects(&self) -> &[SceneObject] {
        &self.objects
    }

    pub fn to_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let config_bytes = bytemuck::bytes_of(&self.config);
        let camera_bytes = bytemuck::bytes_of(&self.camera);
        let objects_bytes = bytemuck::cast_slice(&self.objects);
        let buffer_contents = [config_bytes, camera_bytes, objects_bytes].concat();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Buffer"),
            contents: &buffer_contents,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn update_buffer(&mut self, buffer: &wgpu::Buffer, clear_buffer: &mut bool, queue: &wgpu::Queue) {
        if *clear_buffer {
            self.config.update(true);
        } else {
            self.config.update(false);
        }
        let config_bytes = bytemuck::bytes_of(&self.config);
        let camera_bytes = bytemuck::bytes_of(&self.camera);
        let objects_bytes = bytemuck::cast_slice(&self.objects);
        let buffer_contents = [config_bytes, camera_bytes, objects_bytes].concat();
        queue.write_buffer(buffer, 0, &buffer_contents);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderConfig {
    pub seed: [u32; 4],
    pub size: [u32; 2],
    pub pixel_size: [f32; 2],
    pub max_depth: u32,
    pub samples: u32,
    pub num_objects: u32,
    pub count: u32,
    pub sky_intensity: f32,
    pub sky_color: [f32; 4],
    _padding: [u32; 3],
}

impl RenderConfig {
    pub fn default() -> Self {
        Self {
            seed: rand::random::<[u32; 4]>(),
            size: [960, 540],
            pixel_size: [1.0 / 960.0, 1.0 / 540.0],
            max_depth: 16,
            samples: 16,
            num_objects: 0,
            count: 1,
            sky_intensity: 1.0,
            sky_color: [1.0, 1.0, 1.0, 1.0],
            _padding: [0; 3],
        }
    }

    pub fn new(size: [u32; 2], max_depth: u32, samples: u32) -> Self {
        let seed = rand::random::<[u32; 4]>();
        Self {
            seed: rand::random::<[u32; 4]>(),
            size,
            pixel_size: [1.0 / size[0] as f32, 1.0 / size[1] as f32],
            max_depth,
            samples,
            num_objects: 0,
            count: 1,
            sky_intensity: 1.0,
            sky_color: [1.0, 1.0, 1.0, 1.0],
            _padding: [0; 3],
        }
    }

    pub fn update(&mut self, clear: bool) {
        if clear {
            self.count = 1;
        } else {
            self.count += 1;
        }
        self.seed = rand::random::<[u32; 4]>();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SceneObject {
    object_type: u32,
    index: u32,
}

impl SceneObject {
    pub fn new(object_type: u32, index: u32) -> Self {
        Self {
            object_type,
            index,
        }
    }

    pub fn from_sphere(index: u32) -> Self {
        Self::new(1 as u32, index)
    }

    pub fn from_triangle(index: u32) -> Self {
        Self::new(0 as u32, index)
    }

    pub fn from_sphere_vec(spheres: &[Sphere]) -> Vec<Self> {
        let out = spheres
            .into_iter()
            .enumerate()
            .map(|(i, _)| Self::new(1 as u32, i as u32))
            .collect();
        out
    }

    pub fn add(scene_objects: &mut Vec<SceneObject>, spheres: Option<&[Sphere]>, tris: Option<&[Triangle]>) {
        if let Some(spheres) = spheres {
            let mut i = scene_objects.len();
            for sphere in spheres {
                scene_objects.push(SceneObject::from_sphere(i as u32));
                i += 1;
            }
        }
    
        if let Some(tris) = tris {
            let mut i = scene_objects.len();
            for sphere in spheres {
                scene_objects.push(SceneObject::from_triangle(i as u32));
                i += 1;
            }
        }
    }
}



