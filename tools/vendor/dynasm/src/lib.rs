#![cfg_attr(feature = "filelocal", feature(proc_macro_span))]
//! The dynasm crate contains the procedural macros that power the magic of dynasm-rs. It seamlessly integrates
//! a full dynamic assembler for several assembly dialects with rust code.
//! 
//! As this is a proc-macro crate, it only exports the `dynasm!` and `dynasm_backwards!` macros.
//! Any directives used in these macro invocations are normally local to the invocation itself, unless
//! the `filelocal` crate feature is used. This feature requires a nightly compiler.
//!
//! Additionally, the `dynasm_opmap` and `dynasm_extract` development features can be used to export two additional macros
//! with matching names. These macros expand into instruction listing overviews and are normally only used for documentation purposes.

extern crate proc_macro;

use syn::parse;
use syn::{Token, parse_macro_input};
use proc_macro2::{TokenTree, TokenStream};
use quote::quote;
use proc_macro_error::proc_macro_error;

use std::collections::HashMap;

#[cfg(feature = "filelocal")]
use std::sync::{MutexGuard, Mutex};
#[cfg(feature = "filelocal")]
use std::path::PathBuf;
#[cfg(any(feature = "filelocal", feature = "dynasm_opmap", feature = "dynasm_extract"))]
use proc_macro2::Span;

/// Module with common infrastructure across assemblers
mod common;
/// Module with architecture-specific assembler implementations
mod arch;
/// Module contaning the implementation of directives
mod directive;
/// Module containing utility functions for creating TokenTrees from assembler / directive output
mod serialize;
/// Module containing utility functions for parsing
mod parse_helpers;

/// The whole point. This macro compiles given assembly/rust templates down to `DynasmApi` and `DynasmLabelApi`
/// compliant calls to an assembler.
#[proc_macro]
#[proc_macro_error]
pub fn dynasm(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // try parsing the tokenstream into a dynasm struct containing
    // an abstract representation of the statements to create
    let dynasm = parse_macro_input!(tokens as Dynasm);

    // serialize the resulting output into tokens
    serialize::serialize(&dynasm.target, dynasm.stmts).into()
}

/// Similar to `dynasm!`, but the calls to the assembler are executed in piecewise reversed order.
/// This is to allow the system to be used with assemblers that assemble backwards.
/// Currently this is not supported by the `dynasmrt` crate, but this allows experimentation with it
/// out of tree.
#[proc_macro]
#[proc_macro_error]
pub fn dynasm_backwards(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // try parsing the tokenstream into a dynasm struct containing
    // an abstract representation of the statements to create
    let dynasm = parse_macro_input!(tokens as Dynasm);

    // reverse the statement stream
    let stmts = serialize::invert(dynasm.stmts);

    // serialize the resulting output into tokens
    serialize::serialize(&dynasm.target, stmts).into()
}

/// output from parsing a full dynasm invocation. target represents the first dynasm argument, being the assembler
/// variable being used. stmts contains an abstract representation of the statements to be generated from this dynasm
/// invocation.
struct Dynasm {
    target: TokenTree,
    stmts: Vec<common::Stmt>
}

/// top-level parsing. Handles common prefix symbols and diverts to the selected architecture
/// when an assembly instruction is encountered. When parsing fails an Err() is returned, when
/// non-parsing errors happen err() will be called, but this function returns Ok().
impl parse::Parse for Dynasm {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {

        // parse the assembler target declaration
        let target: syn::Expr = input.parse()?;
        // and just convert it back to a tokentree since that's how we'll always be using it.
        let target = common::delimited(target);

        // get file-local data (alias definitions, current architecture)
        let mut provider = ContextProvider::new();
        let invocation_context = provider.get_context_mut();

        // prepare the statement buffer
        let mut stmts = Vec::new();

        // if we're not at the end of the macro, we should be expecting a semicolon and a new directive/statement/label/op
        while !input.is_empty() {
            let _: Token![;] = input.parse()?;

            // ;; stmt
            if input.peek(Token![;]) {
                let _: Token![;] = input.parse()?;

                // collect all tokentrees till the next ;
                let mut buffer = TokenStream::new();
                while !(input.is_empty() || input.peek(Token![;])) {
                    buffer.extend(std::iter::once(input.parse::<TokenTree>()?));
                }
                // glue an extra ; on there
                buffer.extend(quote! { ; } );

                if !buffer.is_empty() {
                    // ensure that the statement is actually a proper statement and then emit it for serialization
                    let stmt: syn::Stmt = syn::parse2(buffer)?;
                    stmts.push(common::Stmt::Stmt(common::delimited(stmt)));
                }
                continue;
            }

            // ; -> label :
            if input.peek(Token![->]) {
                let _: Token![->] = input.parse()?;

                let name: syn::Ident = input.parse()?;
                let _: Token![:] = input.parse()?;

                stmts.push(common::Stmt::GlobalLabel(name));
                continue;
            }

            // ; => expr
            if input.peek(Token![=>]) {
                let _: Token![=>] = input.parse()?;

                let expr: syn::Expr = input.parse()?;

                stmts.push(common::Stmt::DynamicLabel(common::delimited(expr)));
                continue;
            }

            // ; label :
            if input.peek(syn::Ident) && input.peek2(Token![:]) {

                let name: syn::Ident = input.parse()?;
                let _: Token![:] = input.parse()?;

                stmts.push(common::Stmt::LocalLabel(name));
                continue;
            }


            // ; . directive
            if input.peek(Token![.]) {
                let _: Token![.] = input.parse()?;

                directive::evaluate_directive(invocation_context, &mut stmts, input)?;
            } else {
                // anything else is an assembly instruction which should be in current_arch

                let mut state = State {
                    stmts: &mut stmts,
                    target: &target,
                    invocation_context: &*invocation_context,
                };
                invocation_context.current_arch.compile_instruction(&mut state, input)?;
            }

        }

        Ok(Dynasm {
            target,
            stmts
        })
    }
}

/// This is only compiled when the dynasm_opmap feature is used. It exports the internal assembly listings
/// into a string that can then be included into the documentation for dynasm.
#[cfg(feature = "dynasm_opmap")]
#[proc_macro]
pub fn dynasm_opmap(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {

    // parse to ensure that no macro arguments were provided
    let opmap = parse_macro_input!(tokens as DynasmOpmap);

    let mut s = String::new();
    s.push_str("% Instruction Reference\n\n");

    s.push_str(&match opmap.arch.as_str() {
        "x64" | "x86" => arch::x64::create_opmap(),
        "aarch64" => arch::aarch64::create_opmap(),
        x => panic!("Unknown architecture {}", x)
    });

    let token = quote::quote_spanned! { Span::mixed_site()=>
        #s
    };
    token.into()
}

/// This is only compiled when the dynasm_extract feature is used. It exports the internal assembly listings
/// into a string that can then be included into the documentation for dynasm.
#[cfg(feature = "dynasm_extract")]
#[proc_macro]
pub fn dynasm_extract(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {

    // parse to ensure that no macro arguments were provided
    let opmap = parse_macro_input!(tokens as DynasmOpmap);

    let s = match opmap.arch.as_str() {
        "x64" | "x86" => "UNIMPLEMENTED".into(),
        "aarch64" => arch::aarch64::extract_opmap(),
        x => panic!("Unknown architecture {}", x)
    };

    let token = quote::quote_spanned! { Span::mixed_site()=>
        #s
    };
    token.into()
}

/// As dynasm_opmap takes no args it doesn't parse to anything
struct DynasmOpmap {
    pub arch: String
}

/// As dynasm_opmap takes no args it doesn't parse to anything.
/// This just exists so syn will give an error when no args are present.
impl parse::Parse for DynasmOpmap {
    fn parse(input: parse::ParseStream) -> parse::Result<Self> {
        let arch: syn::Ident = input.parse()?;

        Ok(DynasmOpmap {
            arch: arch.to_string()
        })
    }
}

/// This struct contains all non-parsing state that dynasm! requires while parsing and compiling
struct State<'a> {
    pub stmts: &'a mut Vec<common::Stmt>,
    pub target: &'a TokenTree,
    pub invocation_context: &'a DynasmContext,
}

// File local data implementation.

// context inside of a dynasm invocation, manipulated by directives and such
struct DynasmContext {
    pub current_arch: Box<dyn arch::Arch>,
    pub aliases: HashMap<String, String>,
}

impl DynasmContext {
    fn new() -> DynasmContext {
        DynasmContext {
            current_arch: arch::from_str(arch::CURRENT_ARCH).expect("Invalid default architecture"),
            aliases: HashMap::new()
        }
    }
}

// Oneshot context provider
#[cfg(not(feature = "filelocal"))]
struct ContextProvider {
    context: DynasmContext
}

#[cfg(not(feature = "filelocal"))]
impl ContextProvider {
    pub fn new() -> ContextProvider {
        ContextProvider {
            context: DynasmContext::new()
        }
    }

    pub fn get_context_mut(&mut self) -> &mut DynasmContext {
        &mut self.context
    }
}

/// Filelocal context provider
#[cfg(feature = "filelocal")]
struct ContextProvider {
    guard: MutexGuard<'static, HashMap<PathBuf, DynasmContext>>
}

#[cfg(feature = "filelocal")]
impl ContextProvider {
    pub fn new() -> ContextProvider {
        ContextProvider {
            guard: CONTEXT_STORAGE.lock().unwrap()
        }
    }

    pub fn get_context_mut(&mut self) -> &mut DynasmContext {
        // get the file that generated this macro expansion
        let span = Span::call_site().unstable();

        // and use the file that that was at as scope for resolving dynasm data
        let id = span.source_file().path();

        self.guard.entry(id).or_insert_with(DynasmContext::new)
    }
}

#[cfg(feature = "filelocal")]
lazy_static::lazy_static! {
    static ref CONTEXT_STORAGE: Mutex<HashMap<PathBuf, DynasmContext>> = Mutex::new(HashMap::new());
}
