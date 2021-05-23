// MIT/Apache2 License

#[derive(Debug)]
pub enum Error {
    StaticMsg(&'static str),
    Msg(String),
    RunAfterClose,
    Chalkboard(chalkboard::Error),
}

impl<T: Into<chalkboard::Error>> From<T> for Error {
    #[inline]
    fn from(t: T) -> Error {
        Self::Chalkboard(t.into())
    }
}

pub type Result<T = ()> = std::result::Result<T, Error>;
