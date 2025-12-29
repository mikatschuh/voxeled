use glam::IVec3;

use crate::{gpu::texture_set::Texture, server::frustum::LodLevel};
use std::ops;

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
        Self {
            nx: Vec::with_capacity(capacity),
            px: Vec::with_capacity(capacity),
            ny: Vec::with_capacity(capacity),
            py: Vec::with_capacity(capacity),
            nz: Vec::with_capacity(capacity),
            pz: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.nx.len()
            + self.nx.len()
            + self.px.len()
            + self.ny.len()
            + self.py.len()
            + self.nz.len()
            + self.pz.len()
    }

    pub fn add_nx(&mut self, pos: IVec3, texture: Texture, lod: LodLevel) {
        self.nx.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }

    pub fn add_px(&mut self, pos: IVec3, texture: Texture, lod: LodLevel) {
        self.px.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }

    pub fn add_ny(&mut self, pos: IVec3, texture: Texture, lod: LodLevel) {
        self.ny.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }

    pub fn add_py(&mut self, pos: IVec3, texture: Texture, lod: LodLevel) {
        self.py.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }

    pub fn add_nz(&mut self, pos: IVec3, texture: Texture, lod: LodLevel) {
        self.nz.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }

    pub fn add_pz(&mut self, pos: IVec3, texture: Texture, lod: LodLevel) {
        self.pz.push(Instance {
            pos,
            kind: ((lod as u32) << 16) | texture as u32,
        });
    }

    pub fn debug_cube(&mut self, pos: IVec3, size: u16) {
        self.add_nx(pos, Texture::Debug, size);
        self.add_px(pos, Texture::Debug, size);
        self.add_ny(pos, Texture::Debug, size);
        self.add_py(pos, Texture::Debug, size);
        self.add_nz(pos, Texture::Debug, size);
        self.add_pz(pos, Texture::Debug, size);
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
