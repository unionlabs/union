//! Small library providing some macros helpful for asserting. The API is very
//! similar to the API provided by the stdlib's own `assert_eq!`, `assert_ne!`,
//! `debug_assert_eq!`, or `debug_assert_ne!`.
//!
//! | Name                 | Enabled                       | Equivalent to                                |
//! | -------------------- | ----------------------------- | -------------------------------------------- |
//! | `assert_le!`         | Always                        | `assert!(a <= b)`                            |
//! | `assert_lt!`         | Always                        | `assert!(a < b)`                             |
//! | `assert_ge!`         | Always                        | `assert!(a >= b)`                            |
//! | `assert_gt!`         | Always                        | `assert!(a > b)`                             |
//! | `debug_assert_le!`   | `if cfg!(debug_assertions)`   | `debug_assert!(a <= b)`                      |
//! | `debug_assert_lt!`   | `if cfg!(debug_assertions)`   | `debug_assert!(a < b)`                       |
//! | `debug_assert_ge!`   | `if cfg!(debug_assertions)`   | `debug_assert!(a >= b)`                      |
//! | `debug_assert_gt!`   | `if cfg!(debug_assertions)`   | `debug_assert!(a > b)`                       |
//! | `debug_unreachable!` | `if cfg!(debug_assertions)`   | `unreachable!` when debug_assertions are on. |
//!
//! When one of the assertions fails, it prints out a message like the following:
//!
//! ```text
//! thread 'main' panicked at 'assertion failed: `left < right`
//!  left: `4`,
//! right: `3`', src/main.rs:47:5
//! note: Run with `RUST_BACKTRACE=1` for a backtrace.
//! ```
//!
//! # Example
//!
//! ```rust
//! #[macro_use]
//! extern crate more_asserts;
//!
//! #[derive(Debug, PartialEq, PartialOrd)]
//! enum Example { Foo, Bar }
//!
//! fn main() {
//!     assert_le!(3, 4);
//!     assert_ge!(10, 10,
//!                "You can pass a message too (just like `assert_eq!`)");
//!     debug_assert_lt!(1.3, 4.5,
//!         "Format syntax is supported ({}).", "also like `assert_eq!`");
//!
//!     assert_gt!(Example::Bar, Example::Foo,
//!         "It works on anything that implements PartialOrd, PartialEq, and Debug!");
//! }
//! ```

#![no_std]
#![deny(missing_docs)]

#[cfg(test)]
extern crate std;

/// Panics if the first expression is not strictly less than the second.
/// Requires that the values be comparable with `<`.
///
/// On failure, panics and prints the values out in a manner similar to
/// prelude's `assert_eq!`.
///
/// Optionally may take an additional message to display on failure, which
/// is formatted using standard format syntax.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate more_asserts;
///
/// fn main() {
///     assert_lt!(3, 4);
///     assert_lt!(3, 4, "With a message");
///     assert_lt!(3, 4, "With a formatted message: {}", "oh no");
/// }
/// ```
#[macro_export]
macro_rules! assert_lt {
    ($left:expr, $right:expr) => ({
        let (left, right) = (&($left), &($right));
        if !(left < right) {
            panic!("assertion failed: `(left < right)`\n  left: `{:?}`,\n right: `{:?}`",
                   left, right);
        }
    });
    ($left:expr, $right:expr, ) => ({
        assert_lt!($left, $right);
    });
    ($left:expr, $right:expr, $($msg_args:tt)+) => ({
        let (left, right) = (&($left), &($right));
        if !(left < right) {
            panic!("assertion failed: `(left < right)`\n  left: `{:?}`,\n right: `{:?}`: {}",
                   left, right, format_args!($($msg_args)+));
        }
    })
}

/// Panics if the first expression is not strictly greater than the second.
/// Requires that the values be comparable with `>`.
///
/// On failure, panics and prints the values out in a manner similar to
/// prelude's `assert_eq!`.
///
/// Optionally may take an additional message to display on failure, which
/// is formatted using standard format syntax.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate more_asserts;
///
/// fn main() {
///     assert_gt!(5, 3);
///     assert_gt!(5, 3, "With a message");
///     assert_gt!(5, 3, "With a formatted message: {}", "oh no");
/// }
/// ```
#[macro_export]
macro_rules! assert_gt {
    ($left:expr, $right:expr) => ({
        let (left, right) = (&($left), &($right));
        if !(left > right) {
            panic!("assertion failed: `(left > right)`\n  left: `{:?}`,\n right: `{:?}`",
                   left, right);
        }
    });
    ($left:expr, $right:expr, ) => ({
        assert_gt!($left, $right);
    });
    ($left:expr, $right:expr, $($msg_args:tt)+) => ({
        let (left, right) = (&($left), &($right));
        if !(left > right) {
            panic!("assertion failed: `(left > right)`\n  left: `{:?}`,\n right: `{:?}`: {}",
                   left, right, format_args!($($msg_args)+));
        }
    })
}

/// Panics if the first expression is not less than or equal to the second.
/// Requires that the values be comparable with `<=`.
///
/// On failure, panics and prints the values out in a manner similar to
/// prelude's `assert_eq!`.
///
/// Optionally may take an additional message to display on failure, which
/// is formatted using standard format syntax.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate more_asserts;
///
/// fn main() {
///     assert_le!(4, 4);
///     assert_le!(4, 5);
///     assert_le!(4, 5, "With a message");
///     assert_le!(4, 4, "With a formatted message: {}", "oh no");
/// }
/// ```
#[macro_export]
macro_rules! assert_le {
    ($left:expr, $right:expr) => ({
        let (left, right) = (&($left), &($right));
        if !(left <= right) {
            panic!("assertion failed: `(left <= right)`\n  left: `{:?}`,\n right: `{:?}`",
                   left, right);
        }
    });
    ($left:expr, $right:expr, ) => ({
        assert_le!($left, $right);
    });
    ($left:expr, $right:expr, $($msg_args:tt)+) => ({
        let (left, right) = (&($left), &($right));
        if !(left <= right) {
            panic!("assertion failed: `(left <= right)`\n  left: `{:?}`,\n right: `{:?}`: {}",
                   left, right, format_args!($($msg_args)+));
        }
    })
}

/// Panics if the first expression is not greater than or equal to the second.
/// Requires that the values be comparable with `>=`.
///
/// On failure, panics and prints the values out in a manner similar to
/// prelude's `assert_eq!`.
///
/// Optionally may take an additional message to display on failure, which
/// is formatted using standard format syntax.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate more_asserts;
///
/// fn main() {
///     assert_ge!(4, 4);
///     assert_ge!(4, 3);
///     assert_ge!(4, 3, "With a message");
///     assert_ge!(4, 4, "With a formatted message: {}", "oh no");
/// }
/// ```
#[macro_export]
macro_rules! assert_ge {
    ($left:expr, $right:expr) => ({
        let (left, right) = (&($left), &($right));
        if !(left >= right) {
            panic!("assertion failed: `(left >= right)`\n  left: `{:?}`,\n right: `{:?}`",
                   left, right);
        }
    });
    ($left:expr, $right:expr, ) => ({
        assert_ge!($left, $right);
    });
    ($left:expr, $right:expr, $($msg_args:tt)+) => ({
        let (left, right) = (&($left), &($right));
        if !(left >= right) {
            panic!("assertion failed: `(left >= right)`\n  left: `{:?}`,\n right: `{:?}`: {}",
                   left, right, format_args!($($msg_args)+));
        }
    })
}

/// Same as `assert_lt!` in debug builds or release builds where the
/// `-C debug-assertions` was provided to the compiler. For all other builds,
/// vanishes without a trace.
///
/// Optionally may take an additional message to display on failure, which
/// is formatted using standard format syntax.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate more_asserts;
///
/// fn main() {
///     // These are compiled to nothing if debug_assertions are off!
///     debug_assert_lt!(3, 4);
///     debug_assert_lt!(3, 4, "With a message");
///     debug_assert_lt!(3, 4, "With a formatted message: {}", "oh no");
/// }
/// ```
#[macro_export]
macro_rules! debug_assert_lt {
    ($($arg:tt)+) => {
        if cfg!(debug_assertions) {
            assert_lt!($($arg)+);
        }
    }
}

/// Same as `assert_gt!` in debug builds or release builds where the
/// `-C debug-assertions` was provided to the compiler. For all other builds,
/// vanishes without a trace.
///
/// Optionally may take an additional message to display on failure, which
/// is formatted using standard format syntax.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate more_asserts;
///
/// fn main() {
///     // These are compiled to nothing if debug_assertions are off!
///     debug_assert_gt!(5, 3);
///     debug_assert_gt!(5, 3, "With a message");
///     debug_assert_gt!(5, 3, "With a formatted message: {}", "oh no");
/// }
/// ```
#[macro_export]
macro_rules! debug_assert_gt {
    ($($arg:tt)+) => {
        if cfg!(debug_assertions) {
            assert_gt!($($arg)+);
        }
    }
}

/// Same as `assert_le!` in debug builds or release builds where the
/// `-C debug-assertions` was provided to the compiler. For all other builds,
/// vanishes without a trace.
///
/// Optionally may take an additional message to display on failure, which
/// is formatted using standard format syntax.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate more_asserts;
///
/// fn main() {
///     // These are compiled to nothing if debug_assertions are off!
///     debug_assert_le!(4, 4);
///     debug_assert_le!(4, 5);
///     debug_assert_le!(4, 5, "With a message");
///     debug_assert_le!(4, 4, "With a formatted message: {}", "oh no");
/// }
/// ```
#[macro_export]
macro_rules! debug_assert_le {
    ($($arg:tt)+) => {
        if cfg!(debug_assertions) {
            assert_le!($($arg)+);
        }
    }
}

/// Same as `assert_ge!` in debug builds or release builds where the
/// `-C debug-assertions` was provided to the compiler. For all other builds,
/// vanishes without a trace.
///
/// Optionally may take an additional message to display on failure, which
/// is formatted using standard format syntax.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate more_asserts;
///
/// fn main() {
///     // These are compiled to nothing if debug_assertions are off!
///     debug_assert_ge!(4, 4);
///     debug_assert_ge!(4, 3);
///     debug_assert_ge!(4, 3, "With a message");
///     debug_assert_ge!(4, 4, "With a formatted message: {}", "oh no");
/// }
/// ```
#[macro_export]
macro_rules! debug_assert_ge {
    ($($arg:tt)+) => {
        if cfg!(debug_assertions) {
            assert_ge!($($arg)+);
        }
    }
}

/// Panics if reached. This is a variant of the standard library's `unreachable!`
/// macro that is controlled by `cfg!(debug_assertions)`.
///
/// Same as prelude's `unreachable!` in debug builds or release builds where the
/// `-C debug-assertions` was provided to the compiler. For all other builds,
/// vanishes without a trace.
///
/// # Example
///
/// ```rust
/// #[macro_use]
/// extern crate more_asserts;
///
/// fn main() {
///     // You probably wouldn't actually use this here
///     let mut value = 0.5;
///     if value < 0.0 {
///         debug_unreachable!("Value out of range {}", value);
///         value = 0.0;
///     }
/// }
/// ```
#[macro_export]
macro_rules! debug_unreachable {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            unreachable!($($arg)*);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::panic::catch_unwind;

    #[derive(PartialOrd, PartialEq, Debug)]
    enum DummyType {
        Foo, Bar, Baz
    }

    #[test]
    fn test_assert_lt() {
        assert_lt!(3, 4);
        assert_lt!(4.0, 4.5);
        assert_lt!("a string", "b string");
        assert_lt!(DummyType::Foo, DummyType::Bar,
                   "Message with {}", "cool formatting");

        let a = &DummyType::Foo;
        let b = &DummyType::Baz;
        assert_lt!(a, b);

        assert!(catch_unwind(|| assert_lt!(5, 3)).is_err());
        assert!(catch_unwind(|| assert_lt!(5, 5)).is_err());
        assert!(catch_unwind(|| assert_lt!(DummyType::Bar, DummyType::Foo)).is_err());
    }

    #[test]
    fn test_assert_gt() {
        assert_gt!(4, 3);
        assert_gt!(4.5, 4.0);
        assert_gt!("b string", "a string");
        assert_gt!(DummyType::Bar, DummyType::Foo,
                   "Message with {}", "cool formatting");

        let a = &DummyType::Foo;
        let b = &DummyType::Baz;
        assert_gt!(b, a);

        assert!(catch_unwind(|| assert_gt!(3, 5)).is_err());
        assert!(catch_unwind(|| assert_gt!(5, 5)).is_err());
        assert!(catch_unwind(|| assert_gt!(DummyType::Foo, DummyType::Bar)).is_err());
    }

    #[test]
    fn test_assert_le() {
        assert_le!(3, 4);
        assert_le!(4, 4);
        assert_le!(4.0, 4.5);
        assert_le!("a string", "a string");
        assert_le!("a string", "b string");
        assert_le!(DummyType::Foo, DummyType::Bar, "Message");
        assert_le!(DummyType::Foo, DummyType::Foo,
                   "Message with {}", "cool formatting");

        let a = &DummyType::Foo;
        let b = &DummyType::Baz;
        assert_le!(a, a);
        assert_le!(a, b);

        assert!(catch_unwind(|| assert_le!(5, 3)).is_err());
        assert!(catch_unwind(|| assert_le!(DummyType::Bar, DummyType::Foo)).is_err());
    }

    #[test]
    fn test_assert_ge() {
        assert_ge!(4, 3);
        assert_ge!(4, 4);
        assert_ge!(4.5, 4.0);
        assert_ge!(5.0, 5.0);
        assert_ge!("a string", "a string");
        assert_ge!("b string", "a string");
        assert_ge!(DummyType::Bar, DummyType::Bar, "Example");
        assert_ge!(DummyType::Bar, DummyType::Foo,
                   "Message with {}", "cool formatting");

        let a = &DummyType::Foo;
        let b = &DummyType::Baz;
        assert_ge!(a, a);
        assert_ge!(b, a);

        assert!(catch_unwind(|| assert_ge!(3, 5)).is_err());
        assert!(catch_unwind(|| assert_ge!(DummyType::Foo, DummyType::Bar)).is_err());
    }

    // @@TODO: how to test the debug macros?
}
