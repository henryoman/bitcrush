use super::{Algorithm, RgbaImage};

#[derive(Debug, Clone, Copy)]
pub struct Standard;

impl Algorithm for Standard {
    fn name(&self) -> &'static str {
        "Standard"
    }

    fn process(&self, _img: &mut RgbaImage) {
        // No-op for now; placeholder for parity with TS "Standard"
    }
}


