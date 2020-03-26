use crate::engine::renderer::Context;
use crate::game::ecs::ECSManager;
use crate::game::registry::Registry;
use crate::game::terrain::chunk::{ChunkPosition, CHUNKSIZE};
use crate::game::terrain::manager::TerrainManager;
use crate::engine::utils::camera::Camera;
use crate::engine::utils::texture::TextureStorage;
use crate::engine::utils::clock::*;
use crate::engine::utils::raycast::VoxelRay;

use crate::game::ecs::components;
use crate::game::ecs::systems::*;
use cgmath::{Point3, Vector3, Zero};
use collision::{Frustum, Aabb3, Relation, Ray3};
use collision::prelude::{Contains, Discrete};
use specs::prelude::*;

use std::path::Path;
use std::sync::Arc;

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

        let camera = Camera::new([8., 0., 0.]); //, DEFAULT_WIDTH as f64/ DEFAULT_HEIGHT as f64);
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
        self.timer.readjust();

        self.handle_input();

        while self.timer.should_update() {
            self.update();
            self.timer.update();
        }

        self.render();
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
                    glium::glutin::WindowEvent::MouseInput { state, button, .. } => {
                        if *state == glium::glutin::ElementState::Released{
                            let position = self.camera.get_position().cast::<f32>().expect("f64 to f32 failed");
                            let front = self.camera.get_front().cast::<f32>().expect("f64 to f32 failed");
                            let mut ray = VoxelRay::new(position, position+front, 8);

                            if let Some((mut position, face)) = ray.until(|b, _f| {
                                if let Some((block, _)) = self.terrain_manager.block_at(b.x, b.y, b.z){
                                    if block != 0  { return true }
                                }
                                false
                            }){
                                let replacer = if *button == glium::glutin::MouseButton::Right{
                                    position += face.cast::<f32>().expect("Couldn't cast f64 to f32");
                                    2
                                }else if *button == glium::glutin::MouseButton::Left{
                                    0
                                }else { 0 };
                                // let c_pos = ChunkPosition::from_world(position.x, position.y, position.z);

                                self.terrain_manager.set_block(position.x, position.y, position.z, replacer);
                            }
                        }
                    },
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

    pub fn render(&mut self) {
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

        let projection = perspective * view;
        let frustum = Frustum::from_matrix4(projection.into()).expect("No frustum!");
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
        let position = position.cast::<f32>().expect("Failed to cast Position to f32");
        let front = front.cast::<f32>().expect("Failed to cast Front to f32");
        let mut ray = VoxelRay::new(position, position+front, 8);
        let r_pos = ray.position;
        let r_dir = ray.direction;

        if let Some((position, _face)) = ray.until(|b, f| {
            if let Some((block, data)) = self.terrain_manager.block_at(b.x, b.y, b.z){
                if block != 0{ //not air
                    let mesh_id = data.get_mesh();
                    if mesh_id != 0{ // not block
                        let hitbox = self.registry.mesh_registry().by_id(mesh_id).expect("Couldn't retrieve mesh").get_hitbox();
                        let pos_v = Vector3::new(b.x.trunc(), b.y.trunc(), b.z.trunc());
                        // println!("Hitbox: {:?} Point: {:?}", Aabb3::new(hitbox.min + pos_v, hitbox.max + pos_v), (b - (f.cast::<f32>().expect("i8 to f32 failed")/100.)));
                        let inf_ray = Ray3::new(r_pos, r_dir);
                        let intersects = inf_ray.intersects(&Aabb3::new(hitbox.min + pos_v, hitbox.max + pos_v));
                        if intersects{
                            return true
                        }else{
                            return false
                        }
                    }
                    return true;
                }
            }
            false
        }){
            if let Some((_block, data)) = self.terrain_manager.block_at(position.x, position.y, position.z){
                // let mut selected = MeshData::new();
                let hitbox = self.registry.mesh_registry().by_id(data.get_mesh()).expect("Couldn't retrieve mesh").get_hitbox();
                let position = Vector3::new(position.x.trunc(), position.y.trunc(), position.z.trunc());
                let model: [[f32; 4]; 4] = cgmath::Matrix4::from_translation(Vector3::new(0., 0., 0.))
                .into();
                let uniforms = uniform! {
                    m: model,
                    v: view,
                    p: perspective,
                    t: texture
                };
                self.context.draw_hitbox(hitbox.min + position, hitbox.max + position, [0., 0., 0., 1.], &uniforms);

            }
        }

        let model: [[f32; 4]; 4] = cgmath::Matrix4::from_translation([0., 0., 0.].into())
        .into();
        let uniforms = uniform! {
            m: model,
            v: view,
            p: perspective
        };

        let start = (position+front).cast::<f32>().expect("nono");
        let end = (position+front*8.).cast::<f32>().expect("no2");
        self.context.draw_line(start, end + Vector3::new(0.5, 0., 0.), [1., 0., 0., 1.], &uniforms); // x - red
        self.context.draw_line(start, end + Vector3::new(0., 0.5, 0.), [0., 1., 0., 1.], &uniforms); // y - green
        self.context.draw_line(start, end + Vector3::new(0., 0., 0.5), [0., 0., 1., 1.], &uniforms); // z - blue

        // self.context.draw_ui();
        self.context.finish_frame();

    }
}
