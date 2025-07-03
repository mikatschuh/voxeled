use glam::IVec3;

use crate::gpu::texture_set;

/// The kind states the orientation and the texture.
/// It has the following layout:
/// ```
///                                           |texture        |orientation
/// |0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|0|
/// ```

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Instance {
    pos: IVec3,
    kind: u32,
}
unsafe impl bytemuck::Pod for Instance {}
unsafe impl bytemuck::Zeroable for Instance {}

impl Instance {
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
                    format: wgpu::VertexFormat::Sint32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<IVec3>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
    pub fn face_nx(pos: IVec3, texture: texture_set::Texture) -> Self {
        Self {
            pos,
            kind: (texture as u32) << 3 | 0,
        }
    }
    pub fn face_px(pos: IVec3, texture: texture_set::Texture) -> Self {
        Self {
            pos,
            kind: (texture as u32) << 3 | 1,
        }
    }
    pub fn face_ny(pos: IVec3, texture: texture_set::Texture) -> Self {
        Self {
            pos,
            kind: (texture as u32) << 3 | 2,
        }
    }
    pub fn face_py(pos: IVec3, texture: texture_set::Texture) -> Self {
        Self {
            pos,
            kind: (texture as u32) << 3 | 3,
        }
    }
    pub fn face_nz(pos: IVec3, texture: texture_set::Texture) -> Self {
        Self {
            pos,
            kind: (texture as u32) << 3 | 4,
        }
    }
    pub fn face_pz(pos: IVec3, texture: texture_set::Texture) -> Self {
        Self {
            pos,
            kind: (texture as u32) << 3 | 5,
        }
    }
}
