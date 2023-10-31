use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{Attribute, Generics, Ident, Meta, WhereClause};

pub fn contains_skip(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        if let Ok(Meta::Path(path)) = attr.parse_meta() {
            if path.to_token_stream().to_string().as_str() == "borsh_skip" {
                return true;
            }
        }
    }
    false
}

pub fn declaration(
    ident_str: &str,
    generics: &Generics,
    cratename: Ident,
) -> (TokenStream2, Vec<TokenStream2>) {
    // Generate function that returns the name of the type.
    let mut declaration_params = vec![];
    let mut where_clause = vec![];
    for type_param in generics.type_params() {
        let type_param_name = &type_param.ident;
        declaration_params.push(quote! {
            <#type_param_name>::declaration()
        });
        where_clause.push(quote! {
            #type_param_name: #cratename::BorshSchema
        });
    }
    let result = if declaration_params.is_empty() {
        quote! {
                #ident_str.to_string()
        }
    } else {
        quote! {
                let params = #cratename::maybestd::vec![#(#declaration_params),*];
                format!(r#"{}<{}>"#, #ident_str, params.join(", "))
        }
    };
    (result, where_clause)
}

pub fn quote_where_clause(
    where_clause: Option<&WhereClause>,
    additions: Vec<TokenStream2>,
) -> TokenStream2 {
    if let Some(WhereClause { predicates, .. }) = where_clause {
        // The original where clause might already have a trailing punctuation
        if predicates.trailing_punct() {
            quote! { where #predicates #(#additions),*}
        } else {
            quote! { where #predicates, #(#additions),*}
        }
    } else if additions.is_empty() {
        TokenStream2::new()
    } else {
        quote! { where #(#additions),*}
    }
}
