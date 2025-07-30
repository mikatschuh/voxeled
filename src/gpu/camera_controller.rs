use glam::{Quat, Vec3};

use crate::time::DeltaTime;

pub trait CameraController {
    const SENSITIVITY: f32;
    const ROLL_SENSITIVITY: f32;

    const ACC_CHANGE_SENSITIVITY: f32;
    fn new(pos: Vec3, dir: Vec3, delta_time: DeltaTime) -> Self;

    fn rotate_around_angle(&mut self, angle: Vec3);

    fn update(&mut self, vector: Vec3);

    fn update_acc(&mut self, change: f32);

    fn toggle_flying(&mut self);

    fn pos(&self) -> Vec3;
    fn rot(&self) -> Quat;
    fn dir(&self) -> Vec3;
}
#[derive(Clone, Debug)]
pub struct SmoothController {
    pos: Vec3,
    rot: Quat,
    angle: Vec3,
    vel: Vec3,
    acc: f32,
    flying: bool,
    delta_time: DeltaTime,
}
impl SmoothController {
    const FRICTION: f32 = 0.9;
    const STANDART_ACC: f32 = 0.01;
    const GRAVITY: f32 = 0.00981;
    const MAX_SPEED: f32 = 10.0;
}
impl CameraController for SmoothController {
    const ACC_CHANGE_SENSITIVITY: f32 = 3.0;
    const SENSITIVITY: f32 = 0.001;
    const ROLL_SENSITIVITY: f32 = 5.0;
    fn new(pos: Vec3, dir: Vec3, delta_time: DeltaTime) -> Self {
        let rot = Quat::IDENTITY
            * Quat::from_axis_angle(Vec3::Y, dir.x)
            * Quat::from_axis_angle(Vec3::X, dir.y)
            * Quat::from_axis_angle(Vec3::Z, dir.z);

        Self {
            pos,
            rot,
            angle: Vec3::from(rot.to_euler(glam::EulerRot::YXZ)),
            vel: Vec3::ZERO,
            acc: Self::STANDART_ACC,
            flying: true,
            delta_time,
        }
    }

    /// Dreht die Kamera um einen Winkel multipliziert mit der Kamera Sensitivität.
    fn rotate_around_angle(&mut self, mut angle: Vec3) {
        angle.z *= Self::ROLL_SENSITIVITY;
        self.angle += angle * Self::SENSITIVITY;

        self.rot = Quat::IDENTITY
            * Quat::from_axis_angle(Vec3::Y, self.angle.x)
            * Quat::from_axis_angle(Vec3::X, self.angle.y)
            * Quat::from_axis_angle(Vec3::Z, self.angle.z);
    }
    /// Bewegt die Kamera in eine Richtung relativ zur Richtung in die die Kamera zeigt.
    fn update(&mut self, vector: Vec3) {
        let vector = vector * self.acc;
        self.vel = ((self.vel + (self.rot * vector) * self.delta_time.get()) * Self::FRICTION)
            .clamp(
                -Vec3::new(Self::MAX_SPEED, Self::MAX_SPEED, Self::MAX_SPEED),
                Vec3::new(Self::MAX_SPEED, Self::MAX_SPEED, Self::MAX_SPEED),
            );

        self.pos += self.vel * self.delta_time.get();
    }
    fn update_acc(&mut self, change: f32) {
        let change = change * Self::ACC_CHANGE_SENSITIVITY;
        self.acc = (self.acc
            * if change >= 0.0 {
                change
            } else {
                1.0 / change.abs()
            })
        .clamp(Self::STANDART_ACC / 50.0, Self::STANDART_ACC * 50.0);
    }
    fn toggle_flying(&mut self) {
        self.flying = !self.flying
    }
    fn pos(&self) -> Vec3 {
        self.pos
    }
    fn rot(&self) -> Quat {
        self.rot
    }
    fn dir(&self) -> Vec3 {
        self.rot * Vec3::Z
    }
}
