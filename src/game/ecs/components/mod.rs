use specs::prelude::{Component, HashMapStorage, VecStorage};
use cgmath::{Vector3, Point3};

// ============ VELOCITY ============
pub struct Velocity(pub Vector3<f64>);

impl Component for Velocity{
    type Storage = VecStorage<Self>;
}

// ============ POSITION ============
pub struct Position(pub Point3<f64>);

impl Component for Position{
    type Storage = VecStorage<Self>;
}

// ============ BOUNDINGBOX ============
/// [0]: TopLeft [1]: BottomRight
pub struct BoundingBox(pub Point3<f64>);

impl Component for BoundingBox{
    type Storage = VecStorage<Self>;
}

// ============ CAMERA ============
pub struct Camera{
    pub looking_at: Vector3<f64>,
}

impl Component for Camera{
    type Storage = HashMapStorage<Self>;
}

// ============ CONTROLLER ============
pub struct Controller{
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool
}

impl Controller{
    pub fn new() -> Self{
        Self{
            forward: false,
            backward: false,
            left: false,
            right: false,
            up: false,
            down: false
        }
    }
}

impl Component for Controller{
    type Storage = HashMapStorage<Self>;
}
