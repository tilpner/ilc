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

extern crate ilc_base;
extern crate ilc_ops;
extern crate ilc_format_weechat;
extern crate ilc_format_energymech;
extern crate chrono;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate glob;

use ilc_base::{Context, Decode, Encode};
use ilc_ops::*;
use ilc_format_weechat::Weechat;
use ilc_format_energymech::Energymech;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

use chrono::{FixedOffset, NaiveDate};

use glob::glob;

use std::str::FromStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::{process, usize};
use std::error::Error;

mod chain;

pub fn main() {
    env_logger::init().unwrap();
    if option_env!("FUSE").is_some() {
        info!("Compiled with FUSEs")
    }

    let args = App::new("ilc")
                   .version(crate_version!())
                   .setting(AppSettings::GlobalVersion)
                   .setting(AppSettings::VersionlessSubcommands)
                   .setting(AppSettings::ArgRequiredElseHelp)
                   .author("Till Höppner <till@hoeppner.ws>")
                   .about("A converter and statistics utility for IRC log files")
                   .arg(Arg::with_name("timezone")
                            .help("UTC offset in the direction of the western hemisphere")
                            .global(true)
                            .takes_value(true)
                            .long("timezone")
                            .short("t"))
                   .arg(Arg::with_name("date")
                            .help("Override the date for this log, ISO 8601, YYYY-MM-DD")
                            .global(true)
                            .takes_value(true)
                            .long("date")
                            .short("d"))
                   .arg(Arg::with_name("infer_date")
                            .help("Try to use the filename as date for the current log")
                            .global(true)
                            .long("infer-date"))
                   .arg(Arg::with_name("channel")
                            .help("Set a channel for the current log")
                            .global(true)
                            .takes_value(true)
                            .long("channel")
                            .short("c"))
                   .arg(Arg::with_name("format")
                            .help("Set the input and output format for the current log")
                            .global(true)
                            .takes_value(true)
                            .long("format")
                            .short("f"))
                   .arg(Arg::with_name("input_format")
                            .help("Set the input format for the current log")
                            .global(true)
                            .conflicts_with("format")
                            .takes_value(true)
                            .long("inf"))
                   .arg(Arg::with_name("output_format")
                            .help("Set the output format for the current log")
                            .global(true)
                            .conflicts_with("format")
                            .takes_value(true)
                            .long("outf"))
                   .arg(Arg::with_name("input_files")
                            .help("Specify an input file, instead of stdin")
                            .global(true)
                            .takes_value(true)
                            .multiple(true)
                            .long("input")
                            .short("i"))
                   .arg(Arg::with_name("output_file")
                            .help("Specify an output file, instead of stdout")
                            .global(true)
                            .takes_value(true)
                            .long("output")
                            .short("o"))
                   .subcommand(SubCommand::with_name("parse")
                                   .about("Parse the input, checking the format"))
                   .subcommand(SubCommand::with_name("convert")
                                   .about("Convert from a source to a target format"))
                   .subcommand(SubCommand::with_name("freq")
                                   .about("Analyse the activity of users by certain metrics")
                                   .arg(Arg::with_name("count")
                                            .help("The number of items to be displayed")
                                            .takes_value(true)
                                            .long("count")))
                   .subcommand(SubCommand::with_name("seen")
                                   .about("Print the last line a nick was active")
                                   .arg(Arg::with_name("nick")
                                            .help("The nick you're looking for")
                                            .takes_value(true)
                                            .required(true)
                                            .index(1)))
                   .subcommand(SubCommand::with_name("sort").about("Sorts a log by time"))
                   .subcommand(SubCommand::with_name("dedup")
                                   .about("Removes duplicate log entries in close proximity"))
                   .subcommand(SubCommand::with_name("merge")
                                   .about("Merges the input logs. This has to keep everything \
                                           in memory"))
                   .get_matches();

    let res = match args.subcommand() {
        ("parse", Some(args)) => {
            let e = Environment(&args);
            parse::parse(&e.context(), &mut e.input(), &mut *e.decoder())
        }
        ("convert", Some(args)) => {
            let e = Environment(&args);
            convert::convert(&e.context(),
                             &mut e.input(),
                             &mut *e.decoder(),
                             &mut *e.output(),
                             &*e.encoder())
        }
        ("freq", Some(args)) => {
            let e = Environment(&args);
            let count = value_t!(args, "count", usize).unwrap_or(usize::MAX);
            freq::freq(count,
                       &e.context(),
                       &mut e.input(),
                       &mut *e.decoder(),
                       &mut e.output())
        }
        ("seen", Some(args)) => {
            let e = Environment(&args);
            let nick = args.value_of("nick").expect("Required argument <nick> not present");
            seen::seen(nick,
                       &e.context(),
                       &mut e.input(),
                       &mut *e.decoder(),
                       &mut *e.output(),
                       &Weechat)
        }
        ("sort", Some(args)) => {
            let e = Environment(&args);
            sort::sort(&e.context(),
                       &mut e.input(),
                       &mut *e.decoder(),
                       &mut *e.output(),
                       &*e.encoder())
        }
        ("dedup", Some(args)) => {
            let e = Environment(&args);
            dedup::dedup(&e.context(),
                         &mut e.input(),
                         &mut *e.decoder(),
                         &mut *e.output(),
                         &*e.encoder())
        }
        ("merge", Some(args)) => {
            let e = Environment(&args);

            let mut inputs = e.inputs();
            // let mut decoders = e.decoders();

            let borrowed_inputs = inputs.iter_mut()
                                        .map(|a| a as &mut BufRead)
                                        .collect();
            // let borrowed_decoders = decoders.iter_mut()
            //                                .map(|a| &mut **a as &mut Decode)
            //                                .collect();
            merge::merge(&e.context(),
                         borrowed_inputs,
                         &mut *e.decoder(),
                         &mut *e.output(),
                         &*e.encoder())
        }
        (sc, _) if !sc.is_empty() => panic!("Unimplemented subcommand `{}`, this is a bug", sc),
        _ => die("No command specified"),
    };

    match res {
        Ok(()) => (),
        Err(e) => error(Box::new(e)),
    }
}

pub fn error(e: Box<Error>) -> ! {
    let _ = writeln!(&mut io::stderr(), "Error: {}", e);
    let mut e = e.cause();
    while let Some(err) = e {
        let _ = writeln!(&mut io::stderr(), "\t{}", err);
        e = err.cause();
    }
    process::exit(1)
}

pub fn die(s: &str) -> ! {
    let _ = writeln!(&mut io::stderr(), "Aborting: {}", s);
    process::exit(1)
}

pub fn decoder(format: &str) -> Option<Box<Decode>> {
    match format {
        "energymech" | "em" => Some(Box::new(Energymech)),
        "weechat" | "w" => Some(Box::new(Weechat)),
        // "irssi" => Some(Box::new(irssi::Irssi)),
        // "binary" => Some(Box::new(Binary)),
        // "msgpack" => Some(Box::new(Msgpack)),
        _ => None,
    }
}

pub fn encoder(format: &str) -> Option<Box<Encode>> {
    match format {
        "energymech" | "em" => Some(Box::new(Energymech)),
        "weechat" | "w" => Some(Box::new(Weechat)),
        // "irssi" => Some(Box::new(irssi::Irssi)),
        // "binary" => Some(Box::new(Binary)),
        // "msgpack" => Some(Box::new(Msgpack)),
        _ => None,
    }
}

pub fn force_decoder(s: Option<&str>) -> Box<Decode> {
    let inf = match s {
        Some(s) => s,
        None => die("You didn't specify the input format"),
    };
    match decoder(&inf) {
        Some(d) => d,
        None => die(&format!("The format `{}` is unknown to me", inf)),
    }
}

pub fn force_encoder<'a>(s: Option<&str>) -> Box<Encode> {
    let outf = match s {
        Some(s) => s,
        None => die("You didn't specify the output format"),
    };
    match encoder(&outf) {
        Some(e) => e,
        None => die(&format!("The format `{}` is unknown to me", outf)),
    }
}

pub struct Environment<'a>(pub &'a ArgMatches<'a>);

impl<'a> Environment<'a> {
    pub fn context(&self) -> Context {
        build_context(self.0)
    }

    pub fn input(&self) -> Box<BufRead> {
        open_files(gather_input(self.0))
    }

    pub fn inputs(&self) -> Vec<Box<BufRead>> {
        gather_input(self.0)
            .iter()
            .map(|path| {
                Box::new(BufReader::new(File::open(path)
                                            .unwrap_or_else(|e| {
                                                error(Box::new(e))
                                            }))) as Box<BufRead>
            })
            .collect()
    }

    pub fn output(&self) -> Box<Write> {
        open_output(self.0)
    }

    pub fn decoder(&self) -> Box<Decode> {
        force_decoder(self.0.value_of("format").or(self.0.value_of("input_format")))
    }

    /* pub fn decoders(&self) -> Vec<Box<Decode>> {
     * self.0
     * .value_of("format")
     * .into_iter()
     * .chain(self.0
     * .values_of("input_formats")
     * .map(|i| Box::new(i) as Box<Iterator<Item = _>>)
     * .unwrap_or(Box::new(iter::empty()) as Box<Iterator<Item = _>>))
     * .map(Option::Some)
     * .map(force_decoder)
     * .collect()
     * } */

    pub fn encoder(&self) -> Box<Encode> {
        force_encoder(self.0.value_of("format").or(self.0.value_of("output_format")))
    }
}


pub fn build_context(args: &ArgMatches) -> Context {
    let mut context = Context {
        timezone: FixedOffset::west(args.value_of("timezone")
                                        .and_then(|s| s.parse().ok())
                                        .unwrap_or(0)),
        override_date: args.value_of("date").and_then(|d| NaiveDate::from_str(&d).ok()),
        channel: args.value_of("channel").map(str::to_owned).clone(),
    };
    if args.is_present("infer_date") {
        let input_files = gather_input(args);
        match input_files.len() {
            0 => die("No input files given, can't infer date"),
            1 => {
                if let Some(date) = input_files.get(0)
                                               .map(PathBuf::as_path)
                                               .and_then(Path::file_stem)
                                               .and_then(OsStr::to_str)
                                               .and_then(|s: &str| NaiveDate::from_str(s).ok()) {
                    context.override_date = Some(date);
                }
            }
            _n => die("Too many input files, can't infer date"),
        }
    }
    context
}

pub fn gather_input(args: &ArgMatches) -> Vec<PathBuf> {
    if let Some(iter) = args.values_of("input_files") {
        iter.flat_map(|p| {
                match glob(p) {
                    Ok(paths) => paths,
                    Err(e) => die(&format!("{}", e.msg)),
                }
            })
            .filter_map(Result::ok)
            .collect()
    } else {
        Vec::new()
    }
}

pub fn open_files(files: Vec<PathBuf>) -> Box<BufRead> {
    if files.len() > 0 {
        Box::new(BufReader::new(chain::Chain::new(files.iter()
                                                       .map(|p| File::open(p).unwrap())
                                                       .collect())))
    } else {
        Box::new(BufReader::new(io::stdin()))
    }
}

pub fn open_output(args: &ArgMatches) -> Box<Write> {
    if let Some(out) = args.value_of("output_file") {
        match File::create(out) {
            Ok(f) => Box::new(BufWriter::new(f)),
            Err(e) => error(Box::new(e)),
        }
    } else {
        Box::new(BufWriter::new(io::stdout()))
    }
}
