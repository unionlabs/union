// Adapted from https://github.com/Alexhuszagh/rust-lexical.

use crate::lexical::exponent::*;

#[test]
fn scientific_exponent_test() {
    // 0 digits in the integer
    assert_eq!(scientific_exponent(0, 0, 5), -6);
    assert_eq!(scientific_exponent(10, 0, 5), 4);
    assert_eq!(scientific_exponent(-10, 0, 5), -16);

    // >0 digits in the integer
    assert_eq!(scientific_exponent(0, 1, 5), 0);
    assert_eq!(scientific_exponent(0, 2, 5), 1);
    assert_eq!(scientific_exponent(0, 2, 20), 1);
    assert_eq!(scientific_exponent(10, 2, 20), 11);
    assert_eq!(scientific_exponent(-10, 2, 20), -9);

    // Underflow
    assert_eq!(
        scientific_exponent(i32::min_value(), 0, 0),
        i32::min_value()
    );
    assert_eq!(
        scientific_exponent(i32::min_value(), 0, 5),
        i32::min_value()
    );

    // Overflow
    assert_eq!(
        scientific_exponent(i32::max_value(), 0, 0),
        i32::max_value() - 1
    );
    assert_eq!(
        scientific_exponent(i32::max_value(), 5, 0),
        i32::max_value()
    );
}

#[test]
fn mantissa_exponent_test() {
    assert_eq!(mantissa_exponent(10, 5, 0), 5);
    assert_eq!(mantissa_exponent(0, 5, 0), -5);
    assert_eq!(
        mantissa_exponent(i32::max_value(), 5, 0),
        i32::max_value() - 5
    );
    assert_eq!(mantissa_exponent(i32::max_value(), 0, 5), i32::max_value());
    assert_eq!(mantissa_exponent(i32::min_value(), 5, 0), i32::min_value());
    assert_eq!(
        mantissa_exponent(i32::min_value(), 0, 5),
        i32::min_value() + 5
    );
}
