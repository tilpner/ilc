extern crate blist;
extern crate ilc_base;

mod ageset;
pub mod freq;

/// No-op log parsing
pub mod parse {
    use ilc_base::{self, Context, Decode};
    use std::io::BufRead;

    /// Simply parse the input, without further validation or conversion. No information is stored.
    /// This will return `Err` if the decoder yields `Err`.
    pub fn parse(ctx: &Context, input: &mut BufRead, decoder: &mut Decode) -> ilc_base::Result<()> {
        for e in decoder.decode(&ctx, input) {
            try!(e);
        }
        Ok(())
    }
}

/// Log format conversion
pub mod convert {
    use ilc_base::{self, Context, Decode, Encode};
    use std::io::{BufRead, Write};

    /// Convert from one format to another, not necessarily different, format. In combination with a
    /// timezone offset, this can be used to correct the timestamps.
    /// Will return `Err` and abort conversion if the decoder yields `Err` or re-encoding fails.
    pub fn convert(ctx: &Context,
                   input: &mut BufRead,
                   decoder: &mut Decode,
                   output: &mut Write,
                   encoder: &Encode)
                   -> ilc_base::Result<()> {
        for e in decoder.decode(&ctx, input) {
            try!(encoder.encode(&ctx, output, &try!(e)));
        }
        Ok(())
    }
}

/// Last-seen of nicks
pub mod seen {
    use ilc_base::{self, Context, Decode, Encode, Event};
    use std::io::{BufRead, Write};

    /// Return the last message of a given nickname, searching from the beginning of the logs.
    /// Will return `Err` if the decoder yields `Err`. This relies on absolute timestamps, and
    /// behaviour without full dates is undefined.
    pub fn seen(nick: &str,
                ctx: &Context,
                input: &mut BufRead,
                decoder: &mut Decode,
                output: &mut Write,
                encoder: &Encode)
                -> ilc_base::Result<()> {
        let mut last: Option<Event> = None;
        for e in decoder.decode(&ctx, input) {
            let m: Event = try!(e);
            if m.ty.involves(nick) &&
               last.as_ref().map_or(true,
                                    |last| m.time.as_timestamp() > last.time.as_timestamp()) {
                last = Some(m)
            }
        }
        if let Some(ref m) = last {
            try!(encoder.encode(&ctx, output, m));
        }
        Ok(())
    }
}

/// Internal (as opposed to external, not to be confused with private) log sorting
pub mod sort {
    use ilc_base::{self, Context, Decode, Encode, Event};
    use std::io::{BufRead, Write};

    /// **Memory-intensive**
    /// Sort the input, discarding faulty events. This will
    /// read *all events* into memory, then sort them by time and write them back.
    /// Behaviour is undefined if events lack full date information.
    ///
    /// *This should be an external merge-sort, but is a placeholder until implementation*
    pub fn sort(ctx: &Context,
                input: &mut BufRead,
                decoder: &mut Decode,
                output: &mut Write,
                encoder: &Encode)
                -> ilc_base::Result<()> {
        let mut events: Vec<Event> = decoder.decode(&ctx, input)
                                            .flat_map(Result::ok)
                                            .collect();

        events.sort_by(|a, b| a.time.cmp(&b.time));
        for e in events {
            try!(encoder.encode(&ctx, output, &e));
        }
        Ok(())
    }
}

/// Event deduplication
pub mod dedup {
    use std::io::{BufRead, Write};
    use std::hash::{Hash, Hasher};
    use ageset::AgeSet;
    use ilc_base::{self, Context, Decode, Encode, Event};

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct NoTimeHash<'a>(pub Event<'a>);

    impl<'a> Hash for NoTimeHash<'a> {
        fn hash<H>(&self, state: &mut H)
            where H: Hasher
        {
            self.0.ty.hash(state);
            self.0.channel.hash(state);
        }
    }

    /// Deduplicate subsequent identical elements, e.g. after a sorting
    /// operation. This will **not** read all events into memory, and only
    /// operate on a short window of events. Therefore, it'll only work correctly
    /// on sorted or very short logs.
    pub fn dedup(ctx: &Context,
                 input: &mut BufRead,
                 decoder: &mut Decode,
                 output: &mut Write,
                 encoder: &Encode)
                 -> ilc_base::Result<()> {
        let mut backlog = AgeSet::new();

        for e in decoder.decode(&ctx, input) {
            if let Ok(e) = e {
                let newest_event = e.clone();
                backlog.prune(move |a: &NoTimeHash| {
                    let age = newest_event.time.as_timestamp() - a.0.time.as_timestamp();
                    age > 5000
                });
                // write `e` if it's a new event
                let n = NoTimeHash(e);
                if !backlog.contains(&n) {
                    try!(encoder.encode(&ctx, output, &n.0));
                    backlog.push(n);
                }
            }
        }
        Ok(())
    }
}
