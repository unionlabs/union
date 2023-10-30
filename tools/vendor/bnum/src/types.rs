//! Type aliases for big signed and unsigned integers. Each is an alias for either a [`BUint`] or a [`BInt`].

use crate::{BInt, BUint};

macro_rules! int_type_doc {
    ($bits: literal, $sign: literal) => {
        concat!($bits, "-bit ", $sign, " integer type.")
    };
}

macro_rules! int_types {
    { $($bits: literal $u: ident $i: ident; ) *}  => {
        $(
            #[doc = int_type_doc!($bits, "unsigned")]
            pub type $u = BUint::<{$bits / 64}>;

            #[doc = int_type_doc!($bits, "signed")]
            pub type $i = BInt::<{$bits / 64}>;
        )*
    };
}

macro_rules! call_types_macro {
    ($name: ident) => {
        $name! {
            128 U128 I128;
            256 U256 I256;
            512 U512 I512;
            1024 U1024 I1024;
            2048 U2048 I2048;
            4096 U4096 I4096;
            8192 U8192 I8192;
        }
    };
}

call_types_macro!(int_types);

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_int_bits {
        { $($bits: literal $u: ident $i: ident; ) *} => {
            $(
                assert_eq!($u::BITS, $bits);
                assert_eq!($i::BITS, $bits);
            )*
        }
    }

    #[test]
    fn test_int_bits() {
        call_types_macro!(assert_int_bits);
    }
}
