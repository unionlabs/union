use quote::quote;
use validator_types::ValueOrPath;

pub fn lit_to_string(lit: &syn::Lit) -> Option<String> {
    match *lit {
        syn::Lit::Str(ref s) => Some(s.value()),
        _ => None,
    }
}

pub fn lit_to_int(lit: &syn::Lit) -> Option<u64> {
    match *lit {
        syn::Lit::Int(ref s) => Some(s.base10_parse().unwrap()),
        _ => None,
    }
}

pub fn lit_to_float(lit: &syn::Lit) -> Option<f64> {
    match *lit {
        syn::Lit::Float(ref s) => Some(s.base10_parse::<f64>().unwrap()),
        syn::Lit::Int(ref s) => Some(s.base10_parse::<f64>().unwrap()),
        _ => None,
    }
}

pub fn lit_to_u64_or_path(lit: &syn::Lit) -> Option<ValueOrPath<u64>> {
    let number = lit_to_int(lit);
    if let Some(number) = number {
        return Some(ValueOrPath::Value(number));
    }

    let path = lit_to_string(lit);
    if let Some(path) = path {
        return Some(ValueOrPath::Path(path));
    }

    None
}

pub fn lit_to_f64_or_path(lit: &syn::Lit) -> Option<ValueOrPath<f64>> {
    let number = lit_to_float(lit);
    if let Some(number) = number {
        return Some(ValueOrPath::Value(number));
    }

    let path = lit_to_string(lit);
    if let Some(path) = path {
        return Some(ValueOrPath::Path(path));
    }

    None
}

pub fn lit_to_bool(lit: &syn::Lit) -> Option<bool> {
    match *lit {
        syn::Lit::Bool(ref s) => Some(s.value),
        _ => None,
    }
}

pub fn option_to_tokens<T: quote::ToTokens>(opt: &Option<T>) -> proc_macro2::TokenStream {
    match opt {
        Some(ref t) => quote!(::std::option::Option::Some(#t)),
        None => quote!(::std::option::Option::None),
    }
}

pub fn value_or_path_to_tokens<T>(value: &ValueOrPath<T>) -> proc_macro2::TokenStream
where
    T: quote::ToTokens + std::clone::Clone + std::cmp::PartialEq + std::fmt::Debug,
{
    match value {
        ValueOrPath::Value(ref t) => quote!(#t),
        ValueOrPath::Path(ref path) => {
            // Global space
            let ident: syn::Path = syn::parse_str(path).unwrap();
            quote!(#ident)
        }
    }
}
