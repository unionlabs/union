macro_rules! impl_desc {
    ($ty: ty) => {
        concat!(
            "Methods which convert a `",
            stringify!($ty),
            "` between different types in a given radix (base)."
        )
    };
}

pub(crate) use impl_desc;

macro_rules! parse_str_radix {
    ($ty: ty) => {
        concat!(
"This function works the same as `from_str_radix` except that it returns a `",
stringify!($ty),
"` instead of a `Result<",
stringify!($ty),
", ParseIntError>`.

# Panics

This function panics if `radix` is not in the range from 2 to 36 inclusive, or if the string cannot be parsed successfully.

# Examples

Basic usage:

```compile_fail
// The below example will fail to compile, as the function will panic at compile time:
use bnum::types::U256;

const DOESNT_COMPILE: U256 = U256::parse_str_radix(\"a13423\", 10);
// Gives a compile error of \"error[E0080]: evaluation of constant value failed... the evaluated program panicked at 'attempt to parse integer from string containing invalid digit'\", 
```"
        )
    }
}

pub(crate) use parse_str_radix;
