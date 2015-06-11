
use chrono::naive::date::NaiveDate;
use chrono::offset::fixed::FixedOffset;

pub struct Context {
    pub timezone: FixedOffset,
    pub override_date: NaiveDate
}
