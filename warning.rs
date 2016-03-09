use std::iter;

fn main() {
    let mut v = (0..20)
                    .map(|_| iter::empty::<()>())
                    .collect::<Vec<_>>();

    for i in 1..8 {
        v.remove(i + 1);
    }
}
