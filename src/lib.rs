// MIT/Apache2 License

#![no_std]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(maybe_uninit_uninit_array)]
#![feature(ptr_internals)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod backend;
pub mod error;
pub mod event;
pub mod geometry;
pub mod monitor;
pub(crate) mod mutex;
pub mod runtime;
pub mod string;
pub mod surface;

pub use error::*;
