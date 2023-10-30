//! Special types handling

use crate::spanned::Sp;

use syn::{
    spanned::Spanned, GenericArgument, Path, PathArguments, PathArguments::AngleBracketed,
    PathSegment, Type, TypePath,
};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Ty {
    Bool,
    Vec,
    Option,
    OptionOption,
    OptionVec,
    Other,
}

impl Ty {
    pub fn from_syn_ty(ty: &syn::Type) -> Sp<Self> {
        use Ty::*;
        let t = |kind| Sp::new(kind, ty.span());

        if is_simple_ty(ty, "bool") {
            t(Bool)
        } else if is_generic_ty(ty, "Vec") {
            t(Vec)
        } else if let Some(subty) = subty_if_name(ty, "Option") {
            if is_generic_ty(subty, "Option") {
                t(OptionOption)
            } else if is_generic_ty(subty, "Vec") {
                t(OptionVec)
            } else {
                t(Option)
            }
        } else {
            t(Other)
        }
    }
}

pub fn sub_type(ty: &syn::Type) -> Option<&syn::Type> {
    subty_if(ty, |_| true)
}

fn only_last_segment(ty: &syn::Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(TypePath {
            qself: None,
            path:
                Path {
                    leading_colon: None,
                    segments,
                },
        }) => only_one(segments.iter()),

        _ => None,
    }
}

fn subty_if<F>(ty: &syn::Type, f: F) -> Option<&syn::Type>
where
    F: FnOnce(&PathSegment) -> bool,
{
    let ty = strip_group(ty);

    only_last_segment(ty)
        .filter(|segment| f(segment))
        .and_then(|segment| {
            if let AngleBracketed(args) = &segment.arguments {
                only_one(args.args.iter()).and_then(|genneric| {
                    if let GenericArgument::Type(ty) = genneric {
                        Some(ty)
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
}

pub fn subty_if_name<'a>(ty: &'a syn::Type, name: &str) -> Option<&'a syn::Type> {
    subty_if(ty, |seg| seg.ident == name)
}

pub fn is_simple_ty(ty: &syn::Type, name: &str) -> bool {
    let ty = strip_group(ty);

    only_last_segment(ty)
        .map(|segment| {
            if let PathArguments::None = segment.arguments {
                segment.ident == name
            } else {
                false
            }
        })
        .unwrap_or(false)
}

// If the struct is placed inside of a macro_rules! declaration,
// in some circumstances, the tokens inside will be enclosed
// in `proc_macro::Group` delimited by invisible `proc_macro::Delimiter::None`.
//
// In syn speak, this is encoded via `*::Group` variants. We don't really care about
// that, so let's just strip it.
//
// Details: https://doc.rust-lang.org/proc_macro/enum.Delimiter.html#variant.None
// See also: https://github.com/TeXitoi/structopt/issues/439
fn strip_group(mut ty: &syn::Type) -> &syn::Type {
    while let Type::Group(group) = ty {
        ty = &*group.elem;
    }

    ty
}

fn is_generic_ty(ty: &syn::Type, name: &str) -> bool {
    subty_if_name(ty, name).is_some()
}

fn only_one<I, T>(mut iter: I) -> Option<T>
where
    I: Iterator<Item = T>,
{
    iter.next().filter(|_| iter.next().is_none())
}
