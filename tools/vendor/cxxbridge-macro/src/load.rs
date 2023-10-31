use crate::clang::{Clang, Node};
use crate::syntax::attrs::OtherAttrs;
use crate::syntax::cfg::CfgExpr;
use crate::syntax::namespace::Namespace;
use crate::syntax::report::Errors;
use crate::syntax::{Api, Discriminant, Doc, Enum, EnumRepr, ForeignName, Pair, Variant};
use flate2::write::GzDecoder;
use memmap::Mmap;
use proc_macro2::{Delimiter, Group, Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use std::env;
use std::fmt::{self, Display};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use syn::{parse_quote, Path};

const CXX_CLANG_AST: &str = "CXX_CLANG_AST";

pub fn load(cx: &mut Errors, apis: &mut [Api]) {
    let ref mut variants_from_header = Vec::new();
    for api in apis {
        if let Api::Enum(enm) = api {
            if enm.variants_from_header {
                if enm.variants.is_empty() {
                    variants_from_header.push(enm);
                } else {
                    let span = span_for_enum_error(enm);
                    cx.error(
                        span,
                        "enum with #![variants_from_header] must be written with no explicit variants",
                    );
                }
            }
        }
    }

    let span = match variants_from_header.get(0) {
        None => return,
        Some(enm) => enm.variants_from_header_attr.clone().unwrap(),
    };

    let ast_dump_path = match env::var_os(CXX_CLANG_AST) {
        Some(ast_dump_path) => PathBuf::from(ast_dump_path),
        None => {
            let msg = format!(
                "environment variable ${} has not been provided",
                CXX_CLANG_AST,
            );
            return cx.error(span, msg);
        }
    };

    let memmap = File::open(&ast_dump_path).and_then(|file| unsafe { Mmap::map(&file) });
    let mut gunzipped;
    let ast_dump_bytes = match match memmap {
        Ok(ref memmap) => {
            let is_gzipped = memmap.get(..2) == Some(b"\x1f\x8b");
            if is_gzipped {
                gunzipped = Vec::new();
                let decode_result = GzDecoder::new(&mut gunzipped).write_all(memmap);
                decode_result.map(|_| gunzipped.as_slice())
            } else {
                Ok(memmap as &[u8])
            }
        }
        Err(error) => Err(error),
    } {
        Ok(bytes) => bytes,
        Err(error) => {
            let msg = format!("failed to read {}: {}", ast_dump_path.display(), error);
            return cx.error(span, msg);
        }
    };

    let ref root: Node = match serde_json::from_slice(ast_dump_bytes) {
        Ok(root) => root,
        Err(error) => {
            let msg = format!("failed to read {}: {}", ast_dump_path.display(), error);
            return cx.error(span, msg);
        }
    };

    let ref mut namespace = Vec::new();
    traverse(cx, root, namespace, variants_from_header, None);

    for enm in variants_from_header {
        if enm.variants.is_empty() {
            let span = &enm.variants_from_header_attr;
            let name = CxxName(&enm.name);
            let msg = format!("failed to find any C++ definition of enum {}", name);
            cx.error(span, msg);
        }
    }
}

fn traverse<'a>(
    cx: &mut Errors,
    node: &'a Node,
    namespace: &mut Vec<&'a str>,
    variants_from_header: &mut [&mut Enum],
    mut idx: Option<usize>,
) {
    match &node.kind {
        Clang::NamespaceDecl(decl) => {
            let name = match &decl.name {
                Some(name) => name,
                // Can ignore enums inside an anonymous namespace.
                None => return,
            };
            namespace.push(name);
            idx = None;
        }
        Clang::EnumDecl(decl) => {
            let name = match &decl.name {
                Some(name) => name,
                None => return,
            };
            idx = None;
            for (i, enm) in variants_from_header.iter_mut().enumerate() {
                if enm.name.cxx == **name && enm.name.namespace.iter().eq(&*namespace) {
                    if !enm.variants.is_empty() {
                        let span = &enm.variants_from_header_attr;
                        let qual_name = CxxName(&enm.name);
                        let msg = format!("found multiple C++ definitions of enum {}", qual_name);
                        cx.error(span, msg);
                        return;
                    }
                    let fixed_underlying_type = match &decl.fixed_underlying_type {
                        Some(fixed_underlying_type) => fixed_underlying_type,
                        None => {
                            let span = &enm.variants_from_header_attr;
                            let name = &enm.name.cxx;
                            let qual_name = CxxName(&enm.name);
                            let msg = format!(
                                "implicit implementation-defined repr for enum {} is not supported yet; consider changing its C++ definition to `enum {}: int {{...}}",
                                qual_name, name,
                            );
                            cx.error(span, msg);
                            return;
                        }
                    };
                    let repr = translate_qual_type(
                        cx,
                        enm,
                        fixed_underlying_type
                            .desugared_qual_type
                            .as_ref()
                            .unwrap_or(&fixed_underlying_type.qual_type),
                    );
                    enm.repr = EnumRepr::Foreign { rust_type: repr };
                    idx = Some(i);
                    break;
                }
            }
            if idx.is_none() {
                return;
            }
        }
        Clang::EnumConstantDecl(decl) => {
            if let Some(idx) = idx {
                let enm = &mut *variants_from_header[idx];
                let span = enm
                    .variants_from_header_attr
                    .as_ref()
                    .unwrap()
                    .path
                    .get_ident()
                    .unwrap()
                    .span();
                let cxx_name = match ForeignName::parse(&decl.name, span) {
                    Ok(foreign_name) => foreign_name,
                    Err(_) => {
                        let span = &enm.variants_from_header_attr;
                        let msg = format!("unsupported C++ variant name: {}", decl.name);
                        return cx.error(span, msg);
                    }
                };
                let rust_name: Ident = match syn::parse_str(&decl.name) {
                    Ok(ident) => ident,
                    Err(_) => format_ident!("__Variant{}", enm.variants.len()),
                };
                let discriminant = match discriminant_value(&node.inner) {
                    ParsedDiscriminant::Constant(discriminant) => discriminant,
                    ParsedDiscriminant::Successor => match enm.variants.last() {
                        None => Discriminant::zero(),
                        Some(last) => match last.discriminant.checked_succ() {
                            Some(discriminant) => discriminant,
                            None => {
                                let span = &enm.variants_from_header_attr;
                                let msg = format!(
                                    "overflow processing discriminant value for variant: {}",
                                    decl.name,
                                );
                                return cx.error(span, msg);
                            }
                        },
                    },
                    ParsedDiscriminant::Fail => {
                        let span = &enm.variants_from_header_attr;
                        let msg = format!(
                            "failed to obtain discriminant value for variant: {}",
                            decl.name,
                        );
                        cx.error(span, msg);
                        Discriminant::zero()
                    }
                };
                enm.variants.push(Variant {
                    cfg: CfgExpr::Unconditional,
                    doc: Doc::new(),
                    attrs: OtherAttrs::none(),
                    name: Pair {
                        namespace: Namespace::ROOT,
                        cxx: cxx_name,
                        rust: rust_name,
                    },
                    discriminant,
                    expr: None,
                });
            }
        }
        _ => {}
    }
    for inner in &node.inner {
        traverse(cx, inner, namespace, variants_from_header, idx);
    }
    if let Clang::NamespaceDecl(_) = &node.kind {
        let _ = namespace.pop().unwrap();
    }
}

fn translate_qual_type(cx: &mut Errors, enm: &Enum, qual_type: &str) -> Path {
    let rust_std_name = match qual_type {
        "char" => "c_char",
        "int" => "c_int",
        "long" => "c_long",
        "long long" => "c_longlong",
        "signed char" => "c_schar",
        "short" => "c_short",
        "unsigned char" => "c_uchar",
        "unsigned int" => "c_uint",
        "unsigned long" => "c_ulong",
        "unsigned long long" => "c_ulonglong",
        "unsigned short" => "c_ushort",
        unsupported => {
            let span = &enm.variants_from_header_attr;
            let qual_name = CxxName(&enm.name);
            let msg = format!(
                "unsupported underlying type for {}: {}",
                qual_name, unsupported,
            );
            cx.error(span, msg);
            "c_int"
        }
    };
    let span = enm
        .variants_from_header_attr
        .as_ref()
        .unwrap()
        .path
        .get_ident()
        .unwrap()
        .span();
    let ident = Ident::new(rust_std_name, span);
    let path = quote_spanned!(span=> ::cxx::core::ffi::#ident);
    parse_quote!(#path)
}

enum ParsedDiscriminant {
    Constant(Discriminant),
    Successor,
    Fail,
}

fn discriminant_value(mut clang: &[Node]) -> ParsedDiscriminant {
    if clang.is_empty() {
        // No discriminant expression provided; use successor of previous
        // descriminant.
        return ParsedDiscriminant::Successor;
    }

    loop {
        if clang.len() != 1 {
            return ParsedDiscriminant::Fail;
        }

        let node = &clang[0];
        match &node.kind {
            Clang::ImplicitCastExpr => clang = &node.inner,
            Clang::ConstantExpr(expr) => match Discriminant::from_str(&expr.value) {
                Ok(discriminant) => return ParsedDiscriminant::Constant(discriminant),
                Err(_) => return ParsedDiscriminant::Fail,
            },
            _ => return ParsedDiscriminant::Fail,
        }
    }
}

fn span_for_enum_error(enm: &Enum) -> TokenStream {
    let enum_token = enm.enum_token;
    let mut brace_token = Group::new(Delimiter::Brace, TokenStream::new());
    brace_token.set_span(enm.brace_token.span);
    quote!(#enum_token #brace_token)
}

struct CxxName<'a>(&'a Pair);

impl<'a> Display for CxxName<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for namespace in &self.0.namespace {
            write!(formatter, "{}::", namespace)?;
        }
        write!(formatter, "{}", self.0.cxx)
    }
}
