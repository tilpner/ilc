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

use log::Event;

pub mod weechat3;
pub mod binary;

pub trait Encode<W> where W: Write {
    fn encode(&self, output: W, event: &Event) -> ::Result<()>;
}

pub trait Decode<R, O> where R: BufRead, O: Iterator<Item = ::Result<Event>> {
    fn decode(&mut self, input: R) -> O;
}
