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

TEXT ·addE2(SB), NOSPLIT, $0-24
	MOVQ x+8(FP), AX
	MOVQ 0(AX), BX
	MOVQ 8(AX), SI
	MOVQ 16(AX), DI
	MOVQ 24(AX), R8
	MOVQ 32(AX), R9
	MOVQ y+16(FP), DX
	ADDQ 0(DX), BX
	ADCQ 8(DX), SI
	ADCQ 16(DX), DI
	ADCQ 24(DX), R8
	ADCQ 32(DX), R9

	// reduce element(BX,SI,DI,R8,R9) using temp registers (R10,R11,R12,R13,R14)
	REDUCE(BX,SI,DI,R8,R9,R10,R11,R12,R13,R14)

	MOVQ res+0(FP), CX
	MOVQ BX, 0(CX)
	MOVQ SI, 8(CX)
	MOVQ DI, 16(CX)
	MOVQ R8, 24(CX)
	MOVQ R9, 32(CX)
	MOVQ 40(AX), BX
	MOVQ 48(AX), SI
	MOVQ 56(AX), DI
	MOVQ 64(AX), R8
	MOVQ 72(AX), R9
	ADDQ 40(DX), BX
	ADCQ 48(DX), SI
	ADCQ 56(DX), DI
	ADCQ 64(DX), R8
	ADCQ 72(DX), R9

	// reduce element(BX,SI,DI,R8,R9) using temp registers (R15,R10,R11,R12,R13)
	REDUCE(BX,SI,DI,R8,R9,R15,R10,R11,R12,R13)

	MOVQ BX, 40(CX)
	MOVQ SI, 48(CX)
	MOVQ DI, 56(CX)
	MOVQ R8, 64(CX)
	MOVQ R9, 72(CX)
	RET

TEXT ·doubleE2(SB), NOSPLIT, $0-16
	MOVQ res+0(FP), DX
	MOVQ x+8(FP), AX
	MOVQ 0(AX), CX
	MOVQ 8(AX), BX
	MOVQ 16(AX), SI
	MOVQ 24(AX), DI
	MOVQ 32(AX), R8
	ADDQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI
	ADCQ R8, R8

	// reduce element(CX,BX,SI,DI,R8) using temp registers (R9,R10,R11,R12,R13)
	REDUCE(CX,BX,SI,DI,R8,R9,R10,R11,R12,R13)

	MOVQ CX, 0(DX)
	MOVQ BX, 8(DX)
	MOVQ SI, 16(DX)
	MOVQ DI, 24(DX)
	MOVQ R8, 32(DX)
	MOVQ 40(AX), CX
	MOVQ 48(AX), BX
	MOVQ 56(AX), SI
	MOVQ 64(AX), DI
	MOVQ 72(AX), R8
	ADDQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI
	ADCQ R8, R8

	// reduce element(CX,BX,SI,DI,R8) using temp registers (R14,R15,R9,R10,R11)
	REDUCE(CX,BX,SI,DI,R8,R14,R15,R9,R10,R11)

	MOVQ CX, 40(DX)
	MOVQ BX, 48(DX)
	MOVQ SI, 56(DX)
	MOVQ DI, 64(DX)
	MOVQ R8, 72(DX)
	RET

TEXT ·subE2(SB), NOSPLIT, $0-24
	XORQ    R8, R8
	MOVQ    x+8(FP), DI
	MOVQ    0(DI), AX
	MOVQ    8(DI), DX
	MOVQ    16(DI), CX
	MOVQ    24(DI), BX
	MOVQ    32(DI), SI
	MOVQ    y+16(FP), DI
	SUBQ    0(DI), AX
	SBBQ    8(DI), DX
	SBBQ    16(DI), CX
	SBBQ    24(DI), BX
	SBBQ    32(DI), SI
	MOVQ    x+8(FP), DI
	MOVQ    $0x6fe802ff40300001, R9
	MOVQ    $0x421ee5da52bde502, R10
	MOVQ    $0xdec1d01aa27a1ae0, R11
	MOVQ    $0xd3f7498be97c5eaf, R12
	MOVQ    $0x04c23a02b586d650, R13
	CMOVQCC R8, R9
	CMOVQCC R8, R10
	CMOVQCC R8, R11
	CMOVQCC R8, R12
	CMOVQCC R8, R13
	ADDQ    R9, AX
	ADCQ    R10, DX
	ADCQ    R11, CX
	ADCQ    R12, BX
	ADCQ    R13, SI
	MOVQ    res+0(FP), R14
	MOVQ    AX, 0(R14)
	MOVQ    DX, 8(R14)
	MOVQ    CX, 16(R14)
	MOVQ    BX, 24(R14)
	MOVQ    SI, 32(R14)
	MOVQ    40(DI), AX
	MOVQ    48(DI), DX
	MOVQ    56(DI), CX
	MOVQ    64(DI), BX
	MOVQ    72(DI), SI
	MOVQ    y+16(FP), DI
	SUBQ    40(DI), AX
	SBBQ    48(DI), DX
	SBBQ    56(DI), CX
	SBBQ    64(DI), BX
	SBBQ    72(DI), SI
	MOVQ    $0x6fe802ff40300001, R15
	MOVQ    $0x421ee5da52bde502, R9
	MOVQ    $0xdec1d01aa27a1ae0, R10
	MOVQ    $0xd3f7498be97c5eaf, R11
	MOVQ    $0x04c23a02b586d650, R12
	CMOVQCC R8, R15
	CMOVQCC R8, R9
	CMOVQCC R8, R10
	CMOVQCC R8, R11
	CMOVQCC R8, R12
	ADDQ    R15, AX
	ADCQ    R9, DX
	ADCQ    R10, CX
	ADCQ    R11, BX
	ADCQ    R12, SI
	MOVQ    res+0(FP), DI
	MOVQ    AX, 40(DI)
	MOVQ    DX, 48(DI)
	MOVQ    CX, 56(DI)
	MOVQ    BX, 64(DI)
	MOVQ    SI, 72(DI)
	RET

TEXT ·negE2(SB), NOSPLIT, $0-16
	MOVQ  res+0(FP), DX
	MOVQ  x+8(FP), AX
	MOVQ  0(AX), BX
	MOVQ  8(AX), SI
	MOVQ  16(AX), DI
	MOVQ  24(AX), R8
	MOVQ  32(AX), R9
	MOVQ  BX, AX
	ORQ   SI, AX
	ORQ   DI, AX
	ORQ   R8, AX
	ORQ   R9, AX
	TESTQ AX, AX
	JNE   l1
	MOVQ  AX, 0(DX)
	MOVQ  AX, 8(DX)
	MOVQ  AX, 16(DX)
	MOVQ  AX, 24(DX)
	MOVQ  AX, 32(DX)
	JMP   l3

l1:
	MOVQ $0x6fe802ff40300001, CX
	SUBQ BX, CX
	MOVQ CX, 0(DX)
	MOVQ $0x421ee5da52bde502, CX
	SBBQ SI, CX
	MOVQ CX, 8(DX)
	MOVQ $0xdec1d01aa27a1ae0, CX
	SBBQ DI, CX
	MOVQ CX, 16(DX)
	MOVQ $0xd3f7498be97c5eaf, CX
	SBBQ R8, CX
	MOVQ CX, 24(DX)
	MOVQ $0x04c23a02b586d650, CX
	SBBQ R9, CX
	MOVQ CX, 32(DX)

l3:
	MOVQ  x+8(FP), AX
	MOVQ  40(AX), BX
	MOVQ  48(AX), SI
	MOVQ  56(AX), DI
	MOVQ  64(AX), R8
	MOVQ  72(AX), R9
	MOVQ  BX, AX
	ORQ   SI, AX
	ORQ   DI, AX
	ORQ   R8, AX
	ORQ   R9, AX
	TESTQ AX, AX
	JNE   l2
	MOVQ  AX, 40(DX)
	MOVQ  AX, 48(DX)
	MOVQ  AX, 56(DX)
	MOVQ  AX, 64(DX)
	MOVQ  AX, 72(DX)
	RET

l2:
	MOVQ $0x6fe802ff40300001, CX
	SUBQ BX, CX
	MOVQ CX, 40(DX)
	MOVQ $0x421ee5da52bde502, CX
	SBBQ SI, CX
	MOVQ CX, 48(DX)
	MOVQ $0xdec1d01aa27a1ae0, CX
	SBBQ DI, CX
	MOVQ CX, 56(DX)
	MOVQ $0xd3f7498be97c5eaf, CX
	SBBQ R8, CX
	MOVQ CX, 64(DX)
	MOVQ $0x04c23a02b586d650, CX
	SBBQ R9, CX
	MOVQ CX, 72(DX)
	RET

TEXT ·mulNonResE2(SB), NOSPLIT, $0-16
	MOVQ x+8(FP), AX
	MOVQ 40(AX), DX
	MOVQ 48(AX), CX
	MOVQ 56(AX), BX
	MOVQ 64(AX), SI
	MOVQ 72(AX), DI
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

	ADDQ 40(AX), DX
	ADCQ 48(AX), CX
	ADCQ 56(AX), BX
	ADCQ 64(AX), SI
	ADCQ 72(AX), DI

	// reduce element(DX,CX,BX,SI,DI) using temp registers (R8,R9,R10,R11,R12)
	REDUCE(DX,CX,BX,SI,DI,R8,R9,R10,R11,R12)

	MOVQ res+0(FP), R13
	MOVQ 0(AX), R8
	MOVQ 8(AX), R9
	MOVQ 16(AX), R10
	MOVQ 24(AX), R11
	MOVQ 32(AX), R12
	MOVQ R8, 40(R13)
	MOVQ R9, 48(R13)
	MOVQ R10, 56(R13)
	MOVQ R11, 64(R13)
	MOVQ R12, 72(R13)
	MOVQ DX, 0(R13)
	MOVQ CX, 8(R13)
	MOVQ BX, 16(R13)
	MOVQ SI, 24(R13)
	MOVQ DI, 32(R13)
	RET

TEXT ·mulAdxE2(SB), $80-24
	NO_LOCAL_POINTERS

	// 	var a, b, c fp.Element
	// 	a.Add(&x.A0, &x.A1)
	// 	b.Add(&y.A0, &y.A1)
	// 	a.Mul(&a, &b)
	// 	b.Mul(&x.A0, &y.A0)
	// 	c.Mul(&x.A1, &y.A1)
	// 	z.A1.Sub(&a, &b).Sub(&z.A1, &c)
	// 	fp.MulBy13(&c)
	// 	z.A0.Add(&c, &b)

	CMPB ·supportAdx(SB), $1
	JNE  l4
	MOVQ x+8(FP), AX
	MOVQ 40(AX), R14
	MOVQ 48(AX), R15
	MOVQ 56(AX), CX
	MOVQ 64(AX), BX
	MOVQ 72(AX), SI

	// A -> BP
	// t[0] -> DI
	// t[1] -> R8
	// t[2] -> R9
	// t[3] -> R10
	// t[4] -> R11
	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 40(DX), DX

	// (A,t[0])  := x[0]*y[0] + A
	MULXQ R14, DI, R8

	// (A,t[1])  := x[1]*y[0] + A
	MULXQ R15, AX, R9
	ADOXQ AX, R8

	// (A,t[2])  := x[2]*y[0] + A
	MULXQ CX, AX, R10
	ADOXQ AX, R9

	// (A,t[3])  := x[3]*y[0] + A
	MULXQ BX, AX, R11
	ADOXQ AX, R10

	// (A,t[4])  := x[4]*y[0] + A
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R12
	ADCXQ DI, AX
	MOVQ  R12, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 48(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[1] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[1] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[1] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[1] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[1] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R12
	ADCXQ DI, AX
	MOVQ  R12, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 56(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[2] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[2] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[2] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[2] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[2] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R12
	ADCXQ DI, AX
	MOVQ  R12, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 64(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[3] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[3] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[3] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[3] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[3] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R12
	ADCXQ DI, AX
	MOVQ  R12, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 72(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[4] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[4] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[4] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[4] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[4] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R12
	ADCXQ DI, AX
	MOVQ  R12, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// reduce element(DI,R8,R9,R10,R11) using temp registers (R14,R15,CX,BX,SI)
	REDUCE(DI,R8,R9,R10,R11,R14,R15,CX,BX,SI)

	MOVQ DI, s5-48(SP)
	MOVQ R8, s6-56(SP)
	MOVQ R9, s7-64(SP)
	MOVQ R10, s8-72(SP)
	MOVQ R11, s9-80(SP)
	MOVQ x+8(FP), AX
	MOVQ y+16(FP), DX
	MOVQ 40(AX), R14
	MOVQ 48(AX), R15
	MOVQ 56(AX), CX
	MOVQ 64(AX), BX
	MOVQ 72(AX), SI
	ADDQ 0(AX), R14
	ADCQ 8(AX), R15
	ADCQ 16(AX), CX
	ADCQ 24(AX), BX
	ADCQ 32(AX), SI
	MOVQ R14, s0-8(SP)
	MOVQ R15, s1-16(SP)
	MOVQ CX, s2-24(SP)
	MOVQ BX, s3-32(SP)
	MOVQ SI, s4-40(SP)
	MOVQ 0(DX), R14
	MOVQ 8(DX), R15
	MOVQ 16(DX), CX
	MOVQ 24(DX), BX
	MOVQ 32(DX), SI
	ADDQ 40(DX), R14
	ADCQ 48(DX), R15
	ADCQ 56(DX), CX
	ADCQ 64(DX), BX
	ADCQ 72(DX), SI

	// A -> BP
	// t[0] -> DI
	// t[1] -> R8
	// t[2] -> R9
	// t[3] -> R10
	// t[4] -> R11
	// clear the flags
	XORQ AX, AX
	MOVQ s0-8(SP), DX

	// (A,t[0])  := x[0]*y[0] + A
	MULXQ R14, DI, R8

	// (A,t[1])  := x[1]*y[0] + A
	MULXQ R15, AX, R9
	ADOXQ AX, R8

	// (A,t[2])  := x[2]*y[0] + A
	MULXQ CX, AX, R10
	ADOXQ AX, R9

	// (A,t[3])  := x[3]*y[0] + A
	MULXQ BX, AX, R11
	ADOXQ AX, R10

	// (A,t[4])  := x[4]*y[0] + A
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R13
	ADCXQ DI, AX
	MOVQ  R13, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ s1-16(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[1] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[1] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[1] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[1] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[1] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R13
	ADCXQ DI, AX
	MOVQ  R13, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ s2-24(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[2] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[2] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[2] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[2] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[2] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R13
	ADCXQ DI, AX
	MOVQ  R13, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ s3-32(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[3] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[3] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[3] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[3] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[3] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R13
	ADCXQ DI, AX
	MOVQ  R13, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ s4-40(SP), DX

	// (A,t[0])  := t[0] + x[0]*y[4] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[4] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[4] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[4] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[4] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R13
	ADCXQ DI, AX
	MOVQ  R13, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// reduce element(DI,R8,R9,R10,R11) using temp registers (R14,R15,CX,BX,SI)
	REDUCE(DI,R8,R9,R10,R11,R14,R15,CX,BX,SI)

	MOVQ DI, s0-8(SP)
	MOVQ R8, s1-16(SP)
	MOVQ R9, s2-24(SP)
	MOVQ R10, s3-32(SP)
	MOVQ R11, s4-40(SP)
	MOVQ x+8(FP), AX
	MOVQ 0(AX), R14
	MOVQ 8(AX), R15
	MOVQ 16(AX), CX
	MOVQ 24(AX), BX
	MOVQ 32(AX), SI

	// A -> BP
	// t[0] -> DI
	// t[1] -> R8
	// t[2] -> R9
	// t[3] -> R10
	// t[4] -> R11
	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 0(DX), DX

	// (A,t[0])  := x[0]*y[0] + A
	MULXQ R14, DI, R8

	// (A,t[1])  := x[1]*y[0] + A
	MULXQ R15, AX, R9
	ADOXQ AX, R8

	// (A,t[2])  := x[2]*y[0] + A
	MULXQ CX, AX, R10
	ADOXQ AX, R9

	// (A,t[3])  := x[3]*y[0] + A
	MULXQ BX, AX, R11
	ADOXQ AX, R10

	// (A,t[4])  := x[4]*y[0] + A
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R12
	ADCXQ DI, AX
	MOVQ  R12, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 8(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[1] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[1] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[1] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[1] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[1] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R12
	ADCXQ DI, AX
	MOVQ  R12, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 16(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[2] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[2] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[2] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[2] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[2] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R12
	ADCXQ DI, AX
	MOVQ  R12, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 24(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[3] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[3] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[3] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[3] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[3] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R12
	ADCXQ DI, AX
	MOVQ  R12, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// clear the flags
	XORQ AX, AX
	MOVQ y+16(FP), DX
	MOVQ 32(DX), DX

	// (A,t[0])  := t[0] + x[0]*y[4] + A
	MULXQ R14, AX, BP
	ADOXQ AX, DI

	// (A,t[1])  := t[1] + x[1]*y[4] + A
	ADCXQ BP, R8
	MULXQ R15, AX, BP
	ADOXQ AX, R8

	// (A,t[2])  := t[2] + x[2]*y[4] + A
	ADCXQ BP, R9
	MULXQ CX, AX, BP
	ADOXQ AX, R9

	// (A,t[3])  := t[3] + x[3]*y[4] + A
	ADCXQ BP, R10
	MULXQ BX, AX, BP
	ADOXQ AX, R10

	// (A,t[4])  := t[4] + x[4]*y[4] + A
	ADCXQ BP, R11
	MULXQ SI, AX, BP
	ADOXQ AX, R11

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ DI, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R12
	ADCXQ DI, AX
	MOVQ  R12, DI

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R8, DI
	MULXQ q<>+8(SB), AX, R8
	ADOXQ AX, DI

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ R9, R8
	MULXQ q<>+16(SB), AX, R9
	ADOXQ AX, R8

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ R10, R9
	MULXQ q<>+24(SB), AX, R10
	ADOXQ AX, R9

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ R11, R10
	MULXQ q<>+32(SB), AX, R11
	ADOXQ AX, R10

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, R11
	ADOXQ BP, R11

	// reduce element(DI,R8,R9,R10,R11) using temp registers (R14,R15,CX,BX,SI)
	REDUCE(DI,R8,R9,R10,R11,R14,R15,CX,BX,SI)

	XORQ    DX, DX
	MOVQ    s0-8(SP), R14
	MOVQ    s1-16(SP), R15
	MOVQ    s2-24(SP), CX
	MOVQ    s3-32(SP), BX
	MOVQ    s4-40(SP), SI
	SUBQ    DI, R14
	SBBQ    R8, R15
	SBBQ    R9, CX
	SBBQ    R10, BX
	SBBQ    R11, SI
	MOVQ    DI, s0-8(SP)
	MOVQ    R8, s1-16(SP)
	MOVQ    R9, s2-24(SP)
	MOVQ    R10, s3-32(SP)
	MOVQ    R11, s4-40(SP)
	MOVQ    $0x6fe802ff40300001, DI
	MOVQ    $0x421ee5da52bde502, R8
	MOVQ    $0xdec1d01aa27a1ae0, R9
	MOVQ    $0xd3f7498be97c5eaf, R10
	MOVQ    $0x04c23a02b586d650, R11
	CMOVQCC DX, DI
	CMOVQCC DX, R8
	CMOVQCC DX, R9
	CMOVQCC DX, R10
	CMOVQCC DX, R11
	ADDQ    DI, R14
	ADCQ    R8, R15
	ADCQ    R9, CX
	ADCQ    R10, BX
	ADCQ    R11, SI
	SUBQ    s5-48(SP), R14
	SBBQ    s6-56(SP), R15
	SBBQ    s7-64(SP), CX
	SBBQ    s8-72(SP), BX
	SBBQ    s9-80(SP), SI
	MOVQ    $0x6fe802ff40300001, DI
	MOVQ    $0x421ee5da52bde502, R8
	MOVQ    $0xdec1d01aa27a1ae0, R9
	MOVQ    $0xd3f7498be97c5eaf, R10
	MOVQ    $0x04c23a02b586d650, R11
	CMOVQCC DX, DI
	CMOVQCC DX, R8
	CMOVQCC DX, R9
	CMOVQCC DX, R10
	CMOVQCC DX, R11
	ADDQ    DI, R14
	ADCQ    R8, R15
	ADCQ    R9, CX
	ADCQ    R10, BX
	ADCQ    R11, SI
	MOVQ    res+0(FP), AX
	MOVQ    R14, 40(AX)
	MOVQ    R15, 48(AX)
	MOVQ    CX, 56(AX)
	MOVQ    BX, 64(AX)
	MOVQ    SI, 72(AX)
	MOVQ    s5-48(SP), DI
	MOVQ    s6-56(SP), R8
	MOVQ    s7-64(SP), R9
	MOVQ    s8-72(SP), R10
	MOVQ    s9-80(SP), R11
	MOVQ    s0-8(SP), R14
	MOVQ    s1-16(SP), R15
	MOVQ    s2-24(SP), CX
	MOVQ    s3-32(SP), BX
	MOVQ    s4-40(SP), SI
	ADDQ    DI, R14
	ADCQ    R8, R15
	ADCQ    R9, CX
	ADCQ    R10, BX
	ADCQ    R11, SI

	// reduce element(R14,R15,CX,BX,SI) using temp registers (DI,R8,R9,R10,R11)
	REDUCE(R14,R15,CX,BX,SI,DI,R8,R9,R10,R11)

	MOVQ s5-48(SP), DI
	MOVQ s6-56(SP), R8
	MOVQ s7-64(SP), R9
	MOVQ s8-72(SP), R10
	MOVQ s9-80(SP), R11
	MOVQ R14, s5-48(SP)
	MOVQ R15, s6-56(SP)
	MOVQ CX, s7-64(SP)
	MOVQ BX, s8-72(SP)
	MOVQ SI, s9-80(SP)
	ADDQ DI, DI
	ADCQ R8, R8
	ADCQ R9, R9
	ADCQ R10, R10
	ADCQ R11, R11

	// reduce element(DI,R8,R9,R10,R11) using temp registers (R14,R15,CX,BX,SI)
	REDUCE(DI,R8,R9,R10,R11,R14,R15,CX,BX,SI)

	ADDQ DI, DI
	ADCQ R8, R8
	ADCQ R9, R9
	ADCQ R10, R10
	ADCQ R11, R11

	// reduce element(DI,R8,R9,R10,R11) using temp registers (R14,R15,CX,BX,SI)
	REDUCE(DI,R8,R9,R10,R11,R14,R15,CX,BX,SI)

	MOVQ DI, s0-8(SP)
	MOVQ R8, s1-16(SP)
	MOVQ R9, s2-24(SP)
	MOVQ R10, s3-32(SP)
	MOVQ R11, s4-40(SP)
	ADDQ DI, DI
	ADCQ R8, R8
	ADCQ R9, R9
	ADCQ R10, R10
	ADCQ R11, R11

	// reduce element(DI,R8,R9,R10,R11) using temp registers (R14,R15,CX,BX,SI)
	REDUCE(DI,R8,R9,R10,R11,R14,R15,CX,BX,SI)

	ADDQ s0-8(SP), DI
	ADCQ s1-16(SP), R8
	ADCQ s2-24(SP), R9
	ADCQ s3-32(SP), R10
	ADCQ s4-40(SP), R11

	// reduce element(DI,R8,R9,R10,R11) using temp registers (R14,R15,CX,BX,SI)
	REDUCE(DI,R8,R9,R10,R11,R14,R15,CX,BX,SI)

	ADDQ s5-48(SP), DI
	ADCQ s6-56(SP), R8
	ADCQ s7-64(SP), R9
	ADCQ s8-72(SP), R10
	ADCQ s9-80(SP), R11

	// reduce element(DI,R8,R9,R10,R11) using temp registers (R14,R15,CX,BX,SI)
	REDUCE(DI,R8,R9,R10,R11,R14,R15,CX,BX,SI)

	MOVQ DI, 0(AX)
	MOVQ R8, 8(AX)
	MOVQ R9, 16(AX)
	MOVQ R10, 24(AX)
	MOVQ R11, 32(AX)
	RET

l4:
	MOVQ res+0(FP), AX
	MOVQ AX, (SP)
	MOVQ x+8(FP), AX
	MOVQ AX, 8(SP)
	MOVQ y+16(FP), AX
	MOVQ AX, 16(SP)
	CALL ·mulGenericE2(SB)
	RET
