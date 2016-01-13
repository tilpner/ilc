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
extern crate glob;
extern crate blist;

use std::process;
use std::io::{ self, Read, BufRead, BufReader, Write, BufWriter };
use std::fs::File;
use std::error::Error;
use std::str::FromStr;
use std::collections::HashMap;

use docopt::Docopt;

use chrono::offset::fixed::FixedOffset;
use chrono::naive::date::NaiveDate;

use glob::glob;

use ilc::context::Context;
use ilc::format::{ self, Encode, Decode };
use ilc::event::{ Event, Type, NoTimeHash };

use ageset::AgeSet;

mod chain;
mod ageset;

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
  ilc parse [options] [-i FILE...]
  ilc convert [options] [-i FILE...]
  ilc freq [options] [-i FILE...]
  ilc sort [options] [-i FILE...]
  ilc dedup [options] [-i FILE...]
  ilc (-h | --help | -v | --version)

Options:
  -h --help         Show this screen.
  -v --version      Show the version (duh).
  --date DATE       Override the date for this log. ISO 8601, YYYY-MM-DD.
  --tz SECONDS      UTC offset in the direction of the western hemisphere.
  --channel CH      Set a channel for the given log.
  --inf INF         Set the input format.
  --outf OUTF       Set the output format.
  --in -i IN        Give an input file, instead of stdin.
  --out -o OUT      Give an output file, instead of stdout.
"#;

#[derive(RustcDecodable, Debug)]
struct Args {
    cmd_parse: bool,
    cmd_convert: bool,
    cmd_freq: bool,
    cmd_sort: bool,
    cmd_dedup: bool,
    arg_file: Vec<String>,
    flag_in: Vec<String>,
    flag_out: Option<String>,
    flag_inf: Option<String>,
    flag_outf: Option<String>,
    flag_help: bool,
    flag_version: bool,
    flag_date: Option<String>,
    flag_tz: Option<String>,
    flag_channel: Option<String>
}

fn error(e: Box<Error>) -> ! {
    let _ = writeln!(&mut io::stderr(), "Error: {}", e);
    let mut e = e.cause();
    while let Some(err) = e {
        let _ = writeln!(&mut io::stderr(), "\t{}", err);
        e = err.cause();
    }
    process::exit(1)
}

fn die(s: &str) -> ! {
    let _ = writeln!(&mut io::stderr(), "Aborting: {}", s);
    process::exit(1)
}

fn force_decoder<'a, 'b, 'c>(s: Option<String>) -> Box<Decode<'a, 'b, 'c>> {
    let inf = match s {
        Some(s) => s,
        None => die("You didn't specify the input format")
    };
    match format::decoder(&inf) {
        Some(d) => d,
        None => die(&format!("The format `{}` is unknown to me", inf))
    }
}

fn force_encoder<'a, 'b: 'a>(s: Option<String>) -> Box<Encode<'a, &'b mut Write>> {
    let outf = match s {
        Some(s) => s,
        None => die("You didn't specify the output format")
    };
    match format::encoder(&outf) {
        Some(e) => e,
        None => die(&format!("The format `{}` is unknown to me", outf))
    }
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
        timezone: FixedOffset::west(args.flag_tz.and_then(|s| s.parse().ok()).unwrap_or(0)),
        override_date: args.flag_date.and_then(|d| NaiveDate::from_str(&d).ok()),
        channel: args.flag_channel.clone()
    };

    let mut input: Box<BufRead> = if args.flag_in.len() > 0 {
        let input_files = args.flag_in.iter()
            .flat_map(|p| {
                match glob(p) {
                    Ok(paths) => paths,
                    Err(e) => die(&format!("{}", e.msg))
                }
            }).filter_map(Result::ok).map(|p| File::open(p).unwrap()).collect();
        Box::new(BufReader::new(chain::Chain::new(input_files)))
    } else {
        Box::new(BufReader::new(io::stdin()))
    };

    /*let mut output: Box<Write> = if let Some(out) = args.flag_out {
        match File::create(out) {
            Ok(f) => Box::new(BufWriter::new(f)),
            Err(e) => error(Box::new(e))
        }
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };*/

    //if args.cmd_parse {
        let mut i = io::empty();
        let mut decoder: Box<Decode> = Box::new(format::Dummy);//force_decoder(args.flag_inf);
        // let encoder = force_encoder(args.flag_outf);
        let iter = decoder.decode(&context, &mut i);
        //for e in decoder.decode_box(&context, &mut input) {
            //let e = e.unwrap();
            //let _ = encoder.encode(&context, &mut output, &e);
        //}
    /*}*/ /*else if args.cmd_convert {
        let mut decoder = force_decoder(args.flag_inf);
        let encoder = force_encoder(args.flag_outf);
        for e in decoder.decode_box(&context, &mut input) {
            match e {
                Ok(e) => { let _ = encoder.encode(&context, &mut output, &e); },
                Err(e) => error(Box::new(e))
            }
        }
    } else if args.cmd_freq {
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

        let mut stats: HashMap<String, Person> = HashMap::new();

        let mut decoder = force_decoder(args.flag_inf);
        for e in decoder.decode_box(&context, &mut input) {
            let m = match e {
                Ok(m) => m,
                Err(err) => error(Box::new(err))
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
            let _ = write!(&mut output, "{}:\n\tLines: {}\n\tWords: {}\n", name, stat.lines, stat.words);
        }
    } else if args.cmd_sort {
        let mut decoder = force_decoder(args.flag_inf);
        let encoder = force_encoder(args.flag_outf);
        let mut events: Vec<Event> = decoder.decode_box(&context, &mut input)
            .flat_map(Result::ok)
            .collect();

        events.sort_by(|a, b| a.time.cmp(&b.time));
        for e in events {
            let _ = encoder.encode(&context, &mut output, &e);
        }
    } else if args.cmd_dedup {
        let mut decoder = force_decoder(args.flag_inf);
        let encoder = force_encoder(args.flag_outf);
        let mut backlog = AgeSet::new();

        for e in decoder.decode_box(&context, &mut input) {
            if let Ok(e) = e {
                let newest_event = e.clone();
                backlog.prune(move |a: &NoTimeHash| {
                    let age = newest_event.time.as_timestamp() - a.0.time.as_timestamp();
                    age > 5000
                });
                // write `e` if it's a new event
                let n = NoTimeHash(e);
                if !backlog.contains(&n) {
                    let _ = encoder.encode(&context, &mut output, &n.0);
                    backlog.push(n);
                }
            }
        }
    }*/
}
