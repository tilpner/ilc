extern crate chrono;
#[macro_use]
extern crate log;
extern crate rustc_serialize;

pub mod event;
pub mod context;
pub mod error;
pub mod format;
pub mod dummy;

use std::io::{BufRead, Write};

pub use context::Context;
pub use event::{Event, Time};
pub use error::*;

pub trait Encode {
    fn encode<'a>(&'a self,
                  context: &'a Context,
                  output: &'a mut Write,
                  event: &'a Event)
                  -> error::Result<()>;
}

pub trait Decode {
    fn decode<'a>(&'a self,
                  context: &'a Context,
                  input: &'a mut BufRead)
                  -> Box<Iterator<Item = error::Result<Event<'a>>> + 'a>;
}
