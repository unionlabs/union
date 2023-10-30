#![allow(
    clippy::assertions_on_result_states,
    clippy::items_after_statements,
    clippy::non_ascii_literal,
    clippy::octal_escapes
)]

use proc_macro2::{Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use std::iter;
use std::panic;
use std::str::{self, FromStr};

#[test]
fn idents() {
    assert_eq!(
        Ident::new("String", Span::call_site()).to_string(),
        "String"
    );
    assert_eq!(Ident::new("fn", Span::call_site()).to_string(), "fn");
    assert_eq!(Ident::new("_", Span::call_site()).to_string(), "_");
}

#[test]
fn raw_idents() {
    assert_eq!(
        Ident::new_raw("String", Span::call_site()).to_string(),
        "r#String"
    );
    assert_eq!(Ident::new_raw("fn", Span::call_site()).to_string(), "r#fn");
}

#[test]
#[should_panic(expected = "`r#_` cannot be a raw identifier")]
fn ident_raw_underscore() {
    Ident::new_raw("_", Span::call_site());
}

#[test]
#[should_panic(expected = "`r#super` cannot be a raw identifier")]
fn ident_raw_reserved() {
    Ident::new_raw("super", Span::call_site());
}

#[test]
#[should_panic(expected = "Ident is not allowed to be empty; use Option<Ident>")]
fn ident_empty() {
    Ident::new("", Span::call_site());
}

#[test]
#[should_panic(expected = "Ident cannot be a number; use Literal instead")]
fn ident_number() {
    Ident::new("255", Span::call_site());
}

#[test]
#[should_panic(expected = "\"a#\" is not a valid Ident")]
fn ident_invalid() {
    Ident::new("a#", Span::call_site());
}

#[test]
#[should_panic(expected = "not a valid Ident")]
fn raw_ident_empty() {
    Ident::new("r#", Span::call_site());
}

#[test]
#[should_panic(expected = "not a valid Ident")]
fn raw_ident_number() {
    Ident::new("r#255", Span::call_site());
}

#[test]
#[should_panic(expected = "\"r#a#\" is not a valid Ident")]
fn raw_ident_invalid() {
    Ident::new("r#a#", Span::call_site());
}

#[test]
#[should_panic(expected = "not a valid Ident")]
fn lifetime_empty() {
    Ident::new("'", Span::call_site());
}

#[test]
#[should_panic(expected = "not a valid Ident")]
fn lifetime_number() {
    Ident::new("'255", Span::call_site());
}

#[test]
fn lifetime_invalid() {
    let result = panic::catch_unwind(|| Ident::new("'a#", Span::call_site()));
    match result {
        Err(box_any) => {
            let message = box_any.downcast_ref::<String>().unwrap();
            let expected1 = r#""\'a#" is not a valid Ident"#; // 1.31.0 .. 1.53.0
            let expected2 = r#""'a#" is not a valid Ident"#; // 1.53.0 ..
            assert!(
                message == expected1 || message == expected2,
                "panic message does not match expected string\n\
                 \x20   panic message: `{:?}`\n\
                 \x20expected message: `{:?}`",
                message,
                expected2,
            );
        }
        Ok(_) => panic!("test did not panic as expected"),
    }
}

#[test]
fn literal_string() {
    assert_eq!(Literal::string("foo").to_string(), "\"foo\"");
    assert_eq!(Literal::string("\"").to_string(), "\"\\\"\"");
    assert_eq!(Literal::string("didn't").to_string(), "\"didn't\"");
    assert_eq!(
        Literal::string("a\00b\07c\08d\0e\0").to_string(),
        "\"a\\x000b\\x007c\\08d\\0e\\0\"",
    );

    "\"\\\r\n    x\"".parse::<TokenStream>().unwrap();
    "\"\\\r\n  \rx\"".parse::<TokenStream>().unwrap_err();
}

#[test]
fn literal_raw_string() {
    "r\"\r\n\"".parse::<TokenStream>().unwrap();

    fn raw_string_literal_with_hashes(n: usize) -> String {
        let mut literal = String::new();
        literal.push('r');
        literal.extend(iter::repeat('#').take(n));
        literal.push('"');
        literal.push('"');
        literal.extend(iter::repeat('#').take(n));
        literal
    }

    raw_string_literal_with_hashes(255)
        .parse::<TokenStream>()
        .unwrap();

    // https://github.com/rust-lang/rust/pull/95251
    raw_string_literal_with_hashes(256)
        .parse::<TokenStream>()
        .unwrap_err();
}

#[test]
fn literal_byte_string() {
    assert_eq!(Literal::byte_string(b"").to_string(), "b\"\"");
    assert_eq!(
        Literal::byte_string(b"\0\t\n\r\"\\2\x10").to_string(),
        "b\"\\0\\t\\n\\r\\\"\\\\2\\x10\"",
    );
    assert_eq!(
        Literal::byte_string(b"a\00b\07c\08d\0e\0").to_string(),
        "b\"a\\x000b\\x007c\\08d\\0e\\0\"",
    );

    "b\"\\\r\n    x\"".parse::<TokenStream>().unwrap();
    "b\"\\\r\n  \rx\"".parse::<TokenStream>().unwrap_err();
    "b\"\\\r\n  \u{a0}x\"".parse::<TokenStream>().unwrap_err();
    "br\"\u{a0}\"".parse::<TokenStream>().unwrap_err();
}

#[test]
fn literal_c_string() {
    let strings = r###"
        c"hello\x80我叫\u{1F980}"  // from the RFC
        cr"\"
        cr##"Hello "world"!"##
        c"\t\n\r\"\\"
    "###;

    let mut tokens = strings.parse::<TokenStream>().unwrap().into_iter();

    for expected in &[
        r#"c"hello\x80我叫\u{1F980}""#,
        r#"cr"\""#,
        r###"cr##"Hello "world"!"##"###,
        r#"c"\t\n\r\"\\""#,
    ] {
        match tokens.next().unwrap() {
            TokenTree::Literal(literal) => {
                assert_eq!(literal.to_string(), *expected);
            }
            unexpected => panic!("unexpected token: {:?}", unexpected),
        }
    }

    if let Some(unexpected) = tokens.next() {
        panic!("unexpected token: {:?}", unexpected);
    }

    for invalid in &[r#"c"\0""#, r#"c"\x00""#, r#"c"\u{0}""#, "c\"\0\""] {
        if let Ok(unexpected) = invalid.parse::<TokenStream>() {
            panic!("unexpected token: {:?}", unexpected);
        }
    }
}

#[test]
fn literal_character() {
    assert_eq!(Literal::character('x').to_string(), "'x'");
    assert_eq!(Literal::character('\'').to_string(), "'\\''");
    assert_eq!(Literal::character('"').to_string(), "'\"'");
}

#[test]
fn literal_integer() {
    assert_eq!(Literal::u8_suffixed(10).to_string(), "10u8");
    assert_eq!(Literal::u16_suffixed(10).to_string(), "10u16");
    assert_eq!(Literal::u32_suffixed(10).to_string(), "10u32");
    assert_eq!(Literal::u64_suffixed(10).to_string(), "10u64");
    assert_eq!(Literal::u128_suffixed(10).to_string(), "10u128");
    assert_eq!(Literal::usize_suffixed(10).to_string(), "10usize");

    assert_eq!(Literal::i8_suffixed(10).to_string(), "10i8");
    assert_eq!(Literal::i16_suffixed(10).to_string(), "10i16");
    assert_eq!(Literal::i32_suffixed(10).to_string(), "10i32");
    assert_eq!(Literal::i64_suffixed(10).to_string(), "10i64");
    assert_eq!(Literal::i128_suffixed(10).to_string(), "10i128");
    assert_eq!(Literal::isize_suffixed(10).to_string(), "10isize");

    assert_eq!(Literal::u8_unsuffixed(10).to_string(), "10");
    assert_eq!(Literal::u16_unsuffixed(10).to_string(), "10");
    assert_eq!(Literal::u32_unsuffixed(10).to_string(), "10");
    assert_eq!(Literal::u64_unsuffixed(10).to_string(), "10");
    assert_eq!(Literal::u128_unsuffixed(10).to_string(), "10");
    assert_eq!(Literal::usize_unsuffixed(10).to_string(), "10");

    assert_eq!(Literal::i8_unsuffixed(10).to_string(), "10");
    assert_eq!(Literal::i16_unsuffixed(10).to_string(), "10");
    assert_eq!(Literal::i32_unsuffixed(10).to_string(), "10");
    assert_eq!(Literal::i64_unsuffixed(10).to_string(), "10");
    assert_eq!(Literal::i128_unsuffixed(10).to_string(), "10");
    assert_eq!(Literal::isize_unsuffixed(10).to_string(), "10");
}

#[test]
fn literal_float() {
    assert_eq!(Literal::f32_suffixed(10.0).to_string(), "10f32");
    assert_eq!(Literal::f64_suffixed(10.0).to_string(), "10f64");

    assert_eq!(Literal::f32_unsuffixed(10.0).to_string(), "10.0");
    assert_eq!(Literal::f64_unsuffixed(10.0).to_string(), "10.0");
}

#[test]
fn literal_suffix() {
    fn token_count(p: &str) -> usize {
        p.parse::<TokenStream>().unwrap().into_iter().count()
    }

    assert_eq!(token_count("999u256"), 1);
    assert_eq!(token_count("999r#u256"), 3);
    assert_eq!(token_count("1."), 1);
    assert_eq!(token_count("1.f32"), 3);
    assert_eq!(token_count("1.0_0"), 1);
    assert_eq!(token_count("1._0"), 3);
    assert_eq!(token_count("1._m"), 3);
    assert_eq!(token_count("\"\"s"), 1);
    assert_eq!(token_count("r\"\"r"), 1);
    assert_eq!(token_count("b\"\"b"), 1);
    assert_eq!(token_count("br\"\"br"), 1);
    assert_eq!(token_count("r#\"\"#r"), 1);
    assert_eq!(token_count("'c'c"), 1);
    assert_eq!(token_count("b'b'b"), 1);
    assert_eq!(token_count("0E"), 1);
    assert_eq!(token_count("0o0A"), 1);
    assert_eq!(token_count("0E--0"), 4);
    assert_eq!(token_count("0.0ECMA"), 1);
}

#[test]
fn literal_iter_negative() {
    let negative_literal = Literal::i32_suffixed(-3);
    let tokens = TokenStream::from(TokenTree::Literal(negative_literal));
    let mut iter = tokens.into_iter();
    match iter.next().unwrap() {
        TokenTree::Punct(punct) => {
            assert_eq!(punct.as_char(), '-');
            assert_eq!(punct.spacing(), Spacing::Alone);
        }
        unexpected => panic!("unexpected token {:?}", unexpected),
    }
    match iter.next().unwrap() {
        TokenTree::Literal(literal) => {
            assert_eq!(literal.to_string(), "3i32");
        }
        unexpected => panic!("unexpected token {:?}", unexpected),
    }
    assert!(iter.next().is_none());
}

#[test]
fn literal_parse() {
    assert!("1".parse::<Literal>().is_ok());
    assert!("-1".parse::<Literal>().is_ok());
    assert!("-1u12".parse::<Literal>().is_ok());
    assert!("1.0".parse::<Literal>().is_ok());
    assert!("-1.0".parse::<Literal>().is_ok());
    assert!("-1.0f12".parse::<Literal>().is_ok());
    assert!("'a'".parse::<Literal>().is_ok());
    assert!("\"\n\"".parse::<Literal>().is_ok());
    assert!("0 1".parse::<Literal>().is_err());
    assert!(" 0".parse::<Literal>().is_err());
    assert!("0 ".parse::<Literal>().is_err());
    assert!("/* comment */0".parse::<Literal>().is_err());
    assert!("0/* comment */".parse::<Literal>().is_err());
    assert!("0// comment".parse::<Literal>().is_err());
    assert!("- 1".parse::<Literal>().is_err());
    assert!("- 1.0".parse::<Literal>().is_err());
    assert!("-\"\"".parse::<Literal>().is_err());
}

#[test]
fn literal_span() {
    let positive = "0.1".parse::<Literal>().unwrap();
    let negative = "-0.1".parse::<Literal>().unwrap();
    let subspan = positive.subspan(1..2);

    #[cfg(not(span_locations))]
    {
        let _ = negative;
        assert!(subspan.is_none());
    }

    #[cfg(span_locations)]
    {
        assert_eq!(positive.span().start().column, 0);
        assert_eq!(positive.span().end().column, 3);
        assert_eq!(negative.span().start().column, 0);
        assert_eq!(negative.span().end().column, 4);
        assert_eq!(subspan.unwrap().source_text().unwrap(), ".");
    }

    assert!(positive.subspan(1..4).is_none());
}

#[test]
fn roundtrip() {
    fn roundtrip(p: &str) {
        println!("parse: {}", p);
        let s = p.parse::<TokenStream>().unwrap().to_string();
        println!("first: {}", s);
        let s2 = s.parse::<TokenStream>().unwrap().to_string();
        assert_eq!(s, s2);
    }
    roundtrip("a");
    roundtrip("<<");
    roundtrip("<<=");
    roundtrip(
        "
        1
        1.0
        1f32
        2f64
        1usize
        4isize
        4e10
        1_000
        1_0i32
        8u8
        9
        0
        0xffffffffffffffffffffffffffffffff
        1x
        1u80
        1f320
    ",
    );
    roundtrip("'a");
    roundtrip("'_");
    roundtrip("'static");
    roundtrip("'\\u{10__FFFF}'");
    roundtrip("\"\\u{10_F0FF__}foo\\u{1_0_0_0__}\"");
}

#[test]
fn fail() {
    fn fail(p: &str) {
        if let Ok(s) = p.parse::<TokenStream>() {
            panic!("should have failed to parse: {}\n{:#?}", p, s);
        }
    }
    fail("' static");
    fail("r#1");
    fail("r#_");
    fail("\"\\u{0000000}\""); // overlong unicode escape (rust allows at most 6 hex digits)
    fail("\"\\u{999999}\""); // outside of valid range of char
    fail("\"\\u{_0}\""); // leading underscore
    fail("\"\\u{}\""); // empty
    fail("b\"\r\""); // bare carriage return in byte string
    fail("r\"\r\""); // bare carriage return in raw string
    fail("\"\\\r  \""); // backslash carriage return
    fail("'aa'aa");
    fail("br##\"\"#");
    fail("\"\\\n\u{85}\r\"");
}

#[cfg(span_locations)]
#[test]
fn span_test() {
    check_spans(
        "\
/// This is a document comment
testing 123
{
  testing 234
}",
        &[
            (1, 0, 1, 30),  // #
            (1, 0, 1, 30),  // [ ... ]
            (1, 0, 1, 30),  // doc
            (1, 0, 1, 30),  // =
            (1, 0, 1, 30),  // "This is..."
            (2, 0, 2, 7),   // testing
            (2, 8, 2, 11),  // 123
            (3, 0, 5, 1),   // { ... }
            (4, 2, 4, 9),   // testing
            (4, 10, 4, 13), // 234
        ],
    );
}

#[cfg(procmacro2_semver_exempt)]
#[cfg(not(nightly))]
#[test]
fn default_span() {
    let start = Span::call_site().start();
    assert_eq!(start.line, 1);
    assert_eq!(start.column, 0);
    let end = Span::call_site().end();
    assert_eq!(end.line, 1);
    assert_eq!(end.column, 0);
    let source_file = Span::call_site().source_file();
    assert_eq!(source_file.path().to_string_lossy(), "<unspecified>");
    assert!(!source_file.is_real());
}

#[cfg(procmacro2_semver_exempt)]
#[test]
fn span_join() {
    let source1 = "aaa\nbbb"
        .parse::<TokenStream>()
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();
    let source2 = "ccc\nddd"
        .parse::<TokenStream>()
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();

    assert!(source1[0].span().source_file() != source2[0].span().source_file());
    assert_eq!(
        source1[0].span().source_file(),
        source1[1].span().source_file()
    );

    let joined1 = source1[0].span().join(source1[1].span());
    let joined2 = source1[0].span().join(source2[0].span());
    assert!(joined1.is_some());
    assert!(joined2.is_none());

    let start = joined1.unwrap().start();
    let end = joined1.unwrap().end();
    assert_eq!(start.line, 1);
    assert_eq!(start.column, 0);
    assert_eq!(end.line, 2);
    assert_eq!(end.column, 3);

    assert_eq!(
        joined1.unwrap().source_file(),
        source1[0].span().source_file()
    );
}

#[test]
fn no_panic() {
    let s = str::from_utf8(b"b\'\xc2\x86  \x00\x00\x00^\"").unwrap();
    assert!(s.parse::<TokenStream>().is_err());
}

#[test]
fn punct_before_comment() {
    let mut tts = TokenStream::from_str("~// comment").unwrap().into_iter();
    match tts.next().unwrap() {
        TokenTree::Punct(tt) => {
            assert_eq!(tt.as_char(), '~');
            assert_eq!(tt.spacing(), Spacing::Alone);
        }
        wrong => panic!("wrong token {:?}", wrong),
    }
}

#[test]
fn joint_last_token() {
    // This test verifies that we match the behavior of libproc_macro *not* in
    // the range nightly-2020-09-06 through nightly-2020-09-10, in which this
    // behavior was temporarily broken.
    // See https://github.com/rust-lang/rust/issues/76399

    let joint_punct = Punct::new(':', Spacing::Joint);
    let stream = TokenStream::from(TokenTree::Punct(joint_punct));
    let punct = match stream.into_iter().next().unwrap() {
        TokenTree::Punct(punct) => punct,
        _ => unreachable!(),
    };
    assert_eq!(punct.spacing(), Spacing::Joint);
}

#[test]
fn raw_identifier() {
    let mut tts = TokenStream::from_str("r#dyn").unwrap().into_iter();
    match tts.next().unwrap() {
        TokenTree::Ident(raw) => assert_eq!("r#dyn", raw.to_string()),
        wrong => panic!("wrong token {:?}", wrong),
    }
    assert!(tts.next().is_none());
}

#[test]
fn test_debug_ident() {
    let ident = Ident::new("proc_macro", Span::call_site());

    #[cfg(not(span_locations))]
    let expected = "Ident(proc_macro)";

    #[cfg(span_locations)]
    let expected = "Ident { sym: proc_macro }";

    assert_eq!(expected, format!("{:?}", ident));
}

#[test]
fn test_debug_tokenstream() {
    let tts = TokenStream::from_str("[a + 1]").unwrap();

    #[cfg(not(span_locations))]
    let expected = "\
TokenStream [
    Group {
        delimiter: Bracket,
        stream: TokenStream [
            Ident {
                sym: a,
            },
            Punct {
                char: '+',
                spacing: Alone,
            },
            Literal {
                lit: 1,
            },
        ],
    },
]\
    ";

    #[cfg(not(span_locations))]
    let expected_before_trailing_commas = "\
TokenStream [
    Group {
        delimiter: Bracket,
        stream: TokenStream [
            Ident {
                sym: a
            },
            Punct {
                char: '+',
                spacing: Alone
            },
            Literal {
                lit: 1
            }
        ]
    }
]\
    ";

    #[cfg(span_locations)]
    let expected = "\
TokenStream [
    Group {
        delimiter: Bracket,
        stream: TokenStream [
            Ident {
                sym: a,
                span: bytes(2..3),
            },
            Punct {
                char: '+',
                spacing: Alone,
                span: bytes(4..5),
            },
            Literal {
                lit: 1,
                span: bytes(6..7),
            },
        ],
        span: bytes(1..8),
    },
]\
    ";

    #[cfg(span_locations)]
    let expected_before_trailing_commas = "\
TokenStream [
    Group {
        delimiter: Bracket,
        stream: TokenStream [
            Ident {
                sym: a,
                span: bytes(2..3)
            },
            Punct {
                char: '+',
                spacing: Alone,
                span: bytes(4..5)
            },
            Literal {
                lit: 1,
                span: bytes(6..7)
            }
        ],
        span: bytes(1..8)
    }
]\
    ";

    let actual = format!("{:#?}", tts);
    if actual.ends_with(",\n]") {
        assert_eq!(expected, actual);
    } else {
        assert_eq!(expected_before_trailing_commas, actual);
    }
}

#[test]
fn default_tokenstream_is_empty() {
    let default_token_stream = <TokenStream as Default>::default();

    assert!(default_token_stream.is_empty());
}

#[test]
fn tokenstream_size_hint() {
    let tokens = "a b (c d) e".parse::<TokenStream>().unwrap();

    assert_eq!(tokens.into_iter().size_hint(), (4, Some(4)));
}

#[test]
fn tuple_indexing() {
    // This behavior may change depending on https://github.com/rust-lang/rust/pull/71322
    let mut tokens = "tuple.0.0".parse::<TokenStream>().unwrap().into_iter();
    assert_eq!("tuple", tokens.next().unwrap().to_string());
    assert_eq!(".", tokens.next().unwrap().to_string());
    assert_eq!("0.0", tokens.next().unwrap().to_string());
    assert!(tokens.next().is_none());
}

#[cfg(span_locations)]
#[test]
fn non_ascii_tokens() {
    check_spans("// abc", &[]);
    check_spans("// ábc", &[]);
    check_spans("// abc x", &[]);
    check_spans("// ábc x", &[]);
    check_spans("/* abc */ x", &[(1, 10, 1, 11)]);
    check_spans("/* ábc */ x", &[(1, 10, 1, 11)]);
    check_spans("/* ab\nc */ x", &[(2, 5, 2, 6)]);
    check_spans("/* áb\nc */ x", &[(2, 5, 2, 6)]);
    check_spans("/*** abc */ x", &[(1, 12, 1, 13)]);
    check_spans("/*** ábc */ x", &[(1, 12, 1, 13)]);
    check_spans(r#""abc""#, &[(1, 0, 1, 5)]);
    check_spans(r#""ábc""#, &[(1, 0, 1, 5)]);
    check_spans(r###"r#"abc"#"###, &[(1, 0, 1, 8)]);
    check_spans(r###"r#"ábc"#"###, &[(1, 0, 1, 8)]);
    check_spans("r#\"a\nc\"#", &[(1, 0, 2, 3)]);
    check_spans("r#\"á\nc\"#", &[(1, 0, 2, 3)]);
    check_spans("'a'", &[(1, 0, 1, 3)]);
    check_spans("'á'", &[(1, 0, 1, 3)]);
    check_spans("//! abc", &[(1, 0, 1, 7), (1, 0, 1, 7), (1, 0, 1, 7)]);
    check_spans("//! ábc", &[(1, 0, 1, 7), (1, 0, 1, 7), (1, 0, 1, 7)]);
    check_spans("//! abc\n", &[(1, 0, 1, 7), (1, 0, 1, 7), (1, 0, 1, 7)]);
    check_spans("//! ábc\n", &[(1, 0, 1, 7), (1, 0, 1, 7), (1, 0, 1, 7)]);
    check_spans("/*! abc */", &[(1, 0, 1, 10), (1, 0, 1, 10), (1, 0, 1, 10)]);
    check_spans("/*! ábc */", &[(1, 0, 1, 10), (1, 0, 1, 10), (1, 0, 1, 10)]);
    check_spans("/*! a\nc */", &[(1, 0, 2, 4), (1, 0, 2, 4), (1, 0, 2, 4)]);
    check_spans("/*! á\nc */", &[(1, 0, 2, 4), (1, 0, 2, 4), (1, 0, 2, 4)]);
    check_spans("abc", &[(1, 0, 1, 3)]);
    check_spans("ábc", &[(1, 0, 1, 3)]);
    check_spans("ábć", &[(1, 0, 1, 3)]);
    check_spans("abc// foo", &[(1, 0, 1, 3)]);
    check_spans("ábc// foo", &[(1, 0, 1, 3)]);
    check_spans("ábć// foo", &[(1, 0, 1, 3)]);
    check_spans("b\"a\\\n c\"", &[(1, 0, 2, 3)]);
}

#[cfg(span_locations)]
fn check_spans(p: &str, mut lines: &[(usize, usize, usize, usize)]) {
    let ts = p.parse::<TokenStream>().unwrap();
    check_spans_internal(ts, &mut lines);
    assert!(lines.is_empty(), "leftover ranges: {:?}", lines);
}

#[cfg(span_locations)]
fn check_spans_internal(ts: TokenStream, lines: &mut &[(usize, usize, usize, usize)]) {
    for i in ts {
        if let Some((&(sline, scol, eline, ecol), rest)) = lines.split_first() {
            *lines = rest;

            let start = i.span().start();
            assert_eq!(start.line, sline, "sline did not match for {}", i);
            assert_eq!(start.column, scol, "scol did not match for {}", i);

            let end = i.span().end();
            assert_eq!(end.line, eline, "eline did not match for {}", i);
            assert_eq!(end.column, ecol, "ecol did not match for {}", i);

            if let TokenTree::Group(g) = i {
                check_spans_internal(g.stream().clone(), lines);
            }
        }
    }
}

#[test]
fn whitespace() {
    // space, horizontal tab, vertical tab, form feed, carriage return, line
    // feed, non-breaking space, left-to-right mark, right-to-left mark
    let various_spaces = " \t\u{b}\u{c}\r\n\u{a0}\u{200e}\u{200f}";
    let tokens = various_spaces.parse::<TokenStream>().unwrap();
    assert_eq!(tokens.into_iter().count(), 0);

    let lone_carriage_return = " \r ";
    lone_carriage_return.parse::<TokenStream>().unwrap_err();
}

#[test]
fn byte_order_mark() {
    let string = "\u{feff}foo";
    let tokens = string.parse::<TokenStream>().unwrap();
    match tokens.into_iter().next().unwrap() {
        TokenTree::Ident(ident) => assert_eq!(ident, "foo"),
        _ => unreachable!(),
    }

    let string = "foo\u{feff}";
    string.parse::<TokenStream>().unwrap_err();
}
