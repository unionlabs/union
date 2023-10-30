#![allow(clippy::let_underscore_untyped)]

use syn::ext::IdentExt;
use syn::parse::ParseStream;
use syn::{Ident, Token};

#[test]
fn test_peek() {
    let _ = |input: ParseStream| {
        let _ = input.peek(Ident);
        let _ = input.peek(Ident::peek_any);
        let _ = input.peek(Token![::]);
    };
}
