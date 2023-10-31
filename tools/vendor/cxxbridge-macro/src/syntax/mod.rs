// Functionality that is shared between the cxxbridge macro and the cmd.

pub mod atom;
pub mod attrs;
pub mod cfg;
pub mod check;
pub mod derive;
mod discriminant;
mod doc;
pub mod error;
pub mod file;
pub mod ident;
mod impls;
mod improper;
pub mod instantiate;
pub mod mangle;
pub mod map;
mod names;
pub mod namespace;
mod parse;
mod pod;
pub mod qualified;
pub mod report;
pub mod resolve;
pub mod set;
pub mod symbol;
mod tokens;
mod toposort;
pub mod trivial;
pub mod types;
mod visit;

use self::attrs::OtherAttrs;
use self::cfg::CfgExpr;
use self::namespace::Namespace;
use self::parse::kw;
use self::symbol::Symbol;
use proc_macro2::{Ident, Span};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Paren};
use syn::{Attribute, Expr, Generics, Lifetime, LitInt, Token, Type as RustType};

pub use self::atom::Atom;
pub use self::derive::{Derive, Trait};
pub use self::discriminant::Discriminant;
pub use self::doc::Doc;
pub use self::names::ForeignName;
pub use self::parse::parse_items;
pub use self::types::Types;

pub enum Api {
    Include(Include),
    Struct(Struct),
    Enum(Enum),
    CxxType(ExternType),
    CxxFunction(ExternFn),
    RustType(ExternType),
    RustFunction(ExternFn),
    TypeAlias(TypeAlias),
    Impl(Impl),
}

pub struct Include {
    pub cfg: CfgExpr,
    pub path: String,
    pub kind: IncludeKind,
    pub begin_span: Span,
    pub end_span: Span,
}

/// Whether to emit `#include "path"` or `#include <path>`.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum IncludeKind {
    /// `#include "quoted/path/to"`
    Quoted,
    /// `#include <bracketed/path/to>`
    Bracketed,
}

pub struct ExternType {
    pub cfg: CfgExpr,
    pub lang: Lang,
    pub doc: Doc,
    pub derives: Vec<Derive>,
    pub attrs: OtherAttrs,
    pub visibility: Token![pub],
    pub type_token: Token![type],
    pub name: Pair,
    pub generics: Lifetimes,
    pub colon_token: Option<Token![:]>,
    pub bounds: Vec<Derive>,
    pub semi_token: Token![;],
    pub trusted: bool,
}

pub struct Struct {
    pub cfg: CfgExpr,
    pub doc: Doc,
    pub derives: Vec<Derive>,
    pub attrs: OtherAttrs,
    pub visibility: Token![pub],
    pub struct_token: Token![struct],
    pub name: Pair,
    pub generics: Lifetimes,
    pub brace_token: Brace,
    pub fields: Vec<Var>,
}

pub struct Enum {
    pub cfg: CfgExpr,
    pub doc: Doc,
    pub derives: Vec<Derive>,
    pub attrs: OtherAttrs,
    pub visibility: Token![pub],
    pub enum_token: Token![enum],
    pub name: Pair,
    pub generics: Lifetimes,
    pub brace_token: Brace,
    pub variants: Vec<Variant>,
    pub variants_from_header: bool,
    pub variants_from_header_attr: Option<Attribute>,
    pub repr: EnumRepr,
    pub explicit_repr: bool,
}

pub enum EnumRepr {
    Native {
        atom: Atom,
        repr_type: Type,
    },
    #[cfg(feature = "experimental-enum-variants-from-header")]
    Foreign {
        rust_type: syn::Path,
    },
}

pub struct ExternFn {
    pub cfg: CfgExpr,
    pub lang: Lang,
    pub doc: Doc,
    pub attrs: OtherAttrs,
    pub visibility: Token![pub],
    pub name: Pair,
    pub sig: Signature,
    pub semi_token: Token![;],
    pub trusted: bool,
}

pub struct TypeAlias {
    pub cfg: CfgExpr,
    pub doc: Doc,
    pub derives: Vec<Derive>,
    pub attrs: OtherAttrs,
    pub visibility: Token![pub],
    pub type_token: Token![type],
    pub name: Pair,
    pub generics: Lifetimes,
    pub eq_token: Token![=],
    pub ty: RustType,
    pub semi_token: Token![;],
}

pub struct Impl {
    pub cfg: CfgExpr,
    pub impl_token: Token![impl],
    pub impl_generics: Lifetimes,
    pub negative: bool,
    pub ty: Type,
    pub ty_generics: Lifetimes,
    pub brace_token: Brace,
    pub negative_token: Option<Token![!]>,
}

#[derive(Clone, Default)]
pub struct Lifetimes {
    pub lt_token: Option<Token![<]>,
    pub lifetimes: Punctuated<Lifetime, Token![,]>,
    pub gt_token: Option<Token![>]>,
}

pub struct Signature {
    pub asyncness: Option<Token![async]>,
    pub unsafety: Option<Token![unsafe]>,
    pub fn_token: Token![fn],
    pub generics: Generics,
    pub receiver: Option<Receiver>,
    pub args: Punctuated<Var, Token![,]>,
    pub ret: Option<Type>,
    pub throws: bool,
    pub paren_token: Paren,
    pub throws_tokens: Option<(kw::Result, Token![<], Token![>])>,
}

pub struct Var {
    pub cfg: CfgExpr,
    pub doc: Doc,
    pub attrs: OtherAttrs,
    pub visibility: Token![pub],
    pub name: Pair,
    pub colon_token: Token![:],
    pub ty: Type,
}

pub struct Receiver {
    pub pinned: bool,
    pub ampersand: Token![&],
    pub lifetime: Option<Lifetime>,
    pub mutable: bool,
    pub var: Token![self],
    pub ty: NamedType,
    pub colon_token: Token![:],
    pub shorthand: bool,
    pub pin_tokens: Option<(kw::Pin, Token![<], Token![>])>,
    pub mutability: Option<Token![mut]>,
}

pub struct Variant {
    pub cfg: CfgExpr,
    pub doc: Doc,
    pub attrs: OtherAttrs,
    pub name: Pair,
    pub discriminant: Discriminant,
    pub expr: Option<Expr>,
}

pub enum Type {
    Ident(NamedType),
    RustBox(Box<Ty1>),
    RustVec(Box<Ty1>),
    UniquePtr(Box<Ty1>),
    SharedPtr(Box<Ty1>),
    WeakPtr(Box<Ty1>),
    Ref(Box<Ref>),
    Ptr(Box<Ptr>),
    Str(Box<Ref>),
    CxxVector(Box<Ty1>),
    Fn(Box<Signature>),
    Void(Span),
    SliceRef(Box<SliceRef>),
    Array(Box<Array>),
}

pub struct Ty1 {
    pub name: Ident,
    pub langle: Token![<],
    pub inner: Type,
    pub rangle: Token![>],
}

pub struct Ref {
    pub pinned: bool,
    pub ampersand: Token![&],
    pub lifetime: Option<Lifetime>,
    pub mutable: bool,
    pub inner: Type,
    pub pin_tokens: Option<(kw::Pin, Token![<], Token![>])>,
    pub mutability: Option<Token![mut]>,
}

pub struct Ptr {
    pub star: Token![*],
    pub mutable: bool,
    pub inner: Type,
    pub mutability: Option<Token![mut]>,
    pub constness: Option<Token![const]>,
}

pub struct SliceRef {
    pub ampersand: Token![&],
    pub lifetime: Option<Lifetime>,
    pub mutable: bool,
    pub bracket: Bracket,
    pub inner: Type,
    pub mutability: Option<Token![mut]>,
}

pub struct Array {
    pub bracket: Bracket,
    pub inner: Type,
    pub semi_token: Token![;],
    pub len: usize,
    pub len_token: LitInt,
}

#[derive(Copy, Clone, PartialEq)]
pub enum Lang {
    Cxx,
    Rust,
}

// An association of a defined Rust name with a fully resolved, namespace
// qualified C++ name.
#[derive(Clone)]
pub struct Pair {
    pub namespace: Namespace,
    pub cxx: ForeignName,
    pub rust: Ident,
}

// Wrapper for a type which needs to be resolved before it can be printed in
// C++.
#[derive(PartialEq, Eq, Hash)]
pub struct NamedType {
    pub rust: Ident,
    pub generics: Lifetimes,
}
