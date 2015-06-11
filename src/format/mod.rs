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

use std::io::{ BufRead, Write };
use std::borrow::Cow;

use event::Event;
use context::Context;

pub mod weechat3;
pub mod energymech;
//pub mod binary;

pub trait Encode<'a, W> where W: Write {
    fn encode(&'a self, context: &'a Context, output: W, event: &'a Event) -> ::Result<()>;
}

pub trait Decode<'a, R, O> where R: BufRead, O: Iterator<Item = ::Result<Event<'a>>> {
    fn decode(&'a mut self, context: &'a Context, input: R) -> O;
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
