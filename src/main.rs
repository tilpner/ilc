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

#![feature(libc, plugin)]
#![plugin(regex_macros)]

extern crate ilc;
extern crate docopt;
extern crate rustc_serialize;
extern crate libc;
extern crate regex;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::fs::File;
use std::io::{ self, BufReader };

use docopt::Docopt;

use ilc::format::{ self, Encode, Decode };

static USAGE: &'static str = r#"
d8b   888
Y8P   888
      888
888   888    .d8888b
888   888   d88P"
888   888   888
888   888   Y88b.
888   888    "Y8888P

A converter and statistics utility for IRC log files.

Usage:
  ilc parse <file>...
  ilc

Options:
  -h --help         Show this screen.
  -v --version      Show the version (duh).
"#;

#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_parse: bool,
    arg_file: Vec<String>,
    flag_help: bool,
    flag_version: bool
}

fn main() {
    env_logger::init().unwrap();
    let args: Args = Docopt::new(USAGE)
               .and_then(|d| d.decode())
               .unwrap_or_else(|e| e.exit());
    if args.flag_help {
        println!("{}", USAGE);
        unsafe { libc::funcs::c95::stdlib::exit(1) }
    }

    if args.cmd_parse {
        let mut parser = format::weechat3::Weechat3;
        for file in args.arg_file {
            let f: BufReader<File> = BufReader::new(File::open(file).unwrap());
            let iter = parser.decode(f);
            for e in iter {
                info!("Parsed: {:?}", e);
                drop(parser.encode(io::stdout(), &e.unwrap()));
            }
        }
    }
}
