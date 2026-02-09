mod cam_controller;
mod collision;
#[cfg(test)]
mod test;
mod time;
mod verlet;

pub use cam_controller::CamController;

use glam::IVec3;
use glam::Vec3;
pub use time::DeltaTime;
pub use time::DeltaTimeMeter;

pub use verlet::Body;
pub use verlet::TCBody;

pub use collision::Aabb;
pub use collision::Voxel;

pub fn block(v: Vec3) -> IVec3 {
    v.floor().as_ivec3()
}
