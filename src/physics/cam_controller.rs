use glam::{Quat, Vec3};

use crate::{
    gpu::projection::{View, dir_from_angle},
    physics::{TCBody, time::DeltaTime, verlet::Body},
};

pub struct CamController {
    body: TCBody,

    free_cam: bool,
    speed: f32, // camera speed

    yaw: f32, // source of truth
    pitch: f32,

    rot: Quat, // cached calculations
    dir: Vec3,

    delta_time: DeltaTime,
}

impl CamController {
    const FRICTION: f32 = 1.0;
    const STANDART_SPEED: f32 = 100.0;
    const MAX_SPEED: f32 = 100.0;
    const ACC_CHANGE_SENSITIVITY: f32 = 3.0;
    const SENSITIVITY: f32 = 0.001;

    pub fn new(pos: Vec3, yaw: f32, pitch: f32, free_cam: bool, delta_time: DeltaTime) -> Self {
        let dir = dir_from_angle(yaw, pitch);
        let rot = Quat::from_rotation_y(yaw) * Quat::from_rotation_z(pitch);

        Self {
            body: TCBody::new(pos),

            free_cam,
            speed: Self::STANDART_SPEED,

            rot,
            dir,
            yaw,
            pitch,

            delta_time,
        }
    }

    /// Dreht die Kamera um einen Winkel multipliziert mit der Kamera Sensitivität.
    pub fn rotate_around_angle(&mut self, yaw: f32, pitch: f32) {
        self.yaw += yaw * Self::SENSITIVITY;
        self.pitch += pitch * Self::SENSITIVITY;

        self.dir = dir_from_angle(self.yaw, self.pitch);
        self.rot = Quat::from_rotation_y(yaw) * Quat::from_rotation_z(pitch);
    }

    /// Bewegt die Kamera in eine Richtung relativ zur Richtung in die die Kamera zeigt.
    pub fn add_input(&mut self, input_vector: Vec3) {
        let (yaw_sin, yaw_cos) = self.yaw.sin_cos();
        let forward = dir_from_angle(self.yaw, self.pitch);
        let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        let impuls = forward * input_vector.x * self.speed
            + right * input_vector.z * self.speed
            + Vec3::Y * input_vector.y * self.speed;

        self.body.add_impuls(impuls);
    }

    pub fn add_acc(&mut self, acc: Vec3) {
        self.body.add_impuls(acc * self.delta_time());
    }

    /// Takes a function which takes the current and the next position and returns the resolved position.
    pub fn advance_pos(&mut self, contrain: impl FnMut(Vec3, Vec3) -> Vec3) {
        self.body.step(self.delta_time(), Self::FRICTION);

        self.body.constrain(contrain);
    }

    pub fn update_speed(&mut self, change: f32) {
        let change = change * Self::ACC_CHANGE_SENSITIVITY;
        self.speed = (self.speed
            * if change >= 0.0 {
                change
            } else {
                1.0 / change.abs()
            })
        .clamp(
            Self::STANDART_SPEED / Self::MAX_SPEED,
            Self::STANDART_SPEED * Self::MAX_SPEED,
        );
    }

    pub fn toggle_free_cam(&mut self) {
        self.free_cam = !self.free_cam
    }

    pub fn free_cam(&self) -> bool {
        self.free_cam
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time.get_f32()
    }

    pub fn pos(&self) -> Vec3 {
        self.body.pos()
    }

    pub fn dir(&self) -> Vec3 {
        self.dir
    }

    pub fn view(&self) -> View {
        View::new(self.pos(), self.dir)
    }
}
