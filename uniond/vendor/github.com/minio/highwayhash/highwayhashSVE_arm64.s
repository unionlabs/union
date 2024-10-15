//
// Copyright (c) 2024 Minio Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

//+build !noasm,!appengine

#include "textflag.h"

TEXT ·getVectorLength(SB), NOSPLIT, $0
    WORD $0xd2800002 // mov   x2, #0
    WORD $0x04225022 // addvl x2, x2, #1
    WORD $0xd37df042 // lsl   x2, x2, #3
    WORD $0xd2800003 // mov   x3, #0
    WORD $0x04635023 // addpl x3, x3, #1
    WORD $0xd37df063 // lsl   x3, x3, #3
    MOVD R2, vl+0(FP)
    MOVD R3, pl+8(FP)
    RET

TEXT ·updateArm64Sve(SB), NOSPLIT, $0
    MOVD state+0(FP), R0
    MOVD msg_base+8(FP), R1
    MOVD msg_len+16(FP), R2 // length of message
    SUBS $32, R2
    BMI  completeSve

    WORD $0x2518e3e1 // ptrue p1.b
    WORD $0xa5e0a401 // ld1d  z1.d, p1/z, [x0]
    WORD $0xa5e1a402 // ld1d  z2.d, p1/z, [x0, #1, MUL VL]
    WORD $0xa5e2a403 // ld1d  z3.d, p1/z, [x0, #2, MUL VL]
    WORD $0xa5e3a404 // ld1d  z4.d, p1/z, [x0, #3, MUL VL]

    // Load zipper merge constants table pointer
    MOVD $·zipperMergeSve(SB), R3
    WORD $0xa5e0a465 // ld1d  z5.d, p1/z, [x3]
    WORD $0x25b8c006 // mov   z6.s, #0
    WORD $0x25d8e3e2 // ptrue p2.d              /* set every other lane for "s" type */

loopSve:
    WORD $0xa5e0a420 // ld1d  z0.d, p1/z, [x1]
    ADD  $32, R1

    WORD $0x04e00042 // add z2.d, z2.d, z0.d
    WORD $0x04e30042 // add z2.d, z2.d, z3.d
    WORD $0x04e09420 // lsr z0.d, z1.d, #32
    WORD $0x05a6c847 // sel z7.s, p2, z2.s, z6.s
    WORD $0x04d004e0 // mul z0.d, p1/m, z0.d, z7.d
    WORD $0x04a33003 // eor z3.d, z0.d, z3.d
    WORD $0x04e10081 // add z1.d, z4.d, z1.d
    WORD $0x04e09440 // lsr z0.d, z2.d, #32
    WORD $0x05a6c827 // sel z7.s, p2, z1.s, z6.s
    WORD $0x04d004e0 // mul z0.d, p1/m, z0.d, z7.d
    WORD $0x04a43004 // eor z4.d, z0.d, z4.d
    WORD $0x05253040 // tbl z0.b, z2.b, z5.b
    WORD $0x04e00021 // add z1.d, z1.d, z0.d
    WORD $0x05253020 // tbl z0.b, z1.b, z5.b
    WORD $0x04e00042 // add z2.d, z2.d, z0.d

    SUBS $32, R2
    BPL  loopSve

    WORD $0xe5e0e401 // st1d z1.d, p1, [x0]
    WORD $0xe5e1e402 // st1d z2.d, p1, [x0, #1, MUL VL]
    WORD $0xe5e2e403 // st1d z3.d, p1, [x0, #2, MUL VL]
    WORD $0xe5e3e404 // st1d z4.d, p1, [x0, #3, MUL VL]

completeSve:
    RET

TEXT ·updateArm64Sve2(SB), NOSPLIT, $0
    MOVD state+0(FP), R0
    MOVD msg_base+8(FP), R1
    MOVD msg_len+16(FP), R2 // length of message
    SUBS $32, R2
    BMI  completeSve2

    WORD $0x2518e3e1 // ptrue p1.b
    WORD $0xa5e0a401 // ld1d  z1.d, p1/z, [x0]
    WORD $0xa5e1a402 // ld1d  z2.d, p1/z, [x0, #1, MUL VL]
    WORD $0xa5e2a403 // ld1d  z3.d, p1/z, [x0, #2, MUL VL]
    WORD $0xa5e3a404 // ld1d  z4.d, p1/z, [x0, #3, MUL VL]

    // Load zipper merge constants table pointer
    MOVD $·zipperMergeSve(SB), R3
    WORD $0xa5e0a465 // ld1d  z5.d, p1/z, [x3]

loopSve2:
    WORD $0xa5e0a420 // ld1d  z0.d, p1/z, [x1]
    ADD  $32, R1

    WORD $0x04e00042 // add z2.d, z2.d, z0.d
    WORD $0x04e30042 // add z2.d, z2.d, z3.d
    WORD $0x04e09420 // lsr z0.d, z1.d, #32
    WORD $0x45c27800 // umullb z0.d, z0.s, z2.s
    WORD $0x04a33003 // eor z3.d, z0.d, z3.d
    WORD $0x04e10081 // add z1.d, z4.d, z1.d
    WORD $0x04e09440 // lsr z0.d, z2.d, #32
    WORD $0x45c17800 // umullb z0.d, z0.s, z1.s
    WORD $0x04a43004 // eor z4.d, z0.d, z4.d
    WORD $0x05253040 // tbl z0.b, z2.b, z5.b
    WORD $0x04e00021 // add z1.d, z1.d, z0.d
    WORD $0x05253020 // tbl z0.b, z1.b, z5.b
    WORD $0x04e00042 // add z2.d, z2.d, z0.d

    SUBS $32, R2
    BPL  loopSve2

    WORD $0xe5e0e401 // st1d z1.d, p1, [x0]
    WORD $0xe5e1e402 // st1d z2.d, p1, [x0, #1, MUL VL]
    WORD $0xe5e2e403 // st1d z3.d, p1, [x0, #2, MUL VL]
    WORD $0xe5e3e404 // st1d z4.d, p1, [x0, #3, MUL VL]

completeSve2:
    RET

DATA ·zipperMergeSve+0x00(SB)/8, $0x000f010e05020c03
DATA ·zipperMergeSve+0x08(SB)/8, $0x070806090d0a040b
DATA ·zipperMergeSve+0x10(SB)/8, $0x101f111e15121c13
DATA ·zipperMergeSve+0x18(SB)/8, $0x171816191d1a141b
GLOBL ·zipperMergeSve(SB), (NOPTR+RODATA), $32
