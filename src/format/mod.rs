//! Traits and structs for conversion between various formats.
//! As the source format may not provide the same information as the
//! target format, all formats must allow for omittable information.

use std::io::{ self, BufRead, Write };

use log::Event;

pub mod weechat3;

pub trait Encode<W> where W: Write {
    fn encode(&self, output: W, event: &Event) -> io::Result<()>;
}

pub trait Decode<R, O> where R: BufRead, O: Iterator<Item = ::Result<Event>> {
    fn decode(&mut self, input: R) -> O;
}
