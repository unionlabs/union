use proc_macro2::{Punct, Spacing, TokenStream};

// None of our generated code requires the `From::from` error conversion
// performed by the standard library's `try!` macro. With this simplified macro
// we see a significant improvement in type checking and borrow checking time of
// the generated code and a slight improvement in binary size.
pub fn replacement() -> TokenStream {
    // Cannot pass `$expr` to `quote!` prior to Rust 1.17.0 so interpolate it.
    let dollar = Punct::new('$', Spacing::Alone);

    quote! {
        #[allow(unused_macros)]
        macro_rules! try {
            (#dollar __expr:expr) => {
                match #dollar __expr {
                    _serde::__private::Ok(__val) => __val,
                    _serde::__private::Err(__err) => {
                        return _serde::__private::Err(__err);
                    }
                }
            }
        }
    }
}
