use crate::engine::renderer::Context;
use conrod;
use std::path::Path;
use crate::engine::WinitDisplay;

use conrod::{widget, Borderable, Sizeable, Colorable, Positionable, Widget};

const CONSOLE_TEST: [&str; 32] = ["Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."; 32];

widget_ids!(struct Ids { console, console_text, console_input, console_button, fps });

pub struct Debug{
    show: bool,
    input_text: String,
    last_fps: f64,
    fps: f64,
    ups: f64
}

const SMOOTHING: f64 = 0.9;
impl Debug{
    pub fn show(&mut self){ self.show = true; }
    pub fn hide(&mut self){ self.show = false; }
    pub fn switch(&mut self){ self.show = !self.show; }
    pub fn set_fps(&mut self, fps: f64){
        // self.fps = (self.last_fps * SMOOTHING) + (fps * (1.0 - SMOOTHING));
        // self.last_fps = fps;
        self.fps = fps;
    }

    pub fn set_ups(&mut self, ups: f64){
        self.ups = ups;
    }
}

pub struct Ui{
    pub ui: conrod::Ui,
    ids: Ids,
    renderer: conrod::backend::glium::Renderer,
    image_map: conrod::image::Map<glium::texture::Texture2d>,

    pub debug: Debug
}

impl Ui{
    pub fn new(display: &WinitDisplay, width: f64, height: f64) -> Self{
        // conrod
        let mut ui = conrod::UiBuilder::new([width, height]).build();
        let ids = Ids::new(ui.widget_id_generator());

        let cargo_dir = env!("CARGO_MANIFEST_DIR");
        let font_path = &Path::new(cargo_dir).join("res/fonts/pixel-operator/PixelOperator.ttf");
        ui.fonts.insert_from_file(font_path).expect("Couldn't load fonts into Ui!");

        let mut renderer = conrod::backend::glium::Renderer::new(&display.0).expect("Couldn't create Ui renderer!");

        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();

        let debug = Debug{
            show: false,
            input_text: String::new(),
            last_fps: 0.0,
            fps: 0.0,
            ups: 0.0
        };

        Self{
            ui,
            ids,
            debug,
            renderer,
            image_map
        }
    }

    pub fn handle_event(&mut self, e: conrod::event::Input){
        self.ui.handle_event(e);
    }

    pub fn draw<T: glium::Surface>(&mut self, display: &glium::Display, frame: &mut T){
        let ui = &mut self.ui.set_widgets();
        if self.debug.show{
            let canvas = widget::Canvas::new()
                .place_on_kid_area(true)
                // .pad(100.0)
                .scroll_kids_vertically()
                .rgba(0.3, 0.3, 0.3, 0.7)
                .middle_of(ui.window)
                .set(self.ids.console, ui);

            let console_w = ui.w_of(self.ids.console).expect("Couldn't get console width");
            let console_h = ui.h_of(self.ids.console).expect("Couldn't get console height");
            let (mut texts, scrollbar) = widget::List::flow_down(CONSOLE_TEST.len())
                // .item_size(25.0)
                .scrollbar_next_to()
                .mid_top_of(self.ids.console)
                .w_of(self.ids.console)
                .h(console_h - (console_h * 0.0625)) // canvas height minus 6.25%
                .set(self.ids.console_text, ui);

            while let Some(text) = texts.next(ui){
                let i = text.i;
                let text_widget = widget::Text::new(CONSOLE_TEST[i])
                    .color(conrod::color::WHITE);
                text.set(text_widget, ui);
            }

            if let Some(s) = scrollbar{
                s.set(ui);
            }

            for event in widget::TextBox::new(&mut self.debug.input_text)
                .parent(self.ids.console)
                .bottom_left()
                .w(console_w * 0.9)
                .h(console_h * 0.0625)
                .text_color(conrod::color::WHITE)
                .rgba(0.3, 0.3, 0.3, 0.9)
                .set(self.ids.console_input, ui){
                    if let conrod::widget::text_box::Event::Update(text) = event{
                        self.debug.input_text = text;
                    }
                }
        }

        widget::Text::new(&format!("framerate: {:.2}", self.debug.fps))
            .top_left_with_margins_on(ui.window, 20.0, 5.0)
            .color(conrod::color::BLACK)
            .font_size(16)
            .set(self.ids.fps, ui);


        self.renderer.fill(display, ui.draw(), &self.image_map);
        self.renderer.draw(display, frame, &self.image_map).expect("Couldn't draw using Ui renderer!");
    }
}
