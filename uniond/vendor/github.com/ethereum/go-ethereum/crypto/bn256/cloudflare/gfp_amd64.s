// +build amd64,!generic

#define storeBlock(a0,a1,a2,a3, r) \
	MOVQ a0,  0+r \
	MOVQ a1,  8+r \
	MOVQ a2, 16+r \
	MOVQ a3, 24+r

#define loadBlock(r, a0,a1,a2,a3) \
	MOVQ  0+r, a0 \
	MOVQ  8+r, a1 \
	MOVQ 16+r, a2 \
	MOVQ 24+r, a3

#define gfpCarry(a0,a1,a2,a3,a4, b0,b1,b2,b3,b4) \
	\ // b = a-p
	MOVQ a0, b0 \
	MOVQ a1, b1 \
	MOVQ a2, b2 \
	MOVQ a3, b3 \
	MOVQ a4, b4 \
	\
	SUBQ ·p2+0(SB), b0 \
	SBBQ ·p2+8(SB), b1 \
	SBBQ ·p2+16(SB), b2 \
	SBBQ ·p2+24(SB), b3 \
	SBBQ $0, b4 \
	\
	\ // if b is negative then return a
	\ // else return b
	CMOVQCC b0, a0 \
	CMOVQCC b1, a1 \
	CMOVQCC b2, a2 \
	CMOVQCC b3, a3

#include "mul_amd64.h"
#include "mul_bmi2_amd64.h"

TEXT ·gfpNeg(SB),0,$0-16
	MOVQ ·p2+0(SB), R8
	MOVQ ·p2+8(SB), R9
	MOVQ ·p2+16(SB), R10
	MOVQ ·p2+24(SB), R11

	MOVQ a+8(FP), DI
	SUBQ 0(DI), R8
	SBBQ 8(DI), R9
	SBBQ 16(DI), R10
	SBBQ 24(DI), R11

	MOVQ $0, AX
	gfpCarry(R8,R9,R10,R11,AX, R12,R13,R14,CX,BX)

	MOVQ c+0(FP), DI
	storeBlock(R8,R9,R10,R11, 0(DI))
	RET

TEXT ·gfpAdd(SB),0,$0-24
	MOVQ a+8(FP), DI
	MOVQ b+16(FP), SI

	loadBlock(0(DI), R8,R9,R10,R11)
	MOVQ $0, R12

	ADDQ  0(SI), R8
	ADCQ  8(SI), R9
	ADCQ 16(SI), R10
	ADCQ 24(SI), R11
	ADCQ $0, R12

	gfpCarry(R8,R9,R10,R11,R12, R13,R14,CX,AX,BX)

	MOVQ c+0(FP), DI
	storeBlock(R8,R9,R10,R11, 0(DI))
	RET

TEXT ·gfpSub(SB),0,$0-24
	MOVQ a+8(FP), DI
	MOVQ b+16(FP), SI

	loadBlock(0(DI), R8,R9,R10,R11)

	MOVQ ·p2+0(SB), R12
	MOVQ ·p2+8(SB), R13
	MOVQ ·p2+16(SB), R14
	MOVQ ·p2+24(SB), CX
	MOVQ $0, AX

	SUBQ  0(SI), R8
	SBBQ  8(SI), R9
	SBBQ 16(SI), R10
	SBBQ 24(SI), R11

	CMOVQCC AX, R12
	CMOVQCC AX, R13
	CMOVQCC AX, R14
	CMOVQCC AX, CX

	ADDQ R12, R8
	ADCQ R13, R9
	ADCQ R14, R10
	ADCQ CX, R11

	MOVQ c+0(FP), DI
	storeBlock(R8,R9,R10,R11, 0(DI))
	RET

TEXT ·gfpMul(SB),0,$160-24
	MOVQ a+8(FP), DI
	MOVQ b+16(FP), SI

	// Jump to a slightly different implementation if MULX isn't supported.
	CMPB ·hasBMI2(SB), $0
	JE   nobmi2Mul

	mulBMI2(0(DI),8(DI),16(DI),24(DI), 0(SI))
	storeBlock( R8, R9,R10,R11,  0(SP))
	storeBlock(R12,R13,R14,CX, 32(SP))
	gfpReduceBMI2()
	JMP end

nobmi2Mul:
	mul(0(DI),8(DI),16(DI),24(DI), 0(SI), 0(SP))
	gfpReduce(0(SP))

end:
	MOVQ c+0(FP), DI
	storeBlock(R12,R13,R14,CX, 0(DI))
	RET
