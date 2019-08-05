use crate::engine::renderer::{Context, DEFAULT_WIDTH, DEFAULT_HEIGHT};
use crate::utils::timer::*;
use crate::utils::camera::Camera;
use crate::engine::mesh::{Mesh, MeshData};
use crate::game::block::{FaceData, BlockType, Direction};
use crate::game::ecs::ECSManager;

use cgmath::{Vector3, Point3, Zero};
use specs::prelude::*;
use crate::game::ecs::components;
use crate::game::ecs::systems::*;

use std::time::Instant;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct ChunkPosition{
    x: isize,
    y: isize,
    z: isize
}

impl ChunkPosition{
    pub fn new(x: isize, y: isize, z: isize) -> Self{
        Self{
            x,
            y,
            z
        }
    }
}

pub struct Game<'a, 'b>{
    context: Context<'static>,
    ecs_manager: ECSManager<'a, 'b>,
    meshes: HashMap<ChunkPosition, Mesh>,
    player: Entity,
    camera: Camera,
    timer: UpdateTimer,
    running: bool
}

impl<'a, 'b> Game<'a, 'b>{
    pub fn new(title: &str) -> Self{
        let context = Context::new(title, "shaders/vertex.glsl", "shaders/fragment.glsl");
        let timer = UpdateTimer::new(16);
        let running = true;

        let camera = Camera::new([0., 0., 5.], DEFAULT_WIDTH as f64/ DEFAULT_HEIGHT as f64);
        let mut ecs_manager = ECSManager::new();
        let meshes = HashMap::new();

        let player_pos = components::Position(camera.get_position());
        let player_vel = components::Velocity(cgmath::Vector3::zero());
        let player_cam = components::Camera{
            looking_at: camera.get_front(),
        };
        let player_controller = components::Controller::new();

        let mut world = ecs_manager.get_mut_world();
        let player = world
                        .create_entity()
                        .with(player_cam)
                        .with(player_pos)
                        .with(player_vel)
                        .with(player_controller)
                        .build();

        Self{
            context,
            ecs_manager,
            player,
            meshes,
            camera,
            timer,
            running
        }
    }

    pub fn run(&mut self){
        self.setup();

        while self.running{
            self.tick();
        }
    }

    pub fn setup(&mut self){
        use crate::game::block;
        let mut mesh = MeshData::new();
        let face_data = FaceData::new([0, 0, 0], BlockType::Dirt, Direction::North);
        mesh.add_face(face_data);

        let pos = ChunkPosition::new(0, 0, 0);
        let mesh = mesh.build(self.context.get_display());
        self.meshes.insert(pos, mesh);

        {
            let mut dt = self.ecs_manager.get_mut_world().write_resource::<DeltaTime>();
            *dt = DeltaTime(to_secs(self.timer.max_ups) as f64 / 1e3);
        }
    }

    pub fn tick(&mut self){
        self.timer.readjust();

        self.handle_input();

        while self.timer.should_update(){
            self.update();
            self.timer.update();
        }

        self.render();
        self.ecs_manager.maintain_world();
    }

    pub fn handle_input(&mut self){
        let mut controller_storage = self.ecs_manager.get_mut_world().write_storage::<components::Controller>();
        let mut controller = controller_storage.get_mut(self.player).expect("Failed to get Player Controller");

        let events = self.context.poll_events();
        for event in &events{

            if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &self.context.display){
                self.context.ui.handle_event(event);
            }

            match event{
                glium::glutin::Event::DeviceEvent{ event, ..} => match event{
                    glium::glutin::DeviceEvent::MouseMotion{ delta } => {
                        self.camera.handle_mouse(delta.0, delta.1);
                        self.context.reset_mouse_position();
                    },
                    _ => (),
                },
                glium::glutin::Event::WindowEvent { event, .. } => match event{
                    glium::glutin::WindowEvent::CloseRequested => self.running = false,
                    glium::glutin::WindowEvent::KeyboardInput{input, ..} => {
                        let pressed = match input.state{
                            glium::glutin::ElementState::Pressed => true,
                            _ => false,
                        };
                        match input.virtual_keycode{
                            Some(key) => {
                                match key{
                                    glium::glutin::VirtualKeyCode::Apostrophe => {
                                        if pressed{
                                            self.context.ui.debug.switch();
                                        }
                                    },
                                    glium::glutin::VirtualKeyCode::P => {
                                        if pressed{
                                            self.context.grab_mouse();
                                            // self.context.reset_mouse_position();
                                        }
                                    },
                                    glium::glutin::VirtualKeyCode::Escape => {
                                        self.running = false;
                                    },
                                    glium::glutin::VirtualKeyCode::W => {
                                        controller.forward = pressed;
                                    },
                                    glium::glutin::VirtualKeyCode::S => {
                                        controller.backward = pressed;
                                    },
                                    glium::glutin::VirtualKeyCode::A => {
                                        controller.left = pressed;
                                    },
                                    glium::glutin::VirtualKeyCode::D => {
                                        controller.right = pressed;
                                    },
                                    glium::glutin::VirtualKeyCode::Space => {
                                        controller.up = pressed;
                                    },
                                    glium::glutin::VirtualKeyCode::LShift => {
                                        controller.down = pressed;
                                    },
                                    _ => (),
                                }
                            },
                            None => (),
                        }
                    },
                    _ => ()
                }
                _ => (),
            }
        }
    }

    pub fn update(&mut self){
        {
            let mut camera_storage = self.ecs_manager.get_mut_world().write_storage::<components::Camera>();
            let mut camera = camera_storage.get_mut(self.player).expect("Failed to get Player Camera");

            camera.looking_at = self.camera.get_front();
        }

        self.ecs_manager.run_systems();

        // sync player position with camera
        let position_storage = self.ecs_manager.get_mut_world().read_storage::<components::Position>();
        let position = position_storage.get(self.player).expect("Failed to get Player Position");
        self.camera.set_positon(position.0);

        self.camera.update();
    }

    pub fn render(&mut self){
        self.context.new_frame();

        self.context.clear_color([0.3, 0.45, 0.65, 1.0]);

        let perspective: [[f32; 4]; 4] = cgmath::perspective(cgmath::Rad::from(cgmath::Deg(90f64)), self.context.get_aspect_ratio(), 0.1f64, 1024f64)
            .cast::<f32>()
            .expect("Couldn't cast Perspective f64 to f32")
            .into();
        let view: [[f32; 4]; 4] = self.camera.get_view()
            .cast::<f32>()
            .expect("Couldn't cast View f64 to f32")
            .into();

        for (pos, mesh) in &self.meshes{
            let (x, y, z) = (pos.x as f32, pos.y as f32, pos.z as f32);
            let vector_pos = cgmath::Vector3::new(x, y, z);
            let model: [[f32; 4]; 4] = cgmath::Matrix4::from_translation(vector_pos).into();

            let uniforms = uniform!{
                m: model,
                v: view,
                p: perspective
            };

            self.context.draw(mesh.get_vb(), mesh.get_ib(), &uniforms);
        }

        // ui
        self.context.ui.debug.set_fps(to_secs(self.timer.max_ups)/1e3);
        self.context.draw_ui();

        self.context.finish_frame();
    }
}
