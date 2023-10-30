//! Cranelift code generation library.
#![deny(missing_docs, trivial_numeric_casts, unused_extern_crates)]
#![warn(unused_import_braces)]
#![cfg_attr(feature = "std", deny(unstable_features))]
#![cfg_attr(feature = "clippy", plugin(clippy(conf_file = "../../clippy.toml")))]
#![cfg_attr(feature="cargo-clippy", allow(
// Produces only a false positive:
                clippy::while_let_loop,
// Produces many false positives, but did produce some valid lints, now fixed:
                clippy::needless_lifetimes,
// Generated code makes some style transgressions, but readability doesn't suffer much:
                clippy::many_single_char_names,
                clippy::identity_op,
                clippy::needless_borrow,
                clippy::cast_lossless,
                clippy::unreadable_literal,
                clippy::assign_op_pattern,
                clippy::empty_line_after_outer_attr,
// Hard to avoid in generated code:
                clippy::cognitive_complexity,
                clippy::too_many_arguments,
// Code generator doesn't have a way to collapse identical arms:
                clippy::match_same_arms,
// These are relatively minor style issues, but would be easy to fix:
                clippy::new_without_default,
                clippy::should_implement_trait,
                clippy::len_without_is_empty))]
#![cfg_attr(
    feature = "cargo-clippy",
    warn(
        clippy::float_arithmetic,
        clippy::mut_mut,
        clippy::nonminimal_bool,
        clippy::map_unwrap_or,
        clippy::unicode_not_nfc,
        clippy::use_self
    )
)]
#![no_std]
// Various bits and pieces of this crate might only be used for one platform or
// another, but it's not really too useful to learn about that all the time. On
// CI we build at least one version of this crate with `--features all-arch`
// which means we'll always detect truly dead code, otherwise if this is only
// built for one platform we don't have to worry too much about trimming
// everything down.
#![cfg_attr(not(feature = "all-arch"), allow(dead_code))]

#[allow(unused_imports)] // #[macro_use] is required for no_std
#[macro_use]
extern crate alloc;

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(not(feature = "std"))]
use hashbrown::{hash_map, HashMap, HashSet};
#[cfg(feature = "std")]
use std::collections::{hash_map, HashMap, HashSet};

pub use crate::context::Context;
pub use crate::value_label::{ValueLabelsRanges, ValueLocRange};
pub use crate::verifier::verify_function;
pub use crate::write::write_function;

pub use cranelift_bforest as bforest;
pub use cranelift_entity as entity;
#[cfg(feature = "unwind")]
pub use gimli;

#[macro_use]
mod machinst;

pub mod binemit;
pub mod cfg_printer;
pub mod cursor;
pub mod data_value;
pub mod dbg;
pub mod dominator_tree;
pub mod flowgraph;
pub mod ir;
pub mod isa;
pub mod loop_analysis;
pub mod print_errors;
pub mod settings;
pub mod timing;
pub mod verifier;
pub mod write;

pub use crate::entity::packed_option;
pub use crate::machinst::buffer::{MachCallSite, MachReloc, MachSrcLoc, MachStackMap, MachTrap};
pub use crate::machinst::TextSectionBuilder;

mod bitset;
mod constant_hash;
mod context;
mod dce;
mod divconst_magic_numbers;
mod fx;
mod inst_predicates;
mod iterators;
mod legalizer;
mod licm;
mod log;
mod nan_canonicalization;
mod remove_constant_phis;
mod result;
mod scoped_hash_map;
mod simple_gvn;
mod simple_preopt;
mod unreachable_code;
mod value_label;

#[cfg(feature = "souper-harvest")]
mod souper_harvest;

pub use crate::result::{CodegenError, CodegenResult};

include!(concat!(env!("OUT_DIR"), "/version.rs"));
