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

// mul(res, x, y *Element)
TEXT ·mul(SB), $24-24

	// the algorithm is described in the Element.Mul declaration (.go)
	// however, to benefit from the ADCX and ADOX carry chains
	// we split the inner loops in 2:
	// for i=0 to N-1
	// 		for j=0 to N-1
	// 		    (A,t[j])  := t[j] + x[j]*y[i] + A
	// 		m := t[0]*q'[0] mod W
	// 		C,_ := t[0] + m*q[0]
	// 		for j=1 to N-1
	// 		    (C,t[j-1]) := t[j] + m*q[j] + C
	// 		t[N-1] = C + A

	NO_LOCAL_POINTERS
	CMPB ·supportAdx(SB), $1
	JNE  l1
	MOVQ x+8(FP), DI

	// x[0] -> R9
	// x[1] -> R10
	// x[2] -> R11
	MOVQ 0(DI), R9
	MOVQ 8(DI), R10
	MOVQ 16(DI), R11
	MOVQ y+16(FP), R12

	// A -> BP
	// t[0] -> R14
	// t[1] -> R13
	// t[2] -> CX
	// t[3] -> BX
	// t[4] -> SI
	// clear the flags
	XORQ AX, AX
	MOVQ 0(R12), DX

	// (A,t[0])  := x[0]*y[0] + A
	MULXQ R9, R14, R13

	// (A,t[1])  := x[1]*y[0] + A
	MULXQ R10, AX, CX
	ADOXQ AX, R13

	// (A,t[2])  := x[2]*y[0] + A
	MULXQ R11, AX, BX
	ADOXQ AX, CX

	// (A,t[3])  := x[3]*y[0] + A
	MULXQ 24(DI), AX, SI
	ADOXQ AX, BX

	// (A,t[4])  := x[4]*y[0] + A
	MULXQ 32(DI), AX, BP
	ADOXQ AX, SI

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R14, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R8
	ADCXQ R14, AX
	MOVQ  R8, R14

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R13, R14
	MULXQ q<>+8(SB), AX, R13
	ADOXQ AX, R14

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ CX, R13
	MULXQ q<>+16(SB), AX, CX
	ADOXQ AX, R13

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ BX, CX
	MULXQ q<>+24(SB), AX, BX
	ADOXQ AX, CX

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ SI, BX
	MULXQ q<>+32(SB), AX, SI
	ADOXQ AX, BX

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, SI
	ADOXQ BP, SI

	// clear the flags
	XORQ AX, AX
	MOVQ 8(R12), DX

	// (A,t[0])  := t[0] + x[0]*y[1] + A
	MULXQ R9, AX, BP
	ADOXQ AX, R14

	// (A,t[1])  := t[1] + x[1]*y[1] + A
	ADCXQ BP, R13
	MULXQ R10, AX, BP
	ADOXQ AX, R13

	// (A,t[2])  := t[2] + x[2]*y[1] + A
	ADCXQ BP, CX
	MULXQ R11, AX, BP
	ADOXQ AX, CX

	// (A,t[3])  := t[3] + x[3]*y[1] + A
	ADCXQ BP, BX
	MULXQ 24(DI), AX, BP
	ADOXQ AX, BX

	// (A,t[4])  := t[4] + x[4]*y[1] + A
	ADCXQ BP, SI
	MULXQ 32(DI), AX, BP
	ADOXQ AX, SI

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R14, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R8
	ADCXQ R14, AX
	MOVQ  R8, R14

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R13, R14
	MULXQ q<>+8(SB), AX, R13
	ADOXQ AX, R14

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ CX, R13
	MULXQ q<>+16(SB), AX, CX
	ADOXQ AX, R13

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ BX, CX
	MULXQ q<>+24(SB), AX, BX
	ADOXQ AX, CX

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ SI, BX
	MULXQ q<>+32(SB), AX, SI
	ADOXQ AX, BX

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, SI
	ADOXQ BP, SI

	// clear the flags
	XORQ AX, AX
	MOVQ 16(R12), DX

	// (A,t[0])  := t[0] + x[0]*y[2] + A
	MULXQ R9, AX, BP
	ADOXQ AX, R14

	// (A,t[1])  := t[1] + x[1]*y[2] + A
	ADCXQ BP, R13
	MULXQ R10, AX, BP
	ADOXQ AX, R13

	// (A,t[2])  := t[2] + x[2]*y[2] + A
	ADCXQ BP, CX
	MULXQ R11, AX, BP
	ADOXQ AX, CX

	// (A,t[3])  := t[3] + x[3]*y[2] + A
	ADCXQ BP, BX
	MULXQ 24(DI), AX, BP
	ADOXQ AX, BX

	// (A,t[4])  := t[4] + x[4]*y[2] + A
	ADCXQ BP, SI
	MULXQ 32(DI), AX, BP
	ADOXQ AX, SI

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R14, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R8
	ADCXQ R14, AX
	MOVQ  R8, R14

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R13, R14
	MULXQ q<>+8(SB), AX, R13
	ADOXQ AX, R14

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ CX, R13
	MULXQ q<>+16(SB), AX, CX
	ADOXQ AX, R13

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ BX, CX
	MULXQ q<>+24(SB), AX, BX
	ADOXQ AX, CX

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ SI, BX
	MULXQ q<>+32(SB), AX, SI
	ADOXQ AX, BX

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, SI
	ADOXQ BP, SI

	// clear the flags
	XORQ AX, AX
	MOVQ 24(R12), DX

	// (A,t[0])  := t[0] + x[0]*y[3] + A
	MULXQ R9, AX, BP
	ADOXQ AX, R14

	// (A,t[1])  := t[1] + x[1]*y[3] + A
	ADCXQ BP, R13
	MULXQ R10, AX, BP
	ADOXQ AX, R13

	// (A,t[2])  := t[2] + x[2]*y[3] + A
	ADCXQ BP, CX
	MULXQ R11, AX, BP
	ADOXQ AX, CX

	// (A,t[3])  := t[3] + x[3]*y[3] + A
	ADCXQ BP, BX
	MULXQ 24(DI), AX, BP
	ADOXQ AX, BX

	// (A,t[4])  := t[4] + x[4]*y[3] + A
	ADCXQ BP, SI
	MULXQ 32(DI), AX, BP
	ADOXQ AX, SI

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R14, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R8
	ADCXQ R14, AX
	MOVQ  R8, R14

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R13, R14
	MULXQ q<>+8(SB), AX, R13
	ADOXQ AX, R14

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ CX, R13
	MULXQ q<>+16(SB), AX, CX
	ADOXQ AX, R13

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ BX, CX
	MULXQ q<>+24(SB), AX, BX
	ADOXQ AX, CX

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ SI, BX
	MULXQ q<>+32(SB), AX, SI
	ADOXQ AX, BX

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, SI
	ADOXQ BP, SI

	// clear the flags
	XORQ AX, AX
	MOVQ 32(R12), DX

	// (A,t[0])  := t[0] + x[0]*y[4] + A
	MULXQ R9, AX, BP
	ADOXQ AX, R14

	// (A,t[1])  := t[1] + x[1]*y[4] + A
	ADCXQ BP, R13
	MULXQ R10, AX, BP
	ADOXQ AX, R13

	// (A,t[2])  := t[2] + x[2]*y[4] + A
	ADCXQ BP, CX
	MULXQ R11, AX, BP
	ADOXQ AX, CX

	// (A,t[3])  := t[3] + x[3]*y[4] + A
	ADCXQ BP, BX
	MULXQ 24(DI), AX, BP
	ADOXQ AX, BX

	// (A,t[4])  := t[4] + x[4]*y[4] + A
	ADCXQ BP, SI
	MULXQ 32(DI), AX, BP
	ADOXQ AX, SI

	// A += carries from ADCXQ and ADOXQ
	MOVQ  $0, AX
	ADCXQ AX, BP
	ADOXQ AX, BP

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R14, DX

	// clear the flags
	XORQ AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, R8
	ADCXQ R14, AX
	MOVQ  R8, R14

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R13, R14
	MULXQ q<>+8(SB), AX, R13
	ADOXQ AX, R14

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ CX, R13
	MULXQ q<>+16(SB), AX, CX
	ADOXQ AX, R13

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ BX, CX
	MULXQ q<>+24(SB), AX, BX
	ADOXQ AX, CX

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ SI, BX
	MULXQ q<>+32(SB), AX, SI
	ADOXQ AX, BX

	// t[4] = C + A
	MOVQ  $0, AX
	ADCXQ AX, SI
	ADOXQ BP, SI

	// reduce element(R14,R13,CX,BX,SI) using temp registers (R8,DI,R12,R9,R10)
	REDUCE(R14,R13,CX,BX,SI,R8,DI,R12,R9,R10)

	MOVQ res+0(FP), AX
	MOVQ R14, 0(AX)
	MOVQ R13, 8(AX)
	MOVQ CX, 16(AX)
	MOVQ BX, 24(AX)
	MOVQ SI, 32(AX)
	RET

l1:
	MOVQ res+0(FP), AX
	MOVQ AX, (SP)
	MOVQ x+8(FP), AX
	MOVQ AX, 8(SP)
	MOVQ y+16(FP), AX
	MOVQ AX, 16(SP)
	CALL ·_mulGeneric(SB)
	RET

TEXT ·fromMont(SB), $8-8
	NO_LOCAL_POINTERS

	// the algorithm is described here
	// https://hackmd.io/@gnark/modular_multiplication
	// when y = 1 we have:
	// for i=0 to N-1
	// 		t[i] = x[i]
	// for i=0 to N-1
	// 		m := t[0]*q'[0] mod W
	// 		C,_ := t[0] + m*q[0]
	// 		for j=1 to N-1
	// 		    (C,t[j-1]) := t[j] + m*q[j] + C
	// 		t[N-1] = C
	CMPB ·supportAdx(SB), $1
	JNE  l2
	MOVQ res+0(FP), DX
	MOVQ 0(DX), R14
	MOVQ 8(DX), R13
	MOVQ 16(DX), CX
	MOVQ 24(DX), BX
	MOVQ 32(DX), SI
	XORQ DX, DX

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R14, DX
	XORQ  AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R14, AX
	MOVQ  BP, R14

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R13, R14
	MULXQ q<>+8(SB), AX, R13
	ADOXQ AX, R14

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ CX, R13
	MULXQ q<>+16(SB), AX, CX
	ADOXQ AX, R13

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ BX, CX
	MULXQ q<>+24(SB), AX, BX
	ADOXQ AX, CX

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ SI, BX
	MULXQ q<>+32(SB), AX, SI
	ADOXQ AX, BX
	MOVQ  $0, AX
	ADCXQ AX, SI
	ADOXQ AX, SI
	XORQ  DX, DX

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R14, DX
	XORQ  AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R14, AX
	MOVQ  BP, R14

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R13, R14
	MULXQ q<>+8(SB), AX, R13
	ADOXQ AX, R14

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ CX, R13
	MULXQ q<>+16(SB), AX, CX
	ADOXQ AX, R13

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ BX, CX
	MULXQ q<>+24(SB), AX, BX
	ADOXQ AX, CX

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ SI, BX
	MULXQ q<>+32(SB), AX, SI
	ADOXQ AX, BX
	MOVQ  $0, AX
	ADCXQ AX, SI
	ADOXQ AX, SI
	XORQ  DX, DX

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R14, DX
	XORQ  AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R14, AX
	MOVQ  BP, R14

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R13, R14
	MULXQ q<>+8(SB), AX, R13
	ADOXQ AX, R14

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ CX, R13
	MULXQ q<>+16(SB), AX, CX
	ADOXQ AX, R13

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ BX, CX
	MULXQ q<>+24(SB), AX, BX
	ADOXQ AX, CX

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ SI, BX
	MULXQ q<>+32(SB), AX, SI
	ADOXQ AX, BX
	MOVQ  $0, AX
	ADCXQ AX, SI
	ADOXQ AX, SI
	XORQ  DX, DX

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R14, DX
	XORQ  AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R14, AX
	MOVQ  BP, R14

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R13, R14
	MULXQ q<>+8(SB), AX, R13
	ADOXQ AX, R14

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ CX, R13
	MULXQ q<>+16(SB), AX, CX
	ADOXQ AX, R13

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ BX, CX
	MULXQ q<>+24(SB), AX, BX
	ADOXQ AX, CX

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ SI, BX
	MULXQ q<>+32(SB), AX, SI
	ADOXQ AX, BX
	MOVQ  $0, AX
	ADCXQ AX, SI
	ADOXQ AX, SI
	XORQ  DX, DX

	// m := t[0]*q'[0] mod W
	MOVQ  qInv0<>(SB), DX
	IMULQ R14, DX
	XORQ  AX, AX

	// C,_ := t[0] + m*q[0]
	MULXQ q<>+0(SB), AX, BP
	ADCXQ R14, AX
	MOVQ  BP, R14

	// (C,t[0]) := t[1] + m*q[1] + C
	ADCXQ R13, R14
	MULXQ q<>+8(SB), AX, R13
	ADOXQ AX, R14

	// (C,t[1]) := t[2] + m*q[2] + C
	ADCXQ CX, R13
	MULXQ q<>+16(SB), AX, CX
	ADOXQ AX, R13

	// (C,t[2]) := t[3] + m*q[3] + C
	ADCXQ BX, CX
	MULXQ q<>+24(SB), AX, BX
	ADOXQ AX, CX

	// (C,t[3]) := t[4] + m*q[4] + C
	ADCXQ SI, BX
	MULXQ q<>+32(SB), AX, SI
	ADOXQ AX, BX
	MOVQ  $0, AX
	ADCXQ AX, SI
	ADOXQ AX, SI

	// reduce element(R14,R13,CX,BX,SI) using temp registers (DI,R8,R9,R10,R11)
	REDUCE(R14,R13,CX,BX,SI,DI,R8,R9,R10,R11)

	MOVQ res+0(FP), AX
	MOVQ R14, 0(AX)
	MOVQ R13, 8(AX)
	MOVQ CX, 16(AX)
	MOVQ BX, 24(AX)
	MOVQ SI, 32(AX)
	RET

l2:
	MOVQ res+0(FP), AX
	MOVQ AX, (SP)
	CALL ·_fromMontGeneric(SB)
	RET
