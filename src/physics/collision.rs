use glam::{IVec3, Vec3};

pub trait Voxel {
    fn solid_at(&self, pos: IVec3) -> bool;

    fn check_volume_for_collision(&self, start_corner: IVec3, end_corner: IVec3) -> bool {
        (start_corner.x..end_corner.x)
            .flat_map(move |x| {
                (start_corner.y..end_corner.y)
                    .flat_map(move |y| (start_corner.z..end_corner.z).map(move |z| (x, y, z)))
            })
            .any(|(x, y, z)| self.solid_at(IVec3::new(x, y, z)))
    }
}

pub const PLAYER_HALF_EXTENTS: Vec3 = Vec3::new(0.3, 0.9, 0.3);

#[derive(Clone, Debug, PartialEq)]
pub struct AABB {
    pos: Vec3,
    half_extends: Vec3,
}

impl AABB {
    /// Computes final position
    fn move_through_voxel(mut self, voxel: &impl Voxel, mut delta: Vec3) -> Vec3 {
        let mut start_corner = IVec3::new();
        (self.pos - self.half_extends).floor().as_ivec3();
        let mut end_corner = (self.pos + self.half_extends).floor().as_ivec3();

        while delta != Vec3::ZERO {
            if delta.x > 0. {
                voxel.check_volume_for_collision(IVec3::new(), end_corner)
            }
        }

        end_pos
    }

    fn collides_with_voxel(&self, voxel: &impl Voxel) -> bool {
        let start_corner = (self.pos - self.half_extends).floor().as_ivec3();
        let end_corner = (self.pos + self.half_extends).floor().as_ivec3();

        (start_corner.x..end_corner.x)
            .flat_map(move |x| {
                (start_corner.y..end_corner.y)
                    .flat_map(move |y| (start_corner.z..end_corner.z).map(move |z| (x, y, z)))
            })
            .any(|(x, y, z)| voxel.solid_at(IVec3::new(x, y, z)))
    }
}
