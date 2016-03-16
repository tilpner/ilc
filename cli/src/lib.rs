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
extern crate serde;
extern crate serde_json;
extern crate glob;
extern crate regex;

use ilc_base::{Context, Decode, Encode};
use ilc_ops::convert::{Filter, Operator, Subject};
use ilc_format_weechat::Weechat;
use ilc_format_energymech::Energymech;

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, SubCommand};

use chrono::{FixedOffset, NaiveDate};

use glob::glob;

use regex::Regex;

use std::str::FromStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::process;
use std::error::Error;


mod chain;
mod stats;

pub struct Cli {
    pub version: String,
    pub master_hash: Option<String>,
    pub notice: &'static str,
}

pub fn main(cli: Cli) {
    env_logger::init().unwrap();
    if option_env!("FUSE").is_some() {
        info!("Compiled with FUSEs")
    }

    let version = match cli.master_hash {
        Some(ref h) => format!("{} ({})", cli.version, h),
        None => cli.version.clone(),
    };
    let args = App::new("ilc")
                   .version(&version[..])
                   .setting(AppSettings::GlobalVersion)
                   .setting(AppSettings::AllowLeadingHyphen)
                   .setting(AppSettings::UnifiedHelpMessage)
                   .setting(AppSettings::VersionlessSubcommands)
                   .setting(AppSettings::ArgRequiredElseHelp)
                   .author("Till Höppner <till@hoeppner.ws>")
                   .about("A converter and statistics utility for IRC log files")
                   .arg(Arg::with_name("time")
                            .help("Timestamp offset, in seconds")
                            .global(true)
                            .takes_value(true)
                            .long("timeoffset")
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
                            .number_of_values(1)
                            .long("input")
                            .short("i"))
                   .arg(Arg::with_name("output_file")
                            .help("Specify an output file, instead of stdout")
                            .global(true)
                            .takes_value(true)
                            .long("output")
                            .short("o"))
                   .arg(Arg::with_name("notice")
                            .help("Print all the notices/licenses")
                            .takes_value(false)
                            .long("notice"))
                   .subcommand(SubCommand::with_name("parse")
                                   .about("Parse the input, checking the format")
                                   .setting(AppSettings::AllowLeadingHyphen))
                   .subcommand(SubCommand::with_name("convert")
                                   .about("Convert from a source to a target format")
                                   .setting(AppSettings::AllowLeadingHyphen)
                                   .arg(Arg::with_name("subject")
                                            .takes_value(true)
                                            .requires("operator")
                                            .long("if")
                                            .possible_values(&["nick", "time", "type", "text"]))
                                   .arg(Arg::with_name("op_not").long("not"))
                                   .arg(Arg::with_name("op_exactly")
                                            .takes_value(true)
                                            .long("exactly"))
                                   .arg(Arg::with_name("op_contains")
                                            .takes_value(true)
                                            .long("contains"))
                                   .arg(Arg::with_name("op_greater")
                                            .takes_value(true)
                                            .long("greater"))
                                   .arg(Arg::with_name("op_less").takes_value(true).long("less"))
                                   .arg(Arg::with_name("op_matches")
                                            .takes_value(true)
                                            .long("matches"))
                                   .group(ArgGroup::with_name("operator").args(&["op_exactly",
                                                                                 "op_contains",
                                                                                 "op_equal",
                                                                                 "op_greater",
                                                                                 "op_less",
                                                                                 "op_matches"])))
                   .subcommand(SubCommand::with_name("stats")
                                   .about("Analyse the activity of users by certain metrics")
                                   .setting(AppSettings::AllowLeadingHyphen))
                   .subcommand(SubCommand::with_name("seen")
                                   .about("Print the last line a nick was active")
                                   .setting(AppSettings::AllowLeadingHyphen)
                                   .arg(Arg::with_name("nick")
                                            .help("The nick you're looking for")
                                            .takes_value(true)
                                            .required(true)
                                            .index(1)))
                   .subcommand(SubCommand::with_name("sort")
                                   .about("Sorts a log by time")
                                   .setting(AppSettings::AllowLeadingHyphen))
                   .subcommand(SubCommand::with_name("dedup")
                                   .about("Removes duplicate log entries in close proximity")
                                   .setting(AppSettings::AllowLeadingHyphen))
                   .subcommand(SubCommand::with_name("merge")
                                   .about("Merges the input logs. This has to keep everything \
                                           in memory")
                                   .setting(AppSettings::AllowLeadingHyphen))
                   .get_matches();

    if args.is_present("notice") {
        println!("{}", cli.notice);
        process::exit(0)
    }

    let res = match args.subcommand() {
        ("parse", Some(args)) => {
            let e = Environment(&args);
            ilc_ops::parse::parse(&e.context(), &mut e.input(), &mut *e.decoder())
        }
        ("convert", Some(args)) => {
            let e = Environment(&args);
            let subject = match args.value_of("subject") {
                Some("nick") => Some(Subject::Nick),
                Some("time") => Some(Subject::Time),
                Some("type") => Some(Subject::Type),
                Some("text") => Some(Subject::Text),
                _ => None,
            };

            let op = {
                if args.is_present("operator") {
                    if let Some(sub) = args.value_of("op_exactly") {
                        Some(Operator::Exactly(sub.into()))
                    } else if let Some(sub) = args.value_of("op_contains") {
                        Some(Operator::Contains(sub.into()))
                    } else if let Some(sub) = args.value_of("op_matches") {
                        match Regex::new(sub) {
                            Ok(regex) => Some(Operator::Matches(regex)),
                            Err(e) => error(Box::new(e)),
                        }
                    } else {
                        // must be numeric operator if not op_exactly, op_contains, or op_matches
                        // unwrap is safe because of .is_present("operator") earlier
                        let num = match args.value_of("operator").unwrap().parse::<i64>() {
                            Ok(n) => n,
                            Err(e) => error(Box::new(e)),
                        };

                        if args.is_present("op_equal") {
                            Some(Operator::Equal(num))
                        } else if args.is_present("op_greater") {
                            Some(Operator::Greater(num))
                        } else if args.is_present("op_less") {
                            Some(Operator::Less(num))
                        } else {
                            None
                        }
                    }
                } else {
                    None
                }
            };

            let filter = subject.and_then(|s| op.map(|o| Filter(s, o)));

            ilc_ops::convert::convert(&e.context(),
                                      &mut e.input(),
                                      &mut *e.decoder(),
                                      &mut *e.output(),
                                      &*e.encoder(),
                                      filter,
                                      args.is_present("op_not"))
        }
        ("stats", Some(args)) => {
            let e = Environment(&args);
            let stats = ilc_ops::stats::stats(&e.context(), &mut e.input(), &mut *e.decoder())
                            .unwrap_or_else(|e| error(Box::new(e)));

            stats::output_as_json(&args, &cli, stats)
        }
        ("seen", Some(args)) => {
            let e = Environment(&args);
            let nick = args.value_of("nick").expect("Required argument <nick> not present");
            ilc_ops::seen::seen(nick,
                                &e.context(),
                                &mut e.input(),
                                &mut *e.decoder(),
                                &mut *e.output(),
                                &Weechat)
        }
        ("sort", Some(args)) => {
            let e = Environment(&args);
            ilc_ops::sort::sort(&e.context(),
                                &mut e.input(),
                                &mut *e.decoder(),
                                &mut *e.output(),
                                &*e.encoder())
        }
        ("dedup", Some(args)) => {
            let e = Environment(&args);
            ilc_ops::dedup::dedup(&e.context(),
                                  &mut e.input(),
                                  &mut *e.decoder(),
                                  &mut *e.output(),
                                  &*e.encoder())
        }
        ("merge", Some(args)) => {
            let e = Environment(&args);

            let mut inputs = e.inputs();
            let borrowed_inputs = inputs.iter_mut()
                                        .map(|a| a as &mut BufRead)
                                        .collect();
            ilc_ops::merge::merge(&e.context(),
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

macro_rules! error {
    ($code: expr, $fmt:expr) => {{
        use std::io::Write;
        let err = std::io::stderr();
        let _ = writeln!(&mut err.lock(), $fmt);
        std::process::exit($code);
    }};
    ($code: expr, $fmt:expr, $($arg:tt)*) => {{
        use std::io::Write;
        let err = std::io::stderr();
        let _ = writeln!(&mut err.lock(), $fmt, $($arg)*);
        std::process::exit($code);
    }};
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
        None => error!(2, "The format `{}` is unknown to me", inf),
    }
}

pub fn force_encoder<'a>(s: Option<&str>) -> Box<Encode> {
    let outf = match s {
        Some(s) => s,
        None => die("You didn't specify the output format"),
    };
    match encoder(&outf) {
        Some(e) => e,
        None => error!(2, "The format `{}` is unknown to me", outf),
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

    pub fn encoder(&self) -> Box<Encode> {
        force_encoder(self.0.value_of("format").or(self.0.value_of("output_format")))
    }
}


pub fn build_context(args: &ArgMatches) -> Context {
    let mut context = Context {
        timezone: FixedOffset::west(args.value_of("time")
                                        .and_then(|s| s.parse::<i32>().ok())
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
            n => error!(3, "Too many input files ({}), can't infer date", n),
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
