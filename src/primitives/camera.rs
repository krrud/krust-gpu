use cgmath::InnerSpace;


pub struct Camera {
    pub origin: cgmath::Point3<f32>,
    pub focus: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub fovy: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.origin, self.focus, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    origin: [f32; 4],
    focus: [f32; 4],
    up: [f32; 4],
    fovy: f32,
    aspect: f32,
    znear: f32,
    zfar: f32,
    _padding: [f32; 2],
}

impl CameraUniform {
    pub fn from(camera: &Camera) -> Self {
        Self {
            origin: [camera.origin.x, camera.origin.y, camera.origin.z, 0.0],
            focus: [camera.focus.x, camera.focus.y, camera.focus.z, 0.0],
            up: [camera.up.x, camera.up.y, camera.up.z, 0.0],
            fovy: camera.fovy,
            aspect: camera.aspect,
            znear: camera.znear,
            zfar: camera.zfar,
            _padding: [0.0; 2],
        }
    }  
}