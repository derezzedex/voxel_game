use cgmath::{InnerSpace, Matrix4, Point3, Vector3, Zero};

const SENSITIVITY: f64 = 0.08;

pub struct Camera {
    position: Point3<f64>,
    front: Vector3<f64>,
    view: Matrix4<f64>,
    yaw: f64,
    pitch: f64,
}

impl Camera {
    pub fn new(position: [f64; 3]) -> Self {
        let position = cgmath::Point3::new(position[0], position[1], position[2]);
        let front = cgmath::Vector3::new(1., 0., 0.);
        let view =
            cgmath::Matrix4::look_at_rh(position, position + front, cgmath::Vector3::unit_y());

        let (yaw, pitch) = (0., 0.);

        Self {
            position,
            front,
            view,
            yaw,
            pitch,
        }
    }

    pub fn handle_mouse(&mut self, (delta_x, delta_y): (f64, f64)) {
        let x = delta_x * SENSITIVITY;
        let y = -delta_y * SENSITIVITY;

        self.yaw += x;
        self.pitch += y;

        self.pitch = self.pitch.clamp(-89.0, 89.0);

        let mut front = Vector3::zero();
        front.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        front.y = self.pitch.to_radians().sin();
        front.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();
        self.front = front.normalize();
    }

    pub fn update(&mut self, position: Point3<f64>) {
        self.position = position;
        self.view = cgmath::Matrix4::look_at_rh(
            self.position,
            self.position + self.front,
            cgmath::Vector3::unit_y(),
        );
    }

    pub fn position(&self) -> Point3<f64> {
        self.position
    }

    pub fn front(&self) -> Vector3<f64> {
        self.front
    }

    pub fn view(&self) -> Matrix4<f64> {
        self.view
    }
}
