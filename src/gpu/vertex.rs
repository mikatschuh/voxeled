use glam::{Quat, Vec3};
/// Repräsentiert einen einzigartigen Punkt in der Szene.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub(super) struct Vertex {
    pub position: Vec3,
    pub tex_coords: [f32; 2],
}
unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
pub(super) const VERTICES: &[Vertex] = &[
    Vertex {
        position: Vec3::new(0.5, 0.5, 1.0),
        tex_coords: [1.0, 0.0],
    }, // A
    Vertex {
        position: Vec3::new(-0.5, 0.5, 1.0),
        tex_coords: [0.0, 0.0],
    }, // B
    Vertex {
        position: Vec3::new(-0.5, -0.5, 1.0),
        tex_coords: [0.0, 1.0],
    }, // C
    Vertex {
        position: Vec3::new(0.5, -0.5, 1.0),
        tex_coords: [1.0, 1.0],
    }, // D
    Vertex {
        position: Vec3::new(0.5 + 1.0, 0.5, 0.7),
        tex_coords: [1.0, 0.0],
    }, // A
    Vertex {
        position: Vec3::new(-0.5 + 1.0, 0.5, 0.7),
        tex_coords: [0.0, 0.0],
    }, // B
    Vertex {
        position: Vec3::new(-0.5 + 1.0, -0.5, 0.7),
        tex_coords: [0.0, 1.0],
    }, // C
    Vertex {
        position: Vec3::new(0.5 + 1.0, -0.5, 0.7),
        tex_coords: [1.0, 1.0],
    }, // D
];
pub const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];
impl Vertex {
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
