// +build arm64,!generic

#define storeBlock(a0,a1,a2,a3, r) \
	MOVD a0,  0+r \
	MOVD a1,  8+r \
	MOVD a2, 16+r \
	MOVD a3, 24+r

#define loadBlock(r, a0,a1,a2,a3) \
	MOVD  0+r, a0 \
	MOVD  8+r, a1 \
	MOVD 16+r, a2 \
	MOVD 24+r, a3

#define loadModulus(p0,p1,p2,p3) \
	MOVD ·p2+0(SB), p0 \
	MOVD ·p2+8(SB), p1 \
	MOVD ·p2+16(SB), p2 \
	MOVD ·p2+24(SB), p3

#include "mul_arm64.h"

TEXT ·gfpNeg(SB),0,$0-16
	MOVD a+8(FP), R0
	loadBlock(0(R0), R1,R2,R3,R4)
	loadModulus(R5,R6,R7,R8)

	SUBS R1, R5, R1
	SBCS R2, R6, R2
	SBCS R3, R7, R3
	SBCS R4, R8, R4

	SUBS R5, R1, R5
	SBCS R6, R2, R6
	SBCS R7, R3, R7
	SBCS R8, R4, R8

	CSEL CS, R5, R1, R1
	CSEL CS, R6, R2, R2
	CSEL CS, R7, R3, R3
	CSEL CS, R8, R4, R4

	MOVD c+0(FP), R0
	storeBlock(R1,R2,R3,R4, 0(R0))
	RET

TEXT ·gfpAdd(SB),0,$0-24
	MOVD a+8(FP), R0
	loadBlock(0(R0), R1,R2,R3,R4)
	MOVD b+16(FP), R0
	loadBlock(0(R0), R5,R6,R7,R8)
	loadModulus(R9,R10,R11,R12)
	MOVD ZR, R0

	ADDS R5, R1
	ADCS R6, R2
	ADCS R7, R3
	ADCS R8, R4
	ADCS ZR, R0

	SUBS  R9, R1, R5
	SBCS R10, R2, R6
	SBCS R11, R3, R7
	SBCS R12, R4, R8
	SBCS  ZR, R0, R0

	CSEL CS, R5, R1, R1
	CSEL CS, R6, R2, R2
	CSEL CS, R7, R3, R3
	CSEL CS, R8, R4, R4

	MOVD c+0(FP), R0
	storeBlock(R1,R2,R3,R4, 0(R0))
	RET

TEXT ·gfpSub(SB),0,$0-24
	MOVD a+8(FP), R0
	loadBlock(0(R0), R1,R2,R3,R4)
	MOVD b+16(FP), R0
	loadBlock(0(R0), R5,R6,R7,R8)
	loadModulus(R9,R10,R11,R12)

	SUBS R5, R1
	SBCS R6, R2
	SBCS R7, R3
	SBCS R8, R4

	CSEL CS, ZR,  R9,  R9
	CSEL CS, ZR, R10, R10
	CSEL CS, ZR, R11, R11
	CSEL CS, ZR, R12, R12

	ADDS  R9, R1
	ADCS R10, R2
	ADCS R11, R3
	ADCS R12, R4

	MOVD c+0(FP), R0
	storeBlock(R1,R2,R3,R4, 0(R0))
	RET

TEXT ·gfpMul(SB),0,$0-24
	MOVD a+8(FP), R0
	loadBlock(0(R0), R1,R2,R3,R4)
	MOVD b+16(FP), R0
	loadBlock(0(R0), R5,R6,R7,R8)

	mul(R9,R10,R11,R12,R13,R14,R15,R16)
	gfpReduce()

	MOVD c+0(FP), R0
	storeBlock(R1,R2,R3,R4, 0(R0))
	RET
