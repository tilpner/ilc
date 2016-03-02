
use chrono::naive::date::NaiveDate;
use chrono::offset::fixed::FixedOffset;

pub struct Context {
    pub timezone: FixedOffset,
    pub override_date: Option<NaiveDate>,
    pub channel: Option<String>,
}

impl Default for Context {
    fn default() -> Context {
        Context {
            timezone: FixedOffset::west(0),
            override_date: None,
            channel: None,
        }
    }
}
