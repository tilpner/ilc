ilc
=========
[![Build Status](https://img.shields.io/travis/tilpner/ilc.svg?style=flat-square)](https://travis-ci.org/tilpner/ilc)
[![Crates.io version](https://img.shields.io/crates/v/ilc.svg?style=flat-square)](https://crates.io/crates/ilc)
[![Crates.io license](https://img.shields.io/crates/l/ilc.svg?style=flat-square)](https://crates.io/crates/ilc)

**So... what is this thing?**

ilc is a library to work with common IRC log formats, as well as a collection
of commonly needed utilities for IRC logs.

The library can convert between most of the EnergyMech and Weechat3 log formats, as well as binary and msgpack representations of them.
The tools can pretty-print them, and count the lines/words that people said in them.

**Are you stupid? Why Rust?**

Uhh, actually... that may have been a suboptimal choice. Nobody cares about performance here
anyways. But it was what I started with, and I didn't feel like rewriting it.

**Fine, I'll use it. Do I really have to compile it?**

Probably. I sporadically [release a binary](https://github.com/tilpner/ilc/releases), but those are for x86-64 Linux. If you want something else, or more recent, you'll have to compile yourself.

**Okay, how do I compile it then?**

Because I'm using experimental features (slice_patterns), you have to use a Rust nightly installation.

`67ee599c56ba9e58cfe190036b7dcc656b20bfdd` is known to compile with

    rustc 1.8.0-nightly (d63b8e539 2016-01-23)
    cargo 0.8.0-nightly (8edc460 2016-01-21)

To compile:

    cargo build --release

**Usage**
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
