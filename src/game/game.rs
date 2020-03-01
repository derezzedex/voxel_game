use crate::engine::renderer::Context;
use crate::game::ecs::ECSManager;
use crate::game::registry::Registry;
use crate::game::terrain::chunk::{ChunkPosition, CHUNKSIZE};
use crate::game::terrain::manager::TerrainManager;
use crate::engine::utils::camera::Camera;
use crate::engine::utils::texture::TextureStorage;
use crate::engine::utils::clock::*;
use crate::engine::utils::raycast::raycast;

use crate::game::ecs::components;
use crate::game::ecs::systems::*;
use cgmath::{Point3, Vector3, Zero};
use collision::{Frustum, Aabb3, Relation};
use specs::prelude::*;

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

#[allow(dead_code)]
pub struct Game {
    context: Context,
    ecs_manager: ECSManager,
    registry: Arc<Registry>,
    terrain_manager: TerrainManager,
    texture_storage: TextureStorage,
    player: Entity,
    camera: Camera,
    timer: Clock,
    running: bool,
}

impl Game {
    pub fn new(title: &str) -> Self {
        let context = Context::new(title, "vertex.glsl", "fragment.glsl");
        let timer = Clock::new(16);
        let running = true;

        let camera = Camera::new([0., 0., 0.]); //, DEFAULT_WIDTH as f64/ DEFAULT_HEIGHT as f64);
        let mut ecs_manager = ECSManager::new();

        let texture_path = Path::new("res")
            .join("img")
            .join("texture")
            .join("atlas.png");
        let texture_storage = TextureStorage::new(
            context.get_display(),
            &texture_path,
            image::ImageFormat::Png,
            16,
        );

        let player_pos = components::Position(camera.get_position());
        let player_vel = components::Velocity(cgmath::Vector3::zero());
        let player_cam = components::Camera {
            looking_at: camera.get_front(),
        };
        let player_controller = components::Controller::new();

        let world = ecs_manager.get_mut_world();
        let player = world
            .create_entity()
            .with(player_cam)
            .with(player_pos)
            .with(player_vel)
            .with(player_controller)
            .build();

        let mut registry = Registry::new();
        registry.setup();
        let registry = Arc::new(registry);
        let terrain_manager = TerrainManager::new(&registry);

        Self {
            context,
            ecs_manager,
            terrain_manager,
            texture_storage,
            player,
            camera,
            registry,
            timer,
            running,
        }
    }

    pub fn run(&mut self) {
        self.setup();

        while self.running {
            self.tick();
        }
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        self.timer.readjust();

        self.handle_input();

        while self.timer.should_update() {
            self.update();
            self.timer.update();
        }

        self.render(now);
        self.ecs_manager.maintain_world();
    }

    pub fn setup(&mut self) {
        {
            let mut dt = self
                .ecs_manager
                .get_mut_world()
                .write_resource::<DeltaTime>();
            *dt = DeltaTime(to_secs(self.timer.max_ups) as f64 / 1e3);
        }


        self.terrain_manager.setup(self.context.get_display());
    }

    pub fn update(&mut self) {
        {
            let mut camera_storage = self
                .ecs_manager
                .get_mut_world()
                .write_storage::<components::Camera>();
            let mut camera = camera_storage
                .get_mut(self.player)
                .expect("Failed to get Player Camera");

            camera.looking_at = self.camera.get_front();
        }

        self.ecs_manager.run_systems();

        // sync player position with camera
        let position_storage = self
            .ecs_manager.read_storage::<components::Position>();
        let position = position_storage
            .get(self.player)
            .expect("Failed to get Player Position").0;
        self.camera.set_positon(position);
        self.camera.update();

        let cam_chunk_pos = ChunkPosition::new(
            (position.x / (CHUNKSIZE - 1) as f64).floor() as isize,
            (position.y / (CHUNKSIZE - 1) as f64).floor() as isize,
            (position.z / (CHUNKSIZE - 1) as f64).floor() as isize,
        );
        self.terrain_manager.update(cam_chunk_pos);
    }

    pub fn handle_input(&mut self) {
        let events = self.context.poll_events();
        for event in &events {
            match event {
                glium::glutin::Event::DeviceEvent { event, .. } => match event {
                    glium::glutin::DeviceEvent::MouseMotion { delta } => {
                        self.camera.handle_mouse(delta.0, delta.1);
                        self.context.reset_mouse_position();
                    },
                    _ => (),
                },
                glium::glutin::Event::WindowEvent { event, .. } => match event {
                    glium::glutin::WindowEvent::CloseRequested => self.running = false,
                    glium::glutin::WindowEvent::KeyboardInput { input, .. } => {
                        let pressed = match input.state {
                            glium::glutin::ElementState::Pressed => true,
                            _ => false,
                        };
                        match input.virtual_keycode {
                            Some(key) => {
                                match key {
                                    glium::glutin::VirtualKeyCode::P => {
                                        if pressed {
                                            self.context.grab_mouse();
                                        }
                                    }
                                    glium::glutin::VirtualKeyCode::Escape => {
                                        self.running = false;
                                    }
                                    glium::glutin::VirtualKeyCode::W => {
                                        let mut controller_storage = self.ecs_manager.write_storage::<components::Controller>();
                                        let mut controller = controller_storage
                                            .get_mut(self.player)
                                            .expect("Failed to get Player Controller");
                                        controller.forward = pressed;
                                    }
                                    glium::glutin::VirtualKeyCode::S => {
                                        let mut controller_storage = self.ecs_manager.write_storage::<components::Controller>();
                                        let mut controller = controller_storage
                                            .get_mut(self.player)
                                            .expect("Failed to get Player Controller");
                                        controller.backward = pressed;
                                    }
                                    glium::glutin::VirtualKeyCode::A => {
                                        let mut controller_storage = self.ecs_manager.write_storage::<components::Controller>();
                                        let mut controller = controller_storage
                                            .get_mut(self.player)
                                            .expect("Failed to get Player Controller");
                                        controller.left = pressed;
                                    }
                                    glium::glutin::VirtualKeyCode::D => {
                                        let mut controller_storage = self.ecs_manager.write_storage::<components::Controller>();
                                        let mut controller = controller_storage
                                            .get_mut(self.player)
                                            .expect("Failed to get Player Controller");
                                        controller.right = pressed;
                                    }
                                    glium::glutin::VirtualKeyCode::Space => {
                                        let mut controller_storage = self.ecs_manager.write_storage::<components::Controller>();
                                        let mut controller = controller_storage
                                            .get_mut(self.player)
                                            .expect("Failed to get Player Controller");
                                        controller.up = pressed;
                                    }
                                    glium::glutin::VirtualKeyCode::LShift => {
                                        let mut controller_storage = self.ecs_manager.write_storage::<components::Controller>();
                                        let mut controller = controller_storage
                                            .get_mut(self.player)
                                            .expect("Failed to get Player Controller");
                                        controller.down = pressed;
                                    }
                                    _ => (),
                                }
                            }
                            None => (),
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    pub fn render(&mut self, _timer: Instant) {
        self.context.new_frame();
        self.context.clear_color([0.3, 0.45, 0.65, 1.0]);
        // self.context.clear_color([0.5, 0.5, 0.5, 1.0]);

        let texture = self
            .texture_storage
            .get_array()
            .sampled()
            .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
            .wrap_function(glium::uniforms::SamplerWrapFunction::Repeat);
        let perspective = cgmath::perspective(
            cgmath::Rad::from(cgmath::Deg(90f64)),
            self.context.get_aspect_ratio(),
            0.1f64,
            1024f64,
        )
        .cast::<f32>() // Casts internal f64 to f32, since 'double' support in video grahics card is fairly recent...
        .expect("Couldn't cast Perspective f64 to f32");
        // .into();

        let view = self
            .camera
            .get_view()
            .cast::<f32>()
            .expect("Couldn't cast View f64 to f32");
            // .into();

        let frustum = Frustum::from_matrix4((perspective * view).into()).expect("No frustum!");
        let view: [[f32; 4]; 4] = view.into();
        let perspective: [[f32; 4]; 4] = perspective.into();

        self.terrain_manager.mesh_chunks(self.context.get_display());
        for mesh_ref in self.terrain_manager.get_meshes().iter(){
            let (position, mesh) = mesh_ref.pair();
            let model_position = Point3::new(position.x as f32, position.y as f32, position.z as f32) * CHUNKSIZE as f32;
            let aabb = Aabb3::new(model_position, model_position + Vector3::new(CHUNKSIZE as f32, CHUNKSIZE as f32, CHUNKSIZE as f32));
            if frustum.contains(&aabb) == Relation::Out{
                continue;
            }

            let model: [[f32; 4]; 4] = cgmath::Matrix4::from_translation([model_position.x , model_position.y, model_position.z].into())
            .into();
            let uniforms = uniform! {
                m: model,
                v: view,
                p: perspective,
                t: texture
            };

            self.context.draw(mesh.get_vb(), mesh.get_ib(), &uniforms);
        }

        let position = self.camera.get_position();
        let front = self.camera.get_front();
        let ray = raycast(position, front, 8., |p, f| { if p.map(|p| p.trunc()) == Point3::new(0., 0., 0.){ true }else{ false} });
        if let Some((position, facing)) = ray{
            use crate::engine::mesh::MeshData;
            use crate::game::terrain::block::Direction;
            let mut selected = MeshData::new();
            let direction = Direction::from(facing.cast::<f32>().expect("no"));
            selected.add_face(position.cast::<f32>().expect("oh no"), direction, [0, 0]);
            let mesh = selected.build(self.context.get_display());

            let model: [[f32; 4]; 4] = cgmath::Matrix4::from_translation(facing.cast::<f32>().expect("no no")/10.)//[position.x as f32, position.y as f32, position.z as f32].into())
            .into();
            let uniforms = uniform! {
                m: model,
                v: view,
                p: perspective,
                t: texture
            };
            self.context.draw(mesh.get_vb(), mesh.get_ib(), &uniforms);
        }

        self.context.draw_ui();

        self.context.finish_frame();
    }
}
