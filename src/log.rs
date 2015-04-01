//! Common structures to represent the actual log data in memory.
//! These will be used by all formats for encoding and decoding.

/// A whole log, in memory. This structure does not specify its
/// use. It may represent a private query, or the log of a channel.
pub struct Log {
    pub entries: Vec<Event>
}

/// All representable events, such as messages, quits, joins
/// and topic changes.
#[derive(Debug)]
pub enum Event {
    Connect {
        time: i64
    },
    Disconnect {
        time: i64
    },
    Msg {
        from: String,
        content: String,
        time: i64
    },
    Action {
        from: String,
        content: String,
        time: i64
    },
    Join {
        nick: String,
        channel: String,
        mask: String,
        time: i64
    },
    Part {
        nick: String,
        channel: String,
        mask: String,
        reason: String,
        time: i64
    },
    Quit {
        nick: String,
        mask: String,
        reason: String,
        time: i64
    },
    Nick {
        old: String,
        new: String,
        time: i64
    },
    Notice {
        nick: String,
        content: String,
        time: i64
    },
    Kick {
        kicked_nick: String,
        kicking_nick: String,
        kick_message: String,
        time: i64
    },
    Topic {
        topic: String,
        time: i64
    },
    TopicChange {
        new_topic: String,
        time: i64
    },
    Mode {
        time: i64
    }
}
