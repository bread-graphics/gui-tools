// MIT/Apache2 License

#![feature(min_specialization)]

#[cfg(feature = "breadx")]
pub mod breadx;
pub mod display;
pub mod event;
pub mod screen;
pub mod surface;
pub mod window;
#[cfg(windows)]
pub mod yaww;

mod error;
pub use error::*;

pub(crate) mod util;

pub mod prelude {
    pub use crate::display::{Display, DisplayExt, DisplayExtOwned};
}
