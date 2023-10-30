// Check interoperability with rustc and clippy lints.

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

    pub mod inside_macro {
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[::pin_project::pin_project]
                #[derive(Debug)]
                pub struct DefaultStruct<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project(
                    project = DefaultStructNamedProj,
                    project_ref = DefaultStructNamedProjRef,
                )]
                #[derive(Debug)]
                pub struct DefaultStructNamed<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project]
                #[derive(Debug)]
                pub struct DefaultTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    project = DefaultTupleStructNamedProj,
                    project_ref = DefaultTupleStructNamedProjRef,
                )]
                #[derive(Debug)]
                pub struct DefaultTupleStructNamed<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    project = DefaultEnumProj,
                    project_ref = DefaultEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum DefaultEnum<T, U> {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }

                #[::pin_project::pin_project(PinnedDrop)]
                #[derive(Debug)]
                pub struct PinnedDropStruct<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropStruct<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                #[::pin_project::pin_project(PinnedDrop)]
                #[derive(Debug)]
                pub struct PinnedDropTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropTupleStruct<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                #[::pin_project::pin_project(
                    PinnedDrop,
                    project = PinnedDropEnumProj,
                    project_ref = PinnedDropEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum PinnedDropEnum<T, U> {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropEnum<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                #[::pin_project::pin_project(project_replace)]
                #[derive(Debug)]
                pub struct ReplaceStruct<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project(
                    project = ReplaceStructNamedProj,
                    project_ref = ReplaceStructNamedProjRef,
                    project_replace = ReplaceStructNamedProjOwn,
                )]
                #[derive(Debug)]
                pub struct ReplaceStructNamed<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project(project_replace)]
                #[derive(Debug)]
                pub struct ReplaceTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    project = ReplaceTupleStructNamedProj,
                    project_ref = ReplaceTupleStructNamedProjRef,
                    project_replace = ReplaceTupleStructNamedProjOwn,
                )]
                #[derive(Debug)]
                pub struct ReplaceTupleStructNamed<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    project = ReplaceEnumProj,
                    project_ref = ReplaceEnumProjRef,
                    project_replace = ReplaceEnumProjOwn,
                )]
                #[derive(Debug)]
                pub enum ReplaceEnum<T, U> {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }

                #[::pin_project::pin_project(UnsafeUnpin)]
                #[derive(Debug)]
                pub struct UnsafeUnpinStruct<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project(UnsafeUnpin)]
                #[derive(Debug)]
                pub struct UnsafeUnpinTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    UnsafeUnpin,
                    project = UnsafeUnpinEnumProj,
                    project_ref = UnsafeUnpinEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum UnsafeUnpinEnum<T, U> {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }

                #[::pin_project::pin_project(!Unpin)]
                #[derive(Debug)]
                pub struct NotUnpinStruct<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project(!Unpin)]
                #[derive(Debug)]
                pub struct NotUnpinTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    !Unpin,
                    project = NotUnpinEnumProj,
                    project_ref = NotUnpinEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum NotUnpinEnum<T, U> {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }

                #[allow(clippy::undocumented_unsafe_blocks)]
                unsafe impl<T: ::pin_project::__private::Unpin, U: ::pin_project::__private::Unpin>
                    ::pin_project::UnsafeUnpin for UnsafeUnpinStruct<T, U>
                {
                }
                #[allow(clippy::undocumented_unsafe_blocks)]
                unsafe impl<T: ::pin_project::__private::Unpin, U: ::pin_project::__private::Unpin>
                    ::pin_project::UnsafeUnpin for UnsafeUnpinTupleStruct<T, U>
                {
                }
                #[allow(clippy::undocumented_unsafe_blocks)]
                unsafe impl<T: ::pin_project::__private::Unpin, U: ::pin_project::__private::Unpin>
                    ::pin_project::UnsafeUnpin for UnsafeUnpinEnum<T, U>
                {
                }
            };
        }

        mac!();
    }
}

pub mod forbid_unsafe {
    #![forbid(unsafe_code)]

    include!("include/basic-safe-part.rs");

    pub mod inside_macro {
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[::pin_project::pin_project]
                #[derive(Debug)]
                pub struct DefaultStruct<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project(
                    project = DefaultStructNamedProj,
                    project_ref = DefaultStructNamedProjRef,
                )]
                #[derive(Debug)]
                pub struct DefaultStructNamed<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project]
                #[derive(Debug)]
                pub struct DefaultTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    project = DefaultTupleStructNamedProj,
                    project_ref = DefaultTupleStructNamedProjRef,
                )]
                #[derive(Debug)]
                pub struct DefaultTupleStructNamed<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    project = DefaultEnumProj,
                    project_ref = DefaultEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum DefaultEnum<T, U> {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }

                #[::pin_project::pin_project(PinnedDrop)]
                #[derive(Debug)]
                pub struct PinnedDropStruct<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropStruct<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                #[::pin_project::pin_project(PinnedDrop)]
                #[derive(Debug)]
                pub struct PinnedDropTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropTupleStruct<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                #[::pin_project::pin_project(
                    PinnedDrop,
                    project = PinnedDropEnumProj,
                    project_ref = PinnedDropEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum PinnedDropEnum<T, U> {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }

                #[::pin_project::pinned_drop]
                impl<T, U> PinnedDrop for PinnedDropEnum<T, U> {
                    fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }

                #[::pin_project::pin_project(project_replace)]
                #[derive(Debug)]
                pub struct ReplaceStruct<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project(
                    project = ReplaceStructNamedProj,
                    project_ref = ReplaceStructNamedProjRef,
                    project_replace = ReplaceStructNamedProjOwn,
                )]
                #[derive(Debug)]
                pub struct ReplaceStructNamed<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project(project_replace)]
                #[derive(Debug)]
                pub struct ReplaceTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    project = ReplaceTupleStructNamedProj,
                    project_ref = ReplaceTupleStructNamedProjRef,
                    project_replace = ReplaceTupleStructNamedProjOwn,
                )]
                #[derive(Debug)]
                pub struct ReplaceTupleStructNamed<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    project = ReplaceEnumProj,
                    project_ref = ReplaceEnumProjRef,
                    project_replace = ReplaceEnumProjOwn,
                )]
                #[derive(Debug)]
                pub enum ReplaceEnum<T, U> {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }

                #[::pin_project::pin_project(UnsafeUnpin)]
                #[derive(Debug)]
                pub struct UnsafeUnpinStruct<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project(UnsafeUnpin)]
                #[derive(Debug)]
                pub struct UnsafeUnpinTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    UnsafeUnpin,
                    project = UnsafeUnpinEnumProj,
                    project_ref = UnsafeUnpinEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum UnsafeUnpinEnum<T, U> {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }

                #[::pin_project::pin_project(!Unpin)]
                #[derive(Debug)]
                pub struct NotUnpinStruct<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[::pin_project::pin_project(!Unpin)]
                #[derive(Debug)]
                pub struct NotUnpinTupleStruct<T, U>(#[pin] pub T, pub U);

                #[::pin_project::pin_project(
                    !Unpin,
                    project = NotUnpinEnumProj,
                    project_ref = NotUnpinEnumProjRef,
                )]
                #[derive(Debug)]
                pub enum NotUnpinEnum<T, U> {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }
            };
        }

        mac!();
    }
}

pub mod box_pointers {
    use pin_project::pin_project;

    #[allow(box_pointers)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct {
        #[pin]
        pub p: Box<isize>,
        pub u: Box<isize>,
    }

    #[allow(box_pointers)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct(#[pin] pub Box<isize>, pub Box<isize>);

    #[allow(box_pointers)] // for the type itself
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum {
        Struct {
            #[pin]
            p: Box<isize>,
            u: Box<isize>,
        },
        Tuple(#[pin] Box<isize>, Box<isize>),
        Unit,
    }

    pub mod inside_macro {
        use pin_project::pin_project;

        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[allow(box_pointers)] // for the type itself
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct Struct {
                    #[pin]
                    pub p: Box<isize>,
                    pub u: Box<isize>,
                }

                #[allow(box_pointers)] // for the type itself
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct TupleStruct(#[pin] pub Box<isize>, pub Box<isize>);

                #[allow(box_pointers)] // for the type itself
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                #[derive(Debug)]
                pub enum Enum {
                    Struct {
                        #[pin]
                        p: Box<isize>,
                        u: Box<isize>,
                    },
                    Tuple(#[pin] Box<isize>, Box<isize>),
                    Unit,
                }
            };
        }

        mac!();
    }
}

pub mod deprecated {
    use pin_project::pin_project;

    #[allow(deprecated)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug, Clone, Copy)]
    #[deprecated]
    pub struct Struct {
        #[deprecated]
        #[pin]
        pub p: (),
        #[deprecated]
        pub u: (),
    }

    #[allow(deprecated)] // for the type itself
    #[pin_project(project_replace)]
    #[derive(Debug, Clone, Copy)]
    #[deprecated]
    pub struct TupleStruct(
        #[deprecated]
        #[pin]
        pub (),
        #[deprecated] pub (),
    );

    #[allow(deprecated)] // for the type itself
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug, Clone, Copy)]
    #[deprecated]
    pub enum Enum {
        #[deprecated]
        Struct {
            #[deprecated]
            #[pin]
            p: (),
            #[deprecated]
            u: (),
        },
        #[deprecated]
        Tuple(
            #[deprecated]
            #[pin]
            (),
            #[deprecated] (),
        ),
        #[deprecated]
        Unit,
    }

    pub mod inside_macro {
        use pin_project::pin_project;

        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[allow(deprecated)] // for the type itself
                #[pin_project(project_replace)]
                #[derive(Debug, Clone, Copy)]
                #[deprecated]
                pub struct Struct {
                    #[deprecated]
                    #[pin]
                    pub p: (),
                    #[deprecated]
                    pub u: (),
                }

                #[allow(deprecated)] // for the type itself
                #[pin_project(project_replace)]
                #[derive(Debug, Clone, Copy)]
                #[deprecated]
                pub struct TupleStruct(
                    #[deprecated]
                    #[pin]
                    pub (),
                    #[deprecated] pub (),
                );

                #[allow(deprecated)] // for the type itself
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                #[derive(Debug, Clone, Copy)]
                #[deprecated]
                pub enum Enum {
                    #[deprecated]
                    Struct {
                        #[deprecated]
                        #[pin]
                        p: (),
                        #[deprecated]
                        u: (),
                    },
                    #[deprecated]
                    Tuple(
                        #[deprecated]
                        #[pin]
                        (),
                        #[deprecated] (),
                    ),
                    #[deprecated]
                    Unit,
                }
            };
        }

        mac!();
    }
}

pub mod explicit_outlives_requirements {
    use pin_project::pin_project;

    #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
    #[pin_project(project_replace)]
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

    #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct<'a, T, U>(#[pin] pub &'a mut T, pub &'a mut U)
    where
        T: ?Sized,
        U: ?Sized;

    #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
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
        Tuple(#[pin] &'a mut T, &'a mut U),
        Unit,
    }

    pub mod inside_macro {
        use pin_project::pin_project;

        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
                #[pin_project(project_replace)]
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

                #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct TupleStruct<'a, T, U>(#[pin] pub &'a mut T, pub &'a mut U)
                where
                    T: ?Sized,
                    U: ?Sized;

                #[allow(explicit_outlives_requirements)] // for the type itself: https://github.com/rust-lang/rust/issues/60993
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
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
                    Tuple(#[pin] &'a mut T, &'a mut U),
                    Unit,
                }
            };
        }

        mac!();
    }
}

pub mod single_use_lifetimes {
    use pin_project::pin_project;

    #[allow(unused_lifetimes)]
    pub trait Trait<'a> {}

    #[allow(unused_lifetimes)] // for the type itself
    #[allow(single_use_lifetimes)] // for the type itself: https://github.com/rust-lang/rust/issues/55058
    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Hrtb<'pin___, T>
    where
        for<'pin> &'pin T: Unpin,
        T: for<'pin> Trait<'pin>,
        for<'pin, 'pin_, 'pin__> &'pin &'pin_ &'pin__ T: Unpin,
    {
        #[pin]
        _f: &'pin___ mut T,
    }

    pub mod inside_macro {
        use pin_project::pin_project;

        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[allow(unused_lifetimes)]
                pub trait Trait<'a> {}

                #[allow(unused_lifetimes)] // for the type itself
                #[allow(single_use_lifetimes)] // for the type itself: https://github.com/rust-lang/rust/issues/55058
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct Hrtb<'pin___, T>
                where
                    for<'pin> &'pin T: Unpin,
                    T: for<'pin> Trait<'pin>,
                    for<'pin, 'pin_, 'pin__> &'pin &'pin_ &'pin__ T: Unpin,
                {
                    #[pin]
                    _f: &'pin___ mut T,
                }
            };
        }

        mac!();
    }
}

pub mod variant_size_differences {
    use pin_project::pin_project;

    #[allow(missing_debug_implementations, missing_copy_implementations)] // https://github.com/rust-lang/rust/pull/74060
    #[allow(variant_size_differences)] // for the type itself
    #[allow(clippy::large_enum_variant)] // for the type itself
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    pub enum Enum {
        V1(u8),
        V2([u8; 1024]),
    }

    pub mod inside_macro {
        use pin_project::pin_project;

        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[allow(missing_debug_implementations, missing_copy_implementations)] // https://github.com/rust-lang/rust/pull/74060
                #[allow(variant_size_differences)] // for the type itself
                #[allow(clippy::large_enum_variant)] // for the type itself
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                pub enum Enum {
                    V1(u8),
                    V2([u8; 1024]),
                }
            };
        }

        mac!();
    }
}

pub mod clippy_mut_mut {
    use pin_project::pin_project;

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct<'a, T, U> {
        #[pin]
        pub pinned: &'a mut T,
        pub unpinned: &'a mut U,
    }

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct<'a, T, U>(#[pin] &'a mut T, &'a mut U);

    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum<'a, T, U> {
        Struct {
            #[pin]
            pinned: &'a mut T,
            unpinned: &'a mut U,
        },
        Tuple(#[pin] &'a mut T, &'a mut U),
        Unit,
    }

    pub mod inside_macro {
        use pin_project::pin_project;

        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct Struct<'a, T, U> {
                    #[pin]
                    pub pinned: &'a mut T,
                    pub unpinned: &'a mut U,
                }

                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct TupleStruct<'a, T, U>(#[pin] &'a mut T, &'a mut U);

                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                #[derive(Debug)]
                pub enum Enum<'a, T, U> {
                    Struct {
                        #[pin]
                        pinned: &'a mut T,
                        unpinned: &'a mut U,
                    },
                    Tuple(#[pin] &'a mut T, &'a mut U),
                    Unit,
                }
            };
        }

        mac!();
    }
}

#[allow(unreachable_pub)]
mod clippy_redundant_pub_crate {
    use pin_project::pin_project;

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct<T, U> {
        #[pin]
        pub pinned: T,
        pub unpinned: U,
    }

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct<T, U>(#[pin] pub T, pub U);

    #[allow(dead_code)]
    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum<T, U> {
        Struct {
            #[pin]
            pinned: T,
            unpinned: U,
        },
        Tuple(#[pin] T, U),
        Unit,
    }

    pub mod inside_macro {
        use pin_project::pin_project;

        #[allow(clippy::redundant_pub_crate)]
        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct Struct<T, U> {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct TupleStruct<T, U>(#[pin] pub T, pub U);

                #[allow(dead_code)]
                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                #[derive(Debug)]
                pub enum Enum<T, U> {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }
            };
        }

        mac!();
    }
}

pub mod clippy_type_repetition_in_bounds {
    use pin_project::pin_project;

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct<T, U>
    where
        Self: Sized,
    {
        #[pin]
        pub pinned: T,
        pub unpinned: U,
    }

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct TupleStruct<T, U>(#[pin] T, U)
    where
        Self: Sized;

    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum<T, U>
    where
        Self: Sized,
    {
        Struct {
            #[pin]
            pinned: T,
            unpinned: U,
        },
        Tuple(#[pin] T, U),
        Unit,
    }

    pub mod inside_macro {
        use pin_project::pin_project;

        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct Struct<T, U>
                where
                    Self: Sized,
                {
                    #[pin]
                    pub pinned: T,
                    pub unpinned: U,
                }

                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct TupleStruct<T, U>(#[pin] T, U)
                where
                    Self: Sized;

                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                #[derive(Debug)]
                pub enum Enum<T, U>
                where
                    Self: Sized,
                {
                    Struct {
                        #[pin]
                        pinned: T,
                        unpinned: U,
                    },
                    Tuple(#[pin] T, U),
                    Unit,
                }
            };
        }

        mac!();
    }
}

pub mod clippy_use_self {
    use pin_project::pin_project;

    pub trait Trait {
        type Assoc;
    }

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Generics<T: Trait<Assoc = Self>>
    where
        Self: Trait<Assoc = Self>,
    {
        _f: T,
    }

    pub mod inside_macro {
        use pin_project::pin_project;

        use super::Trait;

        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct Generics<T: Trait<Assoc = Self>>
                where
                    Self: Trait<Assoc = Self>,
                {
                    _f: T,
                }
            };
        }

        mac!();
    }
}

pub mod clippy_used_underscore_binding {
    use pin_project::pin_project;

    #[pin_project(project_replace)]
    #[derive(Debug)]
    pub struct Struct<T, U> {
        #[pin]
        pub _pinned: T,
        pub _unpinned: U,
    }

    #[pin_project(
        project = EnumProj,
        project_ref = EnumProjRef,
        project_replace = EnumProjOwn,
    )]
    #[derive(Debug)]
    pub enum Enum<T, U> {
        Struct {
            #[pin]
            _pinned: T,
            _unpinned: U,
        },
    }

    pub mod inside_macro {
        use pin_project::pin_project;

        #[rustfmt::skip]
        macro_rules! mac {
            () => {
                #[pin_project(project_replace)]
                #[derive(Debug)]
                pub struct Struct<T, U> {
                    #[pin]
                    pub _pinned: T,
                    pub _unpinned: U,
                }

                #[pin_project(
                    project = EnumProj,
                    project_ref = EnumProjRef,
                    project_replace = EnumProjOwn,
                )]
                #[derive(Debug)]
                pub enum Enum<T, U> {
                    Struct {
                        #[pin]
                        _pinned: T,
                        _unpinned: U,
                    },
                }
            };
        }

        mac!();
    }
}

pub mod clippy_ref_option_ref {
    use pin_project::pin_project;

    #[pin_project]
    #[derive(Debug)]
    pub struct Struct<'a> {
        #[pin]
        pub _pinned: Option<&'a ()>,
        pub _unpinned: Option<&'a ()>,
    }

    #[pin_project(project = EnumProj, project_ref = EnumProjRef)]
    #[derive(Debug)]
    pub enum Enum<'a> {
        Struct {
            #[pin]
            _pinned: Option<&'a ()>,
            _unpinned: Option<&'a ()>,
        },
    }
}
