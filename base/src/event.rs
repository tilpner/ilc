//! Common structures to represent the actual log data in memory.
//! These will be used by all formats for encoding and decoding.

use std::borrow::Cow;
use std::cmp::Ordering;

use chrono::naive::time::NaiveTime;
use chrono::offset::fixed::FixedOffset;
use chrono::offset::local::Local;
use chrono::offset::TimeZone;

/// A whole log, in memory. This structure does not specify its
/// use. It may represent a private query, or the log of a channel.
pub struct Log<'a> {
    pub entries: Vec<Event<'a>>,
}

/// Different log formats carry different amounts of information. Some might
/// hold enough information to calculate precise timestamps, others might
/// only suffice for the time of day.
#[derive(Clone, Debug, PartialEq, Eq, Ord, Hash, RustcEncodable, RustcDecodable)]
pub enum Time {
    Unknown,
    Hms(u8, u8, u8),
    Timestamp(i64),
}

impl Time {
    pub fn from_format(tz: &FixedOffset, s: &str, f: &str) -> Time {
        tz.datetime_from_str(s, f)
          .map(|d| d.timestamp())
          .map(Time::Timestamp)
          .unwrap_or(Time::Unknown)
    }

    pub fn with_format(&self, tz: &FixedOffset, f: &str) -> String {
        match self {
            &Time::Unknown => panic!("Time data for this event is not present"),
            &Time::Hms(h, m, s) => {
                format!("{}",
                        NaiveTime::from_hms(h as u32, m as u32, s as u32).format(f))
            }
            &Time::Timestamp(t) => format!("{}", tz.timestamp(t, 0).format(f)),
        }
    }

    pub fn as_timestamp(&self) -> i64 {
        use self::Time::*;
        match self {
            &Unknown => 0,
            &Hms(h, m, s) => {
                Local::today()
                    .and_hms(h as u32, m as u32, s as u32)
                    .timestamp()
            }
            &Timestamp(i) => i,
        }
    }

    pub fn to_timestamp(&self) -> Time {
        Time::Timestamp(self.as_timestamp())
    }
}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Time) -> Option<Ordering> {
        use self::Time::*;
        match (self, other) {
            (&Unknown, _) | (_, &Unknown) => None,
            (&Hms(a_h, a_m, a_s), &Hms(b_h, b_m, b_s)) => {
                if (a_h >= b_h && a_m >= b_m && a_s > b_s) ||
                   (a_h >= b_h && a_m > b_m && a_s >= b_s) ||
                   (a_h > b_h && a_m >= b_m && a_s >= b_s) {
                    Some(Ordering::Greater)
                } else {
                    Some(Ordering::Less)
                }
            }
            (&Timestamp(a), &Timestamp(b)) => Some(a.cmp(&b)),
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, RustcEncodable, RustcDecodable)]
pub struct Event<'a> {
    pub ty: Type<'a>,
    pub time: Time,
    pub channel: Option<Cow<'a, str>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, RustcEncodable, RustcDecodable)]
pub struct User<'a> {
    nicks: Cow<'a, str>,
}

/// All representable events, such as messages, quits, joins
/// and topic changes.
#[derive(Clone, Debug, Hash, PartialEq, Eq, RustcEncodable, RustcDecodable)]
pub enum Type<'a> {
    Connect,
    Disconnect,
    Msg {
        from: Cow<'a, str>,
        content: Cow<'a, str>,
    },
    Action {
        from: Cow<'a, str>,
        content: Cow<'a, str>,
    },
    Join {
        nick: Cow<'a, str>,
        mask: Option<Cow<'a, str>>,
    },
    Part {
        nick: Cow<'a, str>,
        mask: Option<Cow<'a, str>>,
        reason: Option<Cow<'a, str>>,
    },
    Quit {
        nick: Cow<'a, str>,
        mask: Option<Cow<'a, str>>,
        reason: Option<Cow<'a, str>>,
    },
    Nick {
        old_nick: Cow<'a, str>,
        new_nick: Cow<'a, str>,
    },
    Notice {
        from: Cow<'a, str>,
        content: Cow<'a, str>,
    },
    Kick {
        kicked_nick: Cow<'a, str>,
        kicking_nick: Option<Cow<'a, str>>,
        kick_message: Option<Cow<'a, str>>,
    },
    Topic {
        topic: Cow<'a, str>,
    },
    TopicChange {
        nick: Option<Cow<'a, str>>,
        new_topic: Cow<'a, str>,
    },
    Mode {
        nick: Option<Cow<'a, str>>,
        mode: Cow<'a, str>,
        masks: Cow<'a, str>,
    },
}

impl<'a> Type<'a> {
    pub fn actor(&self) -> Option<&str> {
        use self::Type::*;
        match self {
            &Msg { ref from, .. } => Some(from),
            &Action { ref from, .. } => Some(from),
            &Join { ref nick, .. } => Some(nick),
            &Part { ref nick, .. } => Some(nick),
            &Quit { ref nick, .. } => Some(nick),
            &Nick { ref old_nick, .. } => Some(old_nick),
            &Notice { ref from, .. } => Some(from),
            &Kick { ref kicking_nick, .. } => kicking_nick.as_ref().map(|s| &*s as &str),
            &TopicChange { ref nick, .. } => nick.as_ref().map(|s| &*s as &str),
            &Mode { ref nick, .. } => nick.as_ref().map(|s| &*s as &str),
            _ => None,
        }
    }

    pub fn involves(&self, needle: &str) -> bool {
        use self::Type::*;
        match self {
            &Msg { ref from, ref content, .. } => from == needle || content.contains(needle),
            &Action { ref from, ref content, .. } => from == needle || content.contains(needle),
            &Join { ref nick, .. } => nick == needle,
            &Part { ref nick, ref reason, .. } => {
                nick == needle || reason.as_ref().map_or(false, |r| r.contains(needle))
            }
            &Quit { ref nick, ref reason, .. } => {
                nick == needle || reason.as_ref().map_or(false, |r| r.contains(needle))
            }
            &Nick { ref old_nick, ref new_nick, .. } => old_nick == needle || new_nick == needle,
            &Notice { ref from, ref content, .. } => from == needle || content.contains(needle),
            &Kick { ref kicked_nick, ref kicking_nick, ref kick_message, .. } => {
                *kicked_nick == Cow::Borrowed(needle) ||
                kicking_nick.as_ref().map_or(false, |k| k.as_ref() == Cow::Borrowed(needle)) ||
                kick_message.as_ref().map_or(false, |k| k.as_ref() == Cow::Borrowed(needle))
            }
            &TopicChange { ref nick, ref new_topic, .. } => {
                nick.as_ref().map_or(false, |k| k.as_ref() == needle) || new_topic.contains(needle)
            }
            &Mode { ref nick, .. } => {
                nick.as_ref().map_or(false, |k| k.as_ref() == Cow::Borrowed(needle))
            }
            _ => false,
        }
    }

    pub fn type_desc(&self) -> &'static str {
        use self::Type::*;
        match self {
            &Msg { .. } => "message",
            &Action { .. } => "action",
            &Join { .. } => "join",
            &Part { .. } => "part",
            &Quit { .. } => "quit",
            &Nick { .. } => "nick",
            &Notice { .. } => "notice",
            &Topic { .. } => "topic",
            &TopicChange { .. } => "topic_change",
            &Kick { .. } => "kick",
            &Mode { .. } => "mode",
            &Connect => "connect",
            &Disconnect => "disconnect",
        }
    }

    pub fn text(&self) -> Option<&str> {
        use self::Type::*;
        match self {
            &Msg { ref content, .. } => Some(content),
            &Action { ref content, .. } => Some(content),
            &Part { ref reason, .. } => reason.as_ref().map(|s| s as &str),
            &Quit { ref reason, .. } => reason.as_ref().map(|s| s as &str),
            &Notice { ref content, .. } => Some(content),
            &Kick { ref kick_message, .. } => kick_message.as_ref().map(|s| s as &str),
            &Topic { ref topic, .. } => Some(topic),
            &TopicChange { ref new_topic, .. } => Some(new_topic),
            _ => None,
        }
    }
}
