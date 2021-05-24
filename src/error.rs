// MIT/Apache2 License

use std::fmt;

#[derive(Debug)]
pub enum Error {
    StaticMsg(&'static str),
    Msg(String),
    RunAfterClose,
    Chalkboard(chalkboard::Error),
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaticMsg(m) => f.write_str(m),
            Self::Msg(m) => f.write_str(m),
            Self::RunAfterClose => f.write_str("Attempted to run() a Display after it was closed"),
            Self::Chalkboard(c) => fmt::Display::fmt(c, f),
        }
    }
}

impl std::error::Error for Error {}

impl<T: Into<chalkboard::Error>> From<T> for Error {
    #[inline]
    fn from(t: T) -> Error {
        Self::Chalkboard(t.into())
    }
}

/// Useful type alias for a Result with this crate's local error.
pub type Result<T = ()> = std::result::Result<T, Error>;
