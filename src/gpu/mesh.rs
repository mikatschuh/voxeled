use super::instance::Instance;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub instances: Vec<Instance>,
}
use std::ops;
impl ops::AddAssign<Self> for Mesh {
    fn add_assign(&mut self, mut other: Self) {
        self.instances.append(&mut other.instances);
    }
}
impl ops::AddAssign<Instance> for Mesh {
    fn add_assign(&mut self, other: Instance) {
        self.instances.push(other)
    }
}
impl ops::Add for Mesh {
    type Output = Self;
    fn add(mut self, mut other: Self) -> Self {
        self.instances.append(&mut other.instances);
        self
    }
}
impl Mesh {
    pub fn new() -> Self {
        Self { instances: vec![] }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            instances: Vec::with_capacity(capacity),
        }
    }
}
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
