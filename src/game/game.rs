use crate::engine::renderer::{Context, DEFAULT_WIDTH, DEFAULT_HEIGHT};
use crate::utils::timer::*;
use crate::utils::camera::Camera;
use crate::engine::mesh::{Mesh, MeshData};
use crate::game::terrain::chunk::{ChunkPosition, BlockPosition};
use crate::game::terrain::block::{BlockType, Direction};
use crate::game::ecs::ECSManager;
use crate::utils::raycast::raycast;
use crate::utils::texture::{TextureAtlas, TextureCoords};
use crate::game::terrain::block_registry::*;
use crate::game::terrain::manager::TerrainManager;

use cgmath::{Vector3, Point3, Zero};
use specs::prelude::*;
use crate::game::ecs::components;
use crate::game::ecs::systems::*;

use std::path::Path;
use std::time::Instant;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Game{
    context: Context,
    ecs_manager: ECSManager,
    terrain_manager: TerrainManager,
    player: Entity,
    camera: Camera,
    timer: UpdateTimer,
    running: bool
}

impl Game{
    pub fn new(title: &str) -> Self{
        let context = Context::new(title, "vertex.glsl", "fragment.glsl");
        let timer = UpdateTimer::new(16);
        let running = true;

        let camera = Camera::new([0., 0., 0.], DEFAULT_WIDTH as f64/ DEFAULT_HEIGHT as f64);
        let mut ecs_manager = ECSManager::new();

        let position = ChunkPosition::new(-1, 0, 0);
        let terrain_manager = TerrainManager::new(position, &context);

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
            terrain_manager,
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

    pub fn tick(&mut self){
        let now = Instant::now();
        self.timer.readjust();

        self.handle_input();

        while self.timer.should_update(){
            self.update();
            self.timer.update();
        }

        self.render(now);
        self.ecs_manager.maintain_world();
    }

    pub fn setup(&mut self){
        // self.terrain_manager.create_chunk_at([0, 0, 1]);
        // self.terrain_manager.create_chunk_at([0, 1, 0]);
        // self.terrain_manager.create_chunk_at([1, 0, 0]);
        // self.terrain_manager.test(0, 0, 0, &self.texture_atlas, &self.registry, self.context.get_display());

        // let terrain_image = self.terrain_manager.generate_image(&self.context.display.0);
        // self.context.ui.replace_terrain_image(terrain_image);

        {
            let mut dt = self.ecs_manager.get_mut_world().write_resource::<DeltaTime>();
            *dt = DeltaTime(to_secs(self.timer.max_ups) as f64 / 1e3);
        }

        // self.terrain_manager.test_terrain();
        // {
        //     use crate::game::ecs::systems::Chunks;
        //     let mut chunks = self.ecs_manager.get_mut_world().write_resource::<Chunks>();
        //     *chunks = Chunks(self.terrain_manager.get_chunks());
        // }
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

        let position = ChunkPosition::new(position.0.x as isize >> 4, position.0.y as isize >> 4, position.0.z as isize >> 4);
        self.terrain_manager.update_chunks(position);
    }

    pub fn handle_input(&mut self){
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
                    // glium::glutin::WindowEvent::MouseInput { state, button, .. } => {
                    //     if state == &glium::glutin::ElementState::Pressed{
                    //         if button == &glium::glutin::MouseButton::Left{
                    //             // let position_storage = world.read_storage::<components::Position>();
                    //             // let position = position_storage.get(self.player).expect("Failed to get Player Position");
                    //             // let camera_storage = world.read_storage::<components::Camera>();
                    //             // let camera = camera_storage.get(self.player).expect("Failed to get Player Position");
                    //             let callback = |block| self.terrain_manager.block_at(block);
                    //             let pos = self.camera.get_position().clone();
                    //             let front = self.camera.get_front().clone();
                    //             match raycast(pos, front, 5., callback){
                    //                 Ok((block, face)) => {
                    //                     println!("Hit result: {:?} {:?}", block ,face);
                    //                     let chunk_position = ChunkPosition::new(block.x as isize >> 4, block.y as isize >> 4, block.z as isize >> 4);
                    //                     let mut chunk = self.terrain_manager.get_mut_chunk(chunk_position);
                    //                     let block_position = BlockPosition::new(block.x as isize, block.y as isize, block.z as isize).get_offset();
                    //                     use std::sync::Arc;
                    //
                    //                     if let Some(chunk) = chunk{
                    //                         let c = Arc::make_mut(chunk);
                    //                         println!("{:?}", block_position);
                    //                         c.remove_block(block_position.x as usize, block_position.y as usize, block_position.z as usize);
                    //                     }
                    //                 },
                    //                 _ => (),
                    //             }
                    //         }else if button == &glium::glutin::MouseButton::Right{
                    //             // let position_storage = world.read_storage::<components::Position>();
                    //             // let position = position_storage.get(self.player).expect("Failed to get Player Position");
                    //             // let camera_storage = world.read_storage::<components::Camera>();
                    //             // let camera = camera_storage.get(self.player).expect("Failed to get Player Position");
                    //             let callback = |block| self.terrain_manager.block_at(block);
                    //             let pos = self.camera.get_position().clone();
                    //             let front = self.camera.get_front().clone();
                    //             match raycast(pos, front, 5., callback){
                    //                 Ok((block, face)) => {
                    //                     println!("Hit result: {:?} {:?}", block ,face);
                    //                     let chunk_position = ChunkPosition::new(block.x as isize >> 4, block.y as isize >> 4, block.z as isize >> 4);
                    //                     let mut chunk = self.terrain_manager.get_mut_chunk(chunk_position);
                    //                     let block_position = BlockPosition::new(block.x as isize, block.y as isize, block.z as isize);
                    //                     let block_offset = block_position.get_offset();
                    //                     use std::sync::Arc;
                    //
                    //                     if let Some(chunk) = chunk{
                    //                         let c = Arc::make_mut(chunk);
                    //                         println!("{:?}", block_position);
                    //                         self.terrain_manager.place_block(block, face);
                    //                     }
                    //                 },
                    //                 _ => (),
                    //             }
                    //         }
                    //     }
                    // },
                    // glium::glutin::WindowEvent::CursorLeft{ .. } => println!("Left!"),
                    // glium::glutin::WindowEvent::CursorEntered{ .. } => println!("Entered!"),
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
                                    glium::glutin::VirtualKeyCode::E => {
                                        // if pressed{
                                        //     let position_storage = world.read_storage::<components::Position>();
                                        //     let position = position_storage.get(self.player).expect("Failed to get Player Position").0;
                                        //     let (x, y, z) = (position.x as isize >> 4, position.y as isize >> 4, position.z as isize >> 4);
                                        //     println!("Chunk at: {:?}", (x, -1, z));
                                        //     for i in -4..4{
                                        //         self.terrain_manager.create_chunk_at([x+i, -1, z]);
                                        //     }
                                        // }
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
                                        let mut world = self.ecs_manager.get_mut_world();
                                        let mut controller_storage = world.write_storage::<components::Controller>();
                                        let mut controller = controller_storage.get_mut(self.player).expect("Failed to get Player Controller");
                                        controller.forward = pressed;
                                    },
                                    glium::glutin::VirtualKeyCode::S => {
                                        let mut world = self.ecs_manager.get_mut_world();
                                        let mut controller_storage = world.write_storage::<components::Controller>();
                                        let mut controller = controller_storage.get_mut(self.player).expect("Failed to get Player Controller");
                                        controller.backward = pressed;
                                    },
                                    glium::glutin::VirtualKeyCode::A => {
                                        let mut world = self.ecs_manager.get_mut_world();
                                        let mut controller_storage = world.write_storage::<components::Controller>();
                                        let mut controller = controller_storage.get_mut(self.player).expect("Failed to get Player Controller");
                                        controller.left = pressed;
                                    },
                                    glium::glutin::VirtualKeyCode::D => {
                                        let mut world = self.ecs_manager.get_mut_world();
                                        let mut controller_storage = world.write_storage::<components::Controller>();
                                        let mut controller = controller_storage.get_mut(self.player).expect("Failed to get Player Controller");
                                        controller.right = pressed;
                                    },
                                    glium::glutin::VirtualKeyCode::Space => {
                                        let mut world = self.ecs_manager.get_mut_world();
                                        let mut controller_storage = world.write_storage::<components::Controller>();
                                        let mut controller = controller_storage.get_mut(self.player).expect("Failed to get Player Controller");
                                        controller.up = pressed;
                                    },
                                    glium::glutin::VirtualKeyCode::LShift => {
                                        let mut world = self.ecs_manager.get_mut_world();
                                        let mut controller_storage = world.write_storage::<components::Controller>();
                                        let mut controller = controller_storage.get_mut(self.player).expect("Failed to get Player Controller");
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


    pub fn render(&mut self, timer: Instant){
        self.context.new_frame();

        self.context.clear_color([0.3, 0.45, 0.65, 1.0]);

        let sampled_texture = self.terrain_manager.sampled_atlas();
        // let texture = self.texture_atlas.get_texture().sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);

        let perspective: [[f32; 4]; 4] = cgmath::perspective(cgmath::Rad::from(cgmath::Deg(90f64)), self.context.get_aspect_ratio(), 0.1f64, 1024f64)
            .cast::<f32>() // Casts internal f64 to f32, since 'double' support in video grahics card is fairly recent...
            .expect("Couldn't cast Perspective f64 to f32")
            .into();
        let view: [[f32; 4]; 4] = self.camera.get_view()
            .cast::<f32>()
            .expect("Couldn't cast View f64 to f32")
            .into();

        for ref_mesh in self.terrain_manager.get_meshes(){
            let (pos, mesh) = (ref_mesh.key(), ref_mesh.value());
            let (x, y, z) = (pos.x as f32, pos.y as f32, pos.z as f32);
            let vector_pos = cgmath::Vector3::new(x, y, z) * 16.;
            let model: [[f32; 4]; 4] = cgmath::Matrix4::from_translation(vector_pos).into();

            let uniforms = uniform!{
                m: model,
                v: view,
                p: perspective,
                t: sampled_texture
            };

            self.context.draw(mesh.get_vb(), mesh.get_ib(), &uniforms);
        }

        // ui
        let elapsed = timer.elapsed();
        let time_left = self.timer.max_ups.checked_sub(elapsed).unwrap_or(std::time::Duration::new(0, 0));
        let timer = Instant::now();
        self.terrain_manager.update_queues();
        if self.terrain_manager.dirty_queue_size() != 0 || self.terrain_manager.chunk_queue_size() != 0 || self.terrain_manager.clean_queue_size() != 0{
            while timer.elapsed() < time_left{
                self.terrain_manager.unqueue_created_chunk();
                self.terrain_manager.mesh_chunk();
                self.terrain_manager.build_chunk(self.context.get_display());
            }
        }

        self.context.ui.debug.set_fps(to_secs(time_left));
        self.context.draw_ui();

        self.context.finish_frame();
    }
}
