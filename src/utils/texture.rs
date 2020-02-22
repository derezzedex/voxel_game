use std::io::Cursor;
use std::path::Path;
use image::GenericImageView;

pub type TextureArray = glium::texture::srgb_texture2d_array::SrgbTexture2dArray;
pub type RawImage<'a, T> = glium::texture::RawImage2d<'a, T>;

#[allow(dead_code)]
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
