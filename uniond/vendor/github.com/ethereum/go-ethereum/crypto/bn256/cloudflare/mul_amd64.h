#define mul(a0,a1,a2,a3, rb, stack) \
	MOVQ a0, AX \
	MULQ 0+rb \
	MOVQ AX, R8 \
	MOVQ DX, R9 \
	MOVQ a0, AX \
	MULQ 8+rb \
	ADDQ AX, R9 \
	ADCQ $0, DX \
	MOVQ DX, R10 \
	MOVQ a0, AX \
	MULQ 16+rb \
	ADDQ AX, R10 \
	ADCQ $0, DX \
	MOVQ DX, R11 \
	MOVQ a0, AX \
	MULQ 24+rb \
	ADDQ AX, R11 \
	ADCQ $0, DX \
	MOVQ DX, R12 \
	\
	storeBlock(R8,R9,R10,R11, 0+stack) \
	MOVQ R12, 32+stack \
	\
	MOVQ a1, AX \
	MULQ 0+rb \
	MOVQ AX, R8 \
	MOVQ DX, R9 \
	MOVQ a1, AX \
	MULQ 8+rb \
	ADDQ AX, R9 \
	ADCQ $0, DX \
	MOVQ DX, R10 \
	MOVQ a1, AX \
	MULQ 16+rb \
	ADDQ AX, R10 \
	ADCQ $0, DX \
	MOVQ DX, R11 \
	MOVQ a1, AX \
	MULQ 24+rb \
	ADDQ AX, R11 \
	ADCQ $0, DX \
	MOVQ DX, R12 \
	\
	ADDQ 8+stack, R8 \
	ADCQ 16+stack, R9 \
	ADCQ 24+stack, R10 \
	ADCQ 32+stack, R11 \
	ADCQ $0, R12 \
	storeBlock(R8,R9,R10,R11, 8+stack) \
	MOVQ R12, 40+stack \
	\
	MOVQ a2, AX \
	MULQ 0+rb \
	MOVQ AX, R8 \
	MOVQ DX, R9 \
	MOVQ a2, AX \
	MULQ 8+rb \
	ADDQ AX, R9 \
	ADCQ $0, DX \
	MOVQ DX, R10 \
	MOVQ a2, AX \
	MULQ 16+rb \
	ADDQ AX, R10 \
	ADCQ $0, DX \
	MOVQ DX, R11 \
	MOVQ a2, AX \
	MULQ 24+rb \
	ADDQ AX, R11 \
	ADCQ $0, DX \
	MOVQ DX, R12 \
	\
	ADDQ 16+stack, R8 \
	ADCQ 24+stack, R9 \
	ADCQ 32+stack, R10 \
	ADCQ 40+stack, R11 \
	ADCQ $0, R12 \
	storeBlock(R8,R9,R10,R11, 16+stack) \
	MOVQ R12, 48+stack \
	\
	MOVQ a3, AX \
	MULQ 0+rb \
	MOVQ AX, R8 \
	MOVQ DX, R9 \
	MOVQ a3, AX \
	MULQ 8+rb \
	ADDQ AX, R9 \
	ADCQ $0, DX \
	MOVQ DX, R10 \
	MOVQ a3, AX \
	MULQ 16+rb \
	ADDQ AX, R10 \
	ADCQ $0, DX \
	MOVQ DX, R11 \
	MOVQ a3, AX \
	MULQ 24+rb \
	ADDQ AX, R11 \
	ADCQ $0, DX \
	MOVQ DX, R12 \
	\
	ADDQ 24+stack, R8 \
	ADCQ 32+stack, R9 \
	ADCQ 40+stack, R10 \
	ADCQ 48+stack, R11 \
	ADCQ $0, R12 \
	storeBlock(R8,R9,R10,R11, 24+stack) \
	MOVQ R12, 56+stack

#define gfpReduce(stack) \
	\ // m = (T * N') mod R, store m in R8:R9:R10:R11
	MOVQ ·np+0(SB), AX \
	MULQ 0+stack \
	MOVQ AX, R8 \
	MOVQ DX, R9 \
	MOVQ ·np+0(SB), AX \
	MULQ 8+stack \
	ADDQ AX, R9 \
	ADCQ $0, DX \
	MOVQ DX, R10 \
	MOVQ ·np+0(SB), AX \
	MULQ 16+stack \
	ADDQ AX, R10 \
	ADCQ $0, DX \
	MOVQ DX, R11 \
	MOVQ ·np+0(SB), AX \
	MULQ 24+stack \
	ADDQ AX, R11 \
	\
	MOVQ ·np+8(SB), AX \
	MULQ 0+stack \
	MOVQ AX, R12 \
	MOVQ DX, R13 \
	MOVQ ·np+8(SB), AX \
	MULQ 8+stack \
	ADDQ AX, R13 \
	ADCQ $0, DX \
	MOVQ DX, R14 \
	MOVQ ·np+8(SB), AX \
	MULQ 16+stack \
	ADDQ AX, R14 \
	\
	ADDQ R12, R9 \
	ADCQ R13, R10 \
	ADCQ R14, R11 \
	\
	MOVQ ·np+16(SB), AX \
	MULQ 0+stack \
	MOVQ AX, R12 \
	MOVQ DX, R13 \
	MOVQ ·np+16(SB), AX \
	MULQ 8+stack \
	ADDQ AX, R13 \
	\
	ADDQ R12, R10 \
	ADCQ R13, R11 \
	\
	MOVQ ·np+24(SB), AX \
	MULQ 0+stack \
	ADDQ AX, R11 \
	\
	storeBlock(R8,R9,R10,R11, 64+stack) \
	\
	\ // m * N
	mul(·p2+0(SB),·p2+8(SB),·p2+16(SB),·p2+24(SB), 64+stack, 96+stack) \
	\
	\ // Add the 512-bit intermediate to m*N
	loadBlock(96+stack, R8,R9,R10,R11) \
	loadBlock(128+stack, R12,R13,R14,CX) \
	\
	MOVQ $0, AX \
	ADDQ 0+stack, R8 \
	ADCQ 8+stack, R9 \
	ADCQ 16+stack, R10 \
	ADCQ 24+stack, R11 \
	ADCQ 32+stack, R12 \
	ADCQ 40+stack, R13 \
	ADCQ 48+stack, R14 \
	ADCQ 56+stack, CX \
	ADCQ $0, AX \
	\
	gfpCarry(R12,R13,R14,CX,AX, R8,R9,R10,R11,BX)
