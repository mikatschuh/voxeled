use glam::{Mat4, Vec3};

use super::camera_controller::CameraController;

// pub use super::exotic_cameras::CinematicThirdPersonCamera;

/// Speichert die Eigenschaften einer 3d Kamera.
pub trait Camera3d<C: CameraController> {
    const NEAR_PLANE: f32;
    const FAR_PLANE: f32;

    const FOV: f32;

    fn new(pos: Vec3, dir: Vec3, delta_time: crate::time::DeltaTime) -> Self;

    fn controller(&mut self) -> &mut C;

    fn view_proj(&self, aspect_ratio: f32) -> [[f32; 4]; 4];
}

#[derive(Debug, Copy, Clone)]
pub struct Camera<C> {
    con: C,
}
impl<C: Default> Default for Camera<C> {
    fn default() -> Self {
        Self { con: C::default() }
    }
}
impl<C: CameraController> Camera3d<C> for Camera<C> {
    const NEAR_PLANE: f32 = 0.1;
    const FAR_PLANE: f32 = 10_000.0;

    const FOV: f32 = std::f32::consts::FRAC_PI_2;

    fn new(pos: Vec3, dir: Vec3, delta_time: crate::time::DeltaTime) -> Self {
        Self {
            con: C::new(pos, dir, delta_time),
        }
    }

    fn controller(&mut self) -> &mut C {
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
