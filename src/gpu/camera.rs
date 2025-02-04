use glam::{Mat4, Quat, Vec3};

pub use super::exotic_cameras::CinematicThirdPersonCamera;

/// Speichert die Eigenschaften einer 3d Kamera.
pub trait Camera3d {
    const SENSITIVITY: f32;
    const SPEED: f32;

    const NEAR_PLANE: f32;
    const FAR_PLANE: f32;

    const FOV: f32;

    fn new(pos: Vec3, gear: f32, yaw: f32, roll: f32) -> Self;

    fn rotate_around_angle(&mut self, angle: Vec3);

    fn move_in_direction(&mut self, vector: Vec3, delta_time: f32);

    fn view_proj(&self, aspect_ratio: f32) -> [[f32; 4]; 4];
}

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pos: Vec3,
    rot: Quat,
    angle: Vec3,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: Vec3::new(0.0, 0.0, 0.0),
            rot: Quat::IDENTITY,
            angle: Vec3::ZERO,
        }
    }
}
impl Camera3d for Camera {
    const SENSITIVITY: f32 = 0.001;
    const SPEED: f32 = 0.0000000017;

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
    fn move_in_direction(&mut self, vector: Vec3, delta_time: f32) {
        self.pos += self.rot * (vector * Self::SPEED * delta_time);
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
