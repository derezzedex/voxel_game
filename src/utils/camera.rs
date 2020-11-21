use ultraviolet::{Mat4, Vec3};

#[derive(Debug)]
pub struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,

    aspect: f32,
    fovy: f32,
    near: f32,
    far: f32,

    velocity: Vec3,
    yaw: f32,
    pitch: f32,
}

const SENSITIVITY: f32 = 0.05;

pub enum Direction {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: (0., 0., 2.).into(),
            target: (0., 0., -1.).into(),
            up: Vec3::unit_y(),
            aspect: 800. / 600.,
            fovy: 90f32.to_radians(),
            near: 0.1,
            far: 100.,
            velocity: Vec3::new(0., 0., 0.),
            yaw: -90.,
            pitch: 0.,
        }
    }
}

impl Camera {
    pub fn get_view(&self) -> Mat4 {
        Mat4::look_at(self.eye, self.eye + self.target, self.up)
    }

    pub fn get_projection(&self) -> Mat4 {
        ultraviolet::projection::perspective_wgpu_dx(self.fovy, self.aspect, self.near, self.far)
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>){
        self.aspect = new_size.width as f32/ new_size.height as f32;
    }

    pub fn mouse_update(&mut self, dx: f32, dy: f32, _dt: f32) {
        self.yaw += dx * SENSITIVITY;
        self.pitch += -dy * SENSITIVITY;

        self.pitch = self.pitch.max(-89.9).min(89.9);

        let dx = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        let dy = self.pitch.to_radians().sin();
        let dz = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();

        self.target = Vec3::new(dx, dy, dz).normalized();
    }
}
