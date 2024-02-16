use proc_macro::{Delimiter, Group, Punct, Spacing, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn apply(meta: TokenStream, ts: TokenStream) -> TokenStream {
    let [ident @ TokenTree::Ident(_)]: [TokenTree; 1] =
        meta.into_iter().collect::<Vec<_>>().try_into().unwrap()
    else {
        panic!()
    };

    [
        ident,
        Punct::new('!', Spacing::Alone).into(),
        TokenTree::Group(Group::new(Delimiter::Brace, ts)),
    ]
    .into_iter()
    .collect()
}
