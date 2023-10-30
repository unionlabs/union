use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Token,
};

#[derive(Default)]
pub(crate) struct FromRequestFieldAttr {
    pub(crate) via: Option<(kw::via, syn::Path)>,
}

pub(crate) enum FromRequestContainerAttr {
    Via(syn::Path),
    RejectionDerive(kw::rejection_derive, RejectionDeriveOptOuts),
    None,
}

pub(crate) mod kw {
    syn::custom_keyword!(via);
    syn::custom_keyword!(rejection_derive);
    syn::custom_keyword!(Display);
    syn::custom_keyword!(Debug);
    syn::custom_keyword!(Error);
}

pub(crate) fn parse_field_attrs(attrs: &[syn::Attribute]) -> syn::Result<FromRequestFieldAttr> {
    let attrs = parse_attrs(attrs)?;

    let mut out = FromRequestFieldAttr::default();

    for from_request_attr in attrs {
        match from_request_attr {
            FieldAttr::Via { via, path } => {
                if out.via.is_some() {
                    return Err(double_attr_error("via", via));
                } else {
                    out.via = Some((via, path));
                }
            }
        }
    }

    Ok(out)
}

pub(crate) fn parse_container_attrs(
    attrs: &[syn::Attribute],
) -> syn::Result<FromRequestContainerAttr> {
    let attrs = parse_attrs::<ContainerAttr>(attrs)?;

    let mut out_via = None;
    let mut out_rejection_derive = None;

    // we track the index of the attribute to know which comes last
    // used to give more accurate error messages
    for (idx, from_request_attr) in attrs.into_iter().enumerate() {
        match from_request_attr {
            ContainerAttr::Via { via, path } => {
                if out_via.is_some() {
                    return Err(double_attr_error("via", via));
                } else {
                    out_via = Some((idx, via, path));
                }
            }
            ContainerAttr::RejectionDerive {
                rejection_derive,
                opt_outs,
            } => {
                if out_rejection_derive.is_some() {
                    return Err(double_attr_error("rejection_derive", rejection_derive));
                } else {
                    out_rejection_derive = Some((idx, rejection_derive, opt_outs));
                }
            }
        }
    }

    match (out_via, out_rejection_derive) {
        (Some((via_idx, via, _)), Some((rejection_derive_idx, rejection_derive, _))) => {
            if via_idx > rejection_derive_idx {
                Err(syn::Error::new_spanned(
                    via,
                    "cannot use both `rejection_derive` and `via`",
                ))
            } else {
                Err(syn::Error::new_spanned(
                    rejection_derive,
                    "cannot use both `via` and `rejection_derive`",
                ))
            }
        }
        (Some((_, _, path)), None) => Ok(FromRequestContainerAttr::Via(path)),
        (None, Some((_, rejection_derive, opt_outs))) => Ok(
            FromRequestContainerAttr::RejectionDerive(rejection_derive, opt_outs),
        ),
        (None, None) => Ok(FromRequestContainerAttr::None),
    }
}

pub(crate) fn parse_attrs<T>(attrs: &[syn::Attribute]) -> syn::Result<Punctuated<T, Token![,]>>
where
    T: Parse,
{
    let attrs = attrs
        .iter()
        .filter(|attr| attr.path.is_ident("from_request"))
        .map(|attr| attr.parse_args_with(Punctuated::<T, Token![,]>::parse_terminated))
        .collect::<syn::Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect::<Punctuated<T, Token![,]>>();
    Ok(attrs)
}

fn double_attr_error<T>(ident: &str, spanned: T) -> syn::Error
where
    T: ToTokens,
{
    syn::Error::new_spanned(spanned, format!("`{}` specified more than once", ident))
}

enum ContainerAttr {
    Via {
        via: kw::via,
        path: syn::Path,
    },
    RejectionDerive {
        rejection_derive: kw::rejection_derive,
        opt_outs: RejectionDeriveOptOuts,
    },
}

impl Parse for ContainerAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lh = input.lookahead1();
        if lh.peek(kw::via) {
            let via = input.parse::<kw::via>()?;
            let content;
            syn::parenthesized!(content in input);
            content.parse().map(|path| Self::Via { via, path })
        } else if lh.peek(kw::rejection_derive) {
            let rejection_derive = input.parse::<kw::rejection_derive>()?;
            let content;
            syn::parenthesized!(content in input);
            content.parse().map(|opt_outs| Self::RejectionDerive {
                rejection_derive,
                opt_outs,
            })
        } else {
            Err(lh.error())
        }
    }
}

enum FieldAttr {
    Via { via: kw::via, path: syn::Path },
}

impl Parse for FieldAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lh = input.lookahead1();
        if lh.peek(kw::via) {
            let via = input.parse::<kw::via>()?;
            let content;
            syn::parenthesized!(content in input);
            content.parse().map(|path| Self::Via { via, path })
        } else {
            Err(lh.error())
        }
    }
}

#[derive(Default)]
pub(crate) struct RejectionDeriveOptOuts {
    debug: Option<kw::Debug>,
    display: Option<kw::Display>,
    error: Option<kw::Error>,
}

impl RejectionDeriveOptOuts {
    pub(crate) fn derive_debug(&self) -> bool {
        self.debug.is_none()
    }

    pub(crate) fn derive_display(&self) -> bool {
        self.display.is_none()
    }

    pub(crate) fn derive_error(&self) -> bool {
        self.error.is_none()
    }
}

impl Parse for RejectionDeriveOptOuts {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        fn parse_opt_out<T>(out: &mut Option<T>, ident: &str, input: ParseStream) -> syn::Result<()>
        where
            T: Parse,
        {
            if out.is_some() {
                Err(input.error(format!("`{}` opt out specified more than once", ident)))
            } else {
                *out = Some(input.parse()?);
                Ok(())
            }
        }

        let mut debug = None::<kw::Debug>;
        let mut display = None::<kw::Display>;
        let mut error = None::<kw::Error>;

        while !input.is_empty() {
            input.parse::<Token![!]>()?;

            let lh = input.lookahead1();
            if lh.peek(kw::Debug) {
                parse_opt_out(&mut debug, "Debug", input)?;
            } else if lh.peek(kw::Display) {
                parse_opt_out(&mut display, "Display", input)?;
            } else if lh.peek(kw::Error) {
                parse_opt_out(&mut error, "Error", input)?;
            } else {
                return Err(lh.error());
            }

            input.parse::<Token![,]>().ok();
        }

        if error.is_none() {
            match (debug, display) {
                (Some(debug), Some(_)) => {
                    return Err(syn::Error::new_spanned(debug, "opt out of `Debug` and `Display` requires also opting out of `Error`. Use `#[from_request(rejection_derive(!Debug, !Display, !Error))]`"));
                }
                (Some(debug), None) => {
                    return Err(syn::Error::new_spanned(debug, "opt out of `Debug` requires also opting out of `Error`. Use `#[from_request(rejection_derive(!Debug, !Error))]`"));
                }
                (None, Some(display)) => {
                    return Err(syn::Error::new_spanned(display, "opt out of `Display` requires also opting out of `Error`. Use `#[from_request(rejection_derive(!Display, !Error))]`"));
                }
                (None, None) => {}
            }
        }

        Ok(Self {
            debug,
            display,
            error,
        })
    }
}
