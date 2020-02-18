use glium::glutin;
use cgmath::{Point3, Vector3, Matrix4, EuclideanSpace, InnerSpace, Zero};

const SENSITIVITY: f64 = 0.08;

pub struct Camera{
    position: Point3<f64>,
    front: Vector3<f64>,
    view: Matrix4<f64>,
    yaw: f64,
    pitch: f64,
}

impl Camera{
    pub fn new(position: [f64; 3], aspect_ratio: f64) -> Self{
        let position = cgmath::Point3::new(position[0], position[1], position[2]);
        let front = cgmath::Vector3::new(0., 0., 1.);
        let view = cgmath::Matrix4::look_at(position, position + front, cgmath::Vector3::unit_y());

        let (yaw, pitch) = (0., 0.);

        Self{
            position,
            front,
            view,
            yaw,
            pitch
        }
    }

    pub fn handle_mouse(&mut self, delta_x: f64, delta_y: f64){
        let x = delta_x * SENSITIVITY;
        let y = -delta_y * SENSITIVITY;

        self.yaw += x;
        self.pitch += y;

        if self.pitch > 89.0 { self.pitch = 89.0 }
        if self.pitch < -89.0 { self.pitch = -89.0 }

        let mut front = Vector3::zero();
        front.x = self.yaw.to_radians().cos() * self.pitch.to_radians().cos();
        front.y = self.pitch.to_radians().sin();
        front.z = self.yaw.to_radians().sin() * self.pitch.to_radians().cos();
        self.front = front.normalize();
    }


    pub fn update(&mut self){
        self.view = cgmath::Matrix4::look_at(self.position, self.position + self.front, cgmath::Vector3::unit_y());
    }

    pub fn set_positon(&mut self, pos: Point3<f64>){
        self.position = pos;
    }

    pub fn get_position(&self) -> Point3<f64>{
        self.position
    }

    pub fn get_front(&self) -> Vector3<f64>{
        self.front
    }

    pub fn get_view(&self) -> Matrix4<f64>{
        self.view
    }
}
