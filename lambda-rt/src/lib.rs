#![no_std]
#[doc(hidden)]
pub extern crate alloc;
#[doc(hidden)]
pub mod __ {
    pub use core;
}
use core::sync::atomic::AtomicU8;

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use dyn_clone::DynClone;
#[derive(Clone)]
pub struct B(pub Arc<dyn O>);
pub trait O: Fn(&B) -> B {}
impl<T: Fn(&B) -> B> O for T {}
// dyn_clone::clone_trait_object!(O);

pub fn o<A: O + 'static>(a: A) -> B {
    return B(Arc::new(a));
}

pub fn K(a: B) -> B {
    return B(Arc::new(move |_| a.clone()));
}
pub fn l(a: impl Fn() -> B + Clone + 'static) -> B {
    return B(Arc::new(move |v| a().0(v)));
}
pub fn scott(a: usize, n: usize, b: Arc<[B]>) -> B {
    return B(Arc::new(move |mut x| {
        if a == 0 {
            let mut x = x.clone();
            for b in b.iter() {
                x = x.0(b);
            }
            for _ in 0..n {
                x = K(x);
            }
            return x;
        } else {
            return scott(a - 1, n - 1, b.clone());
        }
    }));
}
pub fn u8_from_term(a: B) -> u8 {
    pub fn go(a: B, x: Arc<AtomicU8>) {
        // let x1 = x.clone();
        let x2 = x.clone();
        a.0(&o(move |a| {
            return a.clone();
        }))
        .0(&o(move |a| {
            x2.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
            go(a.clone(), x2.clone());
            return a.clone();
        }))
        .0(&o(|a| a.clone()));
    }
    let mut x = Arc::new(AtomicU8::new(0));
    go(a, x.clone());
    return x.load(core::sync::atomic::Ordering::Relaxed);
}
pub fn u8_to_term(a: u8) -> B {
    return o(move |arg| {
        let arg = arg.clone();
        return o(move |arg2| {
            let arg2 = arg2.clone();
            if (a == 0) {
                return arg.clone();
            }
            return arg2.0(&u8_to_term(a - 1));
        });
    });
}
impl From<u8> for B {
    fn from(value: u8) -> Self {
        u8_to_term(value)
    }
}
impl From<bool> for B {
    fn from(value: bool) -> Self {
        o(move |a| match a.clone() {
            a => o(move |b| match b.clone() {
                b => {
                    if value {
                        a.clone()
                    } else {
                        b
                    }
                }
            }),
        })
    }
}
pub use gorf_gen_macro::lamc;
use spin::Mutex;
pub fn via_reader<T: Into<B> + Clone + 'static>(r: impl FnMut() -> T + 'static) -> B {
    struct State<F, T> {
        pub cache: Vec<T>,
        pub f: F,
    }
    fn go<F: FnMut() -> T + 'static, T: Into<B> + Clone + 'static>(
        s: Arc<Mutex<State<F, T>>>,
        i: usize,
    ) -> B {
        let j = {
            let mut lock = s.lock();
            while lock.cache.len() <= i {
                let f = (lock.f)();
                lock.cache.push(f);
            }
            lock.cache[i].clone()
        };
        o(move |arg| match s.clone() {
            s => match j.clone() {
                j => ((arg.0)(&j.into()).0)(&go(s, i + 1)),
            },
        })
    }
    return go(
        Arc::new(Mutex::new(State {
            cache: Default::default(),
            f: r,
        })),
        0,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_id(){
    //     let a = lamc!("\\a.a");
    // }
}
