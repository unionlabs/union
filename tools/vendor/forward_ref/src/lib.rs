//! Macros to simplify extending operator traits over references.
//!
//! Adapted from the Rust core library [internal_macros.rs] to remove standard
//! library internal attribute annotations and add usage documentation.
//!
//! [internal_macros.rs]: https://github.com/rust-lang/rust/blob/master/library/core/src/internal_macros.rs

/// Extend a unary operator trait impl over refs.
///
/// Given an implementation of `op T` where T is `Copy`able, implements the unary
/// operator `op &T`.
///
/// # Examples
/**
```rust
use core::ops::Neg;
use forward_ref::forward_ref_unop;

#[derive(Clone, Copy, Debug, PartialEq)]
struct MyInt(i32);

impl Neg for MyInt {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self(self.0.neg())
    }
}

forward_ref_unop!(impl Neg, neg for MyInt);

// Now negation will work for references.
let a = MyInt(1);

assert_eq!(-a, MyInt(-1));
assert_eq!(-&a, MyInt(-1));
```
*/
#[macro_export]
macro_rules! forward_ref_unop {
    (impl $imp:ident, $method:ident for $t:ty) => {
        impl $imp for &$t {
            type Output = <$t as $imp>::Output;

            #[inline]
            fn $method(self) -> <$t as $imp>::Output {
                $imp::$method(*self)
            }
        }
    };
}

/// Extend a binary operator trait impl over refs.
///
/// Given an implementation of `T op U` where T and U are `Copy`able, implements
/// the binary operators:
/// - `&T op U`
/// - `T op &U`
/// - `&T op &U`
///
/// # Examples
/**
```rust
use core::ops::Add;
use forward_ref::forward_ref_binop;

#[derive(Clone, Copy, Debug, PartialEq)]
struct MyInt(i32);

impl Add for MyInt {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

forward_ref_binop!(impl Add, add for MyInt, MyInt);

// Now addition will work for any combination of references and values.
let a = MyInt(1);
let b = MyInt(2);

assert_eq!(a + b, MyInt(3));
assert_eq!(&a + b, MyInt(3));
assert_eq!(a + &b, MyInt(3));
assert_eq!(&a + &b, MyInt(3));
```
*/
#[macro_export]
macro_rules! forward_ref_binop {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl<'a> $imp<$u> for &'a $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: $u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, other)
            }
        }

        impl $imp<&$u> for $t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(self, *other)
            }
        }

        impl $imp<&$u> for &$t {
            type Output = <$t as $imp<$u>>::Output;

            #[inline]
            fn $method(self, other: &$u) -> <$t as $imp<$u>>::Output {
                $imp::$method(*self, *other)
            }
        }
    };
}

/// Extend an assignment operator trait impl over refs.
///
/// Given an implementation of `T op= U` where U is `Copy`able, implements
/// the binary operator `T op= &U`.
///
/// # Examples
/**
```rust
use core::ops::AddAssign;
use forward_ref::forward_ref_op_assign;

#[derive(Clone, Copy, Debug, PartialEq)]
struct MyInt(i32);

impl AddAssign for MyInt {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

forward_ref_op_assign!(impl AddAssign, add_assign for MyInt, MyInt);

// Now addition assignment will also work for references.
let mut a = MyInt(1);
let b = MyInt(2);

a += b;
assert_eq!(a, MyInt(3));

a += &b;
assert_eq!(a, MyInt(5));
```
*/
#[macro_export]
macro_rules! forward_ref_op_assign {
    (impl $imp:ident, $method:ident for $t:ty, $u:ty) => {
        impl $imp<&$u> for $t {
            #[inline]
            fn $method(&mut self, other: &$u) {
                $imp::$method(self, *other);
            }
        }
    };
}
