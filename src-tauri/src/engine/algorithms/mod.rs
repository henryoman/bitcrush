pub mod standard;

use image::{ImageBuffer, Rgba};

pub type RgbaImage = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub trait Algorithm {
    fn name(&self) -> &'static str;
    fn process(&self, img: &mut RgbaImage);
}

pub fn get_algorithm_by_name(name: &str) -> Box<dyn Algorithm + Send + Sync> {
    match name {
        _ => Box::new(standard::Standard),
    }
}

