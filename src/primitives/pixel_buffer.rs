use wgpu::util::DeviceExt;

pub struct PixelBuffer {
    data: Vec<f32>,
}

impl PixelBuffer {
    pub fn new(size: [u32; 2]) -> Self {
        Self {
            data: vec![0.0; (4 * size[0] * size[1]) as usize],
        }
    }

    pub fn to_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pixel Buffer"),
            contents: bytemuck::cast_slice(&self.data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn update_buffer(&mut self, buffer: &wgpu::Buffer, clear_buffer: &bool, queue: &wgpu::Queue) {
        if *clear_buffer {
            self.data = vec![0.0; self.data.len()];
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&self.data));
        }
    }
}