use cgmath::{Point3, Vector3, InnerSpace};

pub const HALF_VOXEL: f32 = 0.5;
pub struct VoxelRay{
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    length: usize,
}

impl VoxelRay{
    pub fn new(position: Point3<f32>, destination: Point3<f32>, length: usize) -> Self{
        // let position = position + Vector3::new(HALF_VOXEL, HALF_VOXEL, HALF_VOXEL);
        let direction = (destination - position).normalize();
        Self{
            position,
            direction,
            length
        }
    }

    //https://www.gamedev.net/forums/topic/624201-voxel-traversal-problem/
    pub fn until<T: Fn(Point3<f32>, Vector3<i8>) -> bool>(&mut self, callback: T) -> Option<(Point3<f32>, Vector3<i8>)>{
        self.position += Vector3::new(HALF_VOXEL, HALF_VOXEL, HALF_VOXEL);
        let mut position = self.position.map(|p| p.floor());
        let step = self.direction.map(|p| if p > 0. { 1. } else if p < 0. { -1. } else { 0. });
        let next_boundary = Vector3::new(
            position.x + (if step.x > 0. { 1. } else { 0. }),
            position.y + (if step.y > 0. { 1. } else { 0. }),
            position.z + (if step.z > 0. { 1. } else { 0. })
        );
        let mut max = Vector3::new(
            (next_boundary.x - self.position.x) / self.direction.x,
            (next_boundary.y - self.position.y) / self.direction.y,
            (next_boundary.z - self.position.z) / self.direction.z,
        );

        if max.x.is_nan() {max.x = std::f32::INFINITY;};
        if max.y.is_nan() {max.y = std::f32::INFINITY;};
        if max.z.is_nan() {max.z = std::f32::INFINITY;};

        let mut delta = Vector3::new(
            step.x / self.direction.x,
            step.y / self.direction.y,
            step.z / self.direction.z
        );

        if delta.x.is_nan() {delta.x = std::f32::INFINITY;};
        if delta.y.is_nan() {delta.y = std::f32::INFINITY;};
        if delta.z.is_nan() {delta.z = std::f32::INFINITY;};

        let mut face = Vector3::new(0, 0, 0);

        let length = self.length as f32 / (self.direction.x.powf(2.) + self.direction.y.powf(2.) + self.direction.z.powf(2.)).sqrt();
        for _ in 0..self.length{
            if callback(position, Vector3::new(0, 0, 0)){
                return Some((position, face));
            }

            if max.x < max.y && max.x < max.z{
                if max.x > length { break }
                position.x += step.x;
                max.x += delta.x;
                face = Vector3::new(-step.x as i8, 0, 0);
            }else if max.y < max.z{
                if max.y > length { break }
                position.y += step.y;
                max.y += delta.y;
                face = Vector3::new(0, -step.y as i8, 0);
            }else{
                if max.z > length { break }
                position.z += step.z;
                max.z += delta.z;
                face = Vector3::new(0, 0, -step.z as i8);
            }
        }

        None
    }
}
