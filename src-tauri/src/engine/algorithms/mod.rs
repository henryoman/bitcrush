pub mod standard;
pub mod enhanced;
pub mod artistic;

use image::{ImageBuffer, Rgba};

pub type RgbaImage = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub trait Algorithm {
    fn process(&self, img: &mut RgbaImage, palette: &[[u8;3]]);
}

pub fn get_algorithm_by_name(name: &str) -> Box<dyn Algorithm + Send + Sync> {
    match name {
        "Enhanced" => Box::new(enhanced::Enhanced),
        "Artistic" => Box::new(artistic::Artistic),
        _ => Box::new(standard::Standard),
    }
}

