use rayon::prelude::*;
use specs::prelude::*;

use crate::game::ecs::components::*;

use cgmath::{Vector3, InnerSpace, Zero};

#[derive(Default)]
pub struct DeltaTime(pub f64);

pub struct MovementSystem;
impl<'a> System<'a> for MovementSystem{
    type SystemData = (
        Read<'a, DeltaTime>,
        ReadStorage<'a, Velocity>,
        WriteStorage<'a, Position>
    );

    fn run(&mut self, (delta, velocities, mut positions): Self::SystemData){
        (&velocities, &mut positions)
            .par_join()
            .for_each(|(vel, pos)|{
                pos.0 += vel.0 * delta.0;
            });
    }
}

const PLAYER_VEL: f64 = 512.0;

pub struct InputSystem;
impl<'a> System<'a> for InputSystem{
    type SystemData = (
        Read<'a, DeltaTime>,
        ReadStorage<'a, Controller>,
        ReadStorage<'a, Camera>,
        WriteStorage<'a, Velocity>
    );

    fn run(&mut self, (delta, controllers, cameras, mut velocity): Self::SystemData){
        (&controllers, &cameras, &mut velocity)
            .par_join()
            .for_each(|(con, cam, vel)|{
                // let mut velo = vel.0;
                let dt = delta.0;

                vel.0 = Vector3::zero();
                let perpendicular = cam.looking_at.cross(Vector3::unit_y()).normalize();
                vel.0 += perpendicular * ((-(con.left as i8)) + con.right as i8) as f64;

                vel.0 += cam.looking_at.cross(perpendicular).normalize() * ((-(con.up as i8)) + con.down as i8) as f64;

                vel.0 += cam.looking_at * ((-(con.backward as i8)) + con.forward as i8) as f64;

                vel.0 *= PLAYER_VEL * dt;
                // println!("Velo{:?}", );
            });
    }
}
