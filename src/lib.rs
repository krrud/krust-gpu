mod primitives;
mod wasm;
mod process;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use js_sys::Function;
use std::iter;
use wgpu::{Buffer, Device, BufferUsages, Extent3d, SamplerBindingType};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use cgmath::{InnerSpace, Vector3, prelude::*};
use wasm::state::StateJS;
use process::glb::{load_glb};
use process::bvh::{BVHNode, BVH};
use primitives::texture::Texture;
use primitives::material::Material;
use primitives::sphere::Sphere;
use primitives::triangle::Triangle;
use primitives::lights::QuadLight;
use primitives::pixel_buffer::PixelBuffer;
use primitives::scene::{Scene, RenderConfig, SceneObject};
use primitives::ray::{Ray, RayBuffer};
use primitives::hit::HitRec;
use primitives::camera::{Camera, CameraUniform, CameraController};


#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub const RENDER_SIZE: [u32; 2] = [1280, 720];

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    diffuse_texture: Texture,
    texture_bind_group: wgpu::BindGroup,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    window: Window,
    trace_pipeline: wgpu::ComputePipeline,
    trace_bind_group: wgpu::BindGroup,
    camera_ray_bind_group: wgpu::BindGroup,
    camera_ray_compute_pipeline: wgpu::ComputePipeline,
    camera_ray_uniform: RayBuffer,
    camera_ray_buffer: wgpu::Buffer,
    accumulation_array: PixelBuffer,
    accumulation_buffer: wgpu::Buffer,
    scene: Scene,
    scene_buffer: wgpu::Buffer,
    render_config: RenderConfig,
    clear_buffer: bool,
}

impl State {
    async fn new(window: Window) -> Self {

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // Log device and backend
        let info = adapter.get_info();
        println!("{:#?}", info);
        log::warn!("{:#?}", info);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, 
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);


        // SCENE SETUP
        //
        //
        //
        let camera = Camera {
            origin: (0.0, 4.0, 6.0).into(),
            focus: (0.0, 0.0, 0.0).into(),
            aperture: 0.0,
            fovy: 50.0,
            aspect: (config.width as f32 / config.height as f32).into(),
        };
        let camera_controller = CameraController::new(0.2);
        let camera_uniform = CameraUniform::from(&camera);
        let camera_buffer = camera_uniform.to_buffer(&device);


        let mut lights = Vec::new();
        let light1 = QuadLight::new(
            Vector3::new(10.0, 8.0, 2.0),  // Position
            Vector3::new(0.0, 0.0, 0.0),   // Aim
            [12.0, 12.0].into(),           // Size
            [1.0, 1.0, 1.0].into(),        // Color
            29.0,                          // Intensity
        );
        lights.push(light1);
        let light_buffer = QuadLight::to_buffer(&lights, &device);

        let mat_orange = Material::new(
            [0.4, 0.1, 0.05, 1.0],
            1.0,
            0.3,
            0.0,
            0.0,
            1.1,
        );

        let mat_gray = Material::new(
            [0.2, 0.2, 0.2, 1.0],
            0.4,
            0.6,
            0.0,
            0.0,
            1.2,
        );

        let mat_black = Material::new(
            [0.0, 0.0, 0.0, 1.0],
            0.5,
            0.1,
            0.0,
            0.0,
            1.3,
        );

        let mat_white = Material::new(
            [0.4, 0.4, 0.4, 1.0],
            1.0,
            0.5,
            0.0,
            0.0,
            1.8,
        );

        let mat_gold = Material::new(
            [0.9, 0.4, 0.1, 1.0],
            1.0,
            0.1,
            1.0,
            0.0,
            1.5,
        );

        let mat_chrome = Material::new(
            [0.4, 0.4, 0.4, 1.0],
            1.0,
            0.3,
            1.0,
            0.0,
            1.8,
        );

        let mat_emissive = Material::new(
            [50.0, 20.0, 5.0, 1.0],
            0.2,
            0.8,
            0.0,
            1.0,
            1.4,
        );

        let glb_bytes = include_bytes!("../assets/test.glb");
        let glb = load_glb(glb_bytes);
        let mut scene_triangles: Vec<Triangle> = vec![];
        for mesh in glb.meshes() {
            for chunk in mesh.indices.chunks(3) {
                let tri = Triangle::new(
                    mesh.vertices[chunk[0] as usize],
                    mesh.vertices[chunk[1] as usize],
                    mesh.vertices[chunk[2] as usize],
                    mesh.normals[chunk[0] as usize],
                    mesh.normals[chunk[1] as usize],
                    mesh.normals[chunk[2] as usize],
                    glb.materials()[mesh.material_index],
                );
                scene_triangles.push(tri);
            }
        }

        let bvh = BVH::new(&mut scene_triangles, 42069);
        let bvh_buffer = bvh.to_buffer(&device);     
 
        let triangle_bytes = bytemuck::cast_slice(&scene_triangles);
        let triangle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Triangle Buffer"),
            contents: &triangle_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });


        let sphere1 = Sphere::new([0.0, 1.0, 0.0], 1.0, mat_orange);
        let sphere2 = Sphere::new([2.0, 1.0, 0.0], 1.0, mat_chrome);
        let sphere3 = Sphere::new([-2.0, 1.0, 0.0], 1.0, mat_white);
        let sphere4 = Sphere::new([0.0, 1.0, 2.0], 1.0, mat_black);
        let sphere5 = Sphere::new([0.0, 1.0, -2.0], 1.0, mat_orange);
        let sphere6 = Sphere::new([0.0, 0.25, 0.0], 0.25, mat_gold);
        let ground = Sphere::new([0.0, -100.0, 0.0], 100.0, mat_gray);
        let mut scene_spheres = vec![sphere2, sphere3, sphere4, sphere5, sphere6, ground];
        let mut scene_objects = SceneObject::from_tri_vec(&scene_triangles);

        let new_sphere = Sphere::new([0.0, 2.5, 0.0], 0.5, mat_emissive);
        scene_spheres.push(new_sphere);
        SceneObject::add(&mut scene_objects, Some(&[new_sphere]), None);

        let sphere_bytes = bytemuck::cast_slice(&scene_spheres);
        let sphere_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sphere Buffer"),
            contents: &sphere_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        

        let render_config = RenderConfig::new(
            size.into(), // pixel dimensions
            8, // ray depth
            1, // samples
        );

        let scene = Scene::from(render_config, camera_uniform, scene_objects);
        let scene_buffer = scene.to_buffer(&device);

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });     

        

        // GLOBAL SHADER DATA
        //
        //
        //
        let shader_structs = include_str!("./shaders/structs.wgsl");
        let shader_functions = include_str!("./shaders/functions.wgsl");
        let ggx = include_str!("./shaders/ggx.wgsl");
        let accumulation_array = PixelBuffer::new([size.width, size.height]);
        let accumulation_buffer = accumulation_array.to_buffer(&device);


        // CAMERA RAY GENEREATION COMPUTE PIPELINE
        //
        //
        //
        let camera_ray_shader = include_str!("./shaders/compute_camera_rays.wgsl");
        let combined_camera_ray_shader = format!("{}\n{}\n{}", shader_structs, shader_functions, camera_ray_shader);
        let cs_camera_rays = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Camera Ray Shader"),
            source: wgpu::ShaderSource::Wgsl(combined_camera_ray_shader.into()),
        });
        let camera_ray_uniform = RayBuffer::new([size.width as u32, size.height as u32]);
        let camera_ray_buffer = camera_ray_uniform.to_buffer(&device);

        let camera_ray_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: Some("Camera Ray Bind Group Layout"),
        });
        
        let camera_ray_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_ray_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera_ray_buffer.as_entire_binding(),
                },
            ],
            label: Some("Camera Bind Group"),
        });

        let camera_ray_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Camera Ray Pipeline Layout"),
            bind_group_layouts: &[&camera_ray_bind_group_layout],
            push_constant_ranges: &[],
        });

        let camera_ray_compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Camera Ray Pipeline"),
            layout: Some(&camera_ray_pipeline_layout),
            module: &cs_camera_rays,
            entry_point: "main",
        });



        // SCENE TRAVERSAL COMPUTE PIPELINE
        //
        //
        //
        let traversal_shader = include_str!("./shaders/compute_traversal.wgsl");
        let combined_traversal_shader = format!("{}\n{}\n{}\n{}", shader_structs, shader_functions, ggx, traversal_shader);
        let trace_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Traversal Shader"),
            source: wgpu::ShaderSource::Wgsl(combined_traversal_shader.into()),
        });

        let sky_bytes = include_bytes!("../assets/sky2.png");
        
        let sky_texture =
            Texture::from_bytes(&device, &queue, sky_bytes, "sky.png").unwrap();

        let trace_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            }, 
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let trace_view = trace_texture.create_view(&Default::default());     

        let trace_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            view_dimension: wgpu::TextureViewDimension::D2,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // wgpu::BindGroupLayoutEntry {
                    //     binding: 5,
                    //     visibility: wgpu::ShaderStages::COMPUTE,
                    //     ty: wgpu::BindingType::Buffer {
                    //         ty: wgpu::BufferBindingType::Storage { read_only: true },
                    //         has_dynamic_offset: false,
                    //         min_binding_size: None,
                    //     },
                    //     count: None,
                    // },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
        });

        let trace_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &trace_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: scene_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: camera_ray_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&trace_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&sky_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&sky_texture.sampler),
                },
                // wgpu::BindGroupEntry {
                //     binding: 5,
                //     resource: sphere_buffer.as_entire_binding(),
                // },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: bvh_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: triangle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: light_buffer.as_entire_binding(),
                },

            ],
        });

        let trace_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&trace_bind_group_layout],
                push_constant_ranges: &[],
        });

        let trace_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&trace_pipeline_layout),
            module: &trace_module,
            entry_point: "main",
        });



        // RENDER PIPELINE
        //
        //
        //
        let render_shader = include_str!("./shaders/shader.wgsl");
        let combined_render_shader = format!("{}\n{}\n{}", shader_structs, shader_functions, render_shader);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(combined_render_shader.into()),
        });

        let diffuse_bytes = include_bytes!("../assets/happy-tree.png");
        
        let diffuse_texture =
            Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&trace_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: accumulation_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: scene_buffer.as_entire_binding(),
                },
            ],
            label: Some("Texture Bind Group"),
        });    

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            operation: wgpu::BlendOperation::Add,
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        },
                        alpha: wgpu::BlendComponent {
                            operation: wgpu::BlendOperation::Add,
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });


        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            diffuse_texture,
            texture_bind_group,
            camera,
            camera_controller,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            window,
            trace_pipeline,
            trace_bind_group,
            camera_ray_bind_group,
            camera_ray_compute_pipeline,
            camera_ray_uniform,
            camera_ray_buffer,
            accumulation_array,
            accumulation_buffer,
            scene,
            scene_buffer,
            render_config,
            clear_buffer: false,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.render_config.size = new_size.into();
            self.render_config.pixel_size = [1.0 / new_size.width as f32, 1.0 / new_size.height as f32];
            self.surface.configure(&self.device, &self.config);

            // Update buffers            
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;            
            self.camera_ray_uniform = RayBuffer::new([new_size.width, new_size.height]);
            self.camera_ray_uniform.update_buffer(&self.camera_ray_buffer, &self.queue);
            self.camera_uniform = CameraUniform::from(&self.camera);
            self.camera_uniform.update_buffer(&self.camera_buffer, &self.queue);
            self.scene.camera = self.camera_uniform;
            self.scene.config = self.render_config;
            self.scene.update_buffer(&self.scene_buffer, &mut self.clear_buffer, &self.queue);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn match_js(&mut self, state_js: &StateJS) {
        if  self.camera.aperture != state_js.camera.aperture{
            self.camera.aperture = state_js.camera.aperture;
            self.clear_buffer = true;
        };
        if self.camera.fovy != state_js.camera.fov {

            self.camera.fovy = state_js.camera.fov;
            self.clear_buffer = true;
        };
        if state_js.config.size[0] != self.render_config.size[0] || state_js.config.size[1] != self.render_config.size[1] {
            // TODO: Need to update trace texture resolution and trace pipeline to handle size change
            // self.resize(state_js.config.size.into());
            // self.clear_buffer = true;
        };
        if state_js.config.sky_intensity != self.render_config.sky_intensity {
            self.render_config.sky_intensity = state_js.config.sky_intensity;
            self.scene.config.sky_intensity = state_js.config.sky_intensity;
            self.clear_buffer = true;
        };
    }

    fn update(&mut self, &state_js: &StateJS) {

        self.match_js(&state_js);
        self.camera_controller.update_camera(&mut self.camera, &mut self.clear_buffer);
        self.accumulation_array.update_buffer(&mut self.accumulation_buffer, &self.clear_buffer, &self.queue);
        self.camera_uniform = CameraUniform::from(&self.camera);    
        self.camera_uniform.update_buffer(&self.camera_buffer, &self.queue);
        self.scene.camera = self.camera_uniform;
        self.scene.update_buffer(&self.scene_buffer, &mut self.clear_buffer, &self.queue);
        self.clear_buffer = false;
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            // Generate camera rays
            let mut camera_ray_compute = encoder.begin_compute_pass(&Default::default());
            camera_ray_compute.set_pipeline(&self.camera_ray_compute_pipeline);
            camera_ray_compute.set_bind_group(0, &self.camera_ray_bind_group, &[]);
            camera_ray_compute.dispatch_workgroups(self.size.width / 16, self.size.height / 16, 1);
        }

        encoder.insert_debug_marker("Ensure camera rays are generated");
        
        {
            // Trace indirect rays
            let mut trace_compute_pass = encoder.begin_compute_pass(&Default::default());
            trace_compute_pass.set_pipeline(&self.trace_pipeline);
            trace_compute_pass.set_bind_group(0, &self.trace_bind_group, &[]);
            trace_compute_pass.dispatch_workgroups(self.size.width / 16, self.size.height / 16, 1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            render_pass.draw(0..6, 0..1); 
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}


#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn confirm() {
    println!("Render started!");
    log::warn!("Render started!");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run(get_js: Function) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Krust GPU")
        .with_inner_size(winit::dpi::LogicalSize::new(RENDER_SIZE[0], RENDER_SIZE[1]))
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("krust-gpu")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => {
                            // *control_flow = ControlFlow::Exit
                        },
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {

                let state_js: StateJS = get_js.call0(&JsValue::null()).unwrap().into_serde().unwrap();
                // let state_js = StateJS::new();
                state.update(&state_js);

                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                    //     *control_flow = ControlFlow::Exit
                    }
                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
