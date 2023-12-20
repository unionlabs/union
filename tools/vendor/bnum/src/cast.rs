//! Panic-free casting between numeric types.

/// Backend implementation trait for panic-free casting between numeric types.

#[cfg_attr(feature = "nightly", const_trait)]
pub trait CastFrom<T> {
    fn cast_from(from: T) -> Self;
}

#[cfg_attr(feature = "nightly", const_trait)]
pub(crate) trait CastTo<U> {
    fn cast_to(self) -> U;
}

macro_rules! as_trait_doc {
    () => {
"Trait which allows panic-free casting between numeric types.

The behavior matches the behavior of the `as` conversion operator between primitive integers. This trait can be used to convert between bnum's integer types, as well as between bnum's integer types and Rust's primitive integers. Conversions between Rust's primitive integers themselves are also defined for consistency."
    };
}

macro_rules! as_method_doc {
    () => {
"Casts `self` to type `T`. The [semantics of numeric casting](https://doc.rust-lang.org/reference/expressions/operator-expr.html#semantics) with the `as` operator are followed, so `<T as As>::as_::<U>` can be used in the same way as `T as U` for numeric conversions.

# Examples
 
```
use bnum::types::{U256, I512, I256, U1024};
use bnum::cast::As;
 
// Cast `u64` to `U256`:
let a = 399872465243u64;
let b: U256 = a.as_();
assert_eq!(a.as_::<u16>(), b.as_());

// Cast `i128` to `I512`:
let c = -2098409234529234584094i128;
let d = c.as_::<I512>();
//assert_eq!(c.as::<I256>(), d.as_());

// Cast `I512` to `U1024` (result will be sign-extended with leading ones):
let e: U1024 = d.as_();
assert_eq!(d, e.as_());

// Cast `U256` to `f64` and back:
let f: f64 = b.as_();
assert_eq!(b, f.as_());
```"
    };
}

#[cfg(feature = "nightly")]
macro_rules! as_trait {
    () => {
        impl<T, U> const CastTo<U> for T
        where
            U: ~const CastFrom<T>,
        {
            fn cast_to(self) -> U {
                U::cast_from(self)
            }
        }

        #[doc = as_trait_doc!()]
        #[const_trait]
        pub trait As {
            #[doc = as_method_doc!()]
            fn as_<T>(self) -> T
            where
                T: CastFrom<Self>,
                Self: Sized;
        }

        impl<U> const As for U {
            #[inline]
            fn as_<T>(self) -> T
            where
                T: ~const CastFrom<Self>,
                Self: Sized,
            {
                T::cast_from(self)
            }
        }
    };
}

#[cfg(not(feature = "nightly"))]
macro_rules! as_trait {
    () => {
        impl<T, U> CastTo<U> for T
        where
            U: CastFrom<T>,
        {
            fn cast_to(self) -> U {
                U::cast_from(self)
            }
        }

        #[doc = as_trait_doc!()]
        pub trait As {
            #[doc = as_method_doc!()]
            fn as_<T>(self) -> T
            where
                T: CastFrom<Self>,
                Self: Sized;
        }

        impl<U> As for U {
            #[inline]
            fn as_<T>(self) -> T
            where
                T: CastFrom<Self>,
                Self: Sized,
            {
                T::cast_from(self)
            }
        }
    };
}

as_trait!();

macro_rules! primitive_cast_impl {
    ($from: ty as [$($ty: ty), *]) => {
        $(crate::nightly::const_impl! {
            impl const CastFrom<$from> for $ty {
                #[inline]
                fn cast_from(from: $from) -> Self {
                    from as Self
                }
            }
        })*
    };
}

macro_rules! multiple_impls {
    ($($from: ty), *) => {
        $(
            primitive_cast_impl!($from as [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64]);
        )*
    };
}

primitive_cast_impl!(bool as [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, bool]);
primitive_cast_impl!(char as [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, char]);
primitive_cast_impl!(u8 as [u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, char]);
multiple_impls!(u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);
