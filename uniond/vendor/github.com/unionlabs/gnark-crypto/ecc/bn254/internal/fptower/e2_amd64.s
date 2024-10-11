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
DATA q<>+0(SB)/8, $0x3c208c16d87cfd47
DATA q<>+8(SB)/8, $0x97816a916871ca8d
DATA q<>+16(SB)/8, $0xb85045b68181585d
DATA q<>+24(SB)/8, $0x30644e72e131a029
GLOBL q<>(SB), (RODATA+NOPTR), $32

// qInv0 q'[0]
DATA qInv0<>(SB)/8, $0x87d20782e4866389
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

// this code is generated and identical to fp.Mul(...)
#define MUL() \
	XORQ  AX, AX;              \
	MOVQ  SI, DX;              \
	MULXQ R14, R10, R11;       \
	MULXQ R15, AX, R12;        \
	ADOXQ AX, R11;             \
	MULXQ CX, AX, R13;         \
	ADOXQ AX, R12;             \
	MULXQ BX, AX, BP;          \
	ADOXQ AX, R13;             \
	MOVQ  $0, AX;              \
	ADOXQ AX, BP;              \
	PUSHQ BP;                  \
	MOVQ  qInv0<>(SB), DX;     \
	IMULQ R10, DX;             \
	XORQ  AX, AX;              \
	MULXQ q<>+0(SB), AX, BP;   \
	ADCXQ R10, AX;             \
	MOVQ  BP, R10;             \
	POPQ  BP;                  \
	ADCXQ R11, R10;            \
	MULXQ q<>+8(SB), AX, R11;  \
	ADOXQ AX, R10;             \
	ADCXQ R12, R11;            \
	MULXQ q<>+16(SB), AX, R12; \
	ADOXQ AX, R11;             \
	ADCXQ R13, R12;            \
	MULXQ q<>+24(SB), AX, R13; \
	ADOXQ AX, R12;             \
	MOVQ  $0, AX;              \
	ADCXQ AX, R13;             \
	ADOXQ BP, R13;             \
	XORQ  AX, AX;              \
	MOVQ  DI, DX;              \
	MULXQ R14, AX, BP;         \
	ADOXQ AX, R10;             \
	ADCXQ BP, R11;             \
	MULXQ R15, AX, BP;         \
	ADOXQ AX, R11;             \
	ADCXQ BP, R12;             \
	MULXQ CX, AX, BP;          \
	ADOXQ AX, R12;             \
	ADCXQ BP, R13;             \
	MULXQ BX, AX, BP;          \
	ADOXQ AX, R13;             \
	MOVQ  $0, AX;              \
	ADCXQ AX, BP;              \
	ADOXQ AX, BP;              \
	PUSHQ BP;                  \
	MOVQ  qInv0<>(SB), DX;     \
	IMULQ R10, DX;             \
	XORQ  AX, AX;              \
	MULXQ q<>+0(SB), AX, BP;   \
	ADCXQ R10, AX;             \
	MOVQ  BP, R10;             \
	POPQ  BP;                  \
	ADCXQ R11, R10;            \
	MULXQ q<>+8(SB), AX, R11;  \
	ADOXQ AX, R10;             \
	ADCXQ R12, R11;            \
	MULXQ q<>+16(SB), AX, R12; \
	ADOXQ AX, R11;             \
	ADCXQ R13, R12;            \
	MULXQ q<>+24(SB), AX, R13; \
	ADOXQ AX, R12;             \
	MOVQ  $0, AX;              \
	ADCXQ AX, R13;             \
	ADOXQ BP, R13;             \
	XORQ  AX, AX;              \
	MOVQ  R8, DX;              \
	MULXQ R14, AX, BP;         \
	ADOXQ AX, R10;             \
	ADCXQ BP, R11;             \
	MULXQ R15, AX, BP;         \
	ADOXQ AX, R11;             \
	ADCXQ BP, R12;             \
	MULXQ CX, AX, BP;          \
	ADOXQ AX, R12;             \
	ADCXQ BP, R13;             \
	MULXQ BX, AX, BP;          \
	ADOXQ AX, R13;             \
	MOVQ  $0, AX;              \
	ADCXQ AX, BP;              \
	ADOXQ AX, BP;              \
	PUSHQ BP;                  \
	MOVQ  qInv0<>(SB), DX;     \
	IMULQ R10, DX;             \
	XORQ  AX, AX;              \
	MULXQ q<>+0(SB), AX, BP;   \
	ADCXQ R10, AX;             \
	MOVQ  BP, R10;             \
	POPQ  BP;                  \
	ADCXQ R11, R10;            \
	MULXQ q<>+8(SB), AX, R11;  \
	ADOXQ AX, R10;             \
	ADCXQ R12, R11;            \
	MULXQ q<>+16(SB), AX, R12; \
	ADOXQ AX, R11;             \
	ADCXQ R13, R12;            \
	MULXQ q<>+24(SB), AX, R13; \
	ADOXQ AX, R12;             \
	MOVQ  $0, AX;              \
	ADCXQ AX, R13;             \
	ADOXQ BP, R13;             \
	XORQ  AX, AX;              \
	MOVQ  R9, DX;              \
	MULXQ R14, AX, BP;         \
	ADOXQ AX, R10;             \
	ADCXQ BP, R11;             \
	MULXQ R15, AX, BP;         \
	ADOXQ AX, R11;             \
	ADCXQ BP, R12;             \
	MULXQ CX, AX, BP;          \
	ADOXQ AX, R12;             \
	ADCXQ BP, R13;             \
	MULXQ BX, AX, BP;          \
	ADOXQ AX, R13;             \
	MOVQ  $0, AX;              \
	ADCXQ AX, BP;              \
	ADOXQ AX, BP;              \
	PUSHQ BP;                  \
	MOVQ  qInv0<>(SB), DX;     \
	IMULQ R10, DX;             \
	XORQ  AX, AX;              \
	MULXQ q<>+0(SB), AX, BP;   \
	ADCXQ R10, AX;             \
	MOVQ  BP, R10;             \
	POPQ  BP;                  \
	ADCXQ R11, R10;            \
	MULXQ q<>+8(SB), AX, R11;  \
	ADOXQ AX, R10;             \
	ADCXQ R12, R11;            \
	MULXQ q<>+16(SB), AX, R12; \
	ADOXQ AX, R11;             \
	ADCXQ R13, R12;            \
	MULXQ q<>+24(SB), AX, R13; \
	ADOXQ AX, R12;             \
	MOVQ  $0, AX;              \
	ADCXQ AX, R13;             \
	ADOXQ BP, R13;             \

TEXT ·addE2(SB), NOSPLIT, $0-24
	MOVQ x+8(FP), AX
	MOVQ 0(AX), BX
	MOVQ 8(AX), SI
	MOVQ 16(AX), DI
	MOVQ 24(AX), R8
	MOVQ y+16(FP), DX
	ADDQ 0(DX), BX
	ADCQ 8(DX), SI
	ADCQ 16(DX), DI
	ADCQ 24(DX), R8

	// reduce element(BX,SI,DI,R8) using temp registers (R9,R10,R11,R12)
	REDUCE(BX,SI,DI,R8,R9,R10,R11,R12)

	MOVQ res+0(FP), CX
	MOVQ BX, 0(CX)
	MOVQ SI, 8(CX)
	MOVQ DI, 16(CX)
	MOVQ R8, 24(CX)
	MOVQ 32(AX), BX
	MOVQ 40(AX), SI
	MOVQ 48(AX), DI
	MOVQ 56(AX), R8
	ADDQ 32(DX), BX
	ADCQ 40(DX), SI
	ADCQ 48(DX), DI
	ADCQ 56(DX), R8

	// reduce element(BX,SI,DI,R8) using temp registers (R13,R14,R15,R9)
	REDUCE(BX,SI,DI,R8,R13,R14,R15,R9)

	MOVQ BX, 32(CX)
	MOVQ SI, 40(CX)
	MOVQ DI, 48(CX)
	MOVQ R8, 56(CX)
	RET

TEXT ·doubleE2(SB), NOSPLIT, $0-16
	MOVQ res+0(FP), DX
	MOVQ x+8(FP), AX
	MOVQ 0(AX), CX
	MOVQ 8(AX), BX
	MOVQ 16(AX), SI
	MOVQ 24(AX), DI
	ADDQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI

	// reduce element(CX,BX,SI,DI) using temp registers (R8,R9,R10,R11)
	REDUCE(CX,BX,SI,DI,R8,R9,R10,R11)

	MOVQ CX, 0(DX)
	MOVQ BX, 8(DX)
	MOVQ SI, 16(DX)
	MOVQ DI, 24(DX)
	MOVQ 32(AX), CX
	MOVQ 40(AX), BX
	MOVQ 48(AX), SI
	MOVQ 56(AX), DI
	ADDQ CX, CX
	ADCQ BX, BX
	ADCQ SI, SI
	ADCQ DI, DI

	// reduce element(CX,BX,SI,DI) using temp registers (R12,R13,R14,R15)
	REDUCE(CX,BX,SI,DI,R12,R13,R14,R15)

	MOVQ CX, 32(DX)
	MOVQ BX, 40(DX)
	MOVQ SI, 48(DX)
	MOVQ DI, 56(DX)
	RET

TEXT ·subE2(SB), NOSPLIT, $0-24
	XORQ    DI, DI
	MOVQ    x+8(FP), SI
	MOVQ    0(SI), AX
	MOVQ    8(SI), DX
	MOVQ    16(SI), CX
	MOVQ    24(SI), BX
	MOVQ    y+16(FP), SI
	SUBQ    0(SI), AX
	SBBQ    8(SI), DX
	SBBQ    16(SI), CX
	SBBQ    24(SI), BX
	MOVQ    x+8(FP), SI
	MOVQ    $0x3c208c16d87cfd47, R8
	MOVQ    $0x97816a916871ca8d, R9
	MOVQ    $0xb85045b68181585d, R10
	MOVQ    $0x30644e72e131a029, R11
	CMOVQCC DI, R8
	CMOVQCC DI, R9
	CMOVQCC DI, R10
	CMOVQCC DI, R11
	ADDQ    R8, AX
	ADCQ    R9, DX
	ADCQ    R10, CX
	ADCQ    R11, BX
	MOVQ    res+0(FP), R12
	MOVQ    AX, 0(R12)
	MOVQ    DX, 8(R12)
	MOVQ    CX, 16(R12)
	MOVQ    BX, 24(R12)
	MOVQ    32(SI), AX
	MOVQ    40(SI), DX
	MOVQ    48(SI), CX
	MOVQ    56(SI), BX
	MOVQ    y+16(FP), SI
	SUBQ    32(SI), AX
	SBBQ    40(SI), DX
	SBBQ    48(SI), CX
	SBBQ    56(SI), BX
	MOVQ    $0x3c208c16d87cfd47, R13
	MOVQ    $0x97816a916871ca8d, R14
	MOVQ    $0xb85045b68181585d, R15
	MOVQ    $0x30644e72e131a029, R8
	CMOVQCC DI, R13
	CMOVQCC DI, R14
	CMOVQCC DI, R15
	CMOVQCC DI, R8
	ADDQ    R13, AX
	ADCQ    R14, DX
	ADCQ    R15, CX
	ADCQ    R8, BX
	MOVQ    res+0(FP), SI
	MOVQ    AX, 32(SI)
	MOVQ    DX, 40(SI)
	MOVQ    CX, 48(SI)
	MOVQ    BX, 56(SI)
	RET

TEXT ·negE2(SB), NOSPLIT, $0-16
	MOVQ  res+0(FP), DX
	MOVQ  x+8(FP), AX
	MOVQ  0(AX), BX
	MOVQ  8(AX), SI
	MOVQ  16(AX), DI
	MOVQ  24(AX), R8
	MOVQ  BX, AX
	ORQ   SI, AX
	ORQ   DI, AX
	ORQ   R8, AX
	TESTQ AX, AX
	JNE   l1
	MOVQ  AX, 0(DX)
	MOVQ  AX, 8(DX)
	MOVQ  AX, 16(DX)
	MOVQ  AX, 24(DX)
	JMP   l3

l1:
	MOVQ $0x3c208c16d87cfd47, CX
	SUBQ BX, CX
	MOVQ CX, 0(DX)
	MOVQ $0x97816a916871ca8d, CX
	SBBQ SI, CX
	MOVQ CX, 8(DX)
	MOVQ $0xb85045b68181585d, CX
	SBBQ DI, CX
	MOVQ CX, 16(DX)
	MOVQ $0x30644e72e131a029, CX
	SBBQ R8, CX
	MOVQ CX, 24(DX)

l3:
	MOVQ  x+8(FP), AX
	MOVQ  32(AX), BX
	MOVQ  40(AX), SI
	MOVQ  48(AX), DI
	MOVQ  56(AX), R8
	MOVQ  BX, AX
	ORQ   SI, AX
	ORQ   DI, AX
	ORQ   R8, AX
	TESTQ AX, AX
	JNE   l2
	MOVQ  AX, 32(DX)
	MOVQ  AX, 40(DX)
	MOVQ  AX, 48(DX)
	MOVQ  AX, 56(DX)
	RET

l2:
	MOVQ $0x3c208c16d87cfd47, CX
	SUBQ BX, CX
	MOVQ CX, 32(DX)
	MOVQ $0x97816a916871ca8d, CX
	SBBQ SI, CX
	MOVQ CX, 40(DX)
	MOVQ $0xb85045b68181585d, CX
	SBBQ DI, CX
	MOVQ CX, 48(DX)
	MOVQ $0x30644e72e131a029, CX
	SBBQ R8, CX
	MOVQ CX, 56(DX)
	RET

TEXT ·mulNonResE2(SB), NOSPLIT, $0-16
	MOVQ x+8(FP), R10
	MOVQ 0(R10), AX
	MOVQ 8(R10), DX
	MOVQ 16(R10), CX
	MOVQ 24(R10), BX
	ADDQ AX, AX
	ADCQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX

	// reduce element(AX,DX,CX,BX) using temp registers (R11,R12,R13,R14)
	REDUCE(AX,DX,CX,BX,R11,R12,R13,R14)

	ADDQ AX, AX
	ADCQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX

	// reduce element(AX,DX,CX,BX) using temp registers (R15,R11,R12,R13)
	REDUCE(AX,DX,CX,BX,R15,R11,R12,R13)

	ADDQ AX, AX
	ADCQ DX, DX
	ADCQ CX, CX
	ADCQ BX, BX

	// reduce element(AX,DX,CX,BX) using temp registers (R14,R15,R11,R12)
	REDUCE(AX,DX,CX,BX,R14,R15,R11,R12)

	ADDQ 0(R10), AX
	ADCQ 8(R10), DX
	ADCQ 16(R10), CX
	ADCQ 24(R10), BX

	// reduce element(AX,DX,CX,BX) using temp registers (R13,R14,R15,R11)
	REDUCE(AX,DX,CX,BX,R13,R14,R15,R11)

	MOVQ    32(R10), SI
	MOVQ    40(R10), DI
	MOVQ    48(R10), R8
	MOVQ    56(R10), R9
	XORQ    R12, R12
	SUBQ    SI, AX
	SBBQ    DI, DX
	SBBQ    R8, CX
	SBBQ    R9, BX
	MOVQ    $0x3c208c16d87cfd47, R13
	MOVQ    $0x97816a916871ca8d, R14
	MOVQ    $0xb85045b68181585d, R15
	MOVQ    $0x30644e72e131a029, R11
	CMOVQCC R12, R13
	CMOVQCC R12, R14
	CMOVQCC R12, R15
	CMOVQCC R12, R11
	ADDQ    R13, AX
	ADCQ    R14, DX
	ADCQ    R15, CX
	ADCQ    R11, BX
	ADDQ    SI, SI
	ADCQ    DI, DI
	ADCQ    R8, R8
	ADCQ    R9, R9

	// reduce element(SI,DI,R8,R9) using temp registers (R13,R14,R15,R11)
	REDUCE(SI,DI,R8,R9,R13,R14,R15,R11)

	ADDQ SI, SI
	ADCQ DI, DI
	ADCQ R8, R8
	ADCQ R9, R9

	// reduce element(SI,DI,R8,R9) using temp registers (R12,R13,R14,R15)
	REDUCE(SI,DI,R8,R9,R12,R13,R14,R15)

	ADDQ SI, SI
	ADCQ DI, DI
	ADCQ R8, R8
	ADCQ R9, R9

	// reduce element(SI,DI,R8,R9) using temp registers (R11,R12,R13,R14)
	REDUCE(SI,DI,R8,R9,R11,R12,R13,R14)

	ADDQ 32(R10), SI
	ADCQ 40(R10), DI
	ADCQ 48(R10), R8
	ADCQ 56(R10), R9

	// reduce element(SI,DI,R8,R9) using temp registers (R15,R11,R12,R13)
	REDUCE(SI,DI,R8,R9,R15,R11,R12,R13)

	ADDQ 0(R10), SI
	ADCQ 8(R10), DI
	ADCQ 16(R10), R8
	ADCQ 24(R10), R9

	// reduce element(SI,DI,R8,R9) using temp registers (R14,R15,R11,R12)
	REDUCE(SI,DI,R8,R9,R14,R15,R11,R12)

	MOVQ res+0(FP), R10
	MOVQ AX, 0(R10)
	MOVQ DX, 8(R10)
	MOVQ CX, 16(R10)
	MOVQ BX, 24(R10)
	MOVQ SI, 32(R10)
	MOVQ DI, 40(R10)
	MOVQ R8, 48(R10)
	MOVQ R9, 56(R10)
	RET

TEXT ·mulAdxE2(SB), $64-24
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
	JNE  l4
	MOVQ x+8(FP), AX
	MOVQ y+16(FP), DX
	MOVQ 32(AX), R14
	MOVQ 40(AX), R15
	MOVQ 48(AX), CX
	MOVQ 56(AX), BX
	MOVQ 32(DX), SI
	MOVQ 40(DX), DI
	MOVQ 48(DX), R8
	MOVQ 56(DX), R9

	// mul (R14,R15,CX,BX) with (SI,DI,R8,R9) into (R10,R11,R12,R13)
	MUL()

	// reduce element(R10,R11,R12,R13) using temp registers (SI,DI,R8,R9)
	REDUCE(R10,R11,R12,R13,SI,DI,R8,R9)

	MOVQ R10, s4-40(SP)
	MOVQ R11, s5-48(SP)
	MOVQ R12, s6-56(SP)
	MOVQ R13, s7-64(SP)
	MOVQ x+8(FP), AX
	MOVQ y+16(FP), DX
	ADDQ 0(AX), R14
	ADCQ 8(AX), R15
	ADCQ 16(AX), CX
	ADCQ 24(AX), BX
	MOVQ 0(DX), SI
	MOVQ 8(DX), DI
	MOVQ 16(DX), R8
	MOVQ 24(DX), R9
	ADDQ 32(DX), SI
	ADCQ 40(DX), DI
	ADCQ 48(DX), R8
	ADCQ 56(DX), R9

	// mul (R14,R15,CX,BX) with (SI,DI,R8,R9) into (R10,R11,R12,R13)
	MUL()

	// reduce element(R10,R11,R12,R13) using temp registers (SI,DI,R8,R9)
	REDUCE(R10,R11,R12,R13,SI,DI,R8,R9)

	MOVQ R10, s0-8(SP)
	MOVQ R11, s1-16(SP)
	MOVQ R12, s2-24(SP)
	MOVQ R13, s3-32(SP)
	MOVQ x+8(FP), AX
	MOVQ y+16(FP), DX
	MOVQ 0(AX), R14
	MOVQ 8(AX), R15
	MOVQ 16(AX), CX
	MOVQ 24(AX), BX
	MOVQ 0(DX), SI
	MOVQ 8(DX), DI
	MOVQ 16(DX), R8
	MOVQ 24(DX), R9

	// mul (R14,R15,CX,BX) with (SI,DI,R8,R9) into (R10,R11,R12,R13)
	MUL()

	// reduce element(R10,R11,R12,R13) using temp registers (SI,DI,R8,R9)
	REDUCE(R10,R11,R12,R13,SI,DI,R8,R9)

	XORQ    DX, DX
	MOVQ    s0-8(SP), R14
	MOVQ    s1-16(SP), R15
	MOVQ    s2-24(SP), CX
	MOVQ    s3-32(SP), BX
	SUBQ    R10, R14
	SBBQ    R11, R15
	SBBQ    R12, CX
	SBBQ    R13, BX
	MOVQ    $0x3c208c16d87cfd47, SI
	MOVQ    $0x97816a916871ca8d, DI
	MOVQ    $0xb85045b68181585d, R8
	MOVQ    $0x30644e72e131a029, R9
	CMOVQCC DX, SI
	CMOVQCC DX, DI
	CMOVQCC DX, R8
	CMOVQCC DX, R9
	ADDQ    SI, R14
	ADCQ    DI, R15
	ADCQ    R8, CX
	ADCQ    R9, BX
	SUBQ    s4-40(SP), R14
	SBBQ    s5-48(SP), R15
	SBBQ    s6-56(SP), CX
	SBBQ    s7-64(SP), BX
	MOVQ    $0x3c208c16d87cfd47, SI
	MOVQ    $0x97816a916871ca8d, DI
	MOVQ    $0xb85045b68181585d, R8
	MOVQ    $0x30644e72e131a029, R9
	CMOVQCC DX, SI
	CMOVQCC DX, DI
	CMOVQCC DX, R8
	CMOVQCC DX, R9
	ADDQ    SI, R14
	ADCQ    DI, R15
	ADCQ    R8, CX
	ADCQ    R9, BX
	MOVQ    res+0(FP), AX
	MOVQ    R14, 32(AX)
	MOVQ    R15, 40(AX)
	MOVQ    CX, 48(AX)
	MOVQ    BX, 56(AX)
	MOVQ    s4-40(SP), SI
	MOVQ    s5-48(SP), DI
	MOVQ    s6-56(SP), R8
	MOVQ    s7-64(SP), R9
	SUBQ    SI, R10
	SBBQ    DI, R11
	SBBQ    R8, R12
	SBBQ    R9, R13
	MOVQ    $0x3c208c16d87cfd47, R14
	MOVQ    $0x97816a916871ca8d, R15
	MOVQ    $0xb85045b68181585d, CX
	MOVQ    $0x30644e72e131a029, BX
	CMOVQCC DX, R14
	CMOVQCC DX, R15
	CMOVQCC DX, CX
	CMOVQCC DX, BX
	ADDQ    R14, R10
	ADCQ    R15, R11
	ADCQ    CX, R12
	ADCQ    BX, R13
	MOVQ    R10, 0(AX)
	MOVQ    R11, 8(AX)
	MOVQ    R12, 16(AX)
	MOVQ    R13, 24(AX)
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

TEXT ·squareAdxE2(SB), $16-16
	NO_LOCAL_POINTERS

	// z.A0 = (x.A0 + x.A1) * (x.A0 - x.A1)
	// z.A1 = 2 * x.A0 * x.A1

	CMPB ·supportAdx(SB), $1
	JNE  l5

	// 2 * x.A0 * x.A1
	MOVQ x+8(FP), AX

	// x.A0[0] -> SI
	// x.A0[1] -> DI
	// x.A0[2] -> R8
	// x.A0[3] -> R9
	MOVQ 0(AX), SI
	MOVQ 8(AX), DI
	MOVQ 16(AX), R8
	MOVQ 24(AX), R9

	// 2 * x.A1[0] -> R14
	// 2 * x.A1[1] -> R15
	// 2 * x.A1[2] -> CX
	// 2 * x.A1[3] -> BX
	MOVQ 32(AX), R14
	MOVQ 40(AX), R15
	MOVQ 48(AX), CX
	MOVQ 56(AX), BX
	ADDQ R14, R14
	ADCQ R15, R15
	ADCQ CX, CX
	ADCQ BX, BX

	// mul (R14,R15,CX,BX) with (SI,DI,R8,R9) into (R10,R11,R12,R13)
	MUL()

	// reduce element(R10,R11,R12,R13) using temp registers (R14,R15,CX,BX)
	REDUCE(R10,R11,R12,R13,R14,R15,CX,BX)

	MOVQ x+8(FP), AX

	// x.A1[0] -> R14
	// x.A1[1] -> R15
	// x.A1[2] -> CX
	// x.A1[3] -> BX
	MOVQ 32(AX), R14
	MOVQ 40(AX), R15
	MOVQ 48(AX), CX
	MOVQ 56(AX), BX
	MOVQ res+0(FP), DX
	MOVQ R10, 32(DX)
	MOVQ R11, 40(DX)
	MOVQ R12, 48(DX)
	MOVQ R13, 56(DX)
	MOVQ R14, R10
	MOVQ R15, R11
	MOVQ CX, R12
	MOVQ BX, R13

	// Add(&x.A0, &x.A1)
	ADDQ SI, R14
	ADCQ DI, R15
	ADCQ R8, CX
	ADCQ R9, BX
	XORQ BP, BP

	// Sub(&x.A0, &x.A1)
	SUBQ    R10, SI
	SBBQ    R11, DI
	SBBQ    R12, R8
	SBBQ    R13, R9
	MOVQ    $0x3c208c16d87cfd47, R10
	MOVQ    $0x97816a916871ca8d, R11
	MOVQ    $0xb85045b68181585d, R12
	MOVQ    $0x30644e72e131a029, R13
	CMOVQCC BP, R10
	CMOVQCC BP, R11
	CMOVQCC BP, R12
	CMOVQCC BP, R13
	ADDQ    R10, SI
	ADCQ    R11, DI
	ADCQ    R12, R8
	ADCQ    R13, R9

	// mul (R14,R15,CX,BX) with (SI,DI,R8,R9) into (R10,R11,R12,R13)
	MUL()

	// reduce element(R10,R11,R12,R13) using temp registers (R14,R15,CX,BX)
	REDUCE(R10,R11,R12,R13,R14,R15,CX,BX)

	MOVQ res+0(FP), AX
	MOVQ R10, 0(AX)
	MOVQ R11, 8(AX)
	MOVQ R12, 16(AX)
	MOVQ R13, 24(AX)
	RET

l5:
	MOVQ res+0(FP), AX
	MOVQ AX, (SP)
	MOVQ x+8(FP), AX
	MOVQ AX, 8(SP)
	CALL ·squareGenericE2(SB)
	RET
