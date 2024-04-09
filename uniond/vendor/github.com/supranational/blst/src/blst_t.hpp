// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0

#ifndef __BLST_T_HPP__
#define __BLST_T_HPP__

/*
 * These templates, blst_384_t and blst_256_t, allow to instantiate slim
 * C++ shims to blst assembly with arbitrary moduli. Well, not literally
 * arbitrary, as there are limitations. Most notably blst_384_t can not
 * actually accommodate 384-bit moduli, only 383 and narrower. This is
 * because of ct_inverse_mod_383's limitation. Though if you abstain
 * from the reciprocal() method, even 384-bit modulus would work. As for
 * blst_256_t, modulus has to be not larger than 2^256-2^192-1.
 */

#ifdef __GNUC__
# pragma GCC diagnostic push
# pragma GCC diagnostic ignored "-Wunused-function"
#endif

extern "C" {
#include "vect.h"
}
#include "bytes.h"

#undef launder // avoid conflict with C++ >=17

#ifdef __GNUC__
# pragma GCC diagnostic pop
#endif

static inline void vec_left_align(limb_t *out, const limb_t *inp, size_t N)
{
    const unsigned int nbits = sizeof(inp[0])*8;
    const unsigned int align = (0 - N) % nbits;
    size_t n = (N + nbits - 1) / nbits;

    if (align) {
        limb_t top = inp[n-1] << align;

        while (--n) {
            limb_t next = inp[n-1];
            out[n] = top | next >> (nbits-align);
            top = next << align;
        }
        out[0] = top;
    } else {
        for (size_t i = 0; i < n; i++)
            out[i] = inp[i];
    }
}

template<const size_t N, const vec384 MOD, const limb_t M0,
                         const vec384 RR, const vec384 ONE>
class blst_384_t {
private:
    vec384 val;

    inline operator const limb_t*() const           { return val;    }
    inline operator limb_t*()                       { return val;    }
    inline limb_t& operator[](size_t i)             { return val[i]; }
    inline const limb_t& operator[](size_t i) const { return val[i]; }

    static const size_t n = sizeof(vec384)/sizeof(limb_t);
public:
    static const size_t nbits = N;
    static constexpr size_t bit_length() { return N; }
    static const unsigned int degree = 1;
    typedef byte pow_t[384/8];
    typedef blst_384_t mem_t;

    inline blst_384_t() {}
    inline blst_384_t(const vec384 p, bool align = false)
    {
        if (align)
            vec_left_align(val, p, N);
        else
            vec_copy(val, p, sizeof(val));
    }
    inline blst_384_t(uint64_t a)
    {
        vec_zero(val, sizeof(val));
        val[0] = a;
        if (a) to();
    }
    inline blst_384_t(int a) : blst_384_t((uint64_t)a) {}

    inline void to_scalar(pow_t& scalar) const
    {
        const union {
            long one;
            char little;
        } is_endian = { 1 };

        if ((size_t)scalar%sizeof(limb_t) == 0 && is_endian.little) {
            from_mont_384((limb_t *)scalar, val, MOD, M0);
        } else {
            vec384 out;
            from_mont_384(out, val, MOD, M0);
            le_bytes_from_limbs(scalar, out, sizeof(pow_t));
            vec_zero(out, sizeof(out));
        }
    }

    static inline const blst_384_t& one()
    {   return *reinterpret_cast<const blst_384_t*>(ONE);   }

    static inline blst_384_t one(bool or_zero)
    {
        blst_384_t ret;
        limb_t mask = ~((limb_t)0 - or_zero);
        for (size_t i = 0; i < n; i++)
            ret[i] = ONE[i] & mask;
        return ret;
    }

    inline blst_384_t& to()
    {   mul_mont_384(val, RR, val, MOD, M0);        return *this;   }
    inline blst_384_t& from()
    {   from_mont_384(val, val, MOD, M0);           return *this;   }

    inline void store(limb_t *p) const
    {   vec_copy(p, val, sizeof(val));   }

    inline blst_384_t& operator+=(const blst_384_t& b)
    {   add_mod_384(val, val, b, MOD);              return *this;   }
    friend inline blst_384_t operator+(const blst_384_t& a, const blst_384_t& b)
    {
        blst_384_t ret;
        add_mod_384(ret, a, b, MOD);
        return ret;
    }

    inline blst_384_t& operator<<=(unsigned l)
    {   lshift_mod_384(val, val, l, MOD);           return *this;   }
    friend inline blst_384_t operator<<(const blst_384_t& a, unsigned l)
    {
        blst_384_t ret;
        lshift_mod_384(ret, a, l, MOD);
        return ret;
    }

    inline blst_384_t& operator>>=(unsigned r)
    {   rshift_mod_384(val, val, r, MOD);           return *this;   }
    friend inline blst_384_t operator>>(const blst_384_t& a, unsigned r)
    {
        blst_384_t ret;
        rshift_mod_384(ret, a, r, MOD);
        return ret;
    }

    inline blst_384_t& operator-=(const blst_384_t& b)
    {   sub_mod_384(val, val, b, MOD);              return *this;   }
    friend inline blst_384_t operator-(const blst_384_t& a, const blst_384_t& b)
    {
        blst_384_t ret;
        sub_mod_384(ret, a, b, MOD);
        return ret;
    }

    inline blst_384_t& cneg(bool flag)
    {   cneg_mod_384(val, val, flag, MOD);          return *this;   }
    friend inline blst_384_t cneg(const blst_384_t& a, bool flag)
    {
        blst_384_t ret;
        cneg_mod_384(ret, a, flag, MOD);
        return ret;
    }
    friend inline blst_384_t operator-(const blst_384_t& a)
    {
        blst_384_t ret;
        cneg_mod_384(ret, a, true, MOD);
        return ret;
    }

    inline blst_384_t& operator*=(const blst_384_t& a)
    {
        if (this == &a) sqr_mont_384(val, val, MOD, M0);
        else            mul_mont_384(val, val, a, MOD, M0);
        return *this;
    }
    friend inline blst_384_t operator*(const blst_384_t& a, const blst_384_t& b)
    {
        blst_384_t ret;
        if (&a == &b)   sqr_mont_384(ret, a, MOD, M0);
        else            mul_mont_384(ret, a, b, MOD, M0);
        return ret;
    }

    // simplified exponentiation, but mind the ^ operator's precedence!
    friend inline blst_384_t operator^(const blst_384_t& a, unsigned p)
    {
        if (p < 2) {
            abort();
        } else if (p == 2) {
            blst_384_t ret;
            sqr_mont_384(ret, a, MOD, M0);
            return ret;
        } else {
            blst_384_t ret = a, sqr = a;
            if ((p&1) == 0) {
                do {
                    sqr_mont_384(sqr, sqr, MOD, M0);
                    p >>= 1;
                } while ((p&1) == 0);
                ret = sqr;
            }
            for (p >>= 1; p; p >>= 1) {
                sqr_mont_384(sqr, sqr, MOD, M0);
                if (p&1)
                    mul_mont_384(ret, ret, sqr, MOD, M0);
            }
            return ret;
        }
    }
    inline blst_384_t& operator^=(unsigned p)
    {
        if (p < 2) {
            abort();
        } else if (p == 2) {
            sqr_mont_384(val, val, MOD, M0);
            return *this;
        }
        return *this = *this^p;
    }
    inline blst_384_t operator()(unsigned p)
    {   return *this^p;   }
    friend inline blst_384_t sqr(const blst_384_t& a)
    {   return a^2;   }

    inline bool is_one() const
    {   return vec_is_equal(val, ONE, sizeof(val));   }

    inline int is_zero() const
    {   return vec_is_zero(val, sizeof(val));   }

    inline void zero()
    {   vec_zero(val, sizeof(val));   }

    friend inline blst_384_t czero(const blst_384_t& a, int set_z)
    {   blst_384_t ret;
        const vec384 zero = { 0 };
        vec_select(ret, zero, a, sizeof(ret), set_z);
        return ret;
    }

    static inline blst_384_t csel(const blst_384_t& a, const blst_384_t& b,
                                  int sel_a)
    {   blst_384_t ret;
        vec_select(ret, a, b, sizeof(ret), sel_a);
        return ret;
    }

    blst_384_t reciprocal() const
    {
        static const blst_384_t MODx{MOD, true};
        static const blst_384_t RRx4 = *reinterpret_cast<const blst_384_t*>(RR)<<2;
        union { vec768 x; vec384 r[2]; } temp;

        ct_inverse_mod_383(temp.x, val, MOD, MODx);
        redc_mont_384(temp.r[0], temp.x, MOD, M0);
        mul_mont_384(temp.r[0], temp.r[0], RRx4, MOD, M0);

        return *reinterpret_cast<blst_384_t*>(temp.r[0]);
    }
    friend inline blst_384_t operator/(unsigned one, const blst_384_t& a)
    {
        if (one == 1)
            return a.reciprocal();
        abort();
    }
    friend inline blst_384_t operator/(const blst_384_t& a, const blst_384_t& b)
    {   return a * b.reciprocal();   }
    inline blst_384_t& operator/=(const blst_384_t& a)
    {   return *this *= a.reciprocal();   }

#ifndef NDEBUG
    inline blst_384_t(const char *hexascii)
    {   limbs_from_hexascii(val, sizeof(val), hexascii); to();   }

    friend inline bool operator==(const blst_384_t& a, const blst_384_t& b)
    {   return vec_is_equal(a, b, sizeof(vec384));   }
    friend inline bool operator!=(const blst_384_t& a, const blst_384_t& b)
    {   return !vec_is_equal(a, b, sizeof(vec384));   }

# if defined(_GLIBCXX_IOSTREAM) || defined(_IOSTREAM_) // non-standard
    friend std::ostream& operator<<(std::ostream& os, const blst_384_t& obj)
    {
        unsigned char be[sizeof(obj)];
        char buf[2+2*sizeof(obj)+1], *str = buf;

        be_bytes_from_limbs(be, blst_384_t{obj}.from(), sizeof(obj));

        *str++ = '0', *str++ = 'x';
        for (size_t i = 0; i < sizeof(obj); i++)
            *str++ = hex_from_nibble(be[i]>>4), *str++ = hex_from_nibble(be[i]);
	*str = '\0';

        return os << buf;
    }
# endif
#endif
};

template<const size_t N, const vec256 MOD, const limb_t M0,
                         const vec256 RR, const vec256 ONE>
class blst_256_t {
    vec256 val;

    inline operator const limb_t*() const           { return val;    }
    inline operator limb_t*()                       { return val;    }
    inline limb_t& operator[](size_t i)             { return val[i]; }
    inline const limb_t& operator[](size_t i) const { return val[i]; }

    static const size_t n = sizeof(vec256)/sizeof(limb_t);
public:
    static const size_t nbits = N;
    static constexpr size_t bit_length() { return N; }
    static const unsigned int degree = 1;
    typedef byte pow_t[256/8];
    typedef blst_256_t mem_t;

    inline blst_256_t() {}
    inline blst_256_t(const vec256 p, bool align = false)
    {
        if (align)
            vec_left_align(val, p, N);
        else
            vec_copy(val, p, sizeof(val));
    }
    inline blst_256_t(uint64_t a)
    {
        vec_zero(val, sizeof(val));
        val[0] = a;
        if (a) to();
    }
    inline blst_256_t(int a) : blst_256_t((uint64_t)a) {}

    inline void to_scalar(pow_t& scalar) const
    {
        const union {
            long one;
            char little;
        } is_endian = { 1 };

        if ((size_t)scalar%sizeof(limb_t) == 0 && is_endian.little) {
            from_mont_256((limb_t *)scalar, val, MOD, M0);
        } else {
            vec256 out;
            from_mont_256(out, val, MOD, M0);
            le_bytes_from_limbs(scalar, out, sizeof(pow_t));
            vec_zero(out, sizeof(out));
        }
    }

    static inline const blst_256_t& one()
    {   return *reinterpret_cast<const blst_256_t*>(ONE);   }

    static inline blst_256_t one(bool or_zero)
    {
        blst_256_t ret;
        limb_t mask = ~((limb_t)0 - or_zero);
        for (size_t i = 0; i < n; i++)
            ret[i] = ONE[i] & mask;
        return ret;
    }

    inline blst_256_t& to()
    {   mul_mont_sparse_256(val, val, RR, MOD, M0); return *this;   }
    inline blst_256_t& to(const uint64_t a[2*n])
    {
        mul_mont_sparse_256(val, RR, (const limb_t*)(a + n), MOD, M0);
        add_mod_256(val, val, (const limb_t*)a, MOD);
        mul_mont_sparse_256(val, RR, val, MOD, M0);

        return *this;
    }
    blst_256_t& to(const unsigned char* bytes, size_t n, bool le = false)
    {
        vec_zero(val, sizeof(val));

        vec256 digit;
        size_t rem = (n - 1) % 32 + 1;
        n -= rem;

        if (le) {
            limbs_from_le_bytes(val, bytes += n, rem);
            mul_mont_sparse_256(val, RR, val, MOD, M0);
            while (n) {
                limbs_from_le_bytes(digit, bytes -= 32, 32);
                add_mod_256(val, val, digit, MOD);
                mul_mont_sparse_256(val, RR, val, MOD, M0);
                n -= 32;
            }
        } else {
            limbs_from_be_bytes(val, bytes, rem);
            mul_mont_sparse_256(val, RR, val, MOD, M0);
            bytes += rem;
            while (n) {
                limbs_from_be_bytes(digit, bytes, 32);
                add_mod_256(val, val, digit, MOD);
                mul_mont_sparse_256(val, RR, val, MOD, M0);
                bytes += 32;
                n -= 32;
            }
        }

        return *this;
    }

    inline blst_256_t& from()
    {   from_mont_256(val, val, MOD, M0); return *this;   }
    inline blst_256_t& from(const uint64_t a[2*n])
    {
        redc_mont_256(val, (const limb_t*)a, MOD, M0);
        mul_mont_sparse_256(val, RR, val, MOD, M0);

        return *this;
    }
    inline blst_256_t& from(const unsigned char *bytes, size_t n, bool le = false)
    {
        if (n > 64)
            return to(bytes, n, le).from();

        if (n > 32) {
            vec512 temp{0};
            if (le) limbs_from_le_bytes(temp, bytes, n);
            else    limbs_from_be_bytes(temp, bytes, n);
            redc_mont_256(val, temp, MOD, M0);
            mul_mont_sparse_256(val, RR, val, MOD, M0);
        } else {
            vec_zero(val, sizeof(val));
            if (le) limbs_from_le_bytes(val, bytes, n);
            else    limbs_from_be_bytes(val, bytes, n);
            mul_mont_sparse_256(val, ONE, val, MOD, M0);
        }

        return *this;
    }

    inline void store(limb_t *p) const
    {   vec_copy(p, val, sizeof(val));   }

    inline blst_256_t& operator+=(const blst_256_t& b)
    {   add_mod_256(val, val, b, MOD);              return *this;   }
    friend inline blst_256_t operator+(const blst_256_t& a, const blst_256_t& b)
    {
        blst_256_t ret;
        add_mod_256(ret, a, b, MOD);
        return ret;
    }

    inline blst_256_t& operator<<=(unsigned l)
    {   lshift_mod_256(val, val, l, MOD);           return *this;   }
    friend inline blst_256_t operator<<(const blst_256_t& a, unsigned l)
    {
        blst_256_t ret;
        lshift_mod_256(ret, a, l, MOD);
        return ret;
    }

    inline blst_256_t& operator>>=(unsigned r)
    {   lshift_mod_256(val, val, r, MOD);           return *this;   }
    friend inline blst_256_t operator>>(blst_256_t a, unsigned r)
    {
        blst_256_t ret;
        lshift_mod_256(ret, a, r, MOD);
        return ret;
    }

    inline blst_256_t& operator-=(const blst_256_t& b)
    {   sub_mod_256(val, val, b, MOD);              return *this;   }
    friend inline blst_256_t operator-(const blst_256_t& a, const blst_256_t& b)
    {
        blst_256_t ret;
        sub_mod_256(ret, a, b, MOD);
        return ret;
    }

    inline blst_256_t& cneg(bool flag)
    {   cneg_mod_256(val, val, flag, MOD);          return *this;   }
    friend inline blst_256_t cneg(const blst_256_t& a, bool flag)
    {
        blst_256_t ret;
        cneg_mod_256(ret, a, flag, MOD);
        return ret;
    }
    friend inline blst_256_t operator-(const blst_256_t& a)
    {
        blst_256_t ret;
        cneg_mod_256(ret, a, true, MOD);
        return ret;
    }

    inline blst_256_t& operator*=(const blst_256_t& a)
    {
        if (this == &a) sqr_mont_sparse_256(val, val, MOD, M0);
        else            mul_mont_sparse_256(val, val, a, MOD, M0);
        return *this;
    }
    friend inline blst_256_t operator*(const blst_256_t& a, const blst_256_t& b)
    {
        blst_256_t ret;
        if (&a == &b)   sqr_mont_sparse_256(ret, a, MOD, M0);
        else            mul_mont_sparse_256(ret, a, b, MOD, M0);
        return ret;
    }

    // simplified exponentiation, but mind the ^ operator's precedence!
    friend inline blst_256_t operator^(const blst_256_t& a, unsigned p)
    {
        if (p < 2) {
            abort();
        } else if (p == 2) {
            blst_256_t ret;
            sqr_mont_sparse_256(ret, a, MOD, M0);
            return ret;
        } else {
            blst_256_t ret = a, sqr = a;
            if ((p&1) == 0) {
                do {
                    sqr_mont_sparse_256(sqr, sqr, MOD, M0);
                    p >>= 1;
                } while ((p&1) == 0);
                ret = sqr;
            }
            for (p >>= 1; p; p >>= 1) {
                sqr_mont_sparse_256(sqr, sqr, MOD, M0);
                if (p&1)
                    mul_mont_sparse_256(ret, ret, sqr, MOD, M0);
            }
            return ret;
        }
    }
    inline blst_256_t& operator^=(unsigned p)
    {
        if (p < 2) {
            abort();
        } else if (p == 2) {
            sqr_mont_sparse_256(val, val, MOD, M0);
            return *this;
        }
        return *this = *this^p;
    }
    inline blst_256_t operator()(unsigned p)
    {   return *this^p;   }
    friend inline blst_256_t sqr(const blst_256_t& a)
    {   return a^2;   }

    inline bool is_one() const
    {   return vec_is_equal(val, ONE, sizeof(val));   }

    inline int is_zero() const
    {   return vec_is_zero(val, sizeof(val));   }

    inline void zero()
    {   vec_zero(val, sizeof(val));   }

    friend inline blst_256_t czero(const blst_256_t& a, int set_z)
    {   blst_256_t ret;
        const vec256 zero = { 0 };
        vec_select(ret, zero, a, sizeof(ret), set_z);
        return ret;
    }

    static inline blst_256_t csel(const blst_256_t& a, const blst_256_t& b,
                                  int sel_a)
    {   blst_256_t ret;
        vec_select(ret, a, b, sizeof(ret), sel_a);
        return ret;
    }

    blst_256_t reciprocal() const
    {
        static const blst_256_t MODx{MOD, true};
        union { vec512 x; vec256 r[2]; } temp;

        ct_inverse_mod_256(temp.x, val, MOD, MODx);
        redc_mont_256(temp.r[0], temp.x, MOD, M0);
        mul_mont_sparse_256(temp.r[0], temp.r[0], RR, MOD, M0);

        return *reinterpret_cast<blst_256_t*>(temp.r[0]);
    }
    friend inline blst_256_t operator/(int one, const blst_256_t& a)
    {
        if (one == 1)
            return a.reciprocal();
        abort();
    }
    friend inline blst_256_t operator/(const blst_256_t& a, const blst_256_t& b)
    {   return a * b.reciprocal();   }
    inline blst_256_t& operator/=(const blst_256_t& a)
    {   return *this *= a.reciprocal();   }

#ifndef NDEBUG
    inline blst_256_t(const char *hexascii)
    {   limbs_from_hexascii(val, sizeof(val), hexascii); to();   }

    friend inline bool operator==(const blst_256_t& a, const blst_256_t& b)
    {   return vec_is_equal(a, b, sizeof(vec256));   }
    friend inline bool operator!=(const blst_256_t& a, const blst_256_t& b)
    {   return !vec_is_equal(a, b, sizeof(vec256));   }

# if defined(_GLIBCXX_IOSTREAM) || defined(_IOSTREAM_) // non-standard
    friend std::ostream& operator<<(std::ostream& os, const blst_256_t& obj)
    {
        unsigned char be[sizeof(obj)];
        char buf[2+2*sizeof(obj)+1], *str=buf;

        be_bytes_from_limbs(be, blst_256_t{obj}.from(), sizeof(obj));

        *str++ = '0', *str++ = 'x';
        for (size_t i = 0; i < sizeof(obj); i++)
            *str++ = hex_from_nibble(be[i]>>4), *str++ = hex_from_nibble(be[i]);
	*str = '\0';

        return os << buf;
    }
# endif
#endif
};
#endif
