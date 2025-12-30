use crate::gpu::texture_set::Texture;
use crate::server::chunks::{ChunkID, Level};
use crate::server::frustum::{chunk_overlaps, Frustum, LodLevel, MAX_LOD};
use crate::server::job::Job;
use crate::server::world_gen::Generator;
use crate::{gpu::mesh::Mesh, threadpool::Threadpool};
use colored::Colorize;
use crossbeam::sync::ShardedLock;
use glam::IVec3;
use std::sync::Arc;
use std::time::Instant;

pub mod chunks;
pub mod frustum;
// mod sampling;
pub mod job;
pub mod meshing;
#[cfg(test)]
mod test;
pub mod voxel;
#[allow(dead_code)]
pub mod world_gen;

/// # Plan for Mesh Generation
///
/// 1. Look up if the chunk already exists.
///    If yes, look if the mesh exists.
///    If yes, use the mesh.
/// 2. If the chunk doesn't exist, generate an occlusion map and a mesh out of it.
pub struct Server<G: Generator> {
    generator: Arc<ShardedLock<G>>,
    level: Arc<Level>,
}

impl<G: Generator> Server<G> {
    pub fn new(generator: G) -> Self {
        Self {
            generator: Arc::new(ShardedLock::new(generator)),
            level: Arc::new(Level::with_capacity(8)),
        }
    }

    pub fn get_mesh(
        &mut self,
        frustum: Frustum,
        threadpool: &mut Threadpool<G>,
        lod_level: LodLevel,
    ) -> Mesh {
        let mut mesh = Mesh::with_capacity(24_000_000);

        let cam_chunk_pos = (frustum.cam_pos / 32.0).as_ivec3();

        let chunks: Vec<ChunkID> = frustum.chunk_ids().collect();

        chunks.iter().copied().for_each(|chunk_id| {
            if self.mesh_ready(chunk_id) {
                return;
            }

            let generator = self.generator.clone();
            let voxel_grid = self.level.clone();

            threadpool.push(Job::GenerateChunkAndMesh {
                voxel_grid,
                chunk_id,
                generator,
            })
        });

        let chunks = self.select_render_chunks(&chunks);

        let mut chunk_count = 0;

        chunks.into_iter().for_each(|chunk_id| {
            let Some(chunk_mesh) = self.level.chunk_op(chunk_id, |chunk| chunk.mesh.clone()) else {
                return;
            };
            let chunk_pos = chunk_id.total_pos();
            let chunk_size = 1 << chunk_id.lod;
            let chunk_mesh = chunk_mesh.read();

            //mesh += chunk_mesh.clone();

            if cam_chunk_pos.x <= chunk_pos.x + chunk_size {
                mesh.nx.append(&mut chunk_mesh.nx.clone())
            }
            if cam_chunk_pos.x >= chunk_pos.x {
                mesh.px.append(&mut chunk_mesh.px.clone())
            }
            if cam_chunk_pos.y <= chunk_pos.y + chunk_size {
                mesh.ny.append(&mut chunk_mesh.ny.clone())
            }
            if cam_chunk_pos.y >= chunk_pos.y {
                mesh.py.append(&mut chunk_mesh.py.clone())
            }
            if cam_chunk_pos.z <= chunk_pos.z + chunk_size {
                mesh.nz.append(&mut chunk_mesh.nz.clone())
            }
            if cam_chunk_pos.z >= chunk_pos.z {
                mesh.pz.append(&mut chunk_mesh.pz.clone())
            }

            chunk_count += 1;
        });

        println!("count: {chunk_count}");

        mesh
    }

    fn select_render_chunks(&self, chunks: &[ChunkID]) -> Vec<ChunkID> {
        let mut selected: Vec<ChunkID> = Vec::new();

        for desired in chunks.iter().copied() {
            let mut candidate = desired;
            if !self.mesh_ready(candidate) {
                let mut next = candidate;
                let mut found = false;
                while next.lod < MAX_LOD {
                    next = next.parent_lod();
                    if self.mesh_ready(next) {
                        candidate = next;
                        found = true;
                        break;
                    }
                }
                if !found && !self.mesh_ready(candidate) {
                    continue;
                }
            }

            selected.retain(|existing| {
                !(chunk_overlaps(existing, candidate) && existing.lod < candidate.lod)
            });
            if selected.iter().any(|existing| {
                chunk_overlaps(existing, candidate) && existing.lod <= candidate.lod
            }) {
                continue;
            }
            selected.push(candidate);
        }

        selected
    }

    fn mesh_ready(&self, chunk_id: ChunkID) -> bool {
        self.level
            .chunk_op(chunk_id, |chunk| chunk.mesh_state.is_done())
            .is_some_and(|is_done| is_done)
    }

    fn debug_chunk_display(chunks: &Vec<ChunkID>, mesh: &mut Mesh) {
        chunks.iter().for_each(|chunk_id| {
            if chunk_id.lod == 0 {
                return;
            }

            let pos = (chunk_id.pos * 32) << chunk_id.lod;

            let size = chunk_id.lod + 5;
            mesh.add_nx(pos, Texture::Debug, size);
            mesh.add_px(pos, Texture::Debug, size);
            mesh.add_ny(pos, Texture::Debug, size);
            mesh.add_py(pos, Texture::Debug, size);
            mesh.add_nz(pos, Texture::Debug, size);
            mesh.add_pz(pos, Texture::Debug, size);
        });
    }
}

#[allow(dead_code)]
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
