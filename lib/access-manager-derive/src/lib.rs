#[proc_macro_derive(AccessManaged)]
pub fn ssz(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    do_ssz(parse_macro_input!(ts as DeriveInput))
        // .inspect(|ts| {
        //     dbg!(ts.to_string());
        // })
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn do_ssz(derive_input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let meta = derive_input
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("ssz"))
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

    let Data::Enum {} = derive_input.data {}

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
