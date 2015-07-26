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
extern crate chrono;
extern crate docopt;
extern crate rustc_serialize;
extern crate libc;
extern crate regex;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::process;
use std::io::{ self, BufReader };
use std::fs::File;
use std::error::Error;
use std::str::FromStr;
use std::collections::HashMap;

use docopt::Docopt;

use chrono::offset::fixed::FixedOffset;
use chrono::naive::date::NaiveDate;

use ilc::context::Context;
use ilc::format::{ self, Encode, Decode, DecodeBox };
use ilc::event::{ Event, Type };

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
  ilc convert <informat> <outformat> [--date DATE] [--tz SECS] [--channel CH]
  ilc freq
  ilc (-h | --help | -v | --version)

Options:
  -h --help         Show this screen.
  -v --version      Show the version (duh).
  --date DATE       Override the date for this log. ISO 8601, YYYY-MM-DD.
  --tz SECONDS      UTC offset in the direction of the western hemisphere. [default: 0]
  --channel CH      Set a channel for the given log.
"#;

#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_parse: bool,
    cmd_convert: bool,
    cmd_freq: bool,
    arg_file: Vec<String>,
    arg_informat: Option<String>,
    arg_outformat: Option<String>,
    flag_help: bool,
    flag_version: bool,
    flag_date: Option<String>,
    flag_tz: i32,
    flag_channel: Option<String>
}

fn error(e: Box<Error>) -> ! {
    println!("{}", e.description());
    let mut e = e.cause();
    while let Some(err) = e {
        println!("\t{}", err.description());
        e = err.cause();
    }
    process::exit(1)
}

fn main() {
    env_logger::init().unwrap();
    let args: Args = Docopt::new(USAGE)
               .and_then(|d| d.decode())
               .unwrap_or_else(|e| e.exit());
    if args.flag_help {
        println!("{}", USAGE);
        process::exit(1)
    }

    let context = Context {
        timezone: FixedOffset::west(args.flag_tz),
        override_date: args.flag_date.and_then(|d| NaiveDate::from_str(&d).ok()),
        channel: args.flag_channel.clone()
    };

    if args.cmd_parse {
        let mut parser = format::energymech::Energymech;
        let formatter = format::binary::Binary;
        for file in args.arg_file {
            let f: BufReader<File> = BufReader::new(File::open(file).unwrap());
            let iter = parser.decode(&context, f);
            for e in iter {
                info!("Parsed: {:?}", e);
                drop(formatter.encode(&context, io::stdout(), &e.unwrap()));
            }
        }
    }

    if args.cmd_convert {
        let stdin = io::stdin();

        let informat = args.arg_informat.expect("Must provide informat");
        let outformat = args.arg_outformat.expect("Must provide outformat");
        let mut decoder = format::decoder(&informat).expect("Decoder not available");
        let encoder = format::encoder(&outformat).expect("Encoder not available");

        let mut lock = stdin.lock();
        for e in decoder.decode_box(&context, &mut lock) {
            match e {
                Ok(e) => {
                    let _ = encoder.encode(&context, &mut io::stdout(), &e);
                },
                Err(e) => error(Box::new(e))
            }
        }
    }

    if args.cmd_freq {
        struct Person {
            lines: u32,
            words: u32
        }

        fn words(s: &str) -> u32 {
            s.split_whitespace().filter(|s| !s.is_empty()).count() as u32
        }

        fn strip_nick_prefix(s: &str) -> &str {
            if s.is_empty() { return s }
            match s.as_bytes()[0] {
                b'~' | b'&' | b'@' | b'%' | b'+' => &s[1..],
                _ => s
            }
        }

        let stdin = io::stdin();

        let mut stats: HashMap<String, Person> = HashMap::new();
        let context = Context {
            timezone: FixedOffset::west(0),
            override_date: Some(NaiveDate::from_ymd(2015, 6, 10)),
            channel: Some("#code".to_owned())
        };

        let mut parser = format::weechat3::Weechat3;
        for e in parser.decode(&context, stdin.lock()) {
            let m = match e {
                Ok(m) => m,
                Err(err) => panic!(err)
            };

            match m {
                Event { ty: Type::Msg { ref from, ref content, .. }, .. } => {
                    let nick = strip_nick_prefix(from);
                    if stats.contains_key(nick) {
                        let p: &mut Person = stats.get_mut(nick).unwrap();
                        p.lines += 1;
                        p.words += words(content);
                    } else {
                        stats.insert(nick.to_owned(), Person {
                            lines: 1,
                            words: words(content)
                        });
                    }
                },
                _ => ()
            }
        }

        let mut stats: Vec<(String, Person)> = stats.into_iter().collect();
        stats.sort_by(|&(_, ref a), &(_, ref b)| b.words.cmp(&a.words));

        for &(ref name, ref stat) in stats.iter().take(10) {
            println!("{}:\n\tLines: {}\n\tWords: {}", name, stat.lines, stat.words)
        }
    }
}
