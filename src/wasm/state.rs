use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Copy, Clone, Debug)]

pub struct StateJS {
    pub aperture: f32,
}

impl StateJS {
    fn new() -> Self {
        Self { aperture: 0.0 }
    }

    fn update(&mut self, aperture: f32) {
        self.aperture = aperture;
    }
}