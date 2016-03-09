extern crate ilc_cli;

use ilc_cli::Cli;

static MASTER: &'static str = include_str!("../.git/refs/heads/master");

fn main() {
    ilc_cli::main(Cli {
        version: env!("CARGO_PKG_VERSION").into(),
        master_hash: MASTER.trim_right().into(),
    });
}
