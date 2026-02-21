use glam::{Mat4, Vec3, Vec4};

pub struct Projection {
    pub aspect: f32,
    pub fov: f32,
    pub znear: f32,

    pub view: Mat4,
}

const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols(
    Vec4::new(1.0, 0.0, 0.0, 0.0),
    Vec4::new(0.0, 1.0, 0.0, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 0.0),
    Vec4::new(0.0, 0.0, 0.5, 1.0),
);

impl Projection {
    pub fn new<F: Into<f32>>(width: u32, height: u32, fov: F, znear: f32, view: View) -> Self {
        let aspect = width as f32 / height as f32;
        let fov = fov.into();

        Self {
            aspect,
            fov,
            znear,

            view: view.calc_matrix(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn update_view(&mut self, view: View) {
        self.view = view.calc_matrix()
    }

    pub fn calc_matrix(&self) -> [[f32; 4]; 4] {
        (proj_matrix(self.fov, self.aspect, self.znear) * self.view).to_cols_array_2d()
    }
}

fn proj_matrix(fov: f32, aspect: f32, znear: f32) -> Mat4 {
    OPENGL_TO_WGPU_MATRIX * perspective_reverse_z(fov, aspect, znear)
}

fn perspective_reverse_z(fovy: f32, aspect: f32, near: f32) -> Mat4 {
    let f = 1.0 / (fovy * 0.5).tan();

    Mat4::from_cols(
        Vec4::new(f / aspect, 0.0, 0.0, 0.0),
        Vec4::new(0.0, f, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 0.0, -1.0),
        Vec4::new(0.0, 0.0, near, 0.0),
    )
}

pub fn dir_from_angle(yaw: f32, pitch: f32) -> Vec3 {
    let (sin_pitch, cos_pitch) = pitch.sin_cos();
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize()
}

pub struct View {
    pos: Vec3,
    dir: Vec3,
}

impl View {
    pub fn new(pos: Vec3, dir: Vec3) -> Self {
        Self { pos, dir }
    }

    pub fn calc_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.pos, self.dir, Vec3::Y)
    }
}
