use std::io::{ self, BufRead, Write };
use std::borrow::ToOwned;

use log::Event;
use format::{ Encode, Decode };

use regex::Regex;

use chrono::*;

pub struct Weechat3;

static NORMAL_LINE: Regex = regex!(r"^(\d+-\d+-\d+ \d+:\d+:\d+)\t[@%+~&]?([^ <-]\S+)\t(.*)");
static ACTION_LINE: Regex = regex!(r"^(\d+-\d+-\d+ \d+:\d+:\d+)\t \*\t(\S+) (.*)");
static OTHER_LINES: Regex = regex!(r"^(\d+-\d+-\d+ \d+:\d+:\d+)\s(?:--|<--|-->)\s(\S+)\s(\S+)\s(\S+)\s(\S+)\s(\S+)(.*)\n$");
//static OTHER_LINES: Regex = regex!(r"(.+)");

static TIME_DATE_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

pub struct Iter<R> where R: BufRead {
    input: R,
    buffer: String
}

impl<R> Iterator for Iter<R> where R: BufRead {
    type Item = ::Result<Event>;
    fn next(&mut self) -> Option<::Result<Event>> {
        fn time(s: &str) -> i64 {
            UTC.datetime_from_str(s, TIME_DATE_FORMAT).unwrap().timestamp()
        }
        fn mask(s: &str) -> String {
            s.trim_left_matches('(').trim_right_matches(')').to_owned()
        }

        loop {
            self.buffer.clear();
            match self.input.read_line(&mut self.buffer) {
                Ok(0) | Err(_) => return None,
                Ok(_) => ()
            }
            let line = &self.buffer;
            if let Some(cap) = NORMAL_LINE.captures(line) {
                return Some(Ok(Event::Msg {
                    from: cap.at(1).unwrap().to_owned(),
                    content: cap.at(2).unwrap().to_owned(),
                    time: time(cap.at(1).unwrap())
                }))
            } else if let Some(cap) = ACTION_LINE.captures(line) {
                return Some(Ok(Event::Action {
                    from: cap.at(1).unwrap().to_owned(),
                    content: cap.at(2).unwrap().to_owned(),
                    time: time(cap.at(1).unwrap())
                }))
            } else if let Some(cap) = OTHER_LINES.captures(line) {
                println!("{:?}", cap.iter().collect::<Vec<_>>());
                if cap.at(4) == Some("has") && cap.at(5) == Some("kicked") {
                    println!("    Matched Event::Kick");
                    return Some(Ok(Event::Kick {
                        kicked_nick: cap.at(6).unwrap().to_owned(),
                        kicking_nick: cap.at(3).unwrap().to_owned(),
                        kick_message: cap.at(4).unwrap().to_owned(),
                        time: time(cap.at(1).unwrap())
                    }))
                } else if cap.at(2) == Some("Topic") && cap.at(3) == Some("for") {
                    return Some(Ok(Event::Topic {
                        topic: { let mut s = cap.at(6).unwrap().to_string(); s.push_str(cap.at(7).unwrap()); s },
                        time: time(cap.at(1).unwrap())
                    }))
                } else if cap.at(3) == Some("has") && cap.at(5) == Some("changed") && cap.at(6) == Some("topic") {
                    return Some(Ok(Event::TopicChange {
                        new_topic: cap.at(5).unwrap().to_owned(),
                        time: time(cap.at(1).unwrap())
                    }))
                } else if cap.at(3) == Some("Mode") {
                    return Some(Ok(Event::Mode {
                        time: time(cap.at(1).unwrap())
                    }))
                } else if cap.at(4) == Some("has") && cap.at(5) == Some("joined") {
                    return Some(Ok(Event::Join {
                        nick: cap.at(2).unwrap().to_owned(),
                        channel: cap.at(6).unwrap().to_owned(),
                        mask: mask(cap.at(3).unwrap()),
                        time: time(cap.at(1).unwrap())
                    }))
                } else if cap.at(4) == Some("has") && cap.at(5) == Some("left") {
                    return Some(Ok(Event::Part {
                        nick: cap.at(2).unwrap().to_owned(),
                        channel: cap.at(6).unwrap().to_owned(),
                        reason: cap.at(7).unwrap().to_owned(),
                        mask: mask(cap.at(3).unwrap()),
                        time: time(cap.at(1).unwrap())
                    }))
                } else if cap.at(5) == Some("now") && cap.at(6) == Some("known") {
                    return Some(Ok(Event::Nick {
                        old: String::new(),
                        new: String::new(),
                        time: time(cap.at(1).unwrap())
                    }))
                }
            }
        }
    }
}

impl<R> Decode<R, Iter<R>> for Weechat3 where R: BufRead {
    fn decode(&mut self, input: R) -> Iter<R> {
        Iter {
            input: input,
            buffer: String::new()
        }
    }
}

impl<W> Encode<W> for Weechat3 where W: Write {
    fn encode(&self, mut output: W, event: &Event) -> io::Result<()> {
        fn date(t: i64) -> String {
            format!("{}", UTC.timestamp(t, 0).format(TIME_DATE_FORMAT))
        }
        match event {
            &Event::Msg { ref from, ref content, ref time } => {
                try!(write!(&mut output, "{}\t{}\t{}\n", date(*time), from, content))
            },
            &Event::Action { ref from, ref content, ref time } => {
                try!(write!(&mut output, "{}\t*\t{} {}\n", date(*time), from, content))
            },
            &Event::Join { ref nick, ref mask, ref channel, ref time } => {
                try!(write!(&mut output, "{}\t-->\t{} ({}) has joined {}\n",
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
            _ => ()
        }
        Ok(())
    }
}
