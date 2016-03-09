use std::io::{BufRead, Write};
use std::iter::Iterator;

use event::Event;
use context::Context;
use format::{Decode, Encode};

use bincode::{self, SizeLimit};

pub struct Binary;

pub struct Iter<'a> {
    input: &'a mut BufRead,
}

impl<'a> Iterator for Iter<'a> {
    type Item = ::Result<Event<'a>>;
    fn next(&mut self) -> Option<::Result<Event<'a>>> {
        Some(bincode::rustc_serialize::decode_from::<_, Event>(&mut self.input,
                                                               SizeLimit::Infinite)
                 .map_err(|_| ::IlcError::BincodeDecode))
    }
}

impl Encode for Binary {
    fn encode<'a>(&'a self,
                  _context: &'a Context,
                  mut output: &'a mut Write,
                  event: &'a Event)
                  -> ::Result<()> {
        bincode::rustc_serialize::encode_into(event, &mut output, SizeLimit::Infinite)
            .map_err(|_| ::IlcError::BincodeEncode)
    }
}

impl Decode for Binary {
    fn decode<'a>(&'a mut self,
                  _context: &'a Context,
                  input: &'a mut BufRead)
                  -> Box<Iterator<Item = ::Result<Event<'a>>> + 'a> {
        Box::new(Iter { input: input })
    }
}
