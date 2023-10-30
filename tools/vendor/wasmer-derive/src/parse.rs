use proc_macro2::Span;
use proc_macro_error::abort;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    token, Ident, LitBool, LitStr, Token,
};

pub enum WasmerAttr {
    Export {
        /// The identifier is an override, otherwise we use the field name as the name
        /// to lookup in `instance.exports`.
        identifier: Option<LitStr>,
        optional: bool,
        aliases: Vec<LitStr>,
        span: Span,
    },
}

#[derive(Debug)]
struct ExportExpr {
    name: Option<LitStr>,
    optional: bool,
    aliases: Vec<LitStr>,
}

#[derive(Debug)]
struct ExportOptions {
    name: Option<LitStr>,
    optional: bool,
    aliases: Vec<LitStr>,
}
impl Parse for ExportOptions {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut name = None;
        let mut optional: bool = false;
        let mut aliases: Vec<LitStr> = vec![];
        loop {
            let ident = input.parse::<Ident>()?;
            let _ = input.parse::<Token![=]>()?;
            let ident_str = ident.to_string();

            match ident_str.as_str() {
                "name" => {
                    name = Some(input.parse::<LitStr>()?);
                }
                "optional" => {
                    optional = input.parse::<LitBool>()?.value;
                }
                "alias" => {
                    let alias = input.parse::<LitStr>()?;
                    aliases.push(alias);
                }
                otherwise => {
                    abort!(
                        ident,
                        "Unrecognized argument in export options: expected `name = \"string\"`, `optional = bool`, or `alias = \"string\"` found `{}`",
                        otherwise
                    );
                }
            }

            match input.parse::<Token![,]>() {
                Ok(_) => continue,
                Err(_) => break,
            }
        }

        Ok(ExportOptions {
            name,
            optional,
            aliases,
        })
    }
}

impl Parse for ExportExpr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name;
        let optional;
        let aliases;
        if input.peek(Ident) {
            let options = input.parse::<ExportOptions>()?;
            name = options.name;
            optional = options.optional;
            aliases = options.aliases;
        } else {
            name = None;
            optional = false;
            aliases = vec![];
        }
        Ok(Self {
            name,
            optional,
            aliases,
        })
    }
}

// allows us to handle parens more cleanly
struct WasmerAttrInner(WasmerAttr);

impl Parse for WasmerAttrInner {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let ident_str = ident.to_string();
        let span = ident.span();
        let out = match ident_str.as_str() {
            "export" => {
                let export_expr;
                let (name, optional, aliases) = if input.peek(token::Paren) {
                    let _: token::Paren = parenthesized!(export_expr in input);

                    let expr = export_expr.parse::<ExportExpr>()?;
                    (expr.name, expr.optional, expr.aliases)
                } else {
                    (None, false, vec![])
                };

                WasmerAttr::Export {
                    identifier: name,
                    optional,
                    aliases,
                    span,
                }
            }
            otherwise => abort!(
                ident,
                "Unexpected identifier `{}`. Expected `export`.",
                otherwise
            ),
        };
        Ok(WasmerAttrInner(out))
    }
}

impl Parse for WasmerAttr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let attr_inner;
        parenthesized!(attr_inner in input);
        Ok(attr_inner.parse::<WasmerAttrInner>()?.0)
    }
}
