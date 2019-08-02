use crate::engine::renderer::Context;
use crate::utils::timer::*;

pub struct Game{
    context: Context,
    timer: UpdateTimer,
    running: bool
}

impl Game{
    pub fn new(title: &str) -> Self{
        let context = Context::new(title, "shaders/vertex.glsl", "shaders/fragment.glsl");
        let timer = UpdateTimer::new(16);
        let running = true;

        Self{
            context,
            timer,
            running
        }
    }

    pub fn run(&mut self){
        while self.running{
            self.tick();
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
    }

    pub fn handle_input(&mut self){
        let events = self.context.poll_events();
        for event in &events{

            if let Some(event) = conrod::backend::winit::convert_event(event.clone(), &self.context.display){
                self.context.ui.handle_event(event, to_secs(self.timer.elapsed));
            }

            match event{
                glium::glutin::Event::WindowEvent { event, .. } => match event{
                    glium::glutin::WindowEvent::CloseRequested => self.running = false,
                    _ => (),
                }
                _ => (),
            }
        }
    }

    pub fn update(&mut self){

    }

    pub fn render(&mut self){
        self.context.new_frame();

        self.context.clear_color([0.3, 0.45, 0.65, 1.0]);

        self.context.draw_ui();

        self.context.finish_frame();
    }
}
