use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Config {
    pub size: [u32; 2],
    pub sky_intensity: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Camera {
    pub aperture: f32,
    pub fov: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct StateJS {
    pub config: Config,
    pub camera: Camera,
}

impl StateJS {
    fn new() -> Self {
        Self {
            config: Config {
                size: [1280, 720],
                sky_intensity: 1.0,
            },
            camera: Camera {
                aperture: 0.0,
                fov: 50.0,
            },
        }
    }
}