use cgmath::Vector3;
use num_enum::TryFromPrimitive;
use cgmath::InnerSpace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(usize)]
pub enum Direction{
    East = 0,
    West = 1,
    Top = 2,
    Bottom = 3,
    North = 4,
    South = 5
}

impl Direction{
    pub fn normal(&self) -> Vector3<isize> {
        match self{
            Direction::East => Vector3::new(1, 0, 0),
            Direction::West => Vector3::new(-1, 0, 0),
            Direction::Top => Vector3::new(0, 1, 0),
            Direction::Bottom => Vector3::new(0, -1, 0),
            Direction::North => Vector3::new(0, 0, 1),
            Direction::South => Vector3::new(0, 0, -1),
        }
    }
}

#[allow(illegal_floating_point_literal_pattern)]
impl From<Vector3<f32>> for Direction{
    fn from(v: Vector3<f32>) -> Self {
        match v.normalize(){
            Vector3 { x: 1., .. } => Direction::East,
            Vector3 { x: -1., .. } => Direction::West,
            Vector3 { y: 1., .. } => Direction::Top,
            Vector3 { y: -1., .. } => Direction::Bottom,
            Vector3 { z: 1., .. } => Direction::North,
            Vector3 { z: -1., .. } | _ => Direction::South,
            // _ => {println!("v: {:?}", v); panic!("More than one direction found!")},
        }
    }
}

#[allow(dead_code)]
#[derive(Default)]
pub struct BlockData{
    faces: [[u32; 2]; 6],
    mesh: usize,
    breakable: bool,
    transparent: bool
}

impl BlockData{
    pub fn new(faces: [[u32; 2]; 6], mesh: usize, breakable: bool, transparent: bool) -> Self{
        Self{
            faces,
            mesh,
            breakable,
            transparent
        }
    }

    pub fn get_mesh(&self) -> usize{
        self.mesh
    }

    pub fn get_face(&self, dir: Direction) -> [u32; 2]{
        self.faces[dir as usize]
    }

    pub fn is_transparent(&self) -> bool{
        self.transparent
    }
}
