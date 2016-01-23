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

**Fine, I'll use it. What, no binaries?**

Mhm, I haven't yet figured out what legal stuff to include in eventual packages, and
nobody would use them anyways...

**Ugh, how do I compile it then?**

Because I'm using experimental features, you have to use a Rust nightly installation.

    cb6a4e2d24616c680a3793b0f92ec0f2f6df00db

is known to compile with

    rustc 1.2.0-nightly (fbb13543f 2015-06-11)
    cargo 0.3.0-nightly (2ac8a86 2015-06-10) (built 2015-06-10)

To compile:

    cargo build --release

**Usage**
```
Usage:
  ilc parse [options] [-i FILE...]
  ilc convert [options] [-i FILE...]
  ilc freq [options] [-i FILE...]
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
```
