use failure::{Fail};
use std::io::{Error, ErrorKind};

#[derive(Debug, Fail)]
pub enum TwinkleError {
    #[fail(display = "parse error")]
    ParseError,
    #[fail(display = "something wrong")]
    SomethingWrong,
}

impl From<TwinkleError> for Error {
    fn from(e: TwinkleError) -> Error {
        std::io::Error::new(ErrorKind::Other, e.to_string())
    }
}

