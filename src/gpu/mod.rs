pub mod camera;
pub mod camera_controller;
// pub mod exotic_cameras;
pub mod mesh;
mod pipeline;
mod texture;
pub mod window;

use crate::make_pipeline;
use camera::Camera3d;
use camera_controller::CameraController;
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
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    pub window: window::Window<'a>,

    render_pipeline: wgpu::RenderPipeline,

    camera: &'a mut C,
    camera_buffer: wgpu::Buffer,

    // Asset things:
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
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
        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

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
        let render_pipeline = device.create_render_pipeline(&make_pipeline!(
            &device,
            &bind_group_layout,
            &config,
            device.create_shader_module(crate::gpu::pipeline::make_shader()),
            &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            }),
        ));
        let mesh = Mesh::default();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            _phantom: std::marker::PhantomData,
            diffuse_bind_group: {
                let texture = Texture::from_image(
                    &device,
                    &queue,
                    &image::load_from_memory(include_bytes!("stone.png")).unwrap(),
                    Some("Test Texture"),
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
            window: window::Window::from(window, true),
            surface,
            device,
            queue,
            config,
            depth_texture,
            render_pipeline,
            camera,
            camera_buffer,
            vertex_buffer,
            index_buffer,
            num_indices: mesh.indices.len() as u32,
        }
    }
    /// Eine Methode welche die Fenstergröße anpasst.
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window.resize(new_size);
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.surface.configure(&self.device, &self.config);
        }
    }
    pub fn reconfigure(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }
    /// Eine Funktion um den Status Quo zu verändern.
    pub fn update(&mut self, keys: &crate::input::KeyMap) {
        if self.window.focused() {
            self.camera
                .controller()
                .rotate_around_angle(glam::Vec3::new(
                    -keys.mouse_motion.x as f32,
                    keys.mouse_motion.y as f32,
                    keys.e.state - keys.q.state,
                ));
            if keys.mouse_wheel.y != 0.0 {
                self.camera.controller().update_acc(keys.mouse_wheel.y)
            }
            self.camera.controller().update(
                glam::Vec3::new(
                    keys.a.state - keys.d.state,
                    keys.shift.state - keys.space.state,
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
    pub fn camera(&mut self) -> &mut C {
        self.camera
    }
    pub fn update_mesh(&mut self, mesh: &Mesh) {
        self.num_indices = mesh.indices.len() as u32;
        self.vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        self.index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
    }
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

    /// Eine Funktion die den Drawer einen neuen Frame zeichnen lässt.
    /// # Errors
    ///
    pub fn try_draw(&mut self) -> Result<(), wgpu::SurfaceError> {
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
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
                    }),
                ],
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
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present(); // sending to gpu
        Ok(())
    }
}
