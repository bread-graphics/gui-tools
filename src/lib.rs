// MIT/Apache2 License

//! `gui-tools` is a toolkit consisting of several backends to native libraries used to create windows,
//! safe Rust wrappers around those windows, a runtime to manage events from these libraries, and a
//! drawing API that provides consistent results for each of these backends.
//!
//! ## Supported Backends
//!
//! * xlib - The X11 library commonly used on Unix and Unix-like platforms. Should work for Linux, as well as
//! BSD variants and possibly MacOS.
//! * win32 - The Win32 API for Windows that is also available via Wine and ReactOS. Should work for Windows,
//! minimum version has not been tested.
//!
//! ## Examples
//!
//! TODO: examples
//!
//! ## Features
//!
//! * `std` (enabled by default) - Enables use of the Rust standard library. This enables the use of system
//!    mutexes, rather than slower, less efficient spinlock mutexes. In addition, this feature is often
//!    used as a gate to other features. Requires the `alloc` feature.
//! * `alloc` (enabled by default) - Enables the use of an allocator. This enables the use of heap memory in
//!   several `gui-tools` structures, which will be used instead of stack-based collections. This also allows
//!   multiple runtimes to be instantiated.
//!
//! * `async` - Feature is currently in progress.
//! * `pl` - Enables use of `parking_lot` mutexes and read/write locks. This allows many `gui-tools` structures
//!    to reduce their size. Requires the `std` feature.

#![no_std]
#![feature(const_fn)]
#![feature(const_fn_union)]
#![feature(maybe_uninit_uninit_array)]
#![warn(clippy::pedantic)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod backend;
pub mod color;
pub mod error;
pub mod event;
pub mod geometry;
pub mod graphics;
pub mod image;
pub mod keyboard;
pub mod monitor;
pub mod mouse;
pub(crate) mod mutex;
pub mod runtime;
pub mod string;
pub mod surface;

#[path = "../tutorials/mod.rs"]
pub mod tutorials;

pub use error::*;
