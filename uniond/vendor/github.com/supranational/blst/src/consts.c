/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#include "consts.h"

/* z = -0xd201000000010000 */
const vec384 BLS12_381_P = {    /* (z-1)^2 * (z^4 - z^2 + 1)/3 + z */
    TO_LIMB_T(0xb9feffffffffaaab), TO_LIMB_T(0x1eabfffeb153ffff),
    TO_LIMB_T(0x6730d2a0f6b0f624), TO_LIMB_T(0x64774b84f38512bf),
    TO_LIMB_T(0x4b1ba7b6434bacd7), TO_LIMB_T(0x1a0111ea397fe69a)
};
const limb_t BLS12_381_p0 = (limb_t)0x89f3fffcfffcfffd;  /* -1/P */

const radix384 BLS12_381_Rx = { /* (1<<384)%P, "radix", one-in-Montgomery */
  { { ONE_MONT_P },
    { 0 } }
};

const vec384 BLS12_381_RR = {   /* (1<<768)%P, "radix"^2, to-Montgomery */
    TO_LIMB_T(0xf4df1f341c341746), TO_LIMB_T(0x0a76e6a609d104f1),
    TO_LIMB_T(0x8de5476c4c95b6d5), TO_LIMB_T(0x67eb88a9939d83c0),
    TO_LIMB_T(0x9a793e85b519952d), TO_LIMB_T(0x11988fe592cae3aa)
};

const vec256 BLS12_381_r = {    /* z^4 - z^2 + 1, group order */
    TO_LIMB_T(0xffffffff00000001), TO_LIMB_T(0x53bda402fffe5bfe),
    TO_LIMB_T(0x3339d80809a1d805), TO_LIMB_T(0x73eda753299d7d48)
};

const vec256 BLS12_381_rRR = {  /* (1<<512)%r, "radix"^2, to-Montgomery */
    TO_LIMB_T(0xc999e990f3f29c6d), TO_LIMB_T(0x2b6cedcb87925c23),
    TO_LIMB_T(0x05d314967254398f), TO_LIMB_T(0x0748d9d99f59ff11)
};
