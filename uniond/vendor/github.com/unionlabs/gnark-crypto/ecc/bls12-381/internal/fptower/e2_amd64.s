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
DATA q<>+0(SB)/8, $0xb9feffffffffaaab
DATA q<>+8(SB)/8, $0x1eabfffeb153ffff
DATA q<>+16(SB)/8, $0x6730d2a0f6b0f624
DATA q<>+24(SB)/8, $0x64774b84f38512bf
DATA q<>+32(SB)/8, $0x4b1ba7b6434bacd7
DATA q<>+40(SB)/8, $0x1a0111ea397fe69a
GLOBL q<>(SB), (RODATA+NOPTR), $48

// qInv0 q'[0]
DATA qInv0<>(SB)/8, $0x89f3fffcfffcfffd
GLOBL qInv0<>(SB), (RODATA+NOPTR), $8

#define REDUCE(ra0, ra1, ra2, ra3, ra4, ra5, rb0, rb1, rb2, rb3, rb4, rb5) \
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
	MOVQ    ra5, rb5;        \
	SBBQ    q<>+40(SB), ra5; \
	CMOVQCS rb0, ra0;        \
	CMOVQCS rb1, ra1;        \
	CMOVQCS rb2, ra2;        \
	CMOVQCS rb3, ra3;        \
	CMOVQCS rb4, ra4;        \
	CMOVQCS rb5, ra5;        \

TEXT ·addE2(SB), NOSPLIT, $0-24
	MOVQ x+8(FP), AX
	MOVQ 0(AX), BX
	MOVQ 8(AX), SI
	MOVQ 16(AX), DI
	MOVQ 24(AX), R8
	MOVQ 32(AX), R9
	MOVQ 40(AX), R10
	MOVQ y+16(FP), DX
	ADDQ 0(DX), BX
	ADCQ 8(DX), SI
	ADCQ 16(DX), DI
	ADCQ 24(DX), R8
	ADCQ 32(DX), R9
	ADCQ 40(DX), R10

	// reduce element(BX,SI,DI,R8,R9,R10) using temp registers (R11,R12,R13,R14,R15,s0-8(SP))
	REDUCE(BX,SI,DI,R8,R9,R10,R11,R12,R13,R14,R15,s0-8(SP))

	MOVQ res+0(FP), CX
	MOVQ BX, 0(CX)
	MOVQ SI, 8(CX)
	MOVQ DI, 16(CX)
	MOVQ R8, 24(CX)
	MOVQ R9, 32(CX)
	MOVQ R10, 40(CX)
	MOVQ 48(AX), BX
	MOVQ 56(AX), SI
	MOVQ 64(AX), DI
	MOVQ 72(AX), R8
	MOVQ 80(AX), R9
	MOVQ 88(AX), R10
	ADDQ 48(DX), BX
	ADCQ 56(DX), SI
	ADCQ 64(DX), DI
	ADCQ 72(DX), R8
	ADCQ 80(DX), R9
	ADCQ 88(DX), R10

	// reduce element(BX,SI,DI,R8,R9,R10) using temp registers (R11,R12,R13,R14,R15,s0-8(SP))
	REDUCE(BX,SI,DI,R8,R9,R10,R11,R12,R13,R14,R15,s0-8(SP))

	MOVQ BX, 48(CX)
	MOVQ SI, 56(CX)
	MOVQ DI, 64(CX)
	MOVQ R8, 72(CX)
	MOVQ R9, 80(CX)
	MOVQ R10, 88(CX)
	RET

TEXT ·doubleE2(SB), NOSPLIT, $0-16
	MOVQ res+0(FP), DX
	MOVQ x+8(FP), AX
	MOVQ 0(AX), CX
	MOVQ 8(AX), BX
	MOVQ 16(AX), SI
	MOVQ 24(AX), DI
	MOVQ 32(AX), R8
	MOVQ 40(AX), R9
	ADDQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI
	ADCQ R8, R8
	ADCQ R9, R9

	// reduce element(CX,BX,SI,DI,R8,R9) using temp registers (R10,R11,R12,R13,R14,R15)
	REDUCE(CX,BX,SI,DI,R8,R9,R10,R11,R12,R13,R14,R15)

	MOVQ CX, 0(DX)
	MOVQ BX, 8(DX)
	MOVQ SI, 16(DX)
	MOVQ DI, 24(DX)
	MOVQ R8, 32(DX)
	MOVQ R9, 40(DX)
	MOVQ 48(AX), CX
	MOVQ 56(AX), BX
	MOVQ 64(AX), SI
	MOVQ 72(AX), DI
	MOVQ 80(AX), R8
	MOVQ 88(AX), R9
	ADDQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI
	ADCQ R8, R8
	ADCQ R9, R9

	// reduce element(CX,BX,SI,DI,R8,R9) using temp registers (R10,R11,R12,R13,R14,R15)
	REDUCE(CX,BX,SI,DI,R8,R9,R10,R11,R12,R13,R14,R15)

	MOVQ CX, 48(DX)
	MOVQ BX, 56(DX)
	MOVQ SI, 64(DX)
	MOVQ DI, 72(DX)
	MOVQ R8, 80(DX)
	MOVQ R9, 88(DX)
	RET

TEXT ·subE2(SB), NOSPLIT, $0-24
	XORQ    R9, R9
	MOVQ    x+8(FP), R8
	MOVQ    0(R8), AX
	MOVQ    8(R8), DX
	MOVQ    16(R8), CX
	MOVQ    24(R8), BX
	MOVQ    32(R8), SI
	MOVQ    40(R8), DI
	MOVQ    y+16(FP), R8
	SUBQ    0(R8), AX
	SBBQ    8(R8), DX
	SBBQ    16(R8), CX
	SBBQ    24(R8), BX
	SBBQ    32(R8), SI
	SBBQ    40(R8), DI
	MOVQ    x+8(FP), R8
	MOVQ    $0xb9feffffffffaaab, R10
	MOVQ    $0x1eabfffeb153ffff, R11
	MOVQ    $0x6730d2a0f6b0f624, R12
	MOVQ    $0x64774b84f38512bf, R13
	MOVQ    $0x4b1ba7b6434bacd7, R14
	MOVQ    $0x1a0111ea397fe69a, R15
	CMOVQCC R9, R10
	CMOVQCC R9, R11
	CMOVQCC R9, R12
	CMOVQCC R9, R13
	CMOVQCC R9, R14
	CMOVQCC R9, R15
	ADDQ    R10, AX
	ADCQ    R11, DX
	ADCQ    R12, CX
	ADCQ    R13, BX
	ADCQ    R14, SI
	ADCQ    R15, DI
	MOVQ    res+0(FP), R10
	MOVQ    AX, 0(R10)
	MOVQ    DX, 8(R10)
	MOVQ    CX, 16(R10)
	MOVQ    BX, 24(R10)
	MOVQ    SI, 32(R10)
	MOVQ    DI, 40(R10)
	MOVQ    48(R8), AX
	MOVQ    56(R8), DX
	MOVQ    64(R8), CX
	MOVQ    72(R8), BX
	MOVQ    80(R8), SI
	MOVQ    88(R8), DI
	MOVQ    y+16(FP), R8
	SUBQ    48(R8), AX
	SBBQ    56(R8), DX
	SBBQ    64(R8), CX
	SBBQ    72(R8), BX
	SBBQ    80(R8), SI
	SBBQ    88(R8), DI
	MOVQ    $0xb9feffffffffaaab, R11
	MOVQ    $0x1eabfffeb153ffff, R12
	MOVQ    $0x6730d2a0f6b0f624, R13
	MOVQ    $0x64774b84f38512bf, R14
	MOVQ    $0x4b1ba7b6434bacd7, R15
	MOVQ    $0x1a0111ea397fe69a, R10
	CMOVQCC R9, R11
	CMOVQCC R9, R12
	CMOVQCC R9, R13
	CMOVQCC R9, R14
	CMOVQCC R9, R15
	CMOVQCC R9, R10
	ADDQ    R11, AX
	ADCQ    R12, DX
	ADCQ    R13, CX
	ADCQ    R14, BX
	ADCQ    R15, SI
	ADCQ    R10, DI
	MOVQ    res+0(FP), R8
	MOVQ    AX, 48(R8)
	MOVQ    DX, 56(R8)
	MOVQ    CX, 64(R8)
	MOVQ    BX, 72(R8)
	MOVQ    SI, 80(R8)
	MOVQ    DI, 88(R8)
	RET

TEXT ·negE2(SB), NOSPLIT, $0-16
	MOVQ  res+0(FP), DX
	MOVQ  x+8(FP), AX
	MOVQ  0(AX), BX
	MOVQ  8(AX), SI
	MOVQ  16(AX), DI
	MOVQ  24(AX), R8
	MOVQ  32(AX), R9
	MOVQ  40(AX), R10
	MOVQ  BX, AX
	ORQ   SI, AX
	ORQ   DI, AX
	ORQ   R8, AX
	ORQ   R9, AX
	ORQ   R10, AX
	TESTQ AX, AX
	JNE   l1
	MOVQ  AX, 0(DX)
	MOVQ  AX, 8(DX)
	MOVQ  AX, 16(DX)
	MOVQ  AX, 24(DX)
	MOVQ  AX, 32(DX)
	MOVQ  AX, 40(DX)
	JMP   l3

l1:
	MOVQ $0xb9feffffffffaaab, CX
	SUBQ BX, CX
	MOVQ CX, 0(DX)
	MOVQ $0x1eabfffeb153ffff, CX
	SBBQ SI, CX
	MOVQ CX, 8(DX)
	MOVQ $0x6730d2a0f6b0f624, CX
	SBBQ DI, CX
	MOVQ CX, 16(DX)
	MOVQ $0x64774b84f38512bf, CX
	SBBQ R8, CX
	MOVQ CX, 24(DX)
	MOVQ $0x4b1ba7b6434bacd7, CX
	SBBQ R9, CX
	MOVQ CX, 32(DX)
	MOVQ $0x1a0111ea397fe69a, CX
	SBBQ R10, CX
	MOVQ CX, 40(DX)

l3:
	MOVQ  x+8(FP), AX
	MOVQ  48(AX), BX
	MOVQ  56(AX), SI
	MOVQ  64(AX), DI
	MOVQ  72(AX), R8
	MOVQ  80(AX), R9
	MOVQ  88(AX), R10
	MOVQ  BX, AX
	ORQ   SI, AX
	ORQ   DI, AX
	ORQ   R8, AX
	ORQ   R9, AX
	ORQ   R10, AX
	TESTQ AX, AX
	JNE   l2
	MOVQ  AX, 48(DX)
	MOVQ  AX, 56(DX)
	MOVQ  AX, 64(DX)
	MOVQ  AX, 72(DX)
	MOVQ  AX, 80(DX)
	MOVQ  AX, 88(DX)
	RET

l2:
	MOVQ $0xb9feffffffffaaab, CX
	SUBQ BX, CX
	MOVQ CX, 48(DX)
	MOVQ $0x1eabfffeb153ffff, CX
	SBBQ SI, CX
	MOVQ CX, 56(DX)
	MOVQ $0x6730d2a0f6b0f624, CX
	SBBQ DI, CX
	MOVQ CX, 64(DX)
	MOVQ $0x64774b84f38512bf, CX
	SBBQ R8, CX
	MOVQ CX, 72(DX)
	MOVQ $0x4b1ba7b6434bacd7, CX
	SBBQ R9, CX
	MOVQ CX, 80(DX)
	MOVQ $0x1a0111ea397fe69a, CX
	SBBQ R10, CX
	MOVQ CX, 88(DX)
	RET

TEXT ·mulNonResE2(SB), NOSPLIT, $0-16
	XORQ    R15, R15
	MOVQ    x+8(FP), R14
	MOVQ    0(R14), AX
	MOVQ    8(R14), DX
	MOVQ    16(R14), CX
	MOVQ    24(R14), BX
	MOVQ    32(R14), SI
	MOVQ    40(R14), DI
	SUBQ    48(R14), AX
	SBBQ    56(R14), DX
	SBBQ    64(R14), CX
	SBBQ    72(R14), BX
	SBBQ    80(R14), SI
	SBBQ    88(R14), DI
	MOVQ    $0xb9feffffffffaaab, R8
	MOVQ    $0x1eabfffeb153ffff, R9
	MOVQ    $0x6730d2a0f6b0f624, R10
	MOVQ    $0x64774b84f38512bf, R11
	MOVQ    $0x4b1ba7b6434bacd7, R12
	MOVQ    $0x1a0111ea397fe69a, R13
	CMOVQCC R15, R8
	CMOVQCC R15, R9
	CMOVQCC R15, R10
	CMOVQCC R15, R11
	CMOVQCC R15, R12
	CMOVQCC R15, R13
	ADDQ    R8, AX
	ADCQ    R9, DX
	ADCQ    R10, CX
	ADCQ    R11, BX
	ADCQ    R12, SI
	ADCQ    R13, DI
	MOVQ    48(R14), R8
	MOVQ    56(R14), R9
	MOVQ    64(R14), R10
	MOVQ    72(R14), R11
	MOVQ    80(R14), R12
	MOVQ    88(R14), R13
	ADDQ    0(R14), R8
	ADCQ    8(R14), R9
	ADCQ    16(R14), R10
	ADCQ    24(R14), R11
	ADCQ    32(R14), R12
	ADCQ    40(R14), R13
	MOVQ    res+0(FP), R15
	MOVQ    AX, 0(R15)
	MOVQ    DX, 8(R15)
	MOVQ    CX, 16(R15)
	MOVQ    BX, 24(R15)
	MOVQ    SI, 32(R15)
	MOVQ    DI, 40(R15)

	// reduce element(R8,R9,R10,R11,R12,R13) using temp registers (AX,DX,CX,BX,SI,DI)
	REDUCE(R8,R9,R10,R11,R12,R13,AX,DX,CX,BX,SI,DI)

	MOVQ R8, 48(R15)
	MOVQ R9, 56(R15)
	MOVQ R10, 64(R15)
	MOVQ R11, 72(R15)
	MOVQ R12, 80(R15)
	MOVQ R13, 88(R15)
	RET

TEXT ·squareAdxE2(SB), $48-16
	NO_LOCAL_POINTERS

	// z.A0 = (x.A0 + x.A1) * (x.A0 - x.A1)
	// z.A1 = 2 * x.A0 * x.A1

	CMPB ·supportAdx(SB), $1
	JNE  l4

	// 2 * x.A0 * x.A1
	MOVQ x+8(FP), AX

	// 2 * x.A1[0] -> R14
	// 2 * x.A1[1] -> R15
	// 2 * x.A1[2] -> CX
	// 2 * x.A1[3] -> BX
	// 2 * x.A1[4] -> SI
	// 2 * x.A1[5] -> DI
	MOVQ 48(AX), R14
	MOVQ 56(AX), R15
	MOVQ 64(AX), CX
	MOVQ 72(AX), BX
	MOVQ 80(AX), SI
	MOVQ 88(AX), DI
	ADDQ R14, R14
	ADCQ R15, R15
	ADCQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI

	// A -> BP
	// t[0] -> R8
	// t[1] -> R9
	// t[2] -> R10
	// t[3] -> R11
	// t[4] -> R12
	// t[5] -> R13
	// clear the flags
	XORQ AX, AX
	MOVQ x+8(FP), DX
	MOVQ 0(DX), DX

	// (A,t[0])  := x[0]*y[0] + A
	MULXQ R14, R8, R9

	// (A,t[1])  := x[1]*y[0] + A
	MULXQ R15, AX, R10
	ADOXQ AX, R9

	// (A,t[2])  := x[2]*y[0] + A
	MULXQ CX, AX, R11
	ADOXQ AX, R10

	// (A,t[3])  := x[3]*y[0] + A
	MULXQ BX, AX, R12
	ADOXQ AX, R11

	// (A,t[4])  := x[4]*y[0] + A
	MULXQ SI, AX, R13
	ADOXQ AX, R12

	// (A,t[5])  := x[5]*y[0] + A
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ x+8(FP), DX
	MOVQ 8(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[1] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[1] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[1] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[1] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[1] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[1] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ x+8(FP), DX
	MOVQ 16(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[2] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[2] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[2] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[2] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[2] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[2] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ x+8(FP), DX
	MOVQ 24(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[3] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[3] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[3] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[3] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[3] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[3] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ x+8(FP), DX
	MOVQ 32(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[4] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[4] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[4] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[4] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[4] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[4] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ x+8(FP), DX
	MOVQ 40(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[5] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[5] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[5] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[5] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[5] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[5] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// reduce element(R8,R9,R10,R11,R12,R13) using temp registers (R14,R15,CX,BX,SI,DI)
	REDUCE(R8,R9,R10,R11,R12,R13,R14,R15,CX,BX,SI,DI)

	MOVQ x+8(FP), AX

	// x.A1[0] -> R14
	// x.A1[1] -> R15
	// x.A1[2] -> CX
	// x.A1[3] -> BX
	// x.A1[4] -> SI
	// x.A1[5] -> DI
	MOVQ 48(AX), R14
	MOVQ 56(AX), R15
	MOVQ 64(AX), CX
	MOVQ 72(AX), BX
	MOVQ 80(AX), SI
	MOVQ 88(AX), DI
	MOVQ res+0(FP), DX
	MOVQ R8, 48(DX)
	MOVQ R9, 56(DX)
	MOVQ R10, 64(DX)
	MOVQ R11, 72(DX)
	MOVQ R12, 80(DX)
	MOVQ R13, 88(DX)
	MOVQ R14, R8
	MOVQ R15, R9
	MOVQ CX, R10
	MOVQ BX, R11
	MOVQ SI, R12
	MOVQ DI, R13

	// Add(&x.A0, &x.A1)
	ADDQ 0(AX), R14
	ADCQ 8(AX), R15
	ADCQ 16(AX), CX
	ADCQ 24(AX), BX
	ADCQ 32(AX), SI
	ADCQ 40(AX), DI
	MOVQ R14, s0-8(SP)
	MOVQ R15, s1-16(SP)
	MOVQ CX, s2-24(SP)
	MOVQ BX, s3-32(SP)
	MOVQ SI, s4-40(SP)
	MOVQ DI, s5-48(SP)
	XORQ BP, BP

	// Sub(&x.A0, &x.A1)
	MOVQ    0(AX), R14
	MOVQ    8(AX), R15
	MOVQ    16(AX), CX
	MOVQ    24(AX), BX
	MOVQ    32(AX), SI
	MOVQ    40(AX), DI
	SUBQ    R8, R14
	SBBQ    R9, R15
	SBBQ    R10, CX
	SBBQ    R11, BX
	SBBQ    R12, SI
	SBBQ    R13, DI
	MOVQ    $0xb9feffffffffaaab, R8
	MOVQ    $0x1eabfffeb153ffff, R9
	MOVQ    $0x6730d2a0f6b0f624, R10
	MOVQ    $0x64774b84f38512bf, R11
	MOVQ    $0x4b1ba7b6434bacd7, R12
	MOVQ    $0x1a0111ea397fe69a, R13
	CMOVQCC BP, R8
	CMOVQCC BP, R9
	CMOVQCC BP, R10
	CMOVQCC BP, R11
	CMOVQCC BP, R12
	CMOVQCC BP, R13
	ADDQ    R8, R14
	ADCQ    R9, R15
	ADCQ    R10, CX
	ADCQ    R11, BX
	ADCQ    R12, SI
	ADCQ    R13, DI

	// A -> BP
	// t[0] -> R8
	// t[1] -> R9
	// t[2] -> R10
	// t[3] -> R11
	// t[4] -> R12
	// t[5] -> R13
	// clear the flags
	XORQ AX, AX
	MOVQ s0-8(SP), DX

	// (A,t[0])  := x[0]*y[0] + A
	MULXQ R14, R8, R9

	// (A,t[1])  := x[1]*y[0] + A
	MULXQ R15, AX, R10
	ADOXQ AX, R9

	// (A,t[2])  := x[2]*y[0] + A
	MULXQ CX, AX, R11
	ADOXQ AX, R10

	// (A,t[3])  := x[3]*y[0] + A
	MULXQ BX, AX, R12
	ADOXQ AX, R11

	// (A,t[4])  := x[4]*y[0] + A
	MULXQ SI, AX, R13
	ADOXQ AX, R12

	// (A,t[5])  := x[5]*y[0] + A
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ s1-16(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[1] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[1] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[1] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[1] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[1] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[1] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ s2-24(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[2] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[2] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[2] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[2] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[2] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[2] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ s3-32(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[3] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[3] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[3] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[3] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[3] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[3] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ s4-40(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[4] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[4] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[4] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[4] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[4] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[4] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ s5-48(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[5] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[5] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[5] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[5] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[5] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[5] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// reduce element(R8,R9,R10,R11,R12,R13) using temp registers (R14,R15,CX,BX,SI,DI)
	REDUCE(R8,R9,R10,R11,R12,R13,R14,R15,CX,BX,SI,DI)

	MOVQ res+0(FP), AX
	MOVQ R8, 0(AX)
	MOVQ R9, 8(AX)
	MOVQ R10, 16(AX)
	MOVQ R11, 24(AX)
	MOVQ R12, 32(AX)
	MOVQ R13, 40(AX)
	RET

l4:
	MOVQ res+0(FP), AX
	MOVQ AX, (SP)
	MOVQ x+8(FP), AX
	MOVQ AX, 8(SP)
	CALL ·squareGenericE2(SB)
	RET

TEXT ·mulAdxE2(SB), $96-24
	NO_LOCAL_POINTERS

	// var a, b, c fp.Element
	// a.Add(&x.A0, &x.A1)
	// b.Add(&y.A0, &y.A1)
	// a.Mul(&a, &b)
	// b.Mul(&x.A0, &y.A0)
	// c.Mul(&x.A1, &y.A1)
	// z.A1.Sub(&a, &b).Sub(&z.A1, &c)
	// z.A0.Sub(&b, &c)

	CMPB ·supportAdx(SB), $1
	JNE  l5
	MOVQ x+8(FP), AX
	MOVQ 48(AX), R14
	MOVQ 56(AX), R15
	MOVQ 64(AX), CX
	MOVQ 72(AX), BX
	MOVQ 80(AX), SI
	MOVQ 88(AX), DI

	// A -> BP
	// t[0] -> R8
	// t[1] -> R9
	// t[2] -> R10
	// t[3] -> R11
	// t[4] -> R12
	// t[5] -> R13
	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 48(DX), DX

	// (A,t[0])  := x[0]*y[0] + A
	MULXQ R14, R8, R9

	// (A,t[1])  := x[1]*y[0] + A
	MULXQ R15, AX, R10
	ADOXQ AX, R9

	// (A,t[2])  := x[2]*y[0] + A
	MULXQ CX, AX, R11
	ADOXQ AX, R10

	// (A,t[3])  := x[3]*y[0] + A
	MULXQ BX, AX, R12
	ADOXQ AX, R11

	// (A,t[4])  := x[4]*y[0] + A
	MULXQ SI, AX, R13
	ADOXQ AX, R12

	// (A,t[5])  := x[5]*y[0] + A
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 56(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[1] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[1] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[1] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[1] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[1] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[1] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 64(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[2] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[2] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[2] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[2] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[2] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[2] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 72(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[3] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[3] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[3] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[3] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[3] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[3] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 80(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[4] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[4] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[4] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[4] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[4] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[4] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 88(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[5] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[5] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[5] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[5] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[5] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[5] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// reduce element(R8,R9,R10,R11,R12,R13) using temp registers (R14,R15,CX,BX,SI,DI)
	REDUCE(R8,R9,R10,R11,R12,R13,R14,R15,CX,BX,SI,DI)

	MOVQ R8, s6-56(SP)
	MOVQ R9, s7-64(SP)
	MOVQ R10, s8-72(SP)
	MOVQ R11, s9-80(SP)
	MOVQ R12, s10-88(SP)
	MOVQ R13, s11-96(SP)
	MOVQ x+8(FP), AX
	MOVQ y+16(FP), DX
	MOVQ 48(AX), R14
	MOVQ 56(AX), R15
	MOVQ 64(AX), CX
	MOVQ 72(AX), BX
	MOVQ 80(AX), SI
	MOVQ 88(AX), DI
	ADDQ 0(AX), R14
	ADCQ 8(AX), R15
	ADCQ 16(AX), CX
	ADCQ 24(AX), BX
	ADCQ 32(AX), SI
	ADCQ 40(AX), DI
	MOVQ R14, s0-8(SP)
	MOVQ R15, s1-16(SP)
	MOVQ CX, s2-24(SP)
	MOVQ BX, s3-32(SP)
	MOVQ SI, s4-40(SP)
	MOVQ DI, s5-48(SP)
	MOVQ 0(DX), R14
	MOVQ 8(DX), R15
	MOVQ 16(DX), CX
	MOVQ 24(DX), BX
	MOVQ 32(DX), SI
	MOVQ 40(DX), DI
	ADDQ 48(DX), R14
	ADCQ 56(DX), R15
	ADCQ 64(DX), CX
	ADCQ 72(DX), BX
	ADCQ 80(DX), SI
	ADCQ 88(DX), DI

	// A -> BP
	// t[0] -> R8
	// t[1] -> R9
	// t[2] -> R10
	// t[3] -> R11
	// t[4] -> R12
	// t[5] -> R13
	// clear the flags
	XORQ AX, AX
	MOVQ s0-8(SP), DX

	// (A,t[0])  := x[0]*y[0] + A
	MULXQ R14, R8, R9

	// (A,t[1])  := x[1]*y[0] + A
	MULXQ R15, AX, R10
	ADOXQ AX, R9

	// (A,t[2])  := x[2]*y[0] + A
	MULXQ CX, AX, R11
	ADOXQ AX, R10

	// (A,t[3])  := x[3]*y[0] + A
	MULXQ BX, AX, R12
	ADOXQ AX, R11

	// (A,t[4])  := x[4]*y[0] + A
	MULXQ SI, AX, R13
	ADOXQ AX, R12

	// (A,t[5])  := x[5]*y[0] + A
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ s1-16(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[1] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[1] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[1] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[1] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[1] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[1] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ s2-24(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[2] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[2] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[2] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[2] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[2] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[2] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ s3-32(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[3] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[3] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[3] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[3] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[3] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[3] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ s4-40(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[4] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[4] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[4] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[4] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[4] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[4] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ s5-48(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[5] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[5] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[5] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[5] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[5] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[5] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// reduce element(R8,R9,R10,R11,R12,R13) using temp registers (R14,R15,CX,BX,SI,DI)
	REDUCE(R8,R9,R10,R11,R12,R13,R14,R15,CX,BX,SI,DI)

	MOVQ R8, s0-8(SP)
	MOVQ R9, s1-16(SP)
	MOVQ R10, s2-24(SP)
	MOVQ R11, s3-32(SP)
	MOVQ R12, s4-40(SP)
	MOVQ R13, s5-48(SP)
	MOVQ x+8(FP), AX
	MOVQ 0(AX), R14
	MOVQ 8(AX), R15
	MOVQ 16(AX), CX
	MOVQ 24(AX), BX
	MOVQ 32(AX), SI
	MOVQ 40(AX), DI

	// A -> BP
	// t[0] -> R8
	// t[1] -> R9
	// t[2] -> R10
	// t[3] -> R11
	// t[4] -> R12
	// t[5] -> R13
	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 0(DX), DX

	// (A,t[0])  := x[0]*y[0] + A
	MULXQ R14, R8, R9

	// (A,t[1])  := x[1]*y[0] + A
	MULXQ R15, AX, R10
	ADOXQ AX, R9

	// (A,t[2])  := x[2]*y[0] + A
	MULXQ CX, AX, R11
	ADOXQ AX, R10

	// (A,t[3])  := x[3]*y[0] + A
	MULXQ BX, AX, R12
	ADOXQ AX, R11

	// (A,t[4])  := x[4]*y[0] + A
	MULXQ SI, AX, R13
	ADOXQ AX, R12

	// (A,t[5])  := x[5]*y[0] + A
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 8(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[1] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[1] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[1] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[1] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[1] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[1] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 16(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[2] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[2] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[2] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[2] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[2] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[2] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 24(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[3] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[3] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[3] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[3] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[3] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[3] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 32(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[4] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[4] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[4] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[4] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[4] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[4] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 40(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[5] + A
	MULXQ R14, AX, BP
	ADOXQ AX, R8

	// (A,t[1])  := t[1] + x[1]*y[5] + A
	ADCXQ BP, R9
	MULXQ R15, AX, BP
	ADOXQ AX, R9

	// (A,t[2])  := t[2] + x[2]*y[5] + A
	ADCXQ BP, R10
	MULXQ CX, AX, BP
	ADOXQ AX, R10

	// (A,t[3])  := t[3] + x[3]*y[5] + A
	ADCXQ BP, R11
	MULXQ BX, AX, BP
	ADOXQ AX, R11

	// (A,t[4])  := t[4] + x[4]*y[5] + A
	ADCXQ BP, R12
	MULXQ SI, AX, BP
	ADOXQ AX, R12

	// (A,t[5])  := t[5] + x[5]*y[5] + A
	ADCXQ BP, R13
	MULXQ DI, AX, BP
	ADOXQ AX, R13

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP
	PUSHQ BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R8, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R8, AX
	MOVQ  BP, R8
	POPQ  BP

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R9, R8
	MULXQ q<>+8(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R10, R9
	MULXQ q<>+16(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R11, R10
	MULXQ q<>+24(SB), AX, R11
	ADOXQ AX, R10

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R12, R11
	MULXQ q<>+32(SB), AX, R12
	ADOXQ AX, R11

	// (C,t[4]) := t[5] + m*q[5] + C
	ADCXQ R13, R12
	MULXQ q<>+40(SB), AX, R13
	ADOXQ AX, R12

	// t[5] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R13
	ADOXQ BP, R13

	// reduce element(R8,R9,R10,R11,R12,R13) using temp registers (R14,R15,CX,BX,SI,DI)
	REDUCE(R8,R9,R10,R11,R12,R13,R14,R15,CX,BX,SI,DI)

	XORQ    DX, DX
	MOVQ    s0-8(SP), R14
	MOVQ    s1-16(SP), R15
	MOVQ    s2-24(SP), CX
	MOVQ    s3-32(SP), BX
	MOVQ    s4-40(SP), SI
	MOVQ    s5-48(SP), DI
	SUBQ    R8, R14
	SBBQ    R9, R15
	SBBQ    R10, CX
	SBBQ    R11, BX
	SBBQ    R12, SI
	SBBQ    R13, DI
	MOVQ    R8, s0-8(SP)
	MOVQ    R9, s1-16(SP)
	MOVQ    R10, s2-24(SP)
	MOVQ    R11, s3-32(SP)
	MOVQ    R12, s4-40(SP)
	MOVQ    R13, s5-48(SP)
	MOVQ    $0xb9feffffffffaaab, R8
	MOVQ    $0x1eabfffeb153ffff, R9
	MOVQ    $0x6730d2a0f6b0f624, R10
	MOVQ    $0x64774b84f38512bf, R11
	MOVQ    $0x4b1ba7b6434bacd7, R12
	MOVQ    $0x1a0111ea397fe69a, R13
	CMOVQCC DX, R8
	CMOVQCC DX, R9
	CMOVQCC DX, R10
	CMOVQCC DX, R11
	CMOVQCC DX, R12
	CMOVQCC DX, R13
	ADDQ    R8, R14
	ADCQ    R9, R15
	ADCQ    R10, CX
	ADCQ    R11, BX
	ADCQ    R12, SI
	ADCQ    R13, DI
	SUBQ    s6-56(SP), R14
	SBBQ    s7-64(SP), R15
	SBBQ    s8-72(SP), CX
	SBBQ    s9-80(SP), BX
	SBBQ    s10-88(SP), SI
	SBBQ    s11-96(SP), DI
	MOVQ    $0xb9feffffffffaaab, R8
	MOVQ    $0x1eabfffeb153ffff, R9
	MOVQ    $0x6730d2a0f6b0f624, R10
	MOVQ    $0x64774b84f38512bf, R11
	MOVQ    $0x4b1ba7b6434bacd7, R12
	MOVQ    $0x1a0111ea397fe69a, R13
	CMOVQCC DX, R8
	CMOVQCC DX, R9
	CMOVQCC DX, R10
	CMOVQCC DX, R11
	CMOVQCC DX, R12
	CMOVQCC DX, R13
	ADDQ    R8, R14
	ADCQ    R9, R15
	ADCQ    R10, CX
	ADCQ    R11, BX
	ADCQ    R12, SI
	ADCQ    R13, DI
	MOVQ    z+0(FP), AX
	MOVQ    R14, 48(AX)
	MOVQ    R15, 56(AX)
	MOVQ    CX, 64(AX)
	MOVQ    BX, 72(AX)
	MOVQ    SI, 80(AX)
	MOVQ    DI, 88(AX)
	MOVQ    s0-8(SP), R8
	MOVQ    s1-16(SP), R9
	MOVQ    s2-24(SP), R10
	MOVQ    s3-32(SP), R11
	MOVQ    s4-40(SP), R12
	MOVQ    s5-48(SP), R13
	SUBQ    s6-56(SP), R8
	SBBQ    s7-64(SP), R9
	SBBQ    s8-72(SP), R10
	SBBQ    s9-80(SP), R11
	SBBQ    s10-88(SP), R12
	SBBQ    s11-96(SP), R13
	MOVQ    $0xb9feffffffffaaab, R14
	MOVQ    $0x1eabfffeb153ffff, R15
	MOVQ    $0x6730d2a0f6b0f624, CX
	MOVQ    $0x64774b84f38512bf, BX
	MOVQ    $0x4b1ba7b6434bacd7, SI
	MOVQ    $0x1a0111ea397fe69a, DI
	CMOVQCC DX, R14
	CMOVQCC DX, R15
	CMOVQCC DX, CX
	CMOVQCC DX, BX
	CMOVQCC DX, SI
	CMOVQCC DX, DI
	ADDQ    R14, R8
	ADCQ    R15, R9
	ADCQ    CX, R10
	ADCQ    BX, R11
	ADCQ    SI, R12
	ADCQ    DI, R13
	MOVQ    R8, 0(AX)
	MOVQ    R9, 8(AX)
	MOVQ    R10, 16(AX)
	MOVQ    R11, 24(AX)
	MOVQ    R12, 32(AX)
	MOVQ    R13, 40(AX)
	RET

l5:
	MOVQ z+0(FP), AX
	MOVQ AX, (SP)
	MOVQ x+8(FP), AX
	MOVQ AX, 8(SP)
	MOVQ y+16(FP), AX
	MOVQ AX, 16(SP)
	CALL ·mulGenericE2(SB)
	RET
