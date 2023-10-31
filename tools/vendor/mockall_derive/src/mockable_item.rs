// vim: tw=80
use super::*;

/// Performs transformations on a function to make it mockable
fn mockable_fn(mut item_fn: ItemFn) -> ItemFn {
    demutify(&mut item_fn.sig.inputs);
    deimplify(&mut item_fn.sig.output);
    item_fn
}

/// Performs transformations on an Item to make it mockable
fn mockable_item(item: Item) -> Item {
    match item {
        Item::Fn(item_fn) => Item::Fn(mockable_fn(item_fn)),
        x => x
    }
}

/// An item that's ready to be mocked.
///
/// It should be functionally identical or near-identical to the original item,
/// but with minor alterations that make it suitable for mocking, such as
/// altered lifetimes.
pub(crate) enum MockableItem {
    Module(MockableModule),
    Struct(MockableStruct)
}

impl From<(Attrs, Item)> for MockableItem {
    fn from((attrs, item): (Attrs, Item)) -> MockableItem {
        match item {
            Item::Impl(item_impl) =>
                MockableItem::Struct(MockableStruct::from(item_impl)),
            Item::ForeignMod(item_foreign_mod) =>
                MockableItem::Module(
                    MockableModule::from((attrs, item_foreign_mod))
                ),
            Item::Mod(item_mod) =>
                MockableItem::Module(MockableModule::from(item_mod)),
            Item::Trait(trait_) =>
                MockableItem::Struct(MockableStruct::from((attrs, trait_))),
            _ => panic!("automock does not support this item type")
        }
    }
}

impl From<MockableStruct> for MockableItem {
    fn from(mock: MockableStruct) -> MockableItem {
        MockableItem::Struct(mock)
    }
}

pub(crate) struct MockableModule {
    pub attrs: TokenStream,
    pub vis: Visibility,
    pub mock_ident: Ident,
    /// Ident of the original module, if any
    pub orig_ident: Option<Ident>,
    pub content: Vec<Item>
}

impl From<(Attrs, ItemForeignMod)> for MockableModule {
    fn from((attrs, foreign): (Attrs, ItemForeignMod)) -> MockableModule {
        let orig_ident = None;
        let mock_ident = attrs.modname.expect(concat!(
            "module name is required when mocking foreign functions,",
            " like `#[automock(mod mock_ffi)]`"
        ));
        let vis = Visibility::Public(VisPublic{
            pub_token: <Token![pub]>::default()
        });
        let attrs = quote!(
            #[deprecated(since = "0.9.0", note = "Using automock directly on an extern block is deprecated.  Instead, wrap the extern block in a module, and automock that, like #[automock] mod ffi { extern \"C\" { fn foo ... } }")]
        );
        let mut content = vec![
            // When mocking extern blocks, we pretend that they're modules, so
            // we need a "use super::*;" to ensure that types can resolve
            Item::Use(ItemUse {
                attrs: Vec::new(),
                vis: Visibility::Inherited,
                use_token: token::Use::default(),
                leading_colon: None,
                tree: UseTree::Path(UsePath {
                    ident: Ident::new("super", Span::call_site()),
                    colon2_token: token::Colon2::default(),
                    tree: Box::new(UseTree::Glob(UseGlob {
                        star_token: token::Star::default()
                    }))
                }),
                semi_token: token::Semi::default()
            })
        ];
        content.extend(foreign.items.into_iter()
            .map(|foreign_item| {
                match foreign_item {
                    ForeignItem::Fn(f) => {
                        let span = f.sig.span();
                        let mut sig = f.sig;

                        // When mocking extern blocks, we pretend that they're
                        // modules.  So we must supersuperfy everything by one
                        // level.
                        let vis = expectation_visibility(&f.vis, 1);

                        for arg in sig.inputs.iter_mut() {
                            if let FnArg::Typed(pt) = arg {
                                *pt.ty = supersuperfy(pt.ty.as_ref(), 1);
                            }
                        }
                        if let ReturnType::Type(_, ty) = &mut sig.output {
                                **ty = supersuperfy(&*ty, 1);
                        }

                        // Foreign functions are always unsafe.  Mock foreign
                        // functions should be unsafe too, to prevent "warning:
                        // unused unsafe" messages.
                        sig.unsafety = Some(Token![unsafe](span));
                        let block = Box::new(Block {
                            brace_token: token::Brace::default(),
                            stmts: Vec::new()
                        });

                        Item::Fn(
                            ItemFn {
                                attrs: f.attrs,
                                vis,
                                sig,
                                block
                            }
                        )
                    },
                    _ => {
                        compile_error(foreign_item.span(),
                            "Unsupported foreign item type"
                        );
                        Item::Verbatim(TokenStream::default())
                    }
                }
            }));
        MockableModule {
            attrs,
            vis,
            mock_ident,
            orig_ident,
            content
        }
    }
}

impl From<ItemMod> for MockableModule {
    fn from(mod_: ItemMod) -> MockableModule {
        let span = mod_.span();
        let vis = mod_.vis;
        let mock_ident = format_ident!("mock_{}", mod_.ident);
        let orig_ident = Some(mod_.ident);
        let content = if let Some((_, content)) = mod_.content {
            content.into_iter()
            .map(mockable_item)
            .collect()
        } else {
            compile_error(span,
            "automock can only mock inline modules, not modules from another file");
            Vec::new()
        };
        MockableModule {
            attrs: TokenStream::new(),
            vis,
            mock_ident,
            orig_ident,
            content
        }
    }
}
