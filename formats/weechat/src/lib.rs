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

#[macro_use]
extern crate log;
extern crate ilc_base;

use std::io::{BufRead, Write};
use std::borrow::ToOwned;
use std::iter::Iterator;

use ilc_base::event::{Event, Time, Type};
use ilc_base::{Context, Decode, Encode};
use ilc_base::format::{rejoin, strip_one};

use log::LogLevel::Info;

pub struct Weechat;

static TIME_DATE_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

pub struct Iter<'a> {
    context: &'a Context,
    input: &'a mut BufRead,
    buffer: Vec<u8>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = ilc_base::Result<Event<'a>>;
    fn next(&mut self) -> Option<ilc_base::Result<Event<'a>>> {
        fn parse_time(c: &Context, date: &str, time: &str) -> Time {
            Time::from_format(&c.timezone, &format!("{} {}", date, time), TIME_DATE_FORMAT)
        }

        loop {
            self.buffer.clear();
            match self.input.read_until(b'\n', &mut self.buffer) {
                Ok(0) | Err(_) => return None,
                Ok(_) => (),
            }

            let buffer = String::from_utf8_lossy(&self.buffer);

            let mut split_tokens: Vec<char> = Vec::new();
            let tokens = buffer.split(|c: char| {
                                   if c.is_whitespace() {
                                       split_tokens.push(c);
                                       true
                                   } else {
                                       false
                                   }
                               })
                               .collect::<Vec<_>>();

            if log_enabled!(Info) {
                info!("Original:  `{}`", buffer);
                info!("Parsing:   {:?}", tokens);
            }

            // slice pattern matching is not stable as of Feb. 2016 and was replaced with
            // nested if-else chains in this module.

            // Don't match on the --> arrows, those are apparently often configured, and
            // that would break parsing for many users.

            if tokens[5] == "has" {
                // 2016-02-25 01:15:05 --> Foo (host@mask.foo) has joined #example
                if tokens[6] == "joined" {
                    return Some(Ok(Event {
                        ty: Type::Join {
                            nick: tokens[3].to_owned().into(),
                            mask: Some(strip_one(tokens[4]).into()),
                        },
                        channel: Some(tokens[7].to_owned().into()),
                        time: parse_time(&self.context, tokens[0], tokens[1]),
                    }));
                }
                // 2016-02-25 01:36:13 <-- Foo (host@mask.foo) has left #channel (Some reason)
                else if tokens[6] == "left" {
                    return Some(Ok(Event {
                        ty: Type::Part {
                            nick: tokens[3].to_owned().into(),
                            mask: Some(strip_one(&tokens[4]).into()),
                            reason: Some(strip_one(&rejoin(&tokens[8..], &split_tokens[8..]))
                                             .into()),
                        },
                        channel: Some(tokens[7].to_owned().into()),
                        time: parse_time(&self.context, tokens[0], tokens[1]),
                    }));
                }
                // 2016-02-25 01:38:55 <-- Foo (host@mask.foo) has quit (Some reason)
                else if tokens[6] == "quit" {
                    return Some(Ok(Event {
                        ty: Type::Quit {
                            nick: tokens[3].to_owned().into(),
                            mask: Some(strip_one(tokens[4]).into()),
                            reason: Some(strip_one(&rejoin(&tokens[7..], &split_tokens[7..]))
                                             .into()),
                        },
                        time: parse_time(&self.context, tokens[0], tokens[1]),
                        channel: self.context.channel.clone().map(Into::into),
                    }));
                }
            } else if tokens[2] == "--" {
                // 2016-02-25 04:32:15	--	Notice(playbot-veno): ""
                if tokens[3].starts_with("Notice(") {
                    return Some(Ok(Event {
                        ty: Type::Notice {
                            from: tokens[3]["Notice(".len()..tokens.len() - 2].to_owned().into(),
                            content: rejoin(&tokens[4..], &split_tokens[4..]),
                        },
                        time: parse_time(&self.context, tokens[0], tokens[1]),
                        channel: self.context.channel.clone().map(Into::into),
                    }));
                }
                // 2014-07-11 15:00:03	--	irc: disconnected from server
                else if tokens[3] == "irc:" && tokens[4] == "disconnected" && tokens[5] == "from" &&
                   tokens[6] == "server" {
                    return Some(Ok(Event {
                        ty: Type::Disconnect,
                        time: parse_time(&self.context, tokens[0], tokens[1]),
                        channel: self.context.channel.clone().map(Into::into),
                    }));
                }
                // 2014-07-11 15:00:03	--	Foo|afk is now known as Foo
                // 2015-05-09 13:56:05	--	You are now known as foo
                else if tokens[5] == "now" && tokens[6] == "known" && tokens[7] == "as" &&
                   (tokens[4] == "is" || tokens[4] == "are") {
                    return Some(Ok(Event {
                        ty: Type::Nick {
                            old_nick: tokens[3].to_owned().into(),
                            new_nick: tokens[8].to_owned().into(),
                        },
                        time: parse_time(&self.context, tokens[0], tokens[1]),
                        channel: self.context.channel.clone().map(Into::into),
                    }));
                }
            }
            // 2016-01-24 20:32:57	 *	nick emotes
            else if tokens[3] == "*" && tokens[2].is_empty() {
                return Some(Ok(Event {
                    ty: Type::Action {
                        from: tokens[4].to_owned().into(),
                        content: rejoin(&tokens[5..], &split_tokens[5..]),
                    },
                    time: parse_time(&self.context, tokens[0], tokens[1]),
                    channel: self.context.channel.clone().map(Into::into),
                }));
            }
            // 2016-01-24 20:32:25	nick	just some message
            else {
                return Some(Ok(Event {
                    ty: Type::Msg {
                        from: tokens[2].to_owned().into(),
                        content: rejoin(&tokens[3..], &split_tokens[3..]),
                    },
                    time: parse_time(&self.context, tokens[0], tokens[1]),
                    channel: self.context.channel.clone().map(Into::into),
                }));
            }
        }
    }
}

impl Decode for Weechat {
    fn decode<'a>(&'a mut self,
                  context: &'a Context,
                  input: &'a mut BufRead)
                  -> Box<Iterator<Item = ilc_base::Result<Event<'a>>> + 'a> {
        Box::new(Iter {
            context: context,
            input: input,
            buffer: Vec::new(),
        })
    }
}

impl Encode for Weechat {
    fn encode<'a>(&'a self,
                  context: &'a Context,
                  mut output: &'a mut Write,
                  event: &'a Event)
                  -> ilc_base::Result<()> {
        match event {
            &Event { ty: Type::Msg { ref from, ref content, .. }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "{}\t{}\t{}",
                              time.with_format(&context.timezone, TIME_DATE_FORMAT),
                              from,
                              content))
            }
            &Event { ty: Type::Action { ref from, ref content, .. }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "{}\t *\t{} {}",
                              time.with_format(&context.timezone, TIME_DATE_FORMAT),
                              from,
                              content))
            }
            &Event { ty: Type::Join { ref nick, ref mask, .. }, ref channel, ref time } => {
                try!(writeln!(&mut output,
                              "{}\t-->\t{} ({}) has joined {}",
                              time.with_format(&context.timezone, TIME_DATE_FORMAT),
                              nick,
                              mask.as_ref().expect("Hostmask not present, but required."),
                              channel.as_ref().expect("Channel not present, but required.")))
            }
            &Event { ty: Type::Part { ref nick, ref mask, ref reason }, ref channel, ref time } => {
                try!(write!(&mut output,
                            "{}\t<--\t{} ({}) has left {}",
                            time.with_format(&context.timezone, TIME_DATE_FORMAT),
                            nick,
                            mask.as_ref().expect("Hostmask not present, but required."),
                            channel.as_ref().expect("Channel not present, but required.")));
                if reason.is_some() && reason.as_ref().unwrap().len() > 0 {
                    try!(write!(&mut output, " ({})", reason.as_ref().unwrap()));
                }
                try!(write!(&mut output, "\n"))
            }
            &Event { ty: Type::Quit { ref nick, ref mask, ref reason }, ref time, .. } => {
                try!(write!(&mut output,
                            "{}\t<--\t{} ({}) has quit",
                            time.with_format(&context.timezone, TIME_DATE_FORMAT),
                            nick,
                            mask.as_ref().expect("Hostmask not present, but required.")));
                if reason.is_some() && reason.as_ref().unwrap().len() > 0 {
                    try!(write!(&mut output, " ({})", reason.as_ref().unwrap()));
                }
                try!(write!(&mut output, "\n"))
            }
            &Event { ty: Type::Disconnect, ref time, .. } => {
                try!(writeln!(&mut output,
                              "{}\t--\tirc: disconnected from server",
                              time.with_format(&context.timezone, TIME_DATE_FORMAT)))
            }
            &Event { ty: Type::Notice { ref from, ref content }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "{}\t--\tNotice({}): {}",
                              time.with_format(&context.timezone, TIME_DATE_FORMAT),
                              from,
                              content))
            }
            _ => (),
        }
        Ok(())
    }
}
