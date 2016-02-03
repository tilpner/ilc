use clap::ArgMatches;

use std::collections::HashMap;

use ilc::event::{ Event, Type };

use super::*;

struct Person {
    lines: u32,
    alpha_lines: u32,
    words: u32
}

fn words_alpha(s: &str) -> (u32, bool) {
    let mut alpha = false;
    let mut words = 0;
    for w in s.split_whitespace() {
        if !w.is_empty() {
            words += 1;
            if w.chars().any(char::is_alphabetic) { alpha = true }
        }
    }
    (words, alpha)
}

fn strip_nick_prefix(s: &str) -> &str {
    if s.is_empty() { return s }
    match s.as_bytes()[0] {
        b'~' | b'&' | b'@' | b'%' | b'+' => &s[1..],
        _ => s
    }
}

pub  fn freq(args: &ArgMatches) {
    let env = Environment(args);
    let (context, mut decoder, mut input, mut output) = (env.context(), env.decoder(), env.input(), env.output());

    let mut stats: HashMap<String, Person> = HashMap::new();

    for e in decoder.decode(&context, &mut input) {
        let m = match e {
            Ok(m) => m,
            Err(err) => error(Box::new(err))
        };

        match m {
            Event { ty: Type::Msg { ref from, ref content, .. }, .. } => {
                let nick = strip_nick_prefix(from);
                if stats.contains_key(nick) {
                    let p: &mut Person = stats.get_mut(nick).unwrap();
                    let (words, alpha) = words_alpha(content);
                    p.lines += 1;
                    if alpha { p.alpha_lines += 1 }
                    p.words += words;
                } else {
                    let (words, alpha) = words_alpha(content);
                    stats.insert(nick.to_owned(), Person {
                        lines: 1,
                        alpha_lines: if alpha { 1 } else { 0 },
                        words: words
                    });
                }
            },
            _ => ()
        }
    }

    let mut stats: Vec<(String, Person)> = stats.into_iter().collect();
    stats.sort_by(|&(_, ref a), &(_, ref b)| b.words.cmp(&a.words));

    for &(ref name, ref stat) in stats.iter() {
        let _ = write!(&mut output,
                       "{}:\n\tTotal lines: {}\n\tLines without alphabetic characters: {}\n\tTotal words: {}\n\tWords per line: {}\n",
                       name, stat.lines, stat.lines - stat.alpha_lines, stat.words, stat.words as f32 / stat.lines as f32);
    }
}
