use mini3d::{glam::{Mat4, Vec3}, graphics::{SCREEN_ASPECT_RATIO, SCREEN_WIDTH, SCREEN_HEIGHT}};

pub(crate) struct Camera {
    eye: Vec3,
    center: Vec3,
    up: Vec3,
    fov: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self { eye: Vec3::ZERO, center: Vec3::Z, up: Vec3::Y, fov: 80.0 }
    }
}

impl Camera {

    pub(crate) fn update(&mut self, eye: Vec3, forward: Vec3, up: Vec3, fov: f32) {
        self.eye = eye;
        self.up = up;
        self.center = eye + forward;
        self.fov = fov;
    }

    pub(crate) fn projection(&self) -> Mat4 {
        let fov_x = f32::to_radians(self.fov);
        let inv_aspect_ration = 1.0 / SCREEN_ASPECT_RATIO as f32;
        let fov_y = 2.0 * f32::atan(inv_aspect_ration * f32::tan(fov_x * 0.5));
        Mat4::perspective_rh(fov_y, SCREEN_ASPECT_RATIO, 0.5, 50.0)
    }

    pub(crate) fn view(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.center, self.up)
    }
}