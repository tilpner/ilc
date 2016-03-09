use clap::ArgMatches;

use chrono::Local;

use serde_json;
use serde::ser::{MapVisitor, Serialize, Serializer};

use ilc_base;
use ilc_ops::stats::Stats;
use Environment;
use Cli;
use error;

struct StatFormat {
    version: String,
    master_hash: Option<String>,
    time: String,
    stats: Stats,
}

impl Serialize for StatFormat {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: Serializer
    {
        struct Visitor<'a>(&'a StatFormat);
        impl<'a> MapVisitor for Visitor<'a> {
            fn visit<S>(&mut self, s: &mut S) -> Result<Option<()>, S::Error>
                where S: Serializer
            {
                try!(s.serialize_struct_elt("version", &self.0.version));
                if let &Some(ref h) = &self.0.master_hash {
                    try!(s.serialize_struct_elt("master_hash", h));
                }
                try!(s.serialize_struct_elt("time", &self.0.time));
                try!(s.serialize_struct_elt("stats", &self.0.stats));
                Ok(None)
            }

            fn len(&self) -> Option<usize> {
                Some(4)
            }
        }
        s.serialize_struct("StatFormat", Visitor(self))
    }
}


pub fn output_as_json(args: &ArgMatches, cli: &Cli, stats: Stats) -> ilc_base::Result<()> {
    let e = Environment(args);
    // let count = value_t!(args, "count", usize).unwrap_or(usize::MAX);
    // let mut stats: Vec<(String, Person)> = stats.into_iter().collect();
    // stats.sort_by(|&(_, ref a), &(_, ref b)| b.words.cmp(&a.words));

    // for &(ref name, ref stat) in stats.iter().take(count) {

    let format = StatFormat {
        version: cli.version.clone(),
        master_hash: cli.master_hash.clone(),
        time: Local::now().to_rfc2822(),
        stats: stats,
    };

    serde_json::to_writer_pretty(&mut e.output(), &format).unwrap_or_else(|e| error(Box::new(e)));
    /* write!(&mut *e.output(),
     * "{}:\n\tTotal lines: {}\n\tLines without alphabetic characters: {}\n\tTotal \
     * words: {}\n\tWords per line: {}\n",
     * name,
     * stat.lines,
     * stat.lines - stat.alpha_lines,
     * stat.words,
     * stat.words as f32 / stat.lines as f32)
     * .unwrap_or_else(|e| error(Box::new(e))); */
    // }
    Ok(())
}
