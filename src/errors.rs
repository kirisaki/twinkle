use failure::{Fail};
use std::io::{Error, ErrorKind};

#[derive(Debug, Fail)]
pub enum TwinkleError {
    #[fail(display = "failed parsing")]
    FailedParsing,
    #[fail(display = "failed serialization")]
    FailedSerialization,
    #[fail(display = "failed to deserialize")]
    FailedDeserialization,
    #[fail(display = "something wrong")]
    SomethingWrong,
}

impl From<TwinkleError> for Error {
    fn from(e: TwinkleError) -> Error {
        Error::new(ErrorKind::Other, e.to_string())
    }
}
