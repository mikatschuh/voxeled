#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex(u32);

impl Vertex {
    /// Gibt das korrekte Layout für die Vertices zurück.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Uint32,
            }],
        }
    }

    pub fn vertices() -> [Vertex; 4] {
        [Vertex(0), Vertex(1), Vertex(2), Vertex(3)]
    }

    pub fn indices() -> [u16; 6] {
        [0, 1, 2, 0, 2, 3]
    }
}
