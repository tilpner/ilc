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
use std::borrow::{ ToOwned, Cow, IntoCow };
use std::iter::{ Iterator };
use std::marker::PhantomData;

use event::{ Event, Type, Time };
use context::Context;
use format::{ Encode, Decode, rejoin, strip_one };

use l::LogLevel::Info;

use chrono::*;

pub struct Weechat3;

static TIME_DATE_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

pub struct Iter<'a, R: 'a> where R: BufRead {
    _phantom: PhantomData<&'a ()>,
    context: &'a Context,
    input: R,
    buffer: String
}

impl<'a: 'b, 'b, R: 'a> Iterator for Iter<'a, R> where R: BufRead {
    type Item = ::Result<Event<'static>>;
    fn next(&mut self) -> Option<::Result<Event<'static>>> {
        fn parse_time<'b, 'c>(c: &Context, date: &'b str, time: &'c str) -> Time {
            Time::from_format(&c.timezone, &format!("{} {}", date, time), TIME_DATE_FORMAT)
        }

        loop {
            self.buffer.clear();
            match self.input.read_line(&mut self.buffer) {
                Ok(0) | Err(_) => return None,
                Ok(_) => ()
            }

            let mut split_tokens: Vec<char> = Vec::new();
            let tokens: Vec<&'b str> = self.buffer.split(|c: char| {
                if c.is_whitespace() { split_tokens.push(c); true } else { false }
            }).collect::<Vec<_>>();

            /*if log_enabled!(Info) {
                info!("Original:  `{}`", self.buffer);
                info!("Parsing:   {:?}", tokens);
            }*/

            match &tokens[..tokens.len() - 1] as &'b [&'b str] {
                /*[date, time, "-->", nick, host, "has", "joined", channel, _..]
                => return Some(Ok(Event {
                    ty: Type::Join {
                        nick: nick.to_owned(),
                        mask: Some(strip_one(host)),
                        time: timestamp(date, time)
                    },
                    channel: Some(channel.into_cow()),
                })),
                [date, time, "<--", nick, host, "has", "left", channel, reason..]
                => return Some(Ok(Event {
                    ty: Type::Part {
                        nick: nick.to_owned(),
                        mask: Some(strip_one(host)),
                        reason: Some(strip_one(&rejoin(reason, &split_tokens[8..]))),
                    },
                    channel: Some(channel.to_owned()),
                    time: timestamp(date, time)
                })),
                [date, time, "<--", nick, host, "has", "quit", reason..]
                => return Some(Ok(Event {
                    ty: Type::Quit {
                        nick: nick.to_owned(),
                        mask: Some(strip_one(host)),
                        reason: Some(strip_one(&rejoin(reason, &split_tokens[7..]))),
                    }
                })),
                [date, time, "--", notice, content..]
                    if notice.starts_with("Notice(")
                => return Some(Ok(Event {
                    ty: Type::Notice {
                        nick: notice["Notice(".len()..notice.len() - 2].to_owned(),
                        content: rejoin(content, &split_tokens[4..]),
                        time: timestamp(date, time)
                    }
                })),
                [date, time, "--", "irc:", "disconnected", "from", "server", _..]
                => return Some(Ok(Event {
                        ty: Type::Disconnect {
                        time: timestamp(date, time)
                    }
                })),
                [date, time, "--", nick, verb, "now", "known", "as", new_nick]
                    if verb == "is" || verb == "are"
                => return Some(Ok(Event {
                    ty: Type::Nick {
                    old: nick.to_owned(), new: new_nick.to_owned(), time: timestamp(date, time)
                    }
                })),*/
                [date, time, sp, "*", nick, msg..]
                    if sp.clone().is_empty()
                => return Some(Ok(Event {
                    ty: Type::Action {
                        from: nick.clone().into_cow(),
                        content: rejoin(msg, &split_tokens[5..]),
                    },
                    time: parse_time(&self.context, &date.clone().to_owned(), &time.clone().to_owned()),
                    channel: None
                })),
                /*[date, time, nick, msg..]
                => return Some(Ok(Event {
                    ty: Type::Msg {
                        from: nick.into(),
                        content: rejoin(msg, &split_tokens[3..]),
                    },
                    time: parse_time(&self.context, &date, &time),
                    channel: None
                })),*/
                _ => ()
            }
        }
    }
}

impl<'a, R: 'a> Decode<'static, R, Iter<'a, R>> for Weechat3 where R: BufRead {
    fn decode(&'a mut self, context: &'a Context, input: R) -> Iter<R> {
        Iter {
            _phantom: PhantomData,
            context: context,
            input: input,
            buffer: String::new()
        }
    }
}

impl<'a, W> Encode<'a, W> for Weechat3 where W: Write {
    fn encode(&'a self, context: &'a Context, mut output: W, event: &'a Event) -> ::Result<()> {
        fn date(t: i64) -> String {
            format!("{}", UTC.timestamp(t, 0).format(TIME_DATE_FORMAT))
        }
        match event {
            &Event { ty: Type::Msg { ref from, ref content, .. }, ref time, .. } => {
                try!(writeln!(&mut output, "{}\t{}\t{}",
                              time.with_format(&context.timezone, TIME_DATE_FORMAT), from, content))
            },
            &Event { ty: Type::Action { ref from, ref content, .. }, ref time, .. } => {
                try!(writeln!(&mut output, "{}\t *\t{} {}",
                              time.with_format(&context.timezone, TIME_DATE_FORMAT), from, content))
            },
            /*&Event::Join { ref nick, ref mask, ref channel, ref time } => {
                try!(writeln!(&mut output, "{}\t-->\t{} ({}) has joined {}",
                date(*time), nick, mask, channel))
            },
            &Event::Part { ref nick, ref mask, ref channel, ref time, ref reason } => {
                try!(write!(&mut output, "{}\t<--\t{} ({}) has left {}",
                date(*time), nick, mask, channel));
                if reason.len() > 0 {
                    try!(write!(&mut output, " ({})", reason));
                }
                try!(write!(&mut output, "\n"))
            },
            &Event::Quit { ref nick, ref mask, ref time, ref reason } => {
                try!(write!(&mut output, "{}\t<--\t{} ({}) has quit", date(*time), nick, mask));
                if reason.len() > 0 {
                    try!(write!(&mut output, " ({})", reason));
                }
                try!(write!(&mut output, "\n"))
            },
            &Event::Disconnect { ref time } => {
                try!(writeln!(&mut output, "{}\t--\tirc: disconnected from server", date(*time)))
            },
            &Event::Notice { ref nick, ref content, ref time } => {
                try!(writeln!(&mut output, "{}\t--\tNotice({}): {}", date(*time), nick, content))
            },*/
            _ => ()
        }
        Ok(())
    }
}
