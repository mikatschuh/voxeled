use glam::{Quat, Vec3};
/// Repräsentiert einen einzigartigen Punkt in der Szene.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub tex_coords: [f32; 2],
}
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
pub(super) const VERTICES: &[Vertex] = &[
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
];
pub const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];
impl Vertex {
    /// Gibt die Szene zurück.
    pub fn get_scene() -> (&'static [Vertex], &'static [u16]) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        Self::add_cube(Vec3::new(0.0, 0.0, 0.0), &mut vertices, &mut indices);
        (vertices.leak(), indices.leak())
    }
    /// Fügt einen Würfel hinzu.
    pub fn add_cube(location: Vec3, vertices: &mut Vec<Vertex>, indices: &mut Vec<u16>) {
        vertices.append(&mut Vec::from([
            Vertex {
                position: location,
                tex_coords: [0.0, 0.0],
            }, // A
            Vertex {
                position: location + Vec3::new(1.0, 0.0, 0.0),
                tex_coords: [1.0, 0.0],
            }, // B
            Vertex {
                position: location + Vec3::new(1.0, 0.0, 1.0),
                tex_coords: [1.0, 1.0],
            }, // C
            Vertex {
                position: location + Vec3::new(0.0, 0.0, 1.0),
                tex_coords: [0.0, 1.0],
            }, // D
            Vertex {
                position: location + Vec3::new(0.0, 1.0, 0.0),
                tex_coords: [0.0, 0.0],
            }, // E
            Vertex {
                position: location + Vec3::new(1.0, 1.0, 0.0),
                tex_coords: [0.0, 0.0],
            }, // F
            Vertex {
                position: location + Vec3::new(1.0, 1.0, 1.0),
                tex_coords: [0.0, 0.0],
            }, // G
            Vertex {
                position: location + Vec3::new(0.0, 1.0, 1.0),
                tex_coords: [0.0, 0.0],
            }, // H
        ]));
        let mut new_indices = [
            0, 2, 1, 0, 3, 2, // down
            0, 1, 5, 0, 5, 4, // front
            0, 7, 3, 0, 4, 7, // left
            3, 6, 2, 3, 7, 6, // back
            1, 2, 6, 1, 6, 5, // right
            4, 5, 6, 4, 6, 7, // up
        ];
        for index in new_indices.iter_mut() {
            *index += indices.len() as u16
        }

        indices.append(&mut Vec::from(new_indices))
    }
    /// Gibt das korrekte Layout für die Vertices zurück.
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        let mut average_position = Vec3::ZERO;
        for vertex in VERTICES.iter() {
            average_position += vertex.position;
        }
        average_position /= VERTICES.len() as f32;
        // println!("{:?}", average_position);

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
