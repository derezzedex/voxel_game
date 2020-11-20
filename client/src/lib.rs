use engine::{
    render::{
        renderer::Renderer,
    },
    utils::{
        timer::Timer,
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

//TODO: Add State Management (Stack, Push, Pop, Transitions, etc.) [remove `focused` bool]
pub struct Game{
    running: bool,
    focused: bool,
    timer: Timer,
    camera: Camera,

    renderer: Renderer,
}

impl Game{
    pub fn new(event_loop: &EventLoop<()>) -> Self{
        let running = true;
        let focused = true;
        let renderer = block_on(Renderer::new(event_loop));
        let timer = Timer::new(20); // 20 updates per second!

        let camera = Camera::default();

        Self{
            running,
            focused,
            timer,
            camera,

            renderer,
        }
    }

    pub fn setup(&mut self){
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
        self.camera.hard_update(self.timer.delta().as_secs_f32());
        self.renderer.uniforms().update_view(&self.camera);
    }

    pub fn render(&mut self){
        // let frametime = std::time::Instant::now();
        self.renderer.start_frame();
        self.renderer.clear();

        self.renderer.final_pass();
        self.renderer.end_frame();
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
                match event{
                    WindowEvent::CloseRequested => self.exit(),
                    WindowEvent::Resized(physical_size) => {
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
            .chain(std::io::stdout())
            .apply()
            .expect("Couldn't create logger");

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
