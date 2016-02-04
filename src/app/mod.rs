use clap::ArgMatches;

use chrono::offset::fixed::FixedOffset;
use chrono::naive::date::NaiveDate;

use glob::glob;

use std::process;
use std::str::FromStr;
use std::path::{ Path, PathBuf };
use std::io::{ self, Write, BufWriter, BufRead, BufReader };
use std::fs::File;
use std::error::Error;
use std::ffi::OsStr;

use ilc::context::Context;
use ilc::format::{ self, Encode, Decode };

use ::chain;

pub mod freq;

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

pub fn force_decoder(s: Option<&str>) -> Box<Decode> {
    let inf = match s {
        Some(s) => s,
        None => die("You didn't specify the input format")
    };
    match format::decoder(&inf) {
        Some(d) => d,
        None => die(&format!("The format `{}` is unknown to me", inf))
    }
}

pub fn force_encoder<'a>(s: Option<&str>) -> Box<Encode> {
    let outf = match s {
        Some(s) => s,
        None => die("You didn't specify the output format")
    };
    match format::encoder(&outf) {
        Some(e) => e,
        None => die(&format!("The format `{}` is unknown to me", outf))
    }
}

pub fn build_context(args: &ArgMatches) -> Context {
    let mut context = Context {
        timezone: FixedOffset::west(args.value_of("timezone").and_then(|s| s.parse().ok()).unwrap_or(0)),
        override_date: args.value_of("date").and_then(|d| NaiveDate::from_str(&d).ok()),
        channel: args.value_of("channel").map(str::to_owned).clone()
    };
    if args.is_present("infer_date") {
        let input_files = gather_input(args);
        match input_files.len() {
            0 => die("No input files given, can't infer date"),
            1 => if let Some(date) = input_files.get(0)
                .map(PathBuf::as_path)
                .and_then(Path::file_stem)
                .and_then(OsStr::to_str)
                .and_then(|s: &str| NaiveDate::from_str(s).ok()) {
                context.override_date = Some(date);
            },
            _n => die("Too many input files, can't infer date")
        }
    }
    context
}

pub fn gather_input(args: &ArgMatches) -> Vec<PathBuf> {
    if let Some(iter) = args.values_of("input_files") {
        iter.flat_map(|p| {
            match glob(p) {
                Ok(paths) => paths,
                Err(e) => die(&format!("{}", e.msg))
            }
        }).filter_map(Result::ok).collect()
    } else { Vec::new() }
}

pub fn open_files(files: Vec<PathBuf>) -> Box<BufRead> {
    if files.len() > 0 {
        Box::new(BufReader::new(chain::Chain::new(files.iter().map(|p| File::open(p).unwrap()).collect())))
    } else {
        Box::new(BufReader::new(io::stdin()))
    }
}

pub fn open_output(args: &ArgMatches) -> Box<Write> {
    if let Some(out) = args.value_of("output_file") {
        match File::create(out) {
            Ok(f) => Box::new(BufWriter::new(f)),
            Err(e) => error(Box::new(e))
        }
    } else {
        Box::new(BufWriter::new(io::stdout()))
    }
}

pub struct Environment<'a>(pub &'a ArgMatches<'a>);

impl<'a> Environment<'a> {
    pub fn context(&self) -> Context { build_context(self.0) }
    pub fn input(&self) -> Box<BufRead> { open_files(gather_input(self.0)) }
    pub fn output(&self) -> Box<Write> { open_output(self.0) }
    pub fn decoder(&self) -> Box<Decode> { force_decoder(self.0.value_of("input_format")) }
    pub fn encoder(&self) -> Box<Encode> { force_encoder(self.0.value_of("output_format")) }
}

pub mod parse {
    use clap::ArgMatches;
    use super::*;
    pub fn parse(args: &ArgMatches) {
        let env = Environment(args);
        let (context, mut decoder, mut input) = (env.context(), env.decoder(), env.input());
        for e in decoder.decode(&context, &mut input) {
            match e {
                Err(e) => { println!("Foo!"); error(Box::new(e)) },
                _ => ()
            }
        }
    }
}

pub mod convert {
    use clap::ArgMatches;
    use super::*;
    pub fn convert(args: &ArgMatches) {
        let env = Environment(args);
        let (context, mut decoder, mut input, encoder, mut output) =
            (env.context(), env.decoder(), env.input(), env.encoder(), env.output());

        for e in decoder.decode(&context, &mut input) {
            match e {
                Ok(e) => { let _ = encoder.encode(&context, &mut output, &e); },
                Err(e) => error(Box::new(e))
            }
        }
    }
}

pub mod seen {
    use clap::ArgMatches;
    use ilc::event::Event;
    use ilc::format::{ self, Encode };
    use super::*;
    pub fn seen(args: &ArgMatches) {
        let env = Environment(args);
        let (context, mut decoder, mut input, mut output) = (env.context(), env.decoder(), env.input(), env.output());

        let nick = args.value_of("nick").expect("Required argument <nick> not present");

        let mut last: Option<Event> = None;
        for e in decoder.decode(&context, &mut input) {
            let m = match e {
                Ok(m) => m,
                Err(err) => error(Box::new(err))
            };

            if m.ty.involves(nick)
            && last.as_ref().map_or(true, |last| m.time.as_timestamp() > last.time.as_timestamp()) { last = Some(m) }
        }
        let encoder = format::weechat3::Weechat3;
        if let Some(ref m) = last {
            let _ = encoder.encode(&context, &mut output, m);
        }
    }
}

pub mod sort {
    use clap::ArgMatches;
    use ilc::event::Event;
    use super::*;
    pub fn sort(args: &ArgMatches) {
        let env = Environment(args);
        let (context, mut decoder, mut input, encoder, mut output) =
            (env.context(), env.decoder(), env.input(), env.encoder(), env.output());

        let mut events: Vec<Event> = decoder.decode(&context, &mut input)
            .flat_map(Result::ok)
            .collect();

        events.sort_by(|a, b| a.time.cmp(&b.time));
        for e in events {
            let _ = encoder.encode(&context, &mut output, &e);
        }
    }
}

pub mod dedup {
    use clap::ArgMatches;
    use ilc::event::NoTimeHash;
    use ::ageset::AgeSet;
    use super::*;
    pub fn dedup(args: &ArgMatches) {
        let env = Environment(args);
        let (context, mut decoder, mut input, encoder, mut output) =
            (env.context(), env.decoder(), env.input(), env.encoder(), env.output());

        let mut backlog = AgeSet::new();

        for e in decoder.decode(&context, &mut input) {
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

    }
}
