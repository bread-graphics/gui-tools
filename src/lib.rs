// MIT/Apache2 License

//! This crate simply acts as an abstraction over a variety of graphical user interface (GUI) libraries for a
//! variety of platforms. `gui-tools` aims to be thread-safe, low-contention and versatile, but most of all tries
//! to put as few layers between the user and the actual API as possible.

#![forbid(unsafe_code)]

#[cfg(unix)]
pub mod breadx;

pub mod display;
pub mod screen;
pub mod window;

pub(crate) mod init;

pub use display::*;
pub use screen::*;
pub use window::*;

#[doc(inline)]
pub use chalkboard::{Error, Result};
