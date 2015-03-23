use std::io::BufRead;
use std::borrow::ToOwned;

use log::Event;
use format::Decode;

use regex::Regex;

use chrono::*;

pub struct Weechat3;

static NORMAL_LINE: Regex = regex!(r"^(\d+-\d+-\d+ \d+:\d+:\d+)\t[@%+~&]?([^ <-]\S+)\t(.*)");
static ACTION_LINE: Regex = regex!(r"^(\d+-\d+-\d+ \d+:\d+:\d+)\t \*\t(\S+) (.*)");
//static OTHER_LINES: Regex = regex!(r"^(\d+-\d+-\d+ \d+:\d+:\d+)\s(?:--|<--|-->)\s(\S+)\s(\S+)\s(\S+)\s(\S+)\s(\S+)\n$");
static OTHER_LINES: Regex = regex!(r"(.*)");

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

        println!("Reading line...");
        self.buffer.clear();
        match self.input.read_line(&mut self.buffer) {
            Ok(0) | Err(_) => return None,
            Ok(_) => ()
        }
        let line = &self.buffer;
        println!("Read line: {}", line);
        if let Some(cap) = NORMAL_LINE.captures(line) {
           return Some(Ok(Event::Msg {
               from: cap.at(1).unwrap().to_owned(),
               content: cap.at(2).unwrap().to_owned(),
               time: time(cap.at(0).unwrap())
           }))
        } else if let Some(cap) = ACTION_LINE.captures(line) {
            return Some(Ok(Event::Action {
                from: cap.at(1).unwrap().to_owned(),
                content: cap.at(2).unwrap().to_owned(),
                time: time(cap.at(0).unwrap())
            }))
        } else if let Some(cap) = OTHER_LINES.captures(line) {
            if cap.at(4) == Some("has") && cap.at(5) == Some("kicked") {
                return Some(Ok(Event::Kick {
                    kicked_nick: cap.at(6).unwrap().to_owned(),
                    kicking_nick: cap.at(3).unwrap().to_owned(),
                    kick_message: cap.at(4).unwrap().to_owned(),
                    time: time(cap.at(0).unwrap())
                }))
            } else if cap.at(3) == Some("has") && cap.at(5) == Some("changed") && cap.at(6) == Some("topic") {
                return Some(Ok(Event::Topic {
                    new_topic: cap.at(5).unwrap().to_owned(),
                    time: time(cap.at(0).unwrap())
                }))
            } else if cap.at(3) == Some("Mode") {
                return Some(Ok(Event::Mode {
                    time: time(cap.at(0).unwrap())
                }))
            } else if cap.at(5) == Some("has") && cap.at(6) == Some("joined") {
                return Some(Ok(Event::Join {
                    nick: cap.at(3).unwrap().to_owned(),
                    mask: String::new(),
                    time: time(cap.at(0).unwrap())
                }))
            } else if cap.at(5) == Some("now") && cap.at(6) == Some("known") {

            }
        }
        Some(Err(::IlcError::Parse(format!("Line `{}` didn't match any rules.", line))))
    }
}

impl<R> Decode<R, Iter<R>> for Weechat3 where R: BufRead {
    fn decode(&mut self, input: R) -> Iter<R> {/*
                for line in input.lines() {
            let line = &*try!(line);
            } else {
                handler.err(&format!("Malformatted line: {}", line));
            }
        }*/
        Iter {
            input: input,
            buffer: String::new()
        }
    }
}
