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

use std::io::{ BufRead, Write };
use std::borrow::ToOwned;
use std::iter::{ Iterator };

use event::{ Event, rejoin };
use context::Context;
use format::{ Encode, Decode };

use l::LogLevel::Info;

use chrono::*;

pub struct Energymech;

static TIME_FORMAT: &'static str = "%H:%M:%S";

pub struct Iter<'a, R: 'a> where R: BufRead {
    context: &'a Context,
    input: R,
    buffer: String
}

impl<'a, R: 'a> Iterator for Iter<'a, R> where R: BufRead {
    type Item = ::Result<Event<'a>>;
    fn next(&mut self) -> Option<::Result<Event<'a>>> {
        fn timestamp(context: &Context, time: &str) -> i64 {
            context.timezone.from_local_date(&context.override_date)
                .and_time(NaiveTime::from_hms(time[0..2].parse::<u32>().unwrap(),
                                              time[3..5].parse::<u32>().unwrap(),
                                              time[6..8].parse::<u32>().unwrap()))
                .single()
                .expect("Transformed log times can't be represented, due to timezone transitions")
                .timestamp()
        }
        fn join(s: &[&str], splits: &[char]) -> String {
            let len = s.iter().map(|s| s.len()).sum();
            let mut out = s.iter().zip(splits.iter()).fold(String::with_capacity(len),
               |mut s, (b, &split)| { s.push_str(b); s.push(split); s });
            out.pop(); out
        }
        fn mask(s: &str) -> String {
            if s.len() >= 2 { s[1..(s.len() - 1)].to_owned() } else { String::new() }
        }

        loop {
            self.buffer.clear();
            match self.input.read_line(&mut self.buffer) {
                Ok(0) | Err(_) => return None,
                Ok(_) => ()
            }

            let mut split_tokens: Vec<char> = Vec::new();
            let tokens = self.buffer.split( |c: char| {
                if c.is_whitespace() { split_tokens.push(c); true } else { false }
            }).collect::<Vec<_>>();
            if log_enabled!(Info) {
                info!("Original:  `{}`", self.buffer);
                info!("Parsing:   {:?}", tokens);
            }
            match tokens[..tokens.len() - 1].as_ref() {
                [time, "*", nick, content..] => return Some(Ok(Event::Action {
                    from: nick.to_owned(), content: join(content, &split_tokens[3..]),
                    time: timestamp(&self.context, &mask(time))
                })),
                [time, "***", old, "is", "now", "known", "as", new] => return Some(Ok(Event::Nick {
                    old: old.to_owned(), new: new.to_owned(),
                    time: timestamp(&self.context, &mask(time))
                })),
                [time, "***", "Joins:", nick, host] => return Some(Ok(Event::Join {
                    nick: nick.to_owned(), mask: mask(host)
                })),
                [time, "***", "Quits:", nick, host, reason..] => return Some(Ok(Event::Quit {
                    nick: nick.to_owned(), mask: mask(host),
                    reason: mask(&join(reason, &split_tokens[5..])),
                    time: timestamp(&self.context, &mask(time))
                })),
                [time, nick, content..]
                    if nick.starts_with('<') && nick.ends_with('>')
                    => return Some(Ok(Event::Msg {
                    from: mask(nick), content: join(content, &split_tokens[2..]),
                    time: timestamp(&self.context, &mask(time))
                })),
                _ => ()
            }
        }
    }
}

impl<'a, R: 'a> Decode<'a, R, Iter<'a, R>> for Energymech where R: BufRead {
    fn decode(&'a mut self, context: &'a Context, input: R) -> Iter<R> {
        Iter {
            context: context,
            input: input,
            buffer: String::new()
        }
    }
}

impl<'a, W> Encode<'a, W> for Energymech where W: Write {
    fn encode(&'a self, context: &'a Context, mut output: W, event: &'a Event) -> ::Result<()> {
        fn date(t: i64) -> String {
            format!("[{}]", UTC.timestamp(t, 0).format(TIME_FORMAT))
        }
        match event {
            &Event::Msg { ref from, ref content, ref time } => {
                try!(writeln!(&mut output, "{} <{}> {}", date(*time), from, content))
            },
            &Event::Action { ref from, ref content, ref time } => {
                try!(writeln!(&mut output, "{} * {} {}", date(*time), from, content))
            },
            &Event::Nick { ref old, ref new, ref time } => {
                try!(writeln!(&mut output, "{} *** {} is now known as {}", date(*time), old, new))
            },
            &Event::Quit { ref nick, ref mask, ref reason, ref time } => {
                try!(writeln!(&mut output, "{} *** Quits: {} ({}) ({})", date(*time), nick, mask, reason))
            },
            _ => ()
        }
        Ok(())
    }
}
