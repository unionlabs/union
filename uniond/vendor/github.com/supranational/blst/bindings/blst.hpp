/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */
#ifndef __BLST_HPP__
#define __BLST_HPP__

#include <string>
#include <cstring>
#include <vector>
#include <memory>

#if __cplusplus >= 201703L
# include <string_view>
# ifndef app__string_view
#  define app__string_view std::string_view // std::basic_string_view<byte>
# endif
#endif

namespace blst {

#if __cplusplus >= 201703L
static const app__string_view None;
#endif

#if __cplusplus < 201103L && !defined(nullptr)
# ifdef __GNUG__
#  define nullptr __null
# elif !defined(_MSVC_LANG) || _MSVC_LANG < 201103L
#  define nullptr 0
# endif
#endif

#ifdef __clang__
# pragma GCC diagnostic push
# pragma GCC diagnostic ignored "-Wextern-c-compat"
#endif

#include "blst.h"

#ifdef __clang__
# pragma GCC diagnostic pop
#endif

class P1_Affine;
class P1;
class P2_Affine;
class P2;
class Pairing;

inline const byte *C_bytes(const void *ptr)
{   return static_cast<const byte*>(ptr);   }

/*
 * As for SecretKey being struct and not class, and lack of constructors
 * with one accepting for example |IKM|. We can't make assumptions about
 * application's policy toward handling secret key material. Hence it's
 * argued that application is entitled for transparent structure, not
 * opaque or semi-opaque class. And in the context it's appropriate not
 * to "entice" developers with idiomatic constructors:-) Though this
 * doesn't really apply to SWIG-assisted interfaces...
 */
struct SecretKey {
#ifdef SWIG
private:
#endif
    blst_scalar key;

#ifdef SWIG
public:
#endif
    void keygen(const byte* IKM, size_t IKM_len,
                const std::string& info = "")
    {   blst_keygen(&key, IKM, IKM_len, C_bytes(info.data()), info.size());   }
    void keygen_v3(const byte* IKM, size_t IKM_len,
                   const std::string& info = "")
    {   blst_keygen_v3(&key, IKM, IKM_len, C_bytes(info.data()), info.size());   }
    void keygen_v4_5(const byte* IKM, size_t IKM_len,
                     const byte* salt, size_t salt_len,
                     const std::string& info = "")
    {   blst_keygen_v4_5(&key, IKM, IKM_len, salt, salt_len,
                               C_bytes(info.data()), info.size());
    }
    void keygen_v5(const byte* IKM, size_t IKM_len,
                   const byte* salt, size_t salt_len,
                   const std::string& info = "")
    {   blst_keygen_v5(&key, IKM, IKM_len, salt, salt_len,
                             C_bytes(info.data()), info.size());
    }
#if __cplusplus >= 201703L
    void keygen(const app__string_view IKM, // string_view by value, cool!
                const std::string& info = "")
    {   keygen(C_bytes(IKM.data()), IKM.size(), info);   }
    void keygen_v3(const app__string_view IKM, // string_view by value, cool!
                   const std::string& info = "")
    {   keygen_v3(C_bytes(IKM.data()), IKM.size(), info);   }
    void keygen_v4_5(const app__string_view IKM, // string_view by value, cool!
                     const app__string_view salt,
                     const std::string& info = "")
    {   keygen_v4_5(C_bytes(IKM.data()), IKM.size(),
                    C_bytes(salt.data()), salt.size(), info);
    }
    void keygen_v5(const app__string_view IKM, // string_view by value, cool!
                   const app__string_view salt,
                   const std::string& info = "")
    {   keygen_v5(C_bytes(IKM.data()), IKM.size(),
                  C_bytes(salt.data()), salt.size(), info);
    }
#endif
    void derive_master_eip2333(const byte* IKM, size_t IKM_len)
    {   blst_derive_master_eip2333(&key, IKM, IKM_len);   }
    void derive_child_eip2333(const SecretKey& SK, unsigned int child_index)
    {   blst_derive_child_eip2333(&key, &SK.key, child_index);   }

    void from_bendian(const byte in[32]) { blst_scalar_from_bendian(&key, in); }
    void from_lendian(const byte in[32]) { blst_scalar_from_lendian(&key, in); }

    void to_bendian(byte out[32]) const
    {   blst_bendian_from_scalar(out, &key);   }
    void to_lendian(byte out[32]) const
    {   blst_lendian_from_scalar(out, &key);   }
};

class Scalar {
private:
    blst_scalar val;

public:
    Scalar() { memset(&val, 0, sizeof(val)); }
    Scalar(const byte* scalar, size_t nbits)
    {   blst_scalar_from_le_bytes(&val, scalar, (nbits+7)/8);   }
    Scalar(const byte *msg, size_t msg_len, const std::string& DST)
    {   (void)hash_to(msg, msg_len, DST);   }
#if __cplusplus >= 201703L
    Scalar(const app__string_view msg, const std::string& DST = "")
    {   (void)hash_to(C_bytes(msg.data()), msg.size(), DST);   }
#endif

    Scalar* hash_to(const byte *msg, size_t msg_len, const std::string& DST = "")
    {   byte elem[48];
        blst_expand_message_xmd(elem, sizeof(elem), msg, msg_len,
                                                    C_bytes(DST.data()), DST.size());
        blst_scalar_from_be_bytes(&val, elem, sizeof(elem));
        return this;
    }
#if __cplusplus >= 201703L
    Scalar* hash_to(const app__string_view msg, const std::string& DST = "")
    {   return hash_to(C_bytes(msg.data()), msg.size(), DST);   }
#endif

    Scalar dup() const { return *this; }
    Scalar* from_bendian(const byte *msg, size_t msg_len)
    {   blst_scalar_from_be_bytes(&val, msg, msg_len); return this;   }
    Scalar* from_lendian(const byte *msg, size_t msg_len)
    {   blst_scalar_from_le_bytes(&val, msg, msg_len); return this;   }
    void to_bendian(byte out[32]) const
    {   blst_bendian_from_scalar(out, &val);   }
    void to_lendian(byte out[32]) const
    {   blst_lendian_from_scalar(out, &val);   }

    Scalar* add(const Scalar& a)
    {   if (!blst_sk_add_n_check(&val, &val, a))
            throw BLST_BAD_SCALAR;
        return this;
    }
    Scalar* add(const SecretKey& a)
    {   if (!blst_sk_add_n_check(&val, &val, &a.key))
            throw BLST_BAD_SCALAR;
        return this;
    }
    Scalar* sub(const Scalar& a)
    {   if (!blst_sk_sub_n_check(&val, &val, a))
            throw BLST_BAD_SCALAR;
        return this;
    }
    Scalar* mul(const Scalar& a)
    {   if (!blst_sk_mul_n_check(&val, &val, a))
            throw BLST_BAD_SCALAR;
        return this;
    }
    Scalar* inverse()
    {   blst_sk_inverse(&val, &val); return this;   }

private:
    friend class P1;
    friend class P2;
    operator const blst_scalar*() const { return &val; }
    operator const byte*() const        { return val.b; }
};

class P1_Affine {
private:
    blst_p1_affine point;

    P1_Affine(const blst_p1_affine *cptr) { point = *cptr; }
public:
    P1_Affine() { memset(&point, 0, sizeof(point)); }
#ifndef SWIG
    P1_Affine(const byte *in)
    {   BLST_ERROR err = blst_p1_deserialize(&point, in);
        if (err != BLST_SUCCESS)
            throw err;
    }
#endif
    P1_Affine(const byte *in, size_t len)
    {   if (len == 0 || len != (in[0]&0x80 ? 48 : 96))
            throw BLST_BAD_ENCODING;
        BLST_ERROR err = blst_p1_deserialize(&point, in);
        if (err != BLST_SUCCESS)
            throw err;
    }
    P1_Affine(const P1& jacobian);

    P1_Affine dup() const { return *this; }
    P1 to_jacobian() const;
    void serialize(byte out[96]) const
    {   blst_p1_affine_serialize(out, &point);   }
    void compress(byte out[48]) const
    {   blst_p1_affine_compress(out, &point);   }
    bool on_curve() const { return blst_p1_affine_on_curve(&point); }
    bool in_group() const { return blst_p1_affine_in_g1(&point);    }
    bool is_inf() const   { return blst_p1_affine_is_inf(&point);   }
    bool is_equal(const P1_Affine& p) const
    {   return blst_p1_affine_is_equal(&point, &p.point);   }
    BLST_ERROR core_verify(const P2_Affine& pk, bool hash_or_encode,
                           const byte* msg, size_t msg_len,
                           const std::string& DST = "",
                           const byte* aug = nullptr, size_t aug_len = 0) const;
#if __cplusplus >= 201703L
    BLST_ERROR core_verify(const P2_Affine& pk, bool hash_or_encode,
                           const app__string_view msg,
                           const std::string& DST = "",
                           const app__string_view aug = None) const;
#endif
    static P1_Affine generator()
    {   return P1_Affine(blst_p1_affine_generator());   }

private:
    friend class Pairing;
    friend class P2_Affine;
    friend class PT;
    friend class P1;
    friend class P1_Affines;
    operator const blst_p1_affine*() const { return &point; }
    operator blst_p1_affine*()             { return &point; }
};

class P1 {
private:
    blst_p1 point;

    P1(const blst_p1 *cptr) { point = *cptr; }
public:
    P1() { memset(&point, 0, sizeof(point)); }
    P1(const SecretKey& sk) { blst_sk_to_pk_in_g1(&point, &sk.key); }
#ifndef SWIG
    P1(const byte *in)
    {   blst_p1_affine a;
        BLST_ERROR err = blst_p1_deserialize(&a, in);
        if (err != BLST_SUCCESS)
            throw err;
        blst_p1_from_affine(&point, &a);
    }
#endif
    P1(const byte *in, size_t len)
    {   if (len == 0 || len != (in[0]&0x80 ? 48 : 96))
            throw BLST_BAD_ENCODING;
        blst_p1_affine a;
        BLST_ERROR err = blst_p1_deserialize(&a, in);
        if (err != BLST_SUCCESS)
            throw err;
        blst_p1_from_affine(&point, &a);
    }
    P1(const P1_Affine& affine) { blst_p1_from_affine(&point, affine); }

    P1 dup() const                      { return *this; }
    P1_Affine to_affine() const         { return P1_Affine(*this);           }
    void serialize(byte out[96]) const  { blst_p1_serialize(out, &point);    }
    void compress(byte out[48]) const   { blst_p1_compress(out, &point);     }
    bool on_curve() const               { return blst_p1_on_curve(&point);   }
    bool in_group() const               { return blst_p1_in_g1(&point);      }
    bool is_inf() const                 { return blst_p1_is_inf(&point);     }
    bool is_equal(const P1& p) const
    {   return blst_p1_is_equal(&point, &p.point);   }
    void aggregate(const P1_Affine& in)
    {   if (blst_p1_affine_in_g1(in))
            blst_p1_add_or_double_affine(&point, &point, in);
        else
            throw BLST_POINT_NOT_IN_GROUP;
    }
    P1* sign_with(const SecretKey& sk)
    {   blst_sign_pk_in_g2(&point, &point, &sk.key); return this;   }
    P1* sign_with(const Scalar& scalar)
    {   blst_sign_pk_in_g2(&point, &point, scalar); return this;   }
    P1* hash_to(const byte* msg, size_t msg_len,
                const std::string& DST = "",
                const byte* aug = nullptr, size_t aug_len = 0)
    {   blst_hash_to_g1(&point, msg, msg_len, C_bytes(DST.data()), DST.size(),
                                aug, aug_len);
        return this;
    }
    P1* encode_to(const byte* msg, size_t msg_len,
                  const std::string& DST = "",
                  const byte* aug = nullptr, size_t aug_len = 0)
    {   blst_encode_to_g1(&point, msg, msg_len, C_bytes(DST.data()), DST.size(),
                                  aug, aug_len);
        return this;
    }
#if __cplusplus >= 201703L
    P1* hash_to(const app__string_view msg, const std::string& DST = "",
                const app__string_view aug = None)
    {   return hash_to(C_bytes(msg.data()), msg.size(), DST,
                       C_bytes(aug.data()), aug.size());
    }
    P1* encode_to(const app__string_view msg, const std::string& DST = "",
                  const app__string_view aug = None)
    {   return encode_to(C_bytes(msg.data()), msg.size(), DST,
                         C_bytes(aug.data()), aug.size());
    }
#endif
    P1* mult(const byte* scalar, size_t nbits)
    {   blst_p1_mult(&point, &point, scalar, nbits); return this;   }
    P1* mult(const Scalar& scalar)
    {   blst_p1_mult(&point, &point, scalar, 255); return this;   }
    P1* cneg(bool flag)
    {   blst_p1_cneg(&point, flag); return this;   }
    P1* neg()
    {   blst_p1_cneg(&point, true); return this;   }
    P1* add(const P1& a)
    {   blst_p1_add_or_double(&point, &point, a); return this;   }
    P1* add(const P1_Affine &a)
    {   blst_p1_add_or_double_affine(&point, &point, a); return this;   }
    P1* dbl()
    {   blst_p1_double(&point, &point); return this;   }
#ifndef SWIG
    static P1 add(const P1& a, const P1& b)
    {   P1 ret; blst_p1_add_or_double(ret, a, b); return ret;   }
    static P1 add(const P1& a, const P1_Affine& b)
    {   P1 ret; blst_p1_add_or_double_affine(ret, a, b); return ret;   }
    static P1 dbl(const P1& a)
    {   P1 ret; blst_p1_double(ret, a); return ret;   }
#endif
    static P1 generator()
    {   return P1(blst_p1_generator());   }

private:
    friend class P1_Affine;
    friend class P1_Affines;
    operator const blst_p1*() const { return &point; }
    operator blst_p1*()             { return &point; }
};

class P1_Affines {
private:
    struct p1_affine_no_init {
        blst_p1_affine point;
        p1_affine_no_init() { }
        operator blst_p1_affine*()              { return &point; }
        operator const blst_p1_affine*() const  { return &point; }
    };

    std::vector<p1_affine_no_init> table;
    size_t wbits, npoints;

public:
#ifndef SWIG
    P1_Affines() {}
    P1_Affines(size_t wbits, const P1_Affine* const points[], size_t npoints)
    {   this->wbits = wbits;
        this->npoints = npoints;
        table.resize(npoints << (wbits-1));
        blst_p1s_mult_wbits_precompute(table[0], wbits,
                        reinterpret_cast<const blst_p1_affine *const*>(points),
                        npoints);
    }
    P1_Affines(size_t wbits, const P1_Affine points[], size_t npoints)
    {   const P1_Affine* const ptrs[2] = { points, nullptr };
        P1_Affines(wbits, ptrs, npoints);
    }
    P1_Affines(size_t wbits, const std::vector<P1_Affine> points)
    {   P1_Affines(wbits, &points[0], points.size());   }

    P1_Affines(size_t wbits, const P1* const points[], size_t npoints)
    {   size_t cap = npoints << (wbits-1);

        this->wbits = wbits;
        this->npoints = npoints;
        table.resize(cap);
        blst_p1s_to_affine(table[cap-npoints],
                           reinterpret_cast<const blst_p1 *const*>(points),
                           npoints);
        const blst_p1_affine* const ptrs[2] = { table[cap-npoints], nullptr };
        blst_p1s_mult_wbits_precompute(table[0], wbits, ptrs, npoints);
    }
    P1_Affines(size_t wbits, const P1 points[], size_t npoints)
    {   const P1* const ptrs[2] = { points, nullptr };
        P1_Affines(wbits, ptrs, npoints);
    }
    P1_Affines(size_t wbits, const std::vector<P1> points)
    {   P1_Affines(wbits, &points[0], points.size());   }

    P1_Affines(const P1* const points[], size_t npoints)
    {   this->wbits = 0;
        this->npoints = npoints;
        table.resize(npoints);
        blst_p1s_to_affine(table[0],
                           reinterpret_cast<const blst_p1 *const*>(points),
                           npoints);
    }
    P1_Affines(const P1 points[], size_t npoints)
    {   const P1* const ptrs[2] = { points, nullptr };
        P1_Affines(ptrs, npoints);
    }
    P1_Affines(const std::vector<P1> points)
    {   P1_Affines(&points[0], points.size());   }

    P1 mult(const byte* const scalars[], size_t nbits) const
    {   P1 ret;

        if (wbits != 0) {
            size_t sz = blst_p1s_mult_wbits_scratch_sizeof(npoints);
            std::unique_ptr<limb_t[]> scratch{new limb_t[sz/sizeof(limb_t)]};
            blst_p1s_mult_wbits(ret, table[0], wbits, npoints,
                                     scalars, nbits, scratch.get());
        } else {
            size_t sz = blst_p1s_mult_pippenger_scratch_sizeof(npoints);
            std::unique_ptr<limb_t[]> scratch{new limb_t[sz/sizeof(limb_t)]};
            const blst_p1_affine* const ptrs[2] = { table[0], nullptr };
            blst_p1s_mult_pippenger(ret, ptrs, npoints,
                                         scalars, nbits, scratch.get());
        }
        return ret;
    }

    static std::vector<P1_Affine> from(const P1* const points[], size_t npoints)
    {   std::vector<P1_Affine> ret;
        ret.resize(npoints);
        blst_p1s_to_affine(ret[0],
                           reinterpret_cast<const blst_p1 *const*>(points),
                           npoints);
        return ret;
    }
    static std::vector<P1_Affine> from(const P1 points[], size_t npoints)
    {   const P1* const ptrs[2] = { points, nullptr };
        return from(ptrs, npoints);
    }
    static std::vector<P1_Affine> from(std::vector<P1> points)
    {   return from(&points[0], points.size());   }
#endif

    static P1 mult_pippenger(const P1_Affine* const points[], size_t npoints,
                             const byte* const scalars[], size_t nbits)
    {   P1 ret;
        size_t sz = blst_p1s_mult_pippenger_scratch_sizeof(npoints);
        std::unique_ptr<limb_t[]> scratch{new limb_t[sz/sizeof(limb_t)]};
        blst_p1s_mult_pippenger(ret,
                    reinterpret_cast<const blst_p1_affine *const*>(points),
                    npoints, scalars, nbits, scratch.get());
        return ret;
    }
#ifndef SWIG
    static P1 mult_pippenger(const P1_Affine points[], size_t npoints,
                             const byte* const scalars[], size_t nbits)
    {   const P1_Affine* const ptrs[2] = { points, nullptr };
        return mult_pippenger(ptrs, npoints, scalars, nbits);
    }
    static P1 mult_pippenger(const std::vector<P1_Affine> points,
                             const byte* const scalars[], size_t nbits)
    {   return mult_pippenger(&points[0], points.size(), scalars, nbits);   }
#endif

    static P1 add(const P1_Affine* const points[], size_t npoints)
    {   P1 ret;
        blst_p1s_add(ret,
                     reinterpret_cast<const blst_p1_affine *const*>(points),
                     npoints);
        return ret;
    }
#ifndef SWIG
    static P1 add(const P1_Affine points[], size_t npoints)
    {   const P1_Affine* const ptrs[2] = { points, nullptr };
        return add(ptrs, npoints);
    }
    static P1 add(const std::vector<P1_Affine> points)
    {   return add(&points[0], points.size());   }
#endif
};

class P2_Affine {
private:
    blst_p2_affine point;

    P2_Affine(const blst_p2_affine *cptr) { point = *cptr; }
public:
    P2_Affine() { memset(&point, 0, sizeof(point)); }
#ifndef SWIG
    P2_Affine(const byte *in)
    {   BLST_ERROR err = blst_p2_deserialize(&point, in);
        if (err != BLST_SUCCESS)
            throw err;
    }
#endif
    P2_Affine(const byte *in, size_t len)
    {   if (len == 0 || len != (in[0]&0x80 ? 96 : 192))
            throw BLST_BAD_ENCODING;
        BLST_ERROR err = blst_p2_deserialize(&point, in);
        if (err != BLST_SUCCESS)
            throw err;
    }
    P2_Affine(const P2& jacobian);

    P2_Affine dup() const { return *this; }
    P2 to_jacobian() const;
    void serialize(byte out[192]) const
    {   blst_p2_affine_serialize(out, &point);   }
    void compress(byte out[96]) const
    {   blst_p2_affine_compress(out, &point);   }
    bool on_curve() const { return blst_p2_affine_on_curve(&point); }
    bool in_group() const { return blst_p2_affine_in_g2(&point);    }
    bool is_inf() const   { return blst_p2_affine_is_inf(&point);   }
    bool is_equal(const P2_Affine& p) const
    {   return blst_p2_affine_is_equal(&point, &p.point);   }
    BLST_ERROR core_verify(const P1_Affine& pk, bool hash_or_encode,
                           const byte* msg, size_t msg_len,
                           const std::string& DST = "",
                           const byte* aug = nullptr, size_t aug_len = 0) const;
#if __cplusplus >= 201703L
    BLST_ERROR core_verify(const P1_Affine& pk, bool hash_or_encode,
                           const app__string_view msg,
                           const std::string& DST = "",
                           const app__string_view aug = None) const;
#endif
    static P2_Affine generator()
    {   return P2_Affine(blst_p2_affine_generator());   }

private:
    friend class Pairing;
    friend class P1_Affine;
    friend class PT;
    friend class P2;
    friend class P2_Affines;
    operator const blst_p2_affine*() const { return &point; }
    operator blst_p2_affine*()             { return &point; }
};

class P2 {
private:
    blst_p2 point;

    P2(const blst_p2 *cptr) { point = *cptr; }
public:
    P2() { memset(&point, 0, sizeof(point)); }
    P2(const SecretKey& sk) { blst_sk_to_pk_in_g2(&point, &sk.key); }
#ifndef SWIG
    P2(const byte *in)
    {   blst_p2_affine a;
        BLST_ERROR err = blst_p2_deserialize(&a, in);
        if (err != BLST_SUCCESS)
            throw err;
        blst_p2_from_affine(&point, &a);
    }
#endif
    P2(const byte *in, size_t len)
    {   if (len == 0 || len != (in[0]&0x80 ? 96 : 192))
            throw BLST_BAD_ENCODING;
        blst_p2_affine a;
        BLST_ERROR err = blst_p2_deserialize(&a, in);
        if (err != BLST_SUCCESS)
            throw err;
        blst_p2_from_affine(&point, &a);
    }
    P2(const P2_Affine& affine) { blst_p2_from_affine(&point, affine); }

    P2 dup() const                      { return *this; }
    P2_Affine to_affine() const         { return P2_Affine(*this);          }
    void serialize(byte out[192]) const { blst_p2_serialize(out, &point);   }
    void compress(byte out[96]) const   { blst_p2_compress(out, &point);    }
    bool on_curve() const               { return blst_p2_on_curve(&point);  }
    bool in_group() const               { return blst_p2_in_g2(&point);     }
    bool is_inf() const                 { return blst_p2_is_inf(&point);    }
    bool is_equal(const P2& p) const
    {   return blst_p2_is_equal(&point, &p.point);   }
    void aggregate(const P2_Affine& in)
    {   if (blst_p2_affine_in_g2(in))
            blst_p2_add_or_double_affine(&point, &point, in);
        else
            throw BLST_POINT_NOT_IN_GROUP;
    }
    P2* sign_with(const SecretKey& sk)
    {   blst_sign_pk_in_g1(&point, &point, &sk.key); return this;   }
    P2* sign_with(const Scalar& scalar)
    {   blst_sign_pk_in_g1(&point, &point, scalar); return this;   }
    P2* hash_to(const byte* msg, size_t msg_len,
                const std::string& DST = "",
                const byte* aug = nullptr, size_t aug_len = 0)
    {   blst_hash_to_g2(&point, msg, msg_len, C_bytes(DST.data()), DST.size(),
                                aug, aug_len);
        return this;
    }
    P2* encode_to(const byte* msg, size_t msg_len,
                  const std::string& DST = "",
                  const byte* aug = nullptr, size_t aug_len = 0)
    {   blst_encode_to_g2(&point, msg, msg_len, C_bytes(DST.data()), DST.size(),
                                  aug, aug_len);
        return this;
    }
#if __cplusplus >= 201703L
    P2* hash_to(const app__string_view msg, const std::string& DST = "",
                const app__string_view aug = None)
    {   return hash_to(C_bytes(msg.data()), msg.size(), DST,
                       C_bytes(aug.data()), aug.size());
    }
    P2* encode_to(const app__string_view msg, const std::string& DST = "",
                  const app__string_view aug = None)
    {   return encode_to(C_bytes(msg.data()), msg.size(), DST,
                         C_bytes(aug.data()), aug.size());
    }
#endif
    P2* mult(const byte* scalar, size_t nbits)
    {   blst_p2_mult(&point, &point, scalar, nbits); return this;   }
    P2* mult(const Scalar& scalar)
    {   blst_p2_mult(&point, &point, scalar, 255); return this;   }
    P2* cneg(bool flag)
    {   blst_p2_cneg(&point, flag); return this;   }
    P2* neg()
    {   blst_p2_cneg(&point, true); return this;   }
    P2* add(const P2& a)
    {   blst_p2_add_or_double(&point, &point, a); return this;   }
    P2* add(const P2_Affine &a)
    {   blst_p2_add_or_double_affine(&point, &point, a); return this;   }
    P2* dbl()
    {   blst_p2_double(&point, &point); return this;   }
#ifndef SWIG
    static P2 add(const P2& a, const P2& b)
    {   P2 ret; blst_p2_add_or_double(ret, a, b); return ret;   }
    static P2 add(const P2& a, const P2_Affine& b)
    {   P2 ret; blst_p2_add_or_double_affine(ret, a, b); return ret;   }
    static P2 dbl(const P2& a)
    {   P2 ret; blst_p2_double(ret, a); return ret;   }
#endif
    static P2 generator()
    {   return P2(blst_p2_generator());   }

private:
    friend class P2_Affine;
    friend class P2_Affines;
    operator const blst_p2*() const { return &point; }
    operator blst_p2*()             { return &point; }
};

class P2_Affines {
private:
    struct p2_affine_no_init {
        blst_p2_affine point;
        p2_affine_no_init() { }
        operator blst_p2_affine*()              { return &point; }
        operator const blst_p2_affine*() const  { return &point; }
    };

    std::vector<p2_affine_no_init> table;
    size_t wbits, npoints;

public:
#ifndef SWIG
    P2_Affines() {}
    P2_Affines(size_t wbits, const P2_Affine* const points[], size_t npoints)
    {   this->wbits = wbits;
        this->npoints = npoints;
        table.resize(npoints << (wbits-1));
        blst_p2s_mult_wbits_precompute(table[0], wbits,
                        reinterpret_cast<const blst_p2_affine *const*>(points),
                        npoints);
    }
    P2_Affines(size_t wbits, const P2_Affine points[], size_t npoints)
    {   const P2_Affine* const ptrs[2] = { points, nullptr };
        P2_Affines(wbits, ptrs, npoints);
    }
    P2_Affines(size_t wbits, const std::vector<P2_Affine> points)
    {   P2_Affines(wbits, &points[0], points.size());   }

    P2_Affines(size_t wbits, const P2* const points[], size_t npoints)
    {   size_t cap = npoints << (wbits-1);

        this->wbits = wbits;
        this->npoints = npoints;
        table.resize(cap);
        blst_p2s_to_affine(table[cap-npoints],
                           reinterpret_cast<const blst_p2 *const*>(points),
                           npoints);
        const blst_p2_affine* const ptrs[2] = { table[cap-npoints], nullptr };
        blst_p2s_mult_wbits_precompute(table[0], wbits, ptrs, npoints);
    }
    P2_Affines(size_t wbits, const P2 points[], size_t npoints)
    {   const P2* const ptrs[2] = { points, nullptr };
        P2_Affines(wbits, ptrs, npoints);
    }
    P2_Affines(size_t wbits, const std::vector<P2> points)
    {   P2_Affines(wbits, &points[0], points.size());   }

    P2_Affines(const P2* const points[], size_t npoints)
    {   this->wbits = 0;
        this->npoints = npoints;
        table.resize(npoints);
        blst_p2s_to_affine(table[0],
                           reinterpret_cast<const blst_p2 *const*>(points),
                           npoints);
    }
    P2_Affines(const P2 points[], size_t npoints)
    {   const P2* const ptrs[2] = { points, nullptr };
        P2_Affines(ptrs, npoints);
    }
    P2_Affines(const std::vector<P2> points)
    {   P2_Affines(&points[0], points.size());   }

    P2 mult(const byte* const scalars[], size_t nbits) const
    {   P2 ret;

        if (wbits != 0) {
            size_t sz = blst_p2s_mult_wbits_scratch_sizeof(npoints);
            std::unique_ptr<limb_t[]> scratch{new limb_t[sz/sizeof(limb_t)]};
            blst_p2s_mult_wbits(ret, table[0], wbits, npoints,
                                     scalars, nbits, scratch.get());
        } else {
            size_t sz = blst_p2s_mult_pippenger_scratch_sizeof(npoints);
            std::unique_ptr<limb_t[]> scratch{new limb_t[sz/sizeof(limb_t)]};
            const blst_p2_affine* const ptrs[2] = { table[0], nullptr };
            blst_p2s_mult_pippenger(ret, ptrs, npoints,
                                         scalars, nbits, scratch.get());
        }
        return ret;
    }

    static std::vector<P2_Affine> from(const P2* const points[], size_t npoints)
    {   std::vector<P2_Affine> ret;
        ret.resize(npoints);
        blst_p2s_to_affine(ret[0],
                           reinterpret_cast<const blst_p2 *const*>(points),
                           npoints);
        return ret;
    }
    static std::vector<P2_Affine> from(const P2 points[], size_t npoints)
    {   const P2* const ptrs[2] = { points, nullptr };
        return from(ptrs, npoints);
    }
    static std::vector<P2_Affine> from(std::vector<P2> points)
    {   return from(&points[0], points.size());   }
#endif

    static P2 mult_pippenger(const P2_Affine* const points[], size_t npoints,
                             const byte* const scalars[], size_t nbits)
    {   P2 ret;
        size_t sz = blst_p2s_mult_pippenger_scratch_sizeof(npoints);
        std::unique_ptr<limb_t[]> scratch{new limb_t[sz/sizeof(limb_t)]};
        blst_p2s_mult_pippenger(ret,
                    reinterpret_cast<const blst_p2_affine *const*>(points),
                    npoints, scalars, nbits, scratch.get());
        return ret;
    }
#ifndef SWIG
    static P2 mult_pippenger(const P2_Affine points[], size_t npoints,
                             const byte* const scalars[], size_t nbits)
    {   const P2_Affine* const ptrs[2] = { points, nullptr };
        return mult_pippenger(ptrs, npoints, scalars, nbits);
    }
    static P2 mult_pippenger(const std::vector<P2_Affine> points,
                             const byte* const scalars[], size_t nbits)
    {   return mult_pippenger(&points[0], points.size(), scalars, nbits);   }
#endif

    static P2 add(const P2_Affine* const points[], size_t npoints)
    {   P2 ret;
        blst_p2s_add(ret,
                     reinterpret_cast<const blst_p2_affine *const*>(points),
                     npoints);
        return ret;
    }
#ifndef SWIG
    static P2 add(const P2_Affine points[], size_t npoints)
    {   const P2_Affine* const ptrs[2] = { points, nullptr };
        return add(ptrs, npoints);
    }
    static P2 add(const std::vector<P2_Affine> points)
    {   return add(&points[0], points.size());   }
#endif
};

inline P1_Affine::P1_Affine(const P1& jacobian)
{   blst_p1_to_affine(&point, jacobian);   }
inline P2_Affine::P2_Affine(const P2& jacobian)
{   blst_p2_to_affine(&point, jacobian);   }

inline P1 P1_Affine::to_jacobian() const { P1 ret(*this); return ret; }
inline P2 P2_Affine::to_jacobian() const { P2 ret(*this); return ret; }

inline P1 G1() { return P1::generator();  }
inline P2 G2() { return P2::generator();  }

inline BLST_ERROR P1_Affine::core_verify(const P2_Affine& pk,
                                         bool hash_or_encode,
                                         const byte* msg, size_t msg_len,
                                         const std::string& DST,
                                         const byte* aug, size_t aug_len) const
{   return blst_core_verify_pk_in_g2(pk, &point, hash_or_encode,
                                         msg, msg_len,
                                         C_bytes(DST.data()), DST.size(),
                                         aug, aug_len);
}
inline BLST_ERROR P2_Affine::core_verify(const P1_Affine& pk,
                                         bool hash_or_encode,
                                         const byte* msg, size_t msg_len,
                                         const std::string& DST,
                                         const byte* aug, size_t aug_len) const
{   return blst_core_verify_pk_in_g1(pk, &point, hash_or_encode,
                                         msg, msg_len,
                                         C_bytes(DST.data()), DST.size(),
                                         aug, aug_len);
}
#if __cplusplus >= 201703L
inline BLST_ERROR P1_Affine::core_verify(const P2_Affine& pk,
                                         bool hash_or_encode,
                                         const app__string_view msg,
                                         const std::string& DST,
                                         const app__string_view aug) const
{   return core_verify(pk, hash_or_encode, C_bytes(msg.data()), msg.size(), DST,
                                           C_bytes(aug.data()), aug.size());
}
inline BLST_ERROR P2_Affine::core_verify(const P1_Affine& pk,
                                         bool hash_or_encode,
                                         const app__string_view msg,
                                         const std::string& DST,
                                         const app__string_view aug) const
{   return core_verify(pk, hash_or_encode, C_bytes(msg.data()), msg.size(), DST,
                                           C_bytes(aug.data()), aug.size());
}
#endif

class PT {
private:
    blst_fp12 value;

    PT(const blst_fp12 *v)  { value = *v; }
public:
    PT(const P1_Affine& p)  { blst_aggregated_in_g1(&value, p); }
    PT(const P2_Affine& q)  { blst_aggregated_in_g2(&value, q); }
    PT(const P2_Affine& q, const P1_Affine& p)
    {   blst_miller_loop(&value, q, p);   }
    PT(const P1_Affine& p, const P2_Affine& q) : PT(q, p) {}
    PT(const P2& q, const P1& p)
    {   blst_miller_loop(&value, P2_Affine(q), P1_Affine(p));   }
    PT(const P1& p, const P2& q) : PT(q, p) {}

    PT dup() const          { return *this; }
    bool is_one() const     { return blst_fp12_is_one(&value); }
    bool is_equal(const PT& p) const
    {   return blst_fp12_is_equal(&value, p);   }
    PT* sqr()               { blst_fp12_sqr(&value, &value);    return this; }
    PT* mul(const PT& p)    { blst_fp12_mul(&value, &value, p); return this; }
    PT* final_exp()         { blst_final_exp(&value, &value);   return this; }
    bool in_group() const   { return blst_fp12_in_group(&value); }
    void to_bendian(byte out[48*12]) const
    {   blst_bendian_from_fp12(out, &value);   }

    static bool finalverify(const PT& gt1, const PT& gt2)
    {   return blst_fp12_finalverify(gt1, gt2);   }
    static PT one() { return PT(blst_fp12_one()); }

private:
    friend class Pairing;
    operator const blst_fp12*() const { return &value; }
};

class Pairing {
private:
    operator blst_pairing*()
    {   return reinterpret_cast<blst_pairing *>(this);   }
    operator const blst_pairing*() const
    {   return reinterpret_cast<const blst_pairing *>(this);   }

    void init(bool hash_or_encode, const byte* DST, size_t DST_len)
    {   // Copy DST to heap, std::string can be volatile, especially in SWIG:-(
        byte *dst = new byte[DST_len];
        memcpy(dst, DST, DST_len);
        blst_pairing_init(*this, hash_or_encode, dst, DST_len);
    }

public:
#ifndef SWIG
    void* operator new(size_t)
    {   return new uint64_t[blst_pairing_sizeof()/sizeof(uint64_t)];   }
    void operator delete(void *ptr)
    {   delete[] static_cast<uint64_t*>(ptr);   }

    Pairing(bool hash_or_encode, const std::string& DST)
    {   init(hash_or_encode, C_bytes(DST.data()), DST.size());   }
#if __cplusplus >= 201703L
    Pairing(bool hash_or_encode, const app__string_view DST)
    {   init(hash_or_encode, C_bytes(DST.data()), DST.size());   }
#endif
#endif
#ifndef SWIGJAVA
    Pairing(bool hash_or_encode, const byte* DST, size_t DST_len)
    {   init(hash_or_encode, DST, DST_len);   }
    ~Pairing() { delete[] blst_pairing_get_dst(*this); }
#endif

    BLST_ERROR aggregate(const P1_Affine* pk, const P2_Affine* sig,
                         const byte* msg, size_t msg_len,
                         const byte* aug = nullptr, size_t aug_len = 0)
    {   return blst_pairing_aggregate_pk_in_g1(*this, *pk, *sig,
                         msg, msg_len, aug, aug_len);
    }
    BLST_ERROR aggregate(const P2_Affine* pk, const P1_Affine* sig,
                         const byte* msg, size_t msg_len,
                         const byte* aug = nullptr, size_t aug_len = 0)
    {   return blst_pairing_aggregate_pk_in_g2(*this, *pk, *sig,
                         msg, msg_len, aug, aug_len);
    }
    BLST_ERROR mul_n_aggregate(const P1_Affine* pk, const P2_Affine* sig,
                               const byte* scalar, size_t nbits,
                               const byte* msg, size_t msg_len,
                               const byte* aug = nullptr, size_t aug_len = 0)
    {   return blst_pairing_mul_n_aggregate_pk_in_g1(*this, *pk, *sig,
                               scalar, nbits, msg, msg_len, aug, aug_len);
    }
    BLST_ERROR mul_n_aggregate(const P2_Affine* pk, const P1_Affine* sig,
                               const byte* scalar, size_t nbits,
                               const byte* msg, size_t msg_len,
                               const byte* aug = nullptr, size_t aug_len = 0)
    {   return blst_pairing_mul_n_aggregate_pk_in_g2(*this, *pk, *sig,
                               scalar, nbits, msg, msg_len, aug, aug_len);
    }
#if __cplusplus >= 201703L
    BLST_ERROR aggregate(const P1_Affine* pk, const P2_Affine* sig,
                         const app__string_view msg,
                         const app__string_view aug = None)
    {   return aggregate(pk, sig, C_bytes(msg.data()), msg.size(),
                                  C_bytes(aug.data()), aug.size());
    }
    BLST_ERROR aggregate(const P2_Affine* pk, const P1_Affine* sig,
                         const app__string_view msg,
                         const app__string_view aug = None)
    {   return aggregate(pk, sig, C_bytes(msg.data()), msg.size(),
                                  C_bytes(aug.data()), aug.size());
    }
    BLST_ERROR mul_n_aggregate(const P1_Affine* pk, const P2_Affine* sig,
                               const byte* scalar, size_t nbits,
                               const app__string_view msg,
                               const app__string_view aug = None)
    {   return mul_n_aggregate(pk, sig, scalar, nbits,
                               C_bytes(msg.data()), msg.size(),
                               C_bytes(aug.data()), aug.size());
    }
    BLST_ERROR mul_n_aggregate(const P2_Affine* pk, const P1_Affine* sig,
                               const byte* scalar, size_t nbits,
                               const app__string_view msg,
                               const app__string_view aug = None)
    {   return mul_n_aggregate(pk, sig, scalar, nbits,
                               C_bytes(msg.data()), msg.size(),
                               C_bytes(aug.data()), aug.size());
    }
#endif
    void commit()
    {   blst_pairing_commit(*this);   }
    BLST_ERROR merge(const Pairing* ctx)
    {   return blst_pairing_merge(*this, *ctx);   }
    bool finalverify(const PT* sig = nullptr) const
    {   return sig == nullptr ? blst_pairing_finalverify(*this, nullptr)
                              : blst_pairing_finalverify(*this, *sig);
    }
    void raw_aggregate(const P2_Affine* q, const P1_Affine* p)
    {   blst_pairing_raw_aggregate(*this, *q, *p);   }
    PT as_fp12()
    {   return PT(blst_pairing_as_fp12(*this));   }
};

} // namespace blst

#endif
