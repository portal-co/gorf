use crate::*;
pub fn debrijun_internal<X: From<usize> + Binder, Y>(x: Term, depth: usize) -> GTerm<X, Y>
where
    <X as Binder>::Var: From<usize>,
{
    match x {
        Var(0) => GTerm::Undef,
        Var(v) => GTerm::Var((depth - v).into()),
        Abs(a) => GTerm::Abs(Box::new((depth.into(), debrijun_internal(*a, depth + 1)))),
        App(b) => {
            let (a, b) = *b;
            let a = debrijun_internal(a, depth);
            let b = debrijun_internal(b, depth);
            return GTerm::App(Box::new((a, b)));
        }
    }
}
pub fn debrijun<X: From<usize> + Binder, Y>(x: Term) -> GTerm<X, Y>
where
    <X as Binder>::Var: From<usize>,
{
    return debrijun_internal(x, 1);
}
pub fn brujin_internal<X: Binder>(
    t: GTerm<X, Infallible>,
    m: &BTreeMap<<X as Binder>::Var, usize>,
) -> Term
where
    <X as Binder>::Var: Eq + Ord + Clone,
{
    match t {
        GTerm::Undef => Var(0),
        GTerm::Var(a) => Var(m[&a]),
        GTerm::Abs(a) => {
            let (k, w) = *a;
            let mut n = BTreeMap::new();
            for (a, b) in m.iter() {
                n.insert(a.clone(), *b + 1);
            }
            n.insert(k.get_var(), 1);
            return Abs(Box::new(brujin_internal(w, &n)));
        }
        GTerm::App(b) => {
            let (a, b) = *b;
            let a = brujin_internal(a, m);
            let b = brujin_internal(b, m);
            return App(Box::new((a, b)));
        }
        GTerm::Mix(x) => match x {},
    }
}
pub fn brujin<X: Binder>(t: GTerm<X, Infallible>) -> Term
where
    <X as Binder>::Var: Eq + Ord + Clone,
{
    return brujin_internal(t, &BTreeMap::new());
}
pub fn brujin_map_f_internal<X: Binder, Y: Binder, M>(
    t: GTerm<X, Infallible>,
    m: &BTreeMap<<X as Binder>::Var, usize>,
    depth: usize,
    into: &mut impl FnMut(usize) -> Y,
) -> GTerm<Y, M>
where
    <X as Binder>::Var: Eq + Ord + Clone,
{
    match t {
        GTerm::Undef => GTerm::Undef,
        GTerm::Var(a) => GTerm::Var(into(depth - m[&a]).get_var()),
        GTerm::Abs(a) => {
            let (k, w) = *a;
            let mut n = BTreeMap::new();
            for (a, b) in m.iter() {
                n.insert(a.clone(), *b + 1);
            }
            n.insert(k.get_var(), 1);
            return abs(into(depth), brujin_map_f_internal(w, &n, depth + 1, into));
        }
        GTerm::App(b) => {
            let (a, b) = *b;
            let a = brujin_map_f_internal(a, m, depth, into);
            let b = brujin_map_f_internal(b, m, depth, into);
            return app(a, b);
        }
        GTerm::Mix(x) => match x {},
    }
}
pub fn brujin_map<X: Binder, Y: Binder + From<usize>, M>(t: GTerm<X, Infallible>) -> GTerm<Y, M>
where
    <X as Binder>::Var: Eq + Ord + Clone,
    <Y as Binder>::Var: From<usize>,
{
    return brujin_map_f_internal(t, &BTreeMap::new(), 1, &mut |a| a.into());
}
pub fn brujin_map_f<X: Binder, Y: Binder, M>(
    t: GTerm<X, Infallible>,
    into: &mut impl FnMut(usize) -> Y,
) -> GTerm<Y, M>
where
    <X as Binder>::Var: Eq + Ord + Clone,
{
    return brujin_map_f_internal(t, &BTreeMap::new(), 1, into);
}
pub fn brujin_inflate_f_internal<X: Binder, Y: Binder, M>(
    t: &GTermRef<'_,X, Infallible>,
    m: &BTreeMap<<X as Binder>::Var, usize>,
    depth: usize,
    into: &mut impl FnMut(usize) -> Y,
) -> GTerm<Y, M>
where
    <X as Binder>::Var: Eq + Ord + Clone
{
    match t {
        GTermRef::Undef => GTerm::Undef,
        GTermRef::Var(a) => GTerm::Var(into(depth - m[&a]).get_var()),
        GTermRef::Abs(a) => {
            let (k, w) = *a;
            let mut n = BTreeMap::new();
            for (a, b) in m.iter() {
                n.insert(a.clone(), *b + 1);
            }
            n.insert(k.get_var_ref().clone(), 1);
            return abs(into(depth), brujin_inflate_f_internal(w, &n, depth + 1, into));
        }
        GTermRef::App(b) => {
            let (a, b) = *b;
            let a = brujin_inflate_f_internal(a, m, depth, into);
            let b = brujin_inflate_f_internal(b, m, depth, into);
            return app(a, b);
        }
        GTermRef::Mix(x) => match *x {},
    }
}
pub fn brujin_inflate<X: Binder, Y: Binder + From<usize>, M>(t: &GTermRef<'_,X, Infallible>) -> GTerm<Y, M>
where
    <X as Binder>::Var: Eq + Ord + Clone,
    <Y as Binder>::Var: From<usize>,
{
    return brujin_inflate_f_internal(t, &BTreeMap::new(), 1, &mut |a| a.into());
}
pub fn brujin_inflate_f<X: Binder, Y: Binder, M>(
    t: &GTermRef<'_,X, Infallible>,
    into: &mut impl FnMut(usize) -> Y,
) -> GTerm<Y, M>
where
    <X as Binder>::Var: Eq + Ord + Clone,
{
    return brujin_inflate_f_internal(t, &BTreeMap::new(), 1, into);
}
