use cgmath::{InnerSpace, Rotation3, Rotation};
use winit::event::*;
use wgpu::util::DeviceExt;

pub struct Camera {
    pub origin: cgmath::Point3<f32>,
    pub focus: cgmath::Point3<f32>,
    pub aperture: f32,
    pub fovy: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn new(origin: cgmath::Point3<f32>, focus: cgmath::Point3<f32>, aperture: f32, fovy: f32, aspect: f32) -> Self {
        Self {
            origin,
            focus,
            aperture,
            fovy,
            aspect,
        }   
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub origin: [f32; 4],
    pub focus: [f32; 4],
    pub aperture: f32,
    pub fovy: f32,
    pub aspect: f32,
    _padding: [f32; 1],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            origin: [0.0; 4],
            focus: [0.0; 4],
            aperture: 0.0,
            fovy: 0.0,
            aspect: 0.0,
            _padding: [0.0; 1],
        }
    }

    pub fn from(camera: &Camera) -> Self {
        Self {
            origin: [camera.origin.x, camera.origin.y, camera.origin.z, 0.0],
            focus: [camera.focus.x, camera.focus.y, camera.focus.z, 0.0],
            aperture: camera.aperture,
            fovy: camera.fovy,
            aspect: camera.aspect,
            _padding: [0.0; 1],
        }
    }  

    pub fn to_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        let bytes = bytemuck::bytes_of(self);
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(bytes),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn update_buffer(&self, buffer: &wgpu::Buffer, queue: &wgpu::Queue) {
        let bytes = bytemuck::bytes_of(self);
        queue.write_buffer(buffer, 0, bytes);
    }
}

pub struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_left_mouse_button_pressed: bool,
    is_middle_mouse_button_pressed: bool,
    is_right_mouse_button_pressed: bool,
    is_shift_pressed: bool,
    last_mouse_position: Option<(f64, f64)>,
    initial_mouse_position: Option<(f32, f32)>,
    is_rotating: bool,
    is_panning: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_left_mouse_button_pressed: false,
            is_middle_mouse_button_pressed: false,
            is_right_mouse_button_pressed: false,
            is_shift_pressed: false,
            last_mouse_position: None,
            initial_mouse_position: None,
            is_rotating: false,
            is_panning: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.is_shift_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                match button {
                    MouseButton::Left => {
                        if self.is_shift_pressed {
                            self.is_panning = *state == ElementState::Pressed;
                        } else {
                            self.is_rotating = *state == ElementState::Pressed;
                        }
                    }
                    MouseButton::Middle => {
                        self.is_middle_mouse_button_pressed = *state == ElementState::Pressed;
                    }
                    MouseButton::Right => {
                        self.is_right_mouse_button_pressed = *state == ElementState::Pressed;
                    }
                    _ => {}
                }
                true
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.last_mouse_position = Some((position.x, position.y));
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, clear_buffer: &mut bool) {
        use cgmath::InnerSpace;
        let forward = camera.focus - camera.origin;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        if self.is_rotating {
            if let Some((last_x, last_y)) = self.last_mouse_position {
                let last_x = last_x as f32;
                let last_y = last_y as f32;
                if let Some((initial_x, initial_y)) = self.initial_mouse_position {
                    let sensitivity = 0.25;  
                    let threshold = 0.00;      
                    let delta_x = initial_x - last_x;
                    let delta_y = initial_y - last_y;
        
                    if delta_x.abs() > threshold && delta_y.abs() > threshold {
                        let yaw = cgmath::Deg(delta_x * sensitivity);
                        let pitch = cgmath::Deg(delta_y * sensitivity);
                        let direction = (camera.focus - camera.origin).normalize();
                        let right = direction.cross(cgmath::Vector3::unit_y()).normalize();
                        let up = right.cross(direction).normalize();
        
                        let yaw_rotation = cgmath::Quaternion::from_axis_angle(up, yaw);
                        let pitch_rotation = cgmath::Quaternion::from_axis_angle(right, pitch);
                
                        let rotation = yaw_rotation * pitch_rotation;
                        let rotated_direction = rotation.rotate_vector(camera.origin - camera.focus);               
        
                        camera.origin = camera.focus + rotated_direction;
                        *clear_buffer = true;
                    }
                }
                self.initial_mouse_position = Some((last_x, last_y));
            }  
                  
        } else if self.is_panning {
            if let Some((last_x, last_y)) = self.last_mouse_position {
                let last_x = last_x as f32;
                let last_y = last_y as f32;
                if let Some((initial_x, initial_y)) = self.initial_mouse_position {
                    let sensitivity = 0.005;
                    let delta_x = initial_x - last_x;
                    let delta_y = last_y - initial_y;
        
                    let direction = (camera.focus - camera.origin).normalize();
                    let right = direction.cross(cgmath::Vector3::unit_y()).normalize();
                    let up = right.cross(direction).normalize();
        
                    let pan_direction_local = cgmath::Vector3::new(delta_x, delta_y, 0.0) * sensitivity;
                    let pan_direction_world = right * pan_direction_local.x + up * pan_direction_local.y;
        
                    camera.origin += pan_direction_world;
                    camera.focus += pan_direction_world;
                    *clear_buffer = true;
                }
                self.initial_mouse_position = Some((last_x, last_y));
            }
        } else {
            self.initial_mouse_position = None;
            self.last_mouse_position = None;
        }

        if self.is_forward_pressed && forward_mag > self.speed {
            camera.origin += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.origin -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(cgmath::Vector3::unit_y()).normalize();

        // Redo radius calc in case the up/ down is pressed.
        let forward = camera.focus - camera.origin;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            camera.origin = camera.focus - (forward - right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.origin = camera.focus - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_up_pressed || self.is_down_pressed || self.is_forward_pressed || self.is_backward_pressed || self.is_left_pressed || self.is_right_pressed {
            *clear_buffer = true;
        }
    }
}



// MATRIX PROJECTION BASED CAMERA
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
