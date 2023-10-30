use proc_macro2::TokenStream;

use syn;
use try;

pub fn wrap_in_const(serde_path: Option<&syn::Path>, code: TokenStream) -> TokenStream {
    let try_replacement = try::replacement();

    let use_serde = match serde_path {
        Some(path) => quote! {
            use #path as _serde;
        },
        None => quote! {
            #[allow(unused_extern_crates, clippy::useless_attribute)]
            extern crate serde as _serde;
        },
    };

    quote! {
        #[doc(hidden)]
        #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
        const _: () = {
            #use_serde
            #try_replacement
            #code
        };
    }
}
