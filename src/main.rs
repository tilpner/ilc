extern crate ilc_cli;

use ilc_cli::Cli;

static MASTER_HASH: &'static str = include_str!("../.git/refs/heads/master");

fn main() {
    ilc_cli::main(Cli {
        version: env!("CARGO_PKG_VERSION").into(),
        master_hash: if MASTER_HASH.is_empty() {
            None
        } else {
            Some(MASTER_HASH.trim_right().to_owned())
        },
    });
}
