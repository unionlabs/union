/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use syn::ext::IdentExt;

use crate::bindgen::ir::ty::{IntKind, PrimitiveType};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReprStyle {
    Rust,
    C,
    Transparent,
}

impl Default for ReprStyle {
    fn default() -> Self {
        ReprStyle::Rust
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ReprType {
    kind: IntKind,
    signed: bool,
}

impl ReprType {
    pub(crate) fn to_primitive(self) -> PrimitiveType {
        PrimitiveType::Integer {
            kind: self.kind,
            signed: self.signed,
            zeroable: true,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ReprAlign {
    Packed,
    Align(u64),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Repr {
    pub style: ReprStyle,
    pub ty: Option<ReprType>,
    pub align: Option<ReprAlign>,
}

impl Repr {
    pub fn load(attrs: &[syn::Attribute]) -> Result<Repr, String> {
        let ids = attrs
            .iter()
            .filter_map(|attr| {
                if let syn::Meta::List(syn::MetaList { path, nested, .. }) =
                    attr.parse_meta().ok()?
                {
                    if path.is_ident("repr") {
                        return Some(nested.into_iter().collect::<Vec<_>>());
                    }
                }
                None
            })
            .flatten()
            .filter_map(|meta| match meta {
                syn::NestedMeta::Meta(syn::Meta::Path(path)) => Some((
                    path.segments.first().unwrap().ident.unraw().to_string(),
                    None,
                )),
                syn::NestedMeta::Meta(syn::Meta::List(syn::MetaList { path, nested, .. })) => {
                    Some((
                        path.segments.first().unwrap().ident.unraw().to_string(),
                        Some(
                            nested
                                .iter()
                                .filter_map(|meta| match meta {
                                    // Only used for #[repr(align(...))].
                                    syn::NestedMeta::Lit(syn::Lit::Int(literal)) => {
                                        Some(literal.base10_digits().to_string())
                                    }
                                    // Only single levels of nesting supported at the moment.
                                    _ => None,
                                })
                                .collect::<Vec<_>>(),
                        ),
                    ))
                }
                _ => None,
            });

        let mut repr = Repr::default();
        for id in ids {
            let (int_kind, signed) = match (id.0.as_ref(), id.1) {
                ("u8", None) => (IntKind::B8, false),
                ("u16", None) => (IntKind::B16, false),
                ("u32", None) => (IntKind::B32, false),
                ("u64", None) => (IntKind::B64, false),
                ("usize", None) => (IntKind::Size, false),
                ("i8", None) => (IntKind::B8, true),
                ("i16", None) => (IntKind::B16, true),
                ("i32", None) => (IntKind::B32, true),
                ("i64", None) => (IntKind::B64, true),
                ("isize", None) => (IntKind::Size, true),
                ("C", None) => {
                    repr.style = ReprStyle::C;
                    continue;
                }
                ("transparent", None) => {
                    repr.style = ReprStyle::Transparent;
                    continue;
                }
                ("packed", args) => {
                    // #[repr(packed(n))] not supported because of some open questions about how
                    // to calculate the native alignment of types. See eqrion/cbindgen#433.
                    if args.is_some() {
                        return Err(
                            "Not-yet-implemented #[repr(packed(...))] encountered.".to_string()
                        );
                    }
                    let align = ReprAlign::Packed;
                    // Only permit a single alignment-setting repr.
                    if let Some(old_align) = repr.align {
                        return Err(format!(
                            "Conflicting #[repr(align(...))] type hints {:?} and {:?}.",
                            old_align, align
                        ));
                    }
                    repr.align = Some(align);
                    continue;
                }
                ("align", Some(args)) => {
                    // #[repr(align(...))] only allows a single argument.
                    if args.len() != 1 {
                        return Err(format!(
                            "Unsupported #[repr(align({}))], align must have exactly one argument.",
                            args.join(", ")
                        ));
                    }
                    // Must be a positive integer.
                    let align = match args.first().unwrap().parse::<u64>() {
                        Ok(align) => align,
                        Err(_) => {
                            return Err(format!("Non-numeric #[repr(align({}))].", args.join(", ")))
                        }
                    };
                    // Must be a power of 2.
                    if !align.is_power_of_two() || align == 0 {
                        return Err(format!("Invalid alignment to #[repr(align({}))].", align));
                    }
                    // Only permit a single alignment-setting repr.
                    if let Some(old_align) = repr.align {
                        return Err(format!(
                            "Conflicting #[repr(align(...))] type hints {:?} and {:?}.",
                            old_align,
                            ReprAlign::Align(align)
                        ));
                    }
                    repr.align = Some(ReprAlign::Align(align));
                    continue;
                }
                (path, args) => match args {
                    None => return Err(format!("Unsupported #[repr({})].", path)),
                    Some(args) => {
                        return Err(format!(
                            "Unsupported #[repr({}({}))].",
                            path,
                            args.join(", ")
                        ));
                    }
                },
            };
            let ty = ReprType {
                kind: int_kind,
                signed,
            };
            if let Some(old_ty) = repr.ty {
                return Err(format!(
                    "Conflicting #[repr(...)] type hints {:?} and {:?}.",
                    old_ty, ty
                ));
            }
            repr.ty = Some(ty);
        }
        Ok(repr)
    }
}
