use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::ParseStream, spanned::Spanned, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    Fields,
};

use crate::MAX_UNION_SELECTOR;

pub fn do_ssz(derive_input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let meta = derive_input
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("ssz"))
        .map(SszMeta::try_from_attribute)
        .try_fold(None, |acc, curr| {
            let curr = curr?;

            match (acc, curr) {
                (None, None) => Ok(None),
                (None, Some(curr)) => Ok(Some(curr)),
                (Some(acc), None) => Ok(Some(acc)),
                (Some(_acc), Some(curr)) => Err(syn::Error::new(
                    curr.span(),
                    "duplicate `#[ssz(...)]` attribute",
                )),
            }
        })?;

    match (&derive_input.data, meta) {
        // container
        (Data::Struct(s), None) => container(&derive_input, s),
        // wrapper
        (Data::Struct(s), Some(SszMeta::Transparent(_))) => wrapper(&derive_input, s),
        (Data::Struct(_), Some(SszMeta::Union(span))) => Err(syn::Error::new(
            span,
            "`#[ssz(union)]` is only valid on enums",
        )),
        (Data::Enum(_), None) => Err(syn::Error::new(
            Span::call_site(),
            "must specify either `#[ssz(transparent)]` or `#[ssz(union)]`, \
            or for enums",
        )),
        (Data::Enum(_), Some(SszMeta::Transparent(span))) => Err(syn::Error::new(
            span,
            "`#[ssz(transparent)]` is not supported on enums due to ambiguities in decoding",
        )),
        (Data::Enum(e), Some(SszMeta::Union(_))) => enum_union(&derive_input, e),
        (Data::Union(u), _) => Err(syn::Error::new(
            u.union_token.span(),
            "unions are not supported",
        )),
    }
}

enum SszMeta {
    Transparent(Span),
    Union(Span),
}

impl SszMeta {
    fn span(&self) -> Span {
        match self {
            SszMeta::Transparent(span) => *span,
            SszMeta::Union(span) => *span,
        }
    }

    fn try_from_attribute(attr: &Attribute) -> syn::Result<Option<Self>> {
        syn::custom_keyword!(transparent);
        syn::custom_keyword!(union);

        let mut ssz_meta = None;

        macro_rules! parse_tag {
            ($input:ident; $($kw:ident => $Variant:ident;)+) => {
                $(
                    if let Some(kw) = $input.parse::<Option<$kw>>()? {
                        if ssz_meta.is_some() {
                            return Err(syn::Error::new_spanned(
                                attr,
                                "duplicate #[ssz(...)] attribute",
                            ));
                        }
                        if !$input.is_empty() {
                            return Err($input.error("unexpected extra tokens"));
                        }

                        ssz_meta = Some(SszMeta::$Variant(kw.span));
                        return Ok(());
                    }
                )+
            }
        }

        attr.parse_args_with(|input: ParseStream| {
            parse_tag!(
                input;
                transparent => Transparent;
                union => Union;
            );

            if !input.is_empty() {
                return Err(input.error("unexpected extra tokens"));
            }

            Ok(())
        })?;

        Ok(ssz_meta)
    }
}

fn container(
    derive_input: &DeriveInput,
    struct_data: &DataStruct,
) -> Result<TokenStream, syn::Error> {
    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = &derive_input.generics.split_for_impl();

    let Fields::Named(..) = &struct_data.fields else {
        return Err(syn::Error::new(
            struct_data.fields.span(),
            "`#[derive(Ssz)]` on structs only supports named fields, use \
            `#[ssz(transparent)]` if the struct only has one field and \
            it should be treated as the inner type",
        ));
    };

    let (ident, ty) = struct_data
        .fields
        .iter()
        .map(|field| {
            (
                field
                    .ident
                    .as_ref()
                    .expect("fields checked above to be named; qed;"),
                &field.ty,
            )
        })
        .unzip::<_, _, Vec<_>, Vec<_>>();

    let num_leaves = ident.len();

    let output = quote! {
        impl #impl_generics ::ssz::Ssz for #name #ty_generics #where_clause {
            const SSZ_FIXED_LEN: Option<::core::num::NonZeroUsize> = '___SSZ_FIXED_LEN: {
                let mut total = 0;

                #(
                    match <#ty as ::ssz::Ssz>::SSZ_FIXED_LEN {
                        Some(v) => {
                            total += v.get();
                        }
                        None => break '___SSZ_FIXED_LEN None,
                    }
                )*

                ::core::num::NonZeroUsize::new(total)
            };

            const TREE_HASH_TYPE: ::ssz::tree_hash::TreeHashType = ::ssz::tree_hash::TreeHashType::Container;

            fn tree_hash_root(&self) -> ::ssz::tree_hash::Hash256 {
                let mut hasher = ::ssz::tree_hash::MerkleHasher::with_leaves(#num_leaves);

                #(
                    hasher.write(&self.#ident.tree_hash_root())
                        .expect("tree hash derive should not apply too many leaves");
                )*

                hasher.finish().expect("tree hash derive should not have a remaining buffer")
            }

            fn ssz_bytes_len(&self) -> ::core::num::NonZeroUsize {
                match <Self as ::ssz::Ssz>::SSZ_FIXED_LEN {
                    Some(len) => len,
                    None => {
                        let mut len: usize = 0;
                        #(
                            match <#ty as ::ssz::Ssz>::SSZ_FIXED_LEN {
                                Some(fixed_len) => {
                                    len = len
                                        .checked_add(fixed_len.get())
                                        .expect("encode ssz_bytes_len length overflow");
                                }
                                None => {
                                    len = len
                                        .checked_add(::ssz::BYTES_PER_LENGTH_OFFSET)
                                        .expect("encode ssz_bytes_len length overflow for offset");
                                    len = len
                                        .checked_add(::ssz::Ssz::ssz_bytes_len(&self.#ident).get())
                                        .expect("encode ssz_bytes_len length overflow for bytes");
                                }
                            }
                        )*

                        ::core::num::NonZeroUsize::new(len).expect("sum of non-zero numbers is non-zero; qed;")
                    }
                }
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                let mut offset: usize = 0;
                #(
                    offset = offset
                        .checked_add(<#ty as ::ssz::Ssz>::SSZ_FIXED_LEN.map(|x| x.get()).unwrap_or(::ssz::BYTES_PER_LENGTH_OFFSET))
                        .expect("encode ssz_append offset overflow");
                )*

                let mut encoder = ::ssz::encode::SszEncoder::container(buf, offset);

                #(
                    encoder.append(&self.#ident);
                )*

                encoder.finalize();
            }

            fn from_ssz_bytes(bytes: &[u8]) -> std::result::Result<Self, ::ssz::decode::DecodeError> {
                match <Self as ::ssz::Ssz>::SSZ_FIXED_LEN {
                    Some(fixed_len) => {
                        if bytes.len() != fixed_len.get() {
                            return Err(::ssz::decode::DecodeError::InvalidByteLength {
                                found: bytes.len(),
                                expected: fixed_len.get(),
                            });
                        }

                        let mut start: usize = 0;
                        let mut end = start;

                        #(
                            let #ident = {
                                // REVIEW: Are these checks necessary, since the length is checked above?
                                start = end;
                                end = end
                                    .checked_add(<#ty as ::ssz::Ssz>::SSZ_FIXED_LEN.expect("type is fixed length").get())
                                    // REVIEW: This check can be removed?
                                    .ok_or_else(|| ::ssz::decode::DecodeError::OutOfBoundsByte {
                                        i: usize::max_value()
                                    })?;
                                let slice = bytes.get(start..end)
                                    .ok_or_else(|| ::ssz::decode::DecodeError::InvalidByteLength {
                                        found: bytes.len(),
                                        expected: end
                                    })?;

                                <#ty as ::ssz::Ssz>::from_ssz_bytes(slice)?
                            };

                        )*

                        Ok(Self {
                            #(
                                #ident,
                            )*
                        })
                    }
                    None => {
                        let mut builder = ::ssz::decode::SszDecoderBuilder::new(bytes);

                        #(
                            builder.register_type::<#ty>()?;
                        )*

                        let mut decoder = builder.build()?;

                        #(
                            let #ident = decoder.decode_next()?;
                        )*


                        Ok(Self {
                            #(
                                #ident,
                            )*
                        })
                    }
                }
            }
        }
    };

    Ok(output)
}

fn wrapper(item: &DeriveInput, struct_data: &DataStruct) -> Result<TokenStream, syn::Error> {
    let name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = &item.generics.split_for_impl();

    let (ty, ident) = match &struct_data.fields {
        Fields::Named(fields) => {
            if fields.named.len() != 1 {
                return Err(syn::Error::new(
                    fields.span(),
                    "`#[ssz(transparent)]` requires one field",
                ));
            }
            fields
                .named
                .first()
                .map(|field| {
                    (
                        &field.ty,
                        syn::Member::from(field.ident.clone().expect("fields is named; qed;")),
                    )
                })
                .expect("length is checked above; qed;")
        }
        Fields::Unnamed(fields) => {
            if fields.unnamed.len() != 1 {
                return Err(syn::Error::new(
                    fields.span(),
                    "`#[ssz(transparent)]` requires one field",
                ));
            }
            fields
                .unnamed
                .first()
                .map(|field| (&field.ty, syn::Member::from(0)))
                .expect("length is checked above; qed;")
        }
        Fields::Unit => {
            return Err(syn::Error::new(
                struct_data.fields.span(),
                "`#[ssz(transparent)]` requires one field",
            ))
        }
    };

    let output = quote! {
        impl #impl_generics ::ssz::Ssz for #name #ty_generics #where_clause {
            const SSZ_FIXED_LEN: Option<::core::num::NonZeroUsize> = <#ty as ::ssz::Ssz>::SSZ_FIXED_LEN;

            const TREE_HASH_TYPE: ::ssz::tree_hash::TreeHashType = <#ty as ::ssz::Ssz>::TREE_HASH_TYPE;

            fn tree_hash_root(&self) -> ::ssz::tree_hash::Hash256 {
                self.#ident.tree_hash_root()
            }

            fn ssz_bytes_len(&self) -> ::core::num::NonZeroUsize {
                self.#ident.ssz_bytes_len()
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                self.#ident.ssz_append(buf)
            }

            fn from_ssz_bytes(bytes: &[u8]) -> std::result::Result<Self, ::ssz::decode::DecodeError> {
                Ok(Self {
                    #ident: <_>::from_ssz_bytes(bytes)?,
                })
            }
        }
    };

    Ok(output)
}

fn enum_union(derive_input: &DeriveInput, enum_data: &DataEnum) -> Result<TokenStream, syn::Error> {
    let name = &derive_input.ident;
    let (impl_generics, ty_generics, where_clause) = &derive_input.generics.split_for_impl();

    let (variant, ty) = enum_data
        .variants
        .iter()
        .map(|variant| {
            if matches!(variant.fields, Fields::Named(..) | Fields::Unit) {
                Err(syn::Error::new(
                    variant.fields.span(),
                    "only newtype variants are supported",
                ))
            } else if variant.fields.len() != 1 {
                Err(syn::Error::new(
                    variant.fields.span(),
                    "Ssz can only be derived for enums with 1 field per variant",
                ))
            } else {
                Ok((
                    &variant.ident,
                    variant
                        .fields
                        .iter()
                        .next()
                        .expect("length is checked above; qed;")
                        .ty
                        .clone(),
                ))
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .unzip::<_, _, Vec<_>, Vec<_>>();

    let union_selectors = compute_selectors(enum_data)?;

    let output = quote! {
        impl #impl_generics ::ssz::Ssz for #name #ty_generics #where_clause {
            const SSZ_FIXED_LEN: Option<::core::num::NonZeroUsize> = {
                // TODO: Do some assertions here?
                None
            };

            const TREE_HASH_TYPE: ::ssz::tree_hash::TreeHashType = ::ssz::tree_hash::TreeHashType::Container;

            fn tree_hash_root(&self) -> ::ssz::tree_hash::Hash256 {
                match self {
                    #(
                        Self::#variant(ref inner) => {
                            let root = inner.tree_hash_root();
                            let selector = #union_selectors;
                            ::ssz::tree_hash::mix_in_selector(&root, selector)
                                .expect("derive macro should prevent out-of-bounds selectors")
                        },
                    )*
                }
            }

            fn ssz_bytes_len(&self) -> ::core::num::NonZeroUsize {
                match self {
                    #(
                        Self::#variant(ref inner) => inner
                            .ssz_bytes_len()
                            .checked_add(1)
                            .expect("encoded length must be less than usize::MAX"),
                    )*
                }
            }

            fn ssz_append(&self, buf: &mut Vec<u8>) {
                match self {
                    #(
                        Self::#variant(ref inner) => {
                            let union_selector: u8 = #union_selectors;
                            debug_assert!(union_selector <= ::ssz::MAX_UNION_SELECTOR);
                            buf.push(union_selector);
                            inner.ssz_append(buf)
                        },
                    )*
                }
            }

            fn from_ssz_bytes(bytes: &[u8]) -> Result<Self, ::ssz::decode::DecodeError> {
                // Sanity check to ensure the definition here does not drift from the one defined in
                // `ssz`.
                // TODO: Make this a const assertion
                debug_assert_eq!(#MAX_UNION_SELECTOR, ::ssz::MAX_UNION_SELECTOR);

                let (selector, body) = ::ssz::decode::split_union_bytes(bytes)?;

                match selector.into() {
                    #(
                        #union_selectors => {
                            <#ty as ::ssz::Ssz>::from_ssz_bytes(body).map(Self::#variant)
                        },
                    )*
                    other => Err(::ssz::decode::DecodeError::UnionSelectorInvalid(other))
                }
            }
        }
    };

    Ok(output)
}

pub(crate) fn compute_selectors(enum_data: &DataEnum) -> Result<Vec<u8>, syn::Error> {
    if enum_data.variants.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "empty enums are not supported",
        ));
    }
    if !(1_usize..MAX_UNION_SELECTOR as usize).contains(&enum_data.variants.len()) {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("enum cannot have more than {MAX_UNION_SELECTOR} variants"),
        ));
    }
    let union_selectors = (0..(enum_data.variants.len() as u8)).collect::<Vec<_>>();

    Ok(union_selectors)
}
