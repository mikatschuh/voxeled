use glam::IVec3;

use crate::gpu::texture_set::Texture;

use super::instance::Instance;

#[derive(Debug, Clone)]
pub struct Mesh {
    pub nx: Vec<Instance>,
    pub px: Vec<Instance>,
    pub ny: Vec<Instance>,
    pub py: Vec<Instance>,
    pub nz: Vec<Instance>,
    pub pz: Vec<Instance>,
}
use std::ops;
impl ops::AddAssign<Self> for Mesh {
    fn add_assign(&mut self, mut other: Self) {
        self.nx.append(&mut other.nx);
        self.px.append(&mut other.px);
        self.ny.append(&mut other.ny);
        self.py.append(&mut other.py);
        self.nz.append(&mut other.nz);
        self.pz.append(&mut other.pz);
    }
}
impl ops::Add for Mesh {
    type Output = Self;
    fn add(mut self, mut other: Self) -> Self {
        self.nx.append(&mut other.nx);
        self.px.append(&mut other.px);
        self.ny.append(&mut other.ny);
        self.py.append(&mut other.py);
        self.nz.append(&mut other.nz);
        self.pz.append(&mut other.pz);
        self
    }
}
impl Mesh {
    pub fn new() -> Self {
        Self {
            nx: vec![],
            px: vec![],
            ny: vec![],
            py: vec![],
            nz: vec![],
            pz: vec![],
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        let ind_capacity = capacity;
        Self {
            nx: Vec::with_capacity(ind_capacity),
            px: Vec::with_capacity(ind_capacity),
            ny: Vec::with_capacity(ind_capacity),
            py: Vec::with_capacity(ind_capacity),
            nz: Vec::with_capacity(ind_capacity),
            pz: Vec::with_capacity(ind_capacity),
        }
    }
    pub fn add_nx(&mut self, pos: IVec3, texture: Texture) {
        self.nx.push(Instance {
            pos,
            kind: texture as u32,
        });
    }
    pub fn add_px(&mut self, pos: IVec3, texture: Texture) {
        self.px.push(Instance {
            pos,
            kind: texture as u32,
        });
    }
    pub fn add_ny(&mut self, pos: IVec3, texture: Texture) {
        self.ny.push(Instance {
            pos,
            kind: texture as u32,
        });
    }
    pub fn add_py(&mut self, pos: IVec3, texture: Texture) {
        self.py.push(Instance {
            pos,
            kind: texture as u32,
        });
    }
    pub fn add_nz(&mut self, pos: IVec3, texture: Texture) {
        self.nz.push(Instance {
            pos,
            kind: texture as u32,
        });
    }
    pub fn add_pz(&mut self, pos: IVec3, texture: Texture) {
        self.pz.push(Instance {
            pos,
            kind: texture as u32,
        });
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
