// +build !purego

// Copyright 2020 ConsenSys Software Inc.
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

#include "textflag.h"
#include "funcdata.h"

// modulus q
DATA q<>+0(SB)/8, $0x6fe802ff40300001
DATA q<>+8(SB)/8, $0x421ee5da52bde502
DATA q<>+16(SB)/8, $0xdec1d01aa27a1ae0
DATA q<>+24(SB)/8, $0xd3f7498be97c5eaf
DATA q<>+32(SB)/8, $0x04c23a02b586d650
GLOBL q<>(SB), (RODATA+NOPTR), $40

// qInv0 q'[0]
DATA qInv0<>(SB)/8, $0x702ff9ff402fffff
GLOBL qInv0<>(SB), (RODATA+NOPTR), $8

#define REDUCE(ra0, ra1, ra2, ra3, ra4, rb0, rb1, rb2, rb3, rb4) \
	MOVQ    ra0, rb0;        \
	SUBQ    q<>(SB), ra0;    \
	MOVQ    ra1, rb1;        \
	SBBQ    q<>+8(SB), ra1;  \
	MOVQ    ra2, rb2;        \
	SBBQ    q<>+16(SB), ra2; \
	MOVQ    ra3, rb3;        \
	SBBQ    q<>+24(SB), ra3; \
	MOVQ    ra4, rb4;        \
	SBBQ    q<>+32(SB), ra4; \
	CMOVQCS rb0, ra0;        \
	CMOVQCS rb1, ra1;        \
	CMOVQCS rb2, ra2;        \
	CMOVQCS rb3, ra3;        \
	CMOVQCS rb4, ra4;        \

TEXT ·reduce(SB), NOSPLIT, $0-8
	MOVQ res+0(FP), AX
	MOVQ 0(AX), DX
	MOVQ 8(AX), CX
	MOVQ 16(AX), BX
	MOVQ 24(AX), SI
	MOVQ 32(AX), DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R8,R9,R10,R11,R12)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10,R11,R12)

	MOVQ DX, 0(AX)
	MOVQ CX, 8(AX)
	MOVQ BX, 16(AX)
	MOVQ SI, 24(AX)
	MOVQ DI, 32(AX)
	RET

// MulBy3(x *Element)
TEXT ·MulBy3(SB), NOSPLIT, $0-8
	MOVQ x+0(FP), AX
	MOVQ 0(AX), DX
	MOVQ 8(AX), CX
	MOVQ 16(AX), BX
	MOVQ 24(AX), SI
	MOVQ 32(AX), DI
	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R8,R9,R10,R11,R12)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10,R11,R12)

	ADDQ 0(AX), DX
	ADCQ 8(AX), CX
	ADCQ 16(AX), BX
	ADCQ 24(AX), SI
	ADCQ 32(AX), DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R13,R14,R15,R8,R9)
	REDUCE(DX,CX,BX,SI,DI,R13,R14,R15,R8,R9)

	MOVQ DX, 0(AX)
	MOVQ CX, 8(AX)
	MOVQ BX, 16(AX)
	MOVQ SI, 24(AX)
	MOVQ DI, 32(AX)
	RET

// MulBy5(x *Element)
TEXT ·MulBy5(SB), NOSPLIT, $0-8
	MOVQ x+0(FP), AX
	MOVQ 0(AX), DX
	MOVQ 8(AX), CX
	MOVQ 16(AX), BX
	MOVQ 24(AX), SI
	MOVQ 32(AX), DI
	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R8,R9,R10,R11,R12)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10,R11,R12)

	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R13,R14,R15,R8,R9)
	REDUCE(DX,CX,BX,SI,DI,R13,R14,R15,R8,R9)

	ADDQ 0(AX), DX
	ADCQ 8(AX), CX
	ADCQ 16(AX), BX
	ADCQ 24(AX), SI
	ADCQ 32(AX), DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R10,R11,R12,R13,R14)
	REDUCE(DX,CX,BX,SI,DI,R10,R11,R12,R13,R14)

	MOVQ DX, 0(AX)
	MOVQ CX, 8(AX)
	MOVQ BX, 16(AX)
	MOVQ SI, 24(AX)
	MOVQ DI, 32(AX)
	RET

// MulBy13(x *Element)
TEXT ·MulBy13(SB), $16-8
	MOVQ x+0(FP), AX
	MOVQ 0(AX), DX
	MOVQ 8(AX), CX
	MOVQ 16(AX), BX
	MOVQ 24(AX), SI
	MOVQ 32(AX), DI
	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R8,R9,R10,R11,R12)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10,R11,R12)

	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R13,R14,R15,s0-8(SP),s1-16(SP))
	REDUCE(DX,CX,BX,SI,DI,R13,R14,R15,s0-8(SP),s1-16(SP))

	MOVQ DX, R13
	MOVQ CX, R14
	MOVQ BX, R15
	MOVQ SI, s0-8(SP)
	MOVQ DI, s1-16(SP)
	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R8,R9,R10,R11,R12)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10,R11,R12)

	ADDQ R13, DX
	ADCQ R14, CX
	ADCQ R15, BX
	ADCQ s0-8(SP), SI
	ADCQ s1-16(SP), DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R8,R9,R10,R11,R12)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10,R11,R12)

	ADDQ 0(AX), DX
	ADCQ 8(AX), CX
	ADCQ 16(AX), BX
	ADCQ 24(AX), SI
	ADCQ 32(AX), DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R8,R9,R10,R11,R12)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10,R11,R12)

	MOVQ DX, 0(AX)
	MOVQ CX, 8(AX)
	MOVQ BX, 16(AX)
	MOVQ SI, 24(AX)
	MOVQ DI, 32(AX)
	RET

// Butterfly(a, b *Element) sets a = a + b; b = a - b
TEXT ·Butterfly(SB), $24-16
	MOVQ    a+0(FP), AX
	MOVQ    0(AX), CX
	MOVQ    8(AX), BX
	MOVQ    16(AX), SI
	MOVQ    24(AX), DI
	MOVQ    32(AX), R8
	MOVQ    CX, R9
	MOVQ    BX, R10
	MOVQ    SI, R11
	MOVQ    DI, R12
	MOVQ    R8, R13
	XORQ    AX, AX
	MOVQ    b+8(FP), DX
	ADDQ    0(DX), CX
	ADCQ    8(DX), BX
	ADCQ    16(DX), SI
	ADCQ    24(DX), DI
	ADCQ    32(DX), R8
	SUBQ    0(DX), R9
	SBBQ    8(DX), R10
	SBBQ    16(DX), R11
	SBBQ    24(DX), R12
	SBBQ    32(DX), R13
	MOVQ    CX, R14
	MOVQ    BX, R15
	MOVQ    SI, s0-8(SP)
	MOVQ    DI, s1-16(SP)
	MOVQ    R8, s2-24(SP)
	MOVQ    $0x6fe802ff40300001, CX
	MOVQ    $0x421ee5da52bde502, BX
	MOVQ    $0xdec1d01aa27a1ae0, SI
	MOVQ    $0xd3f7498be97c5eaf, DI
	MOVQ    $0x04c23a02b586d650, R8
	CMOVQCC AX, CX
	CMOVQCC AX, BX
	CMOVQCC AX, SI
	CMOVQCC AX, DI
	CMOVQCC AX, R8
	ADDQ    CX, R9
	ADCQ    BX, R10
	ADCQ    SI, R11
	ADCQ    DI, R12
	ADCQ    R8, R13
	MOVQ    R14, CX
	MOVQ    R15, BX
	MOVQ    s0-8(SP), SI
	MOVQ    s1-16(SP), DI
	MOVQ    s2-24(SP), R8
	MOVQ    R9, 0(DX)
	MOVQ    R10, 8(DX)
	MOVQ    R11, 16(DX)
	MOVQ    R12, 24(DX)
	MOVQ    R13, 32(DX)

	// reduce element(CX,BX,SI,DI,R8) using temp registers (R9,R10,R11,R12,R13)
	REDUCE(CX,BX,SI,DI,R8,R9,R10,R11,R12,R13)

	MOVQ a+0(FP), AX
	MOVQ CX, 0(AX)
	MOVQ BX, 8(AX)
	MOVQ SI, 16(AX)
	MOVQ DI, 24(AX)
	MOVQ R8, 32(AX)
	RET
