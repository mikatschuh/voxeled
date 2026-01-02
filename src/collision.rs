use glam::{IVec3, Vec3};

use crate::server::{Server, world_gen::Generator};

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
    let moving_min = start - half_extents;
    let moving_max = start + half_extents;

    let (mut min_voxel, mut max_voxel) = sweep_voxel_bounds(start, Vec3::ZERO, half_extents);
    if min_voxel.cmpgt(max_voxel).any() {
        return None;
    }

    let mut best_time = 1.0;
    let mut best_normal = Vec3::ZERO;

    for x in min_voxel.x..=max_voxel.x {
        for y in min_voxel.y..=max_voxel.y {
            for z in min_voxel.z..=max_voxel.z {
                let voxel_pos = IVec3::new(x, y, z);
                if !server.is_solid_physically(voxel_pos) {
                    continue;
                }

                let target_min = Vec3::new(x as f32, y as f32, z as f32);
                let target_max = target_min + Vec3::ONE;

                if let Some((normal, depth)) =
                    overlap_info(moving_min, moving_max, target_min, target_max)
                {
                    best_normal = normal;
                    best_time = 0.0;
                    let _ = depth;
                    return Some(SweepHit {
                        time: best_time,
                        normal: best_normal,
                    });
                }
            }
        }
    }

    if delta.length_squared() <= SWEEP_EPS * SWEEP_EPS {
        return None;
    }

    let mut best_found = false;
    let mut current_t = 0.0;

    let (step_x, mut lead_t_max_x, mut lead_t_delta_x, mut trail_t_max_x) =
        sweep_axis_times(moving_min.x, moving_max.x, delta.x);
    let (step_y, mut lead_t_max_y, mut lead_t_delta_y, mut trail_t_max_y) =
        sweep_axis_times(moving_min.y, moving_max.y, delta.y);
    let (step_z, mut lead_t_max_z, mut lead_t_delta_z, mut trail_t_max_z) =
        sweep_axis_times(moving_min.z, moving_max.z, delta.z);

    loop {
        let next_t = lead_t_max_x.min(lead_t_max_y).min(lead_t_max_z);
        if next_t > 1.0 || next_t > best_time {
            break;
        }

        let step_x_now = step_x != 0 && (lead_t_max_x - next_t).abs() <= SWEEP_EPS;
        let step_y_now = step_y != 0 && (lead_t_max_y - next_t).abs() <= SWEEP_EPS;
        let step_z_now = step_z != 0 && (lead_t_max_z - next_t).abs() <= SWEEP_EPS;

        current_t = next_t;

        if step_x_now {
            if step_x > 0 {
                let new_x = max_voxel.x + 1;
                for y in min_voxel.y..=max_voxel.y {
                    for z in min_voxel.z..=max_voxel.z {
                        let voxel_pos = IVec3::new(new_x, y, z);
                        if !server.is_solid_physically(voxel_pos) {
                            continue;
                        }

                        let target_min = Vec3::new(new_x as f32, y as f32, z as f32);
                        let target_max = target_min + Vec3::ONE;
                        if let Some(hit) =
                            swept_aabb(moving_min, moving_max, delta, target_min, target_max)
                        {
                            if hit.time < best_time {
                                best_time = hit.time;
                                best_normal = hit.normal;
                                best_found = true;
                            }
                        }
                    }
                }
                max_voxel.x += 1;
            } else {
                let new_x = min_voxel.x - 1;
                for y in min_voxel.y..=max_voxel.y {
                    for z in min_voxel.z..=max_voxel.z {
                        let voxel_pos = IVec3::new(new_x, y, z);
                        if !server.is_solid_physically(voxel_pos) {
                            continue;
                        }

                        let target_min = Vec3::new(new_x as f32, y as f32, z as f32);
                        let target_max = target_min + Vec3::ONE;
                        if let Some(hit) =
                            swept_aabb(moving_min, moving_max, delta, target_min, target_max)
                        {
                            if hit.time < best_time {
                                best_time = hit.time;
                                best_normal = hit.normal;
                                best_found = true;
                            }
                        }
                    }
                }
                min_voxel.x -= 1;
            }
            lead_t_max_x += lead_t_delta_x;
        }

        if step_y_now {
            if step_y > 0 {
                let new_y = max_voxel.y + 1;
                for x in min_voxel.x..=max_voxel.x {
                    for z in min_voxel.z..=max_voxel.z {
                        let voxel_pos = IVec3::new(x, new_y, z);
                        if !server.is_solid_physically(voxel_pos) {
                            continue;
                        }

                        let target_min = Vec3::new(x as f32, new_y as f32, z as f32);
                        let target_max = target_min + Vec3::ONE;
                        if let Some(hit) =
                            swept_aabb(moving_min, moving_max, delta, target_min, target_max)
                        {
                            if hit.time < best_time {
                                best_time = hit.time;
                                best_normal = hit.normal;
                                best_found = true;
                            }
                        }
                    }
                }
                max_voxel.y += 1;
            } else {
                let new_y = min_voxel.y - 1;
                for x in min_voxel.x..=max_voxel.x {
                    for z in min_voxel.z..=max_voxel.z {
                        let voxel_pos = IVec3::new(x, new_y, z);
                        if !server.is_solid_physically(voxel_pos) {
                            continue;
                        }

                        let target_min = Vec3::new(x as f32, new_y as f32, z as f32);
                        let target_max = target_min + Vec3::ONE;
                        if let Some(hit) =
                            swept_aabb(moving_min, moving_max, delta, target_min, target_max)
                        {
                            if hit.time < best_time {
                                best_time = hit.time;
                                best_normal = hit.normal;
                                best_found = true;
                            }
                        }
                    }
                }
                min_voxel.y -= 1;
            }
            lead_t_max_y += lead_t_delta_y;
        }

        if step_z_now {
            if step_z > 0 {
                let new_z = max_voxel.z + 1;
                for x in min_voxel.x..=max_voxel.x {
                    for y in min_voxel.y..=max_voxel.y {
                        let voxel_pos = IVec3::new(x, y, new_z);
                        if !server.is_solid_physically(voxel_pos) {
                            continue;
                        }

                        let target_min = Vec3::new(x as f32, y as f32, new_z as f32);
                        let target_max = target_min + Vec3::ONE;
                        if let Some(hit) =
                            swept_aabb(moving_min, moving_max, delta, target_min, target_max)
                        {
                            if hit.time < best_time {
                                best_time = hit.time;
                                best_normal = hit.normal;
                                best_found = true;
                            }
                        }
                    }
                }
                max_voxel.z += 1;
            } else {
                let new_z = min_voxel.z - 1;
                for x in min_voxel.x..=max_voxel.x {
                    for y in min_voxel.y..=max_voxel.y {
                        let voxel_pos = IVec3::new(x, y, new_z);
                        if !server.is_solid_physically(voxel_pos) {
                            continue;
                        }

                        let target_min = Vec3::new(x as f32, y as f32, new_z as f32);
                        let target_max = target_min + Vec3::ONE;
                        if let Some(hit) =
                            swept_aabb(moving_min, moving_max, delta, target_min, target_max)
                        {
                            if hit.time < best_time {
                                best_time = hit.time;
                                best_normal = hit.normal;
                                best_found = true;
                            }
                        }
                    }
                }
                min_voxel.z -= 1;
            }
            lead_t_max_z += lead_t_delta_z;
        }

        while step_x != 0 && trail_t_max_x <= current_t {
            if step_x > 0 {
                min_voxel.x += 1;
            } else {
                max_voxel.x -= 1;
            }
            trail_t_max_x += lead_t_delta_x;
        }
        while step_y != 0 && trail_t_max_y <= current_t {
            if step_y > 0 {
                min_voxel.y += 1;
            } else {
                max_voxel.y -= 1;
            }
            trail_t_max_y += lead_t_delta_y;
        }
        while step_z != 0 && trail_t_max_z <= current_t {
            if step_z > 0 {
                min_voxel.z += 1;
            } else {
                max_voxel.z -= 1;
            }
            trail_t_max_z += lead_t_delta_z;
        }
    }

    if best_found {
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
    if let Some((normal, _depth)) = overlap_info(moving_min, moving_max, target_min, target_max) {
        return Some(SweepHit { time: 0.0, normal });
    }

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

fn overlap_info(
    moving_min: Vec3,
    moving_max: Vec3,
    target_min: Vec3,
    target_max: Vec3,
) -> Option<(Vec3, f32)> {
    if moving_max.x <= target_min.x
        || moving_min.x >= target_max.x
        || moving_max.y <= target_min.y
        || moving_min.y >= target_max.y
        || moving_max.z <= target_min.z
        || moving_min.z >= target_max.z
    {
        return None;
    }

    let pen_x_neg = target_max.x - moving_min.x;
    let pen_x_pos = moving_max.x - target_min.x;
    let (pen_x, normal_x) = if pen_x_neg < pen_x_pos {
        (pen_x_neg, Vec3::new(-1.0, 0.0, 0.0))
    } else {
        (pen_x_pos, Vec3::new(1.0, 0.0, 0.0))
    };

    let pen_y_neg = target_max.y - moving_min.y;
    let pen_y_pos = moving_max.y - target_min.y;
    let (pen_y, normal_y) = if pen_y_neg < pen_y_pos {
        (pen_y_neg, Vec3::new(0.0, -1.0, 0.0))
    } else {
        (pen_y_pos, Vec3::new(0.0, 1.0, 0.0))
    };

    let pen_z_neg = target_max.z - moving_min.z;
    let pen_z_pos = moving_max.z - target_min.z;
    let (pen_z, normal_z) = if pen_z_neg < pen_z_pos {
        (pen_z_neg, Vec3::new(0.0, 0.0, -1.0))
    } else {
        (pen_z_pos, Vec3::new(0.0, 0.0, 1.0))
    };

    if pen_x <= pen_y && pen_x <= pen_z {
        Some((normal_x, pen_x))
    } else if pen_y <= pen_z {
        Some((normal_y, pen_y))
    } else {
        Some((normal_z, pen_z))
    }
}

fn sweep_axis_times(moving_min: f32, moving_max: f32, delta: f32) -> (i32, f32, f32, f32) {
    if delta > 0.0 {
        let lead = moving_max;
        let trail = moving_min;
        let next_lead = lead.floor() + 1.0;
        let next_trail = trail.floor() + 1.0;
        let t_max_lead = if (lead - lead.round()).abs() <= SWEEP_EPS {
            0.0
        } else {
            (next_lead - lead) / delta
        };
        let t_max_trail = (next_trail - trail) / delta;
        let t_delta = 1.0 / delta;
        (1, t_max_lead, t_delta, t_max_trail)
    } else if delta < 0.0 {
        let lead = moving_min;
        let trail = moving_max;
        let next_lead = lead.ceil() - 1.0;
        let next_trail = trail.ceil() - 1.0;
        let t_max_lead = if (lead - lead.round()).abs() <= SWEEP_EPS {
            0.0
        } else {
            (next_lead - lead) / delta
        };
        let t_max_trail = (next_trail - trail) / delta;
        let t_delta = -1.0 / delta;
        (-1, t_max_lead, t_delta, t_max_trail)
    } else {
        (0, f32::INFINITY, f32::INFINITY, f32::INFINITY)
    }
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
