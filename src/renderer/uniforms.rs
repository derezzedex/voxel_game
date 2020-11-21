use ultraviolet::{Mat4, Vec3};
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Uniforms {
    model: Mat4,
    view: Mat4,
    projection: Mat4,
}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            model: Mat4::from_translation(Vec3::new(0., 0., 0.1)),
            view: Mat4::identity(),
            projection: Mat4::identity(),
        }
    }

    pub fn update_view(&mut self, view: Mat4) {
        self.view = view;
    }

    pub fn update_projection(&mut self, projection: Mat4){
        self.projection = projection;
    }

    pub fn update_model(&mut self, model: Vec3) {
        self.model = Mat4::from_translation(model);
    }
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}