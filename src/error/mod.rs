// MIT/Apache2 License

use core::fmt;

mod x11;
pub use x11::X11Error;

/// Container for all errors that can happen.
#[derive(Debug)]
pub enum Error {
    /// An X11 error has occurred.
    X11(X11Error),
    /// More than one runtime cannot be created.
    RuntimeDuplication,
}

impl From<X11Error> for Error {
    #[inline]
    fn from(x: X11Error) -> Error {
        Self::X11(x)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X11(ref x) => fmt::Display::fmt(x, f),
            Self::RuntimeDuplication => {
                f.write_str("Runtimes cannot be duplication without using the alloc library.")
            }
        }
    }
}

/// Result type, for convenience.
pub type Result<T> = core::result::Result<T, Error>;
