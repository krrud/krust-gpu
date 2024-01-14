mod texture;
mod primitives;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use std::iter;
use wgpu::{Buffer, Device, BufferUsages, Extent3d, SamplerBindingType};
use wgpu::util::DeviceExt;
use winit::{
    event::*,
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use cgmath::InnerSpace;
use primitives::material::Material;
use primitives::sphere::Sphere;
use primitives::scene::{Scene, RenderConfig};
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
struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    #[allow(dead_code)]
    diffuse_texture: texture::Texture,
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
    scene: Scene,
    scene_buffer: wgpu::Buffer,
    render_config: RenderConfig,
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
            origin: (0.0, 2.0, 3.0).into(),
            focus: (0.0, 0.0, 0.0).into(),
            aperture: 0.2,
            fovy: 80.0,
            aspect: (config.width as f32 / config.height as f32).into(),
        };
        let camera_controller = CameraController::new(0.2);
        let camera_uniform = CameraUniform::from(&camera);
        let camera_buffer = camera_uniform.to_buffer(&device);


        let mat_orange = Material::new(
            [0.4, 0.1, 0.05, 1.0],
            1.0,
            0.2,
            0.0,
            0.0,
            1.5,
        );

        let mat_gray = Material::new(
            [0.1, 0.1, 0.1, 1.0],
            0.0,
            0.3,
            0.0,
            0.0,
            1.5,
        );

        let mat_white = Material::new(
            [12.6, 12.6, 12.6, 1.0],
            1.0,
            0.5,
            0.0,
            0.0,
            1.3,
        );

        let mat_gold = Material::new(
            [0.8, 0.6, 0.2, 1.0],
            1.0,
            0.2,
            1.0,
            0.0,
            1.5,
        );

        let mat_chrome = Material::new(
            [0.6, 0.6, 0.6, 1.0],
            1.0,
            0.0,
            1.0,
            0.0,
            1.8,
        );

        let sphere1 = Sphere::new([0.0, 1.0, 0.0], 1.0, mat_orange);
        let sphere2 = Sphere::new([2.0, 1.0, 0.0], 1.0, mat_chrome);
        let sphere3 = Sphere::new([-2.0, 1.0, 0.0], 1.0, mat_white);
        let sphere4 = Sphere::new([0.0, 1.0, 2.0], 1.0, mat_gray);
        let sphere5 = Sphere::new([0.0, 1.0, -2.0], 1.0, mat_gold);
        let ground = Sphere::new([0.0, -100.0, 0.0], 100.0, mat_gray);

        let render_config = RenderConfig::new(
            size.into(), // pixel size
            16, // ray depth
            16, // samples
        );

        let scene = Scene::from(render_config, camera_uniform, vec![ground, sphere1, sphere2, sphere3, sphere4, sphere5]);
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
        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Traversal Shader"),
            source: wgpu::ShaderSource::Wgsl(combined_traversal_shader.into()),
        });

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
            module: &cs_module,
            entry_point: "main",
        });



        // Render Pipeline
        //
        //
        //
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/shader.wgsl").into()),
        });

        let diffuse_bytes = include_bytes!("happy-tree.png");
        
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

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
                    }
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
                }
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
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
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
            scene,
            scene_buffer,
            render_config,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn update_camera_aperture(&mut self, new_aperture: f32) {
        self.camera.aperture = new_aperture;
        log::warn!("Aperture Updated to: {}.", new_aperture);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            // Update buffers
            self.camera.aspect = new_size.width as f32 / new_size.height as f32;            
            self.camera_ray_uniform = RayBuffer::new([new_size.width, new_size.height]);
            self.camera_ray_uniform.update_buffer(&self.camera_ray_buffer, &self.queue);
            self.camera_uniform = CameraUniform::from(&self.camera);
            self.camera_uniform.update_buffer(&self.camera_buffer, &self.queue); 
            self.scene.update_buffer(&self.scene_buffer, &self.queue);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform = CameraUniform::from(&self.camera);    
        self.scene.update_buffer(&self.scene_buffer, &self.queue);
        self.camera_uniform.update_buffer(&self.camera_buffer, &self.queue);
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


#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
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
                let dst = doc.get_element_by_id("wasm-example")?;
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
                            *control_flow = ControlFlow::Exit
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
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size)
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = ControlFlow::Exit
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

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn confirm() {
    println!("Render started!");
    log::warn!("Render started!");
}