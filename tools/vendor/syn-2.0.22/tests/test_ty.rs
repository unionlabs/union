#![allow(clippy::uninlined_format_args)]

#[macro_use]
mod macros;

use proc_macro2::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::quote;
use syn::Type;

#[test]
fn test_mut_self() {
    syn::parse_str::<Type>("fn(mut self)").unwrap();
    syn::parse_str::<Type>("fn(mut self,)").unwrap();
    syn::parse_str::<Type>("fn(mut self: ())").unwrap();
    syn::parse_str::<Type>("fn(mut self: ...)").unwrap_err();
    syn::parse_str::<Type>("fn(mut self: mut self)").unwrap_err();
    syn::parse_str::<Type>("fn(mut self::T)").unwrap_err();
}

#[test]
fn test_macro_variable_type() {
    // mimics the token stream corresponding to `$ty<T>`
    let tokens = TokenStream::from_iter(vec![
        TokenTree::Group(Group::new(Delimiter::None, quote! { ty })),
        TokenTree::Punct(Punct::new('<', Spacing::Alone)),
        TokenTree::Ident(Ident::new("T", Span::call_site())),
        TokenTree::Punct(Punct::new('>', Spacing::Alone)),
    ]);

    snapshot!(tokens as Type, @r###"
    Type::Path {
        path: Path {
            segments: [
                PathSegment {
                    ident: "ty",
                    arguments: PathArguments::AngleBracketed {
                        args: [
                            GenericArgument::Type(Type::Path {
                                path: Path {
                                    segments: [
                                        PathSegment {
                                            ident: "T",
                                        },
                                    ],
                                },
                            }),
                        ],
                    },
                },
            ],
        },
    }
    "###);

    // mimics the token stream corresponding to `$ty::<T>`
    let tokens = TokenStream::from_iter(vec![
        TokenTree::Group(Group::new(Delimiter::None, quote! { ty })),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Punct(Punct::new('<', Spacing::Alone)),
        TokenTree::Ident(Ident::new("T", Span::call_site())),
        TokenTree::Punct(Punct::new('>', Spacing::Alone)),
    ]);

    snapshot!(tokens as Type, @r###"
    Type::Path {
        path: Path {
            segments: [
                PathSegment {
                    ident: "ty",
                    arguments: PathArguments::AngleBracketed {
                        colon2_token: Some,
                        args: [
                            GenericArgument::Type(Type::Path {
                                path: Path {
                                    segments: [
                                        PathSegment {
                                            ident: "T",
                                        },
                                    ],
                                },
                            }),
                        ],
                    },
                },
            ],
        },
    }
    "###);
}

#[test]
fn test_group_angle_brackets() {
    // mimics the token stream corresponding to `Option<$ty>`
    let tokens = TokenStream::from_iter(vec![
        TokenTree::Ident(Ident::new("Option", Span::call_site())),
        TokenTree::Punct(Punct::new('<', Spacing::Alone)),
        TokenTree::Group(Group::new(Delimiter::None, quote! { Vec<u8> })),
        TokenTree::Punct(Punct::new('>', Spacing::Alone)),
    ]);

    snapshot!(tokens as Type, @r###"
    Type::Path {
        path: Path {
            segments: [
                PathSegment {
                    ident: "Option",
                    arguments: PathArguments::AngleBracketed {
                        args: [
                            GenericArgument::Type(Type::Group {
                                elem: Type::Path {
                                    path: Path {
                                        segments: [
                                            PathSegment {
                                                ident: "Vec",
                                                arguments: PathArguments::AngleBracketed {
                                                    args: [
                                                        GenericArgument::Type(Type::Path {
                                                            path: Path {
                                                                segments: [
                                                                    PathSegment {
                                                                        ident: "u8",
                                                                    },
                                                                ],
                                                            },
                                                        }),
                                                    ],
                                                },
                                            },
                                        ],
                                    },
                                },
                            }),
                        ],
                    },
                },
            ],
        },
    }
    "###);
}

#[test]
fn test_group_colons() {
    // mimics the token stream corresponding to `$ty::Item`
    let tokens = TokenStream::from_iter(vec![
        TokenTree::Group(Group::new(Delimiter::None, quote! { Vec<u8> })),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("Item", Span::call_site())),
    ]);

    snapshot!(tokens as Type, @r###"
    Type::Path {
        path: Path {
            segments: [
                PathSegment {
                    ident: "Vec",
                    arguments: PathArguments::AngleBracketed {
                        args: [
                            GenericArgument::Type(Type::Path {
                                path: Path {
                                    segments: [
                                        PathSegment {
                                            ident: "u8",
                                        },
                                    ],
                                },
                            }),
                        ],
                    },
                },
                PathSegment {
                    ident: "Item",
                },
            ],
        },
    }
    "###);

    let tokens = TokenStream::from_iter(vec![
        TokenTree::Group(Group::new(Delimiter::None, quote! { [T] })),
        TokenTree::Punct(Punct::new(':', Spacing::Joint)),
        TokenTree::Punct(Punct::new(':', Spacing::Alone)),
        TokenTree::Ident(Ident::new("Element", Span::call_site())),
    ]);

    snapshot!(tokens as Type, @r###"
    Type::Path {
        qself: Some(QSelf {
            ty: Type::Slice {
                elem: Type::Path {
                    path: Path {
                        segments: [
                            PathSegment {
                                ident: "T",
                            },
                        ],
                    },
                },
            },
            position: 0,
        }),
        path: Path {
            leading_colon: Some,
            segments: [
                PathSegment {
                    ident: "Element",
                },
            ],
        },
    }
    "###);
}

#[test]
fn test_trait_object() {
    let tokens = quote!(dyn for<'a> Trait<'a> + 'static);
    snapshot!(tokens as Type, @r###"
    Type::TraitObject {
        dyn_token: Some,
        bounds: [
            TypeParamBound::Trait(TraitBound {
                lifetimes: Some(BoundLifetimes {
                    lifetimes: [
                        GenericParam::Lifetime(LifetimeParam {
                            lifetime: Lifetime {
                                ident: "a",
                            },
                        }),
                    ],
                }),
                path: Path {
                    segments: [
                        PathSegment {
                            ident: "Trait",
                            arguments: PathArguments::AngleBracketed {
                                args: [
                                    GenericArgument::Lifetime(Lifetime {
                                        ident: "a",
                                    }),
                                ],
                            },
                        },
                    ],
                },
            }),
            TypeParamBound::Lifetime {
                ident: "static",
            },
        ],
    }
    "###);

    let tokens = quote!(dyn 'a + Trait);
    snapshot!(tokens as Type, @r###"
    Type::TraitObject {
        dyn_token: Some,
        bounds: [
            TypeParamBound::Lifetime {
                ident: "a",
            },
            TypeParamBound::Trait(TraitBound {
                path: Path {
                    segments: [
                        PathSegment {
                            ident: "Trait",
                        },
                    ],
                },
            }),
        ],
    }
    "###);

    // None of the following are valid Rust types.
    syn::parse_str::<Type>("for<'a> dyn Trait<'a>").unwrap_err();
    syn::parse_str::<Type>("dyn for<'a> 'a + Trait").unwrap_err();
}

#[test]
fn test_trailing_plus() {
    #[rustfmt::skip]
    let tokens = quote!(impl Trait +);
    snapshot!(tokens as Type, @r###"
    Type::ImplTrait {
        bounds: [
            TypeParamBound::Trait(TraitBound {
                path: Path {
                    segments: [
                        PathSegment {
                            ident: "Trait",
                        },
                    ],
                },
            }),
        ],
    }
    "###);

    #[rustfmt::skip]
    let tokens = quote!(dyn Trait +);
    snapshot!(tokens as Type, @r###"
    Type::TraitObject {
        dyn_token: Some,
        bounds: [
            TypeParamBound::Trait(TraitBound {
                path: Path {
                    segments: [
                        PathSegment {
                            ident: "Trait",
                        },
                    ],
                },
            }),
        ],
    }
    "###);

    #[rustfmt::skip]
    let tokens = quote!(Trait +);
    snapshot!(tokens as Type, @r###"
    Type::TraitObject {
        bounds: [
            TypeParamBound::Trait(TraitBound {
                path: Path {
                    segments: [
                        PathSegment {
                            ident: "Trait",
                        },
                    ],
                },
            }),
        ],
    }
    "###);
}
