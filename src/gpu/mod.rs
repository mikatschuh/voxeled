pub mod camera;
pub mod camera_controller;
pub mod instance;
// pub mod exotic_cameras;
pub mod mesh;
mod shader;
mod texture;
pub mod window;

use camera::Camera3d;
use camera_controller::CameraController;
use glam::Vec3;
use instance::Instance;
use mesh::*;
use texture::Texture;
use wgpu::util::DeviceExt;
use winit::event_loop::EventLoopWindowTarget;

/// Ein Drawer. Der Drawer ist der Zugang zur Graphikkarte. Er ist an ein Fenster genüpft.
pub struct Drawer<'a, CC: CameraController, C>
where
    C: Camera3d<CC>,
{
    _phantom: std::marker::PhantomData<CC>,

    // bind groups:
    diffuse_bind_group: wgpu::BindGroup,
    camera_bind_group: wgpu::BindGroup,

    // rendering stuff:
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    depth_texture: Texture,
    depth_texture_bind_group: wgpu::BindGroup,
    depth_texture_bind_group_layout: wgpu::BindGroupLayout,

    render_target_texture: Texture,
    render_target_bind_group: wgpu::BindGroup,
    render_target_bind_group_layout: wgpu::BindGroupLayout,

    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    pub window: window::Window<'a>,

    render_pipeline: wgpu::RenderPipeline,
    post_processing_pipeline: wgpu::RenderPipeline,

    pub camera: &'a mut C,
    camera_buffer: wgpu::Buffer,

    // Asset things:
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,

    mesh: Mesh,
    instance_buffer: wgpu::Buffer,
}

impl<'a, CC: CameraController, C: Camera3d<CC>> Drawer<'a, CC, C> {
    /// Diese Funktion erstellt einen Drawer der mit dem aktuellen Fenster verbunden ist.
    /// Außerdem nimmt sie einen PresentMode entgegen mit dem auf das Fenster gezeichnet werden soll.
    pub async fn connect_to(
        window: &'a winit::window::Window,
        present_mode: wgpu::PresentMode,
        camera: &'a mut C,
    ) -> Drawer<'a, CC, C> {
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let future = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web, we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
                memory_hints: Default::default(),
            },
            None, // Trace path
        );
        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let size = window.inner_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]),
            width: size.width,
            height: size.height,
            present_mode, // surface_caps.present_modes[0] will select it at runtime
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        let (device, queue) = future.await.unwrap();

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
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(
                &camera.view_proj(size.width as f32 / size.height as f32),
            ),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
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
        let render_target_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    // Binding 0: Die 2D-Textur
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT, // Im Fragment-Shader nutzbar
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2, // 2D-Textur
                            multisampled: false,
                        },
                        count: None,
                    },
                    // Binding 1: Der Sampler
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), // Normaler Sampler
                        count: None,
                    },
                ],
            });
        let depth_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Depth Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let depth_texture = Texture::create_depth_texture(&device, &config);
        let depth_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Depth Bind Group"),
            layout: &depth_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&depth_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&depth_texture.sampler),
                },
            ],
        });
        let shader = device.create_shader_module(crate::gpu::shader::make_shader());

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&Vertex::vertices()),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&Vertex::indices()),
            usage: wgpu::BufferUsages::INDEX,
        });
        let instances = Mesh::new();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances.instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let render_target = Texture::create_rendering_target(&device, &config);

        Self {
            _phantom: std::marker::PhantomData,
            diffuse_bind_group: {
                let texture = Texture::from_image(
                    &device,
                    &queue,
                    &image::load_from_memory(include_bytes!("stone.png")).unwrap(),
                    Some("Stone Texture"),
                );
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                    ],
                    label: Some("diffuse_bind_group"),
                })
            },
            camera_bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &camera_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }],
                label: Some("camera_bind_group"),
            }),
            render_target_bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &render_target_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&render_target.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&render_target.sampler),
                    },
                ],
                label: Some("render target bind group"),
            }),
            render_target_texture: render_target,
            window: window::Window::from(window, true),
            surface,
            queue,
            depth_texture,
            depth_texture_bind_group,
            render_pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(
                    &&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[
                            &texture_bind_group_layout,
                            &camera_bind_group_layout,
                        ],
                        push_constant_ranges: &[],
                    }),
                ),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc(), Instance::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Front),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true, // Wichtig für Transparenz
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            }),
            post_processing_pipeline: device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("Post Processing Pipeline"),
                    layout: Some(
                        &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("Pipeline Layout"),
                            bind_group_layouts: &[
                                &render_target_bind_group_layout,
                                &depth_texture_bind_group_layout,
                            ], // Bind Group für die Textur
                            push_constant_ranges: &[],
                        }),
                    ),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "full_screen_quat",
                        buffers: &[], // Keine Vertex-Daten, weil wir ein Fullscreen-Quad generieren
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "apply_effects",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: config.format, // Das Renderziel (z. B. Swapchain-Format)
                            blend: Some(wgpu::BlendState::REPLACE), // Einfaches Overwriting
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList, // Wir rendern ein Quad aus 2 Dreiecken
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: None, // Keine Depth-Tests nötig, weil wir nur das Bild verarbeiten
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                },
            ),
            depth_texture_bind_group_layout,
            render_target_bind_group_layout,
            device,
            config,
            camera,
            camera_buffer,
            vertex_buffer,
            index_buffer,
            mesh: instances,
            instance_buffer,
            num_indices: 6,
        }
    }
    /// Eine Methode welche die Fenstergröße anpasst.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window.resize(new_size);
            self.config.width = new_size.width;
            self.config.height = new_size.height;

            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config);
            self.depth_texture_bind_group =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Depth Bind Group"),
                    layout: &self.depth_texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&self.depth_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.depth_texture.sampler),
                        },
                    ],
                });
            self.render_target_texture =
                texture::Texture::create_rendering_target(&self.device, &self.config);
            self.render_target_bind_group =
                self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.render_target_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(
                                &self.render_target_texture.view,
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(
                                &self.render_target_texture.sampler,
                            ),
                        },
                    ],
                    label: Some("render target bind group"),
                });
            self.surface.configure(&self.device, &self.config);
        }
    }
    pub fn reconfigure(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }
    /// Eine Funktion um den Status Quo zu verändern.
    pub fn update(&mut self, keys: &crate::input::KeyMap) {
        if self.window.focused() {
            if keys.space_double_tap {
                self.camera.controller().toggle_flying();
            }
            self.camera
                .controller()
                .rotate_around_angle(glam::Vec3::new(
                    -keys.mouse_motion.x as f32,
                    -keys.mouse_motion.y as f32,
                    keys.e.state - keys.q.state,
                ));
            if keys.mouse_wheel.y != 0.0 {
                self.camera.controller().update_acc(keys.mouse_wheel.y)
            }
            self.camera.controller().update(
                glam::Vec3::new(
                    keys.a.state - keys.d.state,
                    keys.space.state - keys.shift.state,
                    keys.w.state - keys.s.state,
                )
                .normalize_or_zero(),
            );
        }
        if keys.p.just_pressed() {
            println!("camera position: {}", self.camera.controller().pos());
        }
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&self.camera.view_proj(self.window.aspect_ratio)),
        );
    }
    pub fn update_mesh(&mut self, mesh: &Mesh) {
        self.mesh = mesh.clone();
        self.instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&self.mesh.instances),
                usage: wgpu::BufferUsages::VERTEX,
            });
    }
    /// Eine Funktion die den Drawer einen neuen Frame zeichnen lässt.
    /// # Errors
    ///
    pub fn draw(&mut self, control_flow: &EventLoopWindowTarget<()>) {
        match self.try_draw() {
            Ok(_) => {}
            // Reconfigure the surface if it's lost or outdated
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => self.reconfigure(),
            // The system is out of memory, we should probably quit
            Err(wgpu::SurfaceError::OutOfMemory) => {
                log::error!("OutOfMemory");
                control_flow.exit();
            }

            // This happens when the a frame takes too long to present
            Err(wgpu::SurfaceError::Timeout) => {
                log::warn!("Surface timeout")
            }
        }
    }
    fn try_draw(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Erster Render-Pass: Szene auf Render-Target zeichnen
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass - Main Scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_target_texture.view, // Render auf Textur
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.mesh.instances.len() as _);
        }

        // Zweiter Render-Pass: Post-Processing mit der ersten Textur als Input
        {
            let mut post_process_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass - Post Processing"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view, // Jetzt auf den Bildschirm rendern
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None, // Kein Depth-Buffer nötig
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            post_process_pass.set_pipeline(&self.post_processing_pipeline);
            post_process_pass.set_bind_group(0, &self.render_target_bind_group, &[]); // Die Szene als Textur-Input
            post_process_pass.set_bind_group(1, &self.depth_texture_bind_group, &[]);
            post_process_pass.draw(0..6, 0..1); // Fullscreen-Quad mit 6 Vertices
        }

        // Sende die Commands an die GPU
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present(); // Ausgabe auf den Bildschirm

        Ok(())
    }
}
