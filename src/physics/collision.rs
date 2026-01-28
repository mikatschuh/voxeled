use glam::{IVec3, Vec3};

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
pub struct AABB {
    pos: Vec3,
    half_extends: Vec3,
}

impl AABB {
    pub fn player(pos: Vec3) -> Self {
        Self {
            pos,
            half_extends: PLAYER_HALF_EXTENTS,
        }
    }

    /// Computes final position
    pub fn compute_sweep(mut self, voxel: &impl Voxel, mut delta: Vec3) -> Vec3 {
        todo!()
    }

    fn corners(&self) -> (IVec3, IVec3) {
        (
            (self.pos - self.half_extends).floor().as_ivec3(),
            (self.pos + self.half_extends).floor().as_ivec3(),
        )
    }
}
