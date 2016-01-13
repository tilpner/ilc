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

use bincode::{ self, SizeLimit };

pub struct Binary;

pub struct Iter<'a, R: 'a> where R: BufRead {
    _phantom: PhantomData<&'a ()>,
    input: R
}

impl<'a, R: 'a> Iterator for Iter<'a, R> where R: BufRead {
    type Item = ::Result<Event<'a>>;
    fn next(&mut self) -> Option<::Result<Event<'a>>> {
        Some(bincode::rustc_serialize::decode_from::<R, Event>(&mut self.input, SizeLimit::Infinite)
             .map_err(|_| ::IlcError::BincodeDecode))
    }
}

impl<'a, W> Encode<'a, W> for Binary where W: Write {
    fn encode(&'a self, _context: &'a Context, mut output: W, event: &'a Event) -> ::Result<()> {
        bincode::rustc_serialize::encode_into(event, &mut output, SizeLimit::Infinite)
            .map_err(|_| ::IlcError::BincodeEncode)
    }
}

impl<'a, R: 'a> Decode<'a, R> for Binary where R: BufRead {
    type Output = Iter<'a, R>;
    fn decode(&'a mut self, _context: &'a Context, input: R) -> Iter<R> {
        Iter { _phantom: PhantomData, input: input }
    }
}
