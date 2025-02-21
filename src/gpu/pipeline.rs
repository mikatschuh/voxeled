use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

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
