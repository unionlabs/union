//! Constant-time comparison helpers for [`ByteArray`].

use crate::{ArrayLength, ByteArray};
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq};

/// Constant-time equals.
pub(crate) fn ct_eq<N: ArrayLength<u8>>(a: &ByteArray<N>, b: &ByteArray<N>) -> Choice {
    let mut ret = Choice::from(1);

    for (a, b) in a.iter().zip(b.iter()) {
        ret.conditional_assign(&Choice::from(0), !a.ct_eq(b));
    }

    ret
}

/// Constant-time less than.
///
/// Inputs are interpreted as big endian integers.
pub(crate) fn ct_lt<N: ArrayLength<u8>>(a: &ByteArray<N>, b: &ByteArray<N>) -> Choice {
    let mut borrow = 0;

    // Perform subtraction with borrow a byte-at-a-time, interpreting a
    // no-borrow condition as the less-than case
    for (&a, &b) in a.iter().zip(b.iter()).rev() {
        let c = (b as u16).wrapping_add(borrow >> (u8::BITS - 1));
        borrow = (a as u16).wrapping_sub(c) >> u8::BITS as u8;
    }

    !borrow.ct_eq(&0)
}

#[cfg(test)]
mod tests {
    const A: [u8; 4] = [0, 0, 0, 0];
    const B: [u8; 4] = [0, 0, 0, 1];
    const C: [u8; 4] = [0xFF, 0, 0, 0];
    const D: [u8; 4] = [0xFF, 0, 0, 1];
    const E: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFE];
    const F: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];

    #[test]
    fn ct_eq() {
        use super::ct_eq;

        assert_eq!(ct_eq(&A.into(), &A.into()).unwrap_u8(), 1);
        assert_eq!(ct_eq(&B.into(), &B.into()).unwrap_u8(), 1);
        assert_eq!(ct_eq(&C.into(), &C.into()).unwrap_u8(), 1);
        assert_eq!(ct_eq(&D.into(), &D.into()).unwrap_u8(), 1);
        assert_eq!(ct_eq(&E.into(), &E.into()).unwrap_u8(), 1);
        assert_eq!(ct_eq(&F.into(), &F.into()).unwrap_u8(), 1);

        assert_eq!(ct_eq(&A.into(), &B.into()).unwrap_u8(), 0);
        assert_eq!(ct_eq(&C.into(), &D.into()).unwrap_u8(), 0);
        assert_eq!(ct_eq(&E.into(), &F.into()).unwrap_u8(), 0);
    }

    #[test]
    fn ct_lt() {
        use super::ct_lt;

        assert_eq!(ct_lt(&A.into(), &A.into()).unwrap_u8(), 0);
        assert_eq!(ct_lt(&B.into(), &B.into()).unwrap_u8(), 0);
        assert_eq!(ct_lt(&C.into(), &C.into()).unwrap_u8(), 0);
        assert_eq!(ct_lt(&D.into(), &D.into()).unwrap_u8(), 0);
        assert_eq!(ct_lt(&E.into(), &E.into()).unwrap_u8(), 0);
        assert_eq!(ct_lt(&F.into(), &F.into()).unwrap_u8(), 0);

        assert_eq!(ct_lt(&A.into(), &B.into()).unwrap_u8(), 1);
        assert_eq!(ct_lt(&A.into(), &C.into()).unwrap_u8(), 1);
        assert_eq!(ct_lt(&B.into(), &A.into()).unwrap_u8(), 0);
        assert_eq!(ct_lt(&C.into(), &A.into()).unwrap_u8(), 0);

        assert_eq!(ct_lt(&B.into(), &C.into()).unwrap_u8(), 1);
        assert_eq!(ct_lt(&B.into(), &D.into()).unwrap_u8(), 1);
        assert_eq!(ct_lt(&C.into(), &B.into()).unwrap_u8(), 0);
        assert_eq!(ct_lt(&D.into(), &B.into()).unwrap_u8(), 0);

        assert_eq!(ct_lt(&C.into(), &D.into()).unwrap_u8(), 1);
        assert_eq!(ct_lt(&C.into(), &E.into()).unwrap_u8(), 1);
        assert_eq!(ct_lt(&D.into(), &C.into()).unwrap_u8(), 0);
        assert_eq!(ct_lt(&E.into(), &C.into()).unwrap_u8(), 0);

        assert_eq!(ct_lt(&E.into(), &F.into()).unwrap_u8(), 1);
        assert_eq!(ct_lt(&F.into(), &E.into()).unwrap_u8(), 0);
    }
}
