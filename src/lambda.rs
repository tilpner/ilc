// I'm only a little sorry for this.

// Inline definition of anonymous functions. Examples:
// l!(42;)
// l!(i32 := 42)
// l!(i: i32 := i + 42)
// l!(i: i32, j: i32 -> i32 := i + j)
#[macro_export]
macro_rules! l {
    ($body: expr) => ({ fn f() { $body } f });
    ($res: ty := $body: expr) => ({ fn f() -> $res { $body } f });
    ($($n: ident: $t: ty),+ := $body: expr) => ({
        fn f($($n: $t),+) { $body } f
    });
    ($($n: ident: $t: ty),+ -> $res: ty := $body: expr) => ({
        fn f($($n: $t),+) -> $res { $body } f
    })
}
