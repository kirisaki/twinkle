use std::fmt;
use std::fmt::Display;
use failure::{Backtrace, Context, Fail};

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

impl From<TwinkleError> for std::io::Error {
    fn from(e: TwinkleError) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    }
}
