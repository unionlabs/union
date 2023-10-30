#![allow(
    // clippy is broken and shows wrong warnings
    // clippy on stable does not know yet about the lint name
    unknown_lints,
    // https://github.com/rust-lang/rust-clippy/issues/8867
    clippy::derive_partial_eq_without_eq,
)]

mod deserialize_fromstr;
mod serialize_display;
#[path = "../utils.rs"]
mod utils;

use expect_test::expect;
use utils::*;
