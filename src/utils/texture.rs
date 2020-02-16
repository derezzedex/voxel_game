use std::io::Cursor;
use std::path::Path;
use image::GenericImageView;

#[derive(Copy, Clone, Debug)]
pub struct TextureCoords{
    pub offset: [u8; 2],
    pub top_left: (f32, f32),
    pub top_right: (f32, f32),
    pub bottom_left: (f32, f32),
    pub bottom_right: (f32, f32)
}

impl TextureCoords{
    pub fn new(offset: [u8; 2], tl: (f32, f32), tr: (f32, f32), bl: (f32, f32), br: (f32, f32)) -> TextureCoords{
        TextureCoords{
            offset,
            top_left: tl,
            top_right: tr,
            bottom_left: bl,
            bottom_right: br
        }
    }

    pub fn as_vec(&self) -> [(f32, f32); 4]{
        [self.top_left, self.top_right, self.bottom_left, self.bottom_right]
    }

    pub fn greedy_ready(&self) -> [[f32; 2]; 4]{
        [
            [self.bottom_left.0,    self.bottom_left.1],
            [self.bottom_right.0,   self.bottom_right.1],
            [self.top_left.0,       self.top_left.1],
            [self.top_right.0,      self.top_right.1]
        ]
    }
}

pub type TextureArray = glium::texture::texture2d_array::Texture2dArray;
pub type RawImage<'a, T: Clone + 'a> = glium::texture::RawImage2d<'a, T>;

pub struct TextureStorage{
    texture_array: TextureArray,
    image_dimensions: (u32, u32),
    tile_size: u32,
}

impl TextureStorage{
    pub fn new(display: &glium::Display, image_path: &Path, image_type: image::ImageFormat, tile_size: u32) -> Self{
        let cargo = env!("CARGO_MANIFEST_DIR");
        let path = Path::new(cargo).join(image_path);
        println!("Creating texture array from: {:?}", path);

        let data = std::fs::read(path).expect("Couldn't read image!");
        let bytes = Cursor::new(&data[..]);
        let image = image::load(bytes, image_type).expect("Couldn't load image!").to_rgba();
        let image_dimensions = image.dimensions();
        let mut textures = Vec::new();

        //load sprites from image as atlas
        for x in 0..(image_dimensions.0/tile_size){
            for y in 0..(image_dimensions.1/tile_size){
                let sub_image = image.view(x*tile_size, y*tile_size, tile_size, tile_size).to_image();
                // sub_image.save(format!("C:\\Users\\derezzedex\\Pictures\\atlas\\{}_{}.png", x, y));
                let texture = RawImage::from_raw_rgba_reversed(&sub_image.into_raw(), (tile_size, tile_size));
                textures.push(texture);
            }
        }

        let texture_array = TextureArray::new(display, textures).unwrap();

        Self{
            texture_array,
            image_dimensions,
            tile_size
        }
    }

    pub fn get_array(&self) -> &TextureArray{
        &self.texture_array
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
        let texture = glium::texture::Texture2d::new(display, raw_texture).unwrap();

        // let test = TextureStorage::new(display, path, image::PNG, tile_size);

        TextureAtlas{
            texture,
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
            offset: [x as u8, y as u8],
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
