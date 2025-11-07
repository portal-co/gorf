use std::{collections::BTreeSet, iter::{empty, once}};
use anyhow::Context;
use either::Either;
use id_arena::Id;
use rat_ir::{
    module::{Module, TailCall},
    util::{If, Ret},
    Block, BlockTarget, Bound, BoundExt, BoundOp, BoundSelect, BoundTerm, BoundType, Call, Func,
    Use, Value,
};
use crate::{abs, app, var, Binder, GTerm};
pub trait Thing<O, T, Y, S, D, V: Binder, M> {
    fn go(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
        k: Id<Block<O, T, Y, S>>,
    ) -> anyhow::Result<GTerm<V, M>>;
    fn targets(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
    ) -> impl Iterator<Item = (Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)>;
}
impl<O, T, Y, S, D, V: Binder, M, A: Thing<O, T, Y, S, D, V, M>, B: Thing<O, T, Y, S, D, V, M>>
    Thing<O, T, Y, S, D, V, M> for Either<A, B>
{
    fn go(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
        k: Id<Block<O, T, Y, S>>,
    ) -> anyhow::Result<GTerm<V, M>> {
        match self {
            Either::Left(a) => a.go(m, f, k),
            Either::Right(b) => b.go(m, f, k),
        }
    }
    fn targets(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
    ) -> impl Iterator<Item = (Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)> {
        match self {
            Either::Left(a) => Either::Left(a.targets(m, f)),
            Either::Right(b) => Either::Right(b.targets(m, f)),
        }
    }
}
impl<D, V: Binder, M, B: Bound>
    Thing<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>, D, V, M> for BoundOp<B>
where
    B::O<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>:
        Thing<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>, D, V, M>,
{
    fn go(
        &self,
        m: &Module<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>, D>,
        f: Id<Func<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>>,
        k: Id<Block<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>>,
    ) -> anyhow::Result<GTerm<V, M>> {
        self.0.go(m, f, k)
    }
    fn targets(
        &self,
        m: &Module<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>, D>,
        f: Id<Func<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>>,
    ) -> impl Iterator<
        Item = (
            Id<Func<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>>,
            Id<Block<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>>,
        ),
    > {
        self.0.targets(m, f)
    }
}
impl<D, V: Binder, M, B: Bound>
    Thing<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>, D, V, M> for BoundTerm<B>
where
    B::T<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>:
        Thing<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>, D, V, M>,
{
    fn go(
        &self,
        m: &Module<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>, D>,
        f: Id<Func<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>>,
        k: Id<Block<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>>,
    ) -> anyhow::Result<GTerm<V, M>> {
        self.0.go(m, f, k)
    }
    fn targets(
        &self,
        m: &Module<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>, D>,
        f: Id<Func<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>>,
    ) -> impl Iterator<
        Item = (
            Id<Func<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>>,
            Id<Block<BoundOp<B>, BoundTerm<B>, BoundType<B>, BoundSelect<B>>>,
        ),
    > {
        self.0.targets(m, f)
    }
}
pub fn reachable<O: Thing<O, T, Y, S, D, V, M>, T: Thing<O, T, Y, S, D, V, M>, Y, S, D, V: Binder, M>(
    m: &Module<O, T, Y, S, D>,
    src: &(Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>),
    dst: &(Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>),
    stack: &BTreeSet<(Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)>
) -> bool {
    if dst == src {
        return true;
    }
    if stack.contains(src){
        return false;
    }
    let mut stack = stack.clone();
    stack.insert(src.clone());
    let f = &m.funcs[src.0];
    for i in f.blocks[src.1].insts.iter() {
        if let Value::Operator(a, _, _, _) = &f.opts[*i] {
            for x in a.targets(m, src.0) {
                if reachable(m, &x, dst,&stack) {
                    return true;
                }
            }
        }
    }
    if let Some(a) = f.blocks[src.1].term.as_ref(){
        for x in a.targets(m, src.0) {
            if reachable(m, &x, dst,&stack) {
                return true;
            }
        }
    }
    false
}
pub fn vars<
    'a,
    O: Thing<O, T, Y, S, D, V, M>,
    T: Thing<O, T, Y, S, D, V, M>,
    Y,
    S,
    D,
    V: Binder<Var = V>
        + From<(Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)>
        + From<Id<Value<O, T, Y, S>>>
        + From<String>
        + 'a,
        M,
>(
    m: &'a Module<O, T, Y, S, D>,
    f: Id<Func<O, T, Y, S>>,
    k: Id<Block<O, T, Y, S>>,
) -> impl DoubleEndedIterator<Item = V> + 'a {
    return m
        .funcs
        .iter()
        .flat_map(|x| x.1.blocks.iter().map(move |k| (x.0, k.0)))
        .filter(move |a| reachable(m, a, &(f, k),&BTreeSet::new()))
        .map(V::from);
}
pub fn push<
    O: Thing<O, T, Y, S, D, V, M>,
    T: Thing<O, T, Y, S, D, V, M>,
    Y,
    S,
    D,
    V: Binder<Var = V>
        + From<(Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)>
        + From<Id<Value<O, T, Y, S>>>
        + From<String>,
    M,
>(
    m: &Module<O, T, Y, S, D>,
    f: Id<Func<O, T, Y, S>>,
    k: Id<Block<O, T, Y, S>>,
) -> anyhow::Result<GTerm<V, M>> {
    let mut t = m.funcs[f].blocks[k]
        .term
        .as_ref()
        .context("undefined terminators are unsupported")?
        .go(m, f, k)?;
    for v in m.funcs[f].blocks[k].insts.iter() {
        let u = match &m.funcs[f].opts[*v] {
            Value::Operator(o, r, _, _) => {
                let mut u = o.go(m, f, k)?;
                for us in r.iter() {
                    u = app(u, var(V::from(us.value)));
                }
                u
            }
            Value::BlockParam(i, _, _) => var(V::from(format!("param{i}"))),
            Value::Alias(u, _) => var(V::from(u.value)),
        };
        t = app(u, abs(V::from(*v), t));
    }
    t = abs(V::from(format!("return")), t);
    for (i, _) in m.funcs[f].blocks[k].params.iter().enumerate().rev() {
        t = abs(V::from(format!("param{i}")), t);
    }
    for v in vars(m, f, k).rev() {
        t = abs(v, t);
    }
    return Ok(t);
}
impl<
        O: Thing<O, T, Y, S, D, V, M>,
        T: Thing<O, T, Y, S, D, V, M>,
        Y,
        S,
        D,
        V: Binder<Var = V>
            + From<(Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)>
            + From<Id<Value<O, T, Y, S>>>
            + From<String>,
        M,
    > Thing<O, T, Y, S, D, V, M> for BlockTarget<O, T, Y, S>
{
    fn go(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
        k: Id<Block<O, T, Y, S>>,
    ) -> anyhow::Result<GTerm<V, M>> {
        let mut t = var(V::from((f, self.block)));
        for v in vars(m, f, self.block) {
            t = app(t, var(v));
        }
        for (p, _) in self.prepend.iter().enumerate() {
            let p: V = format!("prepend{p}").into();
            t = app(t, var(p))
        }
        for r in self.args.iter() {
            t = app(t, var(V::from(r.value)));
        }
        t = app(t, var(V::from(format!("return"))));
        for (p, _) in self.prepend.iter().enumerate().rev() {
            let p: V = format!("prepend{p}").into();
            t = abs(p, t);
        }
        return Ok(t);
    }
    fn targets(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
    ) -> impl Iterator<Item = (Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)> {
        once((f, self.block))
    }
}
impl<
        O: 'static,
        T: 'static,
        Y: 'static,
        S: 'static,
        D,
        V: Binder<Var = V>
            + From<(Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)>
            + From<Id<Value<O, T, Y, S>>>
            + From<String>,
        M,
        W: Thing<O, T, Y, S, D, V, M>,
    > Thing<O, T, Y, S, D, V, M> for If<O, T, Y, S, W>
{
    fn go(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
        k: Id<Block<O, T, Y, S>>,
    ) -> anyhow::Result<GTerm<V, M>> {
        let then = self.then.go(m, f, k)?;
        let els = self.r#else.as_ref().unwrap().go(m, f, k)?;
        let v = var(V::from(self.val.value));
        return Ok(app(app(v, then), els));
    }
    fn targets(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
    ) -> impl Iterator<Item = (Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)> {
        self.then
            .targets(&*m, f)
            .chain(self.r#else.iter().flat_map(move |a| a.targets(&*m, f)))
    }
}
impl<
        O: Thing<O, T, Y, S, D, V, M>,
        T: Thing<O, T, Y, S, D, V, M>,
        Y,
        S,
        D,
        V: Binder<Var = V>
            + From<(Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)>
            + From<Id<Value<O, T, Y, S>>>
            + From<String>,
        M,
    > Thing<O, T, Y, S, D, V, M> for TailCall<O, T, Y, S>
{
    fn go(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
        k: Id<Block<O, T, Y, S>>,
    ) -> anyhow::Result<GTerm<V, M>> {
        let mut t = var(V::from((self.func, m.funcs[self.func].entry)));
        for v in vars(m, self.func, m.funcs[self.func].entry) {
            t = app(t, var(v));
        }
        for r in self.params.iter() {
            t = app(t, var(V::from(r.value)));
        }
        t = app(t, var(V::from(format!("return"))));
        return Ok(t);
    }
    fn targets(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
    ) -> impl Iterator<Item = (Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)> {
        once((self.func, m.funcs[self.func].entry))
    }
}
impl<
        O: Thing<O, T, Y, S, D, V, M>,
        T: Thing<O, T, Y, S, D, V, M>,
        Y,
        S,
        D,
        V: Binder<Var = V>
            + From<(Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)>
            + From<Id<Value<O, T, Y, S>>>
            + From<String>,
        M,
    > Thing<O, T, Y, S, D, V, M> for Call<O, T, Y, S>
{
    fn go(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
        k: Id<Block<O, T, Y, S>>,
    ) -> anyhow::Result<GTerm<V, M>> {
        let mut t = var(V::from((self.func, m.funcs[self.func].entry)));
        for v in vars(m, self.func, m.funcs[self.func].entry) {
            t = app(t, var(v));
        }
        return Ok(t);
    }
    fn targets(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
    ) -> impl Iterator<Item = (Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)> {
        once((self.func, m.funcs[self.func].entry))
    }
}
// impl<
//         O,
//         T,
//         Y,
//         S,
//         D,
//         V: Binder<Var = V>
//             + From<(Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)>
//             + From<Id<Value<O, T, Y, S>>>
//             + From<String>,
//         M,
//     > Thing<O, T, Y, S, D, V, M> for crate::rat::export::CallVar
// {
//     fn go(
//         &self,
//         m: &Module<O, T, Y, S, D>,
//         f: Id<Func<O, T, Y, S>>,
//         k: Id<Block<O, T, Y, S>>,
//     ) -> anyhow::Result<GTerm<V, M>> {
//         Ok(abs(V::from(format!("$")), var(V::from(format!("$")))))
//     }
//     fn targets(
//         &self,
//         m: &Module<O, T, Y, S, D>,
//         f: Id<Func<O, T, Y, S>>,
//     ) -> impl Iterator<Item = (Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)> {
//         empty()
//     }
// }
impl<
        O,
        T,
        Y,
        S,
        D,
        V: Binder<Var = V>
            + From<(Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)>
            + From<Id<Value<O, T, Y, S>>>
            + From<String>,
        M,
    > Thing<O, T, Y, S, D, V, M> for Ret<Use<O, T, Y, S>>
{
    fn go(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
        k: Id<Block<O, T, Y, S>>,
    ) -> anyhow::Result<GTerm<V, M>> {
        return Ok(app(
            var(V::from(format!("return"))),
            var(V::from(self.wrapped.value)),
        ));
    }
    fn targets(
        &self,
        m: &Module<O, T, Y, S, D>,
        f: Id<Func<O, T, Y, S>>,
    ) -> impl Iterator<Item = (Id<Func<O, T, Y, S>>, Id<Block<O, T, Y, S>>)> {
        empty()
    }
}
