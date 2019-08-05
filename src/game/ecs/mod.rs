use specs::prelude::*;

pub mod components;
pub mod systems;

use systems::*;

pub struct ECSManager<'a, 'b>{
    world: World,
    dispatcher: Dispatcher<'a, 'b>
}

impl<'a, 'b> ECSManager<'a, 'b>{
    pub fn new() -> Self{
        let mut world = World::new();
        let mut dispatcher = DispatcherBuilder::new()
            .with(InputSystem, "input", &[])
            .with(MovementSystem, "movement", &["input"])
            .build();

        dispatcher.setup(&mut world);

        Self{
            world,
            dispatcher
        }
    }

    // pub fn setup_dispatcher(&mut self){
    //     self.dispatcher.setup(&mut self.world);
    // }

    pub fn get_mut_world(&mut self) -> &mut World{
        &mut self.world
    }

    pub fn run_systems(&mut self){
        self.dispatcher.dispatch(&self.world);
    }

    pub fn maintain_world(&mut self){
        self.world.maintain();
    }

    pub fn read_storage<T: specs::Component>(&self) -> ReadStorage<T>{
        self.world.read_storage::<T>()
    }
}
