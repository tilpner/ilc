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

//! Common structures to represent the actual log data in memory.
//! These will be used by all formats for encoding and decoding.

use std::borrow::Cow;

use chrono::offset::fixed::FixedOffset;
use chrono::offset::TimeZone;

/// A whole log, in memory. This structure does not specify its
/// use. It may represent a private query, or the log of a channel.
pub struct Log<'a> {
    pub entries: Vec<Event<'a>>
}

/// Different log formats carry different amounts of information. Some might
/// hold enough information to calculate precise timestamps, others might
/// only suffice for the time of day.
#[derive(Clone, Debug, PartialEq, Hash, RustcEncodable, RustcDecodable)]
pub enum Time {
    Unknown,
    Hms(u8, u8, u8),
    Timestamp(i64)
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
            &Time::Hms(_h, _m, _s) => unimplemented!(),
            &Time::Timestamp(t) => format!("{}", tz.timestamp(t, 0).format(f))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Hash, RustcEncodable, RustcDecodable)]
pub struct Event<'a> {
    pub ty: Type<'a>,
    pub time: Time,
    pub channel: Option<Cow<'a, str>>
}

/// All representable events, such as messages, quits, joins
/// and topic changes.
#[derive(Clone, Debug, Hash, PartialEq, RustcEncodable, RustcDecodable)]
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
        new_topic: Cow<'a, str>,
    },
    Mode
}
