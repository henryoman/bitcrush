use super::{Algorithm, RgbaImage};

#[derive(Debug, Clone, Copy)]
pub struct Standard;

impl Algorithm for Standard {
    fn process(&self, _img: &mut RgbaImage, _palette: &[[u8;3]]) {
        // No-op for now; placeholder for parity with TS "Standard"
    }
}


