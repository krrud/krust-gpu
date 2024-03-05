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
pub struct SSSData {
    pub scatter_coeff: [f32; 3],
    pub absorption_coeff: [f32; 3],
    pub scale: f32,
    pub anisotropy: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct StateJS {
    pub config: Config,
    pub camera: Camera,
    pub sss: SSSData,
}

impl StateJS {
    pub fn new() -> Self {
        Self {
            config: Config {
                size: [1280, 720],
                sky_intensity: 1.0,
            },
            camera: Camera {
                aperture: 0.0,
                fov: 50.0,
            },
            sss: SSSData {
                scatter_coeff: [2.1, 1.9, 0.2],
                absorption_coeff: [0.04, 0.07, 0.2],
                scale: 1.0,
                anisotropy: 0.5,
            },
        }
    }
}