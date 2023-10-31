// Check interoperability with rustc and clippy lints.

#![forbid(unsafe_code)]
// for old compilers
#![allow(unknown_lints)]
#![warn(nonstandard_style, rust_2018_idioms, unused)]
// Note: This does not guarantee compatibility with forbidding these lints in the future.
// If rustc adds a new lint, we may not be able to keep this.
#![forbid(future_incompatible, rust_2018_compatibility, rust_2021_compatibility)]
// lints forbidden as a part of future_incompatible, rust_2018_compatibility, and rust_2021_compatibility are not included in the list below.
// elided_lifetimes_in_paths, explicit_outlives_requirements, unused_extern_crates:  as a part of rust_2018_idioms
// unsafe_op_in_unsafe_fn: requires Rust 1.52. and, we don't generate unsafe fn.
// non_exhaustive_omitted_patterns: unstable
// unstable_features: no way to generate #![feature(..)] by macros, expect for unstable inner attribute. and this lint is deprecated: https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html#unstable-features
// unused_crate_dependencies, must_not_suspend: unrelated
// unsafe_code: checked in forbid_unsafe module
#![warn(
    box_pointers,
    deprecated_in_future,
    fuzzy_provenance_casts,
    lossy_provenance_casts,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences
)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::restriction)]
#![allow(clippy::blanket_clippy_restriction_lints)] // this is a test, so enable all restriction lints intentionally.
#![allow(clippy::exhaustive_structs, clippy::exhaustive_enums, clippy::single_char_lifetime_names)] // TODO

pub mod basic {
    include!("include/basic.rs");
}

pub mod box_pointers {
    use pin_project_lite::pin_project;

    pin_project! {
        #[derive(Debug)]
        pub struct Struct {
            #[pin]
            pub p: Box<isize>,
            pub u: Box<isize>,
        }
    }

    pin_project! {
        #[project = EnumProj]
        #[project_ref = EnumProjRef]
        #[derive(Debug)]
        pub enum Enum {
            Struct {
                #[pin]
                p: Box<isize>,
                u: Box<isize>,
            },
            Unit,
        }
    }
}

pub mod explicit_outlives_requirements {
    use pin_project_lite::pin_project;

    pin_project! {
        #[derive(Debug)]
        pub struct Struct<'a, T, U>
        where
            T: ?Sized,
            U: ?Sized,
        {
            #[pin]
            pub pinned: &'a mut T,
            pub unpinned: &'a mut U,
        }
    }

    pin_project! {
        #[project = EnumProj]
        #[project_ref = EnumProjRef]
        #[derive(Debug)]
        pub enum Enum<'a, T, U>
        where
            T: ?Sized,
            U: ?Sized,
        {
            Struct {
                #[pin]
                pinned: &'a mut T,
                unpinned: &'a mut U,
            },
            Unit,
        }
    }
}

pub mod variant_size_differences {
    use pin_project_lite::pin_project;

    pin_project! {
        #[project = EnumProj]
        #[project_ref = EnumProjRef]
        #[allow(missing_debug_implementations, missing_copy_implementations)] // https://github.com/rust-lang/rust/pull/74060
        #[allow(variant_size_differences)] // for the type itself
        #[allow(clippy::large_enum_variant)] // for the type itself
        pub enum Enum {
            V1 { f: u8 },
            V2 { f: [u8; 1024] },
        }
    }
}

pub mod clippy_mut_mut {
    use pin_project_lite::pin_project;

    pin_project! {
        #[derive(Debug)]
        pub struct Struct<'a, T, U> {
            #[pin]
            pub pinned: &'a mut T,
            pub unpinned: &'a mut U,
        }
    }

    pin_project! {
        #[project = EnumProj]
        #[project_ref = EnumProjRef]
        #[derive(Debug)]
        pub enum Enum<'a, T, U> {
            Struct {
                #[pin]
                pinned: &'a mut T,
                unpinned: &'a mut U,
            },
            Unit,
        }
    }
}

#[allow(unreachable_pub)]
mod clippy_redundant_pub_crate {
    use pin_project_lite::pin_project;

    pin_project! {
        #[derive(Debug)]
        pub struct Struct<T, U> {
            #[pin]
            pub pinned: T,
            pub unpinned: U,
        }
    }

    pin_project! {
        #[project = EnumProj]
        #[project_ref = EnumProjRef]
        #[derive(Debug)]
        pub enum Enum<T, U> {
            Struct {
                #[pin]
                pinned: T,
                unpinned: U,
            },
            Unit,
        }
    }
}

#[allow(clippy::use_self)]
pub mod clippy_type_repetition_in_bounds {
    use pin_project_lite::pin_project;

    pin_project! {
        #[derive(Debug)]
        pub struct Struct<T, U>
        where
            Struct<T, U>: Sized,
        {
            #[pin]
            pub pinned: T,
            pub unpinned: U,
        }
    }

    pin_project! {
        #[project = EnumProj]
        #[project_ref = EnumProjRef]
        #[derive(Debug)]
        pub enum Enum<T, U>
        where
            Enum<T, U>: Sized,
        {
            Struct {
                #[pin]
                pinned: T,
                unpinned: U,
            },
            Unit,
        }
    }
}

pub mod clippy_used_underscore_binding {
    use pin_project_lite::pin_project;

    pin_project! {
        #[derive(Debug)]
        pub struct Struct<T, U> {
            #[pin]
            pub _pinned: T,
            pub _unpinned: U,
        }
    }

    pin_project! {
        #[project = EnumProj]
        #[project_ref = EnumProjRef]
        #[derive(Debug)]
        pub enum Enum<T, U> {
            Struct {
                #[pin]
                _pinned: T,
                _unpinned: U,
            },
        }
    }
}

pub mod clippy_ref_option_ref {
    use pin_project_lite::pin_project;

    pin_project! {
        pub struct Struct<'a> {
            #[pin]
            pub _pinned: Option<&'a ()>,
            pub _unpinned: Option<&'a ()>,
        }
    }

    pin_project! {
        #[project = EnumProj]
        #[project_ref = EnumProjRef]
        pub enum Enum<'a> {
            Struct {
                #[pin]
                _pinned: Option<&'a ()>,
                _unpinned: Option<&'a ()>,
            },
        }
    }
}
