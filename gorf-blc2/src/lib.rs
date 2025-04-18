#![no_std]
extern crate alloc;
use alloc::{boxed::Box, vec::Vec};
use lambda_calculus::Term;
pub fn le(a: usize) -> Box<dyn Iterator<Item = bool>>{
    if a == 0{
        return Box::new([true,false].into_iter());
    }
    let len = usize::BITS - a.leading_zeros();
    let x = if len == 1{
        Vec::default()
    }else{
        (0..len - 2).rev().map(|a|1 << a).map(|v|a & v != 0).collect()
    };
    return Box::new([true].into_iter().chain(le((len - 1) as usize)).chain(x.into_iter()));
}
pub fn encode(t: &Term) -> Box<dyn Iterator<Item = bool> + '_>{
    match t{
        Term::Var(v) => le(*v),
        Term::Abs(term) => Box::new([false,false].into_iter().chain(encode(&*term))),
        Term::App(t) => {
            let (a,b) = t.as_ref();
            Box::new([false,true].into_iter().chain(encode(a)).chain(encode(b)))
        },
    }
}
pub fn ld(a: &mut impl Iterator<Item = bool>) -> Option<usize>{
    if !a.next()?{
        return Some(0);
    }
    let mut x = 1;
    let l = ld(a)?;
    for _ in (0..l).rev(){
        x = 2 * x + (if a.next()?{
            1
        }else{
            0
        })
    };
    return Some(x);;
}
pub fn decode(a: &mut impl Iterator<Item = bool>) -> Option<Term>{
    match a.next()?{
        true => Some(Term::Var(ld(a)?)),
        false => match a.next()?{
            false => Some(Term::Abs(Box::new(decode(a)?))),
            true => Some(Term::App(Box::new((
                decode(a)?,
                decode(a)?
            ))))
        }
    }
}