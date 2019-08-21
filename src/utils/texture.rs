use std::io::Cursor;
use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub struct TextureCoords{
    top_left: (f32, f32),
    top_right: (f32, f32),
    bottom_left: (f32, f32),
    bottom_right: (f32, f32)
}

impl TextureCoords{
    pub fn new(tl: (f32, f32), tr: (f32, f32), bl: (f32, f32), br: (f32, f32)) -> TextureCoords{
        TextureCoords{
            top_left: tl,
            top_right: tr,
            bottom_left: bl,
            bottom_right: br
        }
    }

    pub fn as_vec(&self) -> [(f32, f32); 4]{
        [self.top_left, self.top_right, self.bottom_left, self.bottom_right]
    }
}

pub struct TextureAtlas{
    texture: glium::texture::texture2d::Texture2d,
    dimensions: (u32, u32),
    tile_size: u32,
}

impl TextureAtlas{
    pub fn new(display: &glium::Display, path: &Path, tile_size: u32) -> TextureAtlas{
        let data = std::fs::read(path).expect("Couldn't read texture!");
        let bytes = Cursor::new(&data[..]);
        let image = image::load(bytes, image::PNG).expect("Couldn't load texture!").to_rgba();
        let dimensions = image.dimensions();

        let raw_texture = glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dimensions);

        TextureAtlas{
            texture: glium::texture::Texture2d::new(display, raw_texture).unwrap(),
            dimensions,
            tile_size
        }
    }

    pub fn get_texture(&self) -> &glium::texture::Texture2d{
        &self.texture
    }

    pub fn get_coords(&self, xy: (u32, u32)) -> TextureCoords{
        let (x, y) = xy;
        let top_left = self.get_top_left(x, y);
        let top_right = self.get_top_right(x, y);
        let bottom_left = self.get_bottom_left(x, y);
        let bottom_right = self.get_bottom_right(x, y);

        TextureCoords{
            top_left,
            top_right,
            bottom_left,
            bottom_right
        }
    }

    pub fn get_top_left(&self, x: u32, y: u32) -> (f32, f32){
        let (w, h) = self.dimensions;
        let nx = (x * self.tile_size) as f32 / w as f32;
        let ny = (y * self.tile_size) as f32 / h as f32;

        (nx, ny)
    }

    pub fn get_top_right(&self, x: u32, y: u32) -> (f32, f32){
        let (w, h) = self.dimensions;
        let nx = (x * self.tile_size + self.tile_size) as f32 / w as f32;
        let ny = (y * self.tile_size) as f32 / h as f32;

        (nx, ny)
    }

    pub fn get_bottom_left(&self, x: u32, y: u32) -> (f32, f32){
        let (w, h) = self.dimensions;
        let nx = (x * self.tile_size) as f32 / w as f32;
        let ny = (y * self.tile_size + self.tile_size) as f32 / h as f32;

        (nx, ny)
    }

    pub fn get_bottom_right(&self, x: u32, y: u32) -> (f32, f32){
        let (w, h) = self.dimensions;
        let nx = (x * self.tile_size + self.tile_size) as f32 / w as f32;
        let ny = (y * self.tile_size + self.tile_size) as f32 / h as f32;

        (nx, ny)
    }
}
