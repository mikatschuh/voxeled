use glam::{Mat4, Vec3};

use super::camera_controller::SmoothController;

// pub use super::exotic_cameras::CinematicThirdPersonCamera;

#[derive(Debug, Clone)]
pub struct Camera {
    con: SmoothController,
}

impl Camera {
    pub const NEAR_PLANE: f32 = 0.1;
    pub const FAR_PLANE: f32 = 10_000.0;

    pub const FOV: f32 = std::f32::consts::FRAC_PI_2;

    pub fn new(pos: Vec3, dir: Vec3, delta_time: crate::time::DeltaTime) -> Self {
        Self {
            con: SmoothController::new(pos, dir, delta_time),
        }
    }

    pub fn controller(&mut self) -> &mut SmoothController {
        &mut self.con
    }

    /// Diese Funktion gibt eine 4*4 Matrix zurück um die Punkte auf den Bildschirm zu projezieren.
    pub fn view_proj(&self, aspect_ratio: f32) -> [[f32; 4]; 4] {
        let proj = Mat4::perspective_rh(Self::FOV, aspect_ratio, Self::NEAR_PLANE, Self::FAR_PLANE);

        // Erstelle die View-Matrix
        let view = Mat4::from_rotation_translation(self.con.rot(), self.con.pos()).inverse();

        // Kombiniere Projektion und View
        (proj * view).to_cols_array_2d()
    }
}
