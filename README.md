ilc
=========
[![Travis](https://img.shields.io/travis/tilpner/ilc.svg?style=flat-square)](https://travis-ci.org/tilpner/ilc)
[![Appveyor](https://img.shields.io/appveyor/ci/tilpner/ilc.svg?style=flat-square)](https://ci.appveyor.com/project/tilpner/ilc)
[![Crates.io version](https://img.shields.io/crates/v/ilc.svg?style=flat-square)](https://crates.io/crates/ilc)
[![Crates.io license](https://img.shields.io/crates/l/ilc.svg?style=flat-square)](https://crates.io/crates/ilc)

#### What is this thing?

ilc is a library to work with common IRC log formats, as well as a collection
of commonly needed utilities for IRC logs.

The library can convert between most of the EnergyMech (ZNC) and Weechat log formats, as well as binary and msgpack representations of them.
The tools can merge, convert, and pretty-print them, or count the lines/words that people said in them.

#### Downloads

There are other builds, including for/with musl, mingw, or ARM. Please [check out the latest release](https://github.com/tilpner/ilc/releases/latest) to see all archives.

| Platform | 64 bit | 32 bit |
| -------- | ------ | ------ |
| Linux    | [Download](https://github.com/tilpner/ilc/releases/download/v0.2.3.1/ilc-x86_64-unknown-linux-gnu.tar.gz) |[Download](https://github.com/tilpner/ilc/releases/download/v0.2.3.1/ilc-i686-unknown-linux-gnu.tar.gz) |
| OS X     | [Download](https://github.com/tilpner/ilc/releases/download/v0.2.3.1/ilc-x86_64-apple-darwin.tar.gz) |[Download](https://github.com/tilpner/ilc/releases/download/v0.2.3.1/ilc-i686-apple-darwin.tar.gz) |
| Windows  | [Download](https://github.com/tilpner/ilc/releases/download/v0.2.3.1/ilc-x86_64-pc-windows-msvc.zip) |[Download](https://github.com/tilpner/ilc/releases/download/v0.2.3.1/ilc-i686-pc-windows-msvc.zip) |

*Note that these links might be broken in the one hour it takes between release and uploading builds, sorry for that.*

#### How can I compile this myself?

Because I'm using experimental features (slice_patterns), you have to use a Rust nightly installation either way.

###### From crates.io

    cargo install ilc

###### From git

`67ee599c56ba9e58cfe190036b7dcc656b20bfdd` is known to compile with

> rustc 1.8.0-nightly (d63b8e539 2016-01-23)
> cargo 0.8.0-nightly (8edc460 2016-01-21)

To compile:

    cargo build --release

#### Usage
```
Usage:
  ilc parse [options] [-i FILE...]
  ilc convert [options] [-i FILE...]
  ilc freq [options] [-i FILE...]
  ilc seen <nick> [options] [-i FILE...]
  ilc sort [options] [-i FILE...]
  ilc dedup [options] [-i FILE...]
  ilc (-h | --help | -v | --version)

Options:
  -h --help         Show this screen.
  -v --version      Show the version (duh).
  --date DATE       Override the date for this log. ISO 8601, YYYY-MM-DD.
  --tz SECONDS      UTC offset in the direction of the western hemisphere.
  --channel CH      Set a channel for the given log.
  --inf INF         Set the input format.
  --outf OUTF       Set the output format.
  --in -i IN        Give an input file, instead of stdin.
  --out -o OUT      Give an output file, instead of stdout.
  --infer-date    Try to use the filename as date for the log.
```

#### Are you stupid? Why Rust?

Uhh, actually... that may have been a suboptimal choice. Nobody cares about performance here
anyways. But it was what I started with, and I didn't feel like rewriting it.

#### I found a bug!

Great! Or... well, not so great. Please file it [here](https://github.com/tilpner/ilc/issues/new)!

#### It doesn't support my format.

Support for irssi is planned, but beyond that you'll have to file an issue or implement it yourself.
PRs are welcome, but I can't promise to merge all of them without revision. :)
