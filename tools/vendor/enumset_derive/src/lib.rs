#![recursion_limit = "256"]

extern crate proc_macro;

use darling::util::SpannedValue;
use darling::*;
use proc_macro::TokenStream;
use proc_macro2::{Literal, Span, TokenStream as SynTokenStream};
use quote::*;
use std::{collections::HashSet, fmt::Display};
use syn::spanned::Spanned;
use syn::{Error, Result, *};

/// Helper function for emitting compile errors.
fn error<T>(span: Span, message: impl Display) -> Result<T> {
    Err(Error::new(span, message))
}

/// Decodes the custom attributes for our custom derive.
#[derive(FromDeriveInput, Default)]
#[darling(attributes(enumset), default)]
struct EnumsetAttrs {
    no_ops: bool,
    no_super_impls: bool,
    #[darling(default)]
    repr: SpannedValue<Option<String>>,
    #[darling(default)]
    serialize_repr: SpannedValue<Option<String>>,
    serialize_deny_unknown: bool,
    #[darling(default)]
    crate_name: Option<String>,

    // legacy options
    serialize_as_list: SpannedValue<bool>, // replaced with serialize_repr
    serialize_as_map: SpannedValue<bool>,  // replaced with serialize_repr
}

/// The internal representation of an enumset.
#[derive(Copy, Clone)]
enum InternalRepr {
    /// internal repr: `u8`
    U8,
    /// internal repr: `u16`
    U16,
    /// internal repr: `u32`
    U32,
    /// internal repr: `u64`
    U64,
    /// internal repr: `u128`
    U128,
    /// internal repr: `[u64; size]`
    Array(usize),
}
impl InternalRepr {
    /// Determines the number of variants supported by this repr.
    fn supported_variants(&self) -> usize {
        match self {
            InternalRepr::U8 => 8,
            InternalRepr::U16 => 16,
            InternalRepr::U32 => 32,
            InternalRepr::U64 => 64,
            InternalRepr::U128 => 128,
            InternalRepr::Array(size) => size * 64,
        }
    }
}

/// The serde representation of the enumset.
#[derive(Copy, Clone)]
enum SerdeRepr {
    /// serde type: `u8`
    U8,
    /// serde type: `u16`
    U16,
    /// serde type: `u32`
    U32,
    /// serde type: `u64`
    U64,
    /// serde type: `u128`
    U128,
    /// serde type: list of `T`
    List,
    /// serde type: map of `T` to `bool`
    Map,
    /// serde type: list of `u64`
    Array,
}
impl SerdeRepr {
    /// Determines the number of variants supported by this repr.
    fn supported_variants(&self) -> Option<usize> {
        match self {
            SerdeRepr::U8 => Some(8),
            SerdeRepr::U16 => Some(16),
            SerdeRepr::U32 => Some(32),
            SerdeRepr::U64 => Some(64),
            SerdeRepr::U128 => Some(128),
            SerdeRepr::List => None,
            SerdeRepr::Map => None,
            SerdeRepr::Array => None,
        }
    }
}

/// An variant in the enum set type.
struct EnumSetValue {
    /// The name of the variant.
    name: Ident,
    /// The discriminant of the variant.
    variant_repr: u32,
}

/// Stores information about the enum set type.
#[allow(dead_code)]
struct EnumSetInfo {
    /// The name of the enum.
    name: Ident,
    /// The crate name to use.
    crate_name: Option<Ident>,
    /// The numeric type to represent the `EnumSet` as in memory.
    explicit_internal_repr: Option<InternalRepr>,
    /// Forces the internal numeric type of the `EnumSet` to be an array.
    internal_repr_force_array: bool,
    /// The numeric type to serialize the enum as.
    explicit_serde_repr: Option<SerdeRepr>,
    /// A list of variants in the enum.
    variants: Vec<EnumSetValue>,

    /// The highest encountered variant discriminant.
    max_discrim: u32,
    /// The span of the highest encountered variant.
    max_discrim_span: Option<Span>,
    /// The current variant discriminant. Used to track, e.g. `A=10,B,C`.
    cur_discrim: u32,
    /// A list of variant names that are already in use.
    used_variant_names: HashSet<String>,
    /// A list of variant discriminants that are already in use.
    used_discriminants: HashSet<u32>,

    /// Avoid generating operator overloads on the enum type.
    no_ops: bool,
    /// Avoid generating implementations for `Clone`, `Copy`, `Eq`, and `PartialEq`.
    no_super_impls: bool,
    /// Disallow unknown bits while deserializing the enum.
    serialize_deny_unknown: bool,
}
impl EnumSetInfo {
    fn new(input: &DeriveInput, attrs: &EnumsetAttrs) -> EnumSetInfo {
        EnumSetInfo {
            name: input.ident.clone(),
            crate_name: attrs
                .crate_name
                .as_ref()
                .map(|x| Ident::new(x, Span::call_site())),
            explicit_internal_repr: None,
            internal_repr_force_array: false,
            explicit_serde_repr: None,
            variants: Vec::new(),
            max_discrim: 0,
            max_discrim_span: None,
            cur_discrim: 0,
            used_variant_names: HashSet::new(),
            used_discriminants: HashSet::new(),
            no_ops: attrs.no_ops,
            no_super_impls: attrs.no_super_impls,
            serialize_deny_unknown: attrs.serialize_deny_unknown,
        }
    }

    /// Explicits sets the serde representation of the enumset from a string.
    fn push_serialize_repr(&mut self, span: Span, ty: &str) -> Result<()> {
        match ty {
            "u8" => self.explicit_serde_repr = Some(SerdeRepr::U8),
            "u16" => self.explicit_serde_repr = Some(SerdeRepr::U16),
            "u32" => self.explicit_serde_repr = Some(SerdeRepr::U32),
            "u64" => self.explicit_serde_repr = Some(SerdeRepr::U64),
            "u128" => self.explicit_serde_repr = Some(SerdeRepr::U128),
            "list" => self.explicit_serde_repr = Some(SerdeRepr::List),
            "map" => self.explicit_serde_repr = Some(SerdeRepr::Map),
            "array" => self.explicit_serde_repr = Some(SerdeRepr::Array),
            _ => error(span, format!("`{}` is not a valid serialized representation.", ty))?,
        }
        Ok(())
    }

    /// Explicitly sets the representation of the enumset from a string.
    fn push_repr(&mut self, span: Span, ty: &str) -> Result<()> {
        match ty {
            "u8" => self.explicit_internal_repr = Some(InternalRepr::U8),
            "u16" => self.explicit_internal_repr = Some(InternalRepr::U16),
            "u32" => self.explicit_internal_repr = Some(InternalRepr::U32),
            "u64" => self.explicit_internal_repr = Some(InternalRepr::U64),
            "u128" => self.explicit_internal_repr = Some(InternalRepr::U128),
            "array" => self.internal_repr_force_array = true,
            _ => error(span, format!("`{}` is not a valid internal enumset representation.", ty))?,
        }
        Ok(())
    }

    /// Adds a variant to the enumset.
    fn push_variant(&mut self, variant: &Variant) -> Result<()> {
        if self.used_variant_names.contains(&variant.ident.to_string()) {
            error(variant.span(), "Duplicated variant name.")
        } else if let Fields::Unit = variant.fields {
            // Parse the discriminant.
            if let Some((_, expr)) = &variant.discriminant {
                if let Expr::Lit(ExprLit { lit: Lit::Int(i), .. }) = expr {
                    match i.base10_parse() {
                        Ok(val) => self.cur_discrim = val,
                        Err(_) => error(expr.span(), "Enum discriminants must fit into `u32`.")?,
                    }
                } else if let Expr::Unary(ExprUnary { op: UnOp::Neg(_), .. }) = expr {
                    error(expr.span(), "Enum discriminants must not be negative.")?;
                } else {
                    error(variant.span(), "Enum discriminants must be literal expressions.")?;
                }
            }

            // Validate the discriminant.
            let discriminant = self.cur_discrim;
            if discriminant >= 0xFFFFFFC0 {
                error(variant.span(), "Maximum discriminant allowed is `0xFFFFFFBF`.")?;
            }
            if self.used_discriminants.contains(&discriminant) {
                error(variant.span(), "Duplicated enum discriminant.")?;
            }

            // Add the variant to the info.
            self.cur_discrim += 1;
            if discriminant > self.max_discrim {
                self.max_discrim = discriminant;
                self.max_discrim_span = Some(variant.span());
            }
            self.variants
                .push(EnumSetValue { name: variant.ident.clone(), variant_repr: discriminant });
            self.used_variant_names.insert(variant.ident.to_string());
            self.used_discriminants.insert(discriminant);

            Ok(())
        } else {
            error(variant.span(), "`#[derive(EnumSetType)]` can only be used on fieldless enums.")
        }
    }

    /// Returns the actual internal representation of the set.
    fn internal_repr(&self) -> InternalRepr {
        match self.explicit_internal_repr {
            Some(x) => x,
            None => match self.max_discrim {
                x if x < 8 && !self.internal_repr_force_array => InternalRepr::U8,
                x if x < 16 && !self.internal_repr_force_array => InternalRepr::U16,
                x if x < 32 && !self.internal_repr_force_array => InternalRepr::U32,
                x if x < 64 && !self.internal_repr_force_array => InternalRepr::U64,
                x => InternalRepr::Array((x as usize + 64) / 64),
            },
        }
    }

    /// Returns the actual serde representation of the set.
    fn serde_repr(&self) -> SerdeRepr {
        match self.explicit_serde_repr {
            Some(x) => x,
            None => match self.max_discrim {
                x if x < 8 => SerdeRepr::U8,
                x if x < 16 => SerdeRepr::U16,
                x if x < 32 => SerdeRepr::U32,
                x if x < 64 => SerdeRepr::U64,
                x if x < 128 => SerdeRepr::U128,
                _ => SerdeRepr::Array,
            },
        }
    }

    /// Validate the enumset type.
    fn validate(&self) -> Result<()> {
        // Gets the span of the maximum value.
        let largest_discriminant_span = match &self.max_discrim_span {
            Some(x) => *x,
            None => Span::call_site(),
        };

        // Check if all bits of the bitset can fit in the memory representation, if one was given.
        if self.internal_repr().supported_variants() <= self.max_discrim as usize {
            error(
                largest_discriminant_span,
                "`repr` is too small to contain the largest discriminant.",
            )?;
        }

        // Check if all bits of the bitset can fit in the serialization representation.
        if let Some(supported_variants) = self.serde_repr().supported_variants() {
            if supported_variants <= self.max_discrim as usize {
                error(
                    largest_discriminant_span,
                    "`serialize_repr` is too small to contain the largest discriminant.",
                )?;
            }
        }

        Ok(())
    }

    /// Returns a bitmask of all variants in the set.
    fn variant_map(&self) -> Vec<u64> {
        let mut vec = vec![0];
        for variant in &self.variants {
            let (idx, bit) = (variant.variant_repr as usize / 64, variant.variant_repr % 64);
            while idx >= vec.len() {
                vec.push(0);
            }
            vec[idx] |= 1u64 << bit;
        }
        vec
    }
}

/// Generates the actual `EnumSetType` impl.
fn enum_set_type_impl(info: EnumSetInfo, warnings: Vec<(Span, &'static str)>) -> SynTokenStream {
    let name = &info.name;

    let enumset = match &info.crate_name {
        Some(crate_name) => quote!(::#crate_name),
        None => {
            #[cfg(feature = "proc-macro-crate")]
            {
                use proc_macro_crate::FoundCrate;

                let crate_name = proc_macro_crate::crate_name("enumset");
                match crate_name {
                    Ok(FoundCrate::Name(name)) => {
                        let ident = Ident::new(&name, Span::call_site());
                        quote!(::#ident)
                    }
                    _ => quote!(::enumset),
                }
            }

            #[cfg(not(feature = "proc-macro-crate"))]
            {
                quote!(::enumset)
            }
        }
    };
    let typed_enumset = quote!(#enumset::EnumSet<#name>);
    let core = quote!(#enumset::__internal::core_export);
    let internal = quote!(#enumset::__internal);
    #[cfg(feature = "serde")]
    let serde = quote!(#enumset::__internal::serde);

    let repr = match info.internal_repr() {
        InternalRepr::U8 => quote! { u8 },
        InternalRepr::U16 => quote! { u16 },
        InternalRepr::U32 => quote! { u32 },
        InternalRepr::U64 => quote! { u64 },
        InternalRepr::U128 => quote! { u128 },
        InternalRepr::Array(size) => quote! { #internal::ArrayRepr<{ #size }> },
    };
    let variant_map = info.variant_map();
    let all_variants = match info.internal_repr() {
        InternalRepr::U8 | InternalRepr::U16 | InternalRepr::U32 | InternalRepr::U64 => {
            let lit = Literal::u64_unsuffixed(variant_map[0]);
            quote! { #lit }
        }
        InternalRepr::U128 => {
            let lit = Literal::u128_unsuffixed(
                variant_map[0] as u128 | variant_map.get(1).map_or(0, |x| (*x as u128) << 64),
            );
            quote! { #lit }
        }
        InternalRepr::Array(size) => {
            let mut new = Vec::new();
            for i in 0..size {
                new.push(Literal::u64_unsuffixed(*variant_map.get(i).unwrap_or(&0)));
            }
            quote! { #internal::ArrayRepr::<{ #size }>([#(#new,)*]) }
        }
    };

    let ops = if info.no_ops {
        quote! {}
    } else {
        quote! {
            #[automatically_derived]
            impl<O: Into<#typed_enumset>> #core::ops::Sub<O> for #name {
                type Output = #typed_enumset;
                fn sub(self, other: O) -> Self::Output {
                    #enumset::EnumSet::only(self) - other.into()
                }
            }
            #[automatically_derived]
            impl<O: Into<#typed_enumset>> #core::ops::BitAnd<O> for #name {
                type Output = #typed_enumset;
                fn bitand(self, other: O) -> Self::Output {
                    #enumset::EnumSet::only(self) & other.into()
                }
            }
            #[automatically_derived]
            impl<O: Into<#typed_enumset>> #core::ops::BitOr<O> for #name {
                type Output = #typed_enumset;
                fn bitor(self, other: O) -> Self::Output {
                    #enumset::EnumSet::only(self) | other.into()
                }
            }
            #[automatically_derived]
            impl<O: Into<#typed_enumset>> #core::ops::BitXor<O> for #name {
                type Output = #typed_enumset;
                fn bitxor(self, other: O) -> Self::Output {
                    #enumset::EnumSet::only(self) ^ other.into()
                }
            }
            #[automatically_derived]
            impl #core::ops::Not for #name {
                type Output = #typed_enumset;
                fn not(self) -> Self::Output {
                    !#enumset::EnumSet::only(self)
                }
            }
            #[automatically_derived]
            impl #core::cmp::PartialEq<#typed_enumset> for #name {
                fn eq(&self, other: &#typed_enumset) -> bool {
                    #enumset::EnumSet::only(*self) == *other
                }
            }
        }
    };

    #[cfg(feature = "serde")]
    let serde_repr = info.serde_repr();

    #[cfg(feature = "serde")]
    let serde_ops = match serde_repr {
        SerdeRepr::U8 | SerdeRepr::U16 | SerdeRepr::U32 | SerdeRepr::U64 | SerdeRepr::U128 => {
            let (serialize_repr, from_fn, to_fn) = match serde_repr {
                SerdeRepr::U8 => (quote! { u8 }, quote! { from_u8 }, quote! { to_u8 }),
                SerdeRepr::U16 => (quote! { u16 }, quote! { from_u16 }, quote! { to_u16 }),
                SerdeRepr::U32 => (quote! { u32 }, quote! { from_u32 }, quote! { to_u32 }),
                SerdeRepr::U64 => (quote! { u64 }, quote! { from_u64 }, quote! { to_u64 }),
                SerdeRepr::U128 => (quote! { u128 }, quote! { from_u128 }, quote! { to_u128 }),
                _ => unreachable!(),
            };
            let check_unknown = if info.serialize_deny_unknown {
                quote! {
                    if value & !#all_variants != 0 {
                        use #serde::de::Error;
                        return #core::prelude::v1::Err(
                            D::Error::custom("enumset contains unknown bits")
                        )
                    }
                }
            } else {
                quote! {}
            };
            quote! {
                fn serialize<S: #serde::Serializer>(
                    set: #enumset::EnumSet<#name>, ser: S,
                ) -> #core::result::Result<S::Ok, S::Error> {
                    let value =
                        <#repr as #enumset::__internal::EnumSetTypeRepr>::#to_fn(&set.__priv_repr);
                    #serde::Serialize::serialize(&value, ser)
                }
                fn deserialize<'de, D: #serde::Deserializer<'de>>(
                    de: D,
                ) -> #core::result::Result<#enumset::EnumSet<#name>, D::Error> {
                    let value = <#serialize_repr as #serde::Deserialize>::deserialize(de)?;
                    #check_unknown
                    let value = <#repr as #enumset::__internal::EnumSetTypeRepr>::#from_fn(value);
                    #core::prelude::v1::Ok(#enumset::EnumSet {
                        __priv_repr: value & #all_variants,
                    })
                }
            }
        }
        SerdeRepr::List => {
            let expecting_str = format!("a list of {}", name);
            quote! {
                fn serialize<S: #serde::Serializer>(
                    set: #enumset::EnumSet<#name>, ser: S,
                ) -> #core::result::Result<S::Ok, S::Error> {
                    use #serde::ser::SerializeSeq;
                    let mut seq = ser.serialize_seq(#core::prelude::v1::Some(set.len()))?;
                    for bit in set {
                        seq.serialize_element(&bit)?;
                    }
                    seq.end()
                }
                fn deserialize<'de, D: #serde::Deserializer<'de>>(
                    de: D,
                ) -> #core::result::Result<#enumset::EnumSet<#name>, D::Error> {
                    struct Visitor;
                    impl <'de> #serde::de::Visitor<'de> for Visitor {
                        type Value = #enumset::EnumSet<#name>;
                        fn expecting(
                            &self, formatter: &mut #core::fmt::Formatter,
                        ) -> #core::fmt::Result {
                            write!(formatter, #expecting_str)
                        }
                        fn visit_seq<A>(
                            mut self, mut seq: A,
                        ) -> #core::result::Result<Self::Value, A::Error> where
                            A: #serde::de::SeqAccess<'de>
                        {
                            let mut accum = #enumset::EnumSet::<#name>::new();
                            while let #core::prelude::v1::Some(val) = seq.next_element::<#name>()? {
                                accum |= val;
                            }
                            #core::prelude::v1::Ok(accum)
                        }
                    }
                    de.deserialize_seq(Visitor)
                }
            }
        }
        SerdeRepr::Map => {
            let expecting_str = format!("a map from {} to bool", name);
            quote! {
                fn serialize<S: #serde::Serializer>(
                    set: #enumset::EnumSet<#name>, ser: S,
                ) -> #core::result::Result<S::Ok, S::Error> {
                    use #serde::ser::SerializeMap;
                    let mut map = ser.serialize_map(#core::prelude::v1::Some(set.len()))?;
                    for bit in set {
                        map.serialize_entry(&bit, &true)?;
                    }
                    map.end()
                }
                fn deserialize<'de, D: #serde::Deserializer<'de>>(
                    de: D,
                ) -> #core::result::Result<#enumset::EnumSet<#name>, D::Error> {
                    struct Visitor;
                    impl <'de> #serde::de::Visitor<'de> for Visitor {
                        type Value = #enumset::EnumSet<#name>;
                        fn expecting(
                            &self, formatter: &mut #core::fmt::Formatter,
                        ) -> #core::fmt::Result {
                            write!(formatter, #expecting_str)
                        }
                        fn visit_map<A>(
                            mut self, mut map: A,
                        ) -> #core::result::Result<Self::Value, A::Error> where
                            A: #serde::de::MapAccess<'de>
                        {
                            let mut accum = #enumset::EnumSet::<#name>::new();
                            while let #core::prelude::v1::Some((val, is_present)) =
                                map.next_entry::<#name, bool>()?
                            {
                                if is_present {
                                    accum |= val;
                                }
                            }
                            #core::prelude::v1::Ok(accum)
                        }
                    }
                    de.deserialize_map(Visitor)
                }
            }
        }
        SerdeRepr::Array => {
            let preferred_size = quote! {
                <<#name as #internal::EnumSetTypePrivate>::Repr as #internal::EnumSetTypeRepr>
                    ::PREFERRED_ARRAY_LEN
            };
            let (check_extra, convert_array) = if info.serialize_deny_unknown {
                (
                    quote! {
                        if _val != 0 {
                            return #core::prelude::v1::Err(
                                D::Error::custom("enumset contains unknown bits")
                            )
                        }
                    },
                    quote! {
                        match #enumset::EnumSet::<#name>::try_from_array(accum) {
                            Some(x) => x,
                            None => #core::prelude::v1::Err(
                                D::Error::custom("enumset contains unknown bits")
                            ),
                        }
                    },
                )
            } else {
                (quote! {}, quote! {
                    #core::prelude::v1::Ok(#enumset::EnumSet::<#name>::from_array(accum))
                })
            };
            quote! {
                fn serialize<S: #serde::Serializer>(
                    set: #enumset::EnumSet<#name>, ser: S,
                ) -> #core::result::Result<S::Ok, S::Error> {
                    // read the enum as an array
                    let array = set.as_array::<{ #preferred_size }>();

                    // find the last non-zero value in the array
                    let mut end = array.len();
                    for i in (0..array.len()).rev() {
                        if array[i] != 0 {
                            break;
                        }
                        end = i + 1;
                    }

                    // serialize the array
                    #serde::Serialize::serialize(&array[..end], ser)
                }
                fn deserialize<'de, D: #serde::Deserializer<'de>>(
                    de: D,
                ) -> #core::result::Result<#enumset::EnumSet<#name>, D::Error> {
                    struct Visitor;
                    impl <'de> #serde::de::Visitor<'de> for Visitor {
                        type Value = #enumset::EnumSet<#name>;
                        fn expecting(
                            &self, formatter: &mut #core::fmt::Formatter,
                        ) -> #core::fmt::Result {
                            write!(formatter, "a list of u64")
                        }
                        fn visit_seq<A>(
                            mut self, mut seq: A,
                        ) -> #core::result::Result<Self::Value, A::Error> where
                            A: #serde::de::SeqAccess<'de>
                        {
                            let mut accum = [0; #preferred_size];

                            let mut i = 0;
                            while let #core::prelude::v1::Some(val) = seq.next_element::<u64>()? {
                                accum[i] = val;
                                i += 1;

                                if i == accum.len() {
                                    break;
                                }
                            }
                            while let #core::prelude::v1::Some(_val) = seq.next_element::<u64>()? {
                                #check_extra
                            }

                            #convert_array
                        }
                    }
                    de.deserialize_seq(Visitor)
                }
            }
        }
    };

    #[cfg(not(feature = "serde"))]
    let serde_ops = quote! {};

    let is_uninhabited = info.variants.is_empty();
    let is_zst = info.variants.len() == 1;
    let into_impl = if is_uninhabited {
        quote! {
            fn enum_into_u32(self) -> u32 {
                panic!(concat!(stringify!(#name), " is uninhabited."))
            }
            unsafe fn enum_from_u32(val: u32) -> Self {
                panic!(concat!(stringify!(#name), " is uninhabited."))
            }
        }
    } else if is_zst {
        let variant = &info.variants[0].name;
        quote! {
            fn enum_into_u32(self) -> u32 {
                self as u32
            }
            unsafe fn enum_from_u32(val: u32) -> Self {
                #name::#variant
            }
        }
    } else {
        let variant_name: Vec<_> = info.variants.iter().map(|x| &x.name).collect();
        let variant_value: Vec<_> = info.variants.iter().map(|x| x.variant_repr).collect();

        let const_field: Vec<_> = ["IS_U8", "IS_U16", "IS_U32", "IS_U64", "IS_U128"]
            .iter()
            .map(|x| Ident::new(x, Span::call_site()))
            .collect();
        let int_type: Vec<_> = ["u8", "u16", "u32", "u64", "u128"]
            .iter()
            .map(|x| Ident::new(x, Span::call_site()))
            .collect();

        quote! {
            fn enum_into_u32(self) -> u32 {
                self as u32
            }
            unsafe fn enum_from_u32(val: u32) -> Self {
                // We put these in const fields so the branches they guard aren't generated even
                // on -O0
                #(const #const_field: bool =
                    #core::mem::size_of::<#name>() == #core::mem::size_of::<#int_type>();)*
                match val {
                    // Every valid variant value has an explicit branch. If they get optimized out,
                    // great. If the representation has changed somehow, and they don't, oh well,
                    // there's still no UB.
                    #(#variant_value => #name::#variant_name,)*
                    // Helps hint to the LLVM that this is a transmute. Note that this branch is
                    // still unreachable.
                    #(x if #const_field => {
                        let x = x as #int_type;
                        *(&x as *const _ as *const #name)
                    })*
                    // Default case. Sometimes causes LLVM to generate a table instead of a simple
                    // transmute, but, oh well.
                    _ => #core::hint::unreachable_unchecked(),
                }
            }
        }
    };

    let eq_impl = if is_uninhabited {
        quote!(panic!(concat!(stringify!(#name), " is uninhabited.")))
    } else {
        quote!((*self as u32) == (*other as u32))
    };

    let super_impls = if info.no_super_impls {
        quote! {}
    } else {
        quote! {
            #[automatically_derived]
            impl #core::cmp::PartialEq for #name {
                fn eq(&self, other: &Self) -> bool {
                    #eq_impl
                }
            }
            #[automatically_derived]
            impl #core::cmp::Eq for #name { }
            #[automatically_derived]
            #[allow(clippy::expl_impl_clone_on_copy)]
            impl #core::clone::Clone for #name {
                fn clone(&self) -> Self {
                    *self
                }
            }
            #[automatically_derived]
            impl #core::marker::Copy for #name { }
        }
    };

    let impl_with_repr = if info.explicit_internal_repr.is_some() {
        quote! {
            #[automatically_derived]
            unsafe impl #enumset::EnumSetTypeWithRepr for #name {
                type Repr = #repr;
            }
        }
    } else {
        quote! {}
    };

    let inherent_impl_blocks = match info.internal_repr() {
        InternalRepr::U8
        | InternalRepr::U16
        | InternalRepr::U32
        | InternalRepr::U64
        | InternalRepr::U128 => {
            let self_as_repr_mask = if is_uninhabited {
                quote! { 0 } // impossible anyway
            } else {
                quote! { 1 << self as #repr }
            };

            quote! {
                #[automatically_derived]
                #[doc(hidden)]
                impl #name {
                    /// Creates a new enumset with only this variant.
                    #[deprecated(note = "This method is an internal implementation detail \
                                         generated by the `enumset` crate's procedural macro. It \
                                         should not be used directly.")]
                    #[doc(hidden)]
                    pub const fn __impl_enumset_internal__const_only(
                        self,
                    ) -> #enumset::EnumSet<#name> {
                        #enumset::EnumSet { __priv_repr: #self_as_repr_mask }
                    }

                    /// Creates a new enumset with this variant added.
                    #[deprecated(note = "This method is an internal implementation detail \
                                         generated by the `enumset` crate's procedural macro. It \
                                         should not be used directly.")]
                    #[doc(hidden)]
                    pub const fn __impl_enumset_internal__const_merge(
                        self, chain: #enumset::EnumSet<#name>,
                    ) -> #enumset::EnumSet<#name> {
                        #enumset::EnumSet { __priv_repr: chain.__priv_repr | #self_as_repr_mask }
                    }
                }
            }
        }
        InternalRepr::Array(size) => {
            quote! {
                #[automatically_derived]
                #[doc(hidden)]
                impl #name {
                    /// Creates a new enumset with only this variant.
                    #[deprecated(note = "This method is an internal implementation detail \
                                         generated by the `enumset` crate's procedural macro. It \
                                         should not be used directly.")]
                    #[doc(hidden)]
                    pub const fn __impl_enumset_internal__const_only(
                        self,
                    ) -> #enumset::EnumSet<#name> {
                        let mut set = #enumset::EnumSet::<#name> {
                            __priv_repr: #internal::ArrayRepr::<{ #size }>([0; #size]),
                        };
                        let bit = self as u32;
                        let (idx, bit) = (bit as usize / 64, bit % 64);
                        set.__priv_repr.0[idx] |= 1u64 << bit;
                        set
                    }

                    /// Creates a new enumset with this variant added.
                    #[deprecated(note = "This method is an internal implementation detail \
                                         generated by the `enumset` crate's procedural macro. It \
                                         should not be used directly.")]
                    #[doc(hidden)]
                    pub const fn __impl_enumset_internal__const_merge(
                        self, mut chain: #enumset::EnumSet<#name>,
                    ) -> #enumset::EnumSet<#name> {
                        let bit = self as u32;
                        let (idx, bit) = (bit as usize / 64, bit % 64);
                        chain.__priv_repr.0[idx] |= 1u64 << bit;
                        chain
                    }
                }
            }
        }
    };

    let mut generated_warnings = SynTokenStream::new();
    for (span, warning) in warnings {
        generated_warnings.extend(quote_spanned! {
            span => {
                #[deprecated(note = #warning)]
                #[allow(non_upper_case_globals)]
                const _w: () = ();
                let _ = _w;
            }
        });
    }

    let bit_width = info.max_discrim + 1;
    let variant_count = info.variants.len() as u32;
    quote! {
        #[automatically_derived]
        unsafe impl #internal::EnumSetTypePrivate for #name {
            type Repr = #repr;
            const ALL_BITS: Self::Repr = #all_variants;
            const BIT_WIDTH: u32 = #bit_width;
            const VARIANT_COUNT: u32 = #variant_count;
            #into_impl
            #serde_ops
        }

        #[automatically_derived]
        unsafe impl #enumset::EnumSetType for #name { }

        #impl_with_repr
        #super_impls
        #inherent_impl_blocks

        #ops

        const _: () = {
            fn __enumset_derive__generated_warnings() {
                #generated_warnings
            }
        };
    }
}

#[proc_macro_derive(EnumSetType, attributes(enumset))]
pub fn derive_enum_set_type(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);
    let attrs: EnumsetAttrs = match EnumsetAttrs::from_derive_input(&input) {
        Ok(attrs) => attrs,
        Err(e) => return e.write_errors().into(),
    };
    match derive_enum_set_type_0(input, attrs) {
        Ok(v) => v,
        Err(e) => e.to_compile_error().into(),
    }
}
fn derive_enum_set_type_0(input: DeriveInput, attrs: EnumsetAttrs) -> Result<TokenStream> {
    if !input.generics.params.is_empty() {
        error(
            input.generics.span(),
            "`#[derive(EnumSetType)]` cannot be used on enums with type parameters.",
        )
    } else if let Data::Enum(data) = &input.data {
        let mut info = EnumSetInfo::new(&input, &attrs);
        let mut warnings = Vec::new();

        // Check enum repr
        for attr in &input.attrs {
            if attr.path().is_ident("repr") {
                let meta: Ident = attr.parse_args()?;
                match meta.to_string().as_str() {
                    "C" | "Rust" => {}
                    "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => {}
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => {}
                    x => error(
                        attr.span(),
                        format!("`#[repr({})]` cannot be used on enumset variants.", x),
                    )?,
                }
            }
        }

        // Parse internal representations
        if let Some(repr) = &*attrs.repr {
            info.push_repr(attrs.repr.span(), repr)?;
        }

        // Parse serialization representations
        if let Some(serialize_repr) = &*attrs.serialize_repr {
            info.push_serialize_repr(attrs.serialize_repr.span(), serialize_repr)?;
        }
        if *attrs.serialize_as_map {
            info.explicit_serde_repr = Some(SerdeRepr::Map);
            warnings.push((
                attrs.serialize_as_map.span(),
                "#[enumset(serialize_as_map)] is deprecated. \
                 Use `#[enumset(serialize_repr = \"map\")]` instead.",
            ));
        }
        if *attrs.serialize_as_list {
            // in old versions, serialize_as_list will override serialize_as_map
            info.explicit_serde_repr = Some(SerdeRepr::List);
            warnings.push((
                attrs.serialize_as_list.span(),
                "#[enumset(serialize_as_list)] is deprecated. \
                 Use `#[enumset(serialize_repr = \"list\")]` instead.",
            ));
        }

        // Parse enum variants
        for variant in &data.variants {
            info.push_variant(variant)?;
        }

        // Validate the enumset
        info.validate()?;

        // Generates the actual `EnumSetType` implementation
        Ok(enum_set_type_impl(info, warnings).into())
    } else {
        error(input.span(), "`#[derive(EnumSetType)]` may only be used on enums")
    }
}
