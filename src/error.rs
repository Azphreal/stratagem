use std::io;
use std::convert::From;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "game was exited before completion")]
    EarlyExit,
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
