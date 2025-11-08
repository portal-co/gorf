use crate::*;
impl<V: Binder, M> GTerm<V, M> {
    pub fn to_args<'a>(&'a self, args: &mut Vec<&'a GTerm<V, M>>) -> &'a GTerm<V, M> {
        let GTerm::App(k) = self else {
            return self;
        };
        let (a, b) = k.as_ref();
        let a = a.to_args(args);
        args.push(b);
        return a;
    }
    pub fn map_vars<X: Binder<Var = X>>(
        self,
        f: &mut impl FnMut(V::Var) -> X,
    ) -> GTerm<V::Wrap<X>, M> {
        match self {
            GTerm::Undef => GTerm::Undef,
            GTerm::Var(v) => GTerm::Var(f(v)),
            GTerm::Abs(k) => {
                let (k, w) = *k;
                let k = k.inside(f);
                let w = w.map_vars(f);
                return abs(k, w);
            }
            GTerm::App(a) => {
                let (a, b) = *a;
                return app(a.map_vars(f), b.map_vars(f));
            }
            GTerm::Mix(m) => GTerm::Mix(m),
        }
    }
    pub fn map_all<X: Binder, Cx>(
        self,
        cx: &mut Cx,
        f: &mut (dyn FnMut(&mut Cx, V::Var) -> X::Var + '_),
        g: &mut (dyn FnMut(&mut Cx, V) -> X + '_),
    ) -> GTerm<X, M> {
        match self {
            GTerm::Undef => GTerm::Undef,
            GTerm::Var(v) => GTerm::Var(f(cx, v)),
            GTerm::Abs(k) => {
                let (k, w) = *k;
                let k = g(cx, k);
                let w = w.map_all(cx, f, g);
                return abs(k, w);
            }
            GTerm::App(a) => {
                let (a, b) = *a;
                return app(a.map_all(cx, f, g), b.map_all(cx, f, g));
            }
            GTerm::Mix(m) => GTerm::Mix(m),
        }
    }
    pub fn map_mix<N>(self, f: &mut impl FnMut(M) -> N) -> GTerm<V, N> {
        return self.lower_mix(&mut |a| GTerm::Mix(f(a)));
    }
    pub fn lower_mix<N>(self, f: &mut impl FnMut(M) -> GTerm<V, N>) -> GTerm<V, N> {
        match self {
            GTerm::Undef => GTerm::Undef,
            GTerm::Var(v) => GTerm::Var(v),
            GTerm::Abs(k) => {
                let (k, w) = *k;
                return abs(k, w.lower_mix(f));
            }
            GTerm::App(a) => {
                let (a, b) = *a;
                return app(a.lower_mix(f), b.lower_mix(f));
            }
            GTerm::Mix(m) => f(m),
        }
    }
    pub fn subst(self, f: &mut impl FnMut(&V::Var) -> Option<GTerm<V, M>>) -> GTerm<V, M> {
        match self {
            GTerm::Undef => GTerm::Undef,
            GTerm::Var(v) => match f(&v) {
                Some(a) => a,
                None => GTerm::Var(v),
            },
            GTerm::Abs(k) => {
                let (k, w) = *k;
                match f(k.get_var_ref()) {
                    Some(_) => abs(k, w),
                    None => abs(k, w.subst(f)),
                }
            }
            GTerm::App(b) => {
                let (a, b) = *b;
                return app(a.subst(f), b.subst(f));
            }
            GTerm::Mix(m) => GTerm::Mix(m),
        }
    }
}
impl<V: Binder, M> GTerm<V, M>
where
    V::Var: Eq,
{
    pub fn subst_var_fun(self, v: &V::Var, f: &mut impl FnMut() -> GTerm<V, M>) -> GTerm<V, M> {
        return self.subst(&mut |x| if x == v { Some(f()) } else { None });
    }
}
impl<V: Binder + Clone, M: Clone> GTerm<V, M>
where
    V::Var: Eq + Clone,
{
    pub fn subst_var(self, v: &V::Var, f: GTerm<V, M>) -> GTerm<V, M> {
        return self.subst_var_fun(v, &mut || f.clone());
    }
    pub fn apply(&mut self, o: GTerm<V, M>) {
        if let GTerm::Abs(a) = self {
            let (ref mut k, ref mut w) = **a;
            *self = w.clone().subst_var(k.get_var_mut(), o);
            return;
        }
        *self = app(self.clone(), o);
    }
}
impl<V: Binder + Clone, M: Clone> Scott<V, M>
where
    V::Var: Eq + Ord + Clone,
{
    pub fn apply(mut self, mut other: GTerm<V, M>) -> Either<GTerm<V, M>, Scott<V, M>> {
        if self.current_case == 0 {
            for x in self.with.into_iter() {
                other.apply(x);
            }
            return Left(other);
        }
        self.current_case -= 1;
        self.cases.pop();
        return Right(self);
    }
    pub fn render(mut self) -> GTerm<V, M>
    where
        V: Binder<Var = V>,
    {
        let mut r = var(self.cases[self.current_case].clone());
        for w in self.with.into_iter() {
            r = app(r, w);
        }
        for c in self.cases.into_iter().rev() {
            r = abs(c, r);
        }
        return r;
    }
}
impl<V: Binder + Clone + Eq, M: Clone + Eq> GTerm<V, M>
where
    V::Var: Eq + Ord + Clone,
{
    pub fn is_sapp(&self) -> Option<GTerm<V, M>> {
        if let GTerm::App(a) = self {
            let (ref a, ref b) = **a;
            if a.clone() == b.clone() {
                return Some(a.clone());
            }
        }
        None
    }
}
impl<'a, V: Binder + Clone + Eq, M: Clone + Eq> GTermRef<'a, V, M>
where
    V::Var: Eq + Ord + Clone,
{
    pub fn is_sapp(&self) -> Option<GTermRef<'a, V, M>> {
        if let GTermRef::App(a) = self {
            let (ref a, ref b) = **a;
            if a.clone() == b.clone() {
                return Some(a.clone());
            }
        }
        None
    }
}
impl<V: Binder + Clone, M: Clone> GTermRef<'_, V, M>
where
    V::Var: Eq + Ord + Clone,
{
    pub fn frees_internal(&self, o: &mut BTreeSet<V::Var>) {
        match self {
            GTermRef::Undef => {}
            GTermRef::Var(s) => {
                o.insert(s.clone());
            }
            GTermRef::Abs(a) => {
                let (ref k, ref w) = **a;
                let mut r = w.frees();
                r.remove(k.get_var_ref());
                o.append(&mut r);
            }
            GTermRef::App(a) => {
                let (ref a, ref b) = **a;
                a.frees_internal(o);
                b.frees_internal(o);
            }
            GTermRef::Mix(m) => {}
        }
    }
    pub fn frees(&self) -> BTreeSet<V::Var> {
        let mut r = BTreeSet::new();
        self.frees_internal(&mut r);
        return r;
    }
}
impl<V: Binder + Clone, M: Clone> GTerm<V, M>
where
    V::Var: Eq + Ord + Clone,
{
    pub fn frees_internal(&self, o: &mut BTreeSet<V::Var>) {
        match self {
            GTerm::Undef => {}
            GTerm::Var(s) => {
                o.insert(s.clone());
            }
            GTerm::Abs(a) => {
                let (ref k, ref w) = **a;
                let mut r = w.frees();
                r.remove(k.get_var_ref());
                o.append(&mut r);
            }
            GTerm::App(a) => {
                let (ref a, ref b) = **a;
                a.frees_internal(o);
                b.frees_internal(o);
            }
            GTerm::Mix(m) => {}
        }
    }
    pub fn frees(&self) -> BTreeSet<V::Var> {
        let mut r = BTreeSet::new();
        self.frees_internal(&mut r);
        return r;
    }
    pub fn r#let(self) -> Let<V, M> {
        let mut l = Let {
            body: self,
            vars: BTreeMap::new(),
        };
        loop {
            let GTerm::App(a) = l.body.clone() else {
                return l;
            };
            let GTerm::Abs(b) = a.0 else {
                return l;
            };
            l.vars.insert(b.0.get_var(), a.1);
            l.body = b.1;
        }
    }
    pub fn scott(&self) -> Option<Scott<V, M>> {
        let mut this = self;
        let mut v = vec![];
        while let GTerm::Abs(k) = this {
            let (ref k, ref w) = **k;
            v.push(k.clone().get_var());
            this = w;
        }
        let mut args = vec![];
        loop {
            if let GTerm::App(b) = this {
                let (ref a, ref b) = **b;
                // if let GTerm::Var(r) = a {
                //     // if v{
                //     //     return Some((v.len(),args));
                //     // }else{
                //     //     return None;
                //     // }
                //     match v.iter().enumerate().find(|a| a.1 == r.get_var_ref()) {
                //         Some((a, b)) => {
                //             return Some(Scott {
                //                 case_count: v.len(),
                //                 current_case: a,
                //                 with: args.into_iter().rev().collect(),
                //                 pda: PhantomData,
                //             })
                //         }
                //         None => return None,
                //     }
                // }
                for f in b.frees() {
                    if !v.contains(&f) {
                        return None;
                    }
                }
                args.push(b.clone());
                this = a;
            } else {
                break;
            }
        }
        if let GTerm::Var(r) = this {
            match v.iter().enumerate().find(|a| a.1 == r.get_var_ref()) {
                Some((a, b)) => {
                    return Some(Scott {
                        cases: v,
                        current_case: a,
                        with: args.into_iter().rev().collect(),
                    })
                }
                None => return None,
            }
        }
        return None;
    }
}
