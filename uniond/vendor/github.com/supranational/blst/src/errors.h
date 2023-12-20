/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */
#ifndef __BLS12_381_ASM_ERRORS_H__
#define __BLS12_381_ASM_ERRORS_H__

typedef enum {
    BLST_SUCCESS = 0,
    BLST_BAD_ENCODING,
    BLST_POINT_NOT_ON_CURVE,
    BLST_POINT_NOT_IN_GROUP,
    BLST_AGGR_TYPE_MISMATCH,
    BLST_VERIFY_FAIL,
    BLST_PK_IS_INFINITY,
} BLST_ERROR;

#endif
