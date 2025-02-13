use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
#[macro_export]
macro_rules! make_pipeline {
    (
        $device:expr,
        $bind_group_layout:expr,
        $config:expr,
        $shader:expr,
        $layout:expr,
    ) => {
        wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&$layout),
            vertex: wgpu::VertexState {
                module: &$shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &$shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: $config.format,
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
                cull_mode: None, // Some(wgpu::Face::Back),
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
        }
    };
}
pub(super) fn make_shader() -> wgpu::ShaderModuleDescriptor<'static> {
    wgpu::ShaderModuleDescriptor {
        label: Some("main"),
        source: wgpu::ShaderSource::Wgsl(
            crate::gpu::pipeline::collect_shader(std::path::PathBuf::from("./src")).into(),
        ),
    }
}
fn collect_shader(path: PathBuf) -> String {
    let mut shader_code = String::new();
    for dir_entry in if let Ok(dir_iter) = fs::read_dir(&path) {
        dir_iter
    } else {
        return shader_code;
    } {
        if let Ok(dir_entry) = dir_entry {
            let entry_path = dir_entry.path();
            if entry_path.is_dir() && entry_path.file_name() != Some(&OsString::from("target")) {
                shader_code += &collect_shader(entry_path)
            } else if entry_path.extension() == Some(&&OsString::from("wgsl")) {
                if let Ok(file_content) = fs::read_to_string(entry_path) {
                    shader_code += &file_content
                }
            }
        }
    }
    shader_code
}
