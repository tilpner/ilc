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

pub use self::energymech::Energymech;
pub use self::weechat::Weechat;
pub use self::binary::Binary;
pub use self::msgpack::Msgpack;

mod energymech;
mod weechat;
// pub mod irssi;
mod binary;
mod msgpack;

pub trait Encode {
    fn encode<'a>(&'a self, context: &'a Context, output: &'a mut Write, event: &'a Event) -> ::Result<()>;
}

pub trait Decode  {
    fn decode<'a>(&'a mut self, context: &'a Context, input: &'a mut BufRead) -> Box<Iterator<Item = ::Result<Event<'a>>> + 'a>;
}

pub struct Dummy;

impl Decode for Dummy {
    fn decode<'a>(&'a mut self, _context: &'a Context, _input: &'a mut BufRead) -> Box<Iterator<Item = ::Result<Event<'a>>> + 'a> {
        Box::new(iter::empty())
    }
}

pub fn decoder(format: &str) -> Option<Box<Decode>> {
    match format {
        "energymech" | "em" => Some(Box::new(Energymech)),
        "weechat" | "w" => Some(Box::new(Weechat)),
//         "irssi" => Some(Box::new(irssi::Irssi)),
        "binary" => Some(Box::new(Binary)),
        "msgpack" => Some(Box::new(Msgpack)),
        _ => None
    }
}

pub fn encoder(format: &str) -> Option<Box<Encode>> {
    match format {
        "energymech" | "em" => Some(Box::new(Energymech)),
        "weechat" | "w" => Some(Box::new(Weechat)),
//         "irssi" => Some(Box::new(irssi::Irssi)),
        "binary" => Some(Box::new(Binary)),
        "msgpack" => Some(Box::new(Msgpack)),
        _ => None
    }
}

fn rejoin(s: &[&str], splits: &[char]) -> Cow<'static, str> {
    let len = s.iter().map(|s| s.len()).fold(0, |a, b| a + b);
    let mut out = s.iter().zip(splits.iter()).fold(String::with_capacity(len),
        |mut s, (b, &split)| { s.push_str(b); s.push(split); s });
    out.pop(); Cow::Owned(out)
}

fn strip_one(s: &str) -> String {
    if s.len() >= 2 { s[1..(s.len() - 1)].to_owned() } else { String::new() }
}
