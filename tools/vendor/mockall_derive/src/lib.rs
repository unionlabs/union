// vim: tw=80
//! Proc Macros for use with Mockall
//!
//! You probably don't want to use this crate directly.  Instead, you should use
//! its reexports via the [`mockall`](https://docs.rs/mockall/latest/mockall)
//! crate.

#![cfg_attr(feature = "nightly_derive", feature(proc_macro_diagnostic))]
#![cfg_attr(test, deny(warnings))]

use cfg_if::cfg_if;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use std::{
    env,
    hash::BuildHasherDefault
};
use syn::{
    *,
    punctuated::Punctuated,
    spanned::Spanned
};

mod automock;
mod mock_function;
mod mock_item;
mod mock_item_struct;
mod mock_trait;
mod mockable_item;
mod mockable_struct;
use crate::automock::Attrs;
use crate::mockable_struct::MockableStruct;
use crate::mock_item::MockItem;
use crate::mock_item_struct::MockItemStruct;
use crate::mockable_item::MockableItem;

// Define deterministic aliases for these common types.
type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<std::collections::hash_map::DefaultHasher>>;
type HashSet<K> = std::collections::HashSet<K, BuildHasherDefault<std::collections::hash_map::DefaultHasher>>;

cfg_if! {
    // proc-macro2's Span::unstable method requires the nightly feature, and it
    // doesn't work in test mode.
    // https://github.com/alexcrichton/proc-macro2/issues/159
    if #[cfg(all(feature = "nightly_derive", not(test)))] {
        fn compile_error(span: Span, msg: &str) {
            span.unstable()
                .error(msg)
                .emit();
        }
    } else {
        fn compile_error(_span: Span, msg: &str) {
            panic!("{}.  More information may be available when mockall is built with the \"nightly\" feature.", msg);
        }
    }
}

fn deanonymize_lifetime(lt: &mut Lifetime) {
    if lt.ident == "_" {
        lt.ident = format_ident!("static");
    }
}

fn deanonymize_path(path: &mut Path) {
    for seg in path.segments.iter_mut() {
        match &mut seg.arguments {
            PathArguments::None => (),
            PathArguments::AngleBracketed(abga) => {
                for ga in abga.args.iter_mut() {
                    if let GenericArgument::Lifetime(lt) = ga {
                        deanonymize_lifetime(lt)
                    }
                }
            },
            _ => compile_error(seg.arguments.span(),
                "Methods returning functions are TODO"),
        }
    }
}

/// Replace any references to the anonymous lifetime `'_` with `'static`.
fn deanonymize(literal_type: &mut Type) {
    match literal_type {
        Type::Array(ta) => deanonymize(ta.elem.as_mut()),
        Type::BareFn(tbf) => {
            if let ReturnType::Type(_, ref mut bt) = tbf.output {
                deanonymize(bt.as_mut());
            }
            for input in tbf.inputs.iter_mut() {
                deanonymize(&mut input.ty);
            }
        },
        Type::Group(tg) => deanonymize(tg.elem.as_mut()),
        Type::Infer(_) => (),
        Type::Never(_) => (),
        Type::Paren(tp) => deanonymize(tp.elem.as_mut()),
        Type::Path(tp) => {
            if let Some(ref mut qself) = tp.qself {
                deanonymize(qself.ty.as_mut());
            }
            deanonymize_path(&mut tp.path);
        },
        Type::Ptr(tptr) => deanonymize(tptr.elem.as_mut()),
        Type::Reference(tr) => {
            if let Some(lt) = tr.lifetime.as_mut() {
                deanonymize_lifetime(lt)
            }
            deanonymize(tr.elem.as_mut());
        },
        Type::Slice(s) => deanonymize(s.elem.as_mut()),
        Type::TraitObject(tto) => {
            for tpb in tto.bounds.iter_mut() {
                match tpb {
                    TypeParamBound::Trait(tb) => deanonymize_path(&mut tb.path),
                    TypeParamBound::Lifetime(lt) => deanonymize_lifetime(lt),
                }
            }
        },
        Type::Tuple(tt) => {
            for ty in tt.elems.iter_mut() {
                deanonymize(ty)
            }
        }
        x => compile_error(x.span(), "Unimplemented type for deanonymize")
    }
}

// If there are any closures in the argument list, turn them into boxed
// functions
fn declosurefy(gen: &Generics, args: &Punctuated<FnArg, Token![,]>) ->
    (Generics, Vec<FnArg>, Vec<TokenStream>)
{
    let mut hm = HashMap::default();

    let mut save_fn_types = |ident: &Ident, tpb: &TypeParamBound| {
        if let TypeParamBound::Trait(tb) = tpb {
            let fident = &tb.path.segments.last().unwrap().ident;
            if ["Fn", "FnMut", "FnOnce"].iter().any(|s| fident == *s) {
                let newty: Type = parse2(quote!(Box<dyn #tb>)).unwrap();
                let subst_ty: Type = parse2(quote!(#ident)).unwrap();
                assert!(hm.insert(subst_ty, newty).is_none(),
                    "A generic parameter had two Fn bounds?");
            }
        }
    };

    // First, build a HashMap of all Fn generic types
    for g in gen.params.iter() {
        if let GenericParam::Type(tp) = g {
            for tpb in tp.bounds.iter() {
                save_fn_types(&tp.ident, tpb);
            }
        }
    }
    if let Some(wc) = &gen.where_clause {
        for pred in wc.predicates.iter() {
            if let WherePredicate::Type(pt) = pred {
                let bounded_ty = &pt.bounded_ty;
                if let Ok(ident) = parse2::<Ident>(quote!(#bounded_ty)) {
                    for tpb in pt.bounds.iter() {
                        save_fn_types(&ident, tpb);
                    }
                } else {
                    // We can't yet handle where clauses this complicated
                }
            }
        }
    }

    // Then remove those types from both the Generics' params and where clause
    let should_remove = |ident: &Ident| {
            let ty: Type = parse2(quote!(#ident)).unwrap();
            hm.contains_key(&ty)
    };
    let params = gen.params.iter()
        .filter(|g| {
            if let GenericParam::Type(tp) = g {
                !should_remove(&tp.ident)
            } else {
                true
            }
        }).cloned()
        .collect::<Punctuated<_, _>>();
    let mut wc2 = gen.where_clause.clone();
    if let Some(wc) = &mut wc2 {
        wc.predicates = wc.predicates.iter()
            .filter(|wp| {
                if let WherePredicate::Type(pt) = wp {
                    let bounded_ty = &pt.bounded_ty;
                    if let Ok(ident) = parse2::<Ident>(quote!(#bounded_ty)) {
                        !should_remove(&ident)
                    } else {
                        // We can't yet handle where clauses this complicated
                        true
                    }
                } else {
                    true
                }
            }).cloned()
            .collect::<Punctuated<_, _>>();
        if wc.predicates.is_empty() {
            wc2 = None;
        }
    }
    let outg = Generics {
        lt_token: if params.is_empty() { None } else { gen.lt_token },
        gt_token: if params.is_empty() { None } else { gen.gt_token },
        params,
        where_clause: wc2
    };

    // Next substitute Box<Fn> into the arguments
    let outargs = args.iter().map(|arg| {
        if let FnArg::Typed(pt) = arg {
            let mut immutable_pt = pt.clone();
            demutify_arg(&mut immutable_pt);
            if let Some(newty) = hm.get(&pt.ty) {
                FnArg::Typed(PatType {
                    attrs: Vec::default(),
                    pat: immutable_pt.pat,
                    colon_token: pt.colon_token,
                    ty: Box::new(newty.clone())
                })
            } else {
                FnArg::Typed(PatType {
                    attrs: Vec::default(),
                    pat: immutable_pt.pat,
                    colon_token: pt.colon_token,
                    ty: pt.ty.clone()
                })
            }
        } else {
            arg.clone()
        }
    }).collect();

    // Finally, Box any closure arguments
    // use filter_map to remove the &self argument
    let callargs = args.iter().filter_map(|arg| {
        match arg {
            FnArg::Typed(pt) => {
                let mut pt2 = pt.clone();
                demutify_arg(&mut pt2);
                let pat = &pt2.pat;
                if pat_is_self(pat) {
                    None
                } else if hm.contains_key(&pt.ty) {
                    Some(quote!(Box::new(#pat)))
                } else {
                    Some(quote!(#pat))
                }
            },
            FnArg::Receiver(_) => None,
        }
    }).collect();
    (outg, outargs, callargs)
}

/// Replace any "impl trait" types with "Box<dyn trait>" or equivalent.
fn deimplify(rt: &mut ReturnType) {
    if let ReturnType::Type(_, ty) = rt {
        if let Type::ImplTrait(ref tit) = &**ty {
            let needs_pin = tit.bounds
                .iter()
                .any(|tpb| {
                    if let TypeParamBound::Trait(tb) = tpb {
                        if let Some(seg) = tb.path.segments.last() {
                            seg.ident == "Future" || seg.ident == "Stream"
                        } else {
                            // It might still be a Future, but we can't guess
                            // what names it might be imported under.  Too bad.
                            false
                        }
                    } else {
                        false
                    }
                });
            let bounds = &tit.bounds;
            if needs_pin {
                *ty = parse2(quote!(::std::pin::Pin<Box<dyn #bounds>>)).unwrap();
            } else {
                *ty = parse2(quote!(Box<dyn #bounds>)).unwrap();
            }
        }
    }
}

/// Remove any generics that place constraints on Self.
fn dewhereselfify(generics: &mut Generics) {
    if let Some(ref mut wc) = &mut generics.where_clause {
        let new_predicates = wc.predicates.iter()
            .filter(|wp| match wp {
                WherePredicate::Type(pt) => {
                    pt.bounded_ty != parse2(quote!(Self)).unwrap()
                },
                _ => true
            }).cloned()
            .collect::<Punctuated<WherePredicate, Token![,]>>();
        wc.predicates = new_predicates;
    }
    if generics.where_clause.as_ref()
        .map(|wc| wc.predicates.is_empty())
        .unwrap_or(false)
    {
        generics.where_clause = None;
    }
}

/// Remove any mutability qualifiers from a method's argument list
fn demutify(inputs: &mut Punctuated<FnArg, token::Comma>) {
    for arg in inputs.iter_mut() {
        match arg {
            FnArg::Receiver(r) => if r.reference.is_none() {
                r.mutability = None
            },
            FnArg::Typed(pt) => demutify_arg(pt),
        }
    }
}

/// Remove any "mut" from a method argument's binding.
fn demutify_arg(arg: &mut PatType) {
    match *arg.pat {
        Pat::Wild(_) => {
            compile_error(arg.span(),
                "Mocked methods must have named arguments");
        },
        Pat::Ident(ref mut pat_ident) => {
            if let Some(r) = &pat_ident.by_ref {
                compile_error(r.span(),
                    "Mockall does not support by-reference argument bindings");
            }
            if let Some((_at, subpat)) = &pat_ident.subpat {
                compile_error(subpat.span(),
                    "Mockall does not support subpattern bindings");
            }
            pat_ident.mutability = None;
        },
        _ => {
            compile_error(arg.span(), "Unsupported argument type");
        }
    };
}

fn deselfify_path(path: &mut Path, actual: &Ident, generics: &Generics) {
    for seg in path.segments.iter_mut() {
        if seg.ident == "Self" {
            seg.ident = actual.clone();
            if let PathArguments::None = seg.arguments {
                if !generics.params.is_empty() {
                    let args = generics.params.iter()
                        .map(|gp| {
                            match gp {
                                GenericParam::Type(tp) => {
                                    let ident = tp.ident.clone();
                                    GenericArgument::Type(
                                        Type::Path(
                                            TypePath {
                                                qself: None,
                                                path: Path::from(ident)
                                            }
                                        )
                                    )
                                },
                                GenericParam::Lifetime(ld) =>{
                                    GenericArgument::Lifetime(
                                        ld.lifetime.clone()
                                    )
                                }
                                _ => unimplemented!(),
                            }
                        }).collect::<Punctuated<_, _>>();
                    seg.arguments = PathArguments::AngleBracketed(
                        AngleBracketedGenericArguments {
                            colon2_token: None,
                            lt_token: generics.lt_token.unwrap(),
                            args,
                            gt_token: generics.gt_token.unwrap(),
                        }
                    );
                }
            } else {
                compile_error(seg.arguments.span(),
                    "Type arguments after Self are unexpected");
            }
        }
        if let PathArguments::AngleBracketed(abga) = &mut seg.arguments
        {
            for arg in abga.args.iter_mut() {
                match arg {
                    GenericArgument::Type(ty) =>
                        deselfify(ty, actual, generics),
                    GenericArgument::Binding(b) =>
                        deselfify(&mut b.ty, actual, generics),
                    _ => /* Nothing to do */(),
                }
            }
        }
    }
}

/// Replace any references to `Self` in `literal_type` with `actual`.
/// `generics` is the Generics field of the parent struct.  Useful for
/// constructor methods.
fn deselfify(literal_type: &mut Type, actual: &Ident, generics: &Generics) {
    match literal_type {
        Type::Slice(s) => {
            deselfify(s.elem.as_mut(), actual, generics);
        },
        Type::Array(a) => {
            deselfify(a.elem.as_mut(), actual, generics);
        },
        Type::Ptr(p) => {
            deselfify(p.elem.as_mut(), actual, generics);
        },
        Type::Reference(r) => {
            deselfify(r.elem.as_mut(), actual, generics);
        },
        Type::Tuple(tuple) => {
            for elem in tuple.elems.iter_mut() {
                deselfify(elem, actual, generics);
            }
        }
        Type::Path(type_path) => {
            if let Some(ref mut qself) = type_path.qself {
                deselfify(qself.ty.as_mut(), actual, generics);
            }
            deselfify_path(&mut type_path.path, actual, generics);
        },
        Type::Paren(p) => {
            deselfify(p.elem.as_mut(), actual, generics);
        },
        Type::Group(g) => {
            deselfify(g.elem.as_mut(), actual, generics);
        },
        Type::Macro(_) | Type::Verbatim(_) => {
            compile_error(literal_type.span(),
                "mockall_derive does not support this type as a return argument");
        },
        Type::TraitObject(tto) => {
            // Change types like `dyn Self` into `dyn MockXXX`.
            for bound in tto.bounds.iter_mut() {
                if let TypeParamBound::Trait(t) = bound {
                    deselfify_path(&mut t.path, actual, generics);
                }
            }
        },
        Type::ImplTrait(_) => {
            /* Should've already been flagged as a compile_error */
        },
        Type::BareFn(_) => {
            /* Bare functions can't have Self arguments.  Nothing to do */
        },
        Type::Infer(_) | Type::Never(_) =>
        {
            /* Nothing to do */
        },
        _ => compile_error(literal_type.span(), "Unsupported type"),
    }
}

/// Change any `Self` in a method's arguments' types with `actual`.
/// `generics` is the Generics field of the parent struct.
fn deselfify_args(
    args: &mut Punctuated<FnArg, Token![,]>,
    actual: &Ident,
    generics: &Generics)
{
    for arg in args.iter_mut() {
        if let FnArg::Typed(pt) = arg {
            deselfify(pt.ty.as_mut(), actual, generics)
        }
    }
}

fn find_ident_from_path(path: &Path) -> (Ident, PathArguments) {
    if path.segments.len() != 1 {
        compile_error(path.span(),
            "mockall_derive only supports structs defined in the current module");
        return (Ident::new("", path.span()), PathArguments::None);
    }
    let last_seg = path.segments.last().unwrap();
    (last_seg.ident.clone(), last_seg.arguments.clone())
}

fn find_lifetimes_in_tpb(bound: &TypeParamBound) -> HashSet<Lifetime> {
    let mut ret = HashSet::default();
    match bound {
        TypeParamBound::Lifetime(lt) => {
            ret.insert(lt.clone());
        },
        TypeParamBound::Trait(tb) => {
            ret.extend(find_lifetimes_in_path(&tb.path));
        },
    };
    ret
}

fn find_lifetimes_in_path(path: &Path) -> HashSet<Lifetime> {
    let mut ret = HashSet::default();
    for seg in path.segments.iter() {
        if let PathArguments::AngleBracketed(abga) = &seg.arguments {
            for arg in abga.args.iter() {
                match arg {
                    GenericArgument::Lifetime(lt) => {
                        ret.insert(lt.clone());
                    },
                    GenericArgument::Type(ty) => {
                        ret.extend(find_lifetimes(ty));
                    },
                    GenericArgument::Binding(b) => {
                        ret.extend(find_lifetimes(&b.ty));
                    },
                    GenericArgument::Constraint(c) => {
                        for bound in c.bounds.iter() {
                            ret.extend(find_lifetimes_in_tpb(bound));
                        }
                    },
                    GenericArgument::Const(_) => ()
                }
            }
        }
    }
    ret
}

fn find_lifetimes(ty: &Type) -> HashSet<Lifetime> {
    match ty {
        Type::Array(ta) => find_lifetimes(ta.elem.as_ref()),
        Type::Group(tg) => find_lifetimes(tg.elem.as_ref()),
        Type::Infer(_ti) => HashSet::default(),
        Type::Never(_tn) => HashSet::default(),
        Type::Paren(tp) => find_lifetimes(tp.elem.as_ref()),
        Type::Path(tp) => {
            let mut ret = find_lifetimes_in_path(&tp.path);
            if let Some(qs) = &tp.qself {
                ret.extend(find_lifetimes(qs.ty.as_ref()));
            }
            ret
        },
        Type::Ptr(tp) => find_lifetimes(tp.elem.as_ref()),
        Type::Reference(tr) => {
            let mut ret = find_lifetimes(tr.elem.as_ref());
            if let Some(lt) = &tr.lifetime {
                ret.insert(lt.clone());
            }
            ret
        },
        Type::Slice(ts) => find_lifetimes(ts.elem.as_ref()),
        Type::TraitObject(tto) => {
            let mut ret = HashSet::default();
            for bound in tto.bounds.iter() {
                ret.extend(find_lifetimes_in_tpb(bound));
            }
            ret
        }
        Type::Tuple(tt) => {
            let mut ret = HashSet::default();
            for ty in tt.elems.iter() {
                ret.extend(find_lifetimes(ty));
            }
            ret
        },
        Type::ImplTrait(tit) => {
            let mut ret = HashSet::default();
            for tpb in tit.bounds.iter() {
                ret.extend(find_lifetimes_in_tpb(tpb));
            }
            ret
        },
        _ => {
            compile_error(ty.span(), "unsupported type in this context");
            HashSet::default()
        }
    }
}


struct AttrFormatter<'a>{
    attrs: &'a [Attribute],
    async_trait: bool,
    doc: bool,
}

impl<'a> AttrFormatter<'a> {
    fn new(attrs: &'a [Attribute]) -> AttrFormatter<'a> {
        Self {
            attrs,
            async_trait: true,
            doc: true
        }
    }

    fn async_trait(&mut self, allowed: bool) -> &mut Self {
        self.async_trait = allowed;
        self
    }

    fn doc(&mut self, allowed: bool) -> &mut Self {
        self.doc = allowed;
        self
    }

    // XXX This logic requires that attributes are imported with their
    // standard names.
    #[allow(clippy::needless_bool)]
    #[allow(clippy::if_same_then_else)]
    fn format(&mut self) -> Vec<Attribute> {
        self.attrs.iter()
            .cloned()
            .filter(|attr| {
                let i = attr.path.get_ident();
                if i.is_none() {
                    false
                } else if *i.as_ref().unwrap() == "derive" {
                    // We can't usefully derive any traits.  Ignore them
                    false
                } else if *i.as_ref().unwrap() == "doc" {
                    self.doc
                } else if *i.as_ref().unwrap() == "async_trait" {
                    self.async_trait
                } else if *i.as_ref().unwrap() == "instrument" {
                    // We can't usefully instrument the mock method, so just
                    // ignore this attribute.
                    // https://docs.rs/tracing/0.1.23/tracing/attr.instrument.html
                    false
                } else {
                    true
                }
            }).collect()
    }
}

/// Determine if this Pat is any kind of `self` binding
fn pat_is_self(pat: &Pat) -> bool {
    if let Pat::Ident(pi) = pat {
        pi.ident == "self"
    } else {
        false
    }
}

/// Add `levels` `super::` to the path.  Return the number of levels added.
fn supersuperfy_path(path: &mut Path, levels: usize) -> usize {
    if let Some(t) = path.segments.last_mut() {
        match &mut t.arguments {
            PathArguments::None => (),
            PathArguments::AngleBracketed(ref mut abga) => {
                for arg in abga.args.iter_mut() {
                    match arg {
                        GenericArgument::Type(ref mut ty) => {
                            *ty = supersuperfy(ty, levels);
                        },
                        GenericArgument::Binding(ref mut binding) => {
                            binding.ty = supersuperfy(&binding.ty, levels);
                        },
                        GenericArgument::Constraint(ref mut constraint) => {
                            supersuperfy_bounds(&mut constraint.bounds, levels);
                        },
                        _ => (),
                    }
                }
            },
            PathArguments::Parenthesized(ref mut pga) => {
                for input in pga.inputs.iter_mut() {
                    *input = supersuperfy(input, levels);
                }
                if let ReturnType::Type(_, ref mut ty) = pga.output {
                    *ty = Box::new(supersuperfy(ty, levels));
                }
            },
        }
    }
    if let Some(t) = path.segments.first() {
        if t.ident == "super" {
            let mut ident = format_ident!("super");
            ident.set_span(path.segments.span());
            let ps = PathSegment {
                ident,
                arguments: PathArguments::None
            };
            for _ in 0..levels {
                path.segments.insert(0, ps.clone());
            }
            levels
        } else {
            0
        }
    } else {
        0
    }
}

/// Replace any references to `super::X` in `original` with `super::super::X`.
fn supersuperfy(original: &Type, levels: usize) -> Type {
    let mut output = original.clone();
    fn recurse(t: &mut Type, levels: usize) {
        match t {
            Type::Slice(s) => {
                recurse(s.elem.as_mut(), levels);
            },
            Type::Array(a) => {
                recurse(a.elem.as_mut(), levels);
            },
            Type::Ptr(p) => {
                recurse(p.elem.as_mut(), levels);
            },
            Type::Reference(r) => {
                recurse(r.elem.as_mut(), levels);
            },
            Type::BareFn(bfn) => {
                if let ReturnType::Type(_, ref mut bt) = bfn.output {
                    recurse(bt.as_mut(), levels);
                }
                for input in bfn.inputs.iter_mut() {
                    recurse(&mut input.ty, levels);
                }
            },
            Type::Tuple(tuple) => {
                for elem in tuple.elems.iter_mut() {
                    recurse(elem, levels);
                }
            }
            Type::Path(type_path) => {
                let added = supersuperfy_path(&mut type_path.path, levels);
                if let Some(ref mut qself) = type_path.qself {
                    recurse(qself.ty.as_mut(), levels);
                    qself.position += added;
                }
            },
            Type::Paren(p) => {
                recurse(p.elem.as_mut(), levels);
            },
            Type::Group(g) => {
                recurse(g.elem.as_mut(), levels);
            },
            Type::Macro(_) | Type::Verbatim(_) => {
                compile_error(t.span(),
                    "mockall_derive does not support this type in this position");
            },
            Type::TraitObject(tto) => {
                for bound in tto.bounds.iter_mut() {
                    if let TypeParamBound::Trait(tb) = bound {
                        supersuperfy_path(&mut tb.path, levels);
                    }
                }
            },
            Type::ImplTrait(_) => {
                /* Should've already been flagged as a compile error */
            },
            Type::Infer(_) | Type::Never(_) =>
            {
                /* Nothing to do */
            },
            _ => compile_error(t.span(), "Unsupported type"),
        }
    }
    recurse(&mut output, levels);
    output
}

fn supersuperfy_generics(generics: &mut Generics, levels: usize) {
    for param in generics.params.iter_mut() {
        if let GenericParam::Type(tp) = param {
            supersuperfy_bounds(&mut tp.bounds, levels);
            if let Some(ty) = tp.default.as_mut() {
                *ty = supersuperfy(ty, levels);
            }
        }
    }
    if let Some(wc) = generics.where_clause.as_mut() {
        for wp in wc.predicates.iter_mut() {
            if let WherePredicate::Type(pt) = wp {
                pt.bounded_ty = supersuperfy(&pt.bounded_ty, levels);
                supersuperfy_bounds(&mut pt.bounds, levels);
            }
        }
    }
}

fn supersuperfy_bounds(
    bounds: &mut Punctuated<TypeParamBound, Token![+]>,
    levels: usize)
{
    for bound in bounds.iter_mut() {
        if let TypeParamBound::Trait(tb) = bound {
            supersuperfy_path(&mut tb.path, levels);
        }
    }
}

/// Generate a suitable mockall::Key generic paramter from any Generics
fn gen_keyid(g: &Generics) -> impl ToTokens {
    match g.params.len() {
        0 => quote!(<()>),
        1 => {
            let (_, tg, _) = g.split_for_impl();
            quote!(#tg)
        },
        _ => {
            // Rust doesn't support variadic Generics, so mockall::Key must
            // always have exactly one generic type.  We need to add parentheses
            // around whatever type generics the caller passes.
            let tps = g.type_params()
            .map(|tp| tp.ident.clone())
            .collect::<Punctuated::<Ident, Token![,]>>();
            quote!(<(#tps)>)
        }
    }
}

/// Generate a mock identifier from the regular one: eg "Foo" => "MockFoo"
fn gen_mock_ident(ident: &Ident) -> Ident {
    format_ident!("Mock{}", ident)
}

/// Generate an identifier for the mock struct's private module: eg "Foo" =>
/// "__mock_Foo"
fn gen_mod_ident(struct_: &Ident, trait_: Option<&Ident>) -> Ident {
    if let Some(t) = trait_ {
        format_ident!("__mock_{}_{}", struct_, t)
    } else {
        format_ident!("__mock_{}", struct_)
    }
}

/// Combine two Generics structs, producing a new one that has the union of
/// their parameters.
fn merge_generics(x: &Generics, y: &Generics) -> Generics {
    /// Compare only the identifiers of two GenericParams
    fn cmp_gp_idents(x: &GenericParam, y: &GenericParam) -> bool {
        use GenericParam::*;

        match (x, y) {
            (Type(xtp), Type(ytp)) => xtp.ident == ytp.ident,
            (Lifetime(xld), Lifetime(yld)) => xld.lifetime == yld.lifetime,
            (Const(xc), Const(yc)) => xc.ident == yc.ident,
            _ => false
        }
    }

    /// Compare only the identifiers of two WherePredicates
    fn cmp_wp_idents(x: &WherePredicate, y: &WherePredicate) -> bool {
        use WherePredicate::*;

        match (x, y) {
            (Type(xpt), Type(ypt)) => xpt.bounded_ty == ypt.bounded_ty,
            (Lifetime(xpl), Lifetime(ypl)) => xpl.lifetime == ypl.lifetime,
            (Eq(xeq), Eq(yeq)) => xeq.lhs_ty == yeq.lhs_ty,
            _ => false
        }
    }

    let mut out = if x.lt_token.is_none() && x.where_clause.is_none() {
        y.clone()
    } else if y.lt_token.is_none() && y.where_clause.is_none() {
        x.clone()
    } else {
        let mut out = x.clone();
        // First merge the params
        'outer_param: for yparam in y.params.iter() {
            // XXX: O(n^2) loop
            for outparam in out.params.iter_mut() {
                if cmp_gp_idents(outparam, yparam) {
                    if let (GenericParam::Type(ref mut ot),
                            GenericParam::Type(yt)) = (outparam, yparam)
                    {
                        ot.attrs.extend(yt.attrs.iter().cloned());
                        ot.colon_token = ot.colon_token.or(yt.colon_token);
                        ot.eq_token = ot.eq_token.or(yt.eq_token);
                        if ot.default.is_none() {
                            ot.default = yt.default.clone();
                        }
                        // XXX this might result in duplicate bounds
                        if ot.bounds != yt.bounds {
                            ot.bounds.extend(yt.bounds.iter().cloned());
                        }
                    }
                    continue 'outer_param;
                }
            }
            out.params.push(yparam.clone());
        }
        out
    };
    // Then merge the where clauses
    match (&mut out.where_clause, &y.where_clause) {
        (_, None) => (),
        (None, Some(wc)) => out.where_clause = Some(wc.clone()),
        (Some(out_wc), Some(y_wc)) => {
            'outer_wc: for ypred in y_wc.predicates.iter() {
                // XXX: O(n^2) loop
                for outpred in out_wc.predicates.iter_mut() {
                    if cmp_wp_idents(outpred, ypred) {
                        if let (WherePredicate::Type(ref mut ot),
                                WherePredicate::Type(yt)) = (outpred, ypred)
                        {
                            match (&mut ot.lifetimes, &yt.lifetimes) {
                                (_, None) => (),
                                (None, Some(bl)) =>
                                    ot.lifetimes = Some(bl.clone()),
                                (Some(obl), Some(ybl)) =>
                                    // XXX: might result in duplicates
                                    obl.lifetimes.extend(
                                        ybl.lifetimes.iter().cloned()),
                            };
                            // XXX: might result in duplicate bounds
                            if ot.bounds != yt.bounds {
                                ot.bounds.extend(yt.bounds.iter().cloned())
                            }
                        }
                        continue 'outer_wc;
                    }
                }
                out_wc.predicates.push(ypred.clone());
            }
        }
    }
    out
}

/// Transform a Vec of lifetimes into a Generics
fn lifetimes_to_generics(lv: &Punctuated<LifetimeDef, Token![,]>)-> Generics {
    if lv.is_empty() {
            Generics::default()
    } else {
        let params = lv.iter()
            .map(|lt| GenericParam::Lifetime(lt.clone()))
            .collect();
        Generics {
            lt_token: Some(Token![<](lv[0].span())),
            gt_token: Some(Token![>](lv[0].span())),
            params,
            where_clause: None
        }
    }
}

/// Split a generics list into three: one for type generics and where predicates
/// that relate to the signature, one for lifetimes that relate to the arguments
/// only, and one for lifetimes that relate to the return type only.
fn split_lifetimes(
    generics: Generics,
    args: &[FnArg],
    rt: &ReturnType)
    -> (Generics,
        Punctuated<LifetimeDef, token::Comma>,
        Punctuated<LifetimeDef, token::Comma>)
{
    if generics.lt_token.is_none() {
        return (generics, Default::default(), Default::default());
    }

    // Check which types and lifetimes are referenced by the arguments
    let mut alts = HashSet::<Lifetime>::default();
    let mut rlts = HashSet::<Lifetime>::default();
    for arg in args {
        match arg {
            FnArg::Receiver(r) => {
                if let Some((_, Some(lt))) = &r.reference {
                    alts.insert(lt.clone());
                }
            },
            FnArg::Typed(pt) => {
                alts.extend(find_lifetimes(pt.ty.as_ref()));
            },
        };
    };

    if let ReturnType::Type(_, ty) = rt {
        rlts.extend(find_lifetimes(ty));
    }

    let mut tv = Punctuated::new();
    let mut alv = Punctuated::new();
    let mut rlv = Punctuated::new();
    for p in generics.params.into_iter() {
        match p {
            GenericParam::Lifetime(ltd) if rlts.contains(&ltd.lifetime) =>
                rlv.push(ltd),
            GenericParam::Lifetime(ltd) if alts.contains(&ltd.lifetime) =>
                alv.push(ltd),
            GenericParam::Lifetime(_) => {
                // Probably a lifetime parameter from the impl block that isn't
                // used by this particular method
            },
            GenericParam::Type(_) => tv.push(p),
            _ => (),
        }
    }

    let tg = if tv.is_empty() {
        Generics::default()
    } else {
        Generics {
            lt_token: generics.lt_token,
            gt_token: generics.gt_token,
            params: tv,
            where_clause: generics.where_clause
        }
    };

    (tg, alv, rlv)
}

/// Return the visibility that should be used for expectation!, given the
/// original method's visibility.
///
/// # Arguments
/// - `vis`:    Original visibility of the item
/// - `levels`: How many modules will the mock item be nested in?
fn expectation_visibility(vis: &Visibility, levels: usize)
    -> Visibility
{
    if levels == 0 {
        return vis.clone();
    }

    let in_token = Token![in](vis.span());
    let super_token = Token![super](vis.span());
    match vis {
        Visibility::Inherited => {
            // Private items need pub(in super::[...]) for each level
            let mut path = Path::from(super_token);
            for _ in 1..levels {
                path.segments.push(super_token.into());
            }
            Visibility::Restricted(VisRestricted{
                pub_token: Token![pub](vis.span()),
                paren_token: token::Paren::default(),
                in_token: Some(in_token),
                path: Box::new(path)
            })
        },
        Visibility::Restricted(vr) => {
            // crate => don't change
            // in crate::* => don't change
            // super => in super::super::super
            // self => in super::super
            // in anything_else => super::super::anything_else
            if vr.path.segments.first().unwrap().ident == "crate" {
                vr.clone().into()
            } else {
                let mut out = vr.clone();
                out.in_token = Some(in_token);
                for _ in 0..levels {
                    out.path.segments.insert(0, super_token.into());
                }
                out.into()
            }
        },
        _ => vis.clone()
    }
}

fn staticize(generics: &Generics) -> Generics {
    let mut ret = generics.clone();
    for lt in ret.lifetimes_mut() {
        lt.lifetime = Lifetime::new("'static", Span::call_site());
    };
    ret
}

fn mock_it<M: Into<MockableItem>>(inputs: M) -> TokenStream
{
    let mockable: MockableItem = inputs.into();
    let mock = MockItem::from(mockable);
    let ts = mock.into_token_stream();
    if env::var("MOCKALL_DEBUG").is_ok() {
        println!("{}", ts);
    }
    ts
}

fn do_mock_once(input: TokenStream) -> TokenStream
{
    let item: MockableStruct = match syn::parse2(input) {
        Ok(mock) => mock,
        Err(err) => {
            return err.to_compile_error();
        }
    };
    mock_it(item)
}

fn do_mock(input: TokenStream) -> TokenStream
{
    cfg_if! {
        if #[cfg(reprocheck)] {
            let ts_a = do_mock_once(input.clone());
            let ts_b = do_mock_once(input.clone());
            assert_eq!(ts_a.to_string(), ts_b.to_string());
        }
    }
    do_mock_once(input)
}

#[proc_macro]
pub fn mock(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_mock(input.into()).into()
}

#[proc_macro_attribute]
pub fn automock(attrs: proc_macro::TokenStream, input: proc_macro::TokenStream)
    -> proc_macro::TokenStream
{
    let attrs: proc_macro2::TokenStream = attrs.into();
    let input: proc_macro2::TokenStream = input.into();
    do_automock(attrs, input).into()
}

fn do_automock_once(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut output = input.clone();
    let attrs: Attrs = match parse2(attrs) {
        Ok(a) => a,
        Err(err) => {
            return err.to_compile_error();
        }
    };
    let item: Item = match parse2(input) {
        Ok(item) => item,
        Err(err) => {
            return err.to_compile_error();
        }
    };
    output.extend(mock_it((attrs, item)));
    output
}

fn do_automock(attrs: TokenStream, input: TokenStream) -> TokenStream {
    cfg_if! {
        if #[cfg(reprocheck)] {
            let ts_a = do_automock_once(attrs.clone(), input.clone());
            let ts_b = do_automock_once(attrs.clone(), input.clone());
            assert_eq!(ts_a.to_string(), ts_b.to_string());
        }
    }
    do_automock_once(attrs, input)
}

#[cfg(test)]
mod t {
    use super::*;

fn assert_contains(output: &str, tokens: TokenStream) {
    let s = tokens.to_string();
    assert!(output.contains(&s), "output does not contain {:?}", &s);
}

fn assert_not_contains(output: &str, tokens: TokenStream) {
    let s = tokens.to_string();
    assert!(!output.contains(&s), "output does not contain {:?}", &s);
}

/// Various tests for overall code generation that are hard or impossible to
/// write as integration tests
mod mock {
    use std::str::FromStr;
    use super::super::*;
    use super::*;

    #[test]
    fn inherent_method_visibility() {
        let code = r#"
            Foo {
                fn foo(&self);
                pub fn bar(&self);
                pub(crate) fn baz(&self);
                pub(super) fn bean(&self);
                pub(in crate::outer) fn boom(&self);
            }
        "#;
        let ts = proc_macro2::TokenStream::from_str(code).unwrap();
        let output = do_mock(ts).to_string();
        assert_not_contains(&output, quote!(pub fn foo));
        assert!(!output.contains(") fn foo"));
        assert_contains(&output, quote!(pub fn bar));
        assert_contains(&output, quote!(pub(crate) fn baz));
        assert_contains(&output, quote!(pub(super) fn bean));
        assert_contains(&output, quote!(pub(in crate::outer) fn boom));

        assert_not_contains(&output, quote!(pub fn expect_foo));
        assert!(!output.contains("pub fn expect_foo"));
        assert!(!output.contains(") fn expect_foo"));
        assert_contains(&output, quote!(pub fn expect_bar));
        assert_contains(&output, quote!(pub(crate) fn expect_baz));
        assert_contains(&output, quote!(pub(super) fn expect_bean));
        assert_contains(&output, quote!(pub(in crate::outer) fn expect_boom));
    }

    #[test]
    fn specific_impl() {
        let code = r#"
            pub Foo<T: 'static> {}
            impl Bar for Foo<u32> {
                fn bar(&self);
            }
            impl Bar for Foo<i32> {
                fn bar(&self);
            }
        "#;
        let ts = proc_macro2::TokenStream::from_str(code).unwrap();
        let output = do_mock(ts).to_string();
        assert_contains(&output, quote!(impl Bar for MockFoo<u32>));
        assert_contains(&output, quote!(impl Bar for MockFoo<i32>));
        // Ensure we don't duplicate the checkpoint function
        assert_not_contains(&output, quote!(
            self.Bar_expectations.checkpoint();
            self.Bar_expectations.checkpoint();
        ));
        // The expect methods should return specific types, not generic ones
        assert_contains(&output, quote!(
            pub fn expect_bar(&mut self) -> &mut __mock_MockFoo_Bar::__bar::Expectation<u32>
        ));
        assert_contains(&output, quote!(
            pub fn expect_bar(&mut self) -> &mut __mock_MockFoo_Bar::__bar::Expectation<i32>
        ));
    }
}

/// Various tests for overall code generation that are hard or impossible to
/// write as integration tests
mod automock {
    use std::str::FromStr;
    use super::super::*;
    use super::*;

    #[test]
    fn doc_comments() {
        let code = r#"
            mod foo {
                /// Function docs
                pub fn bar() { unimplemented!() }
            }
        "#;
        let ts = proc_macro2::TokenStream::from_str(code).unwrap();
        let attrs_ts = proc_macro2::TokenStream::from_str("").unwrap();
        let output = do_automock(attrs_ts, ts).to_string();
        assert_contains(&output, quote!(#[doc=" Function docs"] pub fn bar));
    }

    #[test]
    fn method_visibility() {
        let code = r#"
        impl Foo {
            fn foo(&self) {}
            pub fn bar(&self) {}
            pub(super) fn baz(&self) {}
            pub(crate) fn bang(&self) {}
            pub(in super::x) fn bean(&self) {}
        }"#;
        let ts = proc_macro2::TokenStream::from_str(code).unwrap();
        let attrs_ts = proc_macro2::TokenStream::from_str("").unwrap();
        let output = do_automock(attrs_ts, ts).to_string();
        assert_not_contains(&output, quote!(pub fn foo));
        assert!(!output.contains(") fn foo"));
        assert_not_contains(&output, quote!(pub fn expect_foo));
        assert!(!output.contains(") fn expect_foo"));
        assert_contains(&output, quote!(pub fn bar));
        assert_contains(&output, quote!(pub fn expect_bar));
        assert_contains(&output, quote!(pub(super) fn baz));
        assert_contains(&output, quote!(pub(super) fn expect_baz));
        assert_contains(&output, quote!(pub ( crate ) fn bang));
        assert_contains(&output, quote!(pub ( crate ) fn expect_bang));
        assert_contains(&output, quote!(pub ( in super :: x ) fn bean));
        assert_contains(&output, quote!(pub ( in super :: x ) fn expect_bean));
    }

    #[test]
    #[should_panic(expected = "can only mock inline modules")]
    fn external_module() {
        let code = r#"mod foo;"#;
        let ts = proc_macro2::TokenStream::from_str(code).unwrap();
        let attrs_ts = proc_macro2::TokenStream::from_str("").unwrap();
        do_automock(attrs_ts, ts).to_string();
    }

    #[test]
    fn trait_visibility() {
        let code = r#"
        pub(super) trait Foo {}
        "#;
        let attrs_ts = proc_macro2::TokenStream::from_str("").unwrap();
        let ts = proc_macro2::TokenStream::from_str(code).unwrap();
        let output = do_automock(attrs_ts, ts).to_string();
        assert_contains(&output, quote!(pub ( super ) struct MockFoo));
    }
}

mod deimplify {
    use super::*;

    fn check_deimplify(orig_ts: TokenStream, expected_ts: TokenStream) {
        let mut orig: ReturnType = parse2(orig_ts).unwrap();
        let expected: ReturnType = parse2(expected_ts).unwrap();
        deimplify(&mut orig);
        assert_eq!(quote!(#orig).to_string(), quote!(#expected).to_string());
    }

    // Future is a special case
    #[test]
    fn impl_future() {
        check_deimplify(
            quote!(-> impl Future<Output=i32>),
            quote!(-> ::std::pin::Pin<Box<dyn Future<Output=i32>>>)
        );
    }

    // Future is a special case, wherever it appears
    #[test]
    fn impl_future_reverse() {
        check_deimplify(
            quote!(-> impl Send + Future<Output=i32>),
            quote!(-> ::std::pin::Pin<Box<dyn Send + Future<Output=i32>>>)
        );
    }

    // Stream is a special case
    #[test]
    fn impl_stream() {
        check_deimplify(
            quote!(-> impl Stream<Item=i32>),
            quote!(-> ::std::pin::Pin<Box<dyn Stream<Item=i32>>>)
        );
    }

    #[test]
    fn impl_trait() {
        check_deimplify(
            quote!(-> impl Foo),
            quote!(-> Box<dyn Foo>)
        );
    }

    // With extra bounds
    #[test]
    fn impl_trait2() {
        check_deimplify(
            quote!(-> impl Foo + Send),
            quote!(-> Box<dyn Foo + Send>)
        );
    }
}

mod deselfify {
    use super::*;

    fn check_deselfify(
        orig_ts: TokenStream,
        actual_ts: TokenStream,
        generics_ts: TokenStream,
        expected_ts: TokenStream)
    {
        let mut ty: Type = parse2(orig_ts).unwrap();
        let actual: Ident = parse2(actual_ts).unwrap();
        let generics: Generics = parse2(generics_ts).unwrap();
        let expected: Type = parse2(expected_ts).unwrap();
        deselfify(&mut ty, &actual, &generics);
        assert_eq!(quote!(#ty).to_string(),
                   quote!(#expected).to_string());
    }

    #[test]
    fn future() {
        check_deselfify(
            quote!(Box<dyn Future<Output=Self>>),
            quote!(Foo),
            quote!(),
            quote!(Box<dyn Future<Output=Foo>>)
        );
    }

    #[test]
    fn qself() {
        check_deselfify(
            quote!(<Self as Self>::Self),
            quote!(Foo),
            quote!(),
            quote!(<Foo as Foo>::Foo)
        );
    }

    #[test]
    fn trait_object() {
        check_deselfify(
            quote!(Box<dyn Self>),
            quote!(Foo),
            quote!(),
            quote!(Box<dyn Foo>)
        );
    }

    // A trait object with multiple bounds
    #[test]
    fn trait_object2() {
        check_deselfify(
            quote!(Box<dyn Self + Send>),
            quote!(Foo),
            quote!(),
            quote!(Box<dyn Foo + Send>)
        );
    }
}

mod dewhereselfify {
    use super::*;

    #[test]
    fn lifetime() {
        let mut meth: ImplItemMethod = parse2(quote!(
                fn foo<'a>(&self) where 'a: 'static, Self: Sized;
        )).unwrap();
        let expected: ImplItemMethod = parse2(quote!(
                fn foo<'a>(&self) where 'a: 'static;
        )).unwrap();
        dewhereselfify(&mut meth.sig.generics);
        assert_eq!(meth, expected);
    }

    #[test]
    fn normal_method() {
        let mut meth: ImplItemMethod = parse2(quote!(
                fn foo(&self) where Self: Sized;
        )).unwrap();
        let expected: ImplItemMethod = parse2(quote!(
                fn foo(&self);
        )).unwrap();
        dewhereselfify(&mut meth.sig.generics);
        assert_eq!(meth, expected);
    }

    #[test]
    fn with_real_generics() {
        let mut meth: ImplItemMethod = parse2(quote!(
                fn foo<T>(&self, t: T) where Self: Sized, T: Copy;
        )).unwrap();
        let expected: ImplItemMethod = parse2(quote!(
                fn foo<T>(&self, t: T) where T: Copy;
        )).unwrap();
        dewhereselfify(&mut meth.sig.generics);
        assert_eq!(meth, expected);
    }
}

mod gen_keyid {
    use super::*;

    fn check_gen_keyid(orig: TokenStream, expected: TokenStream) {
        let g: Generics = parse2(orig).unwrap();
        let keyid = gen_keyid(&g);
        assert_eq!(quote!(#keyid).to_string(), quote!(#expected).to_string());
    }

    #[test]
    fn empty() {
        check_gen_keyid(quote!(), quote!(<()>));
    }

    #[test]
    fn onetype() {
        check_gen_keyid(quote!(<T>), quote!(<T>));
    }

    #[test]
    fn twotypes() {
        check_gen_keyid(quote!(<T, V>), quote!(<(T, V)>));
    }
}

mod merge_generics {
    use super::*;

    #[test]
    fn both() {
        let mut g1: Generics = parse2(quote!(<T: 'static, V: Copy> )).unwrap();
        let wc1: WhereClause = parse2(quote!(where T: Default)).unwrap();
        g1.where_clause = Some(wc1);

        let mut g2: Generics = parse2(quote!(<Q: Send, V: Clone>)).unwrap();
        let wc2: WhereClause = parse2(quote!(where T: Sync, Q: Debug)).unwrap();
        g2.where_clause = Some(wc2);

        let gm = super::merge_generics(&g1, &g2);
        let gm_wc = &gm.where_clause;

        let ge: Generics = parse2(quote!(
                <T: 'static, V: Copy + Clone, Q: Send>
        )).unwrap();
        let wce: WhereClause = parse2(quote!(
            where T: Default + Sync, Q: Debug
        )).unwrap();

        assert_eq!(quote!(#ge #wce).to_string(),
                   quote!(#gm #gm_wc).to_string());
    }

    #[test]
    fn eq() {
        let mut g1: Generics = parse2(quote!(<T: 'static, V: Copy> )).unwrap();
        let wc1: WhereClause = parse2(quote!(where T: Default)).unwrap();
        g1.where_clause = Some(wc1.clone());

        let gm = super::merge_generics(&g1, &g1);
        let gm_wc = &gm.where_clause;

        assert_eq!(quote!(#g1 #wc1).to_string(),
                   quote!(#gm #gm_wc).to_string());
    }

    #[test]
    fn lhs_only() {
        let mut g1: Generics = parse2(quote!(<T: 'static, V: Copy> )).unwrap();
        let wc1: WhereClause = parse2(quote!(where T: Default)).unwrap();
        g1.where_clause = Some(wc1.clone());

        let g2 = Generics::default();

        let gm = super::merge_generics(&g1, &g2);
        let gm_wc = &gm.where_clause;

        assert_eq!(quote!(#g1 #wc1).to_string(),
                   quote!(#gm #gm_wc).to_string());
    }

    #[test]
    fn lhs_wc_only() {
        let mut g1 = Generics::default();
        let wc1: WhereClause = parse2(quote!(where T: Default)).unwrap();
        g1.where_clause = Some(wc1.clone());

        let g2 = Generics::default();

        let gm = super::merge_generics(&g1, &g2);
        let gm_wc = &gm.where_clause;

        assert_eq!(quote!(#g1 #wc1).to_string(),
                   quote!(#gm #gm_wc).to_string());
    }

    #[test]
    fn rhs_only() {
        let g1 = Generics::default();
        let mut g2: Generics = parse2(quote!(<Q: Send, V: Clone>)).unwrap();
        let wc2: WhereClause = parse2(quote!(where T: Sync, Q: Debug)).unwrap();
        g2.where_clause = Some(wc2.clone());

        let gm = super::merge_generics(&g1, &g2);
        let gm_wc = &gm.where_clause;

        assert_eq!(quote!(#g2 #wc2).to_string(),
                   quote!(#gm #gm_wc).to_string());
    }
}

mod supersuperfy {
    use super::*;

    fn check_supersuperfy(orig: TokenStream, expected: TokenStream) {
        let orig_ty: Type = parse2(orig).unwrap();
        let expected_ty: Type = parse2(expected).unwrap();
        let output = supersuperfy(&orig_ty, 1);
        assert_eq!(quote!(#output).to_string(),
                   quote!(#expected_ty).to_string());
    }

    #[test]
    fn array() {
        check_supersuperfy(
            quote!([super::X; n]),
            quote!([super::super::X; n])
        );
    }

    #[test]
    fn barefn() {
        check_supersuperfy(
            quote!(fn(super::A) -> super::B),
            quote!(fn(super::super::A) -> super::super::B)
        );
    }

    #[test]
    fn group() {
        let orig = TypeGroup {
            group_token: token::Group::default(),
            elem: Box::new(parse2(quote!(super::T)).unwrap())
        };
        let expected = TypeGroup {
            group_token: token::Group::default(),
            elem: Box::new(parse2(quote!(super::super::T)).unwrap())
        };
        let output = supersuperfy(&Type::Group(orig), 1);
        assert_eq!(quote!(#output).to_string(),
                   quote!(#expected).to_string());
    }

    // Just check that it doesn't panic
    #[test]
    fn infer() {
        check_supersuperfy( quote!(_), quote!(_));
    }

    // Just check that it doesn't panic
    #[test]
    fn never() {
        check_supersuperfy( quote!(!), quote!(!));
    }

    #[test]
    fn paren() {
        check_supersuperfy(
            quote!((super::X)),
            quote!((super::super::X))
        );
    }

    #[test]
    fn path() {
        check_supersuperfy(
            quote!(::super::SuperT<u32>),
            quote!(::super::super::SuperT<u32>)
        );
    }

    #[test]
    fn path_with_qself() {
        check_supersuperfy(
            quote!(<super::X as super::Y>::Foo<u32>),
            quote!(<super::super::X as super::super::Y>::Foo<u32>),
        );
    }

    #[test]
    fn angle_bracketed_generic_arguments() {
        check_supersuperfy(
            quote!(mod_::T<super::X>),
            quote!(mod_::T<super::super::X>)
        );
    }

    #[test]
    fn ptr() {
        check_supersuperfy(
            quote!(*const super::X),
            quote!(*const super::super::X)
        );
    }

    #[test]
    fn reference() {
        check_supersuperfy(
            quote!(&'a mut super::X),
            quote!(&'a mut super::super::X)
        );
    }

    #[test]
    fn slice() {
        check_supersuperfy(
            quote!([super::X]),
            quote!([super::super::X])
        );
    }

    #[test]
    fn trait_object() {
        check_supersuperfy(
            quote!(dyn super::X + super::Y),
            quote!(dyn super::super::X + super::super::Y)
        );
    }

    #[test]
    fn tuple() {
        check_supersuperfy(
            quote!((super::A, super::B)),
            quote!((super::super::A, super::super::B))
        );
    }
}

mod supersuperfy_generics {
    use super::*;

    fn check_supersuperfy_generics(
        orig: TokenStream,
        orig_wc: TokenStream,
        expected: TokenStream,
        expected_wc: TokenStream)
    {
        let mut orig_g: Generics = parse2(orig).unwrap();
        orig_g.where_clause = parse2(orig_wc).unwrap();
        let mut expected_g: Generics = parse2(expected).unwrap();
        expected_g.where_clause = parse2(expected_wc).unwrap();
        let mut output: Generics = orig_g;
        supersuperfy_generics(&mut output, 1);
        let (o_ig, o_tg, o_wc) = output.split_for_impl();
        let (e_ig, e_tg, e_wc) = expected_g.split_for_impl();
        assert_eq!(quote!(#o_ig).to_string(), quote!(#e_ig).to_string());
        assert_eq!(quote!(#o_tg).to_string(), quote!(#e_tg).to_string());
        assert_eq!(quote!(#o_wc).to_string(), quote!(#e_wc).to_string());
    }

    #[test]
    fn default() {
        check_supersuperfy_generics(
            quote!(<T: X = super::Y>), quote!(),
            quote!(<T: X = super::super::Y>), quote!(),
        );
    }

    #[test]
    fn empty() {
        check_supersuperfy_generics(quote!(), quote!(), quote!(), quote!());
    }

    #[test]
    fn everything() {
        check_supersuperfy_generics(
            quote!(<T: super::A = super::B>),
            quote!(where super::C: super::D),
            quote!(<T: super::super::A = super::super::B>),
            quote!(where super::super::C: super::super::D),
        );
    }

    #[test]
    fn bound() {
        check_supersuperfy_generics(
            quote!(<T: super::A>), quote!(),
            quote!(<T: super::super::A>), quote!(),
        );
    }

    #[test]
    fn closure() {
        check_supersuperfy_generics(
            quote!(<F: Fn(u32) -> super::SuperT>), quote!(),
            quote!(<F: Fn(u32) -> super::super::SuperT>), quote!(),
        );
    }

    #[test]
    fn wc_bounded_ty() {
        check_supersuperfy_generics(
            quote!(), quote!(where super::T: X),
            quote!(), quote!(where super::super::T: X),
        );
    }

    #[test]
    fn wc_bounds() {
        check_supersuperfy_generics(
            quote!(), quote!(where T: super::X),
            quote!(), quote!(where T: super::super::X),
        );
    }
}
}
