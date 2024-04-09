/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */
#ifndef __BLS12_384_ASM_EC_OPS_H__
#define __BLS12_384_ASM_EC_OPS_H__
/*
 * Addition that can handle doubling [as well as points at infinity,
 * which are encoded as Z==0] in constant time. It naturally comes at
 * cost, but this subroutine should be called only when independent
 * points are processed, which is considered reasonable compromise.
 * For example, ptype##s_mult_w5 calls it, but since *major* gain is
 * result of pure doublings being effectively divided by amount of
 * points, slightly slower addition can be tolerated. But what is the
 * additional cost more specifically? Best addition result is 11M+5S,
 * while this routine takes 13M+5S (+1M+1S if a4!=0), as per
 *
 * -------------+-------------
 * addition     | doubling
 * -------------+-------------
 * U1 = X1*Z2^2 | U1 = X1
 * U2 = X2*Z1^2 |
 * S1 = Y1*Z2^3 | S1 = Y1
 * S2 = Y2*Z1^3 |
 * zz = Z1*Z2   | zz = Z1
 * H = U2-U1    | H' = 2*Y1
 * R = S2-S1    | R' = 3*X1^2[+a*Z1^4]
 * sx = U1+U2   | sx = X1+X1
 * -------------+-------------
 * H!=0 || R!=0 | H==0 && R==0
 *
 *      X3 = R^2-H^2*sx
 *      Y3 = R*(H^2*U1-X3)-H^3*S1
 *      Z3 = H*zz
 *
 * As for R!=0 condition in context of H==0, a.k.a. P-P. The result is
 * infinity by virtue of Z3 = (U2-U1)*zz = H*zz = 0*zz == 0.
 */
#define POINT_DADD_IMPL(ptype, bits, field) \
static void ptype##_dadd(ptype *out, const ptype *p1, const ptype *p2, \
                         const vec##bits a4) \
{ \
    ptype p3; /* starts as (U1, S1, zz) from addition side */\
    struct { vec##bits H, R, sx; } add, dbl; \
    bool_t p1inf, p2inf, is_dbl; \
\
    add_##field(dbl.sx, p1->X, p1->X);  /* sx = X1+X1 */\
    sqr_##field(dbl.R, p1->X);          /* X1^2 */\
    mul_by_3_##field(dbl.R, dbl.R);     /* R = 3*X1^2 */\
    add_##field(dbl.H, p1->Y, p1->Y);   /* H = 2*Y1 */\
\
    p2inf = vec_is_zero(p2->Z, sizeof(p2->Z)); \
    sqr_##field(p3.X, p2->Z);           /* Z2^2 */\
    mul_##field(p3.Z, p1->Z, p2->Z);    /* Z1*Z2 */\
    p1inf = vec_is_zero(p1->Z, sizeof(p1->Z)); \
    sqr_##field(add.H, p1->Z);          /* Z1^2 */\
\
    if (a4 != NULL) { \
        sqr_##field(p3.Y, add.H);       /* Z1^4, [borrow p3.Y] */\
        mul_##field(p3.Y, p3.Y, a4);    \
        add_##field(dbl.R, dbl.R, p3.Y);/* R = 3*X1^2+a*Z1^4 */\
    } \
\
    mul_##field(p3.Y, p1->Y, p2->Z);    \
    mul_##field(p3.Y, p3.Y, p3.X);      /* S1 = Y1*Z2^3 */\
    mul_##field(add.R, p2->Y, p1->Z);   \
    mul_##field(add.R, add.R, add.H);   /* S2 = Y2*Z1^3 */\
    sub_##field(add.R, add.R, p3.Y);    /* R = S2-S1 */\
\
    mul_##field(p3.X, p3.X, p1->X);     /* U1 = X1*Z2^2 */\
    mul_##field(add.H, add.H, p2->X);   /* U2 = X2*Z1^2 */\
\
    add_##field(add.sx, add.H, p3.X);   /* sx = U1+U2 */\
    sub_##field(add.H, add.H, p3.X);    /* H = U2-U1 */\
\
    /* make the choice between addition and doubling */\
    is_dbl = vec_is_zero(add.H, 2*sizeof(add.H));      \
    vec_select(&p3, p1, &p3, sizeof(p3), is_dbl);      \
    vec_select(&add, &dbl, &add, sizeof(add), is_dbl); \
    /* |p3| and |add| hold all inputs now, |p3| will hold output */\
\
    mul_##field(p3.Z, p3.Z, add.H);     /* Z3 = H*Z1*Z2 */\
\
    sqr_##field(dbl.H, add.H);          /* H^2 */\
    mul_##field(dbl.R, dbl.H, add.H);   /* H^3 */\
    mul_##field(dbl.R, dbl.R, p3.Y);    /* H^3*S1 */\
    mul_##field(p3.Y, dbl.H, p3.X);     /* H^2*U1 */\
\
    mul_##field(dbl.H, dbl.H, add.sx);  /* H^2*sx */\
    sqr_##field(p3.X, add.R);           /* R^2 */\
    sub_##field(p3.X, p3.X, dbl.H);     /* X3 = R^2-H^2*sx */\
\
    sub_##field(p3.Y, p3.Y, p3.X);      /* H^2*U1-X3 */\
    mul_##field(p3.Y, p3.Y, add.R);     /* R*(H^2*U1-X3) */\
    sub_##field(p3.Y, p3.Y, dbl.R);     /* Y3 = R*(H^2*U1-X3)-H^3*S1 */\
\
    vec_select(&p3, p1, &p3, sizeof(ptype), p2inf); \
    vec_select(out, p2, &p3, sizeof(ptype), p1inf); \
}

/*
 * Addition with affine point that can handle doubling [as well as
 * points at infinity, with |p1| being encoded as Z==0 and |p2| as
 * X,Y==0] in constant time. But at what additional cost? Best
 * addition result is 7M+4S, while this routine takes 8M+5S, as per
 *
 * -------------+-------------
 * addition     | doubling
 * -------------+-------------
 * U1 = X1      | U1 = X2
 * U2 = X2*Z1^2 |
 * S1 = Y1      | S1 = Y2
 * S2 = Y2*Z1^3 |
 * H = U2-X1    | H' = 2*Y2
 * R = S2-Y1    | R' = 3*X2^2[+a]
 * sx = X1+U2   | sx = X2+X2
 * zz = H*Z1    | zz = H'
 * -------------+-------------
 * H!=0 || R!=0 | H==0 && R==0
 *
 *      X3 = R^2-H^2*sx
 *      Y3 = R*(H^2*U1-X3)-H^3*S1
 *      Z3 = zz
 *
 * As for R!=0 condition in context of H==0, a.k.a. P-P. The result is
 * infinity by virtue of Z3 = (U2-U1)*zz = H*zz = 0*zz == 0.
 */
#define POINT_DADD_AFFINE_IMPL_A0(ptype, bits, field, one) \
static void ptype##_dadd_affine(ptype *out, const ptype *p1, \
                                            const ptype##_affine *p2) \
{ \
    ptype p3; /* starts as (,, H*Z1) from addition side */\
    struct { vec##bits H, R, sx; } add, dbl; \
    bool_t p1inf, p2inf, is_dbl; \
\
    p2inf = vec_is_zero(p2->X, 2*sizeof(p2->X)); \
    add_##field(dbl.sx, p2->X, p2->X);  /* sx = X2+X2 */\
    sqr_##field(dbl.R, p2->X);          /* X2^2 */\
    mul_by_3_##field(dbl.R, dbl.R);     /* R = 3*X2^2 */\
    add_##field(dbl.H, p2->Y, p2->Y);   /* H = 2*Y2 */\
\
    p1inf = vec_is_zero(p1->Z, sizeof(p1->Z)); \
    sqr_##field(add.H, p1->Z);          /* Z1^2 */\
    mul_##field(add.R, add.H, p1->Z);   /* Z1^3 */\
    mul_##field(add.R, add.R, p2->Y);   /* S2 = Y2*Z1^3 */\
    sub_##field(add.R, add.R, p1->Y);   /* R = S2-Y1 */\
\
    mul_##field(add.H, add.H, p2->X);   /* U2 = X2*Z1^2 */\
\
    add_##field(add.sx, add.H, p1->X);  /* sx = X1+U2 */\
    sub_##field(add.H, add.H, p1->X);   /* H = U2-X1 */\
\
    mul_##field(p3.Z, add.H, p1->Z);    /* Z3 = H*Z1 */\
\
    /* make the choice between addition and doubling */ \
    is_dbl = vec_is_zero(add.H, 2*sizeof(add.H));       \
    vec_select(p3.X, p2, p1, 2*sizeof(p3.X), is_dbl);   \
    vec_select(p3.Z, dbl.H, p3.Z, sizeof(p3.Z), is_dbl);\
    vec_select(&add, &dbl, &add, sizeof(add), is_dbl);  \
    /* |p3| and |add| hold all inputs now, |p3| will hold output */\
\
    sqr_##field(dbl.H, add.H);          /* H^2 */\
    mul_##field(dbl.R, dbl.H, add.H);   /* H^3 */\
    mul_##field(dbl.R, dbl.R, p3.Y);    /* H^3*S1 */\
    mul_##field(p3.Y, dbl.H, p3.X);     /* H^2*U1 */\
\
    mul_##field(dbl.H, dbl.H, add.sx);  /* H^2*sx */\
    sqr_##field(p3.X, add.R);           /* R^2 */\
    sub_##field(p3.X, p3.X, dbl.H);     /* X3 = R^2-H^2*sx */\
\
    sub_##field(p3.Y, p3.Y, p3.X);      /* H^2*U1-X3 */\
    mul_##field(p3.Y, p3.Y, add.R);     /* R*(H^2*U1-X3) */\
    sub_##field(p3.Y, p3.Y, dbl.R);     /* Y3 = R*(H^2*U1-X3)-H^3*S1 */\
\
    vec_select(p3.X, p2,  p3.X, 2*sizeof(p3.X), p1inf); \
    vec_select(p3.Z, one, p3.Z, sizeof(p3.Z), p1inf); \
    vec_select(out, p1, &p3, sizeof(ptype), p2inf); \
}

/*
 * https://www.hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#addition-add-2007-bl
 * with twist to handle either input at infinity, which are encoded as Z==0.
 */
#define POINT_ADD_IMPL(ptype, bits, field) \
static void ptype##_add(ptype *out, const ptype *p1, const ptype *p2) \
{ \
    ptype p3; \
    vec##bits Z1Z1, Z2Z2, U1, S1, H, I, J; \
    bool_t p1inf, p2inf; \
\
    p1inf = vec_is_zero(p1->Z, sizeof(p1->Z)); \
    sqr_##field(Z1Z1, p1->Z);           /* Z1Z1 = Z1^2 */\
\
    mul_##field(p3.Z, Z1Z1, p1->Z);     /* Z1*Z1Z1 */\
    mul_##field(p3.Z, p3.Z, p2->Y);     /* S2 = Y2*Z1*Z1Z1 */\
\
    p2inf = vec_is_zero(p2->Z, sizeof(p2->Z)); \
    sqr_##field(Z2Z2, p2->Z);           /* Z2Z2 = Z2^2 */\
\
    mul_##field(S1, Z2Z2, p2->Z);       /* Z2*Z2Z2 */\
    mul_##field(S1, S1, p1->Y);         /* S1 = Y1*Z2*Z2Z2 */\
\
    sub_##field(p3.Z, p3.Z, S1);        /* S2-S1 */\
    add_##field(p3.Z, p3.Z, p3.Z);      /* r = 2*(S2-S1) */\
\
    mul_##field(U1, p1->X, Z2Z2);       /* U1 = X1*Z2Z2 */\
    mul_##field(H,  p2->X, Z1Z1);       /* U2 = X2*Z1Z1 */\
\
    sub_##field(H, H, U1);              /* H = U2-U1 */\
\
    add_##field(I, H, H);               /* 2*H */\
    sqr_##field(I, I);                  /* I = (2*H)^2 */\
\
    mul_##field(J, H, I);               /* J = H*I */\
    mul_##field(S1, S1, J);             /* S1*J */\
\
    mul_##field(p3.Y, U1, I);           /* V = U1*I */\
\
    sqr_##field(p3.X, p3.Z);            /* r^2 */\
    sub_##field(p3.X, p3.X, J);         /* r^2-J */\
    sub_##field(p3.X, p3.X, p3.Y);      \
    sub_##field(p3.X, p3.X, p3.Y);      /* X3 = r^2-J-2*V */\
\
    sub_##field(p3.Y, p3.Y, p3.X);      /* V-X3 */\
    mul_##field(p3.Y, p3.Y, p3.Z);      /* r*(V-X3) */\
    sub_##field(p3.Y, p3.Y, S1);        \
    sub_##field(p3.Y, p3.Y, S1);        /* Y3 = r*(V-X3)-2*S1*J */\
\
    add_##field(p3.Z, p1->Z, p2->Z);    /* Z1+Z2 */\
    sqr_##field(p3.Z, p3.Z);            /* (Z1+Z2)^2 */\
    sub_##field(p3.Z, p3.Z, Z1Z1);      /* (Z1+Z2)^2-Z1Z1 */\
    sub_##field(p3.Z, p3.Z, Z2Z2);      /* (Z1+Z2)^2-Z1Z1-Z2Z2 */\
    mul_##field(p3.Z, p3.Z, H);         /* Z3 = ((Z1+Z2)^2-Z1Z1-Z2Z2)*H */\
\
    vec_select(&p3, p1, &p3, sizeof(ptype), p2inf); \
    vec_select(out, p2, &p3, sizeof(ptype), p1inf); \
}

/*
 * https://hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#addition-madd-2007-bl
 * with twist to handle either input at infinity, with |p1| encoded as Z==0,
 * and |p2| as X==Y==0.
 */
#define POINT_ADD_AFFINE_IMPL(ptype, bits, field, one) \
static void ptype##_add_affine(ptype *out, const ptype *p1, \
                                           const ptype##_affine *p2) \
{ \
    ptype p3; \
    vec##bits Z1Z1, H, HH, I, J; \
    bool_t p1inf, p2inf; \
\
    p1inf = vec_is_zero(p1->Z, sizeof(p1->Z)); \
\
    sqr_##field(Z1Z1, p1->Z);           /* Z1Z1 = Z1^2 */\
\
    mul_##field(p3.Z, Z1Z1, p1->Z);     /* Z1*Z1Z1 */\
    mul_##field(p3.Z, p3.Z, p2->Y);     /* S2 = Y2*Z1*Z1Z1 */\
\
    p2inf = vec_is_zero(p2->X, 2*sizeof(p2->X)); \
\
    mul_##field(H, p2->X, Z1Z1);        /* U2 = X2*Z1Z1 */\
    sub_##field(H, H, p1->X);           /* H = U2-X1 */\
\
    sqr_##field(HH, H);                 /* HH = H^2 */\
    add_##field(I, HH, HH);             \
    add_##field(I, I, I);               /* I = 4*HH */\
\
    mul_##field(p3.Y, p1->X, I);        /* V = X1*I */\
    mul_##field(J, H, I);               /* J = H*I */\
    mul_##field(I, J, p1->Y);           /* Y1*J */\
\
    sub_##field(p3.Z, p3.Z, p1->Y);     /* S2-Y1 */\
    add_##field(p3.Z, p3.Z, p3.Z);      /* r = 2*(S2-Y1) */\
\
    sqr_##field(p3.X, p3.Z);            /* r^2 */\
    sub_##field(p3.X, p3.X, J);         /* r^2-J */\
    sub_##field(p3.X, p3.X, p3.Y);      \
    sub_##field(p3.X, p3.X, p3.Y);      /* X3 = r^2-J-2*V */\
\
    sub_##field(p3.Y, p3.Y, p3.X);      /* V-X3 */\
    mul_##field(p3.Y, p3.Y, p3.Z);      /* r*(V-X3) */\
    sub_##field(p3.Y, p3.Y, I);         \
    sub_##field(p3.Y, p3.Y, I);         /* Y3 = r*(V-X3)-2*Y1*J */\
\
    add_##field(p3.Z, p1->Z, H);        /* Z1+H */\
    sqr_##field(p3.Z, p3.Z);            /* (Z1+H)^2 */\
    sub_##field(p3.Z, p3.Z, Z1Z1);      /* (Z1+H)^2-Z1Z1 */\
    sub_##field(p3.Z, p3.Z, HH);        /* Z3 = (Z1+H)^2-Z1Z1-HH */\
\
    vec_select(p3.Z, one, p3.Z, sizeof(p3.Z), p1inf); \
    vec_select(p3.X, p2,  p3.X, 2*sizeof(p3.X), p1inf); \
    vec_select(out, p1, &p3, sizeof(ptype), p2inf); \
}

/*
 * https://www.hyperelliptic.org/EFD/g1p/auto-shortw-jacobian-0.html#doubling-dbl-2009-l
 */
#define POINT_DOUBLE_IMPL_A0(ptype, bits, field) \
static void ptype##_double(ptype *p3, const ptype *p1) \
{ \
    vec##bits A, B, C; \
\
    sqr_##field(A, p1->X);              /* A = X1^2 */\
    sqr_##field(B, p1->Y);              /* B = Y1^2 */\
    sqr_##field(C, B);                  /* C = B^2 */\
\
    add_##field(B, B, p1->X);           /* X1+B */\
    sqr_##field(B, B);                  /* (X1+B)^2 */\
    sub_##field(B, B, A);               /* (X1+B)^2-A */\
    sub_##field(B, B, C);               /* (X1+B)^2-A-C */\
    add_##field(B, B, B);               /* D = 2*((X1+B)^2-A-C) */\
\
    mul_by_3_##field(A, A);             /* E = 3*A */\
\
    sqr_##field(p3->X, A);              /* F = E^2 */\
    sub_##field(p3->X, p3->X, B);       \
    sub_##field(p3->X, p3->X, B);       /* X3 = F-2*D */\
\
    add_##field(p3->Z, p1->Z, p1->Z);   /* 2*Z1 */\
    mul_##field(p3->Z, p3->Z, p1->Y);   /* Z3 = 2*Z1*Y1 */\
\
    mul_by_8_##field(C, C);             /* 8*C */\
    sub_##field(p3->Y, B, p3->X);       /* D-X3 */\
    mul_##field(p3->Y, p3->Y, A);       /* E*(D-X3) */\
    sub_##field(p3->Y, p3->Y, C);       /* Y3 = E*(D-X3)-8*C */\
}

#define POINT_LADDER_PRE_IMPL(ptype, bits, field) \
static void ptype##xz_ladder_pre(ptype##xz *pxz, const ptype *p) \
{ \
    mul_##field(pxz->X, p->X, p->Z);    /* X2 = X1*Z1 */\
    sqr_##field(pxz->Z, p->Z);          \
    mul_##field(pxz->Z, pxz->Z, p->Z);  /* Z2 = Z1^3 */\
}

/*
 * https://hyperelliptic.org/EFD/g1p/auto-shortw-xz.html#ladder-ladd-2002-it-3
 * with twist to handle either input at infinity, which are encoded as Z==0.
 * Just in case, order of doubling and addition is reverse in comparison to
 * hyperelliptic.org entry. This was done to minimize temporary storage.
 *
 * XZ1 is |p|, XZ2&XZ4 are in&out |r|, XZ3&XZ5 are in&out |s|.
 */
#define POINT_LADDER_STEP_IMPL_A0(ptype, bits, field, suffix4b) \
static void ptype##xz_ladder_step(ptype##xz *r, ptype##xz *s, \
                                  const ptype##xz *p) \
{ \
    ptype##xz p5; \
    vec##bits A, B, C, D, XX, ZZ; \
    bool_t r_inf, s_inf; \
                                        /* s += r */\
    mul_##field(A, r->X, s->X);         /* A = X2*X3 */\
    mul_##field(B, r->Z, s->Z);         /* B = Z2*Z3 */\
    mul_##field(C, r->X, s->Z);         /* C = X2*Z3 */\
    mul_##field(D, r->Z, s->X);         /* D = X3*Z2 */\
\
    sqr_##field(A, A);                  /* (A[-a*B])^2 */\
    add_##field(p5.X, C, D);            /* C+D */\
    mul_##field(p5.X, p5.X, B);         /* B*(C+D) */\
    mul_by_4b_##suffix4b(B, p5.X);      /* b4*B*(C+D) */\
    sub_##field(p5.X, A, B);            /* (A[-a*B])^2-b4*B*(C+D) */\
    mul_##field(p5.X, p5.X, p->Z);      /* X5 = Z1*((A[-a*B])^2-b4*B*(C+D)) */\
\
    sub_##field(p5.Z, C, D);            /* C-D */\
    sqr_##field(p5.Z, p5.Z);            /* (C-D)^2 */\
    mul_##field(p5.Z, p5.Z, p->X);      /* Z5 = X1*(C-D)^2 */\
\
    r_inf = vec_is_zero(r->Z, sizeof(r->Z)); \
    s_inf = vec_is_zero(s->Z, sizeof(s->Z)); \
\
    vec_select(&p5, r, &p5, sizeof(ptype##xz), s_inf); \
    vec_select(s,   s, &p5, sizeof(ptype##xz), r_inf); \
                                        /* r *= 2 */\
    sqr_##field(XX, r->X);              /* XX = X2^2 */\
    sqr_##field(ZZ, r->Z);              /* ZZ = Z2^2 */\
\
    add_##field(r->Z, r->X, r->Z);      /* X2+Z2 */\
    sqr_##field(r->Z, r->Z);            /* (X2+Z2)^2 */\
    sub_##field(r->Z, r->Z, XX);        /* (X2+Z2)^2-XX */\
    sub_##field(r->Z, r->Z, ZZ);        /* E = (X2+Z2)^2-XX-ZZ */\
\
    sqr_##field(A, XX);                 /* (XX[-a*ZZ])^2 */\
    mul_##field(B, r->Z, ZZ);           /* E*ZZ */\
    mul_by_4b_##suffix4b(C, B);         /* b4*E*ZZ */\
    sub_##field(r->X, A, C);            /* X4 = (XX[-a*ZZ])^2-b4*E*ZZ */\
\
    sqr_##field(ZZ, ZZ);                /* ZZ^2 */\
    mul_by_4b_##suffix4b(B, ZZ);        /* b4*ZZ^2 */\
    mul_##field(r->Z, r->Z, XX);        /* E*(XX[+a*ZZ]) */\
    add_##field(r->Z, r->Z, r->Z);      /* 2*E*(XX[+a*ZZ]) */\
    add_##field(r->Z, r->Z, B);         /* Z4 = 2*E*(XX[+a*ZZ])+b4*ZZ^2 */\
}

/*
 * Recover the |r|'s y-coordinate using Eq. (8) from Brier-Joye,
 * "Weierstraß Elliptic Curves and Side-Channel Attacks", with XZ twist
 * and conversion to Jacobian coordinates from <openssl>/.../ecp_smpl.c,
 * and with twist to recover from |s| at infinity [which occurs when
 * multiplying by (order-1)].
 *
 * X4 = 2*Y1*X2*Z3*Z1*Z2
 * Y4 = 2*b*Z3*(Z1*Z2)^2 + Z3*(a*Z1*Z2+X1*X2)*(X1*Z2+X2*Z1) - X3*(X1*Z2-X2*Z1)^2
 * Z4 = 2*Y1*Z3*Z2^2*Z1
 *
 * Z3x2 = 2*Z3
 * Y1Z3x2 = Y1*Z3x2
 * Z1Z2 = Z1*Z2
 * X1Z2 = X1*Z2
 * X2Z1 = X2*Z1
 * X4 = Y1Z3x2*X2*Z1Z2
 * A = b*Z3x2*(Z1Z2)^2
 * B = Z3*(a*Z1Z2+X1*X2)*(X1Z2+X2Z1)
 * C = X3*(X1Z2-X2Z1)^2
 * Y4 = A+B-C
 * Z4 = Y1Z3x2*Z1Z2*Z2
 *
 * XZ1 is |p|, XZ2 is |r|, XZ3 is |s|, 'a' is 0.
 */
#define POINT_LADDER_POST_IMPL_A0(ptype, bits, field, suffixb) \
static void ptype##xz_ladder_post(ptype *p4, \
                                  const ptype##xz *r, const ptype##xz *s, \
                                  const ptype##xz *p, const vec##bits Y1) \
{ \
    vec##bits Z3x2, Y1Z3x2, Z1Z2, X1Z2, X2Z1, A, B, C; \
    bool_t s_inf; \
\
    add_##field(Z3x2, s->Z, s->Z);      /* Z3x2 = 2*Z3 */\
    mul_##field(Y1Z3x2, Y1, Z3x2);      /* Y1Z3x2 = Y1*Z3x2 */\
    mul_##field(Z1Z2, p->Z, r->Z);      /* Z1Z2 = Z1*Z2 */\
    mul_##field(X1Z2, p->X, r->Z);      /* X1Z2 = X1*Z2 */\
    mul_##field(X2Z1, r->X, p->Z);      /* X2Z1 = X2*Z1 */\
\
    mul_##field(p4->X, Y1Z3x2, r->X);   /* Y1Z3x2*X2 */\
    mul_##field(p4->X, p4->X, Z1Z2);    /* X4 = Y1Z3x2*X2*Z1Z2 */\
\
    sqr_##field(A, Z1Z2);               /* (Z1Z2)^2 */\
    mul_##field(B, A, Z3x2);            /* Z3x2*(Z1Z2)^2 */\
    mul_by_b_##suffixb(A, B);           /* A = b*Z3x2*(Z1Z2)^2 */\
\
    mul_##field(B, p->X, r->X);         /* [a*Z1Z2+]X1*X2 */\
    mul_##field(B, B, s->Z);            /* Z3*([a*Z1Z2+]X1*X2) */\
    add_##field(C, X1Z2, X2Z1);         /* X1Z2+X2Z1 */\
    mul_##field(B, B, C);               /* B = Z3*([a*Z2Z1+]X1*X2)*(X1Z2+X2Z1) */\
\
    sub_##field(C, X1Z2, X2Z1);         /* X1Z2-X2Z1 */\
    sqr_##field(C, C);                  /* (X1Z2-X2Z1)^2 */\
    mul_##field(C, C, s->X);            /* C = X3*(X1Z2-X2Z1)^2 */\
\
    add_##field(A, A, B);               /* A+B */\
    sub_##field(A, A, C);               /* Y4 = A+B-C */\
\
    mul_##field(p4->Z, Z1Z2, r->Z);     /* Z1Z2*Z2 */\
    mul_##field(p4->Z, p4->Z, Y1Z3x2);  /* Y1Z3x2*Z1Z2*Z2 */\
\
    s_inf = vec_is_zero(s->Z, sizeof(s->Z)); \
    vec_select(p4->X, p->X, p4->X, sizeof(p4->X), s_inf); \
    vec_select(p4->Y, Y1,   A,     sizeof(p4->Y), s_inf); \
    vec_select(p4->Z, p->Z, p4->Z, sizeof(p4->Z), s_inf); \
    ptype##_cneg(p4, s_inf); \
                                        /* to Jacobian */\
    mul_##field(p4->X, p4->X, p4->Z);   /* X4 = X4*Z4 */\
    sqr_##field(B, p4->Z);              \
    mul_##field(p4->Y, p4->Y, B);       /* Y4 = Y4*Z4^2 */\
}

#define POINT_IS_EQUAL_IMPL(ptype, bits, field) \
static limb_t ptype##_is_equal(const ptype *p1, const ptype *p2) \
{ \
    vec##bits Z1Z1, Z2Z2; \
    ptype##_affine a1, a2; \
    bool_t is_inf1 = vec_is_zero(p1->Z, sizeof(p1->Z)); \
    bool_t is_inf2 = vec_is_zero(p2->Z, sizeof(p2->Z)); \
\
    sqr_##field(Z1Z1, p1->Z);           /* Z1Z1 = Z1^2 */\
    sqr_##field(Z2Z2, p2->Z);           /* Z2Z2 = Z2^2 */\
\
    mul_##field(a1.X, p1->X, Z2Z2);     /* U1 = X1*Z2Z2 */\
    mul_##field(a2.X, p2->X, Z1Z1);     /* U2 = X2*Z1Z1 */\
\
    mul_##field(a1.Y, p1->Y, p2->Z);    /* Y1*Z2 */\
    mul_##field(a2.Y, p2->Y, p1->Z);    /* Y2*Z1 */\
\
    mul_##field(a1.Y, a1.Y, Z2Z2);      /* S1 = Y1*Z2*Z2Z2 */\
    mul_##field(a2.Y, a2.Y, Z1Z1);      /* S2 = Y2*Z1*Z1Z1 */\
\
    return vec_is_equal(&a1, &a2, sizeof(a1)) & (is_inf1 ^ is_inf2 ^ 1); \
}

/*
 * https://eprint.iacr.org/2015/1060, algorithm 7 with a twist to handle
 * |p3| pointing at either |p1| or |p2|. This is resolved by adding |t5|
 * and replacing few first references to |X3| in the formula, up to step
 * 21, with it. 12M[+27A], doubling and infinity are handled by the
 * formula itself. Infinity is to be encoded as [0, !0, 0].
 */
#define POINT_PROJ_DADD_IMPL_A0(ptype, bits, field, suffixb) \
static void ptype##proj_dadd(ptype##proj *p3, const ptype##proj *p1, \
                                              const ptype##proj *p2) \
{ \
    vec##bits t0, t1, t2, t3, t4, t5; \
\
    mul_##field(t0, p1->X, p2->X);      /* 1.     t0 = X1*X2 */\
    mul_##field(t1, p1->Y, p2->Y);      /* 2.     t1 = Y1*Y2 */\
    mul_##field(t2, p1->Z, p2->Z);      /* 3.     t2 = Z1*Z2 */\
    add_##field(t3, p1->X, p1->Y);      /* 4.     t3 = X1+Y1 */\
    add_##field(t4, p2->X, p2->Y);      /* 5.     t4 = X2+Y2 */\
    mul_##field(t3, t3, t4);            /* 6.     t3 = t3*t4 */\
    add_##field(t4, t0, t1);            /* 7.     t4 = t0+t1 */\
    sub_##field(t3, t3, t4);            /* 8.     t3 = t3-t4 */\
    add_##field(t4, p1->Y, p1->Z);      /* 9.     t4 = Y1+Z1 */\
    add_##field(t5, p2->Y, p2->Z);      /* 10.    t5 = Y2+Z2 */\
    mul_##field(t4, t4, t5);            /* 11.    t4 = t4*t5 */\
    add_##field(t5, t1, t2);            /* 12.    t5 = t1+t2 */\
    sub_##field(t4, t4, t5);            /* 13.    t4 = t4-t5 */\
    add_##field(t5, p1->X, p1->Z);      /* 14.    t5 = X1+Z1 */\
    add_##field(p3->Y, p2->X, p2->Z);   /* 15.    Y3 = X2+Z2 */\
    mul_##field(t5, t5, p3->Y);         /* 16.    t5 = t5*Y3 */\
    add_##field(p3->Y, t0, t2);         /* 17.    Y3 = t0+t2 */\
    sub_##field(p3->Y, t5, p3->Y);      /* 18.    Y3 = t5-Y3 */\
    mul_by_3_##field(t0, t0);           /* 19-20. t0 = 3*t0  */\
    mul_by_3_##field(t5, t2);           /* 21.    t5 = 3*t2  */\
    mul_by_b_##suffixb(t2, t5);         /* 21.    t2 = b*t5  */\
    add_##field(p3->Z, t1, t2);         /* 22.    Z3 = t1+t2 */\
    sub_##field(t1, t1, t2);            /* 23.    t1 = t1-t2 */\
    mul_by_3_##field(t5, p3->Y);        /* 24.    t5 = 3*Y3  */\
    mul_by_b_##suffixb(p3->Y, t5);      /* 24.    Y3 = b*t5  */\
    mul_##field(p3->X, t4, p3->Y);      /* 25.    X3 = t4*Y3 */\
    mul_##field(t2, t3, t1);            /* 26.    t2 = t3*t1 */\
    sub_##field(p3->X, t2, p3->X);      /* 27.    X3 = t2-X3 */\
    mul_##field(p3->Y, p3->Y, t0);      /* 28.    Y3 = Y3*t0 */\
    mul_##field(t1, t1, p3->Z);         /* 29.    t1 = t1*Z3 */\
    add_##field(p3->Y, t1, p3->Y);      /* 30.    Y3 = t1+Y3 */\
    mul_##field(t0, t0, t3);            /* 31.    t0 = t0*t3 */\
    mul_##field(p3->Z, p3->Z, t4);      /* 32.    Z3 = Z3*t4 */\
    add_##field(p3->Z, p3->Z, t0);      /* 33.    Z3 = Z3+t0 */\
}

/*
 * https://eprint.iacr.org/2015/1060, algorithm 8 with a twist to handle
 * |p2| being infinity encoded as [0, 0]. 11M[+21A].
 */
#define POINT_PROJ_DADD_AFFINE_IMPL_A0(ptype, bits, field, suffixb) \
static void ptype##proj_dadd_affine(ptype##proj *out, const ptype##proj *p1, \
                                                      const ptype##_affine *p2) \
{ \
    ptype##proj p3[1]; \
    vec##bits t0, t1, t2, t3, t4; \
    limb_t p2inf = vec_is_zero(p2, sizeof(*p2)); \
\
    mul_##field(t0, p1->X, p2->X);      /* 1.     t0 = X1*X2 */\
    mul_##field(t1, p1->Y, p2->Y);      /* 2.     t1 = Y1*Y2 */\
    add_##field(t3, p1->X, p1->Y);      /* 3.     t3 = X1+Y1 */\
    add_##field(t4, p2->X, p2->Y);      /* 4.     t4 = X2+Y2 */\
    mul_##field(t3, t3, t4);            /* 5.     t3 = t3*t4 */\
    add_##field(t4, t0, t1);            /* 6.     t4 = t0+t1 */\
    sub_##field(t3, t3, t4);            /* 7.     t3 = t3-t4 */\
    mul_##field(t4, p2->Y, p1->Z);      /* 8.     t4 = Y2*Z1 */\
    add_##field(t4, t4, p1->Y);         /* 9.     t4 = t4+Y1 */\
    mul_##field(p3->Y, p2->X, p1->Z);   /* 10.    Y3 = X2*Z1 */\
    add_##field(p3->Y, p3->Y, p1->X);   /* 11.    Y3 = Y3+X1 */\
    mul_by_3_##field(t0, t0);           /* 12-13. t0 = 3*t0  */\
    mul_by_b_##suffixb(t2, p1->Z);      /* 14.    t2 = b*Z1  */\
    mul_by_3_##field(t2, t2);           /* 14.    t2 = 3*t2  */\
    add_##field(p3->Z, t1, t2);         /* 15.    Z3 = t1+t2 */\
    sub_##field(t1, t1, t2);            /* 16.    t1 = t1-t2 */\
    mul_by_b_##suffixb(t2, p3->Y);      /* 17.    t2 = b*Y3  */\
    mul_by_3_##field(p3->Y, t2);        /* 17.    Y3 = 3*t2  */\
    mul_##field(p3->X, t4, p3->Y);      /* 18.    X3 = t4*Y3 */\
    mul_##field(t2, t3, t1);            /* 19.    t2 = t3*t1 */\
    sub_##field(p3->X, t2, p3->X);      /* 20.    X3 = t2-X3 */\
    mul_##field(p3->Y, p3->Y, t0);      /* 21.    Y3 = Y3*t0 */\
    mul_##field(t1, t1, p3->Z);         /* 22.    t1 = t1*Z3 */\
    add_##field(p3->Y, t1, p3->Y);      /* 23.    Y3 = t1+Y3 */\
    mul_##field(t0, t0, t3);            /* 24.    t0 = t0*t3 */\
    mul_##field(p3->Z, p3->Z, t4);      /* 25.    Z3 = Z3*t4 */\
    add_##field(p3->Z, p3->Z, t0);      /* 26.    Z3 = Z3+t0 */\
\
    vec_select(out, p1, p3, sizeof(*out), p2inf); \
}

/*
 * https://eprint.iacr.org/2015/1060, algorithm 9 with a twist to handle
 * |p3| pointing at |p1|. This is resolved by adding |t3| to hold X*Y
 * and reordering operations to bring references to |p1| forward.
 * 6M+2S[+13A].
 */
#define POINT_PROJ_DOUBLE_IMPL_A0(ptype, bits, field, suffixb) \
static void ptype##proj_double(ptype##proj *p3, const ptype##proj *p1) \
{ \
    vec##bits t0, t1, t2, t3; \
\
    sqr_##field(t0, p1->Y);             /* 1.     t0 = Y*Y   */\
    mul_##field(t1, p1->Y, p1->Z);      /* 5.     t1 = Y*Z   */\
    sqr_##field(t2, p1->Z);             /* 6.     t2 = Z*Z   */\
    mul_##field(t3, p1->X, p1->Y);      /* 16.    t3 = X*Y   */\
    lshift_##field(p3->Z, t0, 3);       /* 2-4.   Z3 = 8*t0  */\
    mul_by_b_##suffixb(p3->X, t2);      /* 7.     t2 = b*t2  */\
    mul_by_3_##field(t2, p3->X);        /* 7.     t2 = 3*t2  */\
    mul_##field(p3->X, t2, p3->Z);      /* 8.     X3 = t2*Z3 */\
    add_##field(p3->Y, t0, t2);         /* 9.     Y3 = t0+t2 */\
    mul_##field(p3->Z, t1, p3->Z);      /* 10.    Z3 = t1*Z3 */\
    mul_by_3_##field(t2, t2);           /* 11-12. t2 = 3*t2  */\
    sub_##field(t0, t0, t2);            /* 13.    t0 = t0-t2 */\
    mul_##field(p3->Y, t0, p3->Y);      /* 14.    Y3 = t0*Y3 */\
    add_##field(p3->Y, p3->X, p3->Y);   /* 15.    Y3 = X3+Y3 */\
    mul_##field(p3->X, t0, t3);         /* 17.    X3 = t0*t3 */\
    add_##field(p3->X, p3->X, p3->X);   /* 18.    X3 = X3+X3 */\
}

#define POINT_PROJ_TO_JACOBIAN_IMPL(ptype, bits, field) \
static void ptype##proj_to_Jacobian(ptype *out, const ptype##proj *in) \
{ \
    vec##bits ZZ; \
\
    sqr_##field(ZZ, in->Z); \
    mul_##field(out->X, in->X, in->Z); \
    mul_##field(out->Y, in->Y, ZZ); \
    vec_copy(out->Z, in->Z, sizeof(out->Z)); \
}

#define POINT_TO_PROJECTIVE_IMPL(ptype, bits, field, one) \
static void ptype##_to_projective(ptype##proj *out, const ptype *in) \
{ \
    vec##bits ZZ; \
    limb_t is_inf = vec_is_zero(in->Z, sizeof(in->Z)); \
\
    sqr_##field(ZZ, in->Z); \
    mul_##field(out->X, in->X, in->Z); \
    vec_select(out->Y, one, in->Y, sizeof(out->Y), is_inf); \
    mul_##field(out->Z, ZZ, in->Z); \
}

/******************* !!!!! NOT CONSTANT TIME !!!!! *******************/

/*
 * http://hyperelliptic.org/EFD/g1p/auto-shortw-xyzz.html#addition-add-2008-s
 * http://hyperelliptic.org/EFD/g1p/auto-shortw-xyzz.html#doubling-dbl-2008-s-1
 * with twist to handle either input at infinity. Addition costs 12M+2S,
 * while conditional doubling - 4M+6M+3S.
 */
#define POINTXYZZ_DADD_IMPL(ptype, bits, field) \
static void ptype##xyzz_dadd(ptype##xyzz *p3, const ptype##xyzz *p1, \
                                              const ptype##xyzz *p2) \
{ \
    vec##bits U, S, P, R; \
\
    if (vec_is_zero(p2->ZZZ, 2*sizeof(p2->ZZZ))) { \
        vec_copy(p3, p1, sizeof(*p3));  \
        return; \
    } else if (vec_is_zero(p1->ZZZ, 2*sizeof(p1->ZZZ))) { \
        vec_copy(p3, p2, sizeof(*p3));  \
        return; \
    } \
\
    mul_##field(U, p1->X, p2->ZZ);              /* U1 = X1*ZZ2 */\
    mul_##field(S, p1->Y, p2->ZZZ);             /* S1 = Y1*ZZZ2 */\
    mul_##field(P, p2->X, p1->ZZ);              /* U2 = X2*ZZ1 */\
    mul_##field(R, p2->Y, p1->ZZZ);             /* S2 = Y2*ZZZ1 */\
    sub_##field(P, P, U);                       /* P = U2-U1 */\
    sub_##field(R, R, S);                       /* R = S2-S1 */\
\
    if (!vec_is_zero(P, sizeof(P))) {           /* X1!=X2 */\
        vec##bits PP, PPP, Q;                   /* add |p1| and |p2| */\
\
        sqr_##field(PP, P);                     /* PP = P^2 */\
        mul_##field(PPP, PP, P);                /* PPP = P*PP */\
        mul_##field(Q, U, PP);                  /* Q = U1*PP */\
        sqr_##field(p3->X, R);                  /* R^2 */\
        add_##field(P, Q, Q); \
        sub_##field(p3->X, p3->X, PPP);         /* R^2-PPP */\
        sub_##field(p3->X, p3->X, P);           /* X3 = R^2-PPP-2*Q */\
        sub_##field(Q, Q, p3->X); \
        mul_##field(Q, Q, R);                   /* R*(Q-X3) */\
        mul_##field(p3->Y, S, PPP);             /* S1*PPP */\
        sub_##field(p3->Y, Q, p3->Y);           /* Y3 = R*(Q-X3)-S1*PPP */\
        mul_##field(p3->ZZ, p1->ZZ, p2->ZZ);    /* ZZ1*ZZ2 */\
        mul_##field(p3->ZZZ, p1->ZZZ, p2->ZZZ); /* ZZZ1*ZZZ2 */\
        mul_##field(p3->ZZ, p3->ZZ, PP);        /* ZZ3 = ZZ1*ZZ2*PP */\
        mul_##field(p3->ZZZ, p3->ZZZ, PPP);     /* ZZZ3 = ZZZ1*ZZZ2*PPP */\
    } else if (vec_is_zero(R, sizeof(R))) {     /* X1==X2 && Y1==Y2 */\
        vec##bits V, W, M;                      /* double |p1| */\
\
        add_##field(U, p1->Y, p1->Y);           /* U = 2*Y1 */\
        sqr_##field(V, U);                      /* V = U^2 */\
        mul_##field(W, V, U);                   /* W = U*V */\
        mul_##field(S, p1->X, V);               /* S = X1*V */\
        sqr_##field(M, p1->X); \
        mul_by_3_##field(M, M);                 /* M = 3*X1^2[+a*ZZ1^2] */\
        sqr_##field(p3->X, M); \
        add_##field(U, S, S);                   /* 2*S */\
        sub_##field(p3->X, p3->X, U);           /* X3 = M^2-2*S */\
        mul_##field(p3->Y, W, p1->Y);           /* W*Y1 */\
        sub_##field(S, S, p3->X); \
        mul_##field(S, S, M);                   /* M*(S-X3) */\
        sub_##field(p3->Y, S, p3->Y);           /* Y3 = M*(S-X3)-W*Y1 */\
        mul_##field(p3->ZZ, p1->ZZ, V);         /* ZZ3 = V*ZZ1 */\
        mul_##field(p3->ZZZ, p1->ZZZ, W);       /* ZZ3 = W*ZZZ1 */\
    } else {                                    /* X1==X2 && Y1==-Y2 */\
        vec_zero(p3->ZZZ, 2*sizeof(p3->ZZZ));   /* set |p3| to infinity */\
    } \
}

/*
 * http://hyperelliptic.org/EFD/g1p/auto-shortw-xyzz.html#addition-madd-2008-s
 * http://hyperelliptic.org/EFD/g1p/auto-shortw-xyzz.html#doubling-mdbl-2008-s-1
 * with twists to handle even subtractions and either input at infinity.
 * Addition costs 8M+2S, while conditional doubling - 2M+4M+3S.
 */
#define POINTXYZZ_DADD_AFFINE_IMPL(ptype, bits, field, one) \
static void ptype##xyzz_dadd_affine(ptype##xyzz *p3, const ptype##xyzz *p1, \
                                                     const ptype##_affine *p2, \
                                                     bool_t subtract) \
{ \
    vec##bits P, R; \
\
    if (vec_is_zero(p2, sizeof(*p2))) { \
        vec_copy(p3, p1, sizeof(*p3));  \
        return; \
    } else if (vec_is_zero(p1->ZZZ, 2*sizeof(p1->ZZZ))) { \
        vec_copy(p3->X, p2->X, 2*sizeof(p3->X));\
        cneg_##field(p3->ZZZ, one, subtract);   \
        vec_copy(p3->ZZ, one, sizeof(p3->ZZ));  \
        return; \
    } \
\
    mul_##field(P, p2->X, p1->ZZ);              /* U2 = X2*ZZ1 */\
    mul_##field(R, p2->Y, p1->ZZZ);             /* S2 = Y2*ZZZ1 */\
    cneg_##field(R, R, subtract); \
    sub_##field(P, P, p1->X);                   /* P = U2-X1 */\
    sub_##field(R, R, p1->Y);                   /* R = S2-Y1 */\
\
    if (!vec_is_zero(P, sizeof(P))) {           /* X1!=X2 */\
        vec##bits PP, PPP, Q;                   /* add |p2| to |p1| */\
\
        sqr_##field(PP, P);                     /* PP = P^2 */\
        mul_##field(PPP, PP, P);                /* PPP = P*PP */\
        mul_##field(Q, p1->X, PP);              /* Q = X1*PP */\
        sqr_##field(p3->X, R);                  /* R^2 */\
        add_##field(P, Q, Q); \
        sub_##field(p3->X, p3->X, PPP);         /* R^2-PPP */\
        sub_##field(p3->X, p3->X, P);           /* X3 = R^2-PPP-2*Q */\
        sub_##field(Q, Q, p3->X); \
        mul_##field(Q, Q, R);                   /* R*(Q-X3) */\
        mul_##field(p3->Y, p1->Y, PPP);         /* Y1*PPP */\
        sub_##field(p3->Y, Q, p3->Y);           /* Y3 = R*(Q-X3)-Y1*PPP */\
        mul_##field(p3->ZZ, p1->ZZ, PP);        /* ZZ3 = ZZ1*PP */\
        mul_##field(p3->ZZZ, p1->ZZZ, PPP);     /* ZZZ3 = ZZZ1*PPP */\
    } else if (vec_is_zero(R, sizeof(R))) {     /* X1==X2 && Y1==Y2 */\
        vec##bits U, S, M;                      /* double |p2| */\
\
        add_##field(U, p2->Y, p2->Y);           /* U = 2*Y1 */\
        sqr_##field(p3->ZZ, U);                 /* [ZZ3 =] V = U^2 */\
        mul_##field(p3->ZZZ, p3->ZZ, U);        /* [ZZZ3 =] W = U*V */\
        mul_##field(S, p2->X, p3->ZZ);          /* S = X1*V */\
        sqr_##field(M, p2->X); \
        mul_by_3_##field(M, M);                 /* M = 3*X1^2[+a] */\
        sqr_##field(p3->X, M); \
        add_##field(U, S, S);                   /* 2*S */\
        sub_##field(p3->X, p3->X, U);           /* X3 = M^2-2*S */\
        mul_##field(p3->Y, p3->ZZZ, p2->Y);     /* W*Y1 */\
        sub_##field(S, S, p3->X); \
        mul_##field(S, S, M);                   /* M*(S-X3) */\
        sub_##field(p3->Y, S, p3->Y);           /* Y3 = M*(S-X3)-W*Y1 */\
        cneg_##field(p3->ZZZ, p3->ZZZ, subtract); \
    } else {                                    /* X1==X2 && Y1==-Y2 */\
        vec_zero(p3->ZZZ, 2*sizeof(p3->ZZZ));   /* set |p3| to infinity */\
    } \
}

#define POINTXYZZ_TO_JACOBIAN_IMPL(ptype, bits, field) \
static void ptype##xyzz_to_Jacobian(ptype *out, const ptype##xyzz *in) \
{ \
    mul_##field(out->X, in->X, in->ZZ); \
    mul_##field(out->Y, in->Y, in->ZZZ); \
    vec_copy(out->Z, in->ZZ, sizeof(out->Z)); \
}

#define POINT_TO_XYZZ_IMPL(ptype, bits, field) \
static void ptype##_to_xyzz(ptype##xyzz *out, const ptype *in) \
{ \
    vec_copy(out->X, in->X, 2*sizeof(out->X)); \
    sqr_##field(out->ZZ, in->Z); \
    mul_##field(out->ZZZ, out->ZZ, in->Z); \
}

#endif
