pub mod chunk;
pub mod voxel;
use crate::random::AnimatedNoise;
use chunk::Chunk;
use colored::Colorize;
use crossbeam::channel::{bounded, Receiver};
use glam::IVec3;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

enum LazyMesh {
    Mesh(Box<Mesh>),
    Recv(Receiver<Box<Mesh>>),
}
const PRELOAD_DISTANCE: usize = 10;

pub struct Server {
    chunks: Arc<Mutex<Chunks>>,
    meshes: HashMap<IVec3, LazyMesh>,
    was_initiated: bool,
}
impl Server {
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(Mutex::new(Chunks::default())),
            meshes: HashMap::new(),
            was_initiated: false,
        }
    }
    pub fn init(&mut self, noise: Arc<AnimatedNoise>, threadpool: &mut Threadpool) {
        for_point_in_square(IVec3::ZERO, 10, |chunk_coord| {
            if let Some(..) = self.meshes.get(&chunk_coord) {
            } else {
                let chunk_coord = chunk_coord.clone();

                let noise = noise.clone();

                let (s, r) = bounded(1);
                self.meshes.insert(chunk_coord, LazyMesh::Recv(r));

                threadpool.dynamic_priority(move || {
                    let now = Instant::now();
                    let result = Box::new(crate::server::chunk::generate_mesh_without_cam_occ(
                        chunk_coord,
                        Chunk::from_perlin_noise(chunk_coord, &noise, 0.0).occlusion_map,
                    ));
                    let _ = s.send(result);
                    let time = now.elapsed();
                    let msg = format!(
                        "time it took to build the chunk mesh at {:?}: {:#?}",
                        chunk_coord, time
                    );
                    let msg = if time.as_micros() < 1000 {
                        msg.green()
                    } else {
                        msg.red()
                    };
                    // println!("{}", msg);
                });
            }
        });
        self.was_initiated = true;
        println!(
            "chunks pre-build, size of world: {} kB",
            self.chunks.lock().unwrap().chunks.len() * size_of::<Chunk>() / 1000
        );
    }
    pub fn get_mesh(
        &mut self,
        cam_pos: Vec3,
        viewing_dir: Vec3,
        fov: f32,
        aspect_ratio: f32,
        render_distance: usize,
        noise: Arc<AnimatedNoise>,
        time: f64,
        threadpool: &mut Threadpool,
    ) -> Mesh {
        if !self.was_initiated {
            self.init(noise.clone(), threadpool)
        }
        let mut mesh = Mesh {
            vertices: Vec::with_capacity(2457600),
            indices: Vec::with_capacity(18432000),
        };
        let cam_chunk_pos = cam_pos / 32.0;

        let points = every_chunk_in_frustum(
            cam_chunk_pos,
            viewing_dir,
            fov,
            aspect_ratio,
            render_distance,
        );
        points.iter().for_each(|chunk_coord| {
            if let Some(lazy_mesh) = self.meshes.get_mut(chunk_coord) {
                match lazy_mesh {
                    LazyMesh::Mesh(chunk_mesh) => mesh += *chunk_mesh.clone(),

                    LazyMesh::Recv(recv) => {
                        if let Ok(chunk_mesh) = recv.try_recv() {
                            mesh += *chunk_mesh.clone();
                            *lazy_mesh = LazyMesh::Mesh(chunk_mesh)
                        }
                    }
                }
            } else {
                let chunk_coord = chunk_coord.clone();

                let noise = noise.clone();

                let (s, r) = bounded(1);
                self.meshes.insert(chunk_coord, LazyMesh::Recv(r));

                threadpool.dynamic_priority(move || {
                    let now = Instant::now();
                    let result = Box::new(crate::server::chunk::generate_mesh_without_cam_occ(
                        chunk_coord,
                        Chunk::from_perlin_noise(chunk_coord, &noise, 0.0).occlusion_map,
                    ));
                    let _ = s.send(result);
                    let time = now.elapsed();
                    let msg = format!(
                        "time it took to build the chunk mesh at {:?}: {:#?}",
                        chunk_coord, time
                    );
                    let msg = if time.as_micros() < 1000 {
                        msg.green()
                    } else {
                        msg.red()
                    };
                    // println!("{}", msg);
                });
            }
        });
        points.iter().for_each(|chunk_coord| {
            if let Some(lazy_mesh) = self.meshes.get_mut(chunk_coord) {
                if let LazyMesh::Recv(recv) = lazy_mesh {
                    // println!("{}: waiting", chunk_coord);
                    if let Ok(chunk_mesh) = recv.try_recv() {
                        // println!("{}: got mesh", chunk_coord);
                        mesh += *chunk_mesh.clone();
                        *lazy_mesh = LazyMesh::Mesh(chunk_mesh)
                    }
                }
            }
        });
        // println!("frame done, size of mesh: {} kB", (mesh.vertices.len() * size_of::<crate::gpu::mesh::Vertex>() + mesh.indices.len() * 4) / 1000);
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

pub fn every_chunk_in_frustum(
    position: Vec3,
    direction: Vec3,
    fov: f32,
    aspect_ratio: f32,
    render_distance: usize,
) -> Vec<IVec3> {
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
        let current_height = current_distance * tan_half_fov + 1.0;
        let current_width = current_height * aspect_ratio + 1.0;

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

    points.into_iter().collect::<Vec<IVec3>>()
}
fn for_every_sphere_point<F>(center: IVec3, radius: usize, mut closure: F)
where
    F: FnMut(&IVec3),
{
    struct WayPoint {
        pos: IVec3,
        fuel: usize,
        pointing: IVec3,
    }
    let mut covered_points = HashSet::from([(center)]);

    let new_fuel = radius - 1;
    let mut potential_ways = vec![
        WayPoint {
            pos: center + IVec3::new(1, 0, 0),
            fuel: new_fuel,
            pointing: IVec3::new(1, 0, 0),
        },
        WayPoint {
            pos: center + IVec3::new(-1, 0, 0),
            fuel: new_fuel,
            pointing: IVec3::new(-1, 0, 0),
        },
        WayPoint {
            pos: center + IVec3::new(0, 1, 0),
            fuel: new_fuel,
            pointing: IVec3::new(0, 1, 0),
        },
        WayPoint {
            pos: center + IVec3::new(0, -1, 0),
            fuel: new_fuel,
            pointing: IVec3::new(0, -1, 0),
        },
        WayPoint {
            pos: center + IVec3::new(0, 0, 1),
            fuel: new_fuel,
            pointing: IVec3::new(0, 0, 1),
        },
        WayPoint {
            pos: center + IVec3::new(0, 0, -1),
            fuel: new_fuel,
            pointing: IVec3::new(0, 0, -1),
        },
    ];
    while let Some(way) = potential_ways.pop() {
        covered_points.insert(way.pos);
        if way.fuel > 0 {
            let new_fuel = way.fuel - 1;
            if !covered_points.contains(&(way.pos + way.pointing)) {
                potential_ways.push(WayPoint {
                    pos: way.pos + way.pointing,
                    fuel: new_fuel,
                    pointing: way.pointing,
                })
            };
            let dir = IVec3::new(1, 0, 0);
            if !covered_points.contains(&(way.pos + dir)) && way.pointing != -dir {
                potential_ways.push(WayPoint {
                    pos: way.pos + dir,
                    fuel: new_fuel,
                    pointing: dir,
                })
            };
            let dir = IVec3::new(-1, 0, 0);
            if !covered_points.contains(&(way.pos + dir)) {
                potential_ways.push(WayPoint {
                    pos: way.pos + dir,
                    fuel: new_fuel,
                    pointing: dir,
                })
            }
            let dir = IVec3::new(0, 1, 0);
            if !covered_points.contains(&(way.pos + dir)) && way.pointing != -dir {
                potential_ways.push(WayPoint {
                    pos: way.pos + dir,
                    fuel: new_fuel,
                    pointing: dir,
                })
            }
            let dir = IVec3::new(0, -1, 0);
            if !covered_points.contains(&(way.pos + dir)) && way.pointing != -dir {
                potential_ways.push(WayPoint {
                    pos: way.pos + dir,
                    fuel: new_fuel,
                    pointing: dir,
                })
            }
            let dir = IVec3::new(0, 0, 1);
            if !covered_points.contains(&(way.pos + dir)) && way.pointing != -dir {
                potential_ways.push(WayPoint {
                    pos: way.pos + dir,
                    fuel: new_fuel,
                    pointing: dir,
                })
            }
            let dir = IVec3::new(0, 0, -1);
            if !covered_points.contains(&(way.pos + dir)) && way.pointing != -dir {
                potential_ways.push(WayPoint {
                    pos: way.pos + dir,
                    fuel: new_fuel,
                    pointing: dir,
                })
            }
        }
    }
    covered_points.iter().for_each(|point| closure(point));
}
fn for_point_in_square<F>(pos: IVec3, edge_length: i32, mut f: F)
where
    F: FnMut(IVec3),
{
    for x in (-edge_length + pos.x)..(edge_length + pos.x) {
        for y in (-edge_length + pos.x)..(edge_length + pos.x) {
            for z in (-edge_length + pos.x)..(edge_length + pos.x) {
                f(IVec3::new(x, y, z))
            }
        }
    }
}
