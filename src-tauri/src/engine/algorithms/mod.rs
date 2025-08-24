pub mod standard;
pub mod enhanced;
pub mod floyd_steinberg;
pub mod bayer;

use image::{ImageBuffer, Rgba};

pub type RgbaImage = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub trait Algorithm {
    fn name(&self) -> &'static str;
    fn process(&self, img: &mut RgbaImage, palette: &[[u8;3]]);
}

pub fn get_algorithm_by_name(name: &str) -> Box<dyn Algorithm + Send + Sync> {
    match name {
        "Enhanced" => Box::new(enhanced::Enhanced),
        "Floyd-Steinberg" | "Floyd–Steinberg" => Box::new(floyd_steinberg::FloydSteinberg),
        "Bayer" => Box::new(bayer::Bayer),
        _ => Box::new(standard::Standard),
    }
}

