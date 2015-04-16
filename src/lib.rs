#![feature(plugin, slice_patterns, core, custom_derive)]
#![plugin(regex_macros)]
extern crate regex;
extern crate chrono;
#[macro_use]
extern crate log as l;
extern crate rustc_serialize;
extern crate bincode;

pub mod log;
pub mod format;

use std::convert::From;
use std::{ io, result };

use chrono::format::ParseError;

pub type Result<T> = result::Result<T, IlcError>;

#[derive(Debug)]
pub enum IlcError {
    Parse(String),
    Chrono(ParseError),
    BincodeDecode,
    BincodeEncode,
    Io(io::Error)
}

impl From<ParseError> for IlcError {
    fn from(err: ParseError) -> IlcError { IlcError::Chrono(err) }
}

impl From<io::Error> for IlcError {
    fn from(err: io::Error) -> IlcError { IlcError::Io(err) }
}
