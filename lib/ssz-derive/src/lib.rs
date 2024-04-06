#![recursion_limit = "256"]
//! Provides procedural derive macros for the `Encode` and `Decode` traits of the `eth2_ssz` crate.
//!
//! ## Attributes
//!
//! The following struct/enum attributes are available:
//!
//! - `#[ssz(enum_behaviour = "tag")]`: encodes and decodes an `enum` with 0 fields per variant
//! - `#[ssz(enum_behaviour = "union")]`: encodes and decodes an `enum` with a one-byte variant selector.
//! - `#[ssz(enum_behaviour = "transparent")]`: allows encoding an `enum` by serializing only the
//!     value whilst ignoring outermost the `enum`.
//! - `#[ssz(struct_behaviour = "container")]`: encodes and decodes the `struct` as an SSZ
//!     "container".
//! - `#[ssz(struct_behaviour = "transparent")]`: encodes and decodes a `struct` with exactly one
//!     non-skipped field as if the outermost `struct` does not exist.
//!
//! The following field attributes are available:
//!
//! - `#[ssz(with = "module")]`: uses the methods in `module` to implement `ssz::Encode` and
//!     `ssz::Decode`. This is useful when it's not possible to create an `impl` for that type
//!     (e.g. the type is defined in another crate).
//! - `#[ssz(skip_serializing)]`: this field will not be included in the serialized SSZ vector.
//! - `#[ssz(skip_deserializing)]`: this field will not be expected in the serialized
//!   SSZ vector and it will be initialized from a `Default` implementation.
//!
//! ## Examples
//!
//! ### Structs
//!
//! ```rust
//! use ssz::{Encode, Decode};
//! use ssz::types::{typenum::U4, VariableList};
//!
//! /// Represented as an SSZ "list" wrapped in an SSZ "container".
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! #[ssz(struct_behaviour = "container")]   // "container" is the default behaviour
//! struct TypicalStruct {
//!     foo: VariableList<u8, U4>
//! }
//!
//! assert_eq!(
//!     TypicalStruct { foo: vec![42].try_into().unwrap() }.as_ssz_bytes(),
//!     vec![4, 0, 0, 0, 42]
//! );
//!
//! assert_eq!(
//!     TypicalStruct::from_ssz_bytes(&[4, 0, 0, 0, 42]).unwrap(),
//!     TypicalStruct { foo: vec![42].try_into().unwrap() },
//! );
//!
//! /// Represented as an SSZ "list" *without* an SSZ "container".
//! #[derive(Encode, Decode)]
//! #[ssz(struct_behaviour = "transparent")]
//! struct WrapperStruct {
//!     foo: VariableList<u8, U4>
//! }
//!
//! assert_eq!(
//!     WrapperStruct { foo: vec![42].try_into().unwrap() }.as_ssz_bytes(),
//!     vec![42]
//! );
//!
//! /// Represented as an SSZ "list" *without* an SSZ "container". The `bar` byte is ignored.
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! #[ssz(struct_behaviour = "transparent")]
//! struct WrapperStructSkippedField {
//!     foo: VariableList<u8, U4>,
//!     #[ssz(skip_serializing, skip_deserializing)]
//!     bar: u8,
//! }
//!
//! assert_eq!(
//!     WrapperStructSkippedField { foo: vec![42].try_into().unwrap(), bar: 99 }.as_ssz_bytes(),
//!     vec![42]
//! );
//! assert_eq!(
//!     WrapperStructSkippedField::from_ssz_bytes(&[42]).unwrap(),
//!     WrapperStructSkippedField { foo: vec![42].try_into().unwrap(), bar: 0 }
//! );
//!
//! /// Represented as an SSZ "list" *without* an SSZ "container".
//! #[derive(Encode, Decode)]
//! #[ssz(struct_behaviour = "transparent")]
//! struct NewType(VariableList<u8, U4>);
//!
//! assert_eq!(
//!     NewType(vec![42].try_into().unwrap()).as_ssz_bytes(),
//!     vec![42]
//! );
//!
//! /// Represented as an SSZ "list" *without* an SSZ "container". The `bar` byte is ignored.
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! #[ssz(struct_behaviour = "transparent")]
//! struct NewTypeSkippedField(VariableList<u8, U4>, #[ssz(skip_serializing, skip_deserializing)] u8);
//!
//! assert_eq!(
//!     NewTypeSkippedField(vec![42].try_into().unwrap(), 99).as_ssz_bytes(),
//!     vec![42]
//! );
//! assert_eq!(
//!     NewTypeSkippedField::from_ssz_bytes(&[42]).unwrap(),
//!     NewTypeSkippedField(vec![42].try_into().unwrap(), 0)
//! );
//! ```
//!
//! ### Enums
//!
//! ```rust
//! use ssz::{Encode, Decode};
//! use ssz::types::{typenum::U4, VariableList};
//!
//! /// Represented as an SSZ "union".
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! #[ssz(enum_behaviour = "union")]
//! enum UnionEnum {
//!     Foo(u8),
//!     Bar(VariableList<u8, U4>),
//! }
//!
//! assert_eq!(
//!     UnionEnum::Foo(42).as_ssz_bytes(),
//!     vec![0, 42]
//! );
//! assert_eq!(
//!     UnionEnum::from_ssz_bytes(&[1, 42, 42]).unwrap(),
//!     UnionEnum::Bar(vec![42, 42].try_into().unwrap()),
//! );
//!
//! /// Represented as only the value in the enum variant.
//! #[derive(Debug, PartialEq, Encode)]
//! #[ssz(enum_behaviour = "transparent")]
//! enum TransparentEnum {
//!     Foo(u8),
//!     Bar(VariableList<u8, U4>),
//! }
//!
//! assert_eq!(
//!     TransparentEnum::Foo(42).as_ssz_bytes(),
//!     vec![42]
//! );
//! assert_eq!(
//!     TransparentEnum::Bar(vec![42, 42].try_into().unwrap()).as_ssz_bytes(),
//!     vec![42, 42]
//! );
//!
//! /// Represented as an SSZ "uint8"
//! #[derive(Debug, PartialEq, Encode, Decode)]
//! #[ssz(enum_behaviour = "tag")]
//! enum TagEnum {
//!     Foo,
//!     Bar,
//! }
//! assert_eq!(
//!    TagEnum::Foo.as_ssz_bytes(),
//!    vec![0]
//! );
//! assert_eq!(
//!    TagEnum::from_ssz_bytes(&[1]).unwrap(),
//!    TagEnum::Bar,
//! );
//! ```

use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DataEnum, DataStruct, DeriveInput, Ident, Index};

/// The highest possible union selector value (higher values are reserved for backwards compatible
/// extensions).
const MAX_UNION_SELECTOR: u8 = 127;

const ENUM_TRANSPARENT: &str = "transparent";
const ENUM_UNION: &str = "union";
const ENUM_TAG: &str = "tag";
const NO_ENUM_BEHAVIOUR_ERROR: &str = "enums require an \"enum_behaviour\" attribute with \
    a \"transparent\", \"union\", or \"tag\" value, e.g., #[ssz(enum_behaviour = \"transparent\")]";

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(ssz))]
struct StructOpts {
    #[darling(default)]
    enum_behaviour: Option<String>,
    #[darling(default)]
    struct_behaviour: Option<String>,
}

/// Field-level configuration.
#[derive(Debug, Default, FromMeta)]
struct FieldOpts {
    #[darling(default)]
    with: Option<Ident>,
    #[darling(default)]
    skip_serializing: bool,
    #[darling(default)]
    skip_deserializing: bool,
}

enum Procedure<'a> {
    Struct {
        data: &'a syn::DataStruct,
        behaviour: StructBehaviour,
    },
    Enum {
        data: &'a syn::DataEnum,
        behaviour: EnumBehaviour,
    },
}

enum StructBehaviour {
    Container,
    Transparent,
}

enum EnumBehaviour {
    Union,
    Transparent,
    Tag,
}

impl<'a> Procedure<'a> {
    fn read(item: &'a DeriveInput) -> Self {
        let opts = StructOpts::from_derive_input(item).unwrap();

        match &item.data {
            syn::Data::Struct(data) => {
                if opts.enum_behaviour.is_some() {
                    panic!("cannot use \"enum_behaviour\" for a struct");
                }

                match opts.struct_behaviour.as_deref() {
                    Some("container") | None => Procedure::Struct {
                        data,
                        behaviour: StructBehaviour::Container,
                    },
                    Some("transparent") => Procedure::Struct {
                        data,
                        behaviour: StructBehaviour::Transparent,
                    },
                    Some(other) => panic!(
                        "{} is not a valid struct behaviour, use \"container\" or \"transparent\"",
                        other
                    ),
                }
            }
            syn::Data::Enum(data) => {
                if opts.struct_behaviour.is_some() {
                    panic!("cannot use \"struct_behaviour\" for an enum");
                }

                match opts.enum_behaviour.as_deref() {
                    Some("union") => Procedure::Enum {
                        data,
                        behaviour: EnumBehaviour::Union,
                    },
                    Some("transparent") => Procedure::Enum {
                        data,
                        behaviour: EnumBehaviour::Transparent,
                    },
                    Some("tag") => Procedure::Enum {
                        data,
                        behaviour: EnumBehaviour::Tag,
                    },
                    Some(other) => panic!(
                        "{} is not a valid enum behaviour, use \"container\" or \"transparent\"",
                        other
                    ),
                    None => panic!("{}", NO_ENUM_BEHAVIOUR_ERROR),
                }
            }
            _ => panic!("ssz_derive only supports structs and enums"),
        }
    }
}

fn parse_ssz_fields(
    struct_data: &syn::DataStruct,
) -> Vec<(&syn::Type, Option<&syn::Ident>, FieldOpts)> {
    struct_data
        .fields
        .iter()
        .map(|field| {
            let ty = &field.ty;
            let ident = field.ident.as_ref();

            let field_opts_candidates = field
                .attrs
                .iter()
                .filter(|attr| attr.path.get_ident().map_or(false, |ident| *ident == "ssz"))
                .collect::<Vec<_>>();

            if field_opts_candidates.len() > 1 {
                panic!("more than one field-level \"ssz\" attribute provided")
            }

            let field_opts = field_opts_candidates
                .first()
                .map(|attr| {
                    let meta = attr.parse_meta().unwrap();
                    FieldOpts::from_meta(&meta).unwrap()
                })
                .unwrap_or_default();

            (ty, ident, field_opts)
        })
        .collect()
}

/// Implements `ssz::Encode` for some `struct` or `enum`.
#[proc_macro_derive(Encode, attributes(ssz))]
pub fn ssz_encode_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let procedure = Procedure::read(&item);

    match procedure {
        Procedure::Struct { data, behaviour } => match behaviour {
            StructBehaviour::Transparent => ssz_encode_derive_struct_transparent(&item, data),
            StructBehaviour::Container => ssz_encode_derive_struct(&item, data),
        },
        Procedure::Enum { data, behaviour } => match behaviour {
            EnumBehaviour::Transparent => ssz_encode_derive_enum_transparent(&item, data),
            EnumBehaviour::Union => ssz_encode_derive_enum_union(&item, data),
            EnumBehaviour::Tag => ssz_encode_derive_enum_tag(&item, data),
        },
    }
}

/// Derive `ssz::Encode` for a struct.
///
/// Fields are encoded in the order they are defined.
///
/// ## Field attributes
///
/// - `#[ssz(skip_serializing)]`: the field will not be serialized.
fn ssz_encode_derive_struct(derive_input: &DeriveInput, struct_data: &DataStruct) -> TokenStream {
    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = &derive_input.generics.split_for_impl();

    let field_is_ssz_fixed_len = &mut vec![];
    let field_fixed_len = &mut vec![];
    let field_ssz_bytes_len = &mut vec![];
    let field_encoder_append = &mut vec![];

    for (ty, ident, field_opts) in parse_ssz_fields(struct_data) {
        if field_opts.skip_serializing {
            continue;
        }

        let ident = match ident {
            Some(ref ident) => ident,
            _ => panic!(
                "#[ssz(struct_behaviour = \"container\")] only supports named struct fields."
            ),
        };

        if let Some(module) = field_opts.with {
            let module = quote! { #module::encode };
            field_is_ssz_fixed_len.push(quote! { #module::is_ssz_fixed_len() });
            field_fixed_len.push(quote! { #module::ssz_fixed_len() });
            field_ssz_bytes_len.push(quote! { #module::ssz_bytes_len(&self.#ident) });
            field_encoder_append.push(quote! {
                encoder.append_parameterized(
                    #module::is_ssz_fixed_len(),
                    |buf| #module::ssz_append(&self.#ident, buf)
                )
            });
        } else {
            field_is_ssz_fixed_len.push(quote! { <#ty as ssz::Encode>::is_ssz_fixed_len() });
            field_fixed_len.push(quote! { <#ty as ssz::Encode>::ssz_fixed_len() });
            field_ssz_bytes_len.push(quote! { self.#ident.ssz_bytes_len() });
            field_encoder_append.push(quote! { encoder.append(&self.#ident) });
        }
    }

    let output = quote! {
        impl #impl_generics ssz::Encode for #name #ty_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
                #(
                    #field_is_ssz_fixed_len &&
                )*
                    true
            }

            fn ssz_fixed_len() -> usize {
                if <Self as ssz::Encode>::is_ssz_fixed_len() {
                    let mut len: usize = 0;
                    #(
                        len = len
                            .checked_add(#field_fixed_len)
                            .expect("encode ssz_fixed_len length overflow");
                    )*
                    len
                } else {
                    ssz::BYTES_PER_LENGTH_OFFSET
                }
            }

            fn ssz_bytes_len(&self) -> usize {
                if <Self as ssz::Encode>::is_ssz_fixed_len() {
                    <Self as ssz::Encode>::ssz_fixed_len()
                } else {
                    let mut len: usize = 0;
                    #(
                        if #field_is_ssz_fixed_len {
                            len = len
                                .checked_add(#field_fixed_len)
                                .expect("encode ssz_bytes_len length overflow");
                        } else {
                            len = len
                                .checked_add(ssz::BYTES_PER_LENGTH_OFFSET)
                                .expect("encode ssz_bytes_len length overflow for offset");
                            len = len
                                .checked_add(#field_ssz_bytes_len)
                                .expect("encode ssz_bytes_len length overflow for bytes");
                        }
                    )*

                    len
                }
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                let mut offset: usize = 0;
                #(
                    offset = offset
                        .checked_add(#field_fixed_len)
                        .expect("encode ssz_append offset overflow");
                )*

                let mut encoder = ssz::SszEncoder::container(buf, offset);

                #(
                    #field_encoder_append;
                )*

                encoder.finalize();
            }
        }
    };
    output.into()
}

/// Derive `ssz::Encode` "transparently" for a struct which has exactly one non-skipped field.
///
/// The single field is encoded directly, making the outermost `struct` transparent.
///
/// ## Field attributes
///
/// - `#[ssz(skip_serializing)]`: the field will not be serialized.
fn ssz_encode_derive_struct_transparent(
    derive_input: &DeriveInput,
    struct_data: &DataStruct,
) -> TokenStream {
    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = &derive_input.generics.split_for_impl();
    let ssz_fields = parse_ssz_fields(struct_data);
    let num_fields = ssz_fields
        .iter()
        .filter(|(_, _, field_opts)| !field_opts.skip_deserializing)
        .count();

    if num_fields != 1 {
        panic!(
            "A \"transparent\" struct must have exactly one non-skipped field ({} fields found)",
            num_fields
        );
    }

    let (index, (ty, ident, _field_opts)) = ssz_fields
        .iter()
        .enumerate()
        .find(|(_, (_, _, field_opts))| !field_opts.skip_deserializing)
        .expect("\"transparent\" struct must have at least one non-skipped field");

    // Remove the `_usize` suffix from the value to avoid a compiler warning.
    let index = Index::from(index);

    let output = if let Some(field_name) = ident {
        quote! {
            impl #impl_generics ssz::Encode for #name #ty_generics #where_clause {
                fn is_ssz_fixed_len() -> bool {
                    <#ty as ssz::Encode>::is_ssz_fixed_len()
                }

                fn ssz_fixed_len() -> usize {
                    <#ty as ssz::Encode>::ssz_fixed_len()
                }

                fn ssz_bytes_len(&self) -> usize {
                    self.#field_name.ssz_bytes_len()
                }

                fn ssz_append(&self, buf: &mut Vec<u8>) {
                    self.#field_name.ssz_append(buf)
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics ssz::Encode for #name #ty_generics #where_clause {
                fn is_ssz_fixed_len() -> bool {
                    <#ty as ssz::Encode>::is_ssz_fixed_len()
                }

                fn ssz_fixed_len() -> usize {
                    <#ty as ssz::Encode>::ssz_fixed_len()
                }

                fn ssz_bytes_len(&self) -> usize {
                    self.#index.ssz_bytes_len()
                }

                fn ssz_append(&self, buf: &mut Vec<u8>) {
                    self.#index.ssz_append(buf)
                }
            }
        }
    };

    output.into()
}

/// Derive `ssz::Encode` for an enum in the "transparent" method.
///
/// The "transparent" method is distinct from the "union" method specified in the SSZ specification.
/// When using "transparent", the enum will be ignored and the contained field will be serialized as
/// if the enum does not exist. Since an union variant "selector" is not serialized, it is not
/// possible to reliably decode an enum that is serialized transparently.
///
/// ## Limitations
///
/// Only supports:
/// - Enums with a single field per variant, where
///     - All fields are variably sized from an SSZ-perspective (not fixed size).
///
/// ## Panics
///
/// Will panic at compile-time if the single field requirement isn't met, but will panic *at run
/// time* if the variable-size requirement isn't met.
fn ssz_encode_derive_enum_transparent(
    derive_input: &DeriveInput,
    enum_data: &DataEnum,
) -> TokenStream {
    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = &derive_input.generics.split_for_impl();

    let (patterns, assert_exprs): (Vec<_>, Vec<_>) = enum_data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;

            if variant.fields.len() != 1 {
                panic!("ssz::Encode can only be derived for enums with 1 field per variant");
            }

            let pattern = quote! {
                #name::#variant_name(ref inner)
            };

            let ty = &(&variant.fields).into_iter().next().unwrap().ty;
            let type_assert = quote! {
                !<#ty as ssz::Encode>::is_ssz_fixed_len()
            };
            (pattern, type_assert)
        })
        .unzip();

    let output = quote! {
        impl #impl_generics ssz::Encode for #name #ty_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
                assert!(
                    #(
                        #assert_exprs &&
                    )* true,
                    "not all enum variants are variably-sized"
                );
                false
            }

            fn ssz_bytes_len(&self) -> usize {
                match self {
                    #(
                        #patterns => inner.ssz_bytes_len(),
                    )*
                }
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                match self {
                    #(
                        #patterns => inner.ssz_append(buf),
                    )*
                }
            }
        }
    };
    output.into()
}

/// Derive `ssz::Encode` for an `enum` following the "tag" method.
///
/// The union selector will be determined based upon the order in which the enum variants are
/// defined. E.g., the top-most variant in the enum will have a selector of `0`, the variant
/// beneath it will have a selector of `1` and so on.
///
/// # Limitations
///
/// Only supports enums where each variant has no fields
fn ssz_encode_derive_enum_tag(derive_input: &DeriveInput, enum_data: &DataEnum) -> TokenStream {
    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = &derive_input.generics.split_for_impl();

    let patterns: Vec<_> = enum_data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;

            if !variant.fields.is_empty() {
                panic!("ssz::Encode tag behaviour can only be derived for enums with no fields");
            }

            quote! {
                #name::#variant_name
            }
        })
        .collect();

    let union_selectors = compute_union_selectors(patterns.len());

    let output = quote! {
        impl #impl_generics ssz::Encode for #name #ty_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                1
            }

            fn ssz_bytes_len(&self) -> usize {
                1
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                match self {
                    #(
                        #patterns => {
                            let union_selector: u8 = #union_selectors;
                            debug_assert!(union_selector <= ssz::MAX_UNION_SELECTOR);
                            buf.push(union_selector);
                        },
                    )*
                }
            }
        }
    };
    output.into()
}

/// Derive `ssz::Encode` for an `enum` following the "union" SSZ spec.
///
/// The union selector will be determined based upon the order in which the enum variants are
/// defined. E.g., the top-most variant in the enum will have a selector of `0`, the variant
/// beneath it will have a selector of `1` and so on.
///
/// # Limitations
///
/// Only supports enums where each variant has a single field.
fn ssz_encode_derive_enum_union(derive_input: &DeriveInput, enum_data: &DataEnum) -> TokenStream {
    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = &derive_input.generics.split_for_impl();

    let patterns: Vec<_> = enum_data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;

            if variant.fields.len() != 1 {
                panic!("ssz::Encode can only be derived for enums with 1 field per variant");
            }

            let pattern = quote! {
                #name::#variant_name(ref inner)
            };
            pattern
        })
        .collect();

    let union_selectors = compute_union_selectors(patterns.len());

    let output = quote! {
        impl #impl_generics ssz::Encode for #name #ty_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
                false
            }

            fn ssz_bytes_len(&self) -> usize {
                match self {
                    #(
                        #patterns => inner
                            .ssz_bytes_len()
                            .checked_add(1)
                            .expect("encoded length must be less than usize::max_value"),
                    )*
                }
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                match self {
                    #(
                        #patterns => {
                            let union_selector: u8 = #union_selectors;
                            debug_assert!(union_selector <= ssz::MAX_UNION_SELECTOR);
                            buf.push(union_selector);
                            inner.ssz_append(buf)
                        },
                    )*
                }
            }
        }
    };
    output.into()
}

/// Derive `ssz::Decode` for a struct or enum.
#[proc_macro_derive(Decode, attributes(ssz))]
pub fn ssz_decode_derive(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let procedure = Procedure::read(&item);

    match procedure {
        Procedure::Struct { data, behaviour } => match behaviour {
            StructBehaviour::Transparent => ssz_decode_derive_struct_transparent(&item, data),
            StructBehaviour::Container => ssz_decode_derive_struct(&item, data),
        },
        Procedure::Enum { data, behaviour } => match behaviour {
            EnumBehaviour::Union => ssz_decode_derive_enum_union(&item, data),
            EnumBehaviour::Tag => ssz_decode_derive_enum_tag(&item, data),
            EnumBehaviour::Transparent => panic!(
                "Decode cannot be derived for enum_behaviour \"{}\", only \"{}\" and \"{}\" is valid.",
                ENUM_TRANSPARENT, ENUM_UNION, ENUM_TAG,
            ),
        },
    }
}

/// Implements `ssz::Decode` for some `struct`.
///
/// Fields are decoded in the order they are defined.
///
/// ## Field attributes
///
/// - `#[ssz(skip_deserializing)]`: during de-serialization the field will be instantiated from a
/// `Default` implementation. The decoder will assume that the field was not serialized at all
/// (e.g., if it has been serialized, an error will be raised instead of `Default` overriding it).
fn ssz_decode_derive_struct(item: &DeriveInput, struct_data: &DataStruct) -> TokenStream {
    let name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = &item.generics.split_for_impl();

    let mut register_types = vec![];
    let mut field_names = vec![];
    let mut fixed_decodes = vec![];
    let mut decodes = vec![];
    let mut is_fixed_lens = vec![];
    let mut fixed_lens = vec![];

    for (ty, ident, field_opts) in parse_ssz_fields(struct_data) {
        let ident = match ident {
            Some(ref ident) => ident,
            _ => panic!(
                "#[ssz(struct_behaviour = \"container\")] only supports named struct fields."
            ),
        };

        field_names.push(quote! {
            #ident
        });

        // Field should not be deserialized; use a `Default` impl to instantiate.
        if field_opts.skip_deserializing {
            decodes.push(quote! {
                let #ident = <_>::default();
            });

            fixed_decodes.push(quote! {
                let #ident = <_>::default();
            });

            continue;
        }

        let is_ssz_fixed_len;
        let ssz_fixed_len;
        let from_ssz_bytes;
        if let Some(module) = field_opts.with {
            let module = quote! { #module::decode };

            is_ssz_fixed_len = quote! { #module::is_ssz_fixed_len() };
            ssz_fixed_len = quote! { #module::ssz_fixed_len() };
            from_ssz_bytes = quote! { #module::from_ssz_bytes(slice) };

            register_types.push(quote! {
                builder.register_type_parameterized(#is_ssz_fixed_len, #ssz_fixed_len)?;
            });
            decodes.push(quote! {
                let #ident = decoder.decode_next_with(|slice| #module::from_ssz_bytes(slice))?;
            });
        } else {
            is_ssz_fixed_len = quote! { <#ty as ssz::Decode>::is_ssz_fixed_len() };
            ssz_fixed_len = quote! { <#ty as ssz::Decode>::ssz_fixed_len() };
            from_ssz_bytes = quote! { <#ty as ssz::Decode>::from_ssz_bytes(slice) };

            register_types.push(quote! {
                builder.register_type::<#ty>()?;
            });
            decodes.push(quote! {
                let #ident = decoder.decode_next()?;
            });
        }

        fixed_decodes.push(quote! {
            let #ident = {
                start = end;
                end = end
                    .checked_add(#ssz_fixed_len)
                    .ok_or_else(|| ssz::DecodeError::OutOfBoundsByte {
                        i: usize::max_value()
                    })?;
                let slice = bytes.get(start..end)
                    .ok_or_else(|| ssz::DecodeError::InvalidByteLength {
                        len: bytes.len(),
                        expected: end
                    })?;
                #from_ssz_bytes?
            };
        });
        is_fixed_lens.push(is_ssz_fixed_len);
        fixed_lens.push(ssz_fixed_len);
    }

    let output = quote! {
        impl #impl_generics ssz::Decode for #name #ty_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
                #(
                    #is_fixed_lens &&
                )*
                    true
            }

            fn ssz_fixed_len() -> usize {
                if <Self as ssz::Decode>::is_ssz_fixed_len() {
                    let mut len: usize = 0;
                    #(
                        len = len
                            .checked_add(#fixed_lens)
                            .expect("decode ssz_fixed_len overflow");
                    )*
                    len
                } else {
                    ssz::BYTES_PER_LENGTH_OFFSET
                }
            }

            fn from_ssz_bytes(bytes: &[u8]) -> std::result::Result<Self, ssz::DecodeError> {
                if <Self as ssz::Decode>::is_ssz_fixed_len() {
                    if bytes.len() != <Self as ssz::Decode>::ssz_fixed_len() {
                        return Err(ssz::DecodeError::InvalidByteLength {
                            len: bytes.len(),
                            expected: <Self as ssz::Decode>::ssz_fixed_len(),
                        });
                    }

                    let mut start: usize = 0;
                    let mut end = start;

                    #(
                        #fixed_decodes
                    )*

                    Ok(Self {
                        #(
                            #field_names,
                        )*
                    })
                } else {
                    let mut builder = ssz::SszDecoderBuilder::new(bytes);

                    #(
                        #register_types
                    )*

                    let mut decoder = builder.build()?;

                    #(
                        #decodes
                    )*


                    Ok(Self {
                        #(
                            #field_names,
                        )*
                    })
                }
            }
        }
    };
    output.into()
}

/// Implements `ssz::Decode` "transparently" for a `struct` with exactly one non-skipped field.
///
/// The bytes will be decoded as if they are the inner field, without the outermost struct. The
/// outermost struct will then be applied artificially.
///
/// ## Field attributes
///
/// - `#[ssz(skip_deserializing)]`: during de-serialization the field will be instantiated from a
/// `Default` implementation. The decoder will assume that the field was not serialized at all
/// (e.g., if it has been serialized, an error will be raised instead of `Default` overriding it).
fn ssz_decode_derive_struct_transparent(
    item: &DeriveInput,
    struct_data: &DataStruct,
) -> TokenStream {
    let name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = &item.generics.split_for_impl();
    let ssz_fields = parse_ssz_fields(struct_data);
    let num_fields = ssz_fields
        .iter()
        .filter(|(_, _, field_opts)| !field_opts.skip_deserializing)
        .count();

    if num_fields != 1 {
        panic!(
            "A \"transparent\" struct must have exactly one non-skipped field ({} fields found)",
            num_fields
        );
    }

    let mut fields = vec![];
    let mut wrapped_type = None;

    for (i, (ty, ident, field_opts)) in ssz_fields.into_iter().enumerate() {
        if let Some(name) = ident {
            if field_opts.skip_deserializing {
                fields.push(quote! {
                    #name: <_>::default(),
                });
            } else {
                fields.push(quote! {
                    #name: <_>::from_ssz_bytes(bytes)?,
                });
                wrapped_type = Some(ty);
            }
        } else {
            let index = syn::Index::from(i);
            if field_opts.skip_deserializing {
                fields.push(quote! {
                    #index:<_>::default(),
                });
            } else {
                fields.push(quote! {
                    #index:<_>::from_ssz_bytes(bytes)?,
                });
                wrapped_type = Some(ty);
            }
        }
    }

    let ty = wrapped_type.unwrap();

    let output = quote! {
        impl #impl_generics ssz::Decode for #name #ty_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
                <#ty as ssz::Decode>::is_ssz_fixed_len()
            }

            fn ssz_fixed_len() -> usize {
                <#ty as ssz::Decode>::ssz_fixed_len()
            }

            fn from_ssz_bytes(bytes: &[u8]) -> std::result::Result<Self, ssz::DecodeError> {
                Ok(Self {
                    #(
                        #fields
                    )*

                })
            }
        }
    };
    output.into()
}

/// Derive `ssz::Decode` for an `enum` following the "tag" SSZ spec.
fn ssz_decode_derive_enum_tag(derive_input: &DeriveInput, enum_data: &DataEnum) -> TokenStream {
    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = &derive_input.generics.split_for_impl();

    let patterns: Vec<_> = enum_data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;

            if !variant.fields.is_empty() {
                panic!("ssz::Decode tag behaviour can only be derived for enums with no fields");
            }

            quote! {
                #name::#variant_name
            }
        })
        .collect();

    let union_selectors = compute_union_selectors(patterns.len());

    let output = quote! {
        impl #impl_generics ssz::Decode for #name #ty_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
                true
            }

            fn ssz_fixed_len() -> usize {
                1
            }

            fn from_ssz_bytes(bytes: &[u8]) -> std::result::Result<Self, ssz::DecodeError> {
                let byte = bytes
                    .first()
                    .copied()
                    .ok_or(ssz::DecodeError::OutOfBoundsByte { i: 0 })?;

                match byte {
                    #(
                        #union_selectors => {
                            Ok(#patterns)
                        },
                    )*
                    other => Err(ssz::DecodeError::UnionSelectorInvalid(other)),
                }
            }
        }
    };
    output.into()
}

/// Derive `ssz::Decode` for an `enum` following the "union" SSZ spec.
fn ssz_decode_derive_enum_union(derive_input: &DeriveInput, enum_data: &DataEnum) -> TokenStream {
    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = &derive_input.generics.split_for_impl();

    let (constructors, var_types): (Vec<_>, Vec<_>) = enum_data
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;

            if variant.fields.len() != 1 {
                panic!("ssz::Encode can only be derived for enums with 1 field per variant");
            }

            let constructor = quote! {
                #name::#variant_name
            };

            let ty = &(&variant.fields).into_iter().next().unwrap().ty;
            (constructor, ty)
        })
        .unzip();

    let union_selectors = compute_union_selectors(constructors.len());

    let output = quote! {
        impl #impl_generics ssz::Decode for #name #ty_generics #where_clause {
            fn is_ssz_fixed_len() -> bool {
                false
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ssz::DecodeError> {
                // Sanity check to ensure the definition here does not drift from the one defined in
                // `ssz`.
                debug_assert_eq!(#MAX_UNION_SELECTOR, ssz::MAX_UNION_SELECTOR);

                let (selector, body) = ssz::split_union_bytes(bytes)?;

                match selector.into() {
                    #(
                        #union_selectors => {
                            <#var_types as ssz::Decode>::from_ssz_bytes(body).map(#constructors)
                        },
                    )*
                    other => Err(ssz::DecodeError::UnionSelectorInvalid(other))
                }
            }
        }
    };
    output.into()
}

fn compute_union_selectors(num_variants: usize) -> Vec<u8> {
    let union_selectors = (0..num_variants)
        .map(|i| {
            i.try_into()
                .expect("union selector exceeds u8::max_value, union has too many variants")
        })
        .collect::<Vec<u8>>();

    let highest_selector = union_selectors
        .last()
        .copied()
        .expect("0-variant union is not permitted");

    assert!(
        highest_selector <= MAX_UNION_SELECTOR,
        "union selector {} exceeds limit of {}, enum has too many variants",
        highest_selector,
        MAX_UNION_SELECTOR
    );

    union_selectors
}
