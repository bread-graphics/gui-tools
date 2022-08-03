// This file is part of gui-tools.
//
// gui-tools is free software: you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option)
// any later version.
//
// gui-tools is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty
// of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
// See the GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General
// Public License along with gui-tools. If not, see
// <https://www.gnu.org/licenses/>.

//! `gui-tools` is a library that provides an abstracted windowing
//! system for GUI development.

#![no_std]
#![forbid(unsafe_code, rust_2018_idioms)]

#[macro_use]
mod gates;

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod async_runtime;

mod error;
pub use error::{Error, Result};
mod window;
pub use window::Window;
