use glam::{Mat4, Quat, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub pos: Vec3,
    pub rot: Quat,
    yaw: f32,
    pitch: f32,
    gear: f32,
}
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
impl Camera {
    pub const SENSITIVITY: f32 = 0.0005;
    pub const SPEED: f32 = 0.0000000017;

    const NEAR_PLANE: f32 = 0.001;
    const FAR_PLANE: f32 = 1000.0;

    /// Dreht die Kamera um einen Winkel multipliziert mit der Kamera Sensitivität.
    pub fn rotate_around_angle(&mut self, yaw: f32, pitch: f32, gear: f32) {
        self.yaw += yaw * Self::SENSITIVITY;
        self.pitch += pitch * Self::SENSITIVITY;
        self.gear += gear * Self::SENSITIVITY;

        self.rot = Quat::IDENTITY
            * Quat::from_axis_angle(Vec3::Y, self.yaw)
            * Quat::from_axis_angle(Vec3::X, self.pitch)
            * Quat::from_axis_angle(Vec3::Z, self.gear);
    }
    /// Bewegt die Kamera in eine Richtung relativ zur Richtung in die die Kamera zeigt.
    pub fn move_in_direction(&mut self, vector: Vec3, delta_time: f32) {
        self.pos += self.rot * (vector * Self::SPEED * delta_time);
    }
    /// Viel zu komplizierte Mathematik für das was passiert..
    /// Diese Funktion gibt eine 4*4 Matrix zurück um die Punkte auf den Bildschirm zu projezieren.
    pub fn view_proj(&self, aspect_ratio: f32) -> [[f32; 4]; 4] {
        let x = self.rot.x;
        let y = self.rot.y;
        let z = self.rot.z;
        let w = self.rot.w;
        // Calculate rotation components
        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;
        // Apply negative position for view matrix
        let tx = -self.pos.x;
        let ty = -self.pos.y;
        let tz = -self.pos.z;

        // First create projection matrix
        let proj = Mat4::perspective_rh(
            std::f32::consts::FRAC_PI_3, // 45 degree field of view
            aspect_ratio,
            Self::NEAR_PLANE,
            Self::FAR_PLANE,
        );

        // Then create and multiply with view matrix
        let view = Mat4::from_cols_array_2d(&[
            [1.0 - 2.0 * (yy + zz), 2.0 * (xy - wz), 2.0 * (xz + wy), 0.0],
            [2.0 * (xy + wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - wx), 0.0],
            [2.0 * (xz - wy), 2.0 * (yz + wx), 1.0 - 2.0 * (xx + yy), 0.0],
            [
                tx * (1.0 - 2.0 * (yy + zz)) + ty * 2.0 * (xy - wz) + tz * 2.0 * (xz + wy),
                tx * 2.0 * (xy + wz) + ty * (1.0 - 2.0 * (xx + zz)) + tz * 2.0 * (yz - wx),
                tx * 2.0 * (xz - wy) + ty * 2.0 * (yz + wx) + tz * (1.0 - 2.0 * (xx + yy)),
                1.0,
            ],
        ]);

        (proj * view).to_cols_array_2d()
    }
}
