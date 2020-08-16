use engine::{
    render::{
        renderer::{Renderer, DrawType},
        interface::InterfaceManager,
    },
    utils::{
        MessageChannel,
        timer::Timer,
        debug::DebugInfo,
        camera::{Camera, Direction},
    },
    winit::{
        event::{Event, WindowEvent, DeviceEvent, KeyboardInput, VirtualKeyCode, ElementState},
        event_loop::{EventLoop, ControlFlow},
    },
};

use futures::executor::block_on;
use fern::colors::{Color, ColoredLevelConfig};
use log::info;

mod world;
use world::WorldManager;
use world::manager::MeshPosition;

//TODO: Add State Management (Stack, Push, Pop, Transitions, etc.) [remove `focused` bool]
pub struct Game{
    running: bool,
    focused: bool,
    timer: Timer,
    camera: Camera,

    renderer: Renderer,
    world_manager: WorldManager,
    interface_manager: InterfaceManager,
}

impl Game{
    pub fn new(event_loop: &EventLoop<()>) -> Self{
        let running = true;
        let focused = true;
        let mut renderer = block_on(Renderer::new(event_loop));
        let timer = Timer::new(20); // 20 updates per second!

        let camera = Camera::default();

        let world_manager = WorldManager::new();
        let interface_manager = InterfaceManager::new(&mut renderer);

        Self{
            running,
            focused,
            timer,
            camera,

            renderer,
            world_manager,
            interface_manager,
        }
    }

    pub fn setup(&mut self, channel: MessageChannel<String>){
        self.interface_manager.set_message_channel(channel);
        let device = self.renderer().arc_device().clone();
        self.world_manager.setup(&device);
        self.renderer.window().set_cursor_grab(self.focused).expect("Couldnt grab cursor!");
        self.renderer.window().set_cursor_visible(!self.focused);
    }

    pub fn tick(&mut self){
        self.timer.reset();

        while self.timer.should_update(){
            self.update();
            self.timer.update();
        }
    }

    pub fn update(&mut self){
        self.interface_manager.update_debug("update", 1.);
        self.interface_manager.update();

        self.camera.hard_update(self.timer.delta().as_secs_f32());
        self.renderer.uniforms().update_view(&self.camera);
        self.world_manager.update();
    }

    pub fn render(&mut self){
        // let frametime = std::time::Instant::now();
        self.renderer.start_frame();
        self.renderer.clear();

        for mesh in self.world_manager.meshes().iter(){
            match mesh.key(){
                MeshPosition::ChunkPosition(pos) => self.renderer.draw(DrawType::Opaque, &pos.to_world(), mesh.value()),
                _ => (),
            }
        }
        self.renderer.final_pass();

        self.renderer.draw_interface(&mut self.interface_manager);

        self.renderer.end_frame();
        self.interface_manager.update_debug("frame", 1.);
        // self.interface_manager.update_debug("frametime", frametime.elapsed().as_secs_f64() * 1000.);
    }

    pub fn renderer(&mut self) -> &mut Renderer{
        &mut self.renderer
    }

    pub fn on_key_press(&mut self, key: VirtualKeyCode){
        info!("Pressed {:?}", key);
        match key{
            VirtualKeyCode::W => self.camera.start_moving(Direction::Forward),
            VirtualKeyCode::S => self.camera.start_moving(Direction::Backward),
            VirtualKeyCode::A => self.camera.start_moving(Direction::Left),
            VirtualKeyCode::D => self.camera.start_moving(Direction::Right),
            _ => (),
        }
    }

    pub fn on_key_release(&mut self, key: VirtualKeyCode){
        info!("Released {:?}", key);
        match key{
            VirtualKeyCode::W => self.camera.stop_moving(Direction::Forward),
            VirtualKeyCode::S => self.camera.stop_moving(Direction::Backward),
            VirtualKeyCode::A => self.camera.stop_moving(Direction::Left),
            VirtualKeyCode::D => self.camera.stop_moving(Direction::Right),
            _ => (),
        }
    }

    pub fn process_event(&mut self, event: Event<()>){
        match event{
            Event::DeviceEvent { event, .. } => match event{
                DeviceEvent::MouseMotion{ delta } => {
                    if self.focused{
                        self.camera.mouse_update(delta.0 as f32, delta.1 as f32, self.timer.delta().as_secs_f32());
                    }
                },
                _ => (),
            },
            Event::WindowEvent {
                event,
                window_id,
            } if window_id == self.renderer.window().id() => {
                self.interface_manager.handle_event(&event, self.renderer.window().scale_factor());
                match event{
                    WindowEvent::CloseRequested => self.exit(),
                    WindowEvent::Resized(physical_size) => {
                        self.interface_manager.resize(physical_size, self.renderer.window().scale_factor());
                        self.renderer.resize(physical_size);
                        info!("Resized to {:?}", physical_size);
                        // self.renderer.window().request_redraw();
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.renderer.resize(*new_inner_size);
                        info!("Resized to {:?}", new_inner_size);
                        // self.renderer.window().request_redraw();
                    },
                    WindowEvent::KeyboardInput{ input, .. } => match input{
                        KeyboardInput { virtual_keycode: Some(keycode), state, .. } => {
                            match keycode{
                                // DEBUG
                                VirtualKeyCode::Escape => self.exit(),
                                VirtualKeyCode::Apostrophe => if state == ElementState::Pressed { self.toggle_console() },
                                VirtualKeyCode::F3 => if state == ElementState::Pressed {self.interface_manager.toggle_debug()},
                                // OTHER KEY
                                key => if state == ElementState::Pressed { self.on_key_press(key) } else { self.on_key_release(key) },
                            }
                        },
                        _ => (),
                    },
                    _ => (),
                }

                // self.interface_manager.handle_event(&event);
            },
            _ => (),
        }
    }

    pub fn run(){
        let channel = MessageChannel::new();
        let colors = ColoredLevelConfig::new()
            .debug(Color::Green)
            .info(Color::Cyan)
            .trace(Color::Magenta);
        fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "{}[{}][{}] {}",
                    chrono::Local::now().format("[%H:%M:%S]"),
                    record.target(),
                    colors.color(record.level()),
                    message
                ))
            })
            .level(log::LevelFilter::Off)
            .level_for("client", log::LevelFilter::Trace)
            .level_for("engine", log::LevelFilter::Trace)
            // .chain(std::io::stdout())
            .chain(fern::Output::sender(channel.sender.clone(), ""))
            .apply()
            .expect("Couldn't create logger");

        let event_loop = EventLoop::new();
        let mut game = Game::new(&event_loop);
        game.setup(channel);

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

    pub fn toggle_console(&mut self){
        self.interface_manager.toggle_console();
        self.focused = !self.focused;
        info!("Focused: {}", self.focused);
        self.renderer.window().set_cursor_grab(self.focused);
        self.renderer.window().set_cursor_visible(!self.focused);
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
