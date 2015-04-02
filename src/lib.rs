#![feature(plugin, str_char, slice_patterns, convert, core)]
#![plugin(regex_macros)]
extern crate regex;
extern crate chrono;
#[macro_use]
extern crate log as l;

pub mod log;
pub mod format;

use std::error::FromError;
use std::{ io, result };

use chrono::format::ParseError;

pub type Result<T> = result::Result<T, IlcError>;

#[derive(Debug, PartialEq)]
pub enum IlcError {
    Parse(String),
    Chrono(ParseError),
    Io(io::Error)
}

impl FromError<ParseError> for IlcError {
    fn from_error(err: ParseError) -> IlcError { IlcError::Chrono(err) }
}

impl FromError<io::Error> for IlcError {
    fn from_error(err: io::Error) -> IlcError { IlcError::Io(err) }
}
