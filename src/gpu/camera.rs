use glam::{Quat, Vec3};

const SENSITIVITY: f32 = 0.0005;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub pos: Vec3,
    pub rot: Quat,
    yaw: f32,
    pitch: f32,
    gear: f32,
}
unsafe impl bytemuck::Pod for Camera {}
unsafe impl bytemuck::Zeroable for Camera {}
impl Default for Camera {
    fn default() -> Self {
        Camera {
            pos: Vec3::new(0.0, 0.0, 0.0),
            rot: Quat::IDENTITY,
            yaw: 0.0,
            pitch: 0.0,
            gear: 0.0,
        }
    }
}
const SPEED: f32 = 0.17;
impl Camera {
    pub fn rotate(&mut self, yaw: f32, pitch: f32, gear: f32) {
        self.yaw += yaw * SENSITIVITY;
        self.pitch += pitch * SENSITIVITY;
        self.gear += gear * SENSITIVITY;

        self.rot = Quat::IDENTITY
            * Quat::from_axis_angle(Vec3::Y, self.yaw)
            * Quat::from_axis_angle(Vec3::X, self.pitch)
            * Quat::from_axis_angle(Vec3::Z, self.gear);
    }
    pub fn move_in_direction(&mut self, direction: Vec3, delta_time: f32) {
        // self.pos += direction * SPEED * delta_time * self.rot.;
    }
}
