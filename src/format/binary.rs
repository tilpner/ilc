use std::io::{ BufRead, Write };
use std::iter::Iterator;

use log::Event;
use format::{ Encode, Decode };

use bincode::{ self, SizeLimit };

pub struct Binary;

pub struct Iter<R> where R: BufRead {
    input: R
}

impl<R> Iterator for Iter<R> where R: BufRead {
    type Item = ::Result<Event>;
    fn next(&mut self) -> Option<::Result<Event>> {
        Some(bincode::decode_from::<R, Event>(&mut self.input, SizeLimit::Infinite)
             .map_err(|_| ::IlcError::BincodeDecode))
    }
}

impl<W> Encode<W> for Binary where W: Write {
    fn encode(&self, mut output: W, event: &Event) -> ::Result<()> {
        bincode::encode_into(event, &mut output, SizeLimit::Infinite)
            .map_err(|_| ::IlcError::BincodeEncode)
    }
}

impl<R> Decode<R, Iter<R>> for Binary where R: BufRead {
    fn decode(&mut self, input: R) -> Iter<R> {
        Iter { input: input }
    }
}
