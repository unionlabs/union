#define mulBMI2(a0,a1,a2,a3, rb) \
	MOVQ a0, DX \
	MOVQ $0, R13 \
	MULXQ 0+rb, R8, R9 \
	MULXQ 8+rb, AX, R10 \
	ADDQ AX, R9 \
	MULXQ 16+rb, AX, R11 \
	ADCQ AX, R10 \
	MULXQ 24+rb, AX, R12 \
	ADCQ AX, R11 \
	ADCQ $0, R12 \
	ADCQ $0, R13 \
	\
	MOVQ a1, DX \
	MOVQ $0, R14 \
	MULXQ 0+rb, AX, BX \
	ADDQ AX, R9 \
	ADCQ BX, R10 \
	MULXQ 16+rb, AX, BX \
	ADCQ AX, R11 \
	ADCQ BX, R12 \
	ADCQ $0, R13 \
	MULXQ 8+rb, AX, BX \
	ADDQ AX, R10 \
	ADCQ BX, R11 \
	MULXQ 24+rb, AX, BX \
	ADCQ AX, R12 \
	ADCQ BX, R13 \
	ADCQ $0, R14 \
	\
	MOVQ a2, DX \
	MOVQ $0, CX \
	MULXQ 0+rb, AX, BX \
	ADDQ AX, R10 \
	ADCQ BX, R11 \
	MULXQ 16+rb, AX, BX \
	ADCQ AX, R12 \
	ADCQ BX, R13 \
	ADCQ $0, R14 \
	MULXQ 8+rb, AX, BX \
	ADDQ AX, R11 \
	ADCQ BX, R12 \
	MULXQ 24+rb, AX, BX \
	ADCQ AX, R13 \
	ADCQ BX, R14 \
	ADCQ $0, CX \
	\
	MOVQ a3, DX \
	MULXQ 0+rb, AX, BX \
	ADDQ AX, R11 \
	ADCQ BX, R12 \
	MULXQ 16+rb, AX, BX \
	ADCQ AX, R13 \
	ADCQ BX, R14 \
	ADCQ $0, CX \
	MULXQ 8+rb, AX, BX \
	ADDQ AX, R12 \
	ADCQ BX, R13 \
	MULXQ 24+rb, AX, BX \
	ADCQ AX, R14 \
	ADCQ BX, CX

#define gfpReduceBMI2() \
	\ // m = (T * N') mod R, store m in R8:R9:R10:R11
	MOVQ ·np+0(SB), DX \
	MULXQ 0(SP), R8, R9 \
	MULXQ 8(SP), AX, R10 \
	ADDQ AX, R9 \
	MULXQ 16(SP), AX, R11 \
	ADCQ AX, R10 \
	MULXQ 24(SP), AX, BX \
	ADCQ AX, R11 \
	\
	MOVQ ·np+8(SB), DX \
	MULXQ 0(SP), AX, BX \
	ADDQ AX, R9 \
	ADCQ BX, R10 \
	MULXQ 16(SP), AX, BX \
	ADCQ AX, R11 \
	MULXQ 8(SP), AX, BX \
	ADDQ AX, R10 \
	ADCQ BX, R11 \
	\
	MOVQ ·np+16(SB), DX \
	MULXQ 0(SP), AX, BX \
	ADDQ AX, R10 \
	ADCQ BX, R11 \
	MULXQ 8(SP), AX, BX \
	ADDQ AX, R11 \
	\
	MOVQ ·np+24(SB), DX \
	MULXQ 0(SP), AX, BX \
	ADDQ AX, R11 \
	\
	storeBlock(R8,R9,R10,R11, 64(SP)) \
	\
	\ // m * N
	mulBMI2(·p2+0(SB),·p2+8(SB),·p2+16(SB),·p2+24(SB), 64(SP)) \
	\
	\ // Add the 512-bit intermediate to m*N
	MOVQ $0, AX \
	ADDQ 0(SP), R8 \
	ADCQ 8(SP), R9 \
	ADCQ 16(SP), R10 \
	ADCQ 24(SP), R11 \
	ADCQ 32(SP), R12 \
	ADCQ 40(SP), R13 \
	ADCQ 48(SP), R14 \
	ADCQ 56(SP), CX \
	ADCQ $0, AX \
	\
	gfpCarry(R12,R13,R14,CX,AX, R8,R9,R10,R11,BX)
