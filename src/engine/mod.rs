#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3]
}

impl Vertex{
    pub const fn new(position: [f32; 3], color: [f32; 3]) -> Vertex{
        Vertex{
            position,
            color
        }
    }

    // pub fn set_coords(&mut self, tex_coord: [f32; 2]){
    //     self.tex_coord = tex_coord;
    // }
}

implement_vertex!(Vertex, position, color);

pub struct WinitDisplay(pub glium::Display);

impl conrod::backend::winit::WinitWindow for WinitDisplay {
    fn get_inner_size(&self) -> Option<(u32, u32)> {
        self.0.gl_window().window().get_inner_size().map(Into::into)
    }
    fn hidpi_factor(&self) -> f32 {
        self.0.gl_window().window().get_hidpi_factor() as _
    }
}

// conrod::backend::winit::conversion_fns!();

pub mod renderer;
pub mod ui;
pub mod mesh;
