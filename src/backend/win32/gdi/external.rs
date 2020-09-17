// MIT/Apache2 License

use crate::color::Rgba;

#[path = "ffi.rs"]
mod ffi;

pub use ffi::*;

impl From<Rgba> for GDIPColor {
    #[inline]
    fn from(rgba: Rgba) -> GDIPColor {
        let (r, g, b, a) = rgba.convert_elements::<u8>();
        GDIPColor { r, g, b, a }
    }
}
