#![deny(
    anonymous_parameters,
    clippy::all,
    illegal_floating_point_literal_pattern,
    late_bound_lifetime_arguments,
    path_statements,
    patterns_in_fns_without_body,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unused_extern_crates
)]
#![warn(
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::get_unwrap,
    clippy::nursery,
    clippy::print_stdout,
    clippy::todo,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unwrap_used,
    clippy::use_debug,
    single_use_lifetimes,
    unused_qualifications,
    variant_size_differences
)]
#![allow(
    clippy::missing_const_for_fn, // useless in proc macro
    clippy::redundant_pub_crate, // suggests bad style
    clippy::option_if_let_else, // suggests terrible code
)]
#[allow(unused_macros)]
macro_rules! bug {
    () => { compile_error!("provide an error message to help fix a possible bug") };
    ($descr:literal $($rest:tt)?) => {
        unreachable!(concat!("internal error: ", $descr) $($rest)?)
    }
}

#[macro_use]
mod quote;

mod date;
mod datetime;
mod error;
#[cfg(any(feature = "formatting", feature = "parsing"))]
mod format_description;
mod helpers;
mod offset;
#[cfg(all(feature = "serde", any(feature = "formatting", feature = "parsing")))]
mod serde_format_description;
mod time;
mod to_tokens;

#[cfg(any(feature = "formatting", feature = "parsing"))]
use std::iter::Peekable;

use proc_macro::TokenStream;
#[cfg(any(feature = "formatting", feature = "parsing"))]
use proc_macro::{Ident, TokenTree};

use self::error::Error;

macro_rules! impl_macros {
    ($($name:ident)*) => {$(
        #[proc_macro]
        pub fn $name(input: TokenStream) -> TokenStream {
            use crate::to_tokens::ToTokenTree;

            let mut iter = input.into_iter().peekable();
            match $name::parse(&mut iter) {
                Ok(value) => match iter.peek() {
                    Some(tree) => Error::UnexpectedToken { tree: tree.clone() }.to_compile_error(),
                    None => TokenStream::from(value.into_token_tree()),
                },
                Err(err) => err.to_compile_error(),
            }
        }
    )*};
}

impl_macros![date datetime offset time];

#[cfg(any(feature = "formatting", feature = "parsing"))]
enum FormatDescriptionVersion {
    V1,
    V2,
}

#[cfg(any(feature = "formatting", feature = "parsing"))]
enum VersionOrModuleName {
    Version(FormatDescriptionVersion),
    ModuleName(Ident),
}

#[cfg(any(feature = "formatting", feature = "parsing"))]
fn parse_format_description_version<const NO_EQUALS_IS_MOD_NAME: bool>(
    iter: &mut Peekable<proc_macro::token_stream::IntoIter>,
) -> Result<Option<VersionOrModuleName>, Error> {
    let version_ident = match iter.peek() {
        Some(TokenTree::Ident(ident)) if ident.to_string() == "version" => match iter.next() {
            Some(TokenTree::Ident(ident)) => ident,
            _ => unreachable!(),
        },
        _ => return Ok(None),
    };
    match iter.peek() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => iter.next(),
        _ if NO_EQUALS_IS_MOD_NAME => {
            return Ok(Some(VersionOrModuleName::ModuleName(version_ident)));
        }
        Some(token) => {
            return Err(Error::Custom {
                message: "expected `=`".into(),
                span_start: Some(token.span()),
                span_end: Some(token.span()),
            });
        }
        None => {
            return Err(Error::Custom {
                message: "expected `=`".into(),
                span_start: None,
                span_end: None,
            });
        }
    };
    let version_literal = match iter.next() {
        Some(TokenTree::Literal(literal)) => literal,
        Some(token) => {
            return Err(Error::Custom {
                message: "expected 1 or 2".into(),
                span_start: Some(token.span()),
                span_end: Some(token.span()),
            });
        }
        None => {
            return Err(Error::Custom {
                message: "expected 1 or 2".into(),
                span_start: None,
                span_end: None,
            });
        }
    };
    let version = match version_literal.to_string().as_str() {
        "1" => FormatDescriptionVersion::V1,
        "2" => FormatDescriptionVersion::V2,
        _ => {
            return Err(Error::Custom {
                message: "invalid format description version".into(),
                span_start: Some(version_literal.span()),
                span_end: Some(version_literal.span()),
            });
        }
    };
    helpers::consume_punct(',', iter)?;

    Ok(Some(VersionOrModuleName::Version(version)))
}

#[cfg(any(feature = "formatting", feature = "parsing"))]
#[proc_macro]
pub fn format_description(input: TokenStream) -> TokenStream {
    (|| {
        let mut input = input.into_iter().peekable();
        let version = match parse_format_description_version::<false>(&mut input)? {
            Some(VersionOrModuleName::Version(version)) => Some(version),
            None => None,
            // This branch should never occur here, as `false` is the provided as a const parameter.
            Some(VersionOrModuleName::ModuleName(_)) => bug!("branch should never occur"),
        };
        let (span, string) = helpers::get_string_literal(input)?;
        let items = format_description::parse_with_version(version, &string, span)?;

        Ok(quote! {{
            const DESCRIPTION: &[::time::format_description::FormatItem<'_>] = &[#S(
                items
                    .into_iter()
                    .map(|item| quote! { #S(item), })
                    .collect::<TokenStream>()
            )];
            DESCRIPTION
        }})
    })()
    .unwrap_or_else(|err: Error| err.to_compile_error())
}

#[cfg(all(feature = "serde", any(feature = "formatting", feature = "parsing")))]
#[proc_macro]
pub fn serde_format_description(input: TokenStream) -> TokenStream {
    (|| {
        let mut tokens = input.into_iter().peekable();

        // First, the optional format description version.
        let version = parse_format_description_version::<true>(&mut tokens)?;
        let (version, mod_name) = match version {
            Some(VersionOrModuleName::ModuleName(module_name)) => (None, Some(module_name)),
            Some(VersionOrModuleName::Version(version)) => (Some(version), None),
            None => (None, None),
        };

        // Next, an identifier (the desired module name)
        // Only parse this if it wasn't parsed when attempting to get the version.
        let mod_name = match mod_name {
            Some(mod_name) => mod_name,
            None => match tokens.next() {
                Some(TokenTree::Ident(ident)) => Ok(ident),
                Some(tree) => Err(Error::UnexpectedToken { tree }),
                None => Err(Error::UnexpectedEndOfInput),
            }?,
        };

        // Followed by a comma
        helpers::consume_punct(',', &mut tokens)?;

        // Then, the type to create serde serializers for (e.g., `OffsetDateTime`).
        let formattable = match tokens.next() {
            Some(tree @ TokenTree::Ident(_)) => Ok(tree),
            Some(tree) => Err(Error::UnexpectedToken { tree }),
            None => Err(Error::UnexpectedEndOfInput),
        }?;

        // Another comma
        helpers::consume_punct(',', &mut tokens)?;

        // We now have two options. The user can either provide a format description as a string or
        // they can provide a path to a format description. If the latter, all remaining tokens are
        // assumed to be part of the path.
        let (format, format_description_display) = match tokens.peek() {
            // string literal
            Some(TokenTree::Literal(_)) => {
                let (span, format_string) = helpers::get_string_literal(tokens)?;
                let items = format_description::parse_with_version(version, &format_string, span)?;
                let items: TokenStream =
                    items.into_iter().map(|item| quote! { #S(item), }).collect();
                let items = quote! {
                    const ITEMS: &[::time::format_description::FormatItem<'_>] = &[#S(items)];
                    ITEMS
                };

                (items, String::from_utf8_lossy(&format_string).into_owned())
            }
            // path
            Some(_) => {
                let tokens = tokens.collect::<TokenStream>();
                let tokens_string = tokens.to_string();
                (
                    quote! {{
                        // We can't just do `super::path` because the path could be an absolute
                        // path. In that case, we'd be generating `super::::path`, which is invalid.
                        // Even if we took that into account, it's not possible to know if it's an
                        // external crate, which would just require emitting `path` directly. By
                        // taking this approach, we can leave it to the compiler to do the actual
                        // resolution.
                        mod __path_hack {
                            pub(super) use super::super::*;
                            pub(super) use #S(tokens) as FORMAT;
                        }
                        __path_hack::FORMAT
                    }},
                    tokens_string,
                )
            }
            None => return Err(Error::UnexpectedEndOfInput),
        };

        Ok(serde_format_description::build(
            mod_name,
            formattable,
            format,
            format_description_display,
        ))
    })()
    .unwrap_or_else(|err: Error| err.to_compile_error_standalone())
}
