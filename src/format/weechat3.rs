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
use std::borrow::{ ToOwned };
use std::iter::{ Iterator };

use event::{ Event, Type, Time };
use context::Context;
use format::{ Encode, Decode, rejoin, strip_one };

use l::LogLevel::Info;

pub struct Weechat3;

static TIME_DATE_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

pub struct Iter<'a, R: 'a> where R: BufRead {
    context: &'a Context,
    input: R,
    buffer: String
}

impl<'a, R: 'a> Iterator for Iter<'a, R> where R: BufRead {
    type Item = ::Result<Event<'a>>;
    fn next(&mut self) -> Option<::Result<Event<'a>>> {
        fn parse_time(c: &Context, date: &str, time: &str) -> Time {
            Time::from_format(&c.timezone, &format!("{} {}", date, time), TIME_DATE_FORMAT)
        }

        loop {
            self.buffer.clear();
            match self.input.read_line(&mut self.buffer) {
                Ok(0) | Err(_) => return None,
                Ok(_) => ()
            }

            let mut split_tokens: Vec<char> = Vec::new();
            let tokens = self.buffer.split(|c: char| {
                if c.is_whitespace() { split_tokens.push(c); true } else { false }
            }).collect::<Vec<_>>();

            if log_enabled!(Info) {
                info!("Original:  `{}`", self.buffer);
                info!("Parsing:   {:?}", tokens);
            }

            match &tokens[..tokens.len() - 1] {
                [date, time, "-->", nick, host, "has", "joined", channel, _..]
                => return Some(Ok(Event {
                    ty: Type::Join {
                        nick: nick.to_owned().into(),
                        mask: Some(strip_one(host).into()),
                    },
                    channel: Some(channel.to_owned().into()),
                    time: parse_time(&self.context, date, time)
                })),
                [date, time, "<--", nick, host, "has", "left", channel, reason..]
                => return Some(Ok(Event {
                    ty: Type::Part {
                        nick: nick.to_owned().into(),
                        mask: Some(strip_one(host).into()),
                        reason: Some(strip_one(&rejoin(reason, &split_tokens[8..])).into()),
                    },
                    channel: Some(channel.to_owned().into()),
                    time: parse_time(&self.context, date, time)
                })),
                [date, time, "<--", nick, host, "has", "quit", reason..]
                => return Some(Ok(Event {
                    ty: Type::Quit {
                        nick: nick.to_owned().into(),
                        mask: Some(strip_one(host).into()),
                        reason: Some(strip_one(&rejoin(reason, &split_tokens[7..])).into()),
                    },
                    time: parse_time(&self.context, date, time),
                    channel: self.context.channel.clone().map(Into::into)
                })),
                [date, time, "--", notice, content..]
                    if notice.starts_with("Notice(")
                => return Some(Ok(Event {
                    ty: Type::Notice {
                        from: notice["Notice(".len()..notice.len() - 2].to_owned().into(),
                        content: rejoin(content, &split_tokens[4..]),
                    },
                    time: parse_time(&self.context, date, time),
                    channel: self.context.channel.clone().map(Into::into)
                })),
                [date, time, "--", "irc:", "disconnected", "from", "server", _..]
                => return Some(Ok(Event {
                    ty: Type::Disconnect,
                    time: parse_time(&self.context, date, time),
                    channel: self.context.channel.clone().map(Into::into)
                })),
                [date, time, "--", nick, verb, "now", "known", "as", new_nick]
                    if verb == "is" || verb == "are"
                => return Some(Ok(Event {
                    ty: Type::Nick {
                        old_nick: nick.to_owned().into(),
                        new_nick: new_nick.to_owned().into()
                    },
                    time: parse_time(&self.context, date, time),
                    channel: self.context.channel.clone().map(Into::into)
                })),
                [date, time, sp, "*", nick, msg..]
                    if sp.clone().is_empty()
                => return Some(Ok(Event {
                    ty: Type::Action {
                        from: nick.to_owned().into(),
                        content: rejoin(msg, &split_tokens[5..]),
                    },
                    time: parse_time(&self.context, date, time),
                    channel: self.context.channel.clone().map(Into::into)
                })),
                [date, time, nick, msg..]
                => return Some(Ok(Event {
                    ty: Type::Msg {
                        from: nick.to_owned().into(),
                        content: rejoin(msg, &split_tokens[3..]),
                    },
                    time: parse_time(&self.context, date, time),
                    channel: self.context.channel.clone().map(Into::into)
                })),
                _ => ()
            }
        }
    }
}

impl<'a, R: 'a> Decode<'a, R, Iter<'a, R>> for Weechat3 where R: BufRead {
    fn decode(&'a mut self, context: &'a Context, input: R) -> Iter<R> {
        Iter {
            context: context,
            input: input,
            buffer: String::new()
        }
    }
}

impl<'a, W> Encode<'a, W> for Weechat3 where W: Write {
    fn encode(&'a self, context: &'a Context, mut output: W, event: &'a Event) -> ::Result<()> {
        match event {
            &Event { ty: Type::Msg { ref from, ref content, .. }, ref time, .. } => {
                try!(writeln!(&mut output, "{}\t{}\t{}",
                    time.with_format(&context.timezone, TIME_DATE_FORMAT), from, content))
            },
            &Event { ty: Type::Action { ref from, ref content, .. }, ref time, .. } => {
                try!(writeln!(&mut output, "{}\t *\t{} {}",
                    time.with_format(&context.timezone, TIME_DATE_FORMAT), from, content))
            },
            &Event { ty: Type::Join { ref nick, ref mask, .. }, ref channel, ref time } => {
                try!(writeln!(&mut output, "{}\t-->\t{} ({}) has joined {}",
                    time.with_format(&context.timezone, TIME_DATE_FORMAT), nick,
                    mask.as_ref().expect("Hostmask not present, but required."),
                    channel.as_ref().expect("Channel not present, but required.")))
            },
            &Event { ty: Type::Part { ref nick, ref mask, ref reason }, ref channel, ref time } => {
                try!(write!(&mut output, "{}\t<--\t{} ({}) has left {}",
                    time.with_format(&context.timezone, TIME_DATE_FORMAT), nick,
                    mask.as_ref().expect("Hostmask not present, but required."),
                    channel.as_ref().expect("Channel not present, but required.")));
                if reason.is_some() && reason.as_ref().unwrap().len() > 0 {
                    try!(write!(&mut output, " ({})", reason.as_ref().unwrap()));
                }
                try!(write!(&mut output, "\n"))
            },
            &Event { ty: Type::Quit { ref nick, ref mask, ref reason }, ref time, .. } => {
                try!(write!(&mut output, "{}\t<--\t{} ({}) has quit",
                    time.with_format(&context.timezone, TIME_DATE_FORMAT), nick,
                    mask.as_ref().expect("Hostmask not present, but required.")));
                if reason.is_some() && reason.as_ref().unwrap().len() > 0 {
                    try!(write!(&mut output, " ({})", reason.as_ref().unwrap()));
                }
                try!(write!(&mut output, "\n"))
            },
            &Event { ty: Type::Disconnect, ref time, .. } => {
                try!(writeln!(&mut output, "{}\t--\tirc: disconnected from server",
                    time.with_format(&context.timezone, TIME_DATE_FORMAT)))
            },
            &Event { ty: Type::Notice { ref from, ref content }, ref time, .. } => {
                try!(writeln!(&mut output, "{}\t--\tNotice({}): {}",
                    time.with_format(&context.timezone, TIME_DATE_FORMAT), from, content))
            },
            _ => ()
        }
        Ok(())
    }
}
