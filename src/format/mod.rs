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

//! Traits and structs for conversion between various formats.
//! As the source format may not provide the same information as the
//! target format, all formats must allow for omittable information.

use std::iter;
use std::io::{ BufRead, Write };
use std::borrow::Cow;

use event::Event;
use context::Context;

//pub mod weechat3;
//pub mod energymech;
//pub mod binary;
//pub mod msgpack;

pub trait Encode<'a, W> where W: Write {
    fn encode(&'a self, context: &'a Context, output: W, event: &'a Event) -> ::Result<()>;
}

pub trait Decode<'a, 'b, 'c>  {
    fn decode(&'a mut self, context: &'b Context, input: &'c mut BufRead) -> Box<Iterator<Item = ::Result<Event<'a>>> + 'a>;
}

pub struct Dummy;

impl <'a, 'b, 'c> Decode<'a, 'b, 'c> for Dummy {
    fn decode(&'a mut self, _context: &'b Context, _input: &'c mut BufRead) -> Box<Iterator<Item = ::Result<Event<'a>>> + 'a> {
        Box::new(iter::empty())
    }
}

pub fn decoder<'a, 'b, 'c>(format: &str) -> Option<Box<Decode<'a, 'b, 'c>>> {
    match format {
//        "energymech" => Some(Box::new(energymech::Energymech)),
//        "weechat3" => Some(Box::new(weechat3::Weechat3)),
//        "binary" => Some(Box::new(binary::Binary)),
//        "msgpack" => Some(Box::new(msgpack::Msgpack)),
        _ => None
    }
}

pub fn encoder<'a, 'b: 'a>(format: &str) -> Option<Box<Encode<'a, &'b mut Write>>> {
    match format {
//        "energymech" => Some(Box::new(energymech::Energymech)),
//        "weechat3" => Some(Box::new(weechat3::Weechat3)),
//        "binary" => Some(Box::new(binary::Binary)),
//        "msgpack" => Some(Box::new(msgpack::Msgpack)),
        _ => None
    }
}

fn rejoin(s: &[&str], splits: &[char]) -> Cow<'static, str> {
    let len = s.iter().map(|s| s.len()).sum();
    let mut out = s.iter().zip(splits.iter()).fold(String::with_capacity(len),
        |mut s, (b, &split)| { s.push_str(b); s.push(split); s });
    out.pop(); Cow::Owned(out)
}

fn strip_one(s: &str) -> String {
    if s.len() >= 2 { s[1..(s.len() - 1)].to_owned() } else { String::new() }
}
