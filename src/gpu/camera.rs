use glam::{Mat4, Quat, Vec3};

use crate::time::DeltaTime;

// pub use super::exotic_cameras::CinematicThirdPersonCamera;

#[derive(Clone, Debug)]
pub struct Camera {
    free_cam: bool,

    acc: f32,

    vel: Vec3,
    pos: Vec3,

    rot: Quat,
    angle: Vec3,

    delta_time: crate::time::DeltaTime,
}

impl Camera {
    const FRICTION: f32 = 4.0;
    const STANDART_ACC: f32 = 400.0;
    const MAX_SPEED: f32 = 100.0;
    const ACC_CHANGE_SENSITIVITY: f32 = 3.0;
    const SENSITIVITY: f32 = 0.001;
    const ROLL_SENSITIVITY: f32 = 5.0;

    pub const NEAR_PLANE: f32 = 0.1;
    pub const FAR_PLANE: f32 = 10_000.0;

    pub const FOV: f32 = std::f32::consts::FRAC_PI_2;

    pub fn new(pos: Vec3, dir: Vec3, free_cam: bool, delta_time: DeltaTime) -> Self {
        let forward = if dir.length_squared() > 0.0 {
            dir.normalize()
        } else {
            -Vec3::Z
        };
        let rot = Quat::from_rotation_arc(Vec3::Z, forward);

        Self {
            pos,
            rot,
            angle: Vec3::from(rot.to_euler(glam::EulerRot::YXZ)),
            vel: Vec3::ZERO,
            acc: Self::STANDART_ACC,
            free_cam,
            delta_time,
        }
    }

    /// Dreht die Kamera um einen Winkel multipliziert mit der Kamera Sensitivität.
    pub fn rotate_around_angle(&mut self, mut angle: Vec3) {
        angle.z *= Self::ROLL_SENSITIVITY;
        let up = self.rot * Vec3::Y;
        if up.y < 0.0 {
            angle.x = -angle.x;
        }
        self.angle += angle * Self::SENSITIVITY;

        self.rot = Quat::IDENTITY
            * Quat::from_axis_angle(Vec3::Y, self.angle.x)
            * Quat::from_axis_angle(Vec3::X, self.angle.y)
            * Quat::from_axis_angle(Vec3::Z, self.angle.z);
    }

    /// Bewegt die Kamera in eine Richtung relativ zur Richtung in die die Kamera zeigt.
    pub fn add_input(&mut self, input_vector: Vec3) {
        let acc_vector = self.rot * (self.acc * input_vector);
        self.vel += acc_vector;
    }

    pub fn add_acc(&mut self, acc: Vec3) {
        self.vel += acc * self.delta_time();
    }

    pub fn apply_friction(&mut self) {
        self.vel *= (-Self::FRICTION * self.delta_time()).exp();
    }

    /// Takes a function which takes the current and the next position and returns the next velocity and position
    pub fn advance_pos(&mut self, mut f: impl FnMut(Vec3, Vec3) -> (Vec3, Vec3)) {
        (self.vel, self.pos) = f(self.pos, self.pos + self.vel * self.delta_time.get());
    }

    pub fn update_acc(&mut self, change: f32) {
        let change = change * Self::ACC_CHANGE_SENSITIVITY;
        self.acc = (self.acc
            * if change >= 0.0 {
                change
            } else {
                1.0 / change.abs()
            })
        .clamp(
            Self::STANDART_ACC / Self::MAX_SPEED,
            Self::STANDART_ACC * Self::MAX_SPEED,
        );
    }

    pub fn toggle_free_cam(&mut self) {
        self.free_cam = !self.free_cam
    }

    pub fn free_cam(&self) -> bool {
        self.free_cam
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time.get()
    }

    pub fn pos(&self) -> Vec3 {
        self.pos
    }

    pub fn set_pos(&mut self, pos: Vec3) {
        self.pos = pos;
    }

    pub fn vel(&self) -> Vec3 {
        self.vel
    }

    pub fn set_vel(&mut self, vel: Vec3) {
        self.vel = vel;
    }

    pub fn rot(&self) -> Quat {
        self.rot
    }

    pub fn dir(&self) -> Vec3 {
        self.rot * -Vec3::Z
    }

    /// Diese Funktion gibt eine 4*4 Matrix zurück um die Punkte auf den Bildschirm zu projezieren.
    pub fn view_proj(&self, aspect_ratio: f32) -> [[f32; 4]; 4] {
        let proj = Mat4::perspective_rh(Self::FOV, aspect_ratio, Self::NEAR_PLANE, Self::FAR_PLANE);

        // Erstelle die View-Matrix
        let view = Mat4::from_rotation_translation(self.rot, self.pos).inverse();

        // Kombiniere Projektion und View
        (proj * view).to_cols_array_2d()
    }
}
