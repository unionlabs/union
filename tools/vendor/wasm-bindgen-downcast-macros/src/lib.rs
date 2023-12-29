extern crate proc_macro;

use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(DowncastJS)]
pub fn downcast_js(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive: DeriveInput = syn::parse_macro_input!(tokens);

    let name = &derive.ident;
    let internal = quote!(wasm_bindgen_downcast::internal);

    quote! {
        #[#internal::wasm_bindgen]
        impl #name {
            #[doc(hidden)]
            pub fn __wbgd_downcast_token() -> #internal::Symbol {
                <Self as wasm_bindgen_downcast::DowncastJS>::token().0.clone()
            }
        }

        unsafe impl wasm_bindgen_downcast::DowncastJS for #name {
            fn token() -> &'static #internal::DowncastToken {
                static TOKEN: #internal::Lazy<#internal::DowncastToken> =
                    #internal::Lazy::new(|| #internal::DowncastToken::unique());

                &TOKEN
            }
        }
    }
    .into()
}
