use std::io::{BufRead, Write};
use std::iter::Iterator;

use event::Event;
use context::Context;
use format::{Decode, Encode};

use rustc_serialize::{Decodable, Encodable};
use msgpack::{Decoder, Encoder};
use rmp::decode::ReadError;

pub struct Msgpack;

pub struct Iter<'a> {
    input: &'a mut BufRead,
}

impl<'a> Iterator for Iter<'a> {
    type Item = ::Result<Event<'a>>;
    fn next(&mut self) -> Option<::Result<Event<'a>>> {
        use msgpack::decode;
        match Event::decode(&mut Decoder::new(&mut self.input)) {
            Ok(e) => Some(Ok(e)),
            Err(decode::Error::InvalidMarkerRead(ReadError::UnexpectedEOF)) => None,
            Err(e) => Some(Err(::IlcError::MsgpackDecode(e))),
        }
    }
}

impl Encode for Msgpack {
    fn encode<'a>(&'a self,
                  _context: &'a Context,
                  output: &'a mut Write,
                  event: &'a Event)
                  -> ::Result<()> {
        event.encode(&mut Encoder::new(output))
             .map_err(|e| ::IlcError::MsgpackEncode(e))
    }
}

impl Decode for Msgpack {
    fn decode<'a>(&'a mut self,
                  _context: &'a Context,
                  input: &'a mut BufRead)
                  -> Box<Iterator<Item = ::Result<Event<'a>>> + 'a> {
        Box::new(Iter { input: input })
    }
}
