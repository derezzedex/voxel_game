pub use glam::{Vec3, Mat4};

// #[cfg_attr(rustfmt, rustfmt_skip)]
// const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.0,
//     0.0, 0.0, 0.5, 1.0,
// );

#[derive(Debug)]
pub struct Camera{
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


const DAMP: f32 = 0.75;
const DAMP_LIMIT: f32 = 0.01;
const SENSITIVITY: f32 = 0.05;

impl Default for Camera{
    fn default() -> Self{
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

impl Camera{
    pub fn get_view(&self) -> Mat4{
        Mat4::look_at_rh(self.eye, self.eye + self.target, self.up)
    }

    pub fn get_projection(&self) -> Mat4{
        Mat4::perspective_rh_gl(self.fovy, self.aspect, self.near, self.far)
    }

    pub fn mouse_update(&mut self, dx: f32, dy: f32){
        self.yaw += dx * SENSITIVITY;
        self.pitch += - dy * SENSITIVITY;

        self.pitch = self.pitch.max(-89.9).min(89.9);

        let dx = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        let dy = self.pitch.to_radians().sin();
        let dz = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();

        self.target = Vec3::new(dx, dy, dz).normalize();
    }

    pub fn hard_update(&mut self, dt: f32){
        self.eye.set_x(self.eye.x() + self.velocity.x() * dt);
        self.eye.set_y(self.eye.y() + self.velocity.y() * dt);
        self.eye.set_z(self.eye.z() + self.velocity.z() * dt);
    }

    pub fn smooth_update(&mut self, dt: f32){
        self.eye.set_x(self.eye.x() + self.velocity.x() * dt);
        self.eye.set_y(self.eye.y() + self.velocity.y() * dt);
        self.eye.set_z(self.eye.z() + self.velocity.z() * dt);

        let mut vx = self.velocity.x() * DAMP;
        let mut vy = self.velocity.y() * DAMP;
        let mut vz = self.velocity.z() * DAMP;

        if (vx.is_sign_positive() && vx <= DAMP_LIMIT) || (vx.is_sign_negative() && vx >= -DAMP_LIMIT) {
            vx = 0.;
        }

        if (vy.is_sign_positive() && vy <= DAMP_LIMIT) || (vy.is_sign_negative() && vy >= -DAMP_LIMIT) {
            vy = 0.;
        }

        if (vz.is_sign_positive() && vz <= DAMP_LIMIT) || (vz.is_sign_negative() && vz >= -DAMP_LIMIT) {
            vz = 0.;
        }

        self.velocity.set_x(vx);
        self.velocity.set_y(vy);
        self.velocity.set_z(vz);
    }
}
