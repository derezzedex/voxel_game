use engine::{
    render::{
        renderer::{DrawType, Renderer},
        mesh::{Mesh, Vertex},
    },
    utils::{
        timer::Timer,
        debug::DebugInfo,
        camera::Camera,
    },
    winit::{
        event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode},
        event_loop::{EventLoop, ControlFlow},
    },
};

use dashmap::DashMap;
use futures::executor::block_on;
use log::info;

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct MeshPosition{
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

pub struct Game{
    running: bool,
    renderer: Renderer,
    timer: Timer,

    camera: Camera,
    meshes: DashMap<MeshPosition, Mesh>,

    debug: DebugInfo,
}

impl Game{
    pub fn new(event_loop: &EventLoop<()>) -> Self{
        let running = true;
        let renderer = block_on(Renderer::new(event_loop));
        let timer = Timer::new(20); // 20 updates per second!

        let camera = Camera::default();
        let meshes = DashMap::new();

        let debug = DebugInfo::new();

        Self{
            running,
            renderer,
            timer,

            camera,
            meshes,

            debug,
        }
    }

    pub fn setup(&mut self){
        let vertices = vec![
            //Quad2
            Vertex { position: [ 0.5, -0.5, 1.], tex_coord: [0., 1.], },
            Vertex { position: [ 1.5, -0.5, 1.], tex_coord: [1., 1.], },
            Vertex { position: [ 0.5,  0.5, 1.], tex_coord: [0., 0.], },
            Vertex { position: [ 1.5,  0.5, 1.], tex_coord: [1., 0.], },

            //Quad1
            Vertex { position: [-0.5, -0.5, 0.], tex_coord: [0., 1.], },
            Vertex { position: [ 0.5, -0.5, 0.], tex_coord: [1., 1.], },
            Vertex { position: [-0.5,  0.5, 0.], tex_coord: [0., 0.], },
            Vertex { position: [ 0.5,  0.5, 0.], tex_coord: [1., 0.], },
        ];
        let indices = vec![
            0, 1, 2,
            2, 1, 3,

            4, 5, 6,
            6, 5, 7
        ];

        let mesh = Mesh::new(self.renderer().device(), vertices, indices);
        let position = MeshPosition{ x: -5, y: 0, z: -8 };
        self.meshes.insert(position, mesh);
    }

    pub fn tick(&mut self){
        self.timer.reset();
        self.debug = self.debug.update();

        while self.timer.should_update(){
            self.update();
            self.timer.update();
        }
    }

    pub fn update(&mut self){
        self.debug.updates += 1;
        self.debug.total_updates += 1;

        self.camera.hard_update(self.timer.delta().as_secs_f32());
        self.renderer.uniforms().update_view(&self.camera);
    }

    pub fn render(&mut self){
        self.debug.frames += 1;
        self.renderer.start_frame();
        self.renderer.clear();

        // render meshes
        for map_ref in self.meshes.iter(){
            let (position, mesh) = (map_ref.key(), map_ref.value());
            let position_vec = glam::Vec3::new(position.x as f32, position.y as f32, position.z as f32);
            self.renderer.draw(DrawType::Transparent, &position_vec, mesh);
        }

        self.renderer.final_pass();

        self.renderer.end_frame();
    }

    pub fn renderer(&mut self) -> &mut Renderer{
        &mut self.renderer
    }

    pub fn process_event(&mut self, event: Event<()>){
        match event{
            Event::WindowEvent {
                event,
                window_id,
            } if window_id == self.renderer.window().id() => {
                match event{
                    WindowEvent::CloseRequested => self.exit(),
                    WindowEvent::Resized(physical_size) => {
                        self.renderer.resize(physical_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.renderer.resize(*new_inner_size);
                    },
                    WindowEvent::KeyboardInput{ input, .. } => match input{
                        KeyboardInput { virtual_keycode: Some(keycode), .. } => {
                            match keycode{
                                VirtualKeyCode::Escape => self.exit(),
                                _ => (),
                            }
                        },
                        _ => (),
                    },
                    _ => (),
                }
            },
            _ => (),
        }
    }

    pub fn run(){
        let event_loop = EventLoop::new();
        let mut game = Game::new(&event_loop);
        game.setup();

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::RedrawRequested(_) => {
                    game.render();
                },
                Event::MainEventsCleared => {
                    game.tick();
                    game.renderer().window().request_redraw();
                },
                _ => game.process_event(event),
            }

            if game.is_running(){
                *control_flow = ControlFlow::Poll;
            }else{
                *control_flow = ControlFlow::Exit;
            }
        });
    }

    pub fn is_running(&self) -> bool{
        self.running
    }

    pub fn exit(&mut self){
        self.running = false;
    }

    pub fn get_renderer(&self) -> &Renderer{
        &self.renderer
    }
}
