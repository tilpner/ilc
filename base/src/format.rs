use std::borrow::Cow;

pub fn rejoin(s: &[&str], splits: &[char]) -> Cow<'static, str> {
    let len = s.iter().map(|s| s.len()).fold(0, |a, b| a + b);
    let mut out = s.iter()
                   .zip(splits.iter())
                   .fold(String::with_capacity(len), |mut s, (b, &split)| {
                       s.push_str(b);
                       s.push(split);
                       s
                   });
    out.pop();
    Cow::Owned(out)
}

pub fn strip_one(s: &str) -> String {
    if s.len() >= 2 { s[1..(s.len() - 1)].to_owned() } else { String::new() }
}
