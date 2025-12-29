use glam::{IVec3, Vec3};

use std::vec::IntoIter;

use crate::{server::chunks::ChunkID, FULL_DETAL_DISTANCE};

pub type LodLevel = u16;

pub const MAX_LOD: LodLevel = 8;

#[allow(unused)]
pub fn cube(edges: i32, lod_level: LodLevel) -> IntoIter<ChunkID> {
    let mut chunk_ids = vec![];
    for x in 0..edges >> lod_level {
        for y in 0..edges >> lod_level {
            for z in 0..edges >> lod_level {
                chunk_ids.push(ChunkID::new(lod_level, IVec3::new(x, y, z)))
            }
        }
    }
    chunk_ids.into_iter()
}

pub struct Frustum {
    pub cam_pos: Vec3,
    pub direction: Vec3,

    pub fov: f32,
    pub aspect_ratio: f32,

    pub render_distance: f32,
}

impl Frustum {
    pub fn chunk_ids(self) -> IntoIter<ChunkID> {
        let cam_chunk_pos = self.cam_pos / 32.0;

        let chunks = every_chunk_in_frustum(
            cam_chunk_pos,
            self.direction,
            self.fov,
            self.aspect_ratio,
            self.render_distance,
        );
        let mut chunk_ids: Vec<ChunkID> = Vec::new();

        for chunk_pos in chunks {
            let lod = lod_level_at(cam_chunk_pos, chunk_pos.as_vec3());
            let lod_shift = lod as i32;
            let lod_pos = IVec3::new(
                chunk_pos.x >> lod_shift,
                chunk_pos.y >> lod_shift,
                chunk_pos.z >> lod_shift,
            );

            let candidate = ChunkID::new(lod, lod_pos);

            if chunk_ids.iter().any(|existing| {
                chunk_overlaps(existing, candidate) && existing.lod >= candidate.lod
            }) {
                continue;
            }

            chunk_ids.retain(|existing| {
                !(chunk_overlaps(existing, candidate) && existing.lod > candidate.lod)
            });

            chunk_ids.push(candidate);
        }
        chunk_ids.sort_by(|a, b| {
            (a.pos << a.lod)
                .as_vec3()
                .distance(cam_chunk_pos)
                .total_cmp(&(b.pos << b.lod).as_vec3().distance(cam_chunk_pos))
        });
        chunk_ids.into_iter()
    }
}

fn lod_level_at(cam_chunk_pos: Vec3, chunk_coord: Vec3) -> LodLevel {
    let dst = cam_chunk_pos.distance(chunk_coord);
    match dst {
        dst if dst <= FULL_DETAL_DISTANCE => 0,
        dst if dst <= FULL_DETAL_DISTANCE * 2. => 1,
        dst if dst <= FULL_DETAL_DISTANCE * 4. => 2,
        dst if dst <= FULL_DETAL_DISTANCE * 8. => 3,
        dst if dst <= FULL_DETAL_DISTANCE * 16. => 4,
        dst if dst <= FULL_DETAL_DISTANCE * 32. => 5,
        dst if dst <= FULL_DETAL_DISTANCE * 64. => 6,
        dst if dst <= FULL_DETAL_DISTANCE * 128. => 7,
        _ => MAX_LOD,
    }
}

pub fn chunk_overlaps(a: &ChunkID, b: ChunkID) -> bool {
    if a.lod == b.lod {
        return a.pos == b.pos;
    }

    if a.lod > b.lod {
        let shift = (a.lod - b.lod) as i32;
        return (b.pos >> shift) == a.pos;
    }

    let shift = (b.lod - a.lod) as i32;
    (a.pos >> shift) == b.pos
}

fn every_chunk_in_frustum(
    position: Vec3,
    direction: Vec3,
    fov: f32,
    aspect_ratio: f32,
    render_distance: f32,
) -> Vec<IVec3> {
    let mut points = Vec::new();

    let forward = (-direction).normalize();
    let right = if forward.y.abs() > 0.999 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        forward.cross(Vec3::Y).normalize()
    };
    let up = right.cross(forward).normalize();

    let tan_half_fov = (fov / 2.0).tan();
    let max_distance = render_distance.max(0.0);
    let bounds = max_distance.ceil() as i32 + 1;
    let chunk_pad = 0.5;

    let min = IVec3::new(
        (position.x.floor() as i32) - bounds,
        (position.y.floor() as i32) - bounds,
        (position.z.floor() as i32) - bounds,
    );
    let max = IVec3::new(
        (position.x.ceil() as i32) + bounds,
        (position.y.ceil() as i32) + bounds,
        (position.z.ceil() as i32) + bounds,
    );

    for z in min.z..=max.z {
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let delta = Vec3::new(x as f32, y as f32, z as f32) - position;

                let view_x = delta.dot(right);
                let view_y = delta.dot(up);
                let view_z = delta.dot(forward);

                if view_z < -chunk_pad || view_z > max_distance + chunk_pad {
                    continue;
                }

                let frustum_half_height = view_z * tan_half_fov + chunk_pad;
                let frustum_half_width = frustum_half_height * aspect_ratio + chunk_pad;

                if view_x.abs() > frustum_half_width || view_y.abs() > frustum_half_height {
                    continue;
                }

                points.push(IVec3::new(x, y, z));
            }
        }
    }

    points
}
