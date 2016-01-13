// Copyright 2015 Till Höppner
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::io::{ BufRead, Write };
use std::iter::Iterator;
use std::marker::PhantomData;

use event::Event;
use context::Context;
use format::{ Encode, Decode };

use rustc_serialize::{ Encodable, Decodable };
use msgpack::{ Encoder, Decoder };
use rmp::decode::ReadError;

pub struct Msgpack;

pub struct Iter<'a, R: 'a> where R: BufRead {
    _phantom: PhantomData<&'a ()>,
    input: R
}

impl<'a, R: 'a> Iterator for Iter<'a, R> where R: BufRead {
    type Item = ::Result<Event<'a>>;
    fn next(&mut self) -> Option<::Result<Event<'a>>> {
        use msgpack::decode;
        match Event::decode(&mut Decoder::new(&mut self.input)) {
            Ok(e) => Some(Ok(e)),
            Err(decode::Error::InvalidMarkerRead(ReadError::UnexpectedEOF)) => None,
            Err(e) => Some(Err(::IlcError::MsgpackDecode(e)))
        }
    }
}

impl<'a, W> Encode<'a, W> for Msgpack where W: Write {
    fn encode(&'a self, _context: &'a Context, mut output: W, event: &'a Event) -> ::Result<()> {
        event.encode(&mut Encoder::new(&mut output))
            .map_err(|e| ::IlcError::MsgpackEncode(e))
    }
}

impl<'a, R: 'a> Decode<'a, R> for Msgpack where R: BufRead {
    type Output = Iter<'a, R>;
    fn decode(&'a mut self, _context: &'a Context, input: R) -> Iter<R> {
        Iter { _phantom: PhantomData, input: input }
    }
}
