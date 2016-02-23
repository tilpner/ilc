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

extern crate ilc;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate rustc_serialize;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate glob;
extern crate blist;

use clap::{App, AppSettings, Arg, SubCommand};

mod chain;
mod ageset;
mod app;

fn main() {
    env_logger::init().unwrap();
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
                   .get_matches();

    match args.subcommand() {
        ("parse", Some(args)) => app::parse::parse(args),
        ("convert", Some(args)) => app::convert::convert(args),
        ("freq", Some(args)) => app::freq::freq(args),
        ("seen", Some(args)) => app::seen::seen(args),
        ("sort", Some(args)) => app::sort::sort(args),
        ("dedup", Some(args)) => app::dedup::dedup(args),
        (sc, _) if !sc.is_empty() => panic!("Unimplemented subcommand `{}`, this is a bug", sc),
        _ => (),
    }
}
