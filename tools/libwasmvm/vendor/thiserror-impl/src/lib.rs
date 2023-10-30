#![allow(
    clippy::blocks_in_if_conditions,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::manual_find,
    clippy::manual_map,
    clippy::map_unwrap_or,
    clippy::needless_pass_by_value,
    clippy::option_if_let_else,
    clippy::range_plus_one,
    clippy::single_match_else,
    clippy::too_many_lines,
    clippy::wrong_self_convention
)]

extern crate proc_macro;

mod ast;
mod attr;
mod expand;
mod fmt;
mod generics;
mod prop;
mod valid;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Error, attributes(backtrace, error, from, source))]
pub fn derive_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand::derive(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
