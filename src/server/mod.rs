pub mod chunk;
pub mod voxel;
use chunk::Chunk;
use glam::IVec3;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use voxel::AnimatedNoise;

pub struct Server {
    chunks: Arc<Mutex<Chunks>>,
}
impl Server {
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(Mutex::new(Chunks::default())),
        }
    }
    pub fn get_mesh(
        &mut self,
        cam_pos: Vec3,
        viewing_dir: Vec3,
        fov: f32,
        aspect_ratio: f32,
        render_distance: f32,
        noise: Arc<AnimatedNoise>,
        time: f64,
        threadpool: &mut Threadpool,
    ) -> Mesh {
        let mut mesh = Mesh::default();

        let cam_chunk_pos = cam_pos / 32.0;

        let mut meshes: Vec<Arc<Mutex<Mesh>>> = Vec::new();

        for_every_chunk_in_frustum(
            cam_chunk_pos,
            viewing_dir,
            fov,
            aspect_ratio,
            render_distance,
            |chunk_coord| {
                mesh += crate::server::chunk::generate_mesh(
                    cam_pos,
                    *chunk_coord,
                    self.chunks
                        .lock()
                        .unwrap()
                        .get(*chunk_coord, |pos| {
                            Chunk::from_fractal_noise(pos, &noise, 0.0)
                        })
                        .create_faces(),
                );

                /*let chunk_coord = chunk_coord.clone();
                let chunks = self.chunks.clone();
                meshes.push(Arc::new(Mutex::new(Mesh::default())));
                let mesh_pointer = meshes.last_mut().unwrap().clone();
                let noise = noise.clone();
                threadpool.priority(move |i| {
                    *mesh_pointer.lock().unwrap() = crate::server::chunk::generate_mesh(
                        cam_pos,
                        chunk_coord,
                        chunks
                            .lock()
                            .unwrap()
                            .get(chunk_coord, |pos| {
                                Chunk::from_fractal_noise(pos, &noise, 0.0)
                            })
                            .create_faces(),
                    );
                });*/
            },
        );
        for new_mesh in meshes.into_iter() {
            // mesh += new_mesh.lock().unwrap().clone()
        }

        mesh
    }
}

#[derive(Debug)]
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

use crate::gpu::mesh::Mesh;
use crate::threader::task::Task;
use crate::threader::Threadpool;
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
}

use glam::{Mat3, Vec3};
use std::collections::HashSet;

pub fn for_every_chunk_in_frustum<F>(
    position: Vec3,
    direction: Vec3,
    fov: f32,
    aspect_ratio: f32,
    render_distance: f32,
    mut first: F,
) where
    F: FnMut(&IVec3),
{
    let mut points = HashSet::new();

    // Normalisiere die Blickrichtung
    let forward = (-direction).normalize();

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
    points.insert(start_pos);

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

    points.iter().map(|point| first(point)).collect::<()>();
}
