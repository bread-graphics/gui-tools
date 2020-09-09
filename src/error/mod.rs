// MIT/Apache2 License

use core::{fmt, num::TryFromIntError};

mod x11;
#[cfg(target_os = "linux")]
pub(crate) use x11::x11_status_to_res;
pub use x11::X11Error;

mod win32;
#[cfg(windows)]
pub(crate) use win32::win32error;
pub use win32::Win32Error;

/// Container for all errors that can happen.
///
/// For convenience, all functions in `gui-tools` that can fail in some way return this
/// `Error` type. It contains a collection of every error that can occur.
#[derive(Debug)]
pub enum Error {
    /// An unsupported operation has occurred on the core version.
    CoreUnsupported(&'static str),
    /// An X11 error has occurred.
    X11(X11Error),
    /// A Win32 error has occurred.
    Win32(Win32Error),
    /// An integer conversion error has occurred.
    TryFromInt(TryFromIntError),
    /// An invalid value was set to a color element.
    InvalidColorElement(f32),
    /// No usable backend was found.
    NoBackendFound,
    /// A method from the No-op backend was called.
    NoOpFunctionCalled,
    /// More than one runtime cannot be created.
    RuntimeDuplication,
}

impl From<X11Error> for Error {
    #[inline]
    fn from(x: X11Error) -> Error {
        Self::X11(x)
    }
}

impl From<Win32Error> for Error {
    #[inline]
    fn from(w: Win32Error) -> Error {
        Self::Win32(w)
    }
}

impl From<TryFromIntError> for Error {
    #[inline]
    fn from(tfi: TryFromIntError) -> Error {
        Self::TryFromInt(tfi)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CoreUnsupported(s) => f.write_str(s),
            Self::X11(ref x) => fmt::Display::fmt(x, f),
            Self::Win32(ref w) => fmt::Display::fmt(w, f),
            Self::TryFromInt(ref tfi) => fmt::Display::fmt(tfi, f),
            Self::InvalidColorElement(fl) => write!(f, "Invalid color element: {}", fl),
            Self::NoBackendFound => {
                f.write_str("Unable to find an applicable backend for the runtime")
            }
            Self::NoOpFunctionCalled => {
                f.write_str("A function belonging to a non-existent backend was called")
            }
            Self::RuntimeDuplication => {
                f.write_str("Runtimes cannot be duplication without using the alloc library.")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

/// Result type, for convenience.
pub type Result<T> = core::result::Result<T, Error>;
