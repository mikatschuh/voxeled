use super::Window;
use crate::make_pipeline;
use glam::Vec3;
use vertex::*;
use wgpu::util::DeviceExt;

mod camera;
mod vertex;

pub struct State<'a> {
    // rendering stuff:
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Window<'a>,
    focused: bool,
    render_pipeline: wgpu::RenderPipeline,
    // scene: Scene,
    // scene_buffer: wgpu::Buffer,
    // scene_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a winit::window::Window) -> State<'a> {
        let size = window.inner_size();

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
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate, // surface_caps.present_modes[0] will select it at runtime
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        let (device, queue) = future.await.unwrap();

        let render_pipeline = device.create_render_pipeline(&make_pipeline!(
            &device,
            &bind_group_layout,
            &config,
            &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("main"),
                source: wgpu::ShaderSource::Wgsl(
                    crate::gpu::render::collect_shader(std::path::PathBuf::from(".")).into(),
                ),
            })
        ));
        const SIZE: usize = 4; // Größe des Buffers in Bytes

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_vertices = VERTICES.len() as u32;

        Self {
            window: Window::new(window),
            focused: true,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices,
        }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn update(&mut self, keys: &crate::input::Keys, delta_time: f32) {
        Vec3::new(
            (keys.w - keys.s) as f32,
            (keys.space - keys.shift) as f32,
            (keys.d - keys.a) as f32,
        )
        .normalize_or_zero();
        if keys.esc.just_pressed() {
            self.window.flip_focus()
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                                a: 0.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present(); // sending to gpu
        Ok(())
    }
}
