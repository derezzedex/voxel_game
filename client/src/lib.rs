use futures::executor::block_on;
use log::{info};

use engine::{
    render::Renderer,
    utils::{
        timer::Timer,
        debug::DebugInfo,
    },
    winit::{
        event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode},
        event_loop::{EventLoop, ControlFlow},
    },
};

pub struct Game{
    running: bool,
    renderer: Renderer,
    timer: Timer,

    debug: DebugInfo,
}

impl Game{
    pub fn new(event_loop: &EventLoop<()>) -> Self{
        let running = true;
        let renderer = block_on(Renderer::new(event_loop));
        let timer = Timer::new(20); // 20 updates per second!

        let debug = DebugInfo::new();

        Self{
            running,
            renderer,
            timer,

            debug,
        }
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
    }

    pub fn render(&mut self){
        self.debug.frames += 1;
    }

    pub fn run(){
        let event_loop = EventLoop::new();
        let mut game = Game::new(&event_loop);

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::RedrawRequested(_) => {
                    game.render();
                },
                Event::MainEventsCleared => {
                    game.tick();
                },
                Event::WindowEvent {
                    event,
                    window_id,
                } if window_id == game.get_renderer().get_window().id() => {
                    match event{
                        WindowEvent::CloseRequested => game.exit(),
                        WindowEvent::KeyboardInput{ input, .. } => match input{
                            KeyboardInput { virtual_keycode: Some(keycode), .. } => {
                                match keycode{
                                    VirtualKeyCode::Escape => game.exit(),
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
