use super::chunk::{generate_mesh, Chunk};
use super::voxel::AnimatedNoise;
use glam::IVec3;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

pub struct Chunks {
    chunks: HashMap<IVec3, Mutex<Chunk>>,
}
impl Default for Chunks {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
}

pub struct ChunkHandle<'a> {
    chunk: MutexGuard<'a, Chunk>,
}

use crate::gpu::mesh::Mesh;
impl Chunks {
    pub fn get<'a, F>(&'a mut self, pos: IVec3, mut gen: F) -> MutexGuard<'a, Chunk>
    where
        F: FnMut(IVec3) -> Chunk,
    {
        self.chunks
            .entry(pos)
            .or_insert_with(|| Mutex::new(gen(pos)))
            .lock()
            .unwrap()
    }
    pub fn create_mesh(
        &mut self,
        cam_pos: Vec3,
        viewing_dir: Vec3,
        fov: f32,
        aspect_ratio: f32,
        render_distance: f32,
        noise: &AnimatedNoise,
        time: f64,
    ) -> Mesh {
        let mut mesh = Mesh::default();

        for chunk_coord in
            generate_frustum_points(cam_pos, viewing_dir, fov, aspect_ratio, render_distance)
        {
            // let now = Instant::now();
            mesh += generate_mesh(
                cam_pos,
                chunk_coord,
                self.get(chunk_coord, |pos| {
                    Chunk::from_perlin_noise(pos, noise, time)
                })
                .create_faces(),
            );
            // println!("time it took (single chunk): {:#?}", now.elapsed());
        }

        mesh
    }
}

use glam::{Mat3, Vec3};
use std::collections::HashSet;

pub fn generate_frustum_points(
    position: Vec3,
    direction: Vec3,
    fov: f32,
    aspect_ratio: f32,
    render_distance: f32,
) -> Vec<IVec3> {
    let mut points = HashSet::new();

    // Normalisiere die Blickrichtung
    let forward = direction.normalize();

    // Berechne die Up und Right Vektoren
    let right = if forward.y.abs() > 0.999 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        forward.cross(Vec3::Y).normalize()
    };
    let up = right.cross(forward).normalize();

    // Berechne die Frustum-Parameter
    let tan_half_fov = (fov / 2.0).tan();

    // Berechne die View-Matrix
    let view_matrix = Mat3::from_cols(right, up, forward);

    // Füge den Startpunkt und direkte Nachbarn hinzu
    let start_pos = IVec3::new(
        position.x.round() as i32,
        position.y.round() as i32,
        position.z.round() as i32,
    );

    // Füge den Startblock und seine direkten Nachbarn hinzu
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                points.insert(start_pos + IVec3::new(dx, dy, dz));
            }
        }
    }

    // Iteriere durch alle möglichen Z-Werte, starte bei einem kleinen z-Wert
    let min_z = 1;
    for z in min_z..=(render_distance as i32) {
        let current_distance = z as f32;

        // Berechne die aktuelle Breite und Höhe des Frustums an dieser Z-Position
        let current_height = current_distance * tan_half_fov;
        let current_width = current_height * aspect_ratio;

        let steps_y = (current_height * 2.0).ceil() as i32;
        let steps_x = (current_width * 2.0).ceil() as i32;

        // Iteriere durch alle X und Y Werte an dieser Z-Position
        for y_step in -steps_y..=steps_y {
            let y = (y_step as f32 / steps_y as f32) * current_height;

            for x_step in -steps_x..=steps_x {
                let x = (x_step as f32 / steps_x as f32) * current_width;

                // Berechne den Punkt im View Space
                let view_space = Vec3::new(x, y, current_distance);

                // Transformiere in World Space
                let world_offset = view_matrix * view_space;
                let world_pos = position + world_offset;

                // Konvertiere zu ganzen Zahlen und füge den Punkt hinzu
                points.insert(IVec3::new(
                    world_pos.x.round() as i32,
                    world_pos.y.round() as i32,
                    world_pos.z.round() as i32,
                ));
            }
        }
    }

    points.into_iter().collect()
}
