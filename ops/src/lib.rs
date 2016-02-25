extern crate blist;
extern crate ilc_base;

mod ageset;
pub mod freq;

pub mod parse {
    use ilc_base::{self, Context, Decode};
    use std::io::BufRead;
    pub fn parse(ctx: &Context, input: &mut BufRead, decoder: &mut Decode) -> ilc_base::Result<()> {
        for e in decoder.decode(&ctx, input) {
            try!(e);
        }
        Ok(())
    }
}

pub mod convert {
    use ilc_base::{self, Context, Decode, Encode};
    use std::io::{BufRead, Write};

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

pub mod seen {
    use ilc_base::{self, Context, Decode, Encode, Event};
    use std::io::{BufRead, Write};

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

pub mod sort {
    use ilc_base::{self, Context, Decode, Encode, Event};
    use std::io::{BufRead, Write};

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

pub mod dedup {
    use std::io::{BufRead, Write};
    use std::hash::{Hash, Hasher};
    use ageset::AgeSet;
    use ilc_base::{self, Context, Decode, Encode, Event};

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct NoTimeHash<'a>(pub Event<'a>);

    impl<'a> Hash for NoTimeHash<'a> {
        fn hash<H>(&self, state: &mut H)
            where H: Hasher
        {
            self.0.ty.hash(state);
            self.0.channel.hash(state);
        }
    }

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
