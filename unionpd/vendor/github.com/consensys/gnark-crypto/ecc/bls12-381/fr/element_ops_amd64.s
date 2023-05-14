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
DATA q<>+0(SB)/8, $0xffffffff00000001
DATA q<>+8(SB)/8, $0x53bda402fffe5bfe
DATA q<>+16(SB)/8, $0x3339d80809a1d805
DATA q<>+24(SB)/8, $0x73eda753299d7d48
GLOBL q<>(SB), (RODATA+NOPTR), $32

// qInv0 q'[0]
DATA qInv0<>(SB)/8, $0xfffffffeffffffff
GLOBL qInv0<>(SB), (RODATA+NOPTR), $8

#define REDUCE(ra0, ra1, ra2, ra3, rb0, rb1, rb2, rb3) \
	MOVQ    ra0, rb0;        \
	SUBQ    q<>(SB), ra0;    \
	MOVQ    ra1, rb1;        \
	SBBQ    q<>+8(SB), ra1;  \
	MOVQ    ra2, rb2;        \
	SBBQ    q<>+16(SB), ra2; \
	MOVQ    ra3, rb3;        \
	SBBQ    q<>+24(SB), ra3; \
	CMOVQCS rb0, ra0;        \
	CMOVQCS rb1, ra1;        \
	CMOVQCS rb2, ra2;        \
	CMOVQCS rb3, ra3;        \

TEXT ·reduce(SB), NOSPLIT, $0-8
	MOVQ res+0(FP), AX
	MOVQ 0(AX), DX
	MOVQ 8(AX), CX
	MOVQ 16(AX), BX
	MOVQ 24(AX), SI

	// reduce element(DX,CX,BX,SI) using temp registers (DI,R8,R9,R10)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10)

	MOVQ DX, 0(AX)
	MOVQ CX, 8(AX)
	MOVQ BX, 16(AX)
	MOVQ SI, 24(AX)
	RET

// MulBy3(x *Element)
TEXT ·MulBy3(SB), NOSPLIT, $0-8
	MOVQ x+0(FP), AX
	MOVQ 0(AX), DX
	MOVQ 8(AX), CX
	MOVQ 16(AX), BX
	MOVQ 24(AX), SI
	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI

	// reduce element(DX,CX,BX,SI) using temp registers (DI,R8,R9,R10)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10)

	ADDQ 0(AX), DX
	ADCQ 8(AX), CX
	ADCQ 16(AX), BX
	ADCQ 24(AX), SI

	// reduce element(DX,CX,BX,SI) using temp registers (R11,R12,R13,R14)
	REDUCE(DX,CX,BX,SI,R11,R12,R13,R14)

	MOVQ DX, 0(AX)
	MOVQ CX, 8(AX)
	MOVQ BX, 16(AX)
	MOVQ SI, 24(AX)
	RET

// MulBy5(x *Element)
TEXT ·MulBy5(SB), NOSPLIT, $0-8
	MOVQ x+0(FP), AX
	MOVQ 0(AX), DX
	MOVQ 8(AX), CX
	MOVQ 16(AX), BX
	MOVQ 24(AX), SI
	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI

	// reduce element(DX,CX,BX,SI) using temp registers (DI,R8,R9,R10)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10)

	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI

	// reduce element(DX,CX,BX,SI) using temp registers (R11,R12,R13,R14)
	REDUCE(DX,CX,BX,SI,R11,R12,R13,R14)

	ADDQ 0(AX), DX
	ADCQ 8(AX), CX
	ADCQ 16(AX), BX
	ADCQ 24(AX), SI

	// reduce element(DX,CX,BX,SI) using temp registers (R15,DI,R8,R9)
	REDUCE(DX,CX,BX,SI,R15,DI,R8,R9)

	MOVQ DX, 0(AX)
	MOVQ CX, 8(AX)
	MOVQ BX, 16(AX)
	MOVQ SI, 24(AX)
	RET

// MulBy13(x *Element)
TEXT ·MulBy13(SB), NOSPLIT, $0-8
	MOVQ x+0(FP), AX
	MOVQ 0(AX), DX
	MOVQ 8(AX), CX
	MOVQ 16(AX), BX
	MOVQ 24(AX), SI
	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI

	// reduce element(DX,CX,BX,SI) using temp registers (DI,R8,R9,R10)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10)

	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI

	// reduce element(DX,CX,BX,SI) using temp registers (R11,R12,R13,R14)
	REDUCE(DX,CX,BX,SI,R11,R12,R13,R14)

	MOVQ DX, R11
	MOVQ CX, R12
	MOVQ BX, R13
	MOVQ SI, R14
	ADDQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI

	// reduce element(DX,CX,BX,SI) using temp registers (DI,R8,R9,R10)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10)

	ADDQ R11, DX
	ADCQ R12, CX
	ADCQ R13, BX
	ADCQ R14, SI

	// reduce element(DX,CX,BX,SI) using temp registers (DI,R8,R9,R10)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10)

	ADDQ 0(AX), DX
	ADCQ 8(AX), CX
	ADCQ 16(AX), BX
	ADCQ 24(AX), SI

	// reduce element(DX,CX,BX,SI) using temp registers (DI,R8,R9,R10)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10)

	MOVQ DX, 0(AX)
	MOVQ CX, 8(AX)
	MOVQ BX, 16(AX)
	MOVQ SI, 24(AX)
	RET

// Butterfly(a, b *Element) sets a = a + b; b = a - b
TEXT ·Butterfly(SB), NOSPLIT, $0-16
	MOVQ    a+0(FP), AX
	MOVQ    0(AX), CX
	MOVQ    8(AX), BX
	MOVQ    16(AX), SI
	MOVQ    24(AX), DI
	MOVQ    CX, R8
	MOVQ    BX, R9
	MOVQ    SI, R10
	MOVQ    DI, R11
	XORQ    AX, AX
	MOVQ    b+8(FP), DX
	ADDQ    0(DX), CX
	ADCQ    8(DX), BX
	ADCQ    16(DX), SI
	ADCQ    24(DX), DI
	SUBQ    0(DX), R8
	SBBQ    8(DX), R9
	SBBQ    16(DX), R10
	SBBQ    24(DX), R11
	MOVQ    $0xffffffff00000001, R12
	MOVQ    $0x53bda402fffe5bfe, R13
	MOVQ    $0x3339d80809a1d805, R14
	MOVQ    $0x73eda753299d7d48, R15
	CMOVQCC AX, R12
	CMOVQCC AX, R13
	CMOVQCC AX, R14
	CMOVQCC AX, R15
	ADDQ    R12, R8
	ADCQ    R13, R9
	ADCQ    R14, R10
	ADCQ    R15, R11
	MOVQ    R8, 0(DX)
	MOVQ    R9, 8(DX)
	MOVQ    R10, 16(DX)
	MOVQ    R11, 24(DX)

	// reduce element(CX,BX,SI,DI) using temp registers (R8,R9,R10,R11)
	REDUCE(CX,BX,SI,DI,R8,R9,R10,R11)

	MOVQ a+0(FP), AX
	MOVQ CX, 0(AX)
	MOVQ BX, 8(AX)
	MOVQ SI, 16(AX)
	MOVQ DI, 24(AX)
	RET
