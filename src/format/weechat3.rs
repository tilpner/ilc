use std::io::{ self, BufRead, Write };
use std::borrow::ToOwned;
use std::iter::{ Iterator, AdditiveIterator };

use log::Event;
use format::{ Encode, Decode };

use l::LogLevel::Info;

use chrono::*;

pub struct Weechat3;

static TIME_DATE_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

pub struct Iter<R> where R: BufRead {
    input: R,
    buffer: String
}

impl<R> Iterator for Iter<R> where R: BufRead {
    type Item = ::Result<Event>;
    fn next(&mut self) -> Option<::Result<Event>> {
        fn timestamp(date: &str, time: &str) -> i64 {
            UTC.datetime_from_str(&format!("{} {}", date, time), TIME_DATE_FORMAT).unwrap().timestamp()
        }
        fn join(s: &[&str], splits: &[char]) -> String {
            let len = s.iter().map(|s| s.len()).sum();
            let mut out = s.iter().zip(splits.iter()).fold(String::with_capacity(len),
               |mut s, (b, &split)| { s.push_str(b); s.push(split); s });
            out.pop(); out
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

            let mut split_tokens: Vec<char> = Vec::new();
            let tokens = self.buffer.split( |c: char| {
                if c.is_whitespace() { split_tokens.push(c); true } else { false }
            }).collect::<Vec<_>>();
            if log_enabled!(Info) {
                info!("Original:  `{}`", self.buffer);
                info!("Parsing:   {:?}", tokens);
            }
            match tokens[..tokens.len() - 1].as_ref() {
                [date, time, "-->", nick, host, "has", "joined", channel, _..] => return Some(Ok(Event::Join {
                    nick: nick.to_owned(), channel: channel.to_owned(), mask: mask(host),
                    time: timestamp(date, time)
                })),
                [date, time, "<--", nick, host, "has", "left", channel, reason..] => return Some(Ok(Event::Part {
                    nick: nick.to_owned(), channel: channel.to_owned(), mask: mask(host),
                    reason: mask(&join(reason, &split_tokens[8..])), time: timestamp(date, time)
                })),
                [date, time, "--", notice, content..]
                    if notice.starts_with("Notice(")
                    => return Some(Ok(Event::Notice {
                    nick: notice["Notice(".len()..notice.len() - 2].to_owned(),
                    content: join(content, &split_tokens[4..]),
                    time: timestamp(date, time)
                })),
                [date, time, "--", "irc:", "disconnected", "from", "server", _..] => return Some(Ok(Event::Disconnect {
                    time: timestamp(date, time)
                })),
                [date, time, sp, "*", nick, msg..]
                    if sp.is_empty()
                    => return Some(Ok(Event::Action {
                    from: nick.to_owned(), content: join(msg, &split_tokens[5..]),
                    time: timestamp(date, time)
                })),
                [date, time, nick, msg..] => return Some(Ok(Event::Msg {
                    from: nick.to_owned(),
                    content: join(msg, &split_tokens[3..]),
                    time: timestamp(date, time)
                })),
                _ => ()
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
                try!(writeln!(&mut output, "{}\t{}\t{}", date(*time), from, content))
            },
            &Event::Action { ref from, ref content, ref time } => {
                try!(writeln!(&mut output, "{}\t *\t{} {}", date(*time), from, content))
            },
            &Event::Join { ref nick, ref mask, ref channel, ref time } => {
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
            },
            _ => ()
        }
        Ok(())
    }
}
