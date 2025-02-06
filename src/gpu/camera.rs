use glam::{Mat4, Quat, Vec3};

// pub use super::exotic_cameras::CinematicThirdPersonCamera;

/// Speichert die Eigenschaften einer 3d Kamera.
pub trait Camera3d: Default {
    const SENSITIVITY: f32;

    const ACC_CHANGE_SENSITIVITY: f32;

    const NEAR_PLANE: f32;
    const FAR_PLANE: f32;

    const FOV: f32;

    fn new(pos: Vec3, gear: f32, yaw: f32, roll: f32) -> Self;

    fn rotate_around_angle(&mut self, angle: Vec3);

    fn update(&mut self, vector: Vec3, delta_time: f32);

    fn update_acc(&mut self, change: f32);

    fn view_proj(&self, aspect_ratio: f32) -> [[f32; 4]; 4];
}

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pos: Vec3,
    rot: Quat,
    angle: Vec3,
    vel: Vec3,
    acc: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Vec3::new(0.0, 0.0, 0.0),
            rot: Quat::IDENTITY,
            angle: Vec3::ZERO,
            vel: Vec3::ZERO,
            acc: 0.0000000005,
        }
    }
}
impl Camera3d for Camera {
    const SENSITIVITY: f32 = 0.001;
    const ACC_CHANGE_SENSITIVITY: f32 = 1.0;

    const NEAR_PLANE: f32 = 0.001;
    const FAR_PLANE: f32 = 1000.0;

    const FOV: f32 = std::f32::consts::FRAC_PI_3;

    fn new(pos: Vec3, gear: f32, yaw: f32, roll: f32) -> Self {
        Self {
            pos,
            rot: Quat::IDENTITY
                * Quat::from_axis_angle(Vec3::Y, gear)
                * Quat::from_axis_angle(Vec3::X, yaw)
                * Quat::from_axis_angle(Vec3::Z, roll),
            angle: Vec3::new(gear, yaw, roll),
            ..Default::default()
        }
    }

    /// Dreht die Kamera um einen Winkel multipliziert mit der Kamera Sensitivität.
    fn rotate_around_angle(&mut self, angle: Vec3) {
        self.angle += angle * Self::SENSITIVITY;

        self.rot = Quat::IDENTITY
            * Quat::from_axis_angle(Vec3::Y, self.angle.x)
            * Quat::from_axis_angle(Vec3::X, self.angle.y)
            * Quat::from_axis_angle(Vec3::Z, self.angle.z);
    }
    /// Bewegt die Kamera in eine Richtung relativ zur Richtung in die die Kamera zeigt.
    fn update(&mut self, vector: Vec3, delta_time: f32) {
        self.vel += self.rot * (vector * self.acc * delta_time);

        self.pos += self.vel;

        self.vel *= Self::FRICTION * delta_time;
    }
    fn update_acc(&mut self, change: f32) {
        let change = -change * Self::ACC_CHANGE_SENSITIVITY;
        self.acc = (self.acc
            * if change >= 0.0 {
                change
            } else {
                1.0 / change.abs()
            })
        .max(0.000000000000000001)
    }
    /// Diese Funktion gibt eine 4*4 Matrix zurück um die Punkte auf den Bildschirm zu projezieren.
    fn view_proj(&self, aspect_ratio: f32) -> [[f32; 4]; 4] {
        let proj = Mat4::perspective_rh(Self::FOV, aspect_ratio, Self::NEAR_PLANE, Self::FAR_PLANE);

        // Erstelle die View-Matrix
        let view = Mat4::from_rotation_translation(self.rot, self.pos).inverse();

        // Kombiniere Projektion und View
        (proj * view).to_cols_array_2d()
    }
}
impl Camera {
    const FRICTION: f32 = 0.0000001;
}
