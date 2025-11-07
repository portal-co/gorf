use crate::{Obj, Symbol};
use alloc::collections::BTreeMap;
use alloc::{format, vec};
use core::convert::Infallible;
use gorf_core::{brujin, brujin::debrijun, Id};
use gorf_kiselyov::{RealSki, Ski};
pub fn bake_ski(a: &Ski, x: &mut Obj<Id, Infallible>) -> Id {
    let i: Id = format!("__{a:?}").into();
    if x.syms.contains_key(&i) {
        return i;
    };
    let s = match a {
        Ski::S => Symbol {
            r#impl: debrijun(lambda_calculus::combinators::S()),
            deps: vec![],
        },
        Ski::K => Symbol {
            r#impl: debrijun(lambda_calculus::combinators::K()),
            deps: vec![],
        },
        Ski::B => Symbol {
            r#impl: debrijun(lambda_calculus::combinators::B()),
            deps: vec![],
        },
        Ski::R => Symbol {
            r#impl: debrijun(lambda_calculus::combinators::R()),
            deps: vec![],
        },
        Ski::U => Symbol {
            deps: vec![],
            r#impl: debrijun(lambda_calculus::app(
                lambda_calculus::app(
                    lambda_calculus::combinators::S(),
                    lambda_calculus::combinators::I(),
                ),
                lambda_calculus::combinators::I(),
            )),
        },
        Ski::I => Symbol {
            r#impl: debrijun(lambda_calculus::combinators::I()),
            deps: vec![],
        },
        Ski::App(ski, ski1) => Symbol {
            deps: vec![bake_ski(ski.as_ref(), &mut *x), bake_ski(ski1.as_ref(), x)],
            r#impl: debrijun(lambda_calculus::combinators::I()),
        },
    };
    x.syms.insert(i.clone(), s);
    return i;
}
pub fn bake(a: &Symbol<Id, Infallible>, p: &str, x: &mut Obj<Id, Infallible>) {
    let s = bake_ski(&Ski::convert_default(brujin::brujin(a.r#impl.clone())), x);
    x.syms.insert(
        Id(format!("{p}")),
        Symbol {
            deps: [s].into_iter().chain(a.deps.iter().cloned()).collect(),
            r#impl: debrijun(lambda_calculus::combinators::I()),
        },
    );
}
pub fn cache(a: &Obj<Id, Infallible>) -> Obj<Id, Infallible> {
    return a.syms.iter().fold(
        Obj {
            syms: BTreeMap::new(),
        },
        |mut x, (si, s)| {
            bake(s, &si.0, &mut x);
            return x;
        },
    );
}
