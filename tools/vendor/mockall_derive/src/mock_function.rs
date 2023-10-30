// vim: tw=80
use super::*;

use quote::ToTokens;

/// Convert a trait object reference into a reference to a Boxed trait
///
/// # Returns
///
/// Returns `true` if it was necessary to box the type.
fn dedynify(ty: &mut Type) -> bool {
    if let Type::Reference(ref mut tr) = ty {
        if let Type::TraitObject(ref tto) = tr.elem.as_ref() {
            if let Some(lt) = &tr.lifetime {
                if lt.ident == "static" {
                    // For methods that return 'static references, the user can
                    // usually actually supply one, unlike nonstatic references.
                    // dedynify is unneeded and harmful in such cases.
                    //
                    // But we do need to add parens to prevent parsing errors
                    // when methods like returning add a `+ Send` to the output
                    // type.
                    *tr.elem = parse2(quote!((#tto))).unwrap();
                    return false;
                }
            }

            *tr.elem = parse2(quote!(Box<#tto>)).unwrap();
            return true;
        }
    }
    false
}

/// Convert a special reference type like "&str" into a reference to its owned
/// type like "&String".
fn destrify(ty: &mut Type) {
    if let Type::Reference(ref mut tr) = ty {
        if let Some(lt) = &tr.lifetime {
            if lt.ident == "static" {
                // For methods that return 'static references, the user can
                // usually actually supply one, unlike nonstatic references.
                // destrify is unneeded and harmful in such cases.
                return;
            }
        }

        let path_ty: TypePath = parse2(quote!(Path)).unwrap();
        let pathbuf_ty: Type = parse2(quote!(::std::path::PathBuf)).unwrap();

        let str_ty: TypePath = parse2(quote!(str)).unwrap();
        let string_ty: Type = parse2(quote!(::std::string::String)).unwrap();

        let cstr_ty: TypePath = parse2(quote!(CStr)).unwrap();
        let cstring_ty: Type = parse2(quote!(::std::ffi::CString)).unwrap();

        let osstr_ty: TypePath = parse2(quote!(OsStr)).unwrap();
        let osstring_ty: Type = parse2(quote!(::std::ffi::OsString)).unwrap();

        match tr.elem.as_ref() {
            Type::Path(ref path) if *path == cstr_ty =>
                *tr.elem = cstring_ty,
            Type::Path(ref path) if *path == osstr_ty =>
                *tr.elem = osstring_ty,
            Type::Path(ref path) if *path == path_ty =>
                *tr.elem = pathbuf_ty,
            Type::Path(ref path) if *path == str_ty =>
                *tr.elem = string_ty,
            Type::Slice(ts) => {
                let inner = (*ts.elem).clone();
                let mut segments = Punctuated::new();
                segments.push(format_ident!("std").into());
                segments.push(format_ident!("vec").into());
                let mut v: PathSegment = format_ident!("Vec").into();
                let mut abga_args = Punctuated::new();
                abga_args.push(GenericArgument::Type(inner));
                v.arguments = PathArguments::AngleBracketed(
                    AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: Token![<](Span::call_site()),
                        args: abga_args,
                        gt_token: Token![>](Span::call_site()),
                    }
                );
                segments.push(v);

                *tr.elem = Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: Some(Token![::](Span::call_site())),
                        segments
                    }
                });
            },
            _ => (), // Nothing to do
        };
    }
}

/// Return the owned version of the input.
fn ownify(ty: &Type) -> Type {
    if let Type::Reference(ref tr) = &ty {
        if tr.lifetime.as_ref().map_or(false, |lt| lt.ident == "static")
        {
            // Just a static expectation
            ty.clone()
        } else {
            *tr.elem.clone()
        }
    } else {
        ty.clone()
    }
}

/// Add Send + Sync to a where clause
fn send_syncify(wc: &mut Option<WhereClause>, bounded_ty: Type) {
    let mut bounds = Punctuated::new();
    bounds.push(TypeParamBound::Trait(TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: Path::from(format_ident!("Send"))
    }));
    bounds.push(TypeParamBound::Trait(TraitBound {
        paren_token: None,
        modifier: TraitBoundModifier::None,
        lifetimes: None,
        path: Path::from(format_ident!("Sync"))
    }));
    if wc.is_none() {
        *wc = Some(WhereClause {
            where_token: <Token![where]>::default(),
            predicates: Punctuated::new()
        });
    }
    wc.as_mut().unwrap()
    .predicates.push(
        WherePredicate::Type(
            PredicateType {
                lifetimes: None,
                bounded_ty,
                colon_token: Default::default(),
                bounds
            }
        )
    );
}

/// Build a MockFunction.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Builder<'a> {
    attrs: &'a [Attribute],
    call_levels: Option<usize>,
    levels: usize,
    parent: Option<&'a Ident>,
    sig: &'a Signature,
    struct_: Option<&'a Ident>,
    struct_generics: Option<&'a Generics>,
    trait_: Option<&'a Ident>,
    vis: &'a Visibility
}

impl<'a> Builder<'a> {
    pub fn attrs(&mut self, attrs: &'a[Attribute]) -> &mut Self {
        self.attrs = attrs;
        self
    }

    pub fn build(self) -> MockFunction {
        let mut argnames = Vec::new();
        let mut argty = Vec::new();
        let mut is_static = true;
        let mut predexprs = Vec::new();
        let mut predty = Vec::new();
        let mut refpredty = Vec::new();

        let (mut declosured_generics, declosured_inputs, call_exprs) =
            declosurefy(&self.sig.generics, &self.sig.inputs);

        for fa in declosured_inputs.iter() {
            if let FnArg::Typed(pt) = fa {
                let argname = (*pt.pat).clone();
                if pat_is_self(&argname) {
                    // A weird receiver like `Box<Self>`
                    is_static = false;
                    continue;
                }
                let aty = supersuperfy(&pt.ty, self.levels);
                if let Type::Reference(ref tr) = aty {
                    predexprs.push(quote!(#argname));
                    predty.push((*tr.elem).clone());
                    let tr2 = Type::Reference(TypeReference {
                        and_token: tr.and_token,
                        lifetime: None,
                        mutability: None,
                        elem: tr.elem.clone()
                    });
                    refpredty.push(tr2);
                } else {
                    predexprs.push(quote!(&#argname));
                    predty.push(aty.clone());
                    let tr = TypeReference {
                        and_token: Token![&](Span::call_site()),
                        lifetime: None,
                        mutability: None,
                        elem: Box::new(aty.clone())
                    };
                    refpredty.push(Type::Reference(tr));
                };
                argnames.push(argname);
                argty.push(aty.clone());
            } else {
                is_static = false;
            }
        }
        let (output, boxed) = match self.sig.output {
            ReturnType::Default => (
                Type::Tuple(TypeTuple {
                    paren_token: token::Paren::default(),
                    elems: Punctuated::new(),
                }),
                false,
            ),
            ReturnType::Type(_, ref ty) => {
                let mut output_ty = supersuperfy(ty, self.levels);
                destrify(&mut output_ty);
                let boxed = dedynify(&mut output_ty);
                (output_ty, boxed)
            }
        };
        supersuperfy_generics(&mut declosured_generics, self.levels);
        let owned_output = ownify(&output);
        let mut return_ref = false;
        let mut return_refmut = false;
        if let Type::Reference(ref tr) = &output {
            if tr.lifetime.as_ref().map_or(true, |lt| lt.ident != "static")
            {
                if tr.mutability.is_none() {
                    return_ref = true;
                } else {
                    return_refmut = true;
                }
            }
        };
        if is_static && (return_ref || return_refmut) {
            compile_error(self.sig.span(),
                "Mockall cannot mock static methods that return non-'static references.  It's unclear what the return value's lifetime should be.");
        }
        let struct_generics = self.struct_generics.cloned()
            .unwrap_or_default();
        let (type_generics, salifetimes, srlifetimes) = split_lifetimes(
            struct_generics.clone(),
            &declosured_inputs,
            &ReturnType::Type(<Token![->]>::default(),
                              Box::new(owned_output.clone()))
        );
        let srltg = lifetimes_to_generics(&srlifetimes);
        let (call_generics, malifetimes, mrlifetimes) = split_lifetimes(
            declosured_generics,
            &declosured_inputs,
            &ReturnType::Type(<Token![->]>::default(),
                              Box::new(owned_output.clone()))
        );
        let mrltg = lifetimes_to_generics(&mrlifetimes);
        let cgenerics = merge_generics(&type_generics, &call_generics);
        let egenerics = merge_generics(
            &merge_generics(&cgenerics, &srltg),
            &mrltg);
        let alifetimes = salifetimes.into_iter()
            .collect::<HashSet<LifetimeDef>>()
            .union(&malifetimes.into_iter().collect::<HashSet<_>>())
            .cloned()
            .collect();

        let fn_params = egenerics.type_params()
            .map(|tp| tp.ident.clone())
            .collect();
        let call_levels = self.call_levels.unwrap_or(self.levels);

        MockFunction {
            alifetimes,
            argnames,
            argty,
            attrs: self.attrs.to_vec(),
            call_exprs,
            call_generics,
            call_vis: expectation_visibility(self.vis, call_levels),
            egenerics,
            cgenerics,
            fn_params,
            is_static,
            mod_ident: self.parent.unwrap_or(&Ident::new("FIXME", Span::call_site())).clone(),
            output,
            owned_output,
            boxed,
            predexprs,
            predty,
            refpredty,
            return_ref,
            return_refmut,
            sig: self.sig.clone(),
            struct_: self.struct_.cloned(),
            struct_generics,
            trait_: self.trait_.cloned(),
            type_generics,
            privmod_vis: expectation_visibility(self.vis, self.levels)
        }
    }

    /// How many levels of modules beneath the original function this one is
    /// nested.
    pub fn call_levels(&mut self, levels: usize) -> &mut Self {
        self.call_levels = Some(levels);
        self
    }

    /// How many levels of modules beneath the original function this one's
    /// private module is nested.
    pub fn levels(&mut self, levels: usize) -> &mut Self {
        self.levels = levels;
        self
    }

    /// # Arguments
    ///
    /// * sig:      The signature of the mockable function
    /// * v:        The visibility of the mockable function
    pub fn new(sig: &'a Signature, vis: &'a Visibility) -> Self {
        Builder {
            attrs: &[],
            levels: 0,
            call_levels: None,
            parent: None,
            sig,
            struct_: None,
            struct_generics: None,
            trait_: None,
            vis
        }
    }

    /// Supply the name of the parent module
    pub fn parent(&mut self, ident: &'a Ident) -> &mut Self {
        self.parent = Some(ident);
        self
    }

    /// Supply the name of the parent struct, if any
    pub fn struct_(&mut self, ident: &'a Ident) -> &mut Self {
        self.struct_= Some(ident);
        self
    }

    /// Supply the Generics of the parent struct, if any
    pub fn struct_generics(&mut self, generics: &'a Generics) -> &mut Self {
        self.struct_generics = Some(generics);
        self
    }

    /// Supply the name of the method's trait, if any
    pub fn trait_(&mut self, ident: &'a Ident) -> &mut Self {
        self.trait_ = Some(ident);
        self
    }
}

#[derive(Clone)]
pub(crate) struct MockFunction {
    /// Lifetimes of the mocked method that relate to the arguments but not the
    /// return value
    alifetimes: Punctuated<LifetimeDef, token::Comma>,
    /// Names of the method arguments
    argnames: Vec<Pat>,
    /// Types of the method arguments
    argty: Vec<Type>,
    /// any attributes on the original function, like #[inline]
    pub attrs: Vec<Attribute>,
    /// Expressions that should be used for Expectation::call's arguments
    call_exprs: Vec<TokenStream>,
    /// Generics used for the expectation call
    call_generics: Generics,
    /// Visibility of the mock function itself
    call_vis: Visibility,
    /// Generics of the Expectation object
    egenerics: Generics,
    /// Generics of the Common object
    cgenerics: Generics,
    /// The mock function's generic types as a list of types
    fn_params: Vec<Ident>,
    /// Is this for a static method or free function?
    is_static: bool,
    /// name of the function's parent module
    mod_ident: Ident,
    /// Output type of the Method, supersuperfied.
    output: Type,
    /// Owned version of the output type of the Method, supersuperfied.
    ///
    /// If the real output type is a non-'static reference, then it will differ
    /// from this field.
    owned_output: Type,
    /// True if the `owned_type` is boxed by `Box<>`.
    boxed: bool,
    /// Expressions that create the predicate arguments from the call arguments
    predexprs: Vec<TokenStream>,
    /// Types used for Predicates.  Will be almost the same as args, but every
    /// type will be a non-reference type.
    predty: Vec<Type>,
    /// Does the function return a non-'static reference?
    return_ref: bool,
    /// Does the function return a mutable reference?
    return_refmut: bool,
    /// References to every type in `predty`.
    refpredty: Vec<Type>,
    /// The signature of the mockable function
    sig: Signature,
    /// Name of the parent structure, if any
    struct_: Option<Ident>,
    /// Generics of the parent structure
    struct_generics: Generics,
    /// Name of this method's trait, if the method comes from a trait
    trait_: Option<Ident>,
    /// Type generics of the mock structure
    type_generics: Generics,
    /// Visibility of the expectation and its methods
    privmod_vis: Visibility
}

impl MockFunction {
    /// Return the mock function itself
    ///
    /// # Arguments
    ///
    /// * `modname`:    Name of the parent struct's private module
    // Supplying modname is an unfortunately hack.  Ideally MockFunction
    // wouldn't need to know that.
    pub fn call(&self, modname: Option<&Ident>) -> impl ToTokens {
        let attrs = AttrFormatter::new(&self.attrs).format();
        let call_exprs = &self.call_exprs;
        let (_, tg, _) = if self.is_method_generic() || self.is_static() {
            &self.egenerics
        } else {
            &self.call_generics
        }.split_for_impl();
        let tbf = tg.as_turbofish();
        let name = self.name();
        let desc = self.desc();
        let no_match_msg = quote!(std::format!(
            "{}: No matching expectation found", #desc));
        let sig = &self.sig;
        let (vis, dead_code) = if self.trait_.is_some() {
            (&Visibility::Inherited, quote!())
        } else {
            let dead_code = if let Visibility::Inherited = self.call_vis {
                // This private method may be a helper only used by the struct's
                // other methods, which we are mocking.  If so, the mock method
                // will be dead code.  But we can't simply eliminate it, because
                // it might also be used by other code in the same module.
                quote!(#[allow(dead_code)])
            } else {
                quote!()
            };
            (&self.call_vis, dead_code)
        };
        let substruct_obj = if let Some(trait_) = &self.trait_ {
            let ident = format_ident!("{}_expectations", trait_);
            quote!(#ident.)
        } else {
            quote!()
        };
        let call = if self.return_refmut {
            Ident::new("call_mut", Span::call_site())
        } else {
            Ident::new("call", Span::call_site())
        };
        let mut deref = quote!();
        if self.boxed {
            if self.return_ref {
                deref = quote!(&**);
            } else if self.return_refmut {
                deref = quote!(&mut **);
            }
        }
        if self.is_static {
            let outer_mod_path = self.outer_mod_path(modname);
            quote!(
                // Don't add a doc string.  The original is included in #attrs
                #(#attrs)*
                #dead_code
                #vis #sig {
                    let no_match_msg = #no_match_msg;
                    #deref {
                        let __mockall_guard = #outer_mod_path::EXPECTATIONS
                            .lock().unwrap();
                        /*
                         * TODO: catch panics, then gracefully release the mutex
                         * so it won't be poisoned.  This requires bounding any
                         * generic parameters with UnwindSafe
                         */
                        /* std::panic::catch_unwind(|| */
                        __mockall_guard.#call#tbf(#(#call_exprs,)*)
                        /*)*/
                    }.expect(&no_match_msg)
                }
            )
        } else {
            quote!(
                // Don't add a doc string.  The original is included in #attrs
                #(#attrs)*
                #dead_code
                #vis #sig {
                    let no_match_msg = #no_match_msg;
                    #deref self.#substruct_obj #name.#call#tbf(#(#call_exprs,)*)
                    .expect(&no_match_msg)
                }

            )
        }
    }

    /// Return this method's contribution to its parent's checkpoint method
    pub fn checkpoint(&self) -> impl ToTokens {
        let attrs = AttrFormatter::new(&self.attrs)
            .doc(false)
            .format();
        let inner_mod_ident = self.inner_mod_ident();
        if self.is_static {
            quote!(
                #(#attrs)*
                {
                    let __mockall_timeses = #inner_mod_ident::EXPECTATIONS.lock()
                        .unwrap()
                        .checkpoint()
                        .collect::<Vec<_>>();
                }
            )
        } else {
            let name = &self.name();
            quote!(#(#attrs)* { self.#name.checkpoint(); })
        }
    }

    /// Return a function that creates a Context object for this function
    ///
    /// # Arguments
    ///
    /// * `modname`:    Name of the parent struct's private module
    // Supplying modname is an unfortunately hack.  Ideally MockFunction
    // wouldn't need to know that.
    pub fn context_fn(&self, modname: Option<&Ident>) -> impl ToTokens {
        let attrs = AttrFormatter::new(&self.attrs)
            .doc(false)
            .format();
        let context_docstr = format!("Create a [`Context`]({}{}/struct.Context.html) for mocking the `{}` method",
            modname.map(|m| format!("{}/", m)).unwrap_or_default(),
            self.inner_mod_ident(),
            self.name());
        let context_ident = format_ident!("{}_context", self.name());
        let (_, tg, _) = self.type_generics.split_for_impl();
        let outer_mod_path = self.outer_mod_path(modname);
        let v = &self.call_vis;
        quote!(
            #(#attrs)*
            #[doc = #context_docstr]
            #v fn #context_ident() -> #outer_mod_path::Context #tg
            {
                #outer_mod_path::Context::default()
            }
        )
    }

    /// Generate a code fragment that will print a description of the invocation
    fn desc(&self) -> impl ToTokens {
        let argnames = &self.argnames;
        let name = if let Some(s) = &self.struct_ {
            format!("{}::{}", s, self.sig.ident)
        } else {
            format!("{}::{}", self.mod_ident, self.sig.ident)
        };
        let fields = vec!["{:?}"; argnames.len()].join(", ");
        let fstr = format!("{}({})", name, fields);
        quote!(std::format!(#fstr, #(::mockall::MaybeDebugger(&#argnames)),*))
    }

    /// Generate code for the expect_ method
    ///
    /// # Arguments
    ///
    /// * `modname`:    Name of the parent struct's private module
    /// * `self_args`:  If supplied, these are the
    ///                 AngleBracketedGenericArguments of the self type of the
    ///                 trait impl.  e.g. The `T` in `impl Foo for Bar<T>`.
    // Supplying modname is an unfortunately hack.  Ideally MockFunction
    // wouldn't need to know that.
    pub fn expect(&self, modname: &Ident, self_args: Option<&PathArguments>)
        -> impl ToTokens
    {
        let attrs = AttrFormatter::new(&self.attrs)
            .doc(false)
            .format();
        let name = self.name();
        let expect_ident = format_ident!("expect_{}", &name);
        let expectation_obj = self.expectation_obj(self_args);
        let funcname = &self.sig.ident;
        let (_, tg, _) = if self.is_method_generic() {
            &self.egenerics
        } else {
            &self.call_generics
        }.split_for_impl();
        let (ig, _, wc) = self.call_generics.split_for_impl();
        let mut wc = wc.cloned();
        if self.is_method_generic() && (self.return_ref || self.return_refmut) {
            // Add Senc + Sync, required for downcast, since Expectation
            // stores an Option<#owned_output>
            send_syncify(&mut wc, self.owned_output.clone());
        }
        let tbf = tg.as_turbofish();
        let vis = &self.call_vis;

        #[cfg(not(feature = "nightly_derive"))]
        let must_use = quote!(#[must_use =
                "Must set return value when not using the \"nightly\" feature"
            ]);
        #[cfg(feature = "nightly_derive")]
        let must_use = quote!();

        let substruct_obj = if let Some(trait_) = &self.trait_ {
            let ident = format_ident!("{}_expectations", trait_);
            quote!(#ident.)
        } else {
            quote!()
        };
        let docstr = format!("Create an [`Expectation`]({}/{}/struct.Expectation.html) for mocking the `{}` method",
            modname, self.inner_mod_ident(), funcname);
        quote!(
            #must_use
            #[doc = #docstr]
            #(#attrs)*
            #vis fn #expect_ident #ig(&mut self)
               -> &mut #modname::#expectation_obj
               #wc
            {
                self.#substruct_obj #name.expect#tbf()
            }
        )
    }

    /// Return the name of this function's expecation object
    fn expectation_obj(&self, self_args: Option<&PathArguments>)
        -> impl ToTokens
    {
        let inner_mod_ident = self.inner_mod_ident();
        if let Some(PathArguments::AngleBracketed(abga)) = self_args {
            // staticize any lifetimes that might be present in the Expectation
            // object but not in the self args.  These come from the method's
            // return type.
            let mut abga2 = abga.clone();
            for _ in self.egenerics.lifetimes() {
                let lt = Lifetime::new("'static", Span::call_site());
                let la = GenericArgument::Lifetime(lt);
                abga2.args.insert(0, la);
            }
            assert!(!self.is_method_generic(),
                "specific impls with generic methods are TODO");
            quote!(#inner_mod_ident::Expectation #abga2)
        } else {
            // staticize any lifetimes.  This is necessary for methods that
            // return non-static types, because the Expectation itself must be
            // 'static.
            let segenerics = staticize(&self.egenerics);
            let (_, tg, _) = segenerics.split_for_impl();
            quote!(#inner_mod_ident::Expectation #tg)
        }
    }

    /// Return the name of this function's expecations object
    pub fn expectations_obj(&self) -> impl ToTokens {
        let inner_mod_ident = self.inner_mod_ident();
        if self.is_method_generic() {
            quote!(#inner_mod_ident::GenericExpectations)
        } else {
            quote!(#inner_mod_ident::Expectations)
        }
    }

    pub fn field_definition(&self, modname: Option<&Ident>) -> TokenStream {
        let name = self.name();
        let attrs = AttrFormatter::new(&self.attrs)
            .doc(false)
            .format();
        let expectations_obj = &self.expectations_obj();
        if self.is_method_generic() {
            quote!(#(#attrs)* #name: #modname::#expectations_obj)
        } else {
            // staticize any lifetimes.  This is necessary for methods that
            // return non-static types, because the Expectation itself must be
            // 'static.
            let segenerics = staticize(&self.egenerics);
            let (_, tg, _) = segenerics.split_for_impl();
            quote!(#(#attrs)* #name: #modname::#expectations_obj #tg)
        }
    }

    /// Human-readable name of the mock function
    fn funcname(&self) -> String {
        if let Some(si) = &self.struct_ {
            format!("{}::{}", si, self.name())
        } else {
            format!("{}", self.name())
        }
    }

    fn hrtb(&self) -> Option<BoundLifetimes> {
        if self.alifetimes.is_empty() {
            None
        } else {
            Some(BoundLifetimes {
                lifetimes: self.alifetimes.clone(),
                lt_token: <Token![<]>::default(),
                gt_token: <Token![>]>::default(),
                .. Default::default()
            })
        }
    }

    fn is_expectation_generic(&self) -> bool {
        self.egenerics.params.iter().any(|p| {
            matches!(p, GenericParam::Type(_))
        }) || self.egenerics.where_clause.is_some()
    }

    /// Is the mock method generic (as opposed to a non-generic method of a
    /// generic mock struct)?
    pub fn is_method_generic(&self) -> bool {
        self.call_generics.params.iter().any(|p| {
            matches!(p, GenericParam::Type(_))
        }) || self.call_generics.where_clause.is_some()
    }

    fn outer_mod_path(&self, modname: Option<&Ident>) -> Path {
        let mut path = if let Some(m) = modname {
            Path::from(PathSegment::from(m.clone()))
        } else {
            Path { leading_colon: None, segments: Punctuated::new() }
        };
        path.segments.push(PathSegment::from(self.inner_mod_ident()));
        path
    }

    fn inner_mod_ident(&self) -> Ident {
        format_ident!("__{}", &self.name())
    }

    pub fn is_static(&self) -> bool {
        self.is_static
    }

    pub fn name(&self) -> &Ident {
        &self.sig.ident
    }

    /// Generate code for this function's private module
    pub fn priv_module(&self) -> impl ToTokens {
        let attrs = AttrFormatter::new(&self.attrs)
            .doc(false)
            .format();
        let common = &Common{f: self};
        let context = &Context{f: self};
        let expectation: Box<dyn ToTokens> = if self.return_ref {
            Box::new(RefExpectation{f: self})
        } else if self.return_refmut {
            Box::new(RefMutExpectation{f: self})
        } else {
            Box::new(StaticExpectation{f: self})
        };
        let expectations: Box<dyn ToTokens> = if self.return_ref {
            Box::new(RefExpectations{f: self})
        } else if self.return_refmut {
            Box::new(RefMutExpectations{f: self})
        } else {
            Box::new(StaticExpectations{f: self})
        };
        let generic_expectations = GenericExpectations{f: self};
        let guard: Box<dyn ToTokens> = if self.is_expectation_generic() {
            Box::new(GenericExpectationGuard{f: self})
        } else {
            Box::new(ConcreteExpectationGuard{f: self})
        };
        let matcher = &Matcher{f: self};
        let std_mutexguard = if self.is_static {
            quote!(use ::std::sync::MutexGuard;)
        } else {
            quote!()
        };
        let inner_mod_ident = self.inner_mod_ident();
        let rfunc: Box<dyn ToTokens> = if self.return_ref {
            Box::new(RefRfunc{f: self})
        } else if self.return_refmut {
            Box::new(RefMutRfunc{f: self})
        } else {
            Box::new(StaticRfunc{f: self})
        };
        quote!(
            #(#attrs)*
            #[allow(missing_docs)]
            pub mod #inner_mod_ident {
                use super::*;
                use ::mockall::CaseTreeExt;
                #std_mutexguard
                use ::std::{
                    boxed::Box,
                    mem,
                    ops::{DerefMut, Range},
                    sync::Mutex,
                    vec::Vec,
                };
                #rfunc
                #matcher
                #common
                #expectation
                #expectations
                #generic_expectations
                #guard
                #context
            }
        )
    }
}

/// Holds parts of the expectation that are common for all output types
struct Common<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for Common<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let argnames = &self.f.argnames;
        let predty = &self.f.predty;
        let hrtb = self.f.hrtb();
        let funcname = self.f.funcname();
        let (ig, tg, wc) = self.f.cgenerics.split_for_impl();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let refpredty = &self.f.refpredty;
        let with_generics_idents = (0..self.f.predty.len())
            .map(|i| format_ident!("MockallMatcher{}", i))
            .collect::<Vec<_>>();
        let with_generics = with_generics_idents.iter()
            .zip(self.f.predty.iter())
            .map(|(id, mt)|
                quote!(#id: #hrtb ::mockall::Predicate<#mt> + Send + 'static, )
            ).collect::<TokenStream>();
        let with_args = self.f.argnames.iter()
            .zip(with_generics_idents.iter())
            .map(|(argname, id)| quote!(#argname: #id, ))
            .collect::<TokenStream>();
        let boxed_withargs = argnames.iter()
            .map(|aa| quote!(Box::new(#aa), ))
            .collect::<TokenStream>();
        quote!(
            /// Holds the stuff that is independent of the output type
            struct Common #ig #wc {
                matcher: Mutex<Matcher #tg>,
                seq_handle: Option<::mockall::SeqHandle>,
                times: ::mockall::Times
            }

            impl #ig std::default::Default for Common #tg #wc
            {
                fn default() -> Self {
                    Common {
                        matcher: Mutex::new(Matcher::default()),
                        seq_handle: None,
                        times: ::mockall::Times::default()
                    }
                }
            }

            impl #ig Common #tg #wc {
                fn call(&self, desc: &str) {
                    self.times.call()
                        .unwrap_or_else(|m| {
                            let desc = std::format!(
                                "{}", self.matcher.lock().unwrap());
                            panic!("{}: Expectation({}) {}", #funcname, desc,
                                m);
                        });
                    self.verify_sequence(desc);
                    if ::mockall::ExpectedCalls::TooFew != self.times.is_satisfied() {
                        self.satisfy_sequence()
                    }
                }

                fn in_sequence(&mut self, __mockall_seq: &mut ::mockall::Sequence)
                    -> &mut Self
                {
                    assert!(self.times.is_exact(),
                        "Only Expectations with an exact call count have sequences");
                    self.seq_handle = Some(__mockall_seq.next_handle());
                    self
                }

                fn is_done(&self) -> bool {
                    self.times.is_done()
                }

                #[allow(clippy::ptr_arg)]
                fn matches #lg (&self, #( #argnames: &#predty, )*) -> bool {
                    self.matcher.lock().unwrap().matches(#(#argnames, )*)
                }

                /// Forbid this expectation from ever being called.
                fn never(&mut self) {
                    self.times.never();
                }

                fn satisfy_sequence(&self) {
                    if let Some(__mockall_handle) = &self.seq_handle {
                        __mockall_handle.satisfy()
                    }
                }

                /// Expect this expectation to be called any number of times
                /// contained with the given range.
                fn times<MockallR>(&mut self, __mockall_r: MockallR)
                    where MockallR: Into<::mockall::TimesRange>
                {
                    self.times.times(__mockall_r)
                }

                fn with<#with_generics>(&mut self, #with_args)
                {
                    let mut __mockall_guard = self.matcher.lock().unwrap();
                    *__mockall_guard.deref_mut() =
                        Matcher::Pred(Box::new((#boxed_withargs)));
                }

                fn withf<MockallF>(&mut self, __mockall_f: MockallF)
                    where MockallF: #hrtb Fn(#( #refpredty, )*)
                                    -> bool + Send + 'static
                {
                    let mut __mockall_guard = self.matcher.lock().unwrap();
                    *__mockall_guard.deref_mut() =
                         Matcher::Func(Box::new(__mockall_f));
                }

                fn withf_st<MockallF>(&mut self, __mockall_f: MockallF)
                    where MockallF: #hrtb Fn(#( #refpredty, )*)
                                    -> bool + 'static
                {
                    let mut __mockall_guard = self.matcher.lock().unwrap();
                    *__mockall_guard.deref_mut() =
                         Matcher::FuncSt(
                             ::mockall::Fragile::new(Box::new(__mockall_f))
                        );
                }

                fn verify_sequence(&self, desc: &str) {
                    if let Some(__mockall_handle) = &self.seq_handle {
                        __mockall_handle.verify(desc)
                    }
                }
            }

            impl #ig Drop for Common #tg #wc {
                fn drop(&mut self) {
                    if !::std::thread::panicking() {
                        let desc = std::format!(
                            "{}", self.matcher.lock().unwrap());
                        match self.times.is_satisfied() {
                            ::mockall::ExpectedCalls::TooFew => {
                                panic!("{}: Expectation({}) called {} time(s) which is fewer than expected {}",
                                    #funcname,
                                    desc,
                                    self.times.count(),
                                    self.times.minimum());
                            },
                            ::mockall::ExpectedCalls::TooMany => {
                                panic!("{}: Expectation({}) called {} time(s) which is more than expected {}",
                                    #funcname,
                                    desc,
                                    self.times.count(),
                                    self.times.maximum());
                            },
                            _ => ()
                        }
                    }
                }
            }
        ).to_tokens(tokens);
    }
}

/// Generates methods that are common for all Expectation types
struct CommonExpectationMethods<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for CommonExpectationMethods<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let argnames = &self.f.argnames;
        let hrtb = self.f.hrtb();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let predty = &self.f.predty;
        let with_generics_idents = (0..self.f.predty.len())
            .map(|i| format_ident!("MockallMatcher{}", i))
            .collect::<Vec<_>>();
        let with_generics = with_generics_idents.iter()
            .zip(self.f.predty.iter())
            .map(|(id, mt)|
                quote!(#id: #hrtb ::mockall::Predicate<#mt> + Send + 'static, )
            ).collect::<TokenStream>();
        let with_args = self.f.argnames.iter()
            .zip(with_generics_idents.iter())
            .map(|(argname, id)| quote!(#argname: #id, ))
            .collect::<TokenStream>();
        let v = &self.f.privmod_vis;
        quote!(
            /// Add this expectation to a
            /// [`Sequence`](../../../mockall/struct.Sequence.html).
            #v fn in_sequence(&mut self, __mockall_seq: &mut ::mockall::Sequence)
                -> &mut Self
            {
                self.common.in_sequence(__mockall_seq);
                self
            }

            fn is_done(&self) -> bool {
                self.common.is_done()
            }

            /// Validate this expectation's matcher.
            #[allow(clippy::ptr_arg)]
            fn matches #lg (&self, #(#argnames: &#predty, )*) -> bool {
                self.common.matches(#(#argnames, )*)
            }

            /// Forbid this expectation from ever being called.
            #v fn never(&mut self) -> &mut Self {
                self.common.never();
                self
            }

            /// Create a new, default, [`Expectation`](struct.Expectation.html)
            #v fn new() -> Self {
                Self::default()
            }

            /// Expect this expectation to be called exactly once.  Shortcut for
            /// [`times(1)`](#method.times).
            #v fn once(&mut self) -> &mut Self {
                self.times(1)
            }

            /// Restrict the number of times that that this method may be called.
            ///
            /// The argument may be:
            /// * A fixed number: `.times(4)`
            /// * Various types of range:
            ///   - `.times(5..10)`
            ///   - `.times(..10)`
            ///   - `.times(5..)`
            ///   - `.times(5..=10)`
            ///   - `.times(..=10)`
            /// * The wildcard: `.times(..)`
            #v fn times<MockallR>(&mut self, __mockall_r: MockallR) -> &mut Self
                where MockallR: Into<::mockall::TimesRange>
            {
                self.common.times(__mockall_r);
                self
            }

            /// Set matching crieteria for this Expectation.
            ///
            /// The matching predicate can be anything implemening the
            /// [`Predicate`](../../../mockall/trait.Predicate.html) trait.  Only
            /// one matcher can be set per `Expectation` at a time.
            #v fn with<#with_generics>(&mut self, #with_args) -> &mut Self
            {
                self.common.with(#(#argnames, )*);
                self
            }

            /// Set a matching function for this Expectation.
            ///
            /// This is equivalent to calling [`with`](#method.with) with a
            /// function argument, like `with(predicate::function(f))`.
            #v fn withf<MockallF>(&mut self, __mockall_f: MockallF) -> &mut Self
                where MockallF: #hrtb Fn(#(&#predty, )*)
                                -> bool + Send + 'static
            {
                self.common.withf(__mockall_f);
                self
            }

            /// Single-threaded version of [`withf`](#method.withf).
            /// Can be used when the argument type isn't `Send`.
            #v fn withf_st<MockallF>(&mut self, __mockall_f: MockallF) -> &mut Self
                where MockallF: #hrtb Fn(#(&#predty, )*)
                                -> bool + 'static
            {
                self.common.withf_st(__mockall_f);
                self
            }
        ).to_tokens(tokens);
    }
}

/// Holds the moethods of the Expectations object that are common for all
/// Expectation types
struct CommonExpectationsMethods<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for CommonExpectationsMethods<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (ig, tg, wc) = self.f.egenerics.split_for_impl();
        let v = &self.f.privmod_vis;
        quote!(
            /// A collection of [`Expectation`](struct.Expectations.html)
            /// objects.  Users will rarely if ever use this struct directly.
            #[doc(hidden)]
            #v struct Expectations #ig ( Vec<Expectation #tg>) #wc;

            impl #ig Expectations #tg #wc {
                /// Verify that all current expectations are satisfied and clear
                /// them.
                #v fn checkpoint(&mut self) -> std::vec::Drain<Expectation #tg>
                {
                    self.0.drain(..)
                }

                /// Create a new expectation for this method.
                #v fn expect(&mut self) -> &mut Expectation #tg
                {
                    self.0.push(Expectation::default());
                    let __mockall_l = self.0.len();
                    &mut self.0[__mockall_l - 1]
                }

                #v fn new() -> Self {
                    Self::default()
                }
            }
            impl #ig Default for Expectations #tg #wc
            {
                fn default() -> Self {
                    Expectations(Vec::new())
                }
            }
        ).to_tokens(tokens);
    }
}

/// The ExpectationGuard structure for static methods with no generic types
struct ExpectationGuardCommonMethods<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for ExpectationGuardCommonMethods<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.f.is_static {
            return;
        }

        let argnames = &self.f.argnames;
        let argty = &self.f.argty;
        let (_, tg, _) = self.f.egenerics.split_for_impl();
        let keyid = gen_keyid(&self.f.egenerics);
        let expectations = if self.f.is_expectation_generic() {
            quote!(self.guard
                   .store
                   .get_mut(&::mockall::Key::new::#keyid())
                   .unwrap()
                   .downcast_mut::<Expectations #tg>()
                   .unwrap())
        } else {
            quote!(self.guard)
        };
        let hrtb = self.f.hrtb();
        let output = &self.f.output;
        let predty = &self.f.predty;
        let with_generics_idents = (0..self.f.predty.len())
            .map(|i| format_ident!("MockallMatcher{}", i))
            .collect::<Vec<_>>();
        let with_generics = with_generics_idents.iter()
            .zip(self.f.predty.iter())
            .map(|(id, mt)|
                quote!(#id: #hrtb ::mockall::Predicate<#mt> + Send + 'static, )
            ).collect::<TokenStream>();
        let with_args = self.f.argnames.iter()
            .zip(with_generics_idents.iter())
            .map(|(argname, id)| quote!(#argname: #id, ))
            .collect::<TokenStream>();
        let v = &self.f.privmod_vis;
        quote!(
            /// Just like
            /// [`Expectation::in_sequence`](struct.Expectation.html#method.in_sequence)
            #v fn in_sequence(&mut self,
                __mockall_seq: &mut ::mockall::Sequence)
                -> &mut Expectation #tg
            {
                #expectations.0[self.i].in_sequence(__mockall_seq)
            }

            /// Just like
            /// [`Expectation::never`](struct.Expectation.html#method.never)
            #v fn never(&mut self) -> &mut Expectation #tg {
                #expectations.0[self.i].never()
            }

            /// Just like
            /// [`Expectation::once`](struct.Expectation.html#method.once)
            #v fn once(&mut self) -> &mut Expectation #tg {
                #expectations.0[self.i].once()
            }

            /// Just like
            /// [`Expectation::return_const`](struct.Expectation.html#method.return_const)
            #v fn return_const<MockallOutput>
            (&mut self, __mockall_c: MockallOutput)
                -> &mut Expectation #tg
                where MockallOutput: Clone + Into<#output> + Send + 'static
            {
                #expectations.0[self.i].return_const(__mockall_c)
            }

            /// Just like
            /// [`Expectation::return_const_st`](struct.Expectation.html#method.return_const_st)
            #v fn return_const_st<MockallOutput>
            (&mut self, __mockall_c: MockallOutput)
                -> &mut Expectation #tg
                where MockallOutput: Clone + Into<#output> + 'static
            {
                #expectations.0[self.i].return_const_st(__mockall_c)
            }

            /// Just like
            /// [`Expectation::returning`](struct.Expectation.html#method.returning)
            #v fn returning<MockallF>(&mut self, __mockall_f: MockallF)
                -> &mut Expectation #tg
                where MockallF: #hrtb FnMut(#(#argty, )*)
                    -> #output + Send + 'static
            {
                #expectations.0[self.i].returning(__mockall_f)
            }

            /// Just like
            /// [`Expectation::return_once`](struct.Expectation.html#method.return_once)
            #v fn return_once<MockallF>(&mut self, __mockall_f: MockallF)
                -> &mut Expectation #tg
                where MockallF: #hrtb FnOnce(#(#argty, )*)
                                -> #output + Send + 'static
            {
                #expectations.0[self.i].return_once(__mockall_f)
            }

            /// Just like
            /// [`Expectation::return_once_st`](struct.Expectation.html#method.return_once_st)
            #v fn return_once_st<MockallF>(&mut self, __mockall_f: MockallF)
                -> &mut Expectation #tg
                where MockallF: #hrtb FnOnce(#(#argty, )*)
                                -> #output + 'static
            {
                #expectations.0[self.i].return_once_st(__mockall_f)
            }


            /// Just like
            /// [`Expectation::returning_st`](struct.Expectation.html#method.returning_st)
            #v fn returning_st<MockallF>(&mut self, __mockall_f: MockallF)
                -> &mut Expectation #tg
                where MockallF: #hrtb FnMut(#(#argty, )*)
                                -> #output + 'static
            {
                #expectations.0[self.i].returning_st(__mockall_f)
            }

            /// Just like
            /// [`Expectation::times`](struct.Expectation.html#method.times)
            #v fn times<MockallR>(&mut self, __mockall_r: MockallR)
                -> &mut Expectation #tg
                where MockallR: Into<::mockall::TimesRange>
            {
                #expectations.0[self.i].times(__mockall_r)
            }

            /// Just like
            /// [`Expectation::with`](struct.Expectation.html#method.with)
            #v fn with<#with_generics> (&mut self, #with_args)
                -> &mut Expectation #tg
            {
                #expectations.0[self.i].with(#(#argnames, )*)
            }

            /// Just like
            /// [`Expectation::withf`](struct.Expectation.html#method.withf)
            #v fn withf<MockallF>(&mut self, __mockall_f: MockallF)
                -> &mut Expectation #tg
                where MockallF: #hrtb Fn(#(&#predty, )*)
                                -> bool + Send + 'static
            {
                #expectations.0[self.i].withf(__mockall_f)
            }

            /// Just like
            /// [`Expectation::withf_st`](struct.Expectation.html#method.withf_st)
            #v fn withf_st<MockallF>(&mut self, __mockall_f: MockallF)
                -> &mut Expectation #tg
                where MockallF: #hrtb Fn(#(&#predty, )*)
                                -> bool + 'static
            {
                #expectations.0[self.i].withf_st(__mockall_f)
            }
        ).to_tokens(tokens);
    }
}

/// The ExpectationGuard structure for static methods with no generic types
struct ConcreteExpectationGuard<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for ConcreteExpectationGuard<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.f.is_static {
            return;
        }

        let common_methods = ExpectationGuardCommonMethods{f: self.f};
        let (_, tg, _) = self.f.egenerics.split_for_impl();
        let ltdef = LifetimeDef::new(
            Lifetime::new("'__mockall_lt", Span::call_site())
        );
        let mut e_generics = self.f.egenerics.clone();
        e_generics.lt_token.get_or_insert(<Token![<]>::default());
        e_generics.params.push(GenericParam::Lifetime(ltdef));
        e_generics.gt_token.get_or_insert(<Token![>]>::default());
        let (e_ig, e_tg, e_wc) = e_generics.split_for_impl();
        let (ei_ig, _, _) = e_generics.split_for_impl();
        let v = &self.f.privmod_vis;
        quote!(
            ::mockall::lazy_static! {
                #[doc(hidden)]
                #v static ref EXPECTATIONS:
                    ::std::sync::Mutex<Expectations #tg> =
                    ::std::sync::Mutex::new(Expectations::new());
            }
            /// Like an [`&Expectation`](struct.Expectation.html) but
            /// protected by a Mutex guard.  Useful for mocking static
            /// methods.  Forwards accesses to an `Expectation` object.
            // We must return the MutexGuard to the caller so he can
            // configure the expectation.  But we can't bundle both the
            // guard and the &Expectation into the same structure; the
            // borrow checker won't let us.  Instead we'll record the
            // expectation's position within the Expectations vector so we
            // can proxy its methods.
            //
            // ExpectationGuard is only defined for expectations that return
            // 'static return types.
            #v struct ExpectationGuard #e_ig #e_wc {
                guard: MutexGuard<'__mockall_lt, Expectations #tg>,
                i: usize
            }

            #[allow(clippy::unused_unit)]
            impl #ei_ig ExpectationGuard #e_tg #e_wc
            {
                // Should only be called from the mockall_derive generated
                // code
                #[doc(hidden)]
                #v fn new(mut __mockall_guard: MutexGuard<'__mockall_lt, Expectations #tg>)
                    -> Self
                {
                    __mockall_guard.expect(); // Drop the &Expectation
                    let __mockall_i = __mockall_guard.0.len() - 1;
                    ExpectationGuard{guard: __mockall_guard, i: __mockall_i}
                }

                #common_methods
            }
        ).to_tokens(tokens);
    }
}

/// The ExpectationGuard structure for static methods with generic types
struct GenericExpectationGuard<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for GenericExpectationGuard<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.f.is_static {
            return;
        }

        let common_methods = ExpectationGuardCommonMethods{f: self.f};
        let (_, tg, _) = self.f.egenerics.split_for_impl();
        let keyid = gen_keyid(&self.f.egenerics);
        let ltdef = LifetimeDef::new(
            Lifetime::new("'__mockall_lt", Span::call_site())
        );
        let mut egenerics = self.f.egenerics.clone();
        egenerics.lt_token.get_or_insert(<Token![<]>::default());
        egenerics.params.push(GenericParam::Lifetime(ltdef));
        egenerics.gt_token.get_or_insert(<Token![>]>::default());
        let (e_ig, e_tg, e_wc) = egenerics.split_for_impl();
        let fn_params = &self.f.fn_params;
        let tbf = tg.as_turbofish();
        let v = &self.f.privmod_vis;
        quote!(
            ::mockall::lazy_static! {
                #v static ref EXPECTATIONS:
                    ::std::sync::Mutex<GenericExpectations> =
                    ::std::sync::Mutex::new(GenericExpectations::new());
            }
            /// Like an [`&Expectation`](struct.Expectation.html) but
            /// protected by a Mutex guard.  Useful for mocking static
            /// methods.  Forwards accesses to an `Expectation` object.
            #v struct ExpectationGuard #e_ig #e_wc{
                guard: MutexGuard<'__mockall_lt, GenericExpectations>,
                i: usize,
                _phantom: ::std::marker::PhantomData<(#(#fn_params,)*)>,
            }

            #[allow(clippy::unused_unit)]
            impl #e_ig ExpectationGuard #e_tg #e_wc
            {
                // Should only be called from the mockall_derive generated
                // code
                #[doc(hidden)]
                #v fn new(mut __mockall_guard: MutexGuard<'__mockall_lt, GenericExpectations>)
                    -> Self
                {
                    let __mockall_ee: &mut Expectations #tg =
                        __mockall_guard.store.entry(
                            ::mockall::Key::new::#keyid()
                        ).or_insert_with(||
                            Box::new(Expectations #tbf ::new()))
                        .downcast_mut()
                        .unwrap();
                    __mockall_ee.expect();    // Drop the &Expectation
                    let __mockall_i = __mockall_ee.0.len() - 1;
                    ExpectationGuard{guard: __mockall_guard, i: __mockall_i,
                        _phantom: ::std::marker::PhantomData}
                }

                #common_methods
            }
        ).to_tokens(tokens);
    }
}

/// Generates Context, which manages the context for expectations of static
/// methods.
struct Context<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for Context<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if !self.f.is_static {
            return;
        }

        let ltdef = LifetimeDef::new(
            Lifetime::new("'__mockall_lt", Span::call_site())
        );
        let mut egenerics = self.f.egenerics.clone();
        egenerics.lt_token.get_or_insert(<Token![<]>::default());
        egenerics.params.push(GenericParam::Lifetime(ltdef));
        egenerics.gt_token.get_or_insert(<Token![>]>::default());
        let (_, e_tg, _) = egenerics.split_for_impl();
        let (ty_ig, ty_tg, ty_wc) = self.f.type_generics.split_for_impl();
        let mut meth_generics = self.f.call_generics.clone();
        let ltdef = LifetimeDef::new(
            Lifetime::new("'__mockall_lt", Span::call_site())
        );
        meth_generics.params.push(GenericParam::Lifetime(ltdef));
        let (meth_ig, _meth_tg, meth_wc) = meth_generics.split_for_impl();
        let ctx_fn_params = self.f.struct_generics.type_params()
            .map(|tp| tp.ident.clone())
            .collect::<Punctuated::<Ident, Token![,]>>();
        let v = &self.f.privmod_vis;

        #[cfg(not(feature = "nightly_derive"))]
        let must_use = quote!(#[must_use =
                "Must set return value when not using the \"nightly\" feature"
            ]);
        #[cfg(feature = "nightly_derive")]
        let must_use = quote!();

        quote!(
            /// Manages the context for expectations of static methods.
            ///
            /// Expectations on this method will be validated and cleared when
            /// the `Context` object drops.  The `Context` object does *not*
            /// provide any form of synchronization, so multiple tests that set
            /// expectations on the same static method must provide their own.
            #[must_use = "Context only serves to create expectations" ]
            #v struct Context #ty_ig #ty_wc {
                // Prevent "unused type parameter" errors
                // Surprisingly, PhantomData<Fn(generics)> is Send even if
                // generics are not, unlike PhantomData<generics>
                _phantom: ::std::marker::PhantomData<
                    Box<dyn Fn(#ctx_fn_params) + Send>
                >
            }
            impl #ty_ig Context #ty_tg #ty_wc {
                /// Verify that all current expectations for this method are
                /// satisfied and clear them.
                #v fn checkpoint(&self) {
                    Self::do_checkpoint()
                }
                #[doc(hidden)]
                #v fn do_checkpoint() {
                    let __mockall_timeses = EXPECTATIONS
                        .lock()
                        .unwrap()
                        .checkpoint()
                        .collect::<Vec<_>>();
                }

                /// Create a new expectation for this method.
                #must_use
                #v fn expect #meth_ig ( &self,) -> ExpectationGuard #e_tg
                    #meth_wc
                {
                    ExpectationGuard::new(EXPECTATIONS.lock().unwrap())
                }
            }
            impl #ty_ig Default for Context #ty_tg #ty_wc {
                fn default() -> Self {
                    Context {_phantom: std::marker::PhantomData}
                }
            }
            impl #ty_ig Drop for Context #ty_tg #ty_wc {
                fn drop(&mut self) {
                    Self::do_checkpoint()
                }
            }
        ).to_tokens(tokens);
    }
}

struct Matcher<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for Matcher<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let (ig, tg, wc) = self.f.cgenerics.split_for_impl();
        let argnames = &self.f.argnames;
        let braces = argnames.iter()
            .fold(String::new(), |mut acc, _argname| {
                if acc.is_empty() {
                    acc.push_str("{}");
                } else {
                    acc.push_str(", {}");
                }
                acc
            });
        let fn_params = &self.f.fn_params;
        let hrtb = self.f.hrtb();
        let indices = (0..argnames.len())
            .map(|i| {
                syn::Index::from(i)
            }).collect::<Vec<_>>();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let pred_matches = argnames.iter().enumerate()
            .map(|(i, argname)| {
                let idx = syn::Index::from(i);
                quote!(__mockall_pred.#idx.eval(#argname),)
            }).collect::<TokenStream>();
        let preds = self.f.predty.iter()
            .map(|t| quote!(Box<dyn #hrtb ::mockall::Predicate<#t> + Send>,))
            .collect::<TokenStream>();
        let predty = &self.f.predty;
        let refpredty = &self.f.refpredty;
        quote!(
            enum Matcher #ig #wc {
                Always,
                Func(Box<dyn #hrtb Fn(#( #refpredty, )*) -> bool + Send>),
                // Version of Matcher::Func for closures that aren't Send
                FuncSt(::mockall::Fragile<Box<dyn #hrtb Fn(#( #refpredty, )*) -> bool>>),
                Pred(Box<(#preds)>),
                // Prevent "unused type parameter" errors
                // Surprisingly, PhantomData<Fn(generics)> is Send even if
                // generics are not, unlike PhantomData<generics>
                _Phantom(Box<dyn Fn(#(#fn_params,)*) + Send>)
            }
            impl #ig Matcher #tg #wc {
                #[allow(clippy::ptr_arg)]
                fn matches #lg (&self, #( #argnames: &#predty, )*) -> bool {
                    match self {
                        Matcher::Always => true,
                        Matcher::Func(__mockall_f) =>
                            __mockall_f(#(#argnames, )*),
                        Matcher::FuncSt(__mockall_f) =>
                            (__mockall_f.get())(#(#argnames, )*),
                        Matcher::Pred(__mockall_pred) =>
                            [#pred_matches]
                            .iter()
                            .all(|__mockall_x| *__mockall_x),
                        _ => unreachable!()
                    }
                }
            }

            impl #ig Default for Matcher #tg #wc {
                #[allow(unused_variables)]
                fn default() -> Self {
                    Matcher::Always
                }
            }

            impl #ig ::std::fmt::Display for Matcher #tg #wc {
                fn fmt(&self, __mockall_fmt: &mut ::std::fmt::Formatter<'_>)
                    -> ::std::fmt::Result
                {
                    match self {
                        Matcher::Always => write!(__mockall_fmt, "<anything>"),
                        Matcher::Func(_) => write!(__mockall_fmt, "<function>"),
                        Matcher::FuncSt(_) => write!(__mockall_fmt, "<single threaded function>"),
                        Matcher::Pred(__mockall_p) => {
                            write!(__mockall_fmt, #braces,
                                #(__mockall_p.#indices,)*)
                        }
                        _ => unreachable!(),
                    }
                }
            }
        ).to_tokens(tokens);
    }
}

struct RefRfunc<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for RefRfunc<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fn_params = &self.f.fn_params;
        let (ig, tg, wc) = self.f.egenerics.split_for_impl();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let owned_output = &self.f.owned_output;

        #[cfg(not(feature = "nightly_derive"))]
        let default_err_msg =
            "Returning default values requires the \"nightly\" feature";
        #[cfg(feature = "nightly_derive")]
        let default_err_msg =
            "Can only return default values for types that impl std::Default";

        quote!(
            enum Rfunc #ig #wc {
                Default(Option<#owned_output>),
                Const(#owned_output),
                // Prevent "unused type parameter" errors Surprisingly,
                // PhantomData<Fn(generics)> is Send even if generics are not,
                // unlike PhantomData<generics>
                _Phantom(Mutex<Box<dyn Fn(#(#fn_params,)*) + Send>>)
            }

            impl #ig  Rfunc #tg #wc {
                fn call #lg (&self)
                    -> std::result::Result<&#owned_output, &'static str>
                {
                    match self {
                        Rfunc::Default(Some(ref __mockall_o)) => {
                            ::std::result::Result::Ok(__mockall_o)
                        },
                        Rfunc::Default(None) => {
                            Err(#default_err_msg)
                        },
                        Rfunc::Const(ref __mockall_o) => {
                            ::std::result::Result::Ok(__mockall_o)
                        },
                        Rfunc::_Phantom(_) => unreachable!()
                    }
                }
            }

            impl #ig std::default::Default for Rfunc #tg #wc
            {
                fn default() -> Self {
                    use ::mockall::ReturnDefault;
                    Rfunc::Default(::mockall::DefaultReturner::<#owned_output>
                                ::maybe_return_default())
                }
            }
        ).to_tokens(tokens);
    }
}

struct RefMutRfunc<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for RefMutRfunc<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let argnames = &self.f.argnames;
        let argty = &self.f.argty;
        let fn_params = &self.f.fn_params;
        let (ig, tg, wc) = self.f.egenerics.split_for_impl();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let owned_output = &self.f.owned_output;
        let output = &self.f.output;

        #[cfg(not(feature = "nightly_derive"))]
        let default_err_msg =
            "Returning default values requires the \"nightly\" feature";
        #[cfg(feature = "nightly_derive")]
        let default_err_msg =
            "Can only return default values for types that impl std::Default";

        quote!(
            #[allow(clippy::unused_unit)]
            enum Rfunc #ig #wc {
                Default(Option<#owned_output>),
                Mut((Box<dyn FnMut(#(#argty, )*) -> #owned_output + Send + Sync>),
                    Option<#owned_output>),
                // Version of Rfunc::Mut for closures that aren't Send
                MutSt((::mockall::Fragile<
                           Box<dyn FnMut(#(#argty, )*) -> #owned_output >>
                       ), Option<#owned_output>
                ),
                Var(#owned_output),
                // Prevent "unused type parameter" errors Surprisingly,
                // PhantomData<Fn(generics)> is Send even if generics are not,
                // unlike PhantomData<generics>
                _Phantom(Mutex<Box<dyn Fn(#(#fn_params,)*) + Send>>)
            }

            impl #ig  Rfunc #tg #wc {
                fn call_mut #lg (&mut self, #(#argnames: #argty, )*)
                    -> std::result::Result<#output, &'static str>
                {
                    match self {
                        Rfunc::Default(Some(ref mut __mockall_o)) => {
                            ::std::result::Result::Ok(__mockall_o)
                        },
                        Rfunc::Default(None) => {
                            Err(#default_err_msg)
                        },
                        Rfunc::Mut(ref mut __mockall_f, ref mut __mockall_o) =>
                        {
                            *__mockall_o = Some(__mockall_f(#(#argnames, )*));
                            if let Some(ref mut __mockall_o2) = __mockall_o {
                                ::std::result::Result::Ok(__mockall_o2)
                            } else {
                                unreachable!()
                            }
                        },
                        Rfunc::MutSt(ref mut __mockall_f, ref mut __mockall_o)=>
                        {
                            *__mockall_o = Some((__mockall_f.get_mut())(
                                    #(#argnames, )*)
                            );
                            if let Some(ref mut __mockall_o2) = __mockall_o {
                                ::std::result::Result::Ok(__mockall_o2)
                            } else {
                                unreachable!()
                            }
                        },
                        Rfunc::Var(ref mut __mockall_o) => {
                            ::std::result::Result::Ok(__mockall_o)
                        },
                        Rfunc::_Phantom(_) => unreachable!()
                    }
                }
            }

            impl #ig std::default::Default for Rfunc #tg #wc
            {
                fn default() -> Self {
                    use ::mockall::ReturnDefault;
                    Rfunc::Default(::mockall::DefaultReturner::<#owned_output>
                                ::maybe_return_default())
                }
            }
        ).to_tokens(tokens);
    }
}

struct StaticRfunc<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for StaticRfunc<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let argnames = &self.f.argnames;
        let argty = &self.f.argty;
        let fn_params = &self.f.fn_params;
        let (ig, tg, wc) = self.f.egenerics.split_for_impl();
        let hrtb = self.f.hrtb();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let output = &self.f.output;
        quote!(
            #[allow(clippy::unused_unit)]
            enum Rfunc #ig #wc {
                Default,
                // Indicates that a `return_once` expectation has already
                // returned
                Expired,
                Mut(Box<dyn #hrtb FnMut(#(#argty, )*) -> #output + Send>),
                // Version of Rfunc::Mut for closures that aren't Send
                MutSt(::mockall::Fragile<
                    Box<dyn #hrtb FnMut(#(#argty, )*) -> #output >>
                ),
                Once(Box<dyn #hrtb FnOnce(#(#argty, )*) -> #output + Send>),
                // Version of Rfunc::Once for closure that aren't Send
                OnceSt(::mockall::Fragile<
                    Box<dyn #hrtb FnOnce(#(#argty, )*) -> #output>>
                ),
                // Prevent "unused type parameter" errors Surprisingly,
                // PhantomData<Fn(generics)> is Send even if generics are not,
                // unlike PhantomData<generics>
                _Phantom(Box<dyn Fn(#(#fn_params,)*) + Send>)
            }

            impl #ig  Rfunc #tg #wc {
                fn call_mut #lg (&mut self, #( #argnames: #argty, )* )
                    -> std::result::Result<#output, &'static str>
                {
                    match self {
                        Rfunc::Default => {
                            use ::mockall::ReturnDefault;
                            ::mockall::DefaultReturner::<#output>
                                ::return_default()
                        },
                        Rfunc::Expired => {
                            Err("called twice, but it returns by move")
                        },
                        Rfunc::Mut(__mockall_f) => {
                            ::std::result::Result::Ok(__mockall_f( #(#argnames, )* ))
                        },
                        Rfunc::MutSt(__mockall_f) => {
                            ::std::result::Result::Ok((__mockall_f.get_mut())(#(#argnames,)*))
                        },
                        Rfunc::Once(_) => {
                            if let Rfunc::Once(mut __mockall_f) =
                                mem::replace(self, Rfunc::Expired) {
                                ::std::result::Result::Ok(__mockall_f( #(#argnames, )* ))
                            } else {
                                unreachable!()
                            }
                        },
                        Rfunc::OnceSt(_) => {
                            if let Rfunc::OnceSt(mut __mockall_f) =
                                mem::replace(self, Rfunc::Expired) {
                                ::std::result::Result::Ok((__mockall_f.into_inner())(#(#argnames,)*))
                            } else {
                                unreachable!()
                            }
                        },
                        Rfunc::_Phantom(_) => unreachable!()
                    }
                }
            }

            impl #ig std::default::Default for Rfunc #tg #wc
            {
                fn default() -> Self {
                    Rfunc::Default
                }
            }
        ).to_tokens(tokens);
    }
}

/// An expectation type for functions that take a &self and return a reference
struct RefExpectation<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for RefExpectation<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let argnames = &self.f.argnames;
        let argty = &self.f.argty;
        let common_methods = CommonExpectationMethods{f: self.f};
        let desc = self.f.desc();
        let funcname = self.f.funcname();
        let (ig, tg, wc) = self.f.egenerics.split_for_impl();

        let (_, common_tg, _) = self.f.cgenerics.split_for_impl();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let output = &self.f.output;
        let owned_output = &self.f.owned_output;
        let v = &self.f.privmod_vis;
        quote!(
            /// Expectation type for methods taking a `&self` argument and
            /// returning immutable references.  This is the type returned by
            /// the `expect_*` methods.
            #v struct Expectation #ig #wc {
                common: Common #common_tg,
                rfunc: Rfunc #tg,
            }

            #[allow(clippy::unused_unit)]
            impl #ig Expectation #tg #wc {
                /// Call this [`Expectation`] as if it were the real method.
                #v fn call #lg (&self, #(#argnames: #argty, )*) -> #output
                {
                    self.common.call(&#desc);
                    self.rfunc.call().unwrap_or_else(|m| {
                        let desc = std::format!(
                            "{}", self.common.matcher.lock().unwrap());
                        panic!("{}: Expectation({}) {}", #funcname, desc,
                            m);
                    })
                }

                /// Return a reference to a constant value from the `Expectation`
                #v fn return_const(&mut self, __mockall_o: #owned_output)
                    -> &mut Self
                {
                    self.rfunc = Rfunc::Const(__mockall_o);
                    self
                }

                #common_methods
            }
            impl #ig Default for Expectation #tg #wc
            {
                fn default() -> Self {
                    Expectation {
                        common: Common::default(),
                        rfunc: Rfunc::default()
                    }
                }
            }
        ).to_tokens(tokens);
    }
}

/// For methods that take &mut self and return a reference
struct RefMutExpectation<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for RefMutExpectation<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let common_methods = CommonExpectationMethods{f: self.f};
        let argnames = &self.f.argnames;
        let argty = &self.f.argty;
        let desc = self.f.desc();
        let funcname = self.f.funcname();
        let (ig, tg, wc) = self.f.egenerics.split_for_impl();
        let (_, common_tg, _) = self.f.cgenerics.split_for_impl();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let owned_output = &self.f.owned_output;
        let v = &self.f.privmod_vis;
        quote!(
            /// Expectation type for methods taking a `&mut self` argument and
            /// returning references.  This is the type returned by the
            /// `expect_*` methods.
            #v struct Expectation #ig #wc {
                common: Common #common_tg,
                rfunc: Rfunc #tg
            }

            #[allow(clippy::unused_unit)]
            impl #ig Expectation #tg #wc {
                /// Simulating calling the real method for this expectation
                #v fn call_mut #lg (&mut self, #(#argnames: #argty, )*)
                    -> &mut #owned_output
                {
                    self.common.call(&#desc);
                    let desc = std::format!(
                        "{}", self.common.matcher.lock().unwrap());
                    self.rfunc.call_mut(#(#argnames, )*).unwrap_or_else(|m| {
                            panic!("{}: Expectation({}) {}", #funcname, desc,
                                   m);
                    })
                }

                /// Convenience method that can be used to supply a return value
                /// for a `Expectation`.  The value will be returned by mutable
                /// reference.
                #v fn return_var(&mut self, __mockall_o: #owned_output) -> &mut Self
                {
                    self.rfunc = Rfunc::Var(__mockall_o);
                    self
                }

                /// Supply a closure that the `Expectation` will use to create its
                /// return value.  The return value will be returned by mutable
                /// reference.
                #v fn returning<MockallF>(&mut self, __mockall_f: MockallF)
                    -> &mut Self
                    where MockallF: FnMut(#(#argty, )*) -> #owned_output + Send + Sync + 'static
                {
                    self.rfunc = Rfunc::Mut(Box::new(__mockall_f), None);
                    self
                }

                /// Single-threaded version of [`returning`](#method.returning).
                /// Can be used when the argument or return type isn't `Send`.
                #v fn returning_st<MockallF>(&mut self, __mockall_f: MockallF)
                    -> &mut Self
                    where MockallF: FnMut(#(#argty, )*) -> #owned_output + 'static
                {
                    self.rfunc = Rfunc::MutSt(
                        ::mockall::Fragile::new(Box::new(__mockall_f)), None);
                    self
                }

                #common_methods
            }
            impl #ig Default for Expectation #tg #wc
            {
                fn default() -> Self {
                    Expectation {
                        common: Common::default(),
                        rfunc: Rfunc::default()
                    }
                }
            }
        ).to_tokens(tokens);
    }
}

/// An expectation type for functions return a `'static` value
struct StaticExpectation<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for StaticExpectation<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let common_methods = CommonExpectationMethods{f: self.f};
        let argnames = &self.f.argnames;
        let argty = &self.f.argty;
        let desc = self.f.desc();
        let hrtb = self.f.hrtb();
        let funcname = self.f.funcname();
        let (ig, tg, wc) = self.f.egenerics.split_for_impl();
        let (_, common_tg, _) = self.f.cgenerics.split_for_impl();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let output = &self.f.output;
        let v = &self.f.privmod_vis;

        quote!(
            /// Expectation type for methods that return a `'static` type.
            /// This is the type returned by the `expect_*` methods.
            #v struct Expectation #ig #wc {
                common: Common #common_tg,
                rfunc: Mutex<Rfunc #tg>,
            }

            #[allow(clippy::unused_unit)]
            impl #ig Expectation #tg #wc {
                /// Call this [`Expectation`] as if it were the real method.
                #[doc(hidden)]
                #v fn call #lg (&self, #(#argnames: #argty, )* ) -> #output
                {
                    self.common.call(&#desc);
                    self.rfunc.lock().unwrap().call_mut(#(#argnames, )*)
                        .unwrap_or_else(|message| {
                            let desc = std::format!(
                                "{}", self.common.matcher.lock().unwrap());
                            panic!("{}: Expectation({}) {}", #funcname, desc,
                                   message);
                        })
                }

                /// Return a constant value from the `Expectation`
                ///
                /// The output type must be `Clone`.  The compiler can't always
                /// infer the proper type to use with this method; you will
                /// usually need to specify it explicitly.  i.e.
                /// `return_const(42i32)` instead of `return_const(42)`.
                // We must use Into<#output> instead of #output because where
                // clauses don't accept equality constraints.
                // https://github.com/rust-lang/rust/issues/20041
                #[allow(unused_variables)]
                #v fn return_const<MockallOutput>(&mut self,
                    __mockall_c: MockallOutput)
                    -> &mut Self
                    where MockallOutput: Clone + Into<#output> + Send + 'static
                {
                    self.returning(move |#(#argnames, )*| __mockall_c.clone().into())
                }

                /// Single-threaded version of
                /// [`return_const`](#method.return_const).  This is useful for
                /// return types that are not `Send`.
                ///
                /// The output type must be `Clone`.  The compiler can't always
                /// infer the proper type to use with this method; you will
                /// usually need to specify it explicitly.  i.e.
                /// `return_const(42i32)` instead of `return_const(42)`.
                ///
                /// It is a runtime error to call the mock method from a
                /// different thread than the one that originally called this
                /// method.
                // We must use Into<#output> instead of #output because where
                // clauses don't accept equality constraints.
                // https://github.com/rust-lang/rust/issues/20041
                #[allow(unused_variables)]
                #v fn return_const_st<MockallOutput>(&mut self,
                    __mockall_c: MockallOutput)
                    -> &mut Self
                    where MockallOutput: Clone + Into<#output> + 'static
                {
                    self.returning_st(move |#(#argnames, )*| __mockall_c.clone().into())
                }

                /// Supply an `FnOnce` closure that will provide the return
                /// value for this Expectation.  This is useful for return types
                /// that aren't `Clone`.  It will be an error to call this
                /// method multiple times.
                #v fn return_once<MockallF>(&mut self, __mockall_f: MockallF)
                    -> &mut Self
                    where MockallF: #hrtb FnOnce(#(#argty, )*)
                                    -> #output + Send + 'static
                {
                    {
                        let mut __mockall_guard = self.rfunc.lock().unwrap();
                        *__mockall_guard.deref_mut() =
                            Rfunc::Once(Box::new(__mockall_f));
                    }
                    self
                }

                /// Single-threaded version of
                /// [`return_once`](#method.return_once).  This is useful for
                /// return types that are neither `Send` nor `Clone`.
                ///
                /// It is a runtime error to call the mock method from a
                /// different thread than the one that originally called this
                /// method.  It is also a runtime error to call the method more
                /// than once.
                #v fn return_once_st<MockallF>(&mut self, __mockall_f:
                                                  MockallF) -> &mut Self
                    where MockallF: #hrtb FnOnce(#(#argty, )*)
                                    -> #output + 'static
                {
                    {
                        let mut __mockall_guard = self.rfunc.lock().unwrap();
                        *__mockall_guard.deref_mut() = Rfunc::OnceSt(
                            ::mockall::Fragile::new(Box::new(__mockall_f)));
                    }
                    self
                }

                /// Supply a closure that will provide the return value for this
                /// `Expectation`.  The method's arguments are passed to the
                /// closure by value.
                #v fn returning<MockallF>(&mut self, __mockall_f: MockallF)
                    -> &mut Self
                    where MockallF: #hrtb FnMut(#(#argty, )*)
                                    -> #output + Send + 'static
                {
                    {
                        let mut __mockall_guard = self.rfunc.lock().unwrap();
                        *__mockall_guard.deref_mut() =
                            Rfunc::Mut(Box::new(__mockall_f));
                    }
                    self
                }

                /// Single-threaded version of [`returning`](#method.returning).
                /// Can be used when the argument or return type isn't `Send`.
                ///
                /// It is a runtime error to call the mock method from a
                /// different thread than the one that originally called this
                /// method.
                #v fn returning_st<MockallF>(&mut self, __mockall_f: MockallF)
                    -> &mut Self
                    where MockallF: #hrtb FnMut(#(#argty, )*)
                                    -> #output + 'static
                {
                    {
                        let mut __mockall_guard = self.rfunc.lock().unwrap();
                        *__mockall_guard.deref_mut() = Rfunc::MutSt(
                            ::mockall::Fragile::new(Box::new(__mockall_f)));
                    }
                    self
                }

                #common_methods
            }
            impl #ig Default for Expectation #tg #wc
            {
                fn default() -> Self {
                    Expectation {
                        common: Common::default(),
                        rfunc: Mutex::new(Rfunc::default())
                    }
                }
            }
        ).to_tokens(tokens);
    }
}

/// An collection of RefExpectation's
struct RefExpectations<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for RefExpectations<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let common_methods = CommonExpectationsMethods{f: self.f};
        let argnames = &self.f.argnames;
        let argty = &self.f.argty;
        let (ig, tg, wc) = self.f.egenerics.split_for_impl();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let output = &self.f.output;
        let predexprs = &self.f.predexprs;
        let v = &self.f.privmod_vis;
        quote!(
            #common_methods
            impl #ig Expectations #tg #wc {
                /// Simulate calling the real method.  Every current expectation
                /// will be checked in FIFO order and the first one with
                /// matching arguments will be used.
                #v fn call #lg (&self, #(#argnames: #argty, )* )
                    -> Option<#output>
                {
                    self.0.iter()
                        .find(|__mockall_e|
                              __mockall_e.matches(#(#predexprs, )*) &&
                              (!__mockall_e.is_done() || self.0.len() == 1))
                        .map(move |__mockall_e|
                             __mockall_e.call(#(#argnames),*)
                        )
                }

            }
        ).to_tokens(tokens);
    }
}

/// An collection of RefMutExpectation's
struct RefMutExpectations<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for RefMutExpectations<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let common_methods = CommonExpectationsMethods{f: self.f};
        let argnames = &self.f.argnames;
        let argty = &self.f.argty;
        let (ig, tg, wc) = self.f.egenerics.split_for_impl();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let output = &self.f.output;
        let predexprs = &self.f.predexprs;
        let v = &self.f.privmod_vis;
        quote!(
            #common_methods
            impl #ig Expectations #tg #wc {
                /// Simulate calling the real method.  Every current expectation
                /// will be checked in FIFO order and the first one with
                /// matching arguments will be used.
                #v fn call_mut #lg (&mut self, #(#argnames: #argty, )* )
                    -> Option<#output>
                {
                    let __mockall_n = self.0.len();
                    self.0.iter_mut()
                        .find(|__mockall_e|
                              __mockall_e.matches(#(#predexprs, )*) &&
                              (!__mockall_e.is_done() || __mockall_n == 1))
                        .map(move |__mockall_e|
                             __mockall_e.call_mut(#(#argnames, )*)
                        )
                }

            }
        ).to_tokens(tokens);
    }
}

/// An collection of Expectation's for methods returning static values
struct StaticExpectations<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for StaticExpectations<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let common_methods = CommonExpectationsMethods{f: self.f};
        let argnames = &self.f.argnames;
        let argty = &self.f.argty;
        let (ig, tg, wc) = self.f.egenerics.split_for_impl();
        let lg = lifetimes_to_generics(&self.f.alifetimes);
        let output = &self.f.output;
        let predexprs = &self.f.predexprs;
        let v = &self.f.privmod_vis;
        quote!(
            #common_methods
            impl #ig Expectations #tg #wc {
                /// Simulate calling the real method.  Every current expectation
                /// will be checked in FIFO order and the first one with
                /// matching arguments will be used.
                #v fn call #lg (&self, #(#argnames: #argty, )* )
                    -> Option<#output>
                {
                    self.0.iter()
                        .find(|__mockall_e|
                              __mockall_e.matches(#(#predexprs, )*) &&
                              (!__mockall_e.is_done() || self.0.len() == 1))
                        .map(move |__mockall_e|
                             __mockall_e.call(#(#argnames, )*)
                        )
                }

            }
        ).to_tokens(tokens);
    }
}

struct GenericExpectations<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for GenericExpectations<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if ! self.f.is_expectation_generic() {
            return;
        }
        if ! self.f.is_static() && ! self.f.is_method_generic() {
            return;
        }

        let ge = StaticGenericExpectations{f: self.f};
        let v = &self.f.privmod_vis;
        quote!(
            /// A collection of [`Expectation`](struct.Expectations.html)
            /// objects for a generic method.  Users will rarely if ever use
            /// this struct directly.
            #[doc(hidden)]
            #[derive(Default)]
            #v struct GenericExpectations{
                store: std::collections::hash_map::HashMap<::mockall::Key,
                               Box<dyn ::mockall::AnyExpectations>>
            }
            impl GenericExpectations {
                /// Verify that all current expectations are satisfied and clear
                /// them.  This applies to all sets of generic parameters!
                #v fn checkpoint(&mut self) ->
                    std::collections::hash_map::Drain<::mockall::Key,
                               Box<dyn ::mockall::AnyExpectations>>
                {
                    self.store.drain()
                }

                #v fn new() -> Self {
                    Self::default()
                }
            }
            #ge
        ).to_tokens(tokens);
    }
}

/// Generates methods for GenericExpectations for methods returning static
/// values
struct StaticGenericExpectations<'a> {
    f: &'a MockFunction
}

impl<'a> ToTokens for StaticGenericExpectations<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let argnames = &self.f.argnames;
        let argty = &self.f.argty;
        let (ig, tg, wc) = self.f.egenerics.split_for_impl();
        let keyid = gen_keyid(&self.f.egenerics);
        let mut any_wc = wc.cloned();
        if self.f.return_ref || self.f.return_refmut {
            // Add Senc + Sync, required for downcast, since Expectation
            // stores an Option<#owned_output>
            send_syncify(&mut any_wc, self.f.owned_output.clone());
        }
        let tbf = tg.as_turbofish();
        let output = &self.f.output;
        let v = &self.f.privmod_vis;
        let (call, get, self_, downcast) = if self.f.return_refmut {
            (format_ident!("call_mut"),
             format_ident!("get_mut"),
             quote!(&mut self),
             format_ident!("downcast_mut"))
        } else {
            (format_ident!("call"),
             format_ident!("get"),
             quote!(&self),
             format_ident!("downcast_ref"))
        };
        quote!(
            impl #ig ::mockall::AnyExpectations for Expectations #tg #any_wc {}
            impl GenericExpectations {
                /// Simulating calling the real method.
                #v fn #call #ig (#self_, #(#argnames: #argty, )* )
                    -> Option<#output> #wc
                {
                    self.store.#get(&::mockall::Key::new::#keyid())
                        .map(|__mockall_e| {
                            __mockall_e.#downcast::<Expectations #tg>()
                            .unwrap()
                            .#call(#(#argnames, )*)
                        }).flatten()
                }

                /// Create a new Expectation.
                #v fn expect #ig (&mut self) -> &mut Expectation #tg #any_wc
                {
                    self.store.entry(::mockall::Key::new::#keyid())
                        .or_insert_with(|| Box::new(Expectations #tbf::new()))
                        .downcast_mut::<Expectations #tg>()
                        .unwrap()
                        .expect()
                }
            }
        ).to_tokens(tokens)
    }
}

