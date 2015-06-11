// Copyright 2015 Till Höppner
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

extern crate ilc;
extern crate chrono;

use std::io;
use std::collections::hash_map::*;

use chrono::offset::fixed::FixedOffset;
use chrono::naive::date::NaiveDate;

use ilc::event::{ Event, Type };
use ilc::context::Context;
use ilc::format::{ self, Decode };

struct Person {
    lines: u32,
    words: u32
}

fn words(s: &str) -> u32 {
    s.split_whitespace().filter(|s| !s.is_empty()).count() as u32
}

fn strip_nick(s: &str) -> &str {
    if s.is_empty() { return s }
    match s.as_bytes()[0] {
        b'~' | b'&' | b'@' | b'%' | b'+' => &s[1..],
        _ => s
    }
}

fn main() {
    let stdin = io::stdin();

    let mut stats: HashMap<String, Person> = HashMap::new();
    let context = Context {
        timezone: FixedOffset::west(0),
        override_date: NaiveDate::from_ymd(2015, 6, 10)
    };

    let mut parser = format::weechat3::Weechat3;
    for e in parser.decode(&context, stdin.lock()) {
        let m = match e {
            Ok(m) => m,
            Err(err) => panic!(err)
        };

        match m {
            Event { ty: Type::Msg { ref from, ref content, .. }, .. } => {
                let nick = strip_nick(from);
                if stats.contains_key(nick) {
                    let p: &mut Person = stats.get_mut(nick).unwrap();
                    p.lines += 1;
                    p.words += words(content);
                } else {
                    stats.insert(nick.to_owned(), Person {
                        lines: 1,
                        words: words(content)
                    });
                }
            },
            _ => ()
        }
    }

    let mut stats: Vec<(String, Person)> = stats.into_iter().collect();
    stats.sort_by(|&(_, ref a), &(_, ref b)| b.words.cmp(&a.words));

    for &(ref name, ref stat) in stats.iter().take(10) {
        println!("{}:\n\tLines: {}\n\tWords: {}", name, stat.lines, stat.words)
    }
}
