use glam::{Quat, Vec3};

#[derive(Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}
use std::ops;
impl ops::AddAssign for Mesh {
    fn add_assign(&mut self, mut other: Self) {
        other
            .indices
            .iter_mut()
            .for_each(|index| *index += self.vertices.len() as u32);
        self.indices.append(&mut other.indices);
        self.vertices.append(&mut other.vertices)
    }
}
impl ops::Add for Mesh {
    type Output = Self;
    fn add(mut self, mut other: Self) -> Self {
        self.vertices.append(&mut other.vertices);
        self.indices.append(&mut other.indices);
        self
    }
}
impl ops::Add<Vec<Mesh>> for Mesh {
    type Output = Self;
    fn add(self, other: Vec<Mesh>) -> Self {
        let mut new_mesh = Mesh {
            vertices: Vec::with_capacity(
                self.vertices.len() + other.iter().fold(0, |acc, mesh| acc + mesh.vertices.len()),
            ),
            indices: Vec::with_capacity(
                self.indices.len() + other.iter().fold(0, |acc, mesh| acc + mesh.indices.len()),
            ),
        };
        new_mesh += self;
        for mesh in other {
            new_mesh += mesh
        }
        new_mesh
    }
}
use std::default::Default;
impl Default for Mesh {
    fn default() -> Self {
        Self {
            vertices: Vec::from([
                Vertex {
                    position: Vec3::new(0.5, 0.5, -0.3),
                    tex_coords: [1.0, 0.0],
                }, // A
                Vertex {
                    position: Vec3::new(-0.5, 0.5, -0.3),
                    tex_coords: [0.0, 0.0],
                }, // B
                Vertex {
                    position: Vec3::new(-0.5, -0.5, -0.3),
                    tex_coords: [0.0, 1.0],
                }, // C
                Vertex {
                    position: Vec3::new(0.5, -0.5, -0.3),
                    tex_coords: [1.0, 1.0],
                }, // D
                Vertex {
                    position: Vec3::new(0.5 + 1.0, 0.5, 0.3),
                    tex_coords: [1.0, 0.0],
                }, // A
                Vertex {
                    position: Vec3::new(-0.5 + 1.0, 0.5, 0.3),
                    tex_coords: [0.0, 0.0],
                }, // B
                Vertex {
                    position: Vec3::new(-0.5 + 1.0, -0.5, 0.3),
                    tex_coords: [0.0, 1.0],
                }, // C
                Vertex {
                    position: Vec3::new(0.5 + 1.0, -0.5, 0.3),
                    tex_coords: [1.0, 1.0],
                }, // D
            ]),
            indices: Vec::from([0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4]),
        }
    }
}
impl Mesh {
    pub fn face_nx(pos: Vec3) -> Self {
        Self {
            vertices: vec![
                Vertex {
                    position: pos + Vec3::new(0.0, 0.0, 1.0),
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: pos + Vec3::new(0.0, 0.0, 0.0),
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: pos + Vec3::new(0.0, 1.0, 0.0),
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: pos + Vec3::new(0.0, 1.0, 1.0),
                    tex_coords: [0.0, 0.0],
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }
    pub fn face_px(pos: Vec3) -> Self {
        Self {
            vertices: vec![
                Vertex {
                    position: pos + Vec3::new(1.0, 0.0, 0.0),
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: pos + Vec3::new(1.0, 0.0, 1.0),
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: pos + Vec3::new(1.0, 1.0, 1.0),
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: pos + Vec3::new(1.0, 1.0, 0.0),
                    tex_coords: [0.0, 0.0],
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }
    pub fn face_ny(pos: Vec3) -> Self {
        Self {
            vertices: vec![
                Vertex {
                    position: pos + Vec3::new(0.0, 0.0, 0.0),
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: pos + Vec3::new(0.0, 0.0, 1.0),
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: pos + Vec3::new(1.0, 0.0, 1.0),
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: pos + Vec3::new(1.0, 0.0, 0.0),
                    tex_coords: [0.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }
    pub fn face_py(pos: Vec3) -> Self {
        Self {
            vertices: vec![
                Vertex {
                    position: pos + Vec3::new(0.0, 1.0, 0.0),
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: pos + Vec3::new(1.0, 1.0, 0.0),
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: pos + Vec3::new(1.0, 1.0, 1.0),
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: pos + Vec3::new(0.0, 1.0, 1.0),
                    tex_coords: [0.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }
    pub fn face_nz(pos: Vec3) -> Self {
        Self {
            vertices: vec![
                Vertex {
                    position: pos + Vec3::new(0.0, 0.0, 0.0),
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: pos + Vec3::new(1.0, 0.0, 0.0),
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: pos + Vec3::new(1.0, 1.0, 0.0),
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: pos + Vec3::new(0.0, 1.0, 0.0),
                    tex_coords: [0.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }
    pub fn face_pz(pos: Vec3) -> Self {
        Self {
            vertices: vec![
                Vertex {
                    position: pos + Vec3::new(0.0, 0.0, 1.0),
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: pos + Vec3::new(0.0, 1.0, 1.0),
                    tex_coords: [1.0, 0.0],
                },
                Vertex {
                    position: pos + Vec3::new(1.0, 1.0, 1.0),
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: pos + Vec3::new(1.0, 0.0, 1.0),
                    tex_coords: [0.0, 1.0],
                },
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
        }
    }
}
/// Repräsentiert einen einzigartigen Punkt in der Szene.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub tex_coords: [f32; 2],
}
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
impl Vertex {
    /// Gibt das korrekte Layout für die Vertices zurück.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ],
        }
    }
    pub fn rotate_around_point(&self, axis: Vec3, angle_rad: f32, pivot: Vec3) -> Vec3 {
        // Schritt 1: Punkt ins Ursprungssystem verschieben
        let translated = self.position - pivot;

        // Schritt 2: Quaternion für die Rotation um die Achse erstellen
        let rotation = Quat::from_axis_angle(axis.normalize(), angle_rad);

        // Punkt rotieren
        let rotated = rotation * translated;

        // Schritt 3: Zurückverschieben
        rotated + pivot
    }
}
