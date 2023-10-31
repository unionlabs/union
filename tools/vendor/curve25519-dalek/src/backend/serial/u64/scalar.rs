//! Arithmetic mod \\(2\^{252} + 27742317777372353535851937790883648493\\)
//! with five \\(52\\)-bit unsigned limbs.
//!
//! \\(51\\)-bit limbs would cover the desired bit range (\\(253\\)
//! bits), but isn't large enough to reduce a \\(512\\)-bit number with
//! Montgomery multiplication, so \\(52\\) bits is used instead.  To see
//! that this is safe for intermediate results, note that the largest
//! limb in a \\(5\times 5\\) product of \\(52\\)-bit limbs will be
//!
//! ```text
//! (0xfffffffffffff^2) * 5 = 0x4ffffffffffff60000000000005 (107 bits).
//! ```

use core::fmt::Debug;
use core::ops::{Index, IndexMut};

use zeroize::Zeroize;

use constants;

/// The `Scalar52` struct represents an element in
/// \\(\mathbb Z / \ell \mathbb Z\\) as 5 \\(52\\)-bit limbs.
#[derive(Copy,Clone)]
pub struct Scalar52(pub [u64; 5]);

impl Debug for Scalar52 {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "Scalar52: {:?}", &self.0[..])
    }
}

impl Zeroize for Scalar52 {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl Index<usize> for Scalar52 {
    type Output = u64;
    fn index(&self, _index: usize) -> &u64 {
        &(self.0[_index])
    }
}

impl IndexMut<usize> for Scalar52 {
    fn index_mut(&mut self, _index: usize) -> &mut u64 {
        &mut (self.0[_index])
    }
}

/// u64 * u64 = u128 multiply helper
#[inline(always)]
fn m(x: u64, y: u64) -> u128 {
    (x as u128) * (y as u128)
}

impl Scalar52 {
    /// Return the zero scalar
    pub fn zero() -> Scalar52 {
        Scalar52([0,0,0,0,0])
    }

    /// Unpack a 32 byte / 256 bit scalar into 5 52-bit limbs.
    pub fn from_bytes(bytes: &[u8; 32]) -> Scalar52 {
        let mut words = [0u64; 4];
        for i in 0..4 {
            for j in 0..8 {
                words[i] |= (bytes[(i * 8) + j] as u64) << (j * 8);
            }
        }

        let mask = (1u64 << 52) - 1;
        let top_mask = (1u64 << 48) - 1;
        let mut s = Scalar52::zero();

        s[ 0] =   words[0]                            & mask;
        s[ 1] = ((words[0] >> 52) | (words[1] << 12)) & mask;
        s[ 2] = ((words[1] >> 40) | (words[2] << 24)) & mask;
        s[ 3] = ((words[2] >> 28) | (words[3] << 36)) & mask;
        s[ 4] =  (words[3] >> 16)                     & top_mask;

        s
    }

    /// Reduce a 64 byte / 512 bit scalar mod l
    pub fn from_bytes_wide(bytes: &[u8; 64]) -> Scalar52 {
        let mut words = [0u64; 8];
        for i in 0..8 {
            for j in 0..8 {
                words[i] |= (bytes[(i * 8) + j] as u64) << (j * 8);
            }
        }

        let mask = (1u64 << 52) - 1;
        let mut lo = Scalar52::zero();
        let mut hi = Scalar52::zero();

        lo[0] =   words[ 0]                             & mask;
        lo[1] = ((words[ 0] >> 52) | (words[ 1] << 12)) & mask;
        lo[2] = ((words[ 1] >> 40) | (words[ 2] << 24)) & mask;
        lo[3] = ((words[ 2] >> 28) | (words[ 3] << 36)) & mask;
        lo[4] = ((words[ 3] >> 16) | (words[ 4] << 48)) & mask;
        hi[0] =  (words[ 4] >>  4)                      & mask;
        hi[1] = ((words[ 4] >> 56) | (words[ 5] <<  8)) & mask;
        hi[2] = ((words[ 5] >> 44) | (words[ 6] << 20)) & mask;
        hi[3] = ((words[ 6] >> 32) | (words[ 7] << 32)) & mask;
        hi[4] =   words[ 7] >> 20                             ;

        lo = Scalar52::montgomery_mul(&lo, &constants::R);  // (lo * R) / R = lo
        hi = Scalar52::montgomery_mul(&hi, &constants::RR); // (hi * R^2) / R = hi * R

        Scalar52::add(&hi, &lo)
    }

    /// Pack the limbs of this `Scalar52` into 32 bytes
    pub fn to_bytes(&self) -> [u8; 32] {
        let mut s = [0u8; 32];

        s[0]  =  (self.0[ 0] >>  0)                      as u8;
        s[1]  =  (self.0[ 0] >>  8)                      as u8;
        s[2]  =  (self.0[ 0] >> 16)                      as u8;
        s[3]  =  (self.0[ 0] >> 24)                      as u8;
        s[4]  =  (self.0[ 0] >> 32)                      as u8;
        s[5]  =  (self.0[ 0] >> 40)                      as u8;
        s[6]  = ((self.0[ 0] >> 48) | (self.0[ 1] << 4)) as u8;
        s[7]  =  (self.0[ 1] >>  4)                      as u8;
        s[8]  =  (self.0[ 1] >> 12)                      as u8;
        s[9]  =  (self.0[ 1] >> 20)                      as u8;
        s[10] =  (self.0[ 1] >> 28)                      as u8;
        s[11] =  (self.0[ 1] >> 36)                      as u8;
        s[12] =  (self.0[ 1] >> 44)                      as u8;
        s[13] =  (self.0[ 2] >>  0)                      as u8;
        s[14] =  (self.0[ 2] >>  8)                      as u8;
        s[15] =  (self.0[ 2] >> 16)                      as u8;
        s[16] =  (self.0[ 2] >> 24)                      as u8;
        s[17] =  (self.0[ 2] >> 32)                      as u8;
        s[18] =  (self.0[ 2] >> 40)                      as u8;
        s[19] = ((self.0[ 2] >> 48) | (self.0[ 3] << 4)) as u8;
        s[20] =  (self.0[ 3] >>  4)                      as u8;
        s[21] =  (self.0[ 3] >> 12)                      as u8;
        s[22] =  (self.0[ 3] >> 20)                      as u8;
        s[23] =  (self.0[ 3] >> 28)                      as u8;
        s[24] =  (self.0[ 3] >> 36)                      as u8;
        s[25] =  (self.0[ 3] >> 44)                      as u8;
        s[26] =  (self.0[ 4] >>  0)                      as u8;
        s[27] =  (self.0[ 4] >>  8)                      as u8;
        s[28] =  (self.0[ 4] >> 16)                      as u8;
        s[29] =  (self.0[ 4] >> 24)                      as u8;
        s[30] =  (self.0[ 4] >> 32)                      as u8;
        s[31] =  (self.0[ 4] >> 40)                      as u8;

        s
    }

    /// Compute `a + b` (mod l)
    pub fn add(a: &Scalar52, b: &Scalar52) -> Scalar52 {
        let mut sum = Scalar52::zero();
        let mask = (1u64 << 52) - 1;

        // a + b
        let mut carry: u64 = 0;
        for i in 0..5 {
            carry = a[i] + b[i] + (carry >> 52);
            sum[i] = carry & mask;
        }

        // subtract l if the sum is >= l
        Scalar52::sub(&sum, &constants::L)
    }

    /// Compute `a - b` (mod l)
    pub fn sub(a: &Scalar52, b: &Scalar52) -> Scalar52 {
        let mut difference = Scalar52::zero();
        let mask = (1u64 << 52) - 1;

        // a - b
        let mut borrow: u64 = 0;
        for i in 0..5 {
            borrow = a[i].wrapping_sub(b[i] + (borrow >> 63));
            difference[i] = borrow & mask;
        }

        // conditionally add l if the difference is negative
        let underflow_mask = ((borrow >> 63) ^ 1).wrapping_sub(1);
        let mut carry: u64 = 0;
        for i in 0..5 {
            carry = (carry >> 52) + difference[i] + (constants::L[i] & underflow_mask);
            difference[i] = carry & mask;
        }

        difference
    }

    /// Compute `a * b`
    #[inline(always)]
    pub (crate) fn mul_internal(a: &Scalar52, b: &Scalar52) -> [u128; 9] {
        let mut z = [0u128; 9];

        z[0] = m(a[0],b[0]);
        z[1] = m(a[0],b[1]) + m(a[1],b[0]);
        z[2] = m(a[0],b[2]) + m(a[1],b[1]) + m(a[2],b[0]);
        z[3] = m(a[0],b[3]) + m(a[1],b[2]) + m(a[2],b[1]) + m(a[3],b[0]);
        z[4] = m(a[0],b[4]) + m(a[1],b[3]) + m(a[2],b[2]) + m(a[3],b[1]) + m(a[4],b[0]);
        z[5] =                m(a[1],b[4]) + m(a[2],b[3]) + m(a[3],b[2]) + m(a[4],b[1]);
        z[6] =                               m(a[2],b[4]) + m(a[3],b[3]) + m(a[4],b[2]);
        z[7] =                                              m(a[3],b[4]) + m(a[4],b[3]);
        z[8] =                                                             m(a[4],b[4]);

        z
    }

    /// Compute `a^2`
    #[inline(always)]
    fn square_internal(a: &Scalar52) -> [u128; 9] {
        let aa = [
            a[0]*2,
            a[1]*2,
            a[2]*2,
            a[3]*2,
        ];

        [
            m( a[0],a[0]),
            m(aa[0],a[1]),
            m(aa[0],a[2]) + m( a[1],a[1]),
            m(aa[0],a[3]) + m(aa[1],a[2]),
            m(aa[0],a[4]) + m(aa[1],a[3]) + m( a[2],a[2]),
                            m(aa[1],a[4]) + m(aa[2],a[3]),
                                            m(aa[2],a[4]) + m( a[3],a[3]),
                                                            m(aa[3],a[4]),
                                                                            m(a[4],a[4])
        ]
    }

    /// Compute `limbs/R` (mod l), where R is the Montgomery modulus 2^260
    #[inline(always)]
    pub (crate) fn montgomery_reduce(limbs: &[u128; 9]) -> Scalar52 {

        #[inline(always)]
        fn part1(sum: u128) -> (u128, u64) {
            let p = (sum as u64).wrapping_mul(constants::LFACTOR) & ((1u64 << 52) - 1);
            ((sum + m(p,constants::L[0])) >> 52, p)
        }

        #[inline(always)]
        fn part2(sum: u128) -> (u128, u64) {
            let w = (sum as u64) & ((1u64 << 52) - 1);
            (sum >> 52, w)
        }

        // note: l[3] is zero, so its multiples can be skipped
        let l = &constants::L;

        // the first half computes the Montgomery adjustment factor n, and begins adding n*l to make limbs divisible by R
        let (carry, n0) = part1(        limbs[0]);
        let (carry, n1) = part1(carry + limbs[1] + m(n0,l[1]));
        let (carry, n2) = part1(carry + limbs[2] + m(n0,l[2]) + m(n1,l[1]));
        let (carry, n3) = part1(carry + limbs[3]              + m(n1,l[2]) + m(n2,l[1]));
        let (carry, n4) = part1(carry + limbs[4] + m(n0,l[4])              + m(n2,l[2]) + m(n3,l[1]));

        // limbs is divisible by R now, so we can divide by R by simply storing the upper half as the result
        let (carry, r0) = part2(carry + limbs[5]              + m(n1,l[4])              + m(n3,l[2]) + m(n4,l[1]));
        let (carry, r1) = part2(carry + limbs[6]                           + m(n2,l[4])              + m(n4,l[2]));
        let (carry, r2) = part2(carry + limbs[7]                                        + m(n3,l[4])             );
        let (carry, r3) = part2(carry + limbs[8]                                                     + m(n4,l[4]));
        let         r4 = carry as u64;

        // result may be >= l, so attempt to subtract l
        Scalar52::sub(&Scalar52([r0,r1,r2,r3,r4]), l)
    }

    /// Compute `a * b` (mod l)
    #[inline(never)]
    pub fn mul(a: &Scalar52, b: &Scalar52) -> Scalar52 {
        let ab = Scalar52::montgomery_reduce(&Scalar52::mul_internal(a, b));
        Scalar52::montgomery_reduce(&Scalar52::mul_internal(&ab, &constants::RR))
    }

    /// Compute `a^2` (mod l)
    #[inline(never)]
    #[allow(dead_code)] // XXX we don't expose square() via the Scalar API
    pub fn square(&self) -> Scalar52 {
        let aa = Scalar52::montgomery_reduce(&Scalar52::square_internal(self));
        Scalar52::montgomery_reduce(&Scalar52::mul_internal(&aa, &constants::RR))
    }

    /// Compute `(a * b) / R` (mod l), where R is the Montgomery modulus 2^260
    #[inline(never)]
    pub fn montgomery_mul(a: &Scalar52, b: &Scalar52) -> Scalar52 {
        Scalar52::montgomery_reduce(&Scalar52::mul_internal(a, b))
    }

    /// Compute `(a^2) / R` (mod l) in Montgomery form, where R is the Montgomery modulus 2^260
    #[inline(never)]
    pub fn montgomery_square(&self) -> Scalar52 {
        Scalar52::montgomery_reduce(&Scalar52::square_internal(self))
    }

    /// Puts a Scalar52 in to Montgomery form, i.e. computes `a*R (mod l)`
    #[inline(never)]
    pub fn to_montgomery(&self) -> Scalar52 {
        Scalar52::montgomery_mul(self, &constants::RR)
    }

    /// Takes a Scalar52 out of Montgomery form, i.e. computes `a/R (mod l)`
    #[inline(never)]
    pub fn from_montgomery(&self) -> Scalar52 {
        let mut limbs = [0u128; 9];
        for i in 0..5 {
            limbs[i] = self[i] as u128;
        }
        Scalar52::montgomery_reduce(&limbs)
    }
}


#[cfg(test)]
mod test {
    use super::*;

    /// Note: x is 2^253-1 which is slightly larger than the largest scalar produced by
    /// this implementation (l-1), and should show there are no overflows for valid scalars
    ///
    /// x = 14474011154664524427946373126085988481658748083205070504932198000989141204991
    /// x = 7237005577332262213973186563042994240801631723825162898930247062703686954002 mod l
    /// x = 3057150787695215392275360544382990118917283750546154083604586903220563173085*R mod l in Montgomery form
    pub static X: Scalar52 = Scalar52(
        [0x000fffffffffffff, 0x000fffffffffffff, 0x000fffffffffffff, 0x000fffffffffffff,
         0x00001fffffffffff]);

    /// x^2 = 3078544782642840487852506753550082162405942681916160040940637093560259278169 mod l
    pub static XX: Scalar52 = Scalar52(
        [0x0001668020217559, 0x000531640ffd0ec0, 0x00085fd6f9f38a31, 0x000c268f73bb1cf4,
         0x000006ce65046df0]);

    /// x^2 = 4413052134910308800482070043710297189082115023966588301924965890668401540959*R mod l in Montgomery form
    pub static XX_MONT: Scalar52 = Scalar52(
        [0x000c754eea569a5c, 0x00063b6ed36cb215, 0x0008ffa36bf25886, 0x000e9183614e7543,
         0x0000061db6c6f26f]);

    /// y = 6145104759870991071742105800796537629880401874866217824609283457819451087098
    pub static Y: Scalar52 = Scalar52(
        [0x000b75071e1458fa, 0x000bf9d75e1ecdac, 0x000433d2baf0672b, 0x0005fffcc11fad13,
         0x00000d96018bb825]);

    /// x*y = 36752150652102274958925982391442301741 mod l
    pub static XY: Scalar52 = Scalar52(
        [0x000ee6d76ba7632d, 0x000ed50d71d84e02, 0x00000000001ba634, 0x0000000000000000,
         0x0000000000000000]);

    /// x*y = 658448296334113745583381664921721413881518248721417041768778176391714104386*R mod l in Montgomery form
    pub static XY_MONT: Scalar52 = Scalar52(
        [0x0006d52bf200cfd5, 0x00033fb1d7021570, 0x000f201bc07139d8, 0x0001267e3e49169e,
         0x000007b839c00268]);

    /// a = 2351415481556538453565687241199399922945659411799870114962672658845158063753
    pub static A: Scalar52 = Scalar52(
        [0x0005236c07b3be89, 0x0001bc3d2a67c0c4, 0x000a4aa782aae3ee, 0x0006b3f6e4fec4c4,
         0x00000532da9fab8c]);

    /// b = 4885590095775723760407499321843594317911456947580037491039278279440296187236
    pub static B: Scalar52 = Scalar52(
        [0x000d3fae55421564, 0x000c2df24f65a4bc, 0x0005b5587d69fb0b, 0x00094c091b013b3b,
         0x00000acd25605473]);

    /// a+b = 0
    /// a-b = 4702830963113076907131374482398799845891318823599740229925345317690316127506
    pub static AB: Scalar52 = Scalar52(
        [0x000a46d80f677d12, 0x0003787a54cf8188, 0x0004954f0555c7dc, 0x000d67edc9fd8989,
         0x00000a65b53f5718]);

    // c = (2^512 - 1) % l = 1627715501170711445284395025044413883736156588369414752970002579683115011840
    pub static C: Scalar52 = Scalar52(
        [0x000611e3449c0f00, 0x000a768859347a40, 0x0007f5be65d00e1b, 0x0009a3dceec73d21,
         0x00000399411b7c30]);

    #[test]
    fn mul_max() {
        let res = Scalar52::mul(&X, &X);
        for i in 0..5 {
            assert!(res[i] == XX[i]);
        }
    }

    #[test]
    fn square_max() {
        let res = X.square();
        for i in 0..5 {
            assert!(res[i] == XX[i]);
        }
    }

    #[test]
    fn montgomery_mul_max() {
        let res = Scalar52::montgomery_mul(&X, &X);
        for i in 0..5 {
            assert!(res[i] == XX_MONT[i]);
        }
    }

    #[test]
    fn montgomery_square_max() {
        let res = X.montgomery_square();
        for i in 0..5 {
            assert!(res[i] == XX_MONT[i]);
        }
    }

    #[test]
    fn mul() {
        let res = Scalar52::mul(&X, &Y);
        for i in 0..5 {
            assert!(res[i] == XY[i]);
        }
    }

    #[test]
    fn montgomery_mul() {
        let res = Scalar52::montgomery_mul(&X, &Y);
        for i in 0..5 {
            assert!(res[i] == XY_MONT[i]);
        }
    }

    #[test]
    fn add() {
        let res = Scalar52::add(&A, &B);
        let zero = Scalar52::zero();
        for i in 0..5 {
            assert!(res[i] == zero[i]);
        }
    }

    #[test]
    fn sub() {
        let res = Scalar52::sub(&A, &B);
        for i in 0..5 {
            assert!(res[i] == AB[i]);
        }
    }

    #[test]
    fn from_bytes_wide() {
        let bignum = [255u8; 64]; // 2^512 - 1
        let reduced = Scalar52::from_bytes_wide(&bignum);
        for i in 0..5 {
            assert!(reduced[i] == C[i]);
        }
    }
}
