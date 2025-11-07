use crate::*;
#[derive(Eq, Ord, Clone, PartialEq, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub struct Base<T>(pub T);
simple_binder!( Base<T> => <T>);
#[derive(Eq, Ord, Clone, PartialEq, PartialOrd, Hash, Debug, Serialize, Deserialize)]
pub enum GTerm<V: Binder, M> {
    Undef,
    Var(V::Var),
    Abs(Box<(V, GTerm<V, M>)>),
    App(Box<(GTerm<V, M>, GTerm<V, M>)>),
    Mix(M),
}
#[derive(Eq, Ord, Clone, PartialEq, PartialOrd, Hash, Debug, Serialize)]
pub enum GTermRef<'a, V: Binder, M> {
    Undef,
    Var(V::Var),
    Abs(&'a (V, GTermRef<'a, V, M>)),
    App(&'a (GTermRef<'a, V, M>, GTermRef<'a, V, M>)),
    Mix(M),
}
#[derive(Eq, Ord, Clone, PartialEq, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Scope<T: Binder>(pub BTreeMap<T::Var, T>);

#[derive(Eq, Ord, Clone, PartialEq, PartialOrd, Hash)]
pub struct Scott<V: Binder, M> {
    pub cases: Vec<V::Var>,
    pub current_case: usize,
    pub with: Vec<GTerm<V, M>>,
}
#[derive(Eq, Ord, Clone, PartialEq, PartialOrd, Hash)]
pub struct Let<V: Binder<Var: Ord>, M> {
    pub vars: BTreeMap<V::Var, GTerm<V, M>>,
    pub body: GTerm<V, M>,
}