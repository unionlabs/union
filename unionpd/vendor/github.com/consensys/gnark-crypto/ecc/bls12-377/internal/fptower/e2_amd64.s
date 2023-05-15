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
DATA q<>+0(SB)/8, $0x8508c00000000001
DATA q<>+8(SB)/8, $0x170b5d4430000000
DATA q<>+16(SB)/8, $0x1ef3622fba094800
DATA q<>+24(SB)/8, $0x1a22d9f300f5138f
DATA q<>+32(SB)/8, $0xc63b05c06ca1493b
DATA q<>+40(SB)/8, $0x01ae3a4617c510ea
GLOBL q<>(SB), (RODATA+NOPTR), $48

// qInv0 q'[0]
DATA qInv0<>(SB)/8, $0x8508bfffffffffff
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
	MOVQ    $0x8508c00000000001, R10
	MOVQ    $0x170b5d4430000000, R11
	MOVQ    $0x1ef3622fba094800, R12
	MOVQ    $0x1a22d9f300f5138f, R13
	MOVQ    $0xc63b05c06ca1493b, R14
	MOVQ    $0x01ae3a4617c510ea, R15
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
	MOVQ    $0x8508c00000000001, R11
	MOVQ    $0x170b5d4430000000, R12
	MOVQ    $0x1ef3622fba094800, R13
	MOVQ    $0x1a22d9f300f5138f, R14
	MOVQ    $0xc63b05c06ca1493b, R15
	MOVQ    $0x01ae3a4617c510ea, R10
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
	MOVQ $0x8508c00000000001, CX
	SUBQ BX, CX
	MOVQ CX, 0(DX)
	MOVQ $0x170b5d4430000000, CX
	SBBQ SI, CX
	MOVQ CX, 8(DX)
	MOVQ $0x1ef3622fba094800, CX
	SBBQ DI, CX
	MOVQ CX, 16(DX)
	MOVQ $0x1a22d9f300f5138f, CX
	SBBQ R8, CX
	MOVQ CX, 24(DX)
	MOVQ $0xc63b05c06ca1493b, CX
	SBBQ R9, CX
	MOVQ CX, 32(DX)
	MOVQ $0x01ae3a4617c510ea, CX
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
	MOVQ $0x8508c00000000001, CX
	SUBQ BX, CX
	MOVQ CX, 48(DX)
	MOVQ $0x170b5d4430000000, CX
	SBBQ SI, CX
	MOVQ CX, 56(DX)
	MOVQ $0x1ef3622fba094800, CX
	SBBQ DI, CX
	MOVQ CX, 64(DX)
	MOVQ $0x1a22d9f300f5138f, CX
	SBBQ R8, CX
	MOVQ CX, 72(DX)
	MOVQ $0xc63b05c06ca1493b, CX
	SBBQ R9, CX
	MOVQ CX, 80(DX)
	MOVQ $0x01ae3a4617c510ea, CX
	SBBQ R10, CX
	MOVQ CX, 88(DX)
	RET
