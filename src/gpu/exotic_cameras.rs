use super::camera::Camera3d;

use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct CinematicThirdPersonCamera {
    pos: Vec3,
    rot: Quat,
    angle: Vec3,
    rot_vel: Vec3,
    acc: f32,
}
impl Default for CinematicThirdPersonCamera {
    fn default() -> Self {
        Self {
            pos: Vec3::new(0.0, 0.0, 0.0),
            rot: Quat::IDENTITY,
            angle: Vec3::ZERO,
            rot_vel: Vec3::ZERO,
            acc: 0.0000000017,
        }
    }
}
impl Camera3d for CinematicThirdPersonCamera {
    const SENSITIVITY: f32 = 0.001;
    const ACC_CHANGE_SENSITIVITY: f32 = 0.0001;

    const NEAR_PLANE: f32 = 0.001;
    const FAR_PLANE: f32 = 1000.0;

    const FOV: f32 = std::f32::consts::FRAC_PI_3;

    fn new(pos: Vec3, yaw: f32, pitch: f32, gear: f32) -> Self {
        Self {
            pos,
            rot: Quat::IDENTITY
                * Quat::from_axis_angle(Vec3::Y, yaw)
                * Quat::from_axis_angle(Vec3::X, pitch)
                * Quat::from_axis_angle(Vec3::Z, gear),
            angle: Vec3::new(yaw, pitch, gear),
            rot_vel: Vec3::ZERO,
            ..Default::default()
        }
    }

    /// Dreht die Kamera um einen Winkel multipliziert mit der Kamera Sensitivität.
    fn rotate_around_angle(&mut self, angle: Vec3) {
        self.rot_vel += angle * Self::SENSITIVITY;
    }
    /// Bewegt die Kamera in eine Richtung relativ zur Richtung in die die Kamera zeigt.
    fn update(&mut self, vector: Vec3, delta_time: f32) {
        self.angle += self.rot_vel * delta_time;

        self.rot = Quat::IDENTITY
            * Quat::from_axis_angle(Vec3::Y, self.angle.x)
            * Quat::from_axis_angle(Vec3::X, self.angle.y)
            * Quat::from_axis_angle(Vec3::Z, self.angle.z);

        self.rot_vel *= Self::FRICTION * delta_time;

        self.pos += self.rot * (vector * self.acc * delta_time);
    }
    fn update_acc(&mut self, change: f32) {
        self.acc += change * Self::ACC_CHANGE_SENSITIVITY
    }
    /// Diese Funktion gibt eine 4*4 Matrix zurück um die Punkte auf den Bildschirm zu projezieren.
    fn view_proj(&self, aspect_ratio: f32) -> [[f32; 4]; 4] {
        let x = self.rot.x;
        let y = self.rot.y;
        let z = self.rot.z;
        let w = self.rot.w;
        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;

        // View-Matrix mit invertierter Rotation für die korrekte Kamera-Orientierung
        let view = Mat4::from_cols_array_2d(&[
            [1.0 - 2.0 * (yy + zz), 2.0 * (xy + wz), 2.0 * (xz - wy), 0.0],
            [2.0 * (xy - wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz + wx), 0.0],
            [2.0 * (xz + wy), 2.0 * (yz - wx), 1.0 - 2.0 * (xx + yy), 0.0],
            [
                self.pos.x - Self::ORBIT_DISTANCE * (2.0 * (xz - wy)),
                self.pos.y - Self::ORBIT_DISTANCE * (2.0 * (yz + wx)),
                self.pos.z - Self::ORBIT_DISTANCE * (1.0 - 2.0 * (xx + yy)),
                1.0,
            ],
        ]);

        let proj = Mat4::perspective_rh(Self::FOV, aspect_ratio, Self::NEAR_PLANE, Self::FAR_PLANE);

        (proj * view).to_cols_array_2d()
    }
}
impl CinematicThirdPersonCamera {
    /// Die Distanz von dem Mittelpunkt zur Kamera
    const ORBIT_DISTANCE: f32 = 2.0;
    /// Eine Konstante mit der die Drehgeschwindigkeit multipliziert wird in jedem Frame.
    const FRICTION: f32 = 0.95;
}
