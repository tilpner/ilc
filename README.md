ilc
=========

**So... what is this thing?**

ilc is *supposed to be* a library to work with common IRC log formats, as well as a collection
of commonly needed utilities for IRC logs.

**Supposed to be? What can it do for me now?**

The library can convert between most of the EnergyMech and Weechat3 log formats.
The tools can pretty-print them, and count the lines/words that people said in them.
They're not really configurable yet, so you'd have to recompile it for that... <sup><sup>yesiknowitsucks</sup></sup>

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
