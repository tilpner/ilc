use chrono::naive::date::NaiveDate;
use chrono::offset::fixed::FixedOffset;

pub struct Context {
    pub timezone_in: FixedOffset,
    pub timezone_out: FixedOffset,
    pub override_date: Option<NaiveDate>,
    pub channel: Option<String>,
}

impl Default for Context {
    fn default() -> Context {
        Context {
            timezone_in: FixedOffset::west(0),
            timezone_out: FixedOffset::west(0),
            override_date: None,
            channel: None,
        }
    }
}
