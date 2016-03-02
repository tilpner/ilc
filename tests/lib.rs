extern crate ilc;

use std::default::Default;
use std::io::Cursor;

use ilc::*;

mod files;

#[test]
fn identity() {
    let original = files::read("2016-02-26.log");
    let mut output = Vec::new();

    convert(&Context::default(),
            &mut (&original as &[u8]),
            &mut Energymech,
            &mut output,
            &Energymech)
        .expect("Conversion failed");

    files::write("identity.out", &output);

    // don't assert_eq!, as the failed ouput doesn't help anyone
    assert!(&original == &output);
}

/* #[test]
 * fn merge() {
 * let part1 = Cursor::new(files::read("2016-02-26.log.1"));
 * let part2 = Cursor::new(files::read("2016-02-26.log.2"));
 *
 * let mut output = Vec::new();
 *
 * merge(&Context::default(),
 * vec![&mut part1, &mut part2],
 * &mut Energymech,
 * &mut output,
 * &Energymech)
 * .expect("Merge failed");
 *
 * files::write("merged.out", &output);
 *
 * let original = files::read("2016-02-26.log");
 * assert!(&original == &output);
 * } */
