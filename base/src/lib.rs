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

extern crate chrono;
#[macro_use]
extern crate log;
extern crate rustc_serialize;

pub mod event;
pub mod context;
pub mod error;
pub mod format;
pub mod dummy;

use std::io::{BufRead, Write};

pub use context::Context;
pub use event::{Event, Time};
pub use error::*;

pub trait Encode {
    fn encode<'a>(&'a self,
                  context: &'a Context,
                  output: &'a mut Write,
                  event: &'a Event)
                  -> error::Result<()>;
}

pub trait Decode {
    fn decode<'a>(&'a self,
                  context: &'a Context,
                  input: &'a mut BufRead)
                  -> Box<Iterator<Item = error::Result<Event<'a>>> + 'a>;
}
