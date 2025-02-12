use glam::{Mat4, Quat, Vec3};

use super::camera_controller::SmoothController;

// pub use super::exotic_cameras::CinematicThirdPersonCamera;

/// Speichert die Eigenschaften einer 3d Kamera.
pub trait Camera3d: Default {
    const NEAR_PLANE: f32;
    const FAR_PLANE: f32;

    const FOV: f32;

    fn new(pos: Vec3, dir: Vec3) -> Self;

    fn controller(&mut self) -> &mut SmoothController;

    fn view_proj(&self, aspect_ratio: f32) -> [[f32; 4]; 4];
}

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    con: SmoothController,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            con: SmoothController::default(),
        }
    }
}
impl Camera3d for Camera {
    const NEAR_PLANE: f32 = 0.001;
    const FAR_PLANE: f32 = 1000.0;

    const FOV: f32 = std::f32::consts::FRAC_PI_3;

    fn new(pos: Vec3, dir: Vec3) -> Self {
        Self {
            con: SmoothController::new(pos, dir),
        }
    }
    fn controller(&mut self) -> &mut SmoothController {
        &mut self.con
    }
    /// Diese Funktion gibt eine 4*4 Matrix zurück um die Punkte auf den Bildschirm zu projezieren.
    fn view_proj(&self, aspect_ratio: f32) -> [[f32; 4]; 4] {
        let proj = Mat4::perspective_rh(Self::FOV, aspect_ratio, Self::NEAR_PLANE, Self::FAR_PLANE);

        // Erstelle die View-Matrix
        let view = Mat4::from_rotation_translation(self.con.rot(), self.con.pos()).inverse();

        // Kombiniere Projektion und View
        (proj * view).to_cols_array_2d()
    }
}
