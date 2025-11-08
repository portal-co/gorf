use crate::*;
pub struct Opts {
    pub path: TokenStream,
}
pub fn emit(a: &GTerm<String, Infallible>, opts: &Opts) -> proc_macro2::TokenStream {
    let rt = &opts.path;
    let n = {
        if let Some(s) = a.scott() {
            // let mut v = quote! {
            //     let mut _tv = #rt::alloc::vec::Vec::new();
            // };
            // for a in s.with {
            //     let a = emit(&a, opts);
            //     v = quote! {
            //         #v;
            //         _tv.push(#a)
            //     }
            // }
            let i = s.current_case;
            let n = s.cases.len();
            let with = s.with.iter().map(|a| emit(a, opts));
            let v = quote! {
                #rt::scott(#i,#n,#rt::alloc::sync::Arc::new([#(#with),*]))
            };
            return v;
        }
        match a {
            GTerm::Undef => quote! {
                unreachable!()
            },
            GTerm::Var(v) => {
                let v = format_ident!("{v}");
                quote! {
                    #rt::__::core::clone::Clone::clone(&#v)
                }
            }
            GTerm::Abs(b) => {
                let (b, v) = b.as_ref();
                let w = v.frees();
                let f = w.iter().map(|a| format_ident!("{a}"));
                let t = |x: TokenStream| {
                    f.clone().fold(x, |f, a| {
                        quote! {
                            match   #rt::__::core::clone::Clone::clone(&#a){
                                #a => #f
                            }
                        }
                    })
                };
                let v = emit(v, opts);
                let v = t(v);
                t(quote! {
                       #rt::B(#rt::alloc::sync::Arc::new(move|#b|#v))
                })
            }
            GTerm::App(a) => {
                let (a, b) = a.as_ref();
                let a = emit(a, opts);
                match b {
                    GTerm::Var(v) => {
                        let v = format_ident!("{v}");
                        quote! {
                            (#a.0)(&#v)
                        }
                    }
                    _ => quasiquote! {
                        {
                            let _0 = #{emit(b,opts)};
                            (#a.0)(&_0)
                        }
                    },
                }
            }
            GTerm::Mix(_) => todo!(),
        }
    };
    let w = a.frees();
    let f = w.iter().map(|a| format_ident!("{a}"));
    let mut t = quote! {
        #(let #f = #rt::__::core::clone::Clone::clone(&#f));*
    };
    return quote! {
        {#t;
        #rt::l(move||#n)
        }
    };
}