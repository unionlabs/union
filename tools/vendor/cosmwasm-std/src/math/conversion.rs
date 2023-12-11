/// Grows a big endian signed integer to a bigger size.
/// See <https://en.wikipedia.org/wiki/Sign_extension>
pub const fn grow_be_int<const INPUT_SIZE: usize, const OUTPUT_SIZE: usize>(
    input: [u8; INPUT_SIZE],
) -> [u8; OUTPUT_SIZE] {
    debug_assert!(INPUT_SIZE <= OUTPUT_SIZE);
    // check if sign bit is set
    let mut output = if input[0] & 0b10000000 != 0 {
        // negative number is filled up with 1s
        [0b11111111u8; OUTPUT_SIZE]
    } else {
        [0u8; OUTPUT_SIZE]
    };
    let mut i = 0;

    // copy input to the end of output
    // copy_from_slice is not const, so we have to do this manually
    while i < INPUT_SIZE {
        output[OUTPUT_SIZE - INPUT_SIZE + i] = input[i];
        i += 1;
    }
    output
}

/// Shrinks a big endian signed integer to a smaller size.
/// This is the opposite operation of sign extension.
pub fn shrink_be_int<const INPUT_SIZE: usize, const OUTPUT_SIZE: usize>(
    input: [u8; INPUT_SIZE],
) -> Option<[u8; OUTPUT_SIZE]> {
    debug_assert!(INPUT_SIZE >= OUTPUT_SIZE);

    // check bounds
    if input[0] & 0b10000000 != 0 {
        // a negative number should start with only 1s, otherwise it's too small
        for i in &input[0..(INPUT_SIZE - OUTPUT_SIZE)] {
            if *i != 0b11111111u8 {
                return None;
            }
        }
        // the sign bit also has to be 1
        if input[INPUT_SIZE - OUTPUT_SIZE] & 0b10000000 == 0 {
            return None;
        }
    } else {
        // a positive number should start with only 0s, otherwise it's too large
        for i in &input[0..(INPUT_SIZE - OUTPUT_SIZE)] {
            if *i != 0u8 {
                return None;
            }
        }
        // the sign bit also has to be 0
        if input[INPUT_SIZE - OUTPUT_SIZE] & 0b10000000 != 0 {
            return None;
        }
    }

    // Now, we can just copy the last bytes
    let mut output = [0u8; OUTPUT_SIZE];
    output.copy_from_slice(&input[(INPUT_SIZE - OUTPUT_SIZE)..]);
    Some(output)
}

/// Helper macro to implement `TryFrom` for a type that is just a wrapper around another type.
/// This can be used for all our integer conversions where `bnum` implements `TryFrom`.
macro_rules! forward_try_from {
    ($input: ty, $output: ty) => {
        impl TryFrom<$input> for $output {
            type Error = $crate::ConversionOverflowError;

            fn try_from(value: $input) -> Result<Self, Self::Error> {
                value
                    .0
                    .try_into()
                    .map(Self)
                    .map_err(|_| Self::Error::new(stringify!($input), stringify!($output), value))
            }
        }
    };
}
pub(crate) use forward_try_from;

/// Helper macro to implement `TryFrom` for a conversion from a bigger signed int to a smaller one.
/// This is needed because `bnum` does not implement `TryFrom` for those conversions
/// because of limitations of const generics.
macro_rules! try_from_int_to_int {
    ($input: ty, $output: ty) => {
        // statically assert that the input is bigger than the output
        static_assertions::const_assert!(
            core::mem::size_of::<$input>() > core::mem::size_of::<$output>()
        );
        impl TryFrom<$input> for $output {
            type Error = $crate::ConversionOverflowError;

            fn try_from(value: $input) -> Result<Self, Self::Error> {
                $crate::math::conversion::shrink_be_int(value.to_be_bytes())
                    .ok_or_else(|| Self::Error::new(stringify!($input), stringify!($output), value))
                    .map(Self::from_be_bytes)
            }
        }
    };
}
pub(crate) use try_from_int_to_int;

/// Helper macro to implement `TryFrom` for a conversion from a unsigned int to a smaller or
/// equal sized signed int.
/// This is needed because `bnum` does not implement `TryFrom` for all of those conversions.
macro_rules! try_from_uint_to_int {
    ($input: ty, $output: ty) => {
        // statically assert that...
        // input is unsigned
        static_assertions::const_assert_eq!(stringify!($input).as_bytes()[0], b'U');
        // output is signed
        static_assertions::const_assert_eq!(stringify!($output).as_bytes()[0], b'I');
        // input is bigger than output (otherwise we would not need a `TryFrom` impl)
        static_assertions::const_assert!(
            core::mem::size_of::<$input>() >= core::mem::size_of::<$output>()
        );

        impl TryFrom<$input> for $output {
            type Error = $crate::ConversionOverflowError;

            fn try_from(value: $input) -> Result<Self, Self::Error> {
                use bnum::prelude::As;
                // $input::MAX has to be bigger than $output::MAX, so we can just cast it
                if value.0 > Self::MAX.0.as_() {
                    return Err(Self::Error::new(
                        stringify!($input),
                        stringify!($output),
                        value,
                    ));
                }

                // at this point we know it fits
                Ok(Self(value.0.as_()))
            }
        }
    };
}
pub(crate) use try_from_uint_to_int;

#[cfg(test)]
pub(crate) fn test_try_from_uint_to_int<I, O>(input_type: &'static str, output_type: &'static str)
where
    I: super::num_consts::NumConsts
        + From<u32>
        + Copy
        + TryFrom<O, Error = crate::ConversionOverflowError>
        + core::fmt::Debug
        + core::ops::Add<Output = I>,
    O: TryFrom<I, Error = crate::ConversionOverflowError>
        + From<u32>
        + super::num_consts::NumConsts
        + core::cmp::PartialEq
        + core::fmt::Debug,
    String: From<I>,
{
    let v = I::MAX;
    assert_eq!(
        O::try_from(v),
        Err(crate::ConversionOverflowError::new(
            input_type,
            output_type,
            v
        )),
        "input::MAX value should not fit"
    );

    let max = I::try_from(O::MAX).unwrap();
    assert_eq!(O::try_from(max), Ok(O::MAX), "output::MAX value should fit");

    // but $output::MAX + 1 should not fit
    let v = max + I::ONE;
    assert_eq!(
        O::try_from(v),
        Err(crate::ConversionOverflowError::new(
            input_type,
            output_type,
            v
        )),
        "output::MAX + 1 should not fit"
    );

    // zero should work
    let v = I::ZERO;
    assert_eq!(O::try_from(v), Ok(O::ZERO), "zero should fit");

    // 42 should work
    assert_eq!(
        O::try_from(I::from(42u32)),
        Ok(O::from(42u32)),
        "42 should fit"
    )
}

#[cfg(test)]
pub(crate) fn test_try_from_int_to_uint<I, O>(input_type: &'static str, output_type: &'static str)
where
    I: super::num_consts::NumConsts
        + From<i32>
        + Copy
        + TryFrom<O>
        + core::fmt::Debug
        + core::ops::Add<Output = I>,
    O: TryFrom<I, Error = crate::ConversionOverflowError>
        + From<u32>
        + super::num_consts::NumConsts
        + core::cmp::PartialEq
        + core::fmt::Debug,
    String: From<I>,
    <I as std::convert::TryFrom<O>>::Error: std::fmt::Debug,
{
    if core::mem::size_of::<I>() <= core::mem::size_of::<O>() {
        // if the input type is smaller than the output type, then `I::MAX` should fit into `O`
        let v = I::MAX;
        assert_eq!(
            O::try_from(v),
            Ok(O::try_from(v).unwrap()),
            "input::MAX value should fit"
        );
    } else {
        // if the input is bigger than the output, then `I::MAX` should not fit into `O`
        let v = I::MAX;
        assert_eq!(
            O::try_from(v),
            Err(crate::ConversionOverflowError::new(
                input_type,
                output_type,
                v
            )),
            "input::MAX value should not fit"
        );
        // but `O::MAX` should fit
        let max = I::try_from(O::MAX).unwrap();
        assert_eq!(
            O::try_from(max),
            Ok(O::try_from(max).unwrap()),
            "output::MAX value should fit"
        );
        // while `O::MAX + 1` should not
        let v = max + I::ONE;
        assert_eq!(
            O::try_from(v),
            Err(crate::ConversionOverflowError::new(
                input_type,
                output_type,
                v
            )),
            "output::MAX + 1 should not fit"
        );
    }

    // negative numbers should fail
    let v = I::from(-42i32);
    assert_eq!(
        O::try_from(v),
        Err(crate::ConversionOverflowError::new(
            input_type,
            output_type,
            v
        )),
        "negative numbers should not fit"
    );

    // zero should work
    let v = I::ZERO;
    assert_eq!(O::try_from(v), Ok(O::ZERO), "zero should fit");

    // 42 should work
    assert_eq!(
        O::try_from(I::from(42i32)),
        Ok(O::from(42u32)),
        "42 should fit"
    )
}

/// Helper macro to implement `TryFrom` for a conversion from a signed int to an unsigned int.
/// This is needed because `bnum` does not implement `TryFrom` for all of those conversions.
macro_rules! try_from_int_to_uint {
    ($input: ty, $output: ty) => {
        // statically assert that...
        // input is signed
        static_assertions::const_assert_eq!(stringify!($input).as_bytes()[0], b'I');
        // output is unsigned
        static_assertions::const_assert_eq!(stringify!($output).as_bytes()[0], b'U');

        impl TryFrom<$input> for $output {
            type Error = ConversionOverflowError;

            fn try_from(value: $input) -> Result<Self, Self::Error> {
                use bnum::prelude::As;
                // if $input::MAX is smaller than $output::MAX, we only need to check the sign
                if core::mem::size_of::<$input>() <= core::mem::size_of::<$output>() {
                    if value.is_negative() {
                        return Err(ConversionOverflowError::new(
                            stringify!($input),
                            stringify!($output),
                            value,
                        ));
                    }

                    // otherwise we can just cast it
                    Ok(Self(value.0.as_()))
                } else {
                    // $output::MAX is smaller than $input::MAX.
                    // If it is negative or too big, we error.
                    // We can safely cast $output::MAX to $input size
                    if value.is_negative() || value.0 > <$output>::MAX.0.as_() {
                        return Err(ConversionOverflowError::new(
                            stringify!($input),
                            stringify!($output),
                            value,
                        ));
                    }

                    // at this point we know it fits
                    Ok(Self(value.0.as_()))
                }
            }
        }
    };
}
pub(crate) use try_from_int_to_uint;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grow_be_int_works() {
        // test against rust std's integers
        let i32s = [i32::MIN, -1, 0, 1, 42, i32::MAX];
        for i in i32s {
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i64).to_be_bytes());
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i128).to_be_bytes());
        }
        let i8s = [i8::MIN, -1, 0, 1, 42, i8::MAX];
        for i in i8s {
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i16).to_be_bytes());
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i32).to_be_bytes());
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i64).to_be_bytes());
            assert_eq!(grow_be_int(i.to_be_bytes()), (i as i128).to_be_bytes());
        }
    }

    #[test]
    fn shrink_be_int_works() {
        // test against rust std's integers
        let i32s = [-42, -1, 0i32, 1, 42];
        for i in i32s {
            assert_eq!(
                shrink_be_int(i.to_be_bytes()),
                Some((i as i16).to_be_bytes())
            );
            assert_eq!(
                shrink_be_int(i.to_be_bytes()),
                Some((i as i8).to_be_bytes())
            );
        }
        // these should be too big to fit into an i16 or i8
        let oob = [
            i32::MIN,
            i32::MIN + 10,
            i32::MIN + 1234,
            i32::MAX - 1234,
            i32::MAX - 10,
            i32::MAX,
        ];
        for i in oob {
            // 32 -> 16 bit
            assert_eq!(shrink_be_int::<4, 2>(i.to_be_bytes()), None);
            // 32 -> 8 bit
            assert_eq!(shrink_be_int::<4, 1>(i.to_be_bytes()), None);
        }

        // compare against whole i16 range
        for i in i16::MIN..=i16::MAX {
            let cast = i as i8 as i16;
            if i == cast {
                // if the cast is lossless, `shrink_be_int` should get the same result
                assert_eq!(
                    shrink_be_int::<2, 1>(i.to_be_bytes()),
                    Some((i as i8).to_be_bytes())
                );
            } else {
                // otherwise, we should get None
                assert_eq!(shrink_be_int::<2, 1>(i.to_be_bytes()), None);
            }
        }
    }
}
