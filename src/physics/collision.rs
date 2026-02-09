use glam::{IVec3, Vec3};
use num::Rational32;

use crate::{main, physics::block};

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

const PLAYER_HALF_EXTENTS: Vec3 = Vec3::new(0.3, 0.9, 0.3);

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
            let max_element = delta.abs().max_element();
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
        let (neg_corner, pos_corner) = self.corners();

        let borders = Vec3::new(
            if step.x.is_sign_positive() {
                pos_corner.x
            } else {
                neg_corner.x
            },
            if step.y.is_sign_positive() {
                pos_corner.y
            } else {
                neg_corner.y
            },
            if step.z.is_sign_positive() {
                pos_corner.z
            } else {
                neg_corner.z
            },
        );

        let mut x_corners = {
            let x = borders.x + step.x;

            if x.floor() != borders.x.floor() {
                Some((
                    Vec3::new(x, neg_corner.y, neg_corner.z),
                    Vec3::new(x, pos_corner.y, pos_corner.z),
                ))
            } else {
                None
            }
        };

        let mut y_corners = {
            let y = borders.y + step.y;

            if y.floor() != borders.y.floor() {
                Some((
                    Vec3::new(neg_corner.x, y, neg_corner.z),
                    Vec3::new(pos_corner.x, y, pos_corner.z),
                ))
            } else {
                None
            }
        };

        let mut z_corners = {
            let z = borders.z + step.z;

            if z.floor() != borders.z.floor() {
                Some((
                    Vec3::new(neg_corner.x, neg_corner.y, z),
                    Vec3::new(pos_corner.x, pos_corner.y, z),
                ))
            } else {
                None
            }
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
            let leading = self.pos.x + self.half_extends.x;
            (leading - EPSILON).floor() != (leading + movement - EPSILON).floor()
        } else if movement.is_sign_negative() {
            let trailing = self.pos.x - self.half_extends.x;
            (trailing + EPSILON).floor() != (trailing + movement + EPSILON).floor()
        } else {
            false
        }
    }

    fn moves_into_new_block_on_y(&self, movement: f32) -> bool {
        if movement.is_sign_positive() {
            let leading = self.pos.y + self.half_extends.y;
            (leading - EPSILON).floor() != (leading + movement - EPSILON).floor()
        } else if movement.is_sign_negative() {
            let trailing = self.pos.y - self.half_extends.y;
            (trailing + EPSILON).floor() != (trailing + movement + EPSILON).floor()
        } else {
            false
        }
    }

    fn moves_into_new_block_on_z(&self, movement: f32) -> bool {
        if movement.is_sign_positive() {
            let leading = self.pos.z + self.half_extends.z;
            (leading - EPSILON).floor() != (leading + movement - EPSILON).floor()
        } else if movement.is_sign_negative() {
            let trailing = self.pos.z - self.half_extends.z;
            (trailing + EPSILON).floor() != (trailing + movement + EPSILON).floor()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use glam::{IVec3, Vec3};

    use crate::physics::Aabb;
    use crate::physics::Voxel;

    struct SingleSolid(IVec3);

    impl Voxel for SingleSolid {
        fn solid_at(&self, pos: IVec3) -> bool {
            pos == self.0
        }
    }

    #[test]
    fn test_helper() {
        let aabb = Aabb::player(Vec3::ZERO);

        assert!(aabb.moves_into_new_block_on_x(0.8))
    }

    #[test]
    fn test_touching_boundary_blocks_positive_step() {
        let voxel = SingleSolid(IVec3::new(1, 0, 0));
        let aabb = Aabb::player(Vec3::new(0.7, 0.0, 0.0)); // max face at x=1.0
        let result = aabb.compute_sweep(&voxel, Vec3::new(0.05, 0.0, 0.0));

        assert!((result.x - 0.7).abs() < 1e-6);
    }

    #[test]
    fn test_step_before_boundary_moves() {
        let voxel = SingleSolid(IVec3::new(1, 0, 0));
        let aabb = Aabb::player(Vec3::new(0.69, 0.0, 0.0)); // max face at x=0.99
        let result = aabb.compute_sweep(&voxel, Vec3::new(0.005, 0.0, 0.0));

        assert!((result.x - 0.695).abs() < 1e-6);
    }

    #[test]
    fn test_touching_boundary_blocks_negative_step() {
        let voxel = SingleSolid(IVec3::new(-1, 0, 0));
        let aabb = Aabb::player(Vec3::new(0.3, 0.0, 0.0)); // min face at x=0.0
        let result = aabb.compute_sweep(&voxel, Vec3::new(-0.05, 0.0, 0.0));

        assert!((result.x - 0.3).abs() < 1e-6);
    }
}
