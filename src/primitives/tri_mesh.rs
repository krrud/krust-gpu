
pub struct TriMesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub material_index: usize,
}

impl TriMesh {
    pub fn new() -> Self {
        TriMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            material_index: 0,
        }
    }
}