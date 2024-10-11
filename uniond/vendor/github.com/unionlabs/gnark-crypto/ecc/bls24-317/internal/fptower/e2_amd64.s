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
DATA q<>+0(SB)/8, $0x8d512e565dab2aab
DATA q<>+8(SB)/8, $0xd6f339e43424bf7e
DATA q<>+16(SB)/8, $0x169a61e684c73446
DATA q<>+24(SB)/8, $0xf28fc5a0b7f9d039
DATA q<>+32(SB)/8, $0x1058ca226f60892c
GLOBL q<>(SB), (RODATA+NOPTR), $40

// qInv0 q'[0]
DATA qInv0<>(SB)/8, $0x55b5e0028b047ffd
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
	MOVQ    $0x8d512e565dab2aab, R9
	MOVQ    $0xd6f339e43424bf7e, R10
	MOVQ    $0x169a61e684c73446, R11
	MOVQ    $0xf28fc5a0b7f9d039, R12
	MOVQ    $0x1058ca226f60892c, R13
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
	MOVQ    $0x8d512e565dab2aab, R15
	MOVQ    $0xd6f339e43424bf7e, R9
	MOVQ    $0x169a61e684c73446, R10
	MOVQ    $0xf28fc5a0b7f9d039, R11
	MOVQ    $0x1058ca226f60892c, R12
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
	MOVQ $0x8d512e565dab2aab, CX
	SUBQ BX, CX
	MOVQ CX, 0(DX)
	MOVQ $0xd6f339e43424bf7e, CX
	SBBQ SI, CX
	MOVQ CX, 8(DX)
	MOVQ $0x169a61e684c73446, CX
	SBBQ DI, CX
	MOVQ CX, 16(DX)
	MOVQ $0xf28fc5a0b7f9d039, CX
	SBBQ R8, CX
	MOVQ CX, 24(DX)
	MOVQ $0x1058ca226f60892c, CX
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
	MOVQ $0x8d512e565dab2aab, CX
	SUBQ BX, CX
	MOVQ CX, 40(DX)
	MOVQ $0xd6f339e43424bf7e, CX
	SBBQ SI, CX
	MOVQ CX, 48(DX)
	MOVQ $0x169a61e684c73446, CX
	SBBQ DI, CX
	MOVQ CX, 56(DX)
	MOVQ $0xf28fc5a0b7f9d039, CX
	SBBQ R8, CX
	MOVQ CX, 64(DX)
	MOVQ $0x1058ca226f60892c, CX
	SBBQ R9, CX
	MOVQ CX, 72(DX)
	RET
