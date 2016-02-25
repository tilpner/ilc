use std::{error, fmt, io, result};
use std::error::Error as E;

use chrono::format::ParseError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Parse(String),
    Chrono(ParseError),
    Io(io::Error),
    Custom(Box<error::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match self {
            &Parse(_) => "error while parsing",
            &Chrono(_) => "error while parsing time strings",
            &Io(_) => "error during input/output",
            &Custom(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        use self::Error::*;
        match self {
            &Parse(ref _e) => None,
            &Chrono(ref e) => Some(e),
            &Io(ref e) => Some(e),
            &Custom(ref e) => e.cause(),
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        Error::Chrono(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
