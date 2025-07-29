pub mod chunk;
pub mod voxel;

use crate::{gpu::mesh::Mesh, random::AnimatedNoise, threader::lazy::Lazy, threader::Threadpool};
use chunk::Chunk;
use colored::Colorize;
use glam::IVec3;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

const PRELOAD_DISTANCE: usize = 10;
const CHUNK_GENERATOR: fn(IVec3, &AnimatedNoise, f64, &Chunks) -> Chunk = Chunk::from_landscape;

/// # Plan for Mesh Generation
///
/// 1. Look up if the chunk already exists.
///    If yes, look if the mesh exists.
///    If yes, use the mesh.
/// 2. If the chunk doesn't exist, generate an occlusion map and a mesh out of it.
pub struct Server {
    chunks: Arc<Chunks>,
    meshes: HashMap<IVec3, Lazy<Mesh>>,
    was_initiated: bool,
}
impl Server {
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(Chunks::new()),
            meshes: HashMap::new(),
            was_initiated: false,
        }
    }
    pub fn init(&mut self, noise: Arc<AnimatedNoise>, threadpool: &mut Threadpool) {
        for_point_in_square(IVec3::ZERO, PRELOAD_DISTANCE as i32, |chunk_coord| {
            let noise = noise.clone();
            let chunks = self.chunks.clone();

            let (lazy_mesh, s) = Lazy::open();
            self.meshes.insert(chunk_coord, lazy_mesh);

            threadpool
                .add_priority(move || {
                    // let now = Instant::now();
                    let chunk = CHUNK_GENERATOR(chunk_coord, &noise, 0.0, &chunks);
                    chunks.add(chunk_coord, chunk.clone());

                    if chunk.is_empty {
                        _ = s.send(Box::new(Mesh::new()));
                    } else {
                        _ = s.send(Box::new(
                            crate::server::chunk::generate_mesh_without_cam_occ(
                                &chunk.voxels,
                                chunk_coord,
                                chunk.occlusion_map,
                            ),
                        ));
                    }
                    // print_msg(now, chunk_coord)
                })
                .map_or_else(|| (), |task| task());
        });
        self.was_initiated = true;
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
        let mut mesh = Mesh::with_capacity(24_000_000);
        let cam_chunk_pos = cam_pos / 32.0;

        let mut points = every_chunk_in_frustum(
            cam_chunk_pos,
            viewing_dir,
            fov,
            aspect_ratio,
            render_distance,
        );
        points.sort_by(|a, b| a.y.cmp(&b.y).reverse());
        points.iter().for_each(|chunk_coord| {
            if self.meshes.contains_key(&chunk_coord) {
                return;
            }
            let chunk_coord = chunk_coord.clone();
            let noise = noise.clone();
            let chunks = self.chunks.clone();

            let (lazy_mesh, s) = Lazy::open();
            self.meshes.insert(chunk_coord, lazy_mesh);

            threadpool
                .add_priority(move || {
                    let chunk = CHUNK_GENERATOR(chunk_coord, &noise, 0.0, &chunks);
                    chunks.add(chunk_coord, chunk.clone());

                    if chunk.is_empty {
                        _ = s.send(Box::new(Mesh::new()));
                    } else {
                        _ = s.send(Box::new(
                            crate::server::chunk::generate_mesh_without_cam_occ(
                                &chunk.voxels,
                                chunk_coord,
                                chunk.occlusion_map,
                            ),
                        ));
                    }
                })
                .map_or_else(|| (), |task| task());
        });
        points.iter().for_each(|chunk_coord| {
            let lazy_mesh = self.meshes.get_mut(chunk_coord).unwrap();
            if let Some(chunk_mesh) = lazy_mesh.try_get() {
                mesh += chunk_mesh.clone();
            }
        });
        // println!("frame done, size of mesh: {} kB", (mesh.vertices.len() * size_of::<crate::gpu::mesh::Vertex>() + mesh.indices.len() * 4) / 1000);
        mesh
    }
}
#[inline]
fn print_msg(start: Instant, chunk_coord: IVec3) {
    let time = start.elapsed();
    let msg = format!(
        "time it took to build the chunk mesh at {:?}: {:#?}",
        chunk_coord, time
    );
    let msg = if time.as_micros() < 1000 {
        msg.green()
    } else {
        msg.red()
    };
    println!("{}", msg);
}
#[derive(Debug)]
pub struct Chunks(RwLock<HashMap<IVec3, Arc<RwLock<Chunk>>>>);

impl Chunks {
    fn new() -> Self {
        Self(RwLock::new(HashMap::new()))
    }
    pub fn get(&self, pos: IVec3) -> Option<Arc<RwLock<Chunk>>> {
        return self.0.read().unwrap().get(&pos).cloned();
    }
    pub fn contains(&self, pos: IVec3) -> bool {
        return self.0.read().unwrap().contains_key(&pos);
    }
    pub fn add(&self, pos: IVec3, chunk: Chunk) {
        self.0
            .write()
            .unwrap()
            .insert(pos, Arc::new(RwLock::new(chunk)));
    }
    pub fn for_all_neighbors(&self, pos: IVec3) -> std::vec::IntoIter<Arc<RwLock<Chunk>>> {
        let mut chunks: Vec<Arc<RwLock<Chunk>>> = vec![];
        for neighbor in [
            IVec3::new(1, 0, 0),
            IVec3::new(-1, 0, 0),
            IVec3::new(0, 1, 0),
            IVec3::new(0, -1, 0),
            IVec3::new(0, 0, 1),
            IVec3::new(0, 0, -1),
        ]
        .into_iter()
        {
            if let Some(chunk) = self.get(neighbor) {
                chunks.push(chunk)
            }
        }
        chunks.into_iter()
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
