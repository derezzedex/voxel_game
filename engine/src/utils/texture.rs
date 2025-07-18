use glium::glutin::surface::WindowSurface;
use glium::texture::RawImage2d;
use glium::texture::TextureCreationError;
use glium::texture::srgb_texture2d_array::SrgbTexture2dArray;
use glium::uniforms::MagnifySamplerFilter;
use glium::uniforms::Sampler;
use glium::uniforms::SamplerWrapFunction;
use image::GenericImageView;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::Path;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Image(image::ImageError),
    Graphics(TextureCreationError),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IO(error)
    }
}

impl From<image::ImageError> for Error {
    fn from(error: image::ImageError) -> Self {
        Self::Image(error)
    }
}

impl From<TextureCreationError> for Error {
    fn from(error: TextureCreationError) -> Self {
        Self::Graphics(error)
    }
}

pub struct Array {
    raw: SrgbTexture2dArray,
}

impl Array {
    pub fn from_atlas(
        display: &glium::Display<WindowSurface>,
        path: &Path,
        kind: image::ImageFormat,
        tile_size: u32,
    ) -> Result<Self, Error> {
        let file = File::open(path)?;
        let bytes = BufReader::new(file);
        let image = image::load(bytes, kind)?.to_rgba8();

        let (width, height) = image.dimensions();
        let total = (width / tile_size) * (height / tile_size);
        let mut textures = Vec::with_capacity(total as usize);

        for x in 0..(width / tile_size) {
            for y in 0..(height / tile_size) {
                let sub_image = image
                    .view(x * tile_size, y * tile_size, tile_size, tile_size)
                    .to_image();
                let texture = RawImage2d::from_raw_rgba_reversed(
                    &sub_image.into_raw(),
                    (tile_size, tile_size),
                );
                textures.push(texture);
            }
        }

        let raw = SrgbTexture2dArray::with_mipmaps(
            display,
            textures,
            glium::texture::MipmapsOption::NoMipmap,
        )?;

        Ok(Self { raw })
    }

    pub fn sampler(&self) -> Sampler<'_, SrgbTexture2dArray> {
        self.raw
            .sampled()
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .wrap_function(SamplerWrapFunction::Repeat)
    }
}
