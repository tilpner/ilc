#![macro_escape]

use std::ptr;
use std::cell::UnsafeCell;
use std::ops::Deref;
use std::boxed::FnBox;

pub enum State<V> {
    Evaluated(V),
    Evaluating,
    Unevaluated(Box<FnBox() -> V>)
}

pub struct Lazy<V> {
    state: UnsafeCell<State<V>>,
}

impl<V> Lazy<V> {
    pub fn new(f: Box<FnBox() -> V>) -> Lazy<V> {
        Lazy { state: UnsafeCell::new(State::Unevaluated(f)) }
    }

    pub fn ready(v: V) -> Lazy<V> {
        Lazy { state: UnsafeCell::new(State::Evaluated(v)) }
    }

    pub fn force(&mut self) {
        self.value();
    }

    pub fn unwrap(self) -> V {
        unsafe {
            match self.state.into_inner() {
                State::Unevaluated(f) => f(),
                State::Evaluating => panic!("Illegal state, can't call unwrap during evaluation"),
                State::Evaluated(v) => v
            }
        }
    }

    pub fn value(&self) -> &V {
        unsafe {
            let state = self.state.get();
            match *state {
                State::Evaluated(ref v) => v,
                State::Evaluating => panic!("Illegal state, can't call value during evaluation"),
                State::Unevaluated(_) => {
                    if let State::Unevaluated(f) = ptr::replace(state, State::Evaluating) {
                        ptr::replace(state, State::Evaluated(f()));
                    }
                    if let State::Evaluated(ref v) = *state { return v }
                    unreachable!()
                }
            }
        }
    }
}

impl<V> Deref for Lazy<V> {
    type Target = V;
    fn deref(&self) -> &V { self.value() }
}

#[macro_export]
macro_rules! lazy {
    (@as_expr $val: expr) => { $val };
    ($($val: tt)*) => { Lazy::new(Box::new(move || lazy![@as_expr { $($val)* }])) };
}

#[macro_export]
macro_rules! eager {
    (@as_expr $val: expr) => { $val };
    ($($val: tt)*) => { Lazy::<_>::ready(eager![@as_expr { $($val)* }]) };
}
