use glam::Vec3;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Instance {
    pos: Vec3,
    kind: u16,
}
unsafe impl bytemuck::Pod for Instance {}
unsafe impl bytemuck::Zeroable for Instance {}

impl Instance {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Vec3 { x, y, z },
            kind: 0,
        }
    }
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vec3>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
    pub fn face_nx(pos: Vec3) -> Self {
        Self { pos, kind: 0 }
    }
    pub fn face_px(pos: Vec3) -> Self {
        Self { pos, kind: 1 }
    }
    pub fn face_ny(pos: Vec3) -> Self {
        Self { pos, kind: 2 }
    }
    pub fn face_py(pos: Vec3) -> Self {
        Self { pos, kind: 3 }
    }
    pub fn face_nz(pos: Vec3) -> Self {
        Self { pos, kind: 4 }
    }
    pub fn face_pz(pos: Vec3) -> Self {
        Self { pos, kind: 5 }
    }
}
