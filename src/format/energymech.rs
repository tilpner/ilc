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
use std::borrow::{ ToOwned, Cow };
use std::iter::{ Iterator };

use event::{ Event, Type, Time };
use context::Context;
use format::{ Encode, Decode, rejoin, strip_one };

use l::LogLevel::Info;

use chrono::*;

pub struct Energymech;

static TIME_FORMAT: &'static str = "%H:%M:%S";

pub struct Iter<'a> {
    context: &'a Context,
    input: &'a mut BufRead,
    buffer: Vec<u8>
}

impl<'a> Iterator for Iter<'a> {
    type Item = ::Result<Event<'a>>;
    fn next(&mut self) -> Option<::Result<Event<'a>>> {
        fn parse_time(context: &Context, time: &str) -> Time {
            let h = time[1..3].parse::<u32>().unwrap();
            let m = time[4..6].parse::<u32>().unwrap();
            let s = time[7..9].parse::<u32>().unwrap();
            if let Some(date) = context.override_date {
                Time::Timestamp(context.timezone.from_local_date(&date)
                    .and_time(NaiveTime::from_hms(h, m, s))
                    .single()
                    .expect("Transformed log times can't be represented, due to timezone transitions")
                    .timestamp())
            } else {
                Time::Hms(h as u8, m as u8, s as u8)
            }
        }

        loop {
            self.buffer.clear();
            match self.input.read_until(b'\n', &mut self.buffer) {
                Ok(0) | Err(_) => return None,
                Ok(_) => ()
            }

            let buffer = String::from_utf8_lossy(&self.buffer);

            let mut split_tokens: Vec<char> = Vec::new();
            let tokens = buffer.split( |c: char| {
                if c.is_whitespace() { split_tokens.push(c); true } else { false }
            }).collect::<Vec<_>>();

            if log_enabled!(Info) {
                info!("Original:  `{}`", buffer);
                info!("Parsing:   {:?}", tokens);
            }

            match &tokens[..tokens.len() - 1] {
                [time, "*", nick, content..] => return Some(Ok(Event {
                    ty: Type::Action {
                        from: nick.to_owned().into(),
                        content: rejoin(content, &split_tokens[3..])
                    },
                    time: parse_time(&self.context, time),
                    channel: self.context.channel.clone().map(Into::into)
                })),
                [time, "***", old, "is", "now", "known", "as", new] => return Some(Ok(Event {
                    ty: Type::Nick {
                        old_nick: old.to_owned().into(),
                        new_nick: new.to_owned().into()
                    },
                    time: parse_time(&self.context, time),
                    channel: self.context.channel.clone().map(Into::into)

                })),
                [time, "***", nick, "sets", "mode:", mode, masks..] => return Some(Ok(Event {
                    ty: Type::Mode {
                        nick: Some(nick.to_owned().into()),
                        mode: mode.to_owned().into(),
                        masks: rejoin(&masks, &split_tokens[6..]).to_owned().into()
                    },
                    time: parse_time(&self.context, time),
                    channel: self.context.channel.clone().map(Into::into)

                })),
                [time, "***", "Joins:", nick, host] => return Some(Ok(Event {
                    ty: Type::Join {
                        nick: nick.to_owned().into(),
                        mask: Some(strip_one(host).into())
                    },
                    time: parse_time(&self.context, time),
                    channel: self.context.channel.clone().map(Into::into)

                })),
                [time, "***", "Parts:", nick, host, reason..] => return Some(Ok(Event {
                    ty: Type::Part {
                        nick: nick.to_owned().into(),
                        mask: Some(strip_one(host).into()),
                        reason: Some(strip_one(&rejoin(reason, &split_tokens[5..])).into())
                    },
                    time: parse_time(&self.context, time),
                    channel: self.context.channel.clone().map(Into::into)

                })),
                [time, "***", "Quits:", nick, host, reason..] => return Some(Ok(Event {
                    ty: Type::Quit {
                        nick: nick.to_owned().into(),
                        mask: Some(strip_one(host).into()),
                        reason: Some(strip_one(&rejoin(reason, &split_tokens[5..])).into())
                    },
                    time: parse_time(&self.context, time),
                    channel: self.context.channel.clone().map(Into::into)

                })),
                [time, "***", nick, "changes", "topic", "to", topic..] => return Some(Ok(Event {
                    ty: Type::TopicChange {
                        nick: Some(nick.to_owned().into()),
                        new_topic: strip_one(&rejoin(topic, &split_tokens[6..])).into()
                    },
                    time: parse_time(&self.context, time),
                    channel: self.context.channel.clone().map(Into::into)

                })),
                [time, nick, content..]
                    if nick.starts_with('<') && nick.ends_with('>')
                    => return Some(Ok(Event {
                    ty: Type::Msg {
                        from: strip_one(nick).into(),
                        content: rejoin(content, &split_tokens[2..])
                    },
                    time: parse_time(&self.context, time),
                    channel: self.context.channel.clone().map(Into::into)
                })),
                _ => ()
            }
        }
    }
}

impl Decode for Energymech {
    fn decode<'a>(&'a mut self, context: &'a Context, input: &'a mut BufRead) -> Box<Iterator<Item = ::Result<Event<'a>>> + 'a> {
        Box::new(Iter {
            context: context,
            input: input,
            buffer: Vec::new()
        })
    }
}

impl Encode for Energymech {
    fn encode<'a>(&'a self, context: &'a Context, mut output: &'a mut Write, event: &'a Event) -> ::Result<()> {
        match event {
            &Event { ty: Type::Msg { ref from, ref content }, ref time, .. } => {
                try!(writeln!(&mut output, "[{}] <{}> {}",
                    time.with_format(&context.timezone, TIME_FORMAT), from, content))
            },
            &Event { ty: Type::Action { ref from, ref content }, ref time, .. } => {
                try!(writeln!(&mut output, "[{}] * {} {}",
                    time.with_format(&context.timezone, TIME_FORMAT), from, content))
            },
            &Event { ty: Type::Nick { ref old_nick, ref new_nick }, ref time, .. } => {
                try!(writeln!(&mut output, "[{}] *** {} is now known as {}",
                    time.with_format(&context.timezone, TIME_FORMAT), old_nick, new_nick))
            },
            &Event { ty: Type::Mode { ref nick, ref mode, ref masks }, ref time, .. } => {
                try!(writeln!(&mut output, "[{}] *** {} sets mode: {} {}",
                    time.with_format(&context.timezone, TIME_FORMAT),
                    nick.as_ref().expect("Nickname not present, but required."),
                    mode, masks))
            },
            &Event { ty: Type::Join { ref nick, ref mask }, ref time, .. } => {
                try!(writeln!(&mut output, "[{}] *** Joins: {} ({})",
                    time.with_format(&context.timezone, TIME_FORMAT), nick,
                    mask.as_ref().expect("Mask not present, but required.")))
            },
            &Event { ty: Type::Part { ref nick, ref mask, ref reason }, ref time, .. } => {
                try!(writeln!(&mut output, "[{}] *** Parts: {} ({}) ({})",
                    time.with_format(&context.timezone, TIME_FORMAT), nick,
                    mask.as_ref().expect("Mask not present, but required."),
                    reason.as_ref().unwrap_or(&Cow::Borrowed(""))))
            },
            &Event { ty: Type::Quit { ref nick, ref mask, ref reason }, ref time, .. } => {
                try!(writeln!(&mut output, "[{}] *** Quits: {} ({}) ({})",
                    time.with_format(&context.timezone, TIME_FORMAT), nick,
                    mask.as_ref().expect("Mask not present, but required."),
                    reason.as_ref().expect("Reason not present, but required.")))
            },
            &Event { ty: Type::TopicChange { ref nick, ref new_topic }, ref time, .. } => {
                try!(writeln!(&mut output, "[{}] *** {} changes topic to '{}'",
                    time.with_format(&context.timezone, TIME_FORMAT),
                    nick.as_ref().expect("Nick not present, but required."),
                    new_topic))
            },
            _ => ()
        }
        Ok(())
    }
}
