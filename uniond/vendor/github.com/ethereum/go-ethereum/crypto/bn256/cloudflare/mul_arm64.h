#define mul(c0,c1,c2,c3,c4,c5,c6,c7) \
	MUL R1, R5, c0 \
	UMULH R1, R5, c1 \
	MUL R1, R6, R0 \
	ADDS R0, c1 \
	UMULH R1, R6, c2 \
	MUL R1, R7, R0 \
	ADCS R0, c2 \
	UMULH R1, R7, c3 \
	MUL R1, R8, R0 \
	ADCS R0, c3 \
	UMULH R1, R8, c4 \
	ADCS ZR, c4 \
	\
	MUL R2, R5, R1 \
	UMULH R2, R5, R26 \
	MUL R2, R6, R0 \
	ADDS R0, R26 \
	UMULH R2, R6, R27 \
	MUL R2, R7, R0 \
	ADCS R0, R27 \
	UMULH R2, R7, R29 \
	MUL R2, R8, R0 \
	ADCS R0, R29 \
	UMULH R2, R8, c5 \
	ADCS ZR, c5 \
	ADDS R1, c1 \
	ADCS R26, c2 \
	ADCS R27, c3 \
	ADCS R29, c4 \
	ADCS  ZR, c5 \
	\
	MUL R3, R5, R1 \
	UMULH R3, R5, R26 \
	MUL R3, R6, R0 \
	ADDS R0, R26 \
	UMULH R3, R6, R27 \
	MUL R3, R7, R0 \
	ADCS R0, R27 \
	UMULH R3, R7, R29 \
	MUL R3, R8, R0 \
	ADCS R0, R29 \
	UMULH R3, R8, c6 \
	ADCS ZR, c6 \
	ADDS R1, c2 \
	ADCS R26, c3 \
	ADCS R27, c4 \
	ADCS R29, c5 \
	ADCS  ZR, c6 \
	\
	MUL R4, R5, R1 \
	UMULH R4, R5, R26 \
	MUL R4, R6, R0 \
	ADDS R0, R26 \
	UMULH R4, R6, R27 \
	MUL R4, R7, R0 \
	ADCS R0, R27 \
	UMULH R4, R7, R29 \
	MUL R4, R8, R0 \
	ADCS R0, R29 \
	UMULH R4, R8, c7 \
	ADCS ZR, c7 \
	ADDS R1, c3 \
	ADCS R26, c4 \
	ADCS R27, c5 \
	ADCS R29, c6 \
	ADCS  ZR, c7

#define gfpReduce() \
	\ // m = (T * N') mod R, store m in R1:R2:R3:R4
	MOVD ·np+0(SB), R17 \
	MOVD ·np+8(SB), R25 \
	MOVD ·np+16(SB), R19 \
	MOVD ·np+24(SB), R20 \
	\
	MUL R9, R17, R1 \
	UMULH R9, R17, R2 \
	MUL R9, R25, R0 \
	ADDS R0, R2 \
	UMULH R9, R25, R3 \
	MUL R9, R19, R0 \
	ADCS R0, R3 \
	UMULH R9, R19, R4 \
	MUL R9, R20, R0 \
	ADCS R0, R4 \
	\
	MUL R10, R17, R21 \
	UMULH R10, R17, R22 \
	MUL R10, R25, R0 \
	ADDS R0, R22 \
	UMULH R10, R25, R23 \
	MUL R10, R19, R0 \
	ADCS R0, R23 \
	ADDS R21, R2 \
	ADCS R22, R3 \
	ADCS R23, R4 \
	\
	MUL R11, R17, R21 \
	UMULH R11, R17, R22 \
	MUL R11, R25, R0 \
	ADDS R0, R22 \
	ADDS R21, R3 \
	ADCS R22, R4 \
	\
	MUL R12, R17, R21 \
	ADDS R21, R4 \
	\
	\ // m * N
	loadModulus(R5,R6,R7,R8) \
	mul(R17,R25,R19,R20,R21,R22,R23,R24) \
	\
	\ // Add the 512-bit intermediate to m*N
	MOVD  ZR, R0 \
	ADDS  R9, R17 \
	ADCS R10, R25 \
	ADCS R11, R19 \
	ADCS R12, R20 \
	ADCS R13, R21 \
	ADCS R14, R22 \
	ADCS R15, R23 \
	ADCS R16, R24 \
	ADCS  ZR, R0 \
	\
	\ // Our output is R21:R22:R23:R24. Reduce mod p if necessary.
	SUBS R5, R21, R10 \
	SBCS R6, R22, R11 \
	SBCS R7, R23, R12 \
	SBCS R8, R24, R13 \
	\
	CSEL CS, R10, R21, R1 \
	CSEL CS, R11, R22, R2 \
	CSEL CS, R12, R23, R3 \
	CSEL CS, R13, R24, R4
