/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "point.h"
#include "fields.h"

/*
 * y^2 = x^3 + A'*x + B', isogenous one
 */
static const vec384x Aprime_E2 = {      /* 240*i */
  { 0 },
  { TO_LIMB_T(0xe53a000003135242), TO_LIMB_T(0x01080c0fdef80285),
    TO_LIMB_T(0xe7889edbe340f6bd), TO_LIMB_T(0x0b51375126310601),
    TO_LIMB_T(0x02d6985717c744ab), TO_LIMB_T(0x1220b4e979ea5467) }
};
static const vec384x Bprime_E2 = {      /* 1012 + 1012*i */
  { TO_LIMB_T(0x22ea00000cf89db2), TO_LIMB_T(0x6ec832df71380aa4),
    TO_LIMB_T(0x6e1b94403db5a66e), TO_LIMB_T(0x75bf3c53a79473ba),
    TO_LIMB_T(0x3dd3a569412c0a34), TO_LIMB_T(0x125cdb5e74dc4fd1) },
  { TO_LIMB_T(0x22ea00000cf89db2), TO_LIMB_T(0x6ec832df71380aa4),
    TO_LIMB_T(0x6e1b94403db5a66e), TO_LIMB_T(0x75bf3c53a79473ba),
    TO_LIMB_T(0x3dd3a569412c0a34), TO_LIMB_T(0x125cdb5e74dc4fd1) }
};

static void map_fp2_times_Zz(vec384x map[], const vec384x isogeny_map[],
                             const vec384x Zz_powers[], size_t n)
{
    while (n--)
        mul_fp2(map[n], isogeny_map[n], Zz_powers[n]);
}

static void map_fp2(vec384x acc, const vec384x x, const vec384x map[], size_t n)
{
    while (n--) {
        mul_fp2(acc, acc, x);
        add_fp2(acc, acc, map[n]);
    }
}

static void isogeny_map_to_E2(POINTonE2 *out, const POINTonE2 *p)
{
    /*
     * x = x_num / x_den, where
     * x_num = k_(1,3) * x'^3 + k_(1,2) * x'^2 + k_(1,1) * x' + k_(1,0)
     * ...
     */
    static const vec384x isogeny_map_x_num[] = {    /* (k_(1,*)<<384) % P   */
     {{ TO_LIMB_T(0x47f671c71ce05e62), TO_LIMB_T(0x06dd57071206393e),
        TO_LIMB_T(0x7c80cd2af3fd71a2), TO_LIMB_T(0x048103ea9e6cd062),
        TO_LIMB_T(0xc54516acc8d037f6), TO_LIMB_T(0x13808f550920ea41) },
      { TO_LIMB_T(0x47f671c71ce05e62), TO_LIMB_T(0x06dd57071206393e),
        TO_LIMB_T(0x7c80cd2af3fd71a2), TO_LIMB_T(0x048103ea9e6cd062),
        TO_LIMB_T(0xc54516acc8d037f6), TO_LIMB_T(0x13808f550920ea41) }},
     {{ 0 },
      { TO_LIMB_T(0x5fe55555554c71d0), TO_LIMB_T(0x873fffdd236aaaa3),
        TO_LIMB_T(0x6a6b4619b26ef918), TO_LIMB_T(0x21c2888408874945),
        TO_LIMB_T(0x2836cda7028cabc5), TO_LIMB_T(0x0ac73310a7fd5abd) }},
     {{ TO_LIMB_T(0x0a0c5555555971c3), TO_LIMB_T(0xdb0c00101f9eaaae),
        TO_LIMB_T(0xb1fb2f941d797997), TO_LIMB_T(0xd3960742ef416e1c),
        TO_LIMB_T(0xb70040e2c20556f4), TO_LIMB_T(0x149d7861e581393b) },
      { TO_LIMB_T(0xaff2aaaaaaa638e8), TO_LIMB_T(0x439fffee91b55551),
        TO_LIMB_T(0xb535a30cd9377c8c), TO_LIMB_T(0x90e144420443a4a2),
        TO_LIMB_T(0x941b66d3814655e2), TO_LIMB_T(0x0563998853fead5e) }},
     {{ TO_LIMB_T(0x40aac71c71c725ed), TO_LIMB_T(0x190955557a84e38e),
        TO_LIMB_T(0xd817050a8f41abc3), TO_LIMB_T(0xd86485d4c87f6fb1),
        TO_LIMB_T(0x696eb479f885d059), TO_LIMB_T(0x198e1a74328002d2) },
      { 0 }}
    };
    /* ...
     * x_den = x'^2 + k_(2,1) * x' + k_(2,0)
     */
    static const vec384x isogeny_map_x_den[] = {    /* (k_(2,*)<<384) % P   */
     {{ 0 },
      { TO_LIMB_T(0x1f3affffff13ab97), TO_LIMB_T(0xf25bfc611da3ff3e),
        TO_LIMB_T(0xca3757cb3819b208), TO_LIMB_T(0x3e6427366f8cec18),
        TO_LIMB_T(0x03977bc86095b089), TO_LIMB_T(0x04f69db13f39a952) }},
     {{ TO_LIMB_T(0x447600000027552e), TO_LIMB_T(0xdcb8009a43480020),
        TO_LIMB_T(0x6f7ee9ce4a6e8b59), TO_LIMB_T(0xb10330b7c0a95bc6),
        TO_LIMB_T(0x6140b1fcfb1e54b7), TO_LIMB_T(0x0381be097f0bb4e1) },
      { TO_LIMB_T(0x7588ffffffd8557d), TO_LIMB_T(0x41f3ff646e0bffdf),
        TO_LIMB_T(0xf7b1e8d2ac426aca), TO_LIMB_T(0xb3741acd32dbb6f8),
        TO_LIMB_T(0xe9daf5b9482d581f), TO_LIMB_T(0x167f53e0ba7431b8) }}
    };
    /*
     * y = y' * y_num / y_den, where
     * y_num = k_(3,3) * x'^3 + k_(3,2) * x'^2 + k_(3,1) * x' + k_(3,0)
     * ...
     */
    static const vec384x isogeny_map_y_num[] = {    /* (k_(3,*)<<384) % P   */
     {{ TO_LIMB_T(0x96d8f684bdfc77be), TO_LIMB_T(0xb530e4f43b66d0e2),
        TO_LIMB_T(0x184a88ff379652fd), TO_LIMB_T(0x57cb23ecfae804e1),
        TO_LIMB_T(0x0fd2e39eada3eba9), TO_LIMB_T(0x08c8055e31c5d5c3) },
      { TO_LIMB_T(0x96d8f684bdfc77be), TO_LIMB_T(0xb530e4f43b66d0e2),
        TO_LIMB_T(0x184a88ff379652fd), TO_LIMB_T(0x57cb23ecfae804e1),
        TO_LIMB_T(0x0fd2e39eada3eba9), TO_LIMB_T(0x08c8055e31c5d5c3) }},
     {{ 0 },
      { TO_LIMB_T(0xbf0a71c71c91b406), TO_LIMB_T(0x4d6d55d28b7638fd),
        TO_LIMB_T(0x9d82f98e5f205aee), TO_LIMB_T(0xa27aa27b1d1a18d5),
        TO_LIMB_T(0x02c3b2b2d2938e86), TO_LIMB_T(0x0c7d13420b09807f) }},
     {{ TO_LIMB_T(0xd7f9555555531c74), TO_LIMB_T(0x21cffff748daaaa8),
        TO_LIMB_T(0x5a9ad1866c9bbe46), TO_LIMB_T(0x4870a2210221d251),
        TO_LIMB_T(0x4a0db369c0a32af1), TO_LIMB_T(0x02b1ccc429ff56af) },
      { TO_LIMB_T(0xe205aaaaaaac8e37), TO_LIMB_T(0xfcdc000768795556),
        TO_LIMB_T(0x0c96011a8a1537dd), TO_LIMB_T(0x1c06a963f163406e),
        TO_LIMB_T(0x010df44c82a881e6), TO_LIMB_T(0x174f45260f808feb) }},
     {{ TO_LIMB_T(0xa470bda12f67f35c), TO_LIMB_T(0xc0fe38e23327b425),
        TO_LIMB_T(0xc9d3d0f2c6f0678d), TO_LIMB_T(0x1c55c9935b5a982e),
        TO_LIMB_T(0x27f6c0e2f0746764), TO_LIMB_T(0x117c5e6e28aa9054) },
      { 0 }}
    };
    /* ...
     * y_den = x'^3 + k_(4,2) * x'^2 + k_(4,1) * x' + k_(4,0)
     */
    static const vec384x isogeny_map_y_den[] = {    /* (k_(4,*)<<384) % P   */
     {{ TO_LIMB_T(0x0162fffffa765adf), TO_LIMB_T(0x8f7bea480083fb75),
        TO_LIMB_T(0x561b3c2259e93611), TO_LIMB_T(0x11e19fc1a9c875d5),
        TO_LIMB_T(0xca713efc00367660), TO_LIMB_T(0x03c6a03d41da1151) },
      { TO_LIMB_T(0x0162fffffa765adf), TO_LIMB_T(0x8f7bea480083fb75),
        TO_LIMB_T(0x561b3c2259e93611), TO_LIMB_T(0x11e19fc1a9c875d5),
        TO_LIMB_T(0xca713efc00367660), TO_LIMB_T(0x03c6a03d41da1151) }},
     {{ 0 },
      { TO_LIMB_T(0x5db0fffffd3b02c5), TO_LIMB_T(0xd713f52358ebfdba),
        TO_LIMB_T(0x5ea60761a84d161a), TO_LIMB_T(0xbb2c75a34ea6c44a),
        TO_LIMB_T(0x0ac6735921c1119b), TO_LIMB_T(0x0ee3d913bdacfbf6) }},
     {{ TO_LIMB_T(0x66b10000003affc5), TO_LIMB_T(0xcb1400e764ec0030),
        TO_LIMB_T(0xa73e5eb56fa5d106), TO_LIMB_T(0x8984c913a0fe09a9),
        TO_LIMB_T(0x11e10afb78ad7f13), TO_LIMB_T(0x05429d0e3e918f52) },
      { TO_LIMB_T(0x534dffffffc4aae6), TO_LIMB_T(0x5397ff174c67ffcf),
        TO_LIMB_T(0xbff273eb870b251d), TO_LIMB_T(0xdaf2827152870915),
        TO_LIMB_T(0x393a9cbaca9e2dc3), TO_LIMB_T(0x14be74dbfaee5748) }}
    };
    vec384x Zz_powers[3], map[3], xn, xd, yn, yd;

    /* lay down Z^2 powers in descending order                          */
    sqr_fp2(Zz_powers[2], p->Z);                       /* ZZ^1          */
    sqr_fp2(Zz_powers[1], Zz_powers[2]);               /* ZZ^2  1+1     */
    mul_fp2(Zz_powers[0], Zz_powers[2], Zz_powers[1]); /* ZZ^3  2+1     */

    map_fp2_times_Zz(map, isogeny_map_x_num, Zz_powers, 3);
    mul_fp2(xn, p->X, isogeny_map_x_num[3]);
    add_fp2(xn, xn, map[2]);
    map_fp2(xn, p->X, map, 2);

    map_fp2_times_Zz(map, isogeny_map_x_den, Zz_powers + 1, 2);
    add_fp2(xd, p->X, map[1]);
    map_fp2(xd, p->X, map, 1);
    mul_fp2(xd, xd, Zz_powers[2]);      /* xd *= Z^2                    */

    map_fp2_times_Zz(map, isogeny_map_y_num, Zz_powers, 3);
    mul_fp2(yn, p->X, isogeny_map_y_num[3]);
    add_fp2(yn, yn, map[2]);
    map_fp2(yn, p->X, map, 2);
    mul_fp2(yn, yn, p->Y);              /* yn *= Y                      */

    map_fp2_times_Zz(map, isogeny_map_y_den, Zz_powers, 3);
    add_fp2(yd, p->X, map[2]);
    map_fp2(yd, p->X, map, 2);
    mul_fp2(Zz_powers[2], Zz_powers[2], p->Z);
    mul_fp2(yd, yd, Zz_powers[2]);      /* yd *= Z^3                    */

    /* convert (xn, xd, yn, yd) to Jacobian coordinates                 */
    mul_fp2(out->Z, xd, yd);            /* Z = xd * yd                  */
    mul_fp2(out->X, xn, yd);
    mul_fp2(out->X, out->X, out->Z);    /* X = xn * xd * yd^2           */
    sqr_fp2(out->Y, out->Z);
    mul_fp2(out->Y, out->Y, xd);
    mul_fp2(out->Y, out->Y, yn);        /* Y = yn * xd^3 * yd^2         */
}

static void map_to_isogenous_E2(POINTonE2 *p, const vec384x u)
{
    static const vec384x minus_A = {
      { 0 },
      { TO_LIMB_T(0xd4c4fffffcec5869), TO_LIMB_T(0x1da3f3eed25bfd79),
        TO_LIMB_T(0x7fa833c5136fff67), TO_LIMB_T(0x59261433cd540cbd),
        TO_LIMB_T(0x48450f5f2b84682c), TO_LIMB_T(0x07e05d00bf959233) }
    };
    static const vec384x Z = {              /* -2 - i */
      { TO_LIMB_T(0x87ebfffffff9555c), TO_LIMB_T(0x656fffe5da8ffffa),
        TO_LIMB_T(0x0fd0749345d33ad2), TO_LIMB_T(0xd951e663066576f4),
        TO_LIMB_T(0xde291a3d41e980d3), TO_LIMB_T(0x0815664c7dfe040d) },
      { TO_LIMB_T(0x43f5fffffffcaaae), TO_LIMB_T(0x32b7fff2ed47fffd),
        TO_LIMB_T(0x07e83a49a2e99d69), TO_LIMB_T(0xeca8f3318332bb7a),
        TO_LIMB_T(0xef148d1ea0f4c069), TO_LIMB_T(0x040ab3263eff0206) }
    };
    static const vec384x recip_ZZZ = {      /* 1/(Z^3) */
      { TO_LIMB_T(0x65018f5c28f598eb), TO_LIMB_T(0xe6020417f022d916),
        TO_LIMB_T(0xd6327313288369c7), TO_LIMB_T(0x622ded8eb447156f),
        TO_LIMB_T(0xe52a2aee72c2a01f), TO_LIMB_T(0x089812fb8481ffe4) },
      { TO_LIMB_T(0x2574eb851eb8619f), TO_LIMB_T(0xdba2e97912925604),
        TO_LIMB_T(0x67e495a909e7a18e), TO_LIMB_T(0xdf2da23b8145b8f7),
        TO_LIMB_T(0xcf5d3728310ebf6d), TO_LIMB_T(0x11be446236f4c116) }
    };
    static const vec384x magic_ZZZ = {      /* 1/Z^3 = a + b*i */
                                            /* a^2 + b^2 */
      { TO_LIMB_T(0xaa7eb851eb8508e0), TO_LIMB_T(0x1c54fdf360989374),
        TO_LIMB_T(0xc87f2fc6e716c62e), TO_LIMB_T(0x0124aefb1f9efea7),
        TO_LIMB_T(0xb2f8be63e844865c), TO_LIMB_T(0x08b47f775a7ef35a) },
                                            /* (a^2 + b^2)^((P-3)/4) */
      { TO_LIMB_T(0xe4132bbd838cf70a), TO_LIMB_T(0x01d769ac83772c19),
        TO_LIMB_T(0xa83dd6e974c22e45), TO_LIMB_T(0xbc8ec3e777b08dff),
        TO_LIMB_T(0xc035c2042ecf5da3), TO_LIMB_T(0x073929e97f0850bf) }
    };
    static const vec384x ZxA = {            /* 240 - 480*i */
      { TO_LIMB_T(0xe53a000003135242), TO_LIMB_T(0x01080c0fdef80285),
        TO_LIMB_T(0xe7889edbe340f6bd), TO_LIMB_T(0x0b51375126310601),
        TO_LIMB_T(0x02d6985717c744ab), TO_LIMB_T(0x1220b4e979ea5467) },
      { TO_LIMB_T(0xa989fffff9d8b0d2), TO_LIMB_T(0x3b47e7dda4b7faf3),
        TO_LIMB_T(0xff50678a26dffece), TO_LIMB_T(0xb24c28679aa8197a),
        TO_LIMB_T(0x908a1ebe5708d058), TO_LIMB_T(0x0fc0ba017f2b2466) }
    };
    vec384x uu, tv2, tv4, x2n, gx1, gxd, y2;
#if 0
    vec384x xn, x1n, xd, y, y1, Zuu;
#else
# define xn     p->X
# define y      p->Y
# define xd     p->Z
# define x1n    xn
# define y1     y
# define Zuu    x2n
#endif
#define sgn0_fp2(a) (sgn0_pty_mont_384x((a), BLS12_381_P, p0) & 1)
    bool_t e1, e2;

    /*
     * as per map_to_curve() from poc/sswu_opt.sage at
     * https://github.com/cfrg/draft-irtf-cfrg-hash-to-curve
     * with 9mod16 twists...
     */
    /* x numerator variants                                             */
    sqr_fp2(uu, u);                     /* uu = u^2                     */
    mul_fp2(Zuu, Z, uu);                /* Zuu = Z * uu                 */
    sqr_fp2(tv2, Zuu);                  /* tv2 = Zuu^2                  */
    add_fp2(tv2, tv2, Zuu);             /* tv2 = tv2 + Zuu              */
    add_fp2(x1n, tv2, BLS12_381_Rx.p2); /* x1n = tv2 + 1                */
    mul_fp2(x1n, x1n, Bprime_E2);       /* x1n = x1n * B                */
    mul_fp2(x2n, Zuu, x1n);             /* x2n = Zuu * x1n              */

    /* x denumenator                                                    */
    mul_fp2(xd, minus_A, tv2);          /* xd = -A * tv2                */
    e1 = vec_is_zero(xd, sizeof(xd));   /* e1 = xd == 0                 */
    vec_select(xd, ZxA, xd, sizeof(xd), e1);    /*              # If xd == 0, set xd = Z*A */

    /* y numerators variants                                            */
    sqr_fp2(tv2, xd);                   /* tv2 = xd^2                   */
    mul_fp2(gxd, xd, tv2);              /* gxd = xd^3                   */
    mul_fp2(tv2, Aprime_E2, tv2);       /* tv2 = A * tv2                */
    sqr_fp2(gx1, x1n);                  /* gx1 = x1n^2                  */
    add_fp2(gx1, gx1, tv2);             /* gx1 = gx1 + tv2      # x1n^2 + A*xd^2 */
    mul_fp2(gx1, gx1, x1n);             /* gx1 = gx1 * x1n      # x1n^3 + A*x1n*xd^2 */
    mul_fp2(tv2, Bprime_E2, gxd);       /* tv2 = B * gxd                */
    add_fp2(gx1, gx1, tv2);             /* gx1 = gx1 + tv2      # x1^3 + A*x1*xd^2 + B*xd^3 */
    sqr_fp2(tv4, gxd);                  /* tv4 = gxd^2                  */
    mul_fp2(tv2, gx1, gxd);             /* tv2 = gx1 * gxd              */
    mul_fp2(tv4, tv4, tv2);             /* tv4 = tv4 * tv2      # gx1*gxd^3 */
    e2 = recip_sqrt_fp2(y1, tv4,        /* y1 = tv4^c1          # (gx1*gxd^3)^((p^2-9)/16) */
                        recip_ZZZ, magic_ZZZ);
    mul_fp2(y1, y1, tv2);               /* y1 = y1 * tv2        # gx1*gxd*y1 */
    mul_fp2(y2, y1, uu);                /* y2 = y1 * uu                 */
    mul_fp2(y2, y2, u);                 /* y2 = y2 * u                  */

    /* choose numerators                                                */
    vec_select(xn, x1n, x2n, sizeof(xn), e2);   /* xn = e2 ? x1n : x2n  */
    vec_select(y, y1, y2, sizeof(y), e2);       /* y  = e2 ? y1 : y2    */

    e1 = sgn0_fp2(u);
    e2 = sgn0_fp2(y);
    cneg_fp2(y, y, e1^e2);              /* fix sign of y                */
                                        /* return (xn, xd, y, 1)        */

    /* convert (xn, xd, y, 1) to Jacobian projective coordinates        */
    mul_fp2(p->X, xn, xd);              /* X = xn * xd                  */
    mul_fp2(p->Y, y, gxd);              /* Y = y * xd^3                 */
#ifndef xd
    vec_copy(p->Z, xd, sizeof(xd));     /* Z = xd                       */
#else
# undef xn
# undef y
# undef xd
# undef x1n
# undef y1
# undef Zuu
# undef tv4
#endif
#undef sgn0_fp2
}

#if 0
static const byte h_eff[] = {
    TO_BYTES(0xe8020005aaa95551), TO_BYTES(0x59894c0adebbf6b4),
    TO_BYTES(0xe954cbc06689f6a3), TO_BYTES(0x2ec0ec69d7477c1a),
    TO_BYTES(0x6d82bf015d1212b0), TO_BYTES(0x329c2f178731db95),
    TO_BYTES(0x9986ff031508ffe1), TO_BYTES(0x88e2a8e9145ad768),
    TO_BYTES(0x584c6a0ea91b3528), TO_BYTES(0x0bc69f08f2ee75b3)
};

static void clear_cofactor(POINTonE2 *out, const POINTonE2 *p)
{    POINTonE2_mult_w5(out, p, h_eff, 636);   }
#else
/*
 * As per suggestions in "7. Clearing the cofactor" at
 * https://tools.ietf.org/html/draft-irtf-cfrg-hash-to-curve-06
 */
static void POINTonE2_add_n_dbl(POINTonE2 *out, const POINTonE2 *p, size_t n)
{
    POINTonE2_dadd(out, out, p, NULL);
    while(n--)
        POINTonE2_double(out, out);
}

static void POINTonE2_times_minus_z(POINTonE2 *out, const POINTonE2 *in)
{
    POINTonE2_double(out, in);          /*      1: 0x2                  */
    POINTonE2_add_n_dbl(out, in, 2);    /*   2..4: 0x3..0xc             */
    POINTonE2_add_n_dbl(out, in, 3);    /*   5..8: 0xd..0x68            */
    POINTonE2_add_n_dbl(out, in, 9);    /*  9..18: 0x69..0xd200         */
    POINTonE2_add_n_dbl(out, in, 32);   /* 19..51: ..0xd20100000000     */
    POINTonE2_add_n_dbl(out, in, 16);   /* 52..68: ..0xd201000000010000 */
}

static void psi(POINTonE2 *out, const POINTonE2 *in);

static void clear_cofactor(POINTonE2 *out, const POINTonE2 *p)
{
    POINTonE2 t0, t1;

    /* A.Budroni, F.Pintore, "Efficient hash maps to G2 on BLS curves"  */
    POINTonE2_double(out, p);           /* out = 2P                     */
    psi(out, out);                      /* out = Ψ(2P)                  */
    psi(out, out);                      /* out = Ψ²(2P)                 */

    vec_copy(&t0, p, sizeof(t0));
    POINTonE2_cneg(&t0, 1);             /* t0 = -P                      */
    psi(&t1, &t0);                      /* t1 = -Ψ(P)                   */
    POINTonE2_dadd(out, out, &t0, NULL);/* out = Ψ²(2P) - P             */
    POINTonE2_dadd(out, out, &t1, NULL);/* out = Ψ²(2P) - P - Ψ(P)      */

    POINTonE2_times_minus_z(&t0, p);    /* t0 = [-z]P                   */
    POINTonE2_dadd(&t0, &t0, p, NULL);  /* t0 = [-z + 1]P               */
    POINTonE2_dadd(&t0, &t0, &t1, NULL);/* t0 = [-z + 1]P - Ψ(P)        */
    POINTonE2_times_minus_z(&t1, &t0);  /* t1 = [z² - z]P + [z]Ψ(P)     */
    POINTonE2_dadd(out, out, &t1, NULL);/* out = [z² - z - 1]P          */
                                        /*     + [z - 1]Ψ(P)            */
                                        /*     + Ψ²(2P)                 */
}
#endif

/*
 * |u|, |v| are expected to be in Montgomery representation
 */
static void map_to_g2(POINTonE2 *out, const vec384x u, const vec384x v)
{
    POINTonE2 p;

    map_to_isogenous_E2(&p, u);

    if (v != NULL) {
        map_to_isogenous_E2(out, v);    /* borrow |out|                 */
        POINTonE2_dadd(&p, &p, out, Aprime_E2);
    }

    isogeny_map_to_E2(&p, &p);          /* sprinkle isogenous powder    */
    clear_cofactor(out, &p);
}

void blst_map_to_g2(POINTonE2 *out, const vec384x u, const vec384x v)
{   map_to_g2(out, u, v);   }

static void Encode_to_G2(POINTonE2 *p, const unsigned char *msg, size_t msg_len,
                                       const unsigned char *DST, size_t DST_len,
                                       const unsigned char *aug, size_t aug_len)
{
    vec384x u[1];

    hash_to_field(u[0], 2, aug, aug_len, msg, msg_len, DST, DST_len);
    map_to_g2(p, u[0], NULL);
}

void blst_encode_to_g2(POINTonE2 *p, const unsigned char *msg, size_t msg_len,
                                     const unsigned char *DST, size_t DST_len,
                                     const unsigned char *aug, size_t aug_len)
{   Encode_to_G2(p, msg, msg_len, DST, DST_len, aug, aug_len);   }

static void Hash_to_G2(POINTonE2 *p, const unsigned char *msg, size_t msg_len,
                                     const unsigned char *DST, size_t DST_len,
                                     const unsigned char *aug, size_t aug_len)
{
    vec384x u[2];

    hash_to_field(u[0], 4, aug, aug_len, msg, msg_len, DST, DST_len);
    map_to_g2(p, u[0], u[1]);
}

void blst_hash_to_g2(POINTonE2 *p, const unsigned char *msg, size_t msg_len,
                                   const unsigned char *DST, size_t DST_len,
                                   const unsigned char *aug, size_t aug_len)
{   Hash_to_G2(p, msg, msg_len, DST, DST_len, aug, aug_len);   }

static bool_t POINTonE2_in_G2(const POINTonE2 *P)
{
#if 0
    POINTonE2 t0, t1, t2;

    /* Bowe, S., "Faster subgroup checks for BLS12-381"                 */
    psi(&t0, P);                        /* Ψ(P)                         */
    psi(&t0, &t0);                      /* Ψ²(P)                        */
    psi(&t1, &t0);                      /* Ψ³(P)                        */

    POINTonE2_times_minus_z(&t2, &t1);
    POINTonE2_dadd(&t0, &t0, &t2, NULL);
    POINTonE2_cneg(&t0, 1);
    POINTonE2_dadd(&t0, &t0, P, NULL);  /* [z]Ψ³(P) - Ψ²(P) + P         */

    return vec_is_zero(t0.Z, sizeof(t0.Z));
#else
    POINTonE2 t0, t1;

    /* Scott, M., https://eprint.iacr.org/2021/1130 */
    psi(&t0, P);                            /* Ψ(P) */

    POINTonE2_times_minus_z(&t1, P);
    POINTonE2_cneg(&t1, 1);                 /* [z]P */

    return POINTonE2_is_equal(&t0, &t1);
#endif
}

int blst_p2_in_g2(const POINTonE2 *p)
{   return (int)POINTonE2_in_G2(p);   }

int blst_p2_affine_in_g2(const POINTonE2_affine *p)
{
    POINTonE2 P;

    vec_copy(P.X, p->X, 2*sizeof(P.X));
    vec_select(P.Z, p->X, BLS12_381_Rx.p, sizeof(P.Z),
                     vec_is_zero(p, sizeof(*p)));

    return (int)POINTonE2_in_G2(&P);
}
