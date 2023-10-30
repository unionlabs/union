// vim: tw=80
use super::*;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};

/// A single automock attribute
// This enum is very short-lived, so it's fine not to box it.
#[allow(clippy::large_enum_variant)]
enum Attr {
    Mod(ItemMod),
    Type(TraitItemType),
}

impl Parse for Attr {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![mod]) {
            input.parse().map(Attr::Mod)
        } else if lookahead.peek(Token![type]) {
            input.parse().map(Attr::Type)
        } else {
            Err(lookahead.error())
        }
    }
}

/// automock attributes
#[derive(Debug, Default)]
pub(crate) struct Attrs {
    pub attrs: HashMap<Ident, Type>,
    pub modname: Option<Ident>
}

impl Attrs {
    fn get_path(&self, path: &Path) -> Option<Type> {
        if path.leading_colon.is_none() & (path.segments.len() == 2) {
            if path.segments.first().unwrap().ident == "Self" {
                let ident = &path.segments.last().unwrap().ident;
                self.attrs.get(ident).cloned()
            } else {
                None
            }
        } else {
            None
        }
    }

    pub(crate) fn substitute_item_impl(&self, item_impl: &mut ItemImpl) {
        let (_, trait_path, _) = item_impl.trait_.as_ref()
            .expect("Should only be called for trait item impls");
        let trait_ident = find_ident_from_path(trait_path).0;
        for item in item_impl.items.iter_mut() {
            if let ImplItem::Method(method) = item {
                let sig = &mut method.sig;
                for fn_arg in sig.inputs.iter_mut() {
                    if let FnArg::Typed(arg) = fn_arg {
                        self.substitute_type(&mut arg.ty, &trait_ident);
                    }
                }
                if let ReturnType::Type(_, ref mut ty) = &mut sig.output {
                    self.substitute_type(ty, &trait_ident);
                }
            }
        }
    }

    fn substitute_path_segment(&self, seg: &mut PathSegment, traitname: &Ident)
    {
        match &mut seg.arguments {
            PathArguments::None => /* nothing to do */(),
            PathArguments::Parenthesized(p) => {
                compile_error(p.span(),
                    "Mockall does not support mocking Fn objects.  See https://github.com/asomers/mockall/issues/139");
            },
            PathArguments::AngleBracketed(abga) => {
                for arg in abga.args.iter_mut() {
                    match arg {
                        GenericArgument::Type(ty) => {
                            self.substitute_type(ty, traitname)
                        },
                        GenericArgument::Binding(binding) => {
                            self.substitute_type(&mut binding.ty, traitname);
                        },
                        _ => {
                            /*
                             * Nothing to do, as long as lifetimes can't be
                             * associated types
                             */
                        }
                    }
                }
            },
        }
    }

    /// Recursively substitute types in the input
    fn substitute_type(&self, ty: &mut Type, traitname: &Ident) {
        match ty {
            Type::Slice(s) => {
                self.substitute_type(s.elem.as_mut(), traitname)
            },
            Type::Array(a) => {
                self.substitute_type(a.elem.as_mut(), traitname)
            },
            Type::Ptr(p) => {
                self.substitute_type(p.elem.as_mut(), traitname)
            },
            Type::Reference(r) => {
                self.substitute_type(r.elem.as_mut(), traitname)
            },
            Type::BareFn(bfn) => {
                for fn_arg in bfn.inputs.iter_mut() {
                    self.substitute_type(&mut fn_arg.ty, traitname);
                }
                if let ReturnType::Type(_, ref mut ty) = &mut bfn.output {
                    self.substitute_type(ty, traitname);
                }
            },
            Type::Tuple(tuple) => {
                for elem in tuple.elems.iter_mut() {
                    self.substitute_type(elem, traitname)
                }
            }
            Type::Path(path) => {
                if let Some(ref qself) = path.qself {
                    let qp = if let Type::Path(p) = qself.ty.as_ref() {
                        &p.path
                    } else {
                        panic!("QSelf's type isn't a path?")
                    };
                    let qident = &qp.segments.first().unwrap().ident;
                    if qself.position != 1
                        || qp.segments.len() != 1
                        || path.path.segments.len() != 2
                        || qident != "Self" {
                        compile_error(path.span(),
                            "QSelf is a work in progress");
                    }

                    let mut seg_iter = path.path.segments.iter().rev();
                    let last_seg = seg_iter.next().unwrap();
                    let to_sub = &last_seg.ident;
                    let penultimate_seg = seg_iter.next().unwrap();
                    let qident = &penultimate_seg.ident;
                    drop(seg_iter);

                    if qident != traitname {
                        compile_error(qident.span(),
                            "Mockall does not support QSelf substitutions except for the trait being mocked");
                    }
                    if let Some(new_type) = self.attrs.get(to_sub) {
                        *ty = new_type.clone();
                    } else {
                        compile_error(to_sub.span(),
                            "Unknown type substitution for QSelf");
                    }
                } else if let Some(newty) = self.get_path(&path.path) {
                    *ty = newty;
                } else {
                    for seg in path.path.segments.iter_mut() {
                        self.substitute_path_segment(seg, traitname);
                    }
                }
            },
            Type::TraitObject(to) => {
                for bound in to.bounds.iter_mut() {
                    self.substitute_type_param_bound(bound, traitname);
                }
            },
            Type::ImplTrait(it) => {
                for bound in it.bounds.iter_mut() {
                    self.substitute_type_param_bound(bound, traitname);
                }
            },
            Type::Paren(p) => {
                self.substitute_type(p.elem.as_mut(), traitname)
            },
            Type::Group(g) => {
                self.substitute_type(g.elem.as_mut(), traitname)
            },
            Type::Macro(_) | Type::Verbatim(_) => {
                compile_error(ty.span(),
                    "mockall_derive does not support this type when using associated types");
            },
            Type::Infer(_) | Type::Never(_) => {
                /* Nothing to do */
            },
            _ => compile_error(ty.span(), "Unsupported type"),
        }
    }

    fn substitute_type_param_bound(&self,
                                   bound: &mut TypeParamBound,
                                   traitname: &Ident)
    {
        if let TypeParamBound::Trait(t) = bound {
            match self.get_path(&t.path) {
                None => {
                    for seg in t.path.segments.iter_mut() {
                        self.substitute_path_segment(seg, traitname);
                    }
                },
                Some(Type::Path(type_path)) => {
                    t.path = type_path.path;
                },
                Some(_) => {
                    compile_error(t.path.span(),
                        "Can only substitute paths for trait bounds");
                }
            }
        }
    }

    pub(crate) fn substitute_trait(&self, item: &ItemTrait) -> ItemTrait {
        let mut output = item.clone();
        for trait_item in output.items.iter_mut() {
            match trait_item {
                TraitItem::Type(tity) => {
                    if let Some(ty) = self.attrs.get(&tity.ident) {
                        let span = tity.span();
                        tity.default = Some((Token![=](span), ty.clone()));
                        // Concrete associated types aren't allowed to have
                        // bounds
                        tity.bounds = Punctuated::new();
                    } else {
                        compile_error(tity.span(),
                            "Default value not given for associated type");
                    }
                },
                TraitItem::Method(method) => {
                    let sig = &mut method.sig;
                    for fn_arg in sig.inputs.iter_mut() {
                        if let FnArg::Typed(arg) = fn_arg {
                            self.substitute_type(&mut arg.ty, &item.ident);
                        }
                    }
                    if let ReturnType::Type(_, ref mut ty) = &mut sig.output {
                        self.substitute_type(ty, &item.ident);
                    }
                },
                _ => {
                    // Nothing to do
                }
            }
        }
        output
    }
}

impl Parse for Attrs {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let mut attrs = HashMap::new();
        let mut modname = None;
        while !input.is_empty() {
            let attr: Attr = input.parse()?;
            match attr {
                Attr::Mod(item_mod) => {
                    if let Some((br, _)) = item_mod.content {
                        compile_error(br.span,
                            "mod name attributes must have the form \"mod my_name;\"");
                    }
                    modname = Some(item_mod.ident.clone());
                },
                Attr::Type(trait_item_type) => {
                    let ident = trait_item_type.ident.clone();
                    if let Some((_, ty)) = trait_item_type.default {
                        attrs.insert(ident, ty.clone());
                    } else {
                        compile_error(trait_item_type.span(),
                          "automock type attributes must have a default value");
                    }
                }
            }
        }
        Ok(Attrs{attrs, modname})
    }
}

/// Unit tests for `Attrs`.
#[cfg(test)]
mod t {
    use super::super::*;
    use pretty_assertions::assert_eq;

    fn check_substitute_type(
        attrs: TokenStream,
        input: TokenStream,
        traitname: Ident,
        expected: TokenStream)
    {
        let _self: super::Attrs = parse2(attrs).unwrap();
        let mut in_ty: Type = parse2(input).unwrap();
        let expect_ty: Type = parse2(expected).unwrap();
        _self.substitute_type(&mut in_ty, &traitname);
        assert_eq!(in_ty, expect_ty);
    }

    #[test]
    fn qself() {
        check_substitute_type(quote!(type T = u32;),
                              quote!(<Self as Foo>::T),
                              format_ident!("Foo"),
                              quote!(u32));
    }

    #[test]
    #[should_panic(expected = "Mockall does not support QSelf substitutions except for the trait being mocked")]
    fn qself_other() {
        check_substitute_type(quote!(type T = u32;),
                              quote!(<Self as AsRef>::T),
                              format_ident!("Foo"),
                              quote!(u32));
    }

    #[test]
    #[should_panic(expected = "Unknown type substitution for QSelf")]
    fn unknown_substitution() {
        check_substitute_type(quote!(type T = u32;),
                              quote!(<Self as Foo>::Q),
                              format_ident!("Foo"),
                              quote!(u32));
    }
}
