#![no_std]
extern crate alloc;
use alloc::{
    boxed::Box, collections::{BTreeMap, BTreeSet}, string::String, vec::Vec};use core::{
    fmt::Display,
};
use alloc::format;

use gorf_core::{Binder, GTerm};
use serde::{Deserialize, Serialize};
pub mod make;
#[derive(Serialize, Deserialize)]
pub struct Symbol<V: Binder, M> {
    pub deps: Vec<V>,
    #[serde(bound(
        deserialize = "GTerm<V,M>: Deserialize<'de>",
        serialize = "GTerm<V,M>: Serialize"
    ))]
    pub r#impl: GTerm<V, M>,
}

impl<V: Binder<Var: Clone> + Clone, M: Clone> Clone for Symbol<V, M> {
    fn clone(&self) -> Self {
        Self {
            deps: self.deps.clone(),
            r#impl: self.r#impl.clone(),
        }
    }
}

impl<V: Binder, M> Symbol<V, M> {
    pub fn emit(self) -> GTerm<V, M> {
        return self
            .deps
            .into_iter()
            .fold(self.r#impl, |s, d| GTerm::Abs(Box::new((d, s))));
    }
}
#[derive(Serialize, Deserialize)]
pub struct Obj<V: Binder + Ord, M> {
    #[serde(bound(
        deserialize = "Symbol<V,M>: Deserialize<'de>, V: Deserialize<'de>",
        serialize = "Symbol<V,M>: Serialize, V: Serialize"
    ))]
    pub syms: BTreeMap<V, Symbol<V, M>>,
}
impl<V: Binder<Var: Clone> + Clone + Ord, M: Clone> Clone for Obj<V, M> {
    fn clone(&self) -> Self {
        Self {
            syms: self.syms.clone(),
        }
    }
}
impl<V: Binder<Var: Clone> + From<String> + Display + Ord + Clone, M: Clone> Obj<V, M> {
    pub fn sees(&self, v: &V, w: &V) -> bool {
        if v == w {
            return true;
        }
        match self.syms.get(v) {
            None => false,
            Some(s) => s.deps.iter().any(|d| self.sees(d, w)),
        }
    }
    pub fn rbake(&self, v: &V, stack: &BTreeSet<V>) -> GTerm<V, M> {
        if stack.contains(v) {
            let v = gorf_core::var(V::get_var(format!("@{v}").into()));
            return gorf_core::app(v.clone(), v);
        }
        let mut stack = stack.clone();
        stack.insert(v.clone());
        let s = self.syms.get(v).unwrap();
        let mut b = GTerm::Var(v.get_var_ref().clone());
        for d in s.deps.iter() {
            b = gorf_core::app(b, self.rbake(d, &stack))
        }

        return gorf_core::app(
            gorf_core::abs(
                v.clone(),
                gorf_core::app(
                    gorf_core::var(v.get_var_ref().clone()),
                    gorf_core::var(v.get_var_ref().clone()),
                ),
            ),
            gorf_core::abs(format!("@{v}").into(), b),
        );
    }
    pub fn link(&self, root: &V) -> GTerm<V, M> {
        let mut x = self.rbake(root, &BTreeSet::new());
        for (k, v) in self.syms.clone().into_iter() {
            x = gorf_core::app(gorf_core::abs(k, x), v.emit());
        }
        return x;
    }
}
