use std::iter;
use std::io::BufRead;

use event::Event;
use context::Context;

#[derive(Copy, Clone)]
pub struct Dummy;

impl ::Decode for Dummy {
    fn decode<'a>(&'a self,
                  _context: &'a Context,
                  _input: &'a mut BufRead)
                  -> Box<Iterator<Item = ::Result<Event<'a>>> + 'a> {
        Box::new(iter::empty())
    }
}
