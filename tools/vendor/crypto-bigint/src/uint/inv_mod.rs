use super::Uint;
use crate::{CtChoice, Limb};

impl<const LIMBS: usize> Uint<LIMBS> {
    /// Computes 1/`self` mod 2^k as specified in Algorithm 4 from
    /// A Secure Algorithm for Inversion Modulo 2k by
    /// Sadiel de la Fe and Carles Ferrer. See
    /// <https://www.mdpi.com/2410-387X/2/3/23>.
    ///
    /// Conditions: `self` < 2^k and `self` must be odd
    pub const fn inv_mod2k(&self, k: usize) -> Self {
        let mut x = Self::ZERO;
        let mut b = Self::ONE;
        let mut i = 0;

        while i < k {
            let mut x_i = Self::ZERO;
            let j = b.limbs[0].0 & 1;
            x_i.limbs[0] = Limb(j);
            x = x.bitor(&x_i.shl_vartime(i));

            let t = b.wrapping_sub(self);
            b = Self::ct_select(&b, &t, CtChoice::from_lsb(j)).shr_vartime(1);
            i += 1;
        }
        x
    }

    /// Computes the multiplicative inverse of `self` mod `modulus`, where `modulus` is odd.
    /// In other words `self^-1 mod modulus`.
    /// `bits` and `modulus_bits` are the bounds on the bit size
    /// of `self` and `modulus`, respectively
    /// (the inversion speed will be proportional to `bits + modulus_bits`).
    /// The second element of the tuple is the truthy value if an inverse exists,
    /// otherwise it is a falsy value.
    ///
    /// **Note:** variable time in `bits` and `modulus_bits`.
    ///
    /// The algorithm is the same as in GMP 6.2.1's `mpn_sec_invert`.
    pub const fn inv_odd_mod_bounded(
        &self,
        modulus: &Self,
        bits: usize,
        modulus_bits: usize,
    ) -> (Self, CtChoice) {
        debug_assert!(modulus.ct_is_odd().is_true_vartime());

        let mut a = *self;

        let mut u = Uint::ONE;
        let mut v = Uint::ZERO;

        let mut b = *modulus;

        // `bit_size` can be anything >= `self.bits()` + `modulus.bits()`, setting to the minimum.
        let bit_size = bits + modulus_bits;

        let mut m1hp = *modulus;
        let (m1hp_new, carry) = m1hp.shr_1();
        debug_assert!(carry.is_true_vartime());
        m1hp = m1hp_new.wrapping_add(&Uint::ONE);

        let mut i = 0;
        while i < bit_size {
            debug_assert!(b.ct_is_odd().is_true_vartime());

            let self_odd = a.ct_is_odd();

            // Set `self -= b` if `self` is odd.
            let (new_a, swap) = a.conditional_wrapping_sub(&b, self_odd);
            // Set `b += self` if `swap` is true.
            b = Uint::ct_select(&b, &b.wrapping_add(&new_a), swap);
            // Negate `self` if `swap` is true.
            a = new_a.conditional_wrapping_neg(swap);

            let (new_u, new_v) = Uint::ct_swap(&u, &v, swap);
            let (new_u, cy) = new_u.conditional_wrapping_sub(&new_v, self_odd);
            let (new_u, cyy) = new_u.conditional_wrapping_add(modulus, cy);
            debug_assert!(cy.is_true_vartime() == cyy.is_true_vartime());

            let (new_a, overflow) = a.shr_1();
            debug_assert!(!overflow.is_true_vartime());
            let (new_u, cy) = new_u.shr_1();
            let (new_u, cy) = new_u.conditional_wrapping_add(&m1hp, cy);
            debug_assert!(!cy.is_true_vartime());

            a = new_a;
            u = new_u;
            v = new_v;

            i += 1;
        }

        debug_assert!(!a.ct_is_nonzero().is_true_vartime());

        (v, Uint::ct_eq(&b, &Uint::ONE))
    }

    /// Computes the multiplicative inverse of `self` mod `modulus`, where `modulus` is odd.
    /// Returns `(inverse, Word::MAX)` if an inverse exists, otherwise `(undefined, Word::ZERO)`.
    pub const fn inv_odd_mod(&self, modulus: &Self) -> (Self, CtChoice) {
        self.inv_odd_mod_bounded(modulus, Uint::<LIMBS>::BITS, Uint::<LIMBS>::BITS)
    }
}

#[cfg(test)]
mod tests {
    use crate::{U1024, U256, U64};

    #[test]
    fn inv_mod2k() {
        let v =
            U256::from_be_hex("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f");
        let e =
            U256::from_be_hex("3642e6faeaac7c6663b93d3d6a0d489e434ddc0123db5fa627c7f6e22ddacacf");
        let a = v.inv_mod2k(256);
        assert_eq!(e, a);

        let v =
            U256::from_be_hex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141");
        let e =
            U256::from_be_hex("261776f29b6b106c7680cf3ed83054a1af5ae537cb4613dbb4f20099aa774ec1");
        let a = v.inv_mod2k(256);
        assert_eq!(e, a);
    }

    #[test]
    fn test_invert() {
        let a = U1024::from_be_hex(concat![
            "000225E99153B467A5B451979A3F451DAEF3BF8D6C6521D2FA24BBB17F29544E",
            "347A412B065B75A351EA9719E2430D2477B11CC9CF9C1AD6EDEE26CB15F463F8",
            "BCC72EF87EA30288E95A48AA792226CEC959DCB0672D8F9D80A54CBBEA85CAD8",
            "382EC224DEB2F5784E62D0CC2F81C2E6AD14EBABE646D6764B30C32B87688985"
        ]);
        let m = U1024::from_be_hex(concat![
            "D509E7854ABDC81921F669F1DC6F61359523F3949803E58ED4EA8BC16483DC6F",
            "37BFE27A9AC9EEA2969B357ABC5C0EE214BE16A7D4C58FC620D5B5A20AFF001A",
            "D198D3155E5799DC4EA76652D64983A7E130B5EACEBAC768D28D589C36EC749C",
            "558D0B64E37CD0775C0D0104AE7D98BA23C815185DD43CD8B16292FD94156767"
        ]);

        let (res, is_some) = a.inv_odd_mod(&m);

        let expected = U1024::from_be_hex(concat![
            "B03623284B0EBABCABD5C5881893320281460C0A8E7BF4BFDCFFCBCCBF436A55",
            "D364235C8171E46C7D21AAD0680676E57274A8FDA6D12768EF961CACDD2DAE57",
            "88D93DA5EB8EDC391EE3726CDCF4613C539F7D23E8702200CB31B5ED5B06E5CA",
            "3E520968399B4017BF98A864FABA2B647EFC4998B56774D4F2CB026BC024A336"
        ]);
        assert!(is_some.is_true_vartime());
        assert_eq!(res, expected);
    }

    #[test]
    fn test_invert_bounded() {
        let a = U1024::from_be_hex(concat![
            "0000000000000000000000000000000000000000000000000000000000000000",
            "347A412B065B75A351EA9719E2430D2477B11CC9CF9C1AD6EDEE26CB15F463F8",
            "BCC72EF87EA30288E95A48AA792226CEC959DCB0672D8F9D80A54CBBEA85CAD8",
            "382EC224DEB2F5784E62D0CC2F81C2E6AD14EBABE646D6764B30C32B87688985"
        ]);
        let m = U1024::from_be_hex(concat![
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "D198D3155E5799DC4EA76652D64983A7E130B5EACEBAC768D28D589C36EC749C",
            "558D0B64E37CD0775C0D0104AE7D98BA23C815185DD43CD8B16292FD94156767"
        ]);

        let (res, is_some) = a.inv_odd_mod_bounded(&m, 768, 512);

        let expected = U1024::from_be_hex(concat![
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0000000000000000000000000000000000000000000000000000000000000000",
            "0DCC94E2FE509E6EBBA0825645A38E73EF85D5927C79C1AD8FFE7C8DF9A822FA",
            "09EB396A21B1EF05CBE51E1A8EF284EF01EBDD36A9A4EA17039D8EEFDD934768"
        ]);
        assert!(is_some.is_true_vartime());
        assert_eq!(res, expected);
    }

    #[test]
    fn test_invert_small() {
        let a = U64::from(3u64);
        let m = U64::from(13u64);

        let (res, is_some) = a.inv_odd_mod(&m);

        assert!(is_some.is_true_vartime());
        assert_eq!(U64::from(9u64), res);
    }

    #[test]
    fn test_no_inverse_small() {
        let a = U64::from(14u64);
        let m = U64::from(49u64);

        let (_res, is_some) = a.inv_odd_mod(&m);

        assert!(!is_some.is_true_vartime());
    }
}
