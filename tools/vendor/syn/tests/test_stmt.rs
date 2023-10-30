#![allow(
    clippy::assertions_on_result_states,
    clippy::non_ascii_literal,
    clippy::uninlined_format_args
)]

#[macro_use]
mod macros;

use proc_macro2::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};
use quote::quote;
use syn::Stmt;

#[test]
fn test_raw_operator() {
    let stmt = syn::parse_str::<Stmt>("let _ = &raw const x;").unwrap();

    snapshot!(stmt, @r###"
    Stmt::Local {
        pat: Pat::Wild,
        init: Some(LocalInit {
            expr: Expr::Verbatim(`& raw const x`),
        }),
    }
    "###);
}

#[test]
fn test_raw_variable() {
    let stmt = syn::parse_str::<Stmt>("let _ = &raw;").unwrap();

    snapshot!(stmt, @r###"
    Stmt::Local {
        pat: Pat::Wild,
        init: Some(LocalInit {
            expr: Expr::Reference {
                expr: Expr::Path {
                    path: Path {
                        segments: [
                            PathSegment {
                                ident: "raw",
                            },
                        ],
                    },
                },
            },
        }),
    }
    "###);
}

#[test]
fn test_raw_invalid() {
    assert!(syn::parse_str::<Stmt>("let _ = &raw x;").is_err());
}

#[test]
fn test_none_group() {
    // <Ø async fn f() {} Ø>
    let tokens = TokenStream::from_iter(vec![TokenTree::Group(Group::new(
        Delimiter::None,
        TokenStream::from_iter(vec![
            TokenTree::Ident(Ident::new("async", Span::call_site())),
            TokenTree::Ident(Ident::new("fn", Span::call_site())),
            TokenTree::Ident(Ident::new("f", Span::call_site())),
            TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::Group(Group::new(Delimiter::Brace, TokenStream::new())),
        ]),
    ))]);

    snapshot!(tokens as Stmt, @r###"
    Stmt::Item(Item::Fn {
        vis: Visibility::Inherited,
        sig: Signature {
            asyncness: Some,
            ident: "f",
            generics: Generics,
            output: ReturnType::Default,
        },
        block: Block,
    })
    "###);
}

#[test]
fn test_let_dot_dot() {
    let tokens = quote! {
        let .. = 10;
    };

    snapshot!(tokens as Stmt, @r###"
    Stmt::Local {
        pat: Pat::Rest,
        init: Some(LocalInit {
            expr: Expr::Lit {
                lit: 10,
            },
        }),
    }
    "###);
}

#[test]
fn test_let_else() {
    let tokens = quote! {
        let Some(x) = None else { return 0; };
    };

    snapshot!(tokens as Stmt, @r###"
    Stmt::Local {
        pat: Pat::TupleStruct {
            path: Path {
                segments: [
                    PathSegment {
                        ident: "Some",
                    },
                ],
            },
            elems: [
                Pat::Ident {
                    ident: "x",
                },
            ],
        },
        init: Some(LocalInit {
            expr: Expr::Path {
                path: Path {
                    segments: [
                        PathSegment {
                            ident: "None",
                        },
                    ],
                },
            },
            diverge: Some(Expr::Block {
                block: Block {
                    stmts: [
                        Stmt::Expr(
                            Expr::Return {
                                expr: Some(Expr::Lit {
                                    lit: 0,
                                }),
                            },
                            Some,
                        ),
                    ],
                },
            }),
        }),
    }
    "###);
}

#[test]
fn test_macros() {
    let tokens = quote! {
        fn main() {
            macro_rules! mac {}
            thread_local! { static FOO }
            println!("");
            vec![]
        }
    };

    snapshot!(tokens as Stmt, @r###"
    Stmt::Item(Item::Fn {
        vis: Visibility::Inherited,
        sig: Signature {
            ident: "main",
            generics: Generics,
            output: ReturnType::Default,
        },
        block: Block {
            stmts: [
                Stmt::Item(Item::Macro {
                    ident: Some("mac"),
                    mac: Macro {
                        path: Path {
                            segments: [
                                PathSegment {
                                    ident: "macro_rules",
                                },
                            ],
                        },
                        delimiter: MacroDelimiter::Brace,
                        tokens: TokenStream(``),
                    },
                }),
                Stmt::Macro {
                    mac: Macro {
                        path: Path {
                            segments: [
                                PathSegment {
                                    ident: "thread_local",
                                },
                            ],
                        },
                        delimiter: MacroDelimiter::Brace,
                        tokens: TokenStream(`static FOO`),
                    },
                },
                Stmt::Macro {
                    mac: Macro {
                        path: Path {
                            segments: [
                                PathSegment {
                                    ident: "println",
                                },
                            ],
                        },
                        delimiter: MacroDelimiter::Paren,
                        tokens: TokenStream(`""`),
                    },
                    semi_token: Some,
                },
                Stmt::Expr(
                    Expr::Macro {
                        mac: Macro {
                            path: Path {
                                segments: [
                                    PathSegment {
                                        ident: "vec",
                                    },
                                ],
                            },
                            delimiter: MacroDelimiter::Bracket,
                            tokens: TokenStream(``),
                        },
                    },
                    None,
                ),
            ],
        },
    })
    "###);
}
