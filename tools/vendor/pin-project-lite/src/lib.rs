//! A lightweight version of [pin-project] written with declarative macros.
//!
//! # Examples
//!
//! [`pin_project!`] macro creates a projection type covering all the fields of struct.
//!
//! ```rust
//! use std::pin::Pin;
//!
//! use pin_project_lite::pin_project;
//!
//! pin_project! {
//!     struct Struct<T, U> {
//!         #[pin]
//!         pinned: T,
//!         unpinned: U,
//!     }
//! }
//!
//! impl<T, U> Struct<T, U> {
//!     fn method(self: Pin<&mut Self>) {
//!         let this = self.project();
//!         let _: Pin<&mut T> = this.pinned; // Pinned reference to the field
//!         let _: &mut U = this.unpinned; // Normal reference to the field
//!     }
//! }
//! ```
//!
//! To use [`pin_project!`] on enums, you need to name the projection type
//! returned from the method.
//!
//! ```rust
//! use std::pin::Pin;
//!
//! use pin_project_lite::pin_project;
//!
//! pin_project! {
//!     #[project = EnumProj]
//!     enum Enum<T, U> {
//!         Variant { #[pin] pinned: T, unpinned: U },
//!     }
//! }
//!
//! impl<T, U> Enum<T, U> {
//!     fn method(self: Pin<&mut Self>) {
//!         match self.project() {
//!             EnumProj::Variant { pinned, unpinned } => {
//!                 let _: Pin<&mut T> = pinned;
//!                 let _: &mut U = unpinned;
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! # [pin-project] vs pin-project-lite
//!
//! Here are some similarities and differences compared to [pin-project].
//!
//! ## Similar: Safety
//!
//! pin-project-lite guarantees safety in much the same way as [pin-project].
//! Both are completely safe unless you write other unsafe code.
//!
//! ## Different: Minimal design
//!
//! This library does not tackle as expansive of a range of use cases as
//! [pin-project] does. If your use case is not already covered, please use
//! [pin-project].
//!
//! ## Different: No proc-macro related dependencies
//!
//! This is the **only** reason to use this crate. However, **if you already
//! have proc-macro related dependencies in your crate's dependency graph, there
//! is no benefit from using this crate.** (Note: There is almost no difference
//! in the amount of code generated between [pin-project] and pin-project-lite.)
//!
//! ## Different: No useful error messages
//!
//! This macro does not handle any invalid input. So error messages are not to
//! be useful in most cases. If you do need useful error messages, then upon
//! error you can pass the same input to [pin-project] to receive a helpful
//! description of the compile error.
//!
//! ## Different: No support for custom Unpin implementation
//!
//! pin-project supports this by [`UnsafeUnpin`][unsafe-unpin] and [`!Unpin`][not-unpin].
//!
//! ## Different: No support for tuple structs and tuple variants
//!
//! pin-project supports this.
//!
//! [not-unpin]: https://docs.rs/pin-project/1/pin_project/attr.pin_project.html#unpin
//! [pin-project]: https://github.com/taiki-e/pin-project
//! [unsafe-unpin]: https://docs.rs/pin-project/1/pin_project/attr.pin_project.html#unsafeunpin

#![no_std]
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms, single_use_lifetimes),
        allow(dead_code, unused_variables)
    )
))]
#![warn(rust_2018_idioms, single_use_lifetimes, unreachable_pub)]
#![warn(clippy::default_trait_access, clippy::wildcard_imports)]

/// A macro that creates a projection type covering all the fields of struct.
///
/// This macro creates a projection type according to the following rules:
///
/// - For the field that uses `#[pin]` attribute, makes the pinned reference to the field.
/// - For the other fields, makes the unpinned reference to the field.
///
/// And the following methods are implemented on the original type:
///
/// ```rust
/// # use std::pin::Pin;
/// # type Projection<'a> = &'a ();
/// # type ProjectionRef<'a> = &'a ();
/// # trait Dox {
/// fn project(self: Pin<&mut Self>) -> Projection<'_>;
/// fn project_ref(self: Pin<&Self>) -> ProjectionRef<'_>;
/// # }
/// ```
///
/// By passing an attribute with the same name as the method to the macro,
/// you can name the projection type returned from the method. This allows you
/// to use pattern matching on the projected types.
///
/// ```rust
/// # use pin_project_lite::pin_project;
/// # use std::pin::Pin;
/// pin_project! {
///     #[project = EnumProj]
///     enum Enum<T> {
///         Variant { #[pin] field: T },
///     }
/// }
///
/// impl<T> Enum<T> {
///     fn method(self: Pin<&mut Self>) {
///         let this: EnumProj<'_, T> = self.project();
///         match this {
///             EnumProj::Variant { field } => {
///                 let _: Pin<&mut T> = field;
///             }
///         }
///     }
/// }
/// ```
///
/// By passing the `#[project_replace = MyProjReplace]` attribute you may create an additional
/// method which allows the contents of `Pin<&mut Self>` to be replaced while simultaneously moving
/// out all unpinned fields in `Self`.
///
/// ```rust
/// # use std::pin::Pin;
/// # type MyProjReplace = ();
/// # trait Dox {
/// fn project_replace(self: Pin<&mut Self>, replacement: Self) -> MyProjReplace;
/// # }
/// ```
///
/// Also, note that the projection types returned by `project` and `project_ref` have
/// an additional lifetime at the beginning of generics.
///
/// ```text
/// let this: EnumProj<'_, T> = self.project();
///                    ^^
/// ```
///
/// The visibility of the projected types and projection methods is based on the
/// original type. However, if the visibility of the original type is `pub`, the
/// visibility of the projected types and the projection methods is downgraded
/// to `pub(crate)`.
///
/// # Safety
///
/// `pin_project!` macro guarantees safety in much the same way as [pin-project] crate.
/// Both are completely safe unless you write other unsafe code.
///
/// See [pin-project] crate for more details.
///
/// # Examples
///
/// ```rust
/// use std::pin::Pin;
///
/// use pin_project_lite::pin_project;
///
/// pin_project! {
///     struct Struct<T, U> {
///         #[pin]
///         pinned: T,
///         unpinned: U,
///     }
/// }
///
/// impl<T, U> Struct<T, U> {
///     fn method(self: Pin<&mut Self>) {
///         let this = self.project();
///         let _: Pin<&mut T> = this.pinned; // Pinned reference to the field
///         let _: &mut U = this.unpinned; // Normal reference to the field
///     }
/// }
/// ```
///
/// To use `pin_project!` on enums, you need to name the projection type
/// returned from the method.
///
/// ```rust
/// use std::pin::Pin;
///
/// use pin_project_lite::pin_project;
///
/// pin_project! {
///     #[project = EnumProj]
///     enum Enum<T> {
///         Struct {
///             #[pin]
///             field: T,
///         },
///         Unit,
///     }
/// }
///
/// impl<T> Enum<T> {
///     fn method(self: Pin<&mut Self>) {
///         match self.project() {
///             EnumProj::Struct { field } => {
///                 let _: Pin<&mut T> = field;
///             }
///             EnumProj::Unit => {}
///         }
///     }
/// }
/// ```
///
/// If you want to call the `project()` method multiple times or later use the
/// original [`Pin`] type, it needs to use [`.as_mut()`][`Pin::as_mut`] to avoid
/// consuming the [`Pin`].
///
/// ```rust
/// use std::pin::Pin;
///
/// use pin_project_lite::pin_project;
///
/// pin_project! {
///     struct Struct<T> {
///         #[pin]
///         field: T,
///     }
/// }
///
/// impl<T> Struct<T> {
///     fn call_project_twice(mut self: Pin<&mut Self>) {
///         // `project` consumes `self`, so reborrow the `Pin<&mut Self>` via `as_mut`.
///         self.as_mut().project();
///         self.as_mut().project();
///     }
/// }
/// ```
///
/// # `!Unpin`
///
/// If you want to ensure that [`Unpin`] is not implemented, use `#[pin]`
/// attribute for a [`PhantomPinned`] field.
///
/// ```rust
/// use std::marker::PhantomPinned;
///
/// use pin_project_lite::pin_project;
///
/// pin_project! {
///     struct Struct<T> {
///         field: T,
///         #[pin] // <------ This `#[pin]` is required to make `Struct` to `!Unpin`.
///         _pin: PhantomPinned,
///     }
/// }
/// ```
///
/// Note that using [`PhantomPinned`] without `#[pin]` attribute has no effect.
///
/// [`PhantomPinned`]: core::marker::PhantomPinned
/// [`Pin::as_mut`]: core::pin::Pin::as_mut
/// [`Pin`]: core::pin::Pin
/// [pin-project]: https://github.com/taiki-e/pin-project
#[macro_export]
macro_rules! pin_project {
    ($($tt:tt)*) => {
        $crate::__pin_project_internal! {
            [][][][]
            $($tt)*
        }
    };
}

// limitations:
// - no support for tuple structs and tuple variant (wontfix).
// - no support for multiple trait/lifetime bounds.
// - no support for `Self` in where clauses. (wontfix)
// - no support for overlapping lifetime names. (wontfix)
// - no interoperability with other field attributes.
// - no useful error messages. (wontfix)
// etc...

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_expand {
    (
        [$($proj_mut_ident:ident)?]
        [$($proj_ref_ident:ident)?]
        [$($proj_replace_ident:ident)?]
        [$proj_vis:vis]
        [$(#[$attrs:meta])* $vis:vis $struct_ty_ident:ident $ident:ident]
        [$($def_generics:tt)*]
        [$($impl_generics:tt)*] [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        {
            $($body_data:tt)*
        }
        $(impl $($pinned_drop:tt)*)?
    ) => {
        $crate::__pin_project_reconstruct! {
            [$(#[$attrs])* $vis $struct_ty_ident $ident]
            [$($def_generics)*] [$($impl_generics)*]
            [$($ty_generics)*] [$(where $($where_clause)*)?]
            {
                $($body_data)*
            }
        }

        $crate::__pin_project_make_proj_ty! {
            [$($proj_mut_ident)?]
            [$proj_vis $struct_ty_ident $ident]
            [__pin_project_make_proj_field_mut]
            [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
            {
                $($body_data)*
            }
        }
        $crate::__pin_project_make_proj_ty! {
            [$($proj_ref_ident)?]
            [$proj_vis $struct_ty_ident $ident]
            [__pin_project_make_proj_field_ref]
            [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
            {
                $($body_data)*
            }
        }
        $crate::__pin_project_make_proj_replace_ty! {
            [$($proj_replace_ident)?]
            [$proj_vis $struct_ty_ident]
            [__pin_project_make_proj_field_replace]
            [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
            {
                $($body_data)*
            }
        }

        $crate::__pin_project_constant! {
            [$(#[$attrs])* $vis $struct_ty_ident $ident]
            [$($proj_mut_ident)?] [$($proj_ref_ident)?] [$($proj_replace_ident)?]
            [$proj_vis]
            [$($def_generics)*] [$($impl_generics)*]
            [$($ty_generics)*] [$(where $($where_clause)*)?]
            {
                $($body_data)*
            }
            $(impl $($pinned_drop)*)?
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_constant {
    (
        [$(#[$attrs:meta])* $vis:vis struct $ident:ident]
        [$($proj_mut_ident:ident)?] [$($proj_ref_ident:ident)?] [$($proj_replace_ident:ident)?]
        [$proj_vis:vis]
        [$($def_generics:tt)*]
        [$($impl_generics:tt)*] [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident: $field_ty:ty
            ),+ $(,)?
        }
        $(impl $($pinned_drop:tt)*)?
    ) => {
        #[allow(explicit_outlives_requirements)] // https://github.com/rust-lang/rust/issues/60993
        #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
        // This lint warns of `clippy::*` generated by external macros.
        // We allow this lint for compatibility with older compilers.
        #[allow(clippy::unknown_clippy_lints)]
        #[allow(clippy::redundant_pub_crate)] // This lint warns `pub(crate)` field in private struct.
        #[allow(clippy::used_underscore_binding)]
        const _: () = {
            $crate::__pin_project_make_proj_ty! {
                [$($proj_mut_ident)? Projection]
                [$proj_vis struct $ident]
                [__pin_project_make_proj_field_mut]
                [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
                {
                    $(
                        $(#[$pin])?
                        $field_vis $field: $field_ty
                    ),+
                }
            }
            $crate::__pin_project_make_proj_ty! {
                [$($proj_ref_ident)? ProjectionRef]
                [$proj_vis struct $ident]
                [__pin_project_make_proj_field_ref]
                [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
                {
                    $(
                        $(#[$pin])?
                        $field_vis $field: $field_ty
                    ),+
                }
            }

            impl <$($impl_generics)*> $ident <$($ty_generics)*>
            $(where
                $($where_clause)*)?
            {
                $crate::__pin_project_struct_make_proj_method! {
                    [$($proj_mut_ident)? Projection]
                    [$proj_vis]
                    [project get_unchecked_mut mut]
                    [$($ty_generics)*]
                    {
                        $(
                            $(#[$pin])?
                            $field_vis $field
                        ),+
                    }
                }
                $crate::__pin_project_struct_make_proj_method! {
                    [$($proj_ref_ident)? ProjectionRef]
                    [$proj_vis]
                    [project_ref get_ref]
                    [$($ty_generics)*]
                    {
                        $(
                            $(#[$pin])?
                            $field_vis $field
                        ),+
                    }
                }
                $crate::__pin_project_struct_make_proj_replace_method! {
                    [$($proj_replace_ident)?]
                    [$proj_vis]
                    [ProjectionReplace]
                    [$($ty_generics)*]
                    {
                        $(
                            $(#[$pin])?
                            $field_vis $field
                        ),+
                    }
                }
            }

            $crate::__pin_project_make_unpin_impl! {
                [$vis $ident]
                [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
                $(
                    $field: $crate::__pin_project_make_unpin_bound!(
                        $(#[$pin])? $field_ty
                    )
                ),+
            }

            $crate::__pin_project_make_drop_impl! {
                [$ident]
                [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
                $(impl $($pinned_drop)*)?
            }

            // Ensure that it's impossible to use pin projections on a #[repr(packed)] struct.
            //
            // Taking a reference to a packed field is UB, and applying
            // `#[forbid(unaligned_references)]` makes sure that doing this is a hard error.
            //
            // If the struct ends up having #[repr(packed)] applied somehow,
            // this will generate an (unfriendly) error message. Under all reasonable
            // circumstances, we'll detect the #[repr(packed)] attribute, and generate
            // a much nicer error above.
            //
            // See https://github.com/taiki-e/pin-project/pull/34 for more details.
            //
            // Note:
            // - Lint-based tricks aren't perfect, but they're much better than nothing:
            //   https://github.com/taiki-e/pin-project-lite/issues/26
            //
            // - Enable both unaligned_references and safe_packed_borrows lints
            //   because unaligned_references lint does not exist in older compilers:
            //   https://github.com/taiki-e/pin-project-lite/pull/55
            //   https://github.com/rust-lang/rust/pull/82525
            #[forbid(unaligned_references, safe_packed_borrows)]
            fn __assert_not_repr_packed <$($impl_generics)*> (this: &$ident <$($ty_generics)*>)
            $(where
                $($where_clause)*)?
            {
                $(
                    let _ = &this.$field;
                )+
            }
        };
    };
    (
        [$(#[$attrs:meta])* $vis:vis enum $ident:ident]
        [$($proj_mut_ident:ident)?] [$($proj_ref_ident:ident)?] [$($proj_replace_ident:ident)?]
        [$proj_vis:vis]
        [$($def_generics:tt)*]
        [$($impl_generics:tt)*] [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        {
            $(
                $(#[$variant_attrs:meta])*
                $variant:ident $({
                    $(
                        $(#[$pin:ident])?
                        $field:ident: $field_ty:ty
                    ),+ $(,)?
                })?
            ),+ $(,)?
        }
        $(impl $($pinned_drop:tt)*)?
    ) => {
        #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
        // This lint warns of `clippy::*` generated by external macros.
        // We allow this lint for compatibility with older compilers.
        #[allow(clippy::unknown_clippy_lints)]
        #[allow(clippy::used_underscore_binding)]
        const _: () = {
            impl <$($impl_generics)*> $ident <$($ty_generics)*>
            $(where
                $($where_clause)*)?
            {
                $crate::__pin_project_enum_make_proj_method! {
                    [$($proj_mut_ident)?]
                    [$proj_vis]
                    [project get_unchecked_mut mut]
                    [$($ty_generics)*]
                    {
                        $(
                            $variant $({
                                $(
                                    $(#[$pin])?
                                    $field
                                ),+
                            })?
                        ),+
                    }
                }
                $crate::__pin_project_enum_make_proj_method! {
                    [$($proj_ref_ident)?]
                    [$proj_vis]
                    [project_ref get_ref]
                    [$($ty_generics)*]
                    {
                        $(
                            $variant $({
                                $(
                                    $(#[$pin])?
                                    $field
                                ),+
                            })?
                        ),+
                    }
                }
                $crate::__pin_project_enum_make_proj_replace_method! {
                    [$($proj_replace_ident)?]
                    [$proj_vis]
                    [$($ty_generics)*]
                    {
                        $(
                            $variant $({
                                $(
                                    $(#[$pin])?
                                    $field
                                ),+
                            })?
                        ),+
                    }
                }
            }

            $crate::__pin_project_make_unpin_impl! {
                [$vis $ident]
                [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
                $(
                    $variant: ($(
                        $(
                            $crate::__pin_project_make_unpin_bound!(
                                $(#[$pin])? $field_ty
                            )
                        ),+
                    )?)
                ),+
            }

            $crate::__pin_project_make_drop_impl! {
                [$ident]
                [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
                $(impl $($pinned_drop)*)?
            }

            // We don't need to check for '#[repr(packed)]',
            // since it does not apply to enums.
        };
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_reconstruct {
    (
        [$(#[$attrs:meta])* $vis:vis struct $ident:ident]
        [$($def_generics:tt)*] [$($impl_generics:tt)*]
        [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident: $field_ty:ty
            ),+ $(,)?
        }
    ) => {
        $(#[$attrs])*
        $vis struct $ident $($def_generics)*
        $(where
            $($where_clause)*)?
        {
            $(
                $field_vis $field: $field_ty
            ),+
        }
    };
    (
        [$(#[$attrs:meta])* $vis:vis enum $ident:ident]
        [$($def_generics:tt)*] [$($impl_generics:tt)*]
        [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        {
            $(
                $(#[$variant_attrs:meta])*
                $variant:ident $({
                    $(
                        $(#[$pin:ident])?
                        $field:ident: $field_ty:ty
                    ),+ $(,)?
                })?
            ),+ $(,)?
        }
    ) => {
        $(#[$attrs])*
        $vis enum $ident $($def_generics)*
        $(where
            $($where_clause)*)?
        {
            $(
                $(#[$variant_attrs])*
                $variant $({
                    $(
                        $field: $field_ty
                    ),+
                })?
            ),+
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_proj_ty {
    ([] $($field:tt)*) => {};
    (
        [$proj_ty_ident:ident $default_ident:ident]
        [$proj_vis:vis struct $ident:ident]
        $($field:tt)*
    ) => {};
    (
        [$proj_ty_ident:ident]
        [$proj_vis:vis struct $ident:ident]
        [$__pin_project_make_proj_field:ident]
        [$($impl_generics:tt)*] [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident: $field_ty:ty
            ),+ $(,)?
        }
    ) => {
        $crate::__pin_project_make_proj_ty_body! {
            [$proj_ty_ident]
            [$proj_vis struct $ident]
            [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
            [
                $(
                    $field_vis $field: $crate::$__pin_project_make_proj_field!(
                        $(#[$pin])? $field_ty
                    )
                ),+
            ]
        }
    };
    (
        [$proj_ty_ident:ident]
        [$proj_vis:vis enum $ident:ident]
        [$__pin_project_make_proj_field:ident]
        [$($impl_generics:tt)*] [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        {
            $(
                $(#[$variant_attrs:meta])*
                $variant:ident $({
                    $(
                        $(#[$pin:ident])?
                        $field:ident: $field_ty:ty
                    ),+ $(,)?
                })?
            ),+ $(,)?
        }
    ) => {
        $crate::__pin_project_make_proj_ty_body! {
            [$proj_ty_ident]
            [$proj_vis enum $ident]
            [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
            [
                $(
                    $variant $({
                        $(
                            $field: $crate::$__pin_project_make_proj_field!(
                                $(#[$pin])? $field_ty
                            )
                        ),+
                    })?
                ),+
            ]
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_proj_ty_body {
    (
        [$proj_ty_ident:ident]
        [$proj_vis:vis $struct_ty_ident:ident $ident:ident]
        [$($impl_generics:tt)*] [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        [$($body_data:tt)+]
    ) => {
        #[allow(dead_code)] // This lint warns unused fields/variants.
        #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
        // This lint warns of `clippy::*` generated by external macros.
        // We allow this lint for compatibility with older compilers.
        #[allow(clippy::unknown_clippy_lints)]
        #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`. (only needed for project)
        #[allow(clippy::redundant_pub_crate)] // This lint warns `pub(crate)` field in private struct.
        #[allow(clippy::ref_option_ref)] // This lint warns `&Option<&<ty>>`. (only needed for project_ref)
        #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326
        $proj_vis $struct_ty_ident $proj_ty_ident <'__pin, $($impl_generics)*>
        where
            $ident <$($ty_generics)*>: '__pin
            $(, $($where_clause)*)?
        {
            $($body_data)+
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_proj_replace_ty {
    ([] $($field:tt)*) => {};
    (
        [$proj_ty_ident:ident]
        [$proj_vis:vis struct]
        [$__pin_project_make_proj_field:ident]
        [$($impl_generics:tt)*] [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident: $field_ty:ty
            ),+ $(,)?
        }
    ) => {
        $crate::__pin_project_make_proj_replace_ty_body! {
            [$proj_ty_ident]
            [$proj_vis struct]
            [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
            [
                $(
                    $field_vis $field: $crate::$__pin_project_make_proj_field!(
                        $(#[$pin])? $field_ty
                    )
                ),+
            ]
        }
    };
    (
        [$proj_ty_ident:ident]
        [$proj_vis:vis enum]
        [$__pin_project_make_proj_field:ident]
        [$($impl_generics:tt)*] [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        {
            $(
                $(#[$variant_attrs:meta])*
                $variant:ident $({
                    $(
                        $(#[$pin:ident])?
                        $field:ident: $field_ty:ty
                    ),+ $(,)?
                })?
            ),+ $(,)?
        }
    ) => {
        $crate::__pin_project_make_proj_replace_ty_body! {
            [$proj_ty_ident]
            [$proj_vis enum]
            [$($impl_generics)*] [$($ty_generics)*] [$(where $($where_clause)*)?]
            [
                $(
                    $variant $({
                        $(
                            $field: $crate::$__pin_project_make_proj_field!(
                                $(#[$pin])? $field_ty
                            )
                        ),+
                    })?
                ),+
            ]
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_proj_replace_ty_body {
    (
        [$proj_ty_ident:ident]
        [$proj_vis:vis $struct_ty_ident:ident]
        [$($impl_generics:tt)*] [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        [$($body_data:tt)+]
    ) => {
        #[allow(dead_code)] // This lint warns unused fields/variants.
        #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
        #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`. (only needed for project)
        #[allow(clippy::redundant_pub_crate)] // This lint warns `pub(crate)` field in private struct.
        #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326
        $proj_vis $struct_ty_ident $proj_ty_ident <$($impl_generics)*>
        where
            $($($where_clause)*)?
        {
            $($body_data)+
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_proj_replace_block {
    (
        [$($proj_path:tt)+]
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident
            ),+
        }
    ) => {
        let result = $($proj_path)* {
            $(
                $field: $crate::__pin_project_make_replace_field_proj!(
                    $(#[$pin])? $field
                )
            ),+
        };

        {
            ( $(
                $crate::__pin_project_make_unsafe_drop_in_place_guard!(
                    $(#[$pin])? $field
                ),
            )* );
        }

        result
    };
    ([$($proj_path:tt)+]) => { $($proj_path)* };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_struct_make_proj_method {
    ([] $($variant:tt)*) => {};
    (
        [$proj_ty_ident:ident $_ignored_default_arg:ident]
        [$proj_vis:vis]
        [$method_ident:ident $get_method:ident $($mut:ident)?]
        [$($ty_generics:tt)*]
        $($variant:tt)*
    ) => {
        $crate::__pin_project_struct_make_proj_method! {
            [$proj_ty_ident]
            [$proj_vis]
            [$method_ident $get_method $($mut)?]
            [$($ty_generics)*]
            $($variant)*
        }
    };
    (
        [$proj_ty_ident:ident]
        [$proj_vis:vis]
        [$method_ident:ident $get_method:ident $($mut:ident)?]
        [$($ty_generics:tt)*]
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident
            ),+
        }
    ) => {
        $proj_vis fn $method_ident<'__pin>(
            self: $crate::__private::Pin<&'__pin $($mut)? Self>,
        ) -> $proj_ty_ident <'__pin, $($ty_generics)*> {
            unsafe {
                let Self { $($field),* } = self.$get_method();
                $proj_ty_ident {
                    $(
                        $field: $crate::__pin_project_make_unsafe_field_proj!(
                            $(#[$pin])? $field
                        )
                    ),+
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_struct_make_proj_replace_method {
    ([] $($field:tt)*) => {};
    (
        [$proj_ty_ident:ident]
        [$proj_vis:vis]
        [$_proj_ty_ident:ident]
        [$($ty_generics:tt)*]
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident
            ),+
        }
    ) => {
        $proj_vis fn project_replace(
            self: $crate::__private::Pin<&mut Self>,
            replacement: Self,
        ) -> $proj_ty_ident <$($ty_generics)*> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();

                // Destructors will run in reverse order, so next create a guard to overwrite
                // `self` with the replacement value without calling destructors.
                let __guard = $crate::__private::UnsafeOverwriteGuard::new(__self_ptr, replacement);

                let Self { $($field),* } = &mut *__self_ptr;

                $crate::__pin_project_make_proj_replace_block! {
                    [$proj_ty_ident]
                    {
                        $(
                            $(#[$pin])?
                            $field
                        ),+
                    }
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_enum_make_proj_method {
    ([] $($variant:tt)*) => {};
    (
        [$proj_ty_ident:ident]
        [$proj_vis:vis]
        [$method_ident:ident $get_method:ident $($mut:ident)?]
        [$($ty_generics:tt)*]
        {
            $(
                $variant:ident $({
                    $(
                        $(#[$pin:ident])?
                        $field:ident
                    ),+
                })?
            ),+
        }
    ) => {
        $proj_vis fn $method_ident<'__pin>(
            self: $crate::__private::Pin<&'__pin $($mut)? Self>,
        ) -> $proj_ty_ident <'__pin, $($ty_generics)*> {
            unsafe {
                match self.$get_method() {
                    $(
                        Self::$variant $({
                            $($field),+
                        })? => {
                            $proj_ty_ident::$variant $({
                                $(
                                    $field: $crate::__pin_project_make_unsafe_field_proj!(
                                        $(#[$pin])? $field
                                    )
                                ),+
                            })?
                        }
                    ),+
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_enum_make_proj_replace_method {
    ([] $($field:tt)*) => {};
    (
        [$proj_ty_ident:ident]
        [$proj_vis:vis]
        [$($ty_generics:tt)*]
        {
            $(
                $variant:ident $({
                    $(
                        $(#[$pin:ident])?
                        $field:ident
                    ),+
                })?
            ),+
        }
    ) => {
        $proj_vis fn project_replace(
            self: $crate::__private::Pin<&mut Self>,
            replacement: Self,
        ) -> $proj_ty_ident <$($ty_generics)*> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();

                // Destructors will run in reverse order, so next create a guard to overwrite
                // `self` with the replacement value without calling destructors.
                let __guard = $crate::__private::UnsafeOverwriteGuard::new(__self_ptr, replacement);

                match &mut *__self_ptr {
                    $(
                        Self::$variant $({
                            $($field),+
                        })? => {
                            $crate::__pin_project_make_proj_replace_block! {
                                [$proj_ty_ident :: $variant]
                                $({
                                    $(
                                        $(#[$pin])?
                                        $field
                                    ),+
                                })?
                            }
                        }
                    ),+
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_unpin_impl {
    (
        [$vis:vis $ident:ident]
        [$($impl_generics:tt)*] [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
        $($field:tt)*
    ) => {
        // Automatically create the appropriate conditional `Unpin` implementation.
        //
        // Basically this is equivalent to the following code:
        // ```rust
        // impl<T, U> Unpin for Struct<T, U> where T: Unpin {}
        // ```
        //
        // However, if struct is public and there is a private type field,
        // this would cause an E0446 (private type in public interface).
        //
        // When RFC 2145 is implemented (rust-lang/rust#48054),
        // this will become a lint, rather then a hard error.
        //
        // As a workaround for this, we generate a new struct, containing all of the pinned
        // fields from our #[pin_project] type. This struct is declared within
        // a function, which makes it impossible to be named by user code.
        // This guarantees that it will use the default auto-trait impl for Unpin -
        // that is, it will implement Unpin iff all of its fields implement Unpin.
        // This type can be safely declared as 'public', satisfying the privacy
        // checker without actually allowing user code to access it.
        //
        // This allows users to apply the #[pin_project] attribute to types
        // regardless of the privacy of the types of their fields.
        //
        // See also https://github.com/taiki-e/pin-project/pull/53.
        #[allow(non_snake_case)]
        $vis struct __Origin <'__pin, $($impl_generics)*>
        $(where
            $($where_clause)*)?
        {
            __dummy_lifetime: $crate::__private::PhantomData<&'__pin ()>,
            $($field)*
        }
        impl <'__pin, $($impl_generics)*> $crate::__private::Unpin for $ident <$($ty_generics)*>
        where
            __Origin <'__pin, $($ty_generics)*>: $crate::__private::Unpin
            $(, $($where_clause)*)?
        {
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_drop_impl {
    (
        [$_ident:ident]
        [$($_impl_generics:tt)*] [$($_ty_generics:tt)*] [$(where $($_where_clause:tt)*)?]
        impl $(<
            $( $lifetime:lifetime $(: $lifetime_bound:lifetime)? ),* $(,)?
            $( $generics:ident
                $(: $generics_bound:path)?
                $(: ?$generics_unsized_bound:path)?
                $(: $generics_lifetime_bound:lifetime)?
            ),*
        >)? PinnedDrop for $self_ty:ty
        $(where
            $( $where_clause_ty:ty
                $(: $where_clause_bound:path)?
                $(: ?$where_clause_unsized_bound:path)?
                $(: $where_clause_lifetime_bound:lifetime)?
            ),* $(,)?
        )?
        {
            fn drop($($arg:ident)+: Pin<&mut Self>) {
                $($tt:tt)*
            }
        }
    ) => {
        impl $(<
            $( $lifetime $(: $lifetime_bound)? ,)*
            $( $generics
                $(: $generics_bound)?
                $(: ?$generics_unsized_bound)?
                $(: $generics_lifetime_bound)?
            ),*
        >)? $crate::__private::Drop for $self_ty
        $(where
            $( $where_clause_ty
                $(: $where_clause_bound)?
                $(: ?$where_clause_unsized_bound)?
                $(: $where_clause_lifetime_bound)?
            ),*
        )?
        {
            fn drop(&mut self) {
                // Implementing `__DropInner::__drop_inner` is safe, but calling it is not safe.
                // This is because destructors can be called multiple times in safe code and
                // [double dropping is unsound](https://github.com/rust-lang/rust/pull/62360).
                //
                // `__drop_inner` is defined as a safe method, but this is fine since
                // `__drop_inner` is not accessible by the users and we call `__drop_inner` only
                // once.
                //
                // Users can implement [`Drop`] safely using `pin_project!` and can drop a
                // type that implements `PinnedDrop` using the [`drop`] function safely.
                fn __drop_inner $(<
                    $( $lifetime $(: $lifetime_bound)? ,)*
                    $( $generics
                        $(: $generics_bound)?
                        $(: ?$generics_unsized_bound)?
                        $(: $generics_lifetime_bound)?
                    ),*
                >)? (
                    $($arg)+: $crate::__private::Pin<&mut $self_ty>,
                )
                $(where
                    $( $where_clause_ty
                        $(: $where_clause_bound)?
                        $(: ?$where_clause_unsized_bound)?
                        $(: $where_clause_lifetime_bound)?
                    ),*
                )?
                {
                    // A dummy `__drop_inner` function to prevent users call outer `__drop_inner`.
                    fn __drop_inner() {}
                    $($tt)*
                }

                // Safety - we're in 'drop', so we know that 'self' will
                // never move again.
                let pinned_self: $crate::__private::Pin<&mut Self>
                    = unsafe { $crate::__private::Pin::new_unchecked(self) };
                // We call `__drop_inner` only once. Since `__DropInner::__drop_inner`
                // is not accessible by the users, it is never called again.
                __drop_inner(pinned_self);
            }
        }
    };
    (
        [$ident:ident]
        [$($impl_generics:tt)*] [$($ty_generics:tt)*] [$(where $($where_clause:tt)*)?]
    ) => {
        // Ensure that struct does not implement `Drop`.
        //
        // There are two possible cases:
        // 1. The user type does not implement Drop. In this case,
        // the first blanked impl will not apply to it. This code
        // will compile, as there is only one impl of MustNotImplDrop for the user type
        // 2. The user type does impl Drop. This will make the blanket impl applicable,
        // which will then conflict with the explicit MustNotImplDrop impl below.
        // This will result in a compilation error, which is exactly what we want.
        trait MustNotImplDrop {}
        #[allow(clippy::drop_bounds, drop_bounds)]
        impl<T: $crate::__private::Drop> MustNotImplDrop for T {}
        impl <$($impl_generics)*> MustNotImplDrop for $ident <$($ty_generics)*>
        $(where
            $($where_clause)*)?
        {
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_unpin_bound {
    (#[pin] $field_ty:ty) => {
        $field_ty
    };
    ($field_ty:ty) => {
        $crate::__private::AlwaysUnpin<$field_ty>
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_unsafe_field_proj {
    (#[pin] $field:ident) => {
        $crate::__private::Pin::new_unchecked($field)
    };
    ($field:ident) => {
        $field
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_replace_field_proj {
    (#[pin] $field:ident) => {
        $crate::__private::PhantomData
    };
    ($field:ident) => {
        $crate::__private::ptr::read($field)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_unsafe_drop_in_place_guard {
    (#[pin] $field:ident) => {
        $crate::__private::UnsafeDropInPlaceGuard::new($field)
    };
    ($field:ident) => {
        ()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_proj_field_mut {
    (#[pin] $field_ty:ty) => {
        $crate::__private::Pin<&'__pin mut ($field_ty)>
    };
    ($field_ty:ty) => {
        &'__pin mut ($field_ty)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_proj_field_ref {
    (#[pin] $field_ty:ty) => {
        $crate::__private::Pin<&'__pin ($field_ty)>
    };
    ($field_ty:ty) => {
        &'__pin ($field_ty)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_make_proj_field_replace {
    (#[pin] $field_ty:ty) => {
        $crate::__private::PhantomData<$field_ty>
    };
    ($field_ty:ty) => {
        $field_ty
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_internal {
    // parsing proj_mut_ident
    (
        []
        [$($proj_ref_ident:ident)?]
        [$($proj_replace_ident:ident)?]
        [$($attrs:tt)*]

        #[project = $proj_mut_ident:ident]
        $($tt:tt)*
    ) => {
        $crate::__pin_project_internal! {
            [$proj_mut_ident]
            [$($proj_ref_ident)?]
            [$($proj_replace_ident)?]
            [$($attrs)*]
            $($tt)*
        }
    };
    // parsing proj_ref_ident
    (
        [$($proj_mut_ident:ident)?]
        []
        [$($proj_replace_ident:ident)?]
        [$($attrs:tt)*]

        #[project_ref = $proj_ref_ident:ident]
        $($tt:tt)*
    ) => {
        $crate::__pin_project_internal! {
            [$($proj_mut_ident)?]
            [$proj_ref_ident]
            [$($proj_replace_ident)?]
            [$($attrs)*]
            $($tt)*
        }
    };
    // parsing proj_replace_ident
    (
        [$($proj_mut_ident:ident)?]
        [$($proj_ref_ident:ident)?]
        []
        [$($attrs:tt)*]

        #[project_replace = $proj_replace_ident:ident]
        $($tt:tt)*
    ) => {
        $crate::__pin_project_internal! {
            [$($proj_mut_ident)?]
            [$($proj_ref_ident)?]
            [$proj_replace_ident]
            [$($attrs)*]
            $($tt)*
        }
    };
    // this is actually part of a recursive step that picks off a single non-`pin_project_lite` attribute
    // there could be more to parse
    (
        [$($proj_mut_ident:ident)?]
        [$($proj_ref_ident:ident)?]
        [$($proj_replace_ident:ident)?]
        [$($attrs:tt)*]

        #[$($attr:tt)*]
        $($tt:tt)*
    ) => {
        $crate::__pin_project_internal! {
            [$($proj_mut_ident)?]
            [$($proj_ref_ident)?]
            [$($proj_replace_ident)?]
            [$($attrs)* #[$($attr)*]]
            $($tt)*
        }
    };
    // now determine visibility
    // if public, downgrade
    (
        [$($proj_mut_ident:ident)?]
        [$($proj_ref_ident:ident)?]
        [$($proj_replace_ident:ident)?]
        [$($attrs:tt)*]
        pub $struct_ty_ident:ident $ident:ident
        $($tt:tt)*
    ) => {
        $crate::__pin_project_parse_generics! {
            [$($proj_mut_ident)?]
            [$($proj_ref_ident)?]
            [$($proj_replace_ident)?]
            [$($attrs)*]
            [pub $struct_ty_ident $ident pub(crate)]
            $($tt)*
        }
    };
    (
        [$($proj_mut_ident:ident)?]
        [$($proj_ref_ident:ident)?]
        [$($proj_replace_ident:ident)?]
        [$($attrs:tt)*]
        $vis:vis $struct_ty_ident:ident $ident:ident
        $($tt:tt)*
    ) => {
        $crate::__pin_project_parse_generics! {
            [$($proj_mut_ident)?]
            [$($proj_ref_ident)?]
            [$($proj_replace_ident)?]
            [$($attrs)*]
            [$vis $struct_ty_ident $ident $vis]
            $($tt)*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_parse_generics {
    (
        [$($proj_mut_ident:ident)?]
        [$($proj_ref_ident:ident)?]
        [$($proj_replace_ident:ident)?]
        [$($attrs:tt)*]
        [$vis:vis $struct_ty_ident:ident $ident:ident $proj_ty_vis:vis]
        $(<
            $( $lifetime:lifetime $(: $lifetime_bound:lifetime)? ),* $(,)?
            $( $generics:ident
                $(: $generics_bound:path)?
                $(: ?$generics_unsized_bound:path)?
                $(: $generics_lifetime_bound:lifetime)?
                $(= $generics_default:ty)?
            ),* $(,)?
        >)?
        $(where
            $( $where_clause_ty:ty
                $(: $where_clause_bound:path)?
                $(: ?$where_clause_unsized_bound:path)?
                $(: $where_clause_lifetime_bound:lifetime)?
            ),* $(,)?
        )?
        {
            $($body_data:tt)*
        }
        $(impl $($pinned_drop:tt)*)?
    ) => {
        $crate::__pin_project_expand! {
            [$($proj_mut_ident)?]
            [$($proj_ref_ident)?]
            [$($proj_replace_ident)?]
            [$proj_ty_vis]
            [$($attrs)* $vis $struct_ty_ident $ident]
            [$(<
                $( $lifetime $(: $lifetime_bound)? ,)*
                $( $generics
                    $(: $generics_bound)?
                    $(: ?$generics_unsized_bound)?
                    $(: $generics_lifetime_bound)?
                    $(= $generics_default)?
                ),*
            >)?]
            [$(
                $( $lifetime $(: $lifetime_bound)? ,)*
                $( $generics
                    $(: $generics_bound)?
                    $(: ?$generics_unsized_bound)?
                    $(: $generics_lifetime_bound)?
                ),*
            )?]
            [$( $( $lifetime ,)* $( $generics ),* )?]
            [$(where $( $where_clause_ty
                $(: $where_clause_bound)?
                $(: ?$where_clause_unsized_bound)?
                $(: $where_clause_lifetime_bound)?
            ),* )?]
            {
                $($body_data)*
            }
            $(impl $($pinned_drop)*)?
        }
    };
}

#[doc(hidden)]
pub mod __private {
    use core::mem::ManuallyDrop;
    #[doc(hidden)]
    pub use core::{
        marker::{PhantomData, Unpin},
        ops::Drop,
        pin::Pin,
        ptr,
    };

    // This is an internal helper struct used by `pin_project!`.
    #[doc(hidden)]
    pub struct AlwaysUnpin<T: ?Sized>(PhantomData<T>);

    impl<T: ?Sized> Unpin for AlwaysUnpin<T> {}

    // This is an internal helper used to ensure a value is dropped.
    #[doc(hidden)]
    pub struct UnsafeDropInPlaceGuard<T: ?Sized>(*mut T);

    impl<T: ?Sized> UnsafeDropInPlaceGuard<T> {
        #[doc(hidden)]
        pub unsafe fn new(ptr: *mut T) -> Self {
            Self(ptr)
        }
    }

    impl<T: ?Sized> Drop for UnsafeDropInPlaceGuard<T> {
        fn drop(&mut self) {
            unsafe {
                ptr::drop_in_place(self.0);
            }
        }
    }

    // This is an internal helper used to ensure a value is overwritten without
    // its destructor being called.
    #[doc(hidden)]
    pub struct UnsafeOverwriteGuard<T> {
        target: *mut T,
        value: ManuallyDrop<T>,
    }

    impl<T> UnsafeOverwriteGuard<T> {
        #[doc(hidden)]
        pub unsafe fn new(target: *mut T, value: T) -> Self {
            Self { target, value: ManuallyDrop::new(value) }
        }
    }

    impl<T> Drop for UnsafeOverwriteGuard<T> {
        fn drop(&mut self) {
            unsafe {
                ptr::write(self.target, ptr::read(&*self.value));
            }
        }
    }
}
