use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

pub(super) fn make_shader() -> wgpu::ShaderModuleDescriptor<'static> {
    wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(
            (collect_shader(std::path::PathBuf::from("./src"))
                + &format!(
                    "const RENDER_DISTANCE: f32 = {:.3};",
                    crate::RENDER_DISTANCE
                ))
                .into(),
        ),
    }
}
fn collect_shader(path: PathBuf) -> String {
    let mut shader_code = String::new();
    for dir_entry in if let Ok(dir_iter) = fs::read_dir(&path) {
        dir_iter
    } else {
        panic!("wrong working directory");
    }
    .flatten()
    {
        let entry_path = dir_entry.path();
        if entry_path.is_dir() && entry_path.file_name() != Some(&OsString::from("target")) {
            shader_code += &collect_shader(entry_path)
        } else if entry_path.extension() == Some(&OsString::from("wgsl"))
            && let Ok(file_content) = fs::read_to_string(entry_path)
        {
            shader_code += &file_content
        }
    }
    shader_code
}
