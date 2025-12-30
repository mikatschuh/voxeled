use glam::{IVec3, Vec3};

use crate::server::{world_gen::Generator, Server};

pub const PLAYER_HALF_EXTENTS: Vec3 = Vec3::new(0.3, 0.9, 0.3);
const SWEEP_EPS: f32 = 1.0e-4;
const GROUND_CHECK_DISTANCE: f32 = 0.05;

#[derive(Clone, Copy, Debug)]
pub struct SweepResult {
    pub position: Vec3,
    pub normal: Vec3,
    pub hit: bool,
}

#[derive(Clone, Copy, Debug)]
struct SweepHit {
    time: f32,
    normal: Vec3,
}

pub fn move_player_with_sweep<G: Generator>(
    server: &Server<G>,
    start: Vec3,
    delta: Vec3,
) -> SweepResult {
    move_aabb_with_sweep(server, start, delta, PLAYER_HALF_EXTENTS)
}

pub fn is_on_ground<G: Generator>(server: &Server<G>, pos: Vec3) -> bool {
    // +Y is down in this project, so ground is in the +Y direction.
    let delta = Vec3::new(0.0, GROUND_CHECK_DISTANCE, 0.0);
    sweep_voxels(server, pos, delta, PLAYER_HALF_EXTENTS).is_some_and(|hit| hit.normal.y < -0.5)
}

pub fn move_aabb_with_sweep<G: Generator>(
    server: &Server<G>,
    start: Vec3,
    delta: Vec3,
    half_extents: Vec3,
) -> SweepResult {
    if delta.length_squared() <= SWEEP_EPS * SWEEP_EPS {
        return SweepResult {
            position: start,
            normal: Vec3::ZERO,
            hit: false,
        };
    }

    let mut pos = start;
    let mut remaining = delta;
    let mut normal = Vec3::ZERO;
    let mut hit = false;

    for _ in 0..3 {
        let Some(sweep_hit) = sweep_voxels(server, pos, remaining, half_extents) else {
            pos += remaining;
            break;
        };

        hit = true;
        normal = sweep_hit.normal;

        pos += remaining * sweep_hit.time;

        let remaining_time = 1.0 - sweep_hit.time;
        if remaining_time <= 0.0 {
            break;
        }

        let remaining_delta = remaining * remaining_time;
        pos += sweep_hit.normal * SWEEP_EPS;
        remaining = remaining_delta - sweep_hit.normal * remaining_delta.dot(sweep_hit.normal);
    }

    SweepResult {
        position: pos,
        normal,
        hit,
    }
}

fn sweep_voxels<G: Generator>(
    server: &Server<G>,
    start: Vec3,
    delta: Vec3,
    half_extents: Vec3,
) -> Option<SweepHit> {
    let (min_voxel, max_voxel) = sweep_voxel_bounds(start, delta, half_extents);
    if min_voxel.cmpgt(max_voxel).any() {
        return None;
    }

    let moving_min = start - half_extents;
    let moving_max = start + half_extents;

    let mut best_time = 1.0;
    let mut best_normal = Vec3::ZERO;
    let mut found = false;

    for x in min_voxel.x..=max_voxel.x {
        for y in min_voxel.y..=max_voxel.y {
            for z in min_voxel.z..=max_voxel.z {
                let voxel_pos = IVec3::new(x, y, z);
                if !server.is_solid_physically(voxel_pos) {
                    continue;
                }

                let target_min = Vec3::new(x as f32, y as f32, z as f32);
                let target_max = target_min + Vec3::ONE;

                let Some(hit) = swept_aabb(moving_min, moving_max, delta, target_min, target_max)
                else {
                    continue;
                };

                if hit.time < best_time {
                    best_time = hit.time;
                    best_normal = hit.normal;
                    found = true;
                }
            }
        }
    }

    if found {
        Some(SweepHit {
            time: best_time,
            normal: best_normal,
        })
    } else {
        None
    }
}

fn sweep_voxel_bounds(start: Vec3, delta: Vec3, half_extents: Vec3) -> (IVec3, IVec3) {
    let start_min = start - half_extents;
    let start_max = start + half_extents;
    let end_min = start_min + delta;
    let end_max = start_max + delta;

    let sweep_min = start_min.min(end_min);
    let sweep_max = start_max.max(end_max);

    let min_voxel = sweep_min.floor().as_ivec3();
    let max_voxel = (sweep_max - Vec3::splat(SWEEP_EPS)).floor().as_ivec3();

    (min_voxel, max_voxel)
}

fn swept_aabb(
    moving_min: Vec3,
    moving_max: Vec3,
    delta: Vec3,
    target_min: Vec3,
    target_max: Vec3,
) -> Option<SweepHit> {
    let (tx_entry, tx_exit) = axis_sweep(
        moving_min.x,
        moving_max.x,
        delta.x,
        target_min.x,
        target_max.x,
    )?;
    let (ty_entry, ty_exit) = axis_sweep(
        moving_min.y,
        moving_max.y,
        delta.y,
        target_min.y,
        target_max.y,
    )?;
    let (tz_entry, tz_exit) = axis_sweep(
        moving_min.z,
        moving_max.z,
        delta.z,
        target_min.z,
        target_max.z,
    )?;

    let entry_time = tx_entry.max(ty_entry).max(tz_entry);
    let exit_time = tx_exit.min(ty_exit).min(tz_exit);

    if entry_time > exit_time || exit_time < 0.0 || entry_time > 1.0 {
        return None;
    }

    let normal = if entry_time < 0.0 {
        Vec3::ZERO
    } else if tx_entry >= ty_entry && tx_entry >= tz_entry {
        Vec3::new(if delta.x > 0.0 { -1.0 } else { 1.0 }, 0.0, 0.0)
    } else if ty_entry >= tz_entry {
        Vec3::new(0.0, if delta.y > 0.0 { -1.0 } else { 1.0 }, 0.0)
    } else {
        Vec3::new(0.0, 0.0, if delta.z > 0.0 { -1.0 } else { 1.0 })
    };

    Some(SweepHit {
        time: entry_time.max(0.0),
        normal,
    })
}

fn axis_sweep(
    moving_min: f32,
    moving_max: f32,
    delta: f32,
    target_min: f32,
    target_max: f32,
) -> Option<(f32, f32)> {
    if delta > 0.0 {
        let entry = (target_min - moving_max) / delta;
        let exit = (target_max - moving_min) / delta;
        Some((entry, exit))
    } else if delta < 0.0 {
        let entry = (target_max - moving_min) / delta;
        let exit = (target_min - moving_max) / delta;
        Some((entry, exit))
    } else if moving_max < target_min || moving_min > target_max {
        None
    } else {
        Some((f32::NEG_INFINITY, f32::INFINITY))
    }
}
