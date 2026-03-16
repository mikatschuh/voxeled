use std::{collections::HashMap, time::Instant};

use texture::Texture;
use winit::{dpi::PhysicalSize, event_loop::EventLoopWindowTarget};

use crate::{
    config::Config,
    gpu::{
        gpu_allocator::GPUSlotAllocator,
        profiling::PerformanceStats,
        projection::{Projection, View},
    },
};

// pub mod exotic_cameras;
#[allow(dead_code)]
mod gpu_allocator;
mod profiling;
pub mod projection;
mod shader;
mod texture;
pub mod texture_set;
pub mod window;

/// Ein Drawer. Der Drawer ist der Zugang zur Graphikkarte. Er ist an ein Fenster genüpft.
pub struct Gpu<'a> {
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

    render_pipeline: wgpu::RenderPipeline,
    post_processing_pipeline: wgpu::RenderPipeline,

    // camera
    proj: Projection,
    view_proj_buffer: wgpu::Buffer,

    // Asset things:
    vertices_per_face: u32,

    vram_cache: gpu_allocator::GPUSlotAllocator,
    mesh_map: HashMap<voxine::ChunkID, (u64, gpu_allocator::SlotID)>,
    frustum_allocs: voxine::FrustumAllocations,
    perf_stats: PerformanceStats,
}

impl<'a> Gpu<'a> {
    /// Diese Funktion erstellt einen Drawer der mit dem aktuellen Fenster verbunden ist.
    /// Außerdem nimmt sie einen PresentMode entgegen mit dem auf das Fenster gezeichnet werden soll.
    pub async fn connect_to(
        window: &'a winit::window::Window,
        present_mode: wgpu::PresentMode,
        config: &Config,
    ) -> Gpu<'a> {
        let PhysicalSize { width, height } = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();
        let required_features = wgpu::Features::PUSH_CONSTANTS;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let device_descriptor = wgpu::DeviceDescriptor {
            required_features,
            required_limits: wgpu::Limits {
                max_push_constant_size: 16, // chunk metadata = 4 * u32
                ..Default::default()
            },
            ..Default::default()
        };

        let future = adapter.request_device(&device_descriptor);
        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let size = window.inner_size();

        let surface_config = wgpu::SurfaceConfiguration {
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
        if surface_config.width > 0 && surface_config.height > 0 {
            surface.configure(&device, &surface_config);
        }

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2Array,
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

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: std::mem::size_of::<[[f32; 4]; 4]>() as u64,
            mapped_at_creation: false,
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
                label: Some("Camera Bind Group Layout"),
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
        let depth_texture = Texture::create_depth_texture(&device, &surface_config);
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

        let render_target = Texture::create_rendering_target(&device, &surface_config);

        Self {
            proj: Projection::new(width, height, config.fov, config.near_plane),

            mesh_map: HashMap::with_capacity(10_000),
            vram_cache: GPUSlotAllocator::new(32 * 32 * 4, 100_000),
            frustum_allocs: voxine::FrustumAllocations::default(config.max_chunks),

            diffuse_bind_group: {
                let texture = Texture::from_images(
                    &device,
                    &queue,
                    &[
                        image::load_from_memory(texture_set::Texture::CrackedStone.bytes())
                            .unwrap(),
                        image::load_from_memory(texture_set::Texture::Stone.bytes()).unwrap(),
                        image::load_from_memory(texture_set::Texture::Dirt0.bytes()).unwrap(),
                        image::load_from_memory(texture_set::Texture::Dirt1.bytes()).unwrap(),
                        image::load_from_memory(texture_set::Texture::Debug.bytes()).unwrap(),
                    ],
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
            surface,
            queue,
            depth_texture,
            depth_texture_bind_group,
            render_pipeline: device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: Some("Render Pipeline Layout"),
                        bind_group_layouts: &[
                            &texture_bind_group_layout,
                            &camera_bind_group_layout,
                        ],
                        push_constant_ranges: &[wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::VERTEX,
                            range: 0..16,
                        }],
                    }),
                ),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[voxine::Instance::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: surface_config.format,
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
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back), // DEBUG: Culling komplett deaktiviert
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
                    depth_compare: wgpu::CompareFunction::Greater,
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
                        entry_point: Some("full_screen_quat"),
                        buffers: &[], // Keine Vertex-Daten, weil wir ein Fullscreen-Quad generieren
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("post_processing"),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: surface_config.format, // Das Renderziel (z. B. Swapchain-Format)
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
            config: surface_config,
            view_proj_buffer: camera_buffer,
            vertices_per_face: 4,
            perf_stats: PerformanceStats::new(),
        }
    }
    /// Eine Methode welche die Fenstergröße anpasst.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            let PhysicalSize { width, height } = new_size;

            self.config.width = width;
            self.config.height = height;

            self.proj.resize(width, height);

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
    pub fn update_view(&mut self, view: View) {
        self.queue.write_buffer(
            &self.view_proj_buffer,
            0,
            bytemuck::cast_slice(&self.proj.calc_matrix(view)),
        );
    }

    pub fn update_mesh(
        &mut self,
        mesh_recv: &mut voxine::MpscReceiver<(voxine::ChunkID, voxine::Mesh)>,
        allowed_time: f64,
    ) {
        let now = Instant::now();
        while let Ok((chunk_id, mesh)) = mesh_recv.pop() {
            let upload_start = Instant::now();
            let mesh_len = mesh.len_in_bytes() as u64;
            if let Some((slot_size, slot_id)) = self.mesh_map.get_mut(&chunk_id) {
                let updated_slot =
                    self.vram_cache
                        .write_slot(&self.device, &self.queue, *slot_id, mesh.bytes());
                *slot_id = updated_slot;
                *slot_size = mesh_len;
            } else {
                let allocated_slot = self
                    .vram_cache
                    .allocate_slot(&self.device, mesh.len_in_bytes());
                let updated_slot = self.vram_cache.write_slot(
                    &self.device,
                    &self.queue,
                    allocated_slot,
                    mesh.bytes(),
                );

                self.mesh_map.insert(chunk_id, (mesh_len, updated_slot));
            }
            self.perf_stats.mesh_updates += 1;
            self.perf_stats.uploaded_bytes += mesh_len;
            self.perf_stats.mesh_update_time.add(upload_start.elapsed());
            self.perf_stats.maybe_report();
            if now.elapsed().as_secs_f64() >= allowed_time {
                return;
            }
        }
    }

    /// Eine Funktion die den Drawer einen neuen Frame zeichnen lässt.
    /// # Errors
    ///
    pub fn draw(&mut self, frustum: voxine::Frustum, control_flow: &EventLoopWindowTarget<()>) {
        match self.try_draw(frustum) {
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
            Err(wgpu::SurfaceError::Other) => log::warn!("generic error while drawing"),
        }
    }

    fn try_draw(&mut self, frustum: voxine::Frustum) -> Result<(), wgpu::SurfaceError> {
        let draw_start = Instant::now();
        let acquire_start = Instant::now();
        let output = self.surface.get_current_texture()?;
        self.perf_stats.acquire_time.add(acquire_start.elapsed());
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Erster Render-Pass: Szene auf Render-Target zeichnen
        let main_pass_start = Instant::now();
        let mut visible_chunks = 0_u64;
        let mut visible_faces = 0_u64;
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
                        load: wgpu::LoadOp::Clear(0.0),
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

            let chunks = frustum.flood_fill(&mut self.frustum_allocs, &self.mesh_map);

            for chunk in chunks {
                let Some((size, slot_id)) = self.mesh_map.get(&chunk).cloned() else {
                    continue;
                };
                if size == 0 {
                    continue;
                }

                let chunk_bytes = chunk.bytes();
                render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, &chunk_bytes);

                let (buffer, offset) = self.vram_cache.buffer_and_offset(slot_id);
                render_pass.set_vertex_buffer(0, buffer.slice(offset..offset + size));

                let face_count = size >> 2;
                render_pass.draw(0..self.vertices_per_face, 0..face_count as u32);
                visible_chunks += 1;
                visible_faces += face_count;
            }
        }
        self.perf_stats
            .main_pass_time
            .add(main_pass_start.elapsed());
        self.perf_stats.visible_chunks += visible_chunks;
        self.perf_stats.visible_faces += visible_faces;

        // Zweiter Render-Pass: Post-Processing mit der ersten Textur als Input
        let post_process_start = Instant::now();
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
        self.perf_stats
            .post_process_time
            .add(post_process_start.elapsed());

        // Sende die Commands an die GPU
        let submit_start = Instant::now();
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present(); // Ausgabe auf den Bildschirm
        self.perf_stats
            .submit_present_time
            .add(submit_start.elapsed());
        self.perf_stats.frames += 1;
        self.perf_stats.total_draw_time.add(draw_start.elapsed());
        self.perf_stats.maybe_report();

        Ok(())
    }
}
