use gltf::Gltf;
use crate::primitives::tri_mesh::TriMesh;
use crate::primitives::material::Material;

pub struct GLBScene {
    meshes: Vec<TriMesh>,
    materials: Vec<Material>,
}

impl GLBScene {
    pub fn new() -> Self {
        GLBScene {
            meshes: Vec::new(),
            materials: Vec::new(),
        }
    }

    pub fn meshes(&self) -> &Vec<TriMesh> {
        &self.meshes
    }

    pub fn materials(&self) -> &Vec<Material> {
        &self.materials
    }
}

pub fn load_glb(data: &[u8]) -> GLBScene {
    let mut scene = GLBScene::new();
    let (glb, buffers, _images) = gltf::import_slice(data).unwrap();

    // camera
    for node in glb.nodes() {
        if let Some(camera) = node.camera() {
            let location = node.transform().decomposed();
            match camera.projection() {
                gltf::camera::Projection::Perspective(perspective) => {
                    let fov = perspective.yfov();
                    let aspect_ratio = perspective.aspect_ratio().unwrap_or(1.777);
                    let near = perspective.znear();
                    let far = perspective.zfar().unwrap_or(1000.0);
                },
                _ => {}
            }
        }
    }

    // meshes
    for mesh in glb.meshes() {
        let mut tri_mesh = TriMesh::new();
        for primitive in mesh.primitives() {
            tri_mesh.material_index = primitive.material().index().unwrap() as usize;
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));    

            if let Some(iter) = reader.read_positions() {
                for vertex_pos in iter {
                    tri_mesh.vertices.push(vertex_pos);
                }
            }
                    
            if let Some(iter) = reader.read_normals() {
                for vertex_norm in iter {
                    tri_mesh.normals.push(vertex_norm);
                }
            }
            
            if let Some(indices) = reader.read_indices() {
                match indices {
                    gltf::mesh::util::ReadIndices::U8(iter) => {
                        for vertex_idx in iter {
                            tri_mesh.indices.push(vertex_idx as u32);
                        }
                    }
                    gltf::mesh::util::ReadIndices::U16(iter) => {
                        for vertex_idx in iter {
                            tri_mesh.indices.push(vertex_idx as u32);
                        }
                    }
                    gltf::mesh::util::ReadIndices::U32(iter) => {
                        for vertex_idx in iter {
                            tri_mesh.indices.push(vertex_idx as u32);
                        }
                    }
                }
            }          
        } 
        scene.meshes.push(tri_mesh);    
    }

    // materials
    for material in glb.materials() {
        let mat_idx  = material.index();
        let pbr = material.pbr_metallic_roughness();
        let base_color = pbr.base_color_factor();
        let metallic = pbr.metallic_factor();
        let roughness = pbr.roughness_factor();
        let mut ior = 1.5;
        let mut specular = 1.0;
        let mut refract = 0.0;

        if let Some(val) = material.ior() {
            ior = val;
        };
        if let Some(spec) = material.specular() {
            specular = spec.specular_factor();
        };
        if let Some(transmission) = material.transmission() {
            refract = transmission.transmission_factor();
        };

        log::warn!("ior: {:?}", ior);
        log::warn!("specular: {:?}", specular);
        log::warn!("refract: {:?}", refract);

        // TODO: handle textures
        let mut base_color_texture_idx = None;
        let mut met_rough_texture_idx = None;
        if let Some(base_color_texture) = pbr.base_color_texture() {
            base_color_texture_idx = Some(base_color_texture.texture().index());
        };

        if let Some(met_rough_texture) = pbr.metallic_roughness_texture() {
            met_rough_texture_idx = Some(met_rough_texture.texture().index());
        };

        let material = Material::new(
            base_color,
            specular,
            roughness,
            metallic,   
            refract,
            ior,
        );
        scene.materials.push(material);
    }
    scene
}
