#[macro_use]
extern crate log;
extern crate ilc_base;
extern crate chrono;

use std::io::{BufRead, Write};
use std::borrow::{Cow, ToOwned};
use std::iter::Iterator;

use ilc_base::event::{Event, Time, Type};
use ilc_base::format::{rejoin, strip_one};
use ilc_base::{Context, Decode, Encode};

use log::LogLevel::Info;

use chrono::*;

#[derive(Copy, Clone)]
pub struct Energymech;

static TIME_FORMAT: &'static str = "%H:%M:%S";

pub struct Iter<'a> {
    context: &'a Context,
    input: &'a mut BufRead,
    buffer: Vec<u8>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = ilc_base::Result<Event<'a>>;
    fn next(&mut self) -> Option<ilc_base::Result<Event<'a>>> {
        fn parse_time(context: &Context, time: &str) -> Time {
            let h = time[1..3].parse::<u32>().unwrap();
            let m = time[4..6].parse::<u32>().unwrap();
            let s = time[7..9].parse::<u32>().unwrap();
            if let Some(date) = context.override_date {
                Time::Timestamp(context.timezone_in
                                       .from_local_date(&date)
                                       .and_time(NaiveTime::from_hms(h, m, s))
                                       .single()
                                       .expect("Transformed log times can't be represented, due \
                                                to timezone transitions")
                                       .timestamp())
            } else {
                Time::Hms(h as u8, m as u8, s as u8)
            }
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

            let len = tokens.len();

            // [21:53:49] * Foo emotes
            if len >= 4 && tokens[1] == "*" {
                return Some(Ok(Event {
                    ty: Type::Action {
                        from: tokens[2].to_owned().into(),
                        content: rejoin(&tokens[3..], &split_tokens[3..]),
                    },
                    time: parse_time(&self.context, tokens[0]),
                    channel: self.context.channel.clone().map(Into::into),
                }));
            }

            if len >= 2 && tokens[1] == "***" {
                // [21:24:57] *** Foo is now known as Bar
                if len >= 8 && tokens[3] == "is" && tokens[4] == "now" && tokens[5] == "known" &&
                   tokens[6] == "as" {
                    return Some(Ok(Event {
                        ty: Type::Nick {
                            old_nick: tokens[2].to_owned().into(),
                            new_nick: tokens[7].to_owned().into(),
                        },
                        time: parse_time(&self.context, tokens[0]),
                        channel: self.context.channel.clone().map(Into::into),
                    }));
                }

                // [23:21:17] *** Paster was kicked by fripp.mozilla.org (Channel flood triggered (limit is 5 lines in 3 secs))
                if len >= 8 && tokens[3] == "was" && tokens[4] == "kicked" && tokens[5] == "by" {
                    return Some(Ok(Event {
                        ty: Type::Kick {
                            kicked_nick: tokens[2].to_owned().into(),
                            kicking_nick: Some(tokens[6].to_owned().into()),
                            kick_message: Some(strip_one(&rejoin(&tokens[7..],
                                                                 &split_tokens[7..]))
                                                   .into()),
                        },
                        time: parse_time(&self.context, tokens[0]),
                        channel: self.context.channel.clone().map(Into::into),
                    }));
                }

                // [21:49:59] *** ChanServ sets mode: +v Foo
                if len >= 7 && tokens[3] == "sets" && tokens[4] == "mode:" {
                    return Some(Ok(Event {
                        ty: Type::Mode {
                            nick: Some(tokens[2].to_owned().into()),
                            mode: tokens[5].to_owned().into(),
                            masks: rejoin(&tokens[6..], &split_tokens[6..]).to_owned().into(),
                        },
                        time: parse_time(&self.context, tokens[0]),
                        channel: self.context.channel.clone().map(Into::into),
                    }));
                }

                // [21:49:59] *** Joins: Foo (host@some.mask)
                if len >= 5 && tokens[2] == "Joins:" {
                    return Some(Ok(Event {
                        ty: Type::Join {
                            nick: tokens[3].to_owned().into(),
                            mask: Some(strip_one(tokens[4]).into()),
                        },
                        time: parse_time(&self.context, tokens[0]),
                        channel: self.context.channel.clone().map(Into::into),
                    }));
                }

                // [03:52:11] *** Parts: Foo (some@host.mask) (A reason? Nah...)
                if len >= 6 && tokens[2] == "Parts:" {
                    return Some(Ok(Event {
                        ty: Type::Part {
                            nick: tokens[3].to_owned().into(),
                            mask: Some(strip_one(tokens[4]).into()),
                            reason: Some(strip_one(&rejoin(&tokens[5..], &split_tokens[5..]))
                                             .into()),
                        },
                        time: parse_time(&self.context, tokens[0]),
                        channel: self.context.channel.clone().map(Into::into),
                    }));
                }

                // [03:48:33] *** Quits: Foo (just@a.hostmask) (Ping timeout: 42 seconds)
                if len >= 6 && tokens[2] == "Quits:" {
                    return Some(Ok(Event {
                        ty: Type::Quit {
                            nick: tokens[3].to_owned().into(),
                            mask: Some(strip_one(tokens[4]).into()),
                            reason: Some(strip_one(&rejoin(&tokens[5..], &split_tokens[5..]))
                                             .into()),
                        },
                        time: parse_time(&self.context, tokens[0]),
                        channel: self.context.channel.clone().map(Into::into),
                    }));
                }

                // [09:44:56] *** Foo changes topic to 'Hi there, why are you reading this comment?'
                if len >= 7 && tokens[3] == "changes" && tokens[4] == "topic" && tokens[5] == "to" {
                    return Some(Ok(Event {
                        ty: Type::TopicChange {
                            nick: Some(tokens[2].to_owned().into()),
                            new_topic: strip_one(&rejoin(&tokens[6..], &split_tokens[6..])).into(),
                        },
                        time: parse_time(&self.context, tokens[0]),
                        channel: self.context.channel.clone().map(Into::into),
                    }));
                }
            }

            // [03:36:01] <Foo> Just some moderately ugly code, nothing special to be found here.
            if len >= 3 && tokens[1].starts_with('<') && tokens[1].ends_with('>') {
                return Some(Ok(Event {
                    ty: Type::Msg {
                        from: strip_one(tokens[1]).into(),
                        content: rejoin(&tokens[2..], &split_tokens[2..]),
                    },
                    time: parse_time(&self.context, tokens[0]),
                    channel: self.context.channel.clone().map(Into::into),
                }));
            }

            // [10:25:22] -playbot- true
            if len >= 3 && tokens[1].starts_with('-') && tokens[1].ends_with('-') {
                return Some(Ok(Event {
                    ty: Type::Notice {
                        from: strip_one(tokens[1]).into(),
                        content: rejoin(&tokens[2..], &split_tokens[2..]),
                    },
                    time: parse_time(&self.context, tokens[0]),
                    channel: self.context.channel.clone().map(Into::into),
                }));
            }
            if option_env!("FUSE").is_some() {
                panic!("Shouldn't reach here, this is a bug!")
            }
        }
    }
}

impl Decode for Energymech {
    fn decode<'a>(&'a self,
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

impl Encode for Energymech {
    fn encode<'a>(&'a self,
                  context: &'a Context,
                  mut output: &'a mut Write,
                  event: &'a Event)
                  -> ilc_base::Result<()> {
        match event {
            &Event { ty: Type::Msg { ref from, ref content }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "[{}] <{}> {}",
                              time.with_format(&context.timezone_out, TIME_FORMAT),
                              from,
                              content))
            }
            &Event { ty: Type::Notice { ref from, ref content }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "[{}] -{}- {}",
                              time.with_format(&context.timezone_out, TIME_FORMAT),
                              from,
                              content))
            }
            &Event { ty: Type::Action { ref from, ref content }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "[{}] * {} {}",
                              time.with_format(&context.timezone_out, TIME_FORMAT),
                              from,
                              content))
            }
            &Event { ty: Type::Nick { ref old_nick, ref new_nick }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "[{}] *** {} is now known as {}",
                              time.with_format(&context.timezone_out, TIME_FORMAT),
                              old_nick,
                              new_nick))
            }
            &Event { ty: Type::Mode { ref nick, ref mode, ref masks }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "[{}] *** {} sets mode: {} {}",
                              time.with_format(&context.timezone_out, TIME_FORMAT),
                              nick.as_ref().expect("Nickname not present, but required."),
                              mode,
                              masks))
            }
            &Event { ty: Type::Join { ref nick, ref mask }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "[{}] *** Joins: {} ({})",
                              time.with_format(&context.timezone_out, TIME_FORMAT),
                              nick,
                              mask.as_ref().expect("Mask not present, but required.")))
            }
            &Event { ty: Type::Part { ref nick, ref mask, ref reason }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "[{}] *** Parts: {} ({}) ({})",
                              time.with_format(&context.timezone_out, TIME_FORMAT),
                              nick,
                              mask.as_ref().expect("Mask not present, but required."),
                              reason.as_ref().unwrap_or(&Cow::Borrowed(""))))
            }
            &Event { ty: Type::Quit { ref nick, ref mask, ref reason }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "[{}] *** Quits: {} ({}) ({})",
                              time.with_format(&context.timezone_out, TIME_FORMAT),
                              nick,
                              mask.as_ref().expect("Mask not present, but required."),
                              reason.as_ref().expect("Reason not present, but required.")))
            }
            &Event { ty: Type::TopicChange { ref nick, ref new_topic }, ref time, .. } => {
                try!(writeln!(&mut output,
                              "[{}] *** {} changes topic to '{}'",
                              time.with_format(&context.timezone_out, TIME_FORMAT),
                              nick.as_ref().expect("Nick not present, but required."),
                              new_topic))
            }
            _ => {
                if option_env!("FUSE").is_some() {
                    panic!("Shouldn't reach here, this is a bug!")
                }
                ()
            }
        }
        Ok(())
    }
}
