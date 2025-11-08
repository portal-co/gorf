#![no_std]
extern crate alloc;
use alloc::boxed::Box;
use core::iter::once;
use lambda_calculus::Term;
pub fn encode(a: &Term) -> Box<dyn Iterator<Item = bool> + '_> {
    match a {
        Term::Var(a) => Box::new((0..=*a).map(|_| true).chain(once(false))),
        Term::Abs(term) => Box::new([false, false].into_iter().chain(encode(&term))),
        Term::App(a) => {
            let (a, b) = &**a;
            return Box::new([false, true].into_iter().chain(encode(a)).chain(encode(b)));
        }
    }
}
pub fn decode(a: &mut (dyn Iterator<Item = bool> + '_)) -> Option<Term> {
    Some(match a.next()? {
        true => {
            let mut i = 0;
            while a.next()? {
                i += 1
            }
            Term::Var(i)
        }
        false => match a.next()? {
            false => Term::Abs(Box::new(decode(a)?)),
            true => Term::App(Box::new((decode(a)?, decode(a)?))),
        },
    })
}
