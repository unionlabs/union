// -*- mode: rust; -*-
//
// This file is part of curve25519-dalek.
// Copyright (c) 2016-2021 isis agora lovecruft
// Copyright (c) 2016-2019 Henry de Valence
// See LICENSE for licensing information.
//
// Authors:
// - isis agora lovecruft <isis@patternsinthevoid.net>
// - Henry de Valence <hdevalence@hdevalence.ca>

//! Internal macros.

/// Define borrow and non-borrow variants of `Add`.
macro_rules! define_add_variants {
    (LHS = $lhs:ty, RHS = $rhs:ty, Output = $out:ty) => {
        impl<'b> Add<&'b $rhs> for $lhs {
            type Output = $out;
            fn add(self, rhs: &'b $rhs) -> $out {
                &self + rhs
            }
        }

        impl<'a> Add<$rhs> for &'a $lhs {
            type Output = $out;
            fn add(self, rhs: $rhs) -> $out {
                self + &rhs
            }
        }

        impl Add<$rhs> for $lhs {
            type Output = $out;
            fn add(self, rhs: $rhs) -> $out {
                &self + &rhs
            }
        }
    }
}

/// Define non-borrow variants of `AddAssign`.
macro_rules! define_add_assign_variants {
    (LHS = $lhs:ty, RHS = $rhs:ty) => {
        impl AddAssign<$rhs> for $lhs {
            fn add_assign(&mut self, rhs: $rhs) {
                *self += &rhs;
            }
        }
    }
}

/// Define borrow and non-borrow variants of `Sub`.
macro_rules! define_sub_variants {
    (LHS = $lhs:ty, RHS = $rhs:ty, Output = $out:ty) => {
        impl<'b> Sub<&'b $rhs> for $lhs {
            type Output = $out;
            fn sub(self, rhs: &'b $rhs) -> $out {
                &self - rhs
            }
        }

        impl<'a> Sub<$rhs> for &'a $lhs {
            type Output = $out;
            fn sub(self, rhs: $rhs) -> $out {
                self - &rhs
            }
        }

        impl Sub<$rhs> for $lhs {
            type Output = $out;
            fn sub(self, rhs: $rhs) -> $out {
                &self - &rhs
            }
        }
    }
}

/// Define non-borrow variants of `SubAssign`.
macro_rules! define_sub_assign_variants {
    (LHS = $lhs:ty, RHS = $rhs:ty) => {
        impl SubAssign<$rhs> for $lhs {
            fn sub_assign(&mut self, rhs: $rhs) {
                *self -= &rhs;
            }
        }
    }
}

/// Define borrow and non-borrow variants of `Mul`.
macro_rules! define_mul_variants {
    (LHS = $lhs:ty, RHS = $rhs:ty, Output = $out:ty) => {
        impl<'b> Mul<&'b $rhs> for $lhs {
            type Output = $out;
            fn mul(self, rhs: &'b $rhs) -> $out {
                &self * rhs
            }
        }

        impl<'a> Mul<$rhs> for &'a $lhs {
            type Output = $out;
            fn mul(self, rhs: $rhs) -> $out {
                self * &rhs
            }
        }

        impl Mul<$rhs> for $lhs {
            type Output = $out;
            fn mul(self, rhs: $rhs) -> $out {
                &self * &rhs
            }
        }
    }
}

/// Define non-borrow variants of `MulAssign`.
macro_rules! define_mul_assign_variants {
    (LHS = $lhs:ty, RHS = $rhs:ty) => {
        impl MulAssign<$rhs> for $lhs {
            fn mul_assign(&mut self, rhs: $rhs) {
                *self *= &rhs;
            }
        }
    }
}

