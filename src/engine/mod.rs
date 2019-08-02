#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2]
}

impl Vertex{
    pub const fn new(position: [f32; 3], tex_coord: [f32; 2]) -> Vertex{
        Vertex{
            position,
            tex_coord
        }
    }
}

implement_vertex!(Vertex, position, tex_coord);

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
