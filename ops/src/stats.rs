//! Per-nick word/line statistics
use ilc_base::{self, Context, Decode, Event, Time};
use ilc_base::event::Type;

use std::collections::HashMap;
use std::io::BufRead;

use chrono::{Datelike, NaiveDateTime, Timelike};

use serde::ser::{MapVisitor, Serialize, Serializer};

pub type Day = [u32; 24];
/// Weeks start on mondays.
pub type Week = [Day; 7];

pub struct Stats {
    pub freqs: HashMap<String, NickStat>,
    pub week: Week,
}

impl Serialize for Stats {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        struct Visitor<'a>(&'a Stats);
        impl<'a> MapVisitor for Visitor<'a> {
            fn visit<S>(&mut self, s: &mut S) -> Result<Option<()>, S::Error>
                where S: Serializer
            {
                try!(s.serialize_struct_elt("freqs", &self.0.freqs));
                try!(s.serialize_struct_elt("week", &self.0.week));
                Ok(None)
            }

            fn len(&self) -> Option<usize> {
                Some(1)
            }
        }
        s.serialize_struct("Stats", Visitor(self))
    }
}

pub struct NickStat {
    pub lines: u32,
    pub alpha_lines: u32,
    pub words: u32,
}

impl Serialize for NickStat {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        struct Visitor<'a>(&'a NickStat);
        impl<'a> MapVisitor for Visitor<'a> {
            fn visit<S>(&mut self, s: &mut S) -> Result<Option<()>, S::Error>
                where S: Serializer
            {
                try!(s.serialize_struct_elt("lines", self.0.lines));
                try!(s.serialize_struct_elt("alpha_lines", self.0.alpha_lines));
                try!(s.serialize_struct_elt("words", self.0.words));
                Ok(None)
            }

            fn len(&self) -> Option<usize> {
                Some(3)
            }
        }

        s.serialize_struct("NickStat", Visitor(self))
    }
}

fn words_alpha(s: &str) -> (u32, bool) {
    let mut alpha = false;
    let mut words = 0;
    for w in s.split_whitespace() {
        if !w.is_empty() {
            words += 1;
            if w.chars().any(char::is_alphabetic) {
                alpha = true
            }
        }
    }
    (words, alpha)
}

fn strip_nick(s: &str) -> &str {
    if s.is_empty() {
        return s;
    }
    match s.as_bytes()[0] {
        b'~' | b'&' | b'@' | b'%' | b'+' => &s[1..],
        _ => s,
    }
    .trim_right_matches('_')
}

/// Return all active nicks, with lines, words and words per lines counted.
pub fn stats(ctx: &Context, input: &mut BufRead, decoder: &mut Decode) -> ilc_base::Result<Stats> {
    let mut freqs: HashMap<String, NickStat> = HashMap::new();
    let mut week: Week = [[0; 24]; 7];

    for e in decoder.decode(&ctx, input) {
        let m = try!(e);
        match m {
            Event { ty: Type::Msg { ref from, ref content, .. }, ref time, .. } => {
                if let &Time::Timestamp(stamp) = time {
                    let date = NaiveDateTime::from_timestamp(stamp, 0);
                    let dow = date.weekday().num_days_from_monday() as usize;
                    let hour = date.hour() as usize;
                    week[dow][hour] += 1;
                }

                let nick = strip_nick(from);
                if freqs.contains_key(nick) {
                    let p: &mut NickStat = freqs.get_mut(nick).unwrap();
                    let (words, alpha) = words_alpha(content);
                    p.lines += 1;
                    if alpha {
                        p.alpha_lines += 1
                    }
                    p.words += words;
                } else {
                    let (words, alpha) = words_alpha(content);
                    freqs.insert(nick.to_owned(),
                                 NickStat {
                                     lines: 1,
                                     alpha_lines: if alpha { 1 } else { 0 },
                                     words: words,
                                 });
                }
            }
            _ => (),
        }
    }

    Ok(Stats {
        freqs: freqs,
        week: week,
    })
}
