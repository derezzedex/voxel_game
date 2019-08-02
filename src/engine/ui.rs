use crate::engine::renderer::Context;
use conrod;
use std::path::Path;
use crate::engine::WinitDisplay;

use conrod::{widget, Colorable, Positionable, Widget};

widget_ids!(struct Ids { text });

pub struct Ui{
    pub ui: conrod::Ui,
    ids: Ids,
    renderer: conrod::backend::glium::Renderer,
    image_map: conrod::image::Map<glium::texture::Texture2d>
}

impl Ui{
    pub fn new(display: &WinitDisplay, width: f64, height: f64) -> Self{
        // conrod
        // let mut theme = conrod::Theme::default();
        // theme.background_color = conrod::color::LIGHT_GRAY;

        let mut ui = conrod::UiBuilder::new([width, height]).build();
        let ids = Ids::new(ui.widget_id_generator());

        let cargo_dir = env!("CARGO_MANIFEST_DIR");
        let font_path = &Path::new(cargo_dir).join("res/fonts/roboto/Roboto-Regular.ttf");
        ui.fonts.insert_from_file(font_path).expect("Couldn't load fonts into Ui!");

        let mut renderer = conrod::backend::glium::Renderer::new(&display.0).expect("Couldn't create Ui renderer!");

        let image_map = conrod::image::Map::<glium::texture::Texture2d>::new();
        Self{
            ui,
            ids,
            renderer,
            image_map
        }
    }

    pub fn handle_event(&mut self, e: conrod::event::Input, t: f32){
        self.ui.handle_event(e);

        let ui = &mut self.ui.set_widgets();
        widget::Text::new(&format!("frametime: {:.1}", t))
            .top_left_of(ui.window)
            .color(conrod::color::BLACK)
            .font_size(16)
            .set(self.ids.text, ui);
    }

    pub fn draw<T: glium::Surface>(&mut self, display: &glium::Display, frame: &mut T){
        self.renderer.fill(display, self.ui.draw(), &self.image_map);
        self.renderer.draw(display, frame, &self.image_map).expect("Couldn't draw using Ui renderer!");
    }
}
