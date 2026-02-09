use glam::{IVec3, Vec3};
use num::Rational32;

use crate::main;

pub trait Voxel {
    fn solid_at(&self, pos: IVec3) -> bool;

    fn check_volume_for_collision(&self, (start_corner, end_corner): (IVec3, IVec3)) -> bool {
        (start_corner.x..=end_corner.x)
            .flat_map(move |x| {
                (start_corner.y..=end_corner.y)
                    .flat_map(move |y| (start_corner.z..=end_corner.z).map(move |z| (x, y, z)))
            })
            .any(|(x, y, z)| self.solid_at(IVec3::new(x, y, z)))
    }
}

const PLAYER_HALF_EXTENTS: Vec3 = Vec3::new(3., 0.9, 3.);

const EPSILON: f32 = 0.000001;

#[derive(Clone, Debug, PartialEq)]
pub struct Aabb {
    pos: Vec3,
    half_extends: Vec3,
}

impl Aabb {
    pub fn player(pos: Vec3) -> Self {
        Self {
            pos,
            half_extends: PLAYER_HALF_EXTENTS,
        }
    }

    pub fn new(pos: Vec3, half_extends: Vec3) -> Self {
        Self { pos, half_extends }
    }

    /// Computes final position
    pub fn compute_sweep(mut self, voxel: &impl Voxel, mut delta: Vec3) -> Vec3 {
        loop {
            let max_element = delta.max_element();
            if max_element > 1. {
                let step = delta / max_element;
                self.aabb_step_along_path(voxel, step);
                delta -= step;
            } else {
                self.aabb_step_along_path(voxel, delta);

                return self.pos;
            }
        }
    }

    fn corners_blocked(&self) -> (IVec3, IVec3) {
        (
            block(self.pos - self.half_extends),
            block(self.pos + self.half_extends),
        )
    }

    fn corners(&self) -> (Vec3, Vec3) {
        (self.pos - self.half_extends, self.pos + self.half_extends)
    }

    fn aabb_step_along_path(&mut self, voxel: &impl Voxel, step: Vec3) {
        let step_x_positive = step.x.is_sign_positive();
        let step_y_positive = step.y.is_sign_positive();
        let step_z_positive = step.z.is_sign_positive();

        let (neg_corner, pos_corner) = self.corners();

        let mut x_corners = if self.moves_into_new_block_on_x(step.x) {
            let x = if step_x_positive {
                pos_corner.x + 1.
            } else {
                neg_corner.x - 1.
            };

            Some((
                Vec3::new(x, neg_corner.y, neg_corner.z),
                Vec3::new(x, pos_corner.y, pos_corner.z),
            ))
        } else {
            None
        };

        let mut y_corners = if self.moves_into_new_block_on_y(step.y) {
            let y = if step_y_positive {
                pos_corner.y + 1.
            } else {
                neg_corner.y - 1.
            };

            Some((
                Vec3::new(neg_corner.x, y, neg_corner.z),
                Vec3::new(pos_corner.x, y, pos_corner.z),
            ))
        } else {
            None
        };

        let mut z_corners = if self.moves_into_new_block_on_z(step.z) {
            let z = if step_z_positive {
                pos_corner.z + 1.
            } else {
                neg_corner.z - 1.
            };

            Some((
                Vec3::new(neg_corner.x, neg_corner.y, z),
                Vec3::new(pos_corner.x, pos_corner.y, z),
            ))
        } else {
            None
        };

        if !x_corners
            .map(|x_corners| {
                voxel.check_volume_for_collision((block(x_corners.0), block(x_corners.1)))
            })
            .is_some_and(|coll| coll)
        {
            self.pos.x += step.x
        } else if step.x.is_sign_positive() {
            let frame = pos_corner.x;
            self.pos.x += frame.ceil() - frame;
        } else {
            let frame = neg_corner.x;
            self.pos.x += frame.floor() - frame;
        }

        if !y_corners
            .map(|y_corners| {
                voxel.check_volume_for_collision((block(y_corners.0), block(y_corners.1)))
            })
            .is_some_and(|coll| coll)
        {
            self.pos.y += step.y
        } else if step.y.is_sign_positive() {
            let frame = pos_corner.y;
            self.pos.y += frame.ceil() - frame;
        } else {
            let frame = neg_corner.y;
            self.pos.y += frame.floor() - frame;
        }

        if !z_corners
            .map(|z_corners| {
                voxel.check_volume_for_collision((block(z_corners.0), block(z_corners.1)))
            })
            .is_some_and(|coll| coll)
        {
            self.pos.z += step.z
        } else if step.z.is_sign_positive() {
            let frame = pos_corner.z;
            self.pos.z += frame.ceil() - frame;
        } else {
            let frame = neg_corner.z;
            self.pos.z += frame.floor() - frame;
        }

        // x_corners.as_mut().map(|x_corners| if y_corners.is_some() &&step.y.is_sign_positive() {x_corners.1.y+1})}
    }

    fn moves_into_new_block_on_x(&self, movement: f32) -> bool {
        if movement.is_sign_positive() {
            (self.pos.x + self.half_extends.x).floor()
                != (self.pos.x + self.half_extends.x + movement).floor()
        } else {
            (self.pos.x - self.half_extends.x).floor()
                != (self.pos.x - self.half_extends.x + movement).floor()
        }
    }

    fn moves_into_new_block_on_y(&self, movement: f32) -> bool {
        if movement.is_sign_positive() {
            (self.pos.y + self.half_extends.y).floor()
                != (self.pos.y + self.half_extends.y + movement).floor()
        } else {
            (self.pos.y - self.half_extends.y).floor()
                != (self.pos.y - self.half_extends.y + movement).floor()
        }
    }

    fn moves_into_new_block_on_z(&self, movement: f32) -> bool {
        if movement.is_sign_positive() {
            (self.pos.z + self.half_extends.z).floor()
                != (self.pos.z + self.half_extends.z + movement).floor()
        } else {
            (self.pos.z - self.half_extends.z).floor()
                != (self.pos.z - self.half_extends.z + movement).floor()
        }
    }
}

fn block(v: Vec3) -> IVec3 {
    v.floor().as_ivec3()
}

#[cfg(test)]
mod test {
    use glam::Vec3;

    use crate::physics::Aabb;

    #[test]
    fn test_helper() {
        let aabb = Aabb::player(Vec3::ZERO);

        assert!(aabb.moves_into_new_block_on_x(0.8))
    }
}
