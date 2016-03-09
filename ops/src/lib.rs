#[macro_use]
extern crate log;
extern crate blist;
extern crate bit_set;
extern crate serde;
extern crate ilc_base;

mod ageset;
pub mod stats;

/// No-op log parsing
pub mod parse {
    use ilc_base::{self, Context, Decode};
    use std::io::BufRead;

    /// Simply parse the input, without further validation or conversion. No information is stored.
    /// This will return `Err` if the decoder yields `Err`.
    pub fn parse(ctx: &Context, input: &mut BufRead, decoder: &mut Decode) -> ilc_base::Result<()> {
        for e in decoder.decode(&ctx, input) {
            match e {
                Ok(e) => debug!("{:?}", e),
                Err(e) => error!("{:?}", e),
            }
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

/// "Efficient" n-way merging
pub mod merge {
    use std::io::{BufRead, Write};
    use bit_set::BitSet;
    use ilc_base::{self, Context, Decode, Encode, Event};

    /// Merge several individually sorted logs, *without* reading everything
    /// into memory.
    ///
    /// The `input` and `decode` parameter will be zipped, so make sure they match up.
    ///
    /// Output will be inconsistent if every input isn't sorted by itself.
    /// Logs with incomplete dates are free to do to weird stuff.
    pub fn merge<'a>(ctx: &Context,
                     input: Vec<&'a mut BufRead>,
                     decode: &mut Decode,
                     output: &mut Write,
                     encode: &Encode)
                     -> ilc_base::Result<()> {
        let mut events = input.into_iter()
                              .map(|i| decode.decode(&ctx, i).peekable())
                              .collect::<Vec<_>>();
        let mut empty = BitSet::with_capacity(events.len());

        loop {
            if events.is_empty() {
                return Ok(());
            }

            let earliest_idx = {
                // Lifetimes can be a lot easier if you don't nest closures.
                // Uglier too, yes. But hey, at least it compiles.
                // Currently earliest event
                let mut current = None;
                // Index of stream that has "current" as head
                let mut stream_idx = None;

                for (idx, stream) in events.iter_mut().enumerate() {
                    let peek = stream.peek();
                    if let Some(ev) = peek {
                        // Ignore errors in stream
                        if let &Ok(ref ev) = ev {
                            if current.map(|c: &Event| ev.time < c.time).unwrap_or(true) {
                                current = Some(ev);
                                stream_idx = Some(idx);
                            }
                        }
                    } else {
                        empty.insert(idx);
                    }
                }
                stream_idx
            };

            // Safe because of matching against Some(&Ok(ref ev)) earlier
            let earliest = earliest_idx.map(|idx| events[idx].next().unwrap().unwrap());

            if let Some(event) = earliest {
                try!(encode.encode(&ctx, output, &event));
            }

            // Keep non-empty streams
            for (offset, idx) in empty.iter().enumerate() {
                // `remove` returns an iterator. It's empty, but Rust doesn't know that,
                // so suppress the warning like this.
                let _ = events.remove(offset + idx);
            }
            empty.clear();
        }
    }
}
