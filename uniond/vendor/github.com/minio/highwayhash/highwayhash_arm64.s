//
// Minio Cloud Storage, (C) 2017 Minio, Inc.
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

// Use github.com/minio/asm2plan9s on this file to assemble ARM instructions to
// the opcodes of their Plan9 equivalents

#include "textflag.h"

#define REDUCE_MOD(x0, x1, x2, x3, tmp0, tmp1, y0, y1) \
	MOVD $0x3FFFFFFFFFFFFFFF, tmp0 \
	AND  tmp0, x3                  \
	MOVD x2, y0                    \
	MOVD x3, y1                    \
	                               \
	MOVD x2, tmp0                  \
	MOVD x3, tmp1                  \
	LSL  $1, tmp1                  \
	LSR  $63, tmp0                 \
	MOVD tmp1, x3                  \
	ORR  tmp0, x3                  \
	                               \
	LSL  $1, x2                    \
	                               \
	MOVD y0, tmp0                  \
	MOVD y1, tmp1                  \
	LSL  $2, tmp1                  \
	LSR  $62, tmp0                 \
	MOVD tmp1, y1                  \
	ORR  tmp0, y1                  \
	                               \
	LSL  $2, y0                    \
	                               \
	EOR  x0, y0                    \
	EOR  x2, y0                    \
	EOR  x1, y1                    \
	EOR  x3, y1

#define UPDATE(MSG1, MSG2) \
	\ // Add message
	VADD MSG1.D2, V2.D2, V2.D2                                          \
	VADD MSG2.D2, V3.D2, V3.D2                                          \
	                                                                    \
	\ // v1 += mul0
	VADD V4.D2, V2.D2, V2.D2                                            \
	VADD V5.D2, V3.D2, V3.D2                                            \
	                                                                    \
	\ // First pair of multiplies
	VTBL V29.B16, [V0.B16, V1.B16], V10.B16                             \
	VTBL V30.B16, [V2.B16, V3.B16], V11.B16                             \
	                                                                    \
	\ // VUMULL  V10.S2, V11.S2, V12.D2 /* assembler support missing */
	\ // VUMULL2 V10.S4, V11.S4, V13.D2 /* assembler support missing */
	WORD $0x2eaac16c                                                    \ // umull  v12.2d, v11.2s, v10.2s
	WORD $0x6eaac16d                                                    \ // umull2 v13.2d, v11.4s, v10.4s
	                                                                    \
	\ // v0 += mul1
	VADD V6.D2, V0.D2, V0.D2                                            \
	VADD V7.D2, V1.D2, V1.D2                                            \
	                                                                    \
	\ // Second pair of multiplies
	VTBL V29.B16, [V2.B16, V3.B16], V15.B16                             \
	VTBL V30.B16, [V0.B16, V1.B16], V14.B16                             \
	                                                                    \
	\ // EOR multiplication result in
	VEOR V12.B16, V4.B16, V4.B16                                        \
	VEOR V13.B16, V5.B16, V5.B16                                        \
	                                                                    \
	\ // VUMULL  V14.S2, V15.S2, V16.D2 /* assembler support missing */
	\ // VUMULL2 V14.S4, V15.S4, V17.D2 /* assembler support missing */
	WORD $0x2eaec1f0                                                    \ // umull  v16.2d, v15.2s, v14.2s
	WORD $0x6eaec1f1                                                    \ // umull2 v17.2d, v15.4s, v14.4s
	                                                                    \
	\ // First pair of zipper-merges
	VTBL V28.B16, [V2.B16], V18.B16                                     \
	VADD V18.D2, V0.D2, V0.D2                                           \
	VTBL V28.B16, [V3.B16], V19.B16                                     \
	VADD V19.D2, V1.D2, V1.D2                                           \
	                                                                    \
	\ // Second pair of zipper-merges
	VTBL V28.B16, [V0.B16], V20.B16                                     \
	VADD V20.D2, V2.D2, V2.D2                                           \
	VTBL V28.B16, [V1.B16], V21.B16                                     \
	VADD V21.D2, V3.D2, V3.D2                                           \
	                                                                    \
	\ // EOR multiplication result in
	VEOR V16.B16, V6.B16, V6.B16                                        \
	VEOR V17.B16, V7.B16, V7.B16

// func initializeArm64(state *[16]uint64, key []byte)
TEXT ·initializeArm64(SB), NOSPLIT, $0
	MOVD state+0(FP), R0
	MOVD key_base+8(FP), R1

	VLD1 (R1), [V1.S4, V2.S4]

	VREV64 V1.S4, V3.S4
	VREV64 V2.S4, V4.S4

	MOVD $·asmConstants(SB), R3
	VLD1 (R3), [V5.S4, V6.S4, V7.S4, V8.S4]
	VEOR V5.B16, V1.B16, V1.B16
	VEOR V6.B16, V2.B16, V2.B16
	VEOR V7.B16, V3.B16, V3.B16
	VEOR V8.B16, V4.B16, V4.B16

	VST1.P [V1.D2, V2.D2, V3.D2, V4.D2], 64(R0)
	VST1   [V5.D2, V6.D2, V7.D2, V8.D2], (R0)
	RET

TEXT ·updateArm64(SB), NOSPLIT, $0
	MOVD state+0(FP), R0
	MOVD msg_base+8(FP), R1
	MOVD msg_len+16(FP), R2 // length of message
	SUBS $32, R2
	BMI  complete

	// Definition of registers
	//  v0 = v0.lo
	//  v1 = v0.hi
	//  v2 = v1.lo
	//  v3 = v1.hi
	//  v4 = mul0.lo
	//  v5 = mul0.hi
	//  v6 = mul1.lo
	//  v7 = mul1.hi

	// Load zipper merge constants table pointer
	MOVD $·asmZipperMerge(SB), R3

	// and load zipper merge constants into v28, v29, and v30
	VLD1 (R3), [V28.B16, V29.B16, V30.B16]

	VLD1.P 64(R0), [V0.D2, V1.D2, V2.D2, V3.D2]
	VLD1   (R0), [V4.D2, V5.D2, V6.D2, V7.D2]
	SUBS   $64, R0

loop:
	// Main loop
	VLD1.P 32(R1), [V26.S4, V27.S4]

	UPDATE(V26, V27)

	SUBS $32, R2
	BPL  loop

	// Store result
	VST1.P [V0.D2, V1.D2, V2.D2, V3.D2], 64(R0)
	VST1   [V4.D2, V5.D2, V6.D2, V7.D2], (R0)

complete:
	RET

// func finalizeArm64(out []byte, state *[16]uint64)
TEXT ·finalizeArm64(SB), NOSPLIT, $0-32
	MOVD state+24(FP), R0
	MOVD out_base+0(FP), R1
	MOVD out_len+8(FP), R2

	// Load zipper merge constants table pointer
	MOVD $·asmZipperMerge(SB), R3

	// and load zipper merge constants into v28, v29, and v30
	VLD1 (R3), [V28.B16, V29.B16, V30.B16]

	VLD1.P 64(R0), [V0.D2, V1.D2, V2.D2, V3.D2]
	VLD1   (R0), [V4.D2, V5.D2, V6.D2, V7.D2]
	SUB    $64, R0

	VREV64 V1.S4, V26.S4
	VREV64 V0.S4, V27.S4
	UPDATE(V26, V27)

	VREV64 V1.S4, V26.S4
	VREV64 V0.S4, V27.S4
	UPDATE(V26, V27)

	VREV64 V1.S4, V26.S4
	VREV64 V0.S4, V27.S4
	UPDATE(V26, V27)

	VREV64 V1.S4, V26.S4
	VREV64 V0.S4, V27.S4
	UPDATE(V26, V27)

	CMP $8, R2
	BEQ skipUpdate // Just 4 rounds for 64-bit checksum

	VREV64 V1.S4, V26.S4
	VREV64 V0.S4, V27.S4
	UPDATE(V26, V27)

	VREV64 V1.S4, V26.S4
	VREV64 V0.S4, V27.S4
	UPDATE(V26, V27)

	CMP $16, R2
	BEQ skipUpdate // 6 rounds for 128-bit checksum

	VREV64 V1.S4, V26.S4
	VREV64 V0.S4, V27.S4
	UPDATE(V26, V27)

	VREV64 V1.S4, V26.S4
	VREV64 V0.S4, V27.S4
	UPDATE(V26, V27)

	VREV64 V1.S4, V26.S4
	VREV64 V0.S4, V27.S4
	UPDATE(V26, V27)

	VREV64 V1.S4, V26.S4
	VREV64 V0.S4, V27.S4
	UPDATE(V26, V27)

skipUpdate:
	// Store result
	VST1.P [V0.D2, V1.D2, V2.D2, V3.D2], 64(R0)
	VST1   [V4.D2, V5.D2, V6.D2, V7.D2], (R0)
	SUB    $64, R0

	CMP $8, R2
	BEQ hash64
	CMP $16, R2
	BEQ hash128

	// 256-bit checksum
	MOVD 0*8(R0), R8
	MOVD 1*8(R0), R9
	MOVD 4*8(R0), R10
	MOVD 5*8(R0), R11
	MOVD 8*8(R0), R4
	MOVD 9*8(R0), R5
	MOVD 12*8(R0), R6
	MOVD 13*8(R0), R7
	ADD  R4, R8
	ADD  R5, R9
	ADD  R6, R10
	ADD  R7, R11

	REDUCE_MOD(R8, R9, R10, R11, R4, R5, R6, R7)
	MOVD R6, 0(R1)
	MOVD R7, 8(R1)

	MOVD 2*8(R0), R8
	MOVD 3*8(R0), R9
	MOVD 6*8(R0), R10
	MOVD 7*8(R0), R11
	MOVD 10*8(R0), R4
	MOVD 11*8(R0), R5
	MOVD 14*8(R0), R6
	MOVD 15*8(R0), R7
	ADD  R4, R8
	ADD  R5, R9
	ADD  R6, R10
	ADD  R7, R11

	REDUCE_MOD(R8, R9, R10, R11, R4, R5, R6, R7)
	MOVD R6, 16(R1)
	MOVD R7, 24(R1)
	RET

hash128:
	MOVD 0*8(R0), R8
	MOVD 1*8(R0), R9
	MOVD 6*8(R0), R10
	MOVD 7*8(R0), R11
	ADD  R10, R8
	ADD  R11, R9
	MOVD 8*8(R0), R10
	MOVD 9*8(R0), R11
	ADD  R10, R8
	ADD  R11, R9
	MOVD 14*8(R0), R10
	MOVD 15*8(R0), R11
	ADD  R10, R8
	ADD  R11, R9
	MOVD R8, 0(R1)
	MOVD R9, 8(R1)
	RET

hash64:
	MOVD 0*8(R0), R4
	MOVD 4*8(R0), R5
	MOVD 8*8(R0), R6
	MOVD 12*8(R0), R7
	ADD  R5, R4
	ADD  R7, R6
	ADD  R6, R4
	MOVD R4, (R1)
	RET

DATA ·asmConstants+0x00(SB)/8, $0xdbe6d5d5fe4cce2f
DATA ·asmConstants+0x08(SB)/8, $0xa4093822299f31d0
DATA ·asmConstants+0x10(SB)/8, $0x13198a2e03707344
DATA ·asmConstants+0x18(SB)/8, $0x243f6a8885a308d3
DATA ·asmConstants+0x20(SB)/8, $0x3bd39e10cb0ef593
DATA ·asmConstants+0x28(SB)/8, $0xc0acf169b5f18a8c
DATA ·asmConstants+0x30(SB)/8, $0xbe5466cf34e90c6c
DATA ·asmConstants+0x38(SB)/8, $0x452821e638d01377
GLOBL ·asmConstants(SB), 8, $64

// Constants for TBL instructions
DATA ·asmZipperMerge+0x0(SB)/8, $0x000f010e05020c03 // zipper merge constant
DATA ·asmZipperMerge+0x8(SB)/8, $0x070806090d0a040b
DATA ·asmZipperMerge+0x10(SB)/8, $0x0f0e0d0c07060504 // setup first register for multiply
DATA ·asmZipperMerge+0x18(SB)/8, $0x1f1e1d1c17161514
DATA ·asmZipperMerge+0x20(SB)/8, $0x0b0a090803020100 // setup second register for multiply
DATA ·asmZipperMerge+0x28(SB)/8, $0x1b1a191813121110
GLOBL ·asmZipperMerge(SB), 8, $48
