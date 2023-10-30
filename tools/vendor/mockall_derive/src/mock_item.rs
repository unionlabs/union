// vim: tw=80
use super::*;

use crate::{
    mock_function::MockFunction,
    mockable_item::{MockableItem, MockableModule}
};

/// A Mock item
pub(crate) enum MockItem {
    Module(MockItemModule),
    Struct(MockItemStruct)
}

impl From<MockableItem> for MockItem {
    fn from(mockable: MockableItem) -> MockItem {
        match mockable {
            MockableItem::Struct(s) => MockItem::Struct(
                MockItemStruct::from(s)
            ),
            MockableItem::Module(mod_) => MockItem::Module(
                MockItemModule::from(mod_)
            )
        }
    }
}

impl ToTokens for MockItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            MockItem::Module(mod_) => mod_.to_tokens(tokens),
            MockItem::Struct(s) => s.to_tokens(tokens)
        }
    }
}

enum MockItemContent {
    Fn(Box<MockFunction>),
    Tokens(TokenStream)
}

pub(crate) struct MockItemModule {
    attrs: TokenStream,
    vis: Visibility,
    mock_ident: Ident,
    orig_ident: Option<Ident>,
    content: Vec<MockItemContent>
}

impl From<MockableModule> for MockItemModule {
    fn from(mod_: MockableModule) -> MockItemModule {
        let mock_ident = mod_.mock_ident.clone();
        let orig_ident = mod_.orig_ident;
        let mut content = Vec::new();
        for item in mod_.content.into_iter() {
            let span = item.span();
            match item {
                Item::ExternCrate(_) | Item::Impl(_) =>
                {
                    // Ignore
                },
                Item::Static(is) => {
                    content.push(
                        MockItemContent::Tokens(is.into_token_stream())
                    );
                },
                Item::Const(ic) => {
                    content.push(
                        MockItemContent::Tokens(ic.into_token_stream())
                    );
                },
                Item::Fn(f) => {
                    let mf = mock_function::Builder::new(&f.sig, &f.vis)
                        .attrs(&f.attrs)
                        .parent(&mock_ident)
                        .levels(1)
                        .call_levels(0)
                        .build();
                    content.push(MockItemContent::Fn(Box::new(mf)));
                },
                Item::ForeignMod(ifm) => {
                    for item in ifm.items {
                        if let ForeignItem::Fn(mut f) = item {
                            // Foreign functions are always unsafe.  Mock
                            // foreign functions should be unsafe too, to
                            // prevent "warning: unused unsafe" messages.
                            f.sig.unsafety = Some(Token![unsafe](f.span()));
                            let mf = mock_function::Builder::new(&f.sig, &f.vis)
                                .attrs(&f.attrs)
                                .parent(&mock_ident)
                                .levels(1)
                                .call_levels(0)
                                .build();
                            content.push(MockItemContent::Fn(Box::new(mf)));
                        } else {
                            compile_error(item.span(),
                                "Mockall does not yet support  this type in this position.  Please open an issue with your use case at https://github.com/asomers/mockall");
                        }
                    }
                },
                Item::Mod(_)
                    | Item::Struct(_) | Item::Enum(_)
                    | Item::Union(_) | Item::Trait(_) =>
                {
                    compile_error(span,
                        "Mockall does not yet support deriving nested mocks");
                },
                Item::Type(ty) => {
                    content.push(
                        MockItemContent::Tokens(ty.into_token_stream())
                    );
                },
                Item::TraitAlias(ta) => {
                    content.push
                        (MockItemContent::Tokens(ta.into_token_stream())
                    );
                },
                Item::Use(u) => {
                    content.push(
                        MockItemContent::Tokens(u.into_token_stream())
                    );
                },
                _ => compile_error(span, "Unsupported item")
            }
        }
        MockItemModule {
            attrs: mod_.attrs,
            vis: mod_.vis,
            mock_ident: mod_.mock_ident,
            orig_ident,
            content
        }
    }
}

impl ToTokens for MockItemModule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut body = TokenStream::new();
        let mut cp_body = TokenStream::new();
        let attrs = &self.attrs;
        let modname = &self.mock_ident;
        let vis = &self.vis;

        for item in self.content.iter() {
            match item {
                MockItemContent::Tokens(ts) => ts.to_tokens(&mut body),
                MockItemContent::Fn(f) => {
                    let call = f.call(None);
                    let ctx_fn = f.context_fn(None);
                    let priv_mod = f.priv_module();
                    quote!(
                        #priv_mod
                        #call
                        #ctx_fn
                    ).to_tokens(&mut body);
                    f.checkpoint().to_tokens(&mut cp_body);
                },
            }
        }

        quote!(
            /// Verify that all current expectations for every function in
            /// this module are satisfied and clear them.
            pub fn checkpoint() { #cp_body }
        ).to_tokens(&mut body);
        let docstr = {
            if let Some(ident) = &self.orig_ident {
                let inner = format!("Mock version of the `{}` module", ident);
                quote!( #[doc = #inner])
            } else {
                // Typically an extern FFI block.  Not really anything good we
                // can put in the doc string.
                quote!(#[allow(missing_docs)])
            }
        };
        quote!(
            #[allow(unused_imports)]
            #attrs
            #docstr
            #vis mod #modname {
                #body
        }).to_tokens(tokens);
    }
}
