use glam::{IVec3, Vec3};

use crate::main;

pub trait Voxel {
    fn solid_at(&self, pos: IVec3) -> bool;

    fn check_volume_for_collision(&self, (start_corner, end_corner): (IVec3, IVec3)) -> bool {
        (start_corner.x..end_corner.x)
            .flat_map(move |x| {
                (start_corner.y..end_corner.y)
                    .flat_map(move |y| (start_corner.z..end_corner.z).map(move |z| (x, y, z)))
            })
            .any(|(x, y, z)| self.solid_at(IVec3::new(x, y, z)))
    }
}

const PLAYER_HALF_EXTENTS: Vec3 = Vec3::new(0.3, 0.9, 0.3);

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
    pub fn compute_sweep(self, voxel: &impl Voxel, mut delta: Vec3) -> Vec3 {
        let Aabb {
            mut pos,
            half_extends,
        } = self;

        let max_element = delta.max_element();
        let dir = delta / max_element;

        let n = max_element.abs().floor();
        let step = delta / n;

        for i in 0..1 {
            pos += step
        }
        pos
    }

    fn corners(&self) -> (IVec3, IVec3) {
        (
            block(self.pos - self.half_extends),
            block(self.pos + self.half_extends),
        )
    }

    fn aabb_step_along_path(self, voxel: &impl Voxel, delta: &mut Vec3) {
        let x_checked = if self.moves_into_new_block(*delta * Vec3::X) {
            let (lower_corner, upper_corner) = match main_axis {
                Vec3::X => (
                    Vec3::new(
                        self.pos.x + self.half_extends.x + 1.,
                        self.pos.y - self.half_extends.y,
                        self.pos.z - self.half_extends.z,
                    ),
                    Vec3::new(
                        self.pos.x + self.half_extends.x + 1.,
                        self.pos.y + self.half_extends.y,
                        self.pos.z + self.half_extends.z,
                    ),
                ),
                Vec3::NEG_X => (
                    Vec3::new(
                        self.pos.x - self.half_extends.x + 1.,
                        self.pos.y - self.half_extends.y,
                        self.pos.z - self.half_extends.z,
                    ),
                    Vec3::new(
                        self.pos.x - self.half_extends.x + 1.,
                        self.pos.y + self.half_extends.y,
                        self.pos.z + self.half_extends.z,
                    ),
                ),
                Vec3::Y => (
                    Vec3::new(
                        self.pos.x - self.half_extends.x,
                        self.pos.y + self.half_extends.y + 1.,
                        self.pos.z - self.half_extends.z,
                    ),
                    Vec3::new(
                        self.pos.x + self.half_extends.x,
                        self.pos.y + self.half_extends.y + 1.,
                        self.pos.z + self.half_extends.z,
                    ),
                ),
                Vec3::NEG_Y => (
                    Vec3::new(
                        self.pos.x - self.half_extends.x,
                        self.pos.y - self.half_extends.y + 1.,
                        self.pos.z - self.half_extends.z,
                    ),
                    Vec3::new(
                        self.pos.x + self.half_extends.x,
                        self.pos.y - self.half_extends.y + 1.,
                        self.pos.z + self.half_extends.z,
                    ),
                ),
                Vec3::Z => (
                    Vec3::new(
                        self.pos.x - self.half_extends.x,
                        self.pos.y - self.half_extends.y,
                        self.pos.z + self.half_extends.z + 1.,
                    ),
                    Vec3::new(
                        self.pos.x + self.half_extends.x,
                        self.pos.y + self.half_extends.y,
                        self.pos.z + self.half_extends.z + 1.,
                    ),
                ),
                Vec3::NEG_Z => (
                    Vec3::new(
                        self.pos.x - self.half_extends.x,
                        self.pos.y - self.half_extends.y,
                        self.pos.z - self.half_extends.z + 1.,
                    ),
                    Vec3::new(
                        self.pos.x + self.half_extends.x,
                        self.pos.y + self.half_extends.y,
                        self.pos.z - self.half_extends.z + 1.,
                    ),
                ),
                _ => panic!("the main axis should match one unit axis!"),
            };
            // check main axis
            voxel.check_volume_for_collision((block(lower_corner), block(upper_corner)));
        };
    }

    fn moves_into_new_block(&self, movement: Vec3) -> bool {
        self.pos.floor() != (self.pos + movement).floor()
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

        assert!(!aabb.moves_into_new_block(Vec3::X * 0.4))
    }
}
