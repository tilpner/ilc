//! Log format conversion
use regex::Regex;

use ilc_base::{self, Context, Decode, Encode, Event};
use std::io::{BufRead, Write};

#[derive(Copy, Clone)]
pub enum Subject {
    Nick,
    Time,
    Type,
    Text,
}

pub enum Operator {
    Exactly(String),
    Contains(String),
    Matches(Regex),
    Equal(i64),
    Greater(i64),
    Less(i64),
}

pub struct Filter(pub Subject, pub Operator);

impl Filter {
    pub fn satisfied_by(&self, e: &Event) -> bool {
        use self::Subject::*;
        use self::Operator::*;

        match (self.0, &self.1) {
            (Nick, &Exactly(ref s)) => e.ty.actor().map_or(false, |e| e == s),
            (Nick, &Contains(ref s)) => e.ty.actor().map_or(false, |e| e.contains(s)),
            (Nick, &Matches(ref r)) => e.ty.actor().map_or(false, |e| r.is_match(e)),
            (Nick, _op) => false,
            (Time, &Equal(t)) => e.time.as_timestamp() == t,
            (Time, &Greater(t)) => e.time.as_timestamp() > t,
            (Time, &Less(t)) => e.time.as_timestamp() < t,
            (Time, _op) => false,
            (Type, &Exactly(ref s)) => e.ty.type_desc() == s,
            (Type, &Contains(ref s)) => e.ty.type_desc().contains(s),
            (Type, &Matches(ref r)) => r.is_match(e.ty.type_desc()),
            (Type, _op) => false,
            (Text, &Exactly(ref s)) => e.ty.text().map_or(false, |t| t == s),
            (Text, &Contains(ref s)) => e.ty.text().map_or(false, |t| t.contains(s)),
            (Text, &Matches(ref r)) => e.ty.text().map_or(false, |t| r.is_match(t)),
            (Text, _op) => false,
        }
    }
}

/// Convert from one format to another, not necessarily different, format. In combination with a
/// timezone offset, this can be used to correct the timestamps.
/// Will return `Err` and abort conversion if the decoder yields `Err` or re-encoding fails.
pub fn convert(ctx: &Context,
               input: &mut BufRead,
               decoder: &mut Decode,
               output: &mut Write,
               encoder: &Encode,
               filter: Option<Filter>,
               not: bool)
               -> ilc_base::Result<()> {
    if let Some(f) = filter {
        for e in decoder.decode(&ctx, input) {
            let e = try!(e);
            if not ^ f.satisfied_by(&e) {
                try!(encoder.encode(&ctx, output, &e))
            }
        }
    } else {
        // fast path for filter-less conversion, probably premature
        for e in decoder.decode(&ctx, input) {
            try!(encoder.encode(&ctx, output, &try!(e)));
        }
    }
    Ok(())
}
