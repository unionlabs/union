// +build amd64,blsasm amd64,blsadx

#include "textflag.h"

// addition w/ modular reduction
// a = (a + b) % p
TEXT ·addAssign(SB), NOSPLIT, $0-16
	// |
	MOVQ a+0(FP), DI
	MOVQ b+8(FP), SI

	// |
	MOVQ (DI), R8
	MOVQ 8(DI), R9
	MOVQ 16(DI), R10
	MOVQ 24(DI), R11
	MOVQ 32(DI), R12
	MOVQ 40(DI), R13

	// |
	ADDQ (SI), R8
	ADCQ 8(SI), R9
	ADCQ 16(SI), R10
	ADCQ 24(SI), R11
	ADCQ 32(SI), R12
	ADCQ 40(SI), R13

	// |
	MOVQ R8, R14
	MOVQ R9, R15
	MOVQ R10, CX
	MOVQ R11, DX
	MOVQ R12, SI
	MOVQ R13, BX
	MOVQ $0xb9feffffffffaaab, AX
	SUBQ AX, R14
	MOVQ $0x1eabfffeb153ffff, AX
	SBBQ AX, R15
	MOVQ $0x6730d2a0f6b0f624, AX
	SBBQ AX, CX
	MOVQ $0x64774b84f38512bf, AX
	SBBQ AX, DX
	MOVQ $0x4b1ba7b6434bacd7, AX
	SBBQ AX, SI
	MOVQ $0x1a0111ea397fe69a, AX
	SBBQ AX, BX
	CMOVQCC R14, R8
	CMOVQCC R15, R9
	CMOVQCC CX, R10
	CMOVQCC DX, R11
	CMOVQCC SI, R12
	CMOVQCC BX, R13

	// |
	MOVQ R8, (DI)
	MOVQ R9, 8(DI)
	MOVQ R10, 16(DI)
	MOVQ R11, 24(DI)
	MOVQ R12, 32(DI)
	MOVQ R13, 40(DI)
	RET

/*	 | end											*/


// addition w/ modular reduction
// c = (a + b) % p
TEXT ·add(SB), NOSPLIT, $0-24
	// |
	MOVQ a+8(FP), DI
	MOVQ b+16(FP), SI

	// |
	MOVQ (DI), R8
	MOVQ 8(DI), R9
	MOVQ 16(DI), R10
	MOVQ 24(DI), R11
	MOVQ 32(DI), R12
	MOVQ 40(DI), R13

	// |
	ADDQ (SI), R8
	ADCQ 8(SI), R9
	ADCQ 16(SI), R10
	ADCQ 24(SI), R11
	ADCQ 32(SI), R12
	ADCQ 40(SI), R13

	// |
	MOVQ R8, R14
	MOVQ R9, R15
	MOVQ R10, CX
	MOVQ R11, DX
	MOVQ R12, SI
	MOVQ R13, BX
	MOVQ $0xb9feffffffffaaab, DI
	SUBQ DI, R14
	MOVQ $0x1eabfffeb153ffff, DI
	SBBQ DI, R15
	MOVQ $0x6730d2a0f6b0f624, DI
	SBBQ DI, CX
	MOVQ $0x64774b84f38512bf, DI
	SBBQ DI, DX
	MOVQ $0x4b1ba7b6434bacd7, DI
	SBBQ DI, SI
	MOVQ $0x1a0111ea397fe69a, DI
	SBBQ DI, BX
	CMOVQCC R14, R8
	CMOVQCC R15, R9
	CMOVQCC CX, R10
	CMOVQCC DX, R11
	CMOVQCC SI, R12
	CMOVQCC BX, R13

	// |
	MOVQ c+0(FP), DI
	MOVQ R8, (DI)
	MOVQ R9, 8(DI)
	MOVQ R10, 16(DI)
	MOVQ R11, 24(DI)
	MOVQ R12, 32(DI)
	MOVQ R13, 40(DI)
	RET
/*	 | end													*/


// addition w/o reduction check
// c = (a + b)
TEXT ·ladd(SB), NOSPLIT, $0-24
	// |
	MOVQ a+8(FP), DI
	MOVQ b+16(FP), SI

	// |
	MOVQ (DI), R8
	MOVQ 8(DI), R9
	MOVQ 16(DI), R10
	MOVQ 24(DI), R11
	MOVQ 32(DI), R12
	MOVQ 40(DI), R13

	// |
	ADDQ (SI), R8
	ADCQ 8(SI), R9
	ADCQ 16(SI), R10
	ADCQ 24(SI), R11
	ADCQ 32(SI), R12
	ADCQ 40(SI), R13

	// |
	MOVQ c+0(FP), DI
	MOVQ R8, (DI)
	MOVQ R9, 8(DI)
	MOVQ R10, 16(DI)
	MOVQ R11, 24(DI)
	MOVQ R12, 32(DI)
	MOVQ R13, 40(DI)
	RET
/*	 | end													*/


// addition w/o reduction check
// a = a + b
TEXT ·laddAssign(SB), NOSPLIT, $0-16
	// |
	MOVQ a+0(FP), DI
	MOVQ b+8(FP), SI

	// |
	MOVQ (DI), R8
	MOVQ 8(DI), R9
	MOVQ 16(DI), R10
	MOVQ 24(DI), R11
	MOVQ 32(DI), R12
	MOVQ 40(DI), R13

	// |
	ADDQ (SI), R8
	ADCQ 8(SI), R9
	ADCQ 16(SI), R10
	ADCQ 24(SI), R11
	ADCQ 32(SI), R12
	ADCQ 40(SI), R13

	// |
	MOVQ a+0(FP), DI
	MOVQ R8, (DI)
	MOVQ R9, 8(DI)
	MOVQ R10, 16(DI)
	MOVQ R11, 24(DI)
	MOVQ R12, 32(DI)
	MOVQ R13, 40(DI)
	RET
/*	 | end													*/


// subtraction w/ modular reduction
// c = (a - b) % p
TEXT ·sub(SB), NOSPLIT, $0-24
	// |
	MOVQ a+8(FP), DI
	MOVQ b+16(FP), SI
	XORQ AX, AX

	// |
	MOVQ (DI), R8
	MOVQ 8(DI), R9
	MOVQ 16(DI), R10
	MOVQ 24(DI), R11
	MOVQ 32(DI), R12
	MOVQ 40(DI), R13
	SUBQ (SI), R8
	SBBQ 8(SI), R9
	SBBQ 16(SI), R10
	SBBQ 24(SI), R11
	SBBQ 32(SI), R12
	SBBQ 40(SI), R13

	// |
	MOVQ $0xb9feffffffffaaab, R14
	MOVQ $0x1eabfffeb153ffff, R15
	MOVQ $0x6730d2a0f6b0f624, CX
	MOVQ $0x64774b84f38512bf, DX
	MOVQ $0x4b1ba7b6434bacd7, SI
	MOVQ $0x1a0111ea397fe69a, BX
	CMOVQCC AX, R14
	CMOVQCC AX, R15
	CMOVQCC AX, CX
	CMOVQCC AX, DX
	CMOVQCC AX, SI
	CMOVQCC AX, BX
	ADDQ R14, R8
	ADCQ R15, R9
	ADCQ CX, R10
	ADCQ DX, R11
	ADCQ SI, R12
	ADCQ BX, R13

	// |
	MOVQ c+0(FP), DI
	MOVQ R8, (DI)
	MOVQ R9, 8(DI)
	MOVQ R10, 16(DI)
	MOVQ R11, 24(DI)
	MOVQ R12, 32(DI)
	MOVQ R13, 40(DI)
	RET
/*	 | end													*/


// subtraction w/ modular reduction
// a = (a - b) % p
TEXT ·subAssign(SB), NOSPLIT, $0-16
	// |
	MOVQ a+0(FP), DI
	MOVQ b+8(FP), SI
	XORQ AX, AX

	// |
	MOVQ (DI), R8
	MOVQ 8(DI), R9
	MOVQ 16(DI), R10
	MOVQ 24(DI), R11
	MOVQ 32(DI), R12
	MOVQ 40(DI), R13
	SUBQ (SI), R8
	SBBQ 8(SI), R9
	SBBQ 16(SI), R10
	SBBQ 24(SI), R11
	SBBQ 32(SI), R12
	SBBQ 40(SI), R13

	// |
	MOVQ $0xb9feffffffffaaab, R14
	MOVQ $0x1eabfffeb153ffff, R15
	MOVQ $0x6730d2a0f6b0f624, CX
	MOVQ $0x64774b84f38512bf, DX
	MOVQ $0x4b1ba7b6434bacd7, SI
	MOVQ $0x1a0111ea397fe69a, BX
	CMOVQCC AX, R14
	CMOVQCC AX, R15
	CMOVQCC AX, CX
	CMOVQCC AX, DX
	CMOVQCC AX, SI
	CMOVQCC AX, BX
	ADDQ R14, R8
	ADCQ R15, R9
	ADCQ CX, R10
	ADCQ DX, R11
	ADCQ SI, R12
	ADCQ BX, R13

	// |
	MOVQ a+0(FP), DI
	MOVQ R8, (DI)
	MOVQ R9, 8(DI)
	MOVQ R10, 16(DI)
	MOVQ R11, 24(DI)
	MOVQ R12, 32(DI)
	MOVQ R13, 40(DI)
	RET
/*	 | end													*/


// subtraction w/o reduction check
// a = (a - b)
TEXT ·lsubAssign(SB), NOSPLIT, $0-16
	// |
	MOVQ a+0(FP), DI
	MOVQ b+8(FP), SI

	// |
	MOVQ (DI), R8
	MOVQ 8(DI), R9
	MOVQ 16(DI), R10
	MOVQ 24(DI), R11
	MOVQ 32(DI), R12
	MOVQ 40(DI), R13
	SUBQ (SI), R8
	SBBQ 8(SI), R9
	SBBQ 16(SI), R10
	SBBQ 24(SI), R11
	SBBQ 32(SI), R12
	SBBQ 40(SI), R13
	
	// |
	MOVQ a+0(FP), DI
	MOVQ R8, (DI)
	MOVQ R9, 8(DI)
	MOVQ R10, 16(DI)
	MOVQ R11, 24(DI)
	MOVQ R12, 32(DI)
	MOVQ R13, 40(DI)
	RET
/*	 | end													*/

// doubling w/ reduction
// c = (2 * a) % p
TEXT ·double(SB), NOSPLIT, $0-16
	// |
	MOVQ a+8(FP), DI

	MOVQ (DI), R8
	MOVQ 8(DI), R9
	MOVQ 16(DI), R10
	MOVQ 24(DI), R11
	MOVQ 32(DI), R12
	MOVQ 40(DI), R13
	ADDQ R8, R8
	ADCQ R9, R9
	ADCQ R10, R10
	ADCQ R11, R11
	ADCQ R12, R12
	ADCQ R13, R13

	// |
	MOVQ R8, R14
	MOVQ R9, R15
	MOVQ R10, CX
	MOVQ R11, DX
	MOVQ R12, SI
	MOVQ R13, BX
	MOVQ $0xb9feffffffffaaab, DI
	SUBQ DI, R14
	MOVQ $0x1eabfffeb153ffff, DI
	SBBQ DI, R15
	MOVQ $0x6730d2a0f6b0f624, DI
	SBBQ DI, CX
	MOVQ $0x64774b84f38512bf, DI
	SBBQ DI, DX
	MOVQ $0x4b1ba7b6434bacd7, DI
	SBBQ DI, SI
	MOVQ $0x1a0111ea397fe69a, DI
	SBBQ DI, BX
	CMOVQCC R14, R8
	CMOVQCC R15, R9
	CMOVQCC CX, R10
	CMOVQCC DX, R11
	CMOVQCC SI, R12
	CMOVQCC BX, R13

	// |
	MOVQ c+0(FP), DI
	MOVQ R8, (DI)
	MOVQ R9, 8(DI)
	MOVQ R10, 16(DI)
	MOVQ R11, 24(DI)
	MOVQ R12, 32(DI)
	MOVQ R13, 40(DI)
	RET
/*	 | end													*/


// doubling w/ reduction
// a = (2 * a) % p
TEXT ·doubleAssign(SB), NOSPLIT, $0-8
	// |
	MOVQ a+0(FP), DI

	MOVQ (DI), R8
	MOVQ 8(DI), R9
	MOVQ 16(DI), R10
	MOVQ 24(DI), R11
	MOVQ 32(DI), R12
	MOVQ 40(DI), R13
	ADDQ R8, R8
	ADCQ R9, R9
	ADCQ R10, R10
	ADCQ R11, R11
	ADCQ R12, R12
	ADCQ R13, R13

	// |
	MOVQ R8, R14
	MOVQ R9, R15
	MOVQ R10, CX
	MOVQ R11, DX
	MOVQ R12, SI
	MOVQ R13, BX
	MOVQ $0xb9feffffffffaaab, AX
	SUBQ AX, R14
	MOVQ $0x1eabfffeb153ffff, AX
	SBBQ AX, R15
	MOVQ $0x6730d2a0f6b0f624, AX
	SBBQ AX, CX
	MOVQ $0x64774b84f38512bf, AX
	SBBQ AX, DX
	MOVQ $0x4b1ba7b6434bacd7, AX
	SBBQ AX, SI
	MOVQ $0x1a0111ea397fe69a, AX
	SBBQ AX, BX
	CMOVQCC R14, R8
	CMOVQCC R15, R9
	CMOVQCC CX, R10
	CMOVQCC DX, R11
	CMOVQCC SI, R12
	CMOVQCC BX, R13

	MOVQ R8, (DI)
	MOVQ R9, 8(DI)
	MOVQ R10, 16(DI)
	MOVQ R11, 24(DI)
	MOVQ R12, 32(DI)
	MOVQ R13, 40(DI)
	RET
/*	 | end													*/


// doubling w/o reduction
// c = 2 * a
TEXT ·ldouble(SB), NOSPLIT, $0-16
	// |
	MOVQ a+8(FP), DI

	MOVQ (DI), R8
	MOVQ 8(DI), R9
	MOVQ 16(DI), R10
	MOVQ 24(DI), R11
	MOVQ 32(DI), R12
	MOVQ 40(DI), R13

	// |
	ADDQ R8, R8
	ADCQ R9, R9
	ADCQ R10, R10
	ADCQ R11, R11
	ADCQ R12, R12
	ADCQ R13, R13

	// |
	MOVQ c+0(FP), DI
	MOVQ R8, (DI)
	MOVQ R9, 8(DI)
	MOVQ R10, 16(DI)
	MOVQ R11, 24(DI)
	MOVQ R12, 32(DI)
	MOVQ R13, 40(DI)

	RET
/*	 | end													*/


TEXT ·_neg(SB), NOSPLIT, $0-16
	// |
	MOVQ a+8(FP), DI

	// |
	MOVQ $0xb9feffffffffaaab, R8
	MOVQ $0x1eabfffeb153ffff, R9
	MOVQ $0x6730d2a0f6b0f624, R10
	MOVQ $0x64774b84f38512bf, R11
	MOVQ $0x4b1ba7b6434bacd7, R12
	MOVQ $0x1a0111ea397fe69a, R13
	SUBQ (DI), R8
	SBBQ 8(DI), R9
	SBBQ 16(DI), R10
	SBBQ 24(DI), R11
	SBBQ 32(DI), R12
	SBBQ 40(DI), R13

	// |
	MOVQ c+0(FP), DI
	MOVQ R8, (DI)
	MOVQ R9, 8(DI)
	MOVQ R10, 16(DI)
	MOVQ R11, 24(DI)
	MOVQ R12, 32(DI)
	MOVQ R13, 40(DI)
	RET
/*	 | end													*/


// multiplication without using MULX/ADX
// c = a * b % p
TEXT ·mulNoADX(SB), NOSPLIT, $24-24
	// |

/* inputs                                  */

	MOVQ a+8(FP), DI
	MOVQ b+16(FP), SI
	MOVQ $0x00, R9
	MOVQ $0x00, R10
	MOVQ $0x00, R11
	MOVQ $0x00, R12
	MOVQ $0x00, R13
	MOVQ $0x00, R14
	MOVQ $0x00, R15

	// |

/* i0                                   */

	// | a0 @ CX
	MOVQ (DI), CX

	// | a0 * b0
	MOVQ (SI), AX
	MULQ CX
	MOVQ AX, (SP)
	MOVQ DX, R8

	// | a0 * b1
	MOVQ 8(SI), AX
	MULQ CX
	ADDQ AX, R8
	ADCQ DX, R9

	// | a0 * b2
	MOVQ 16(SI), AX
	MULQ CX
	ADDQ AX, R9
	ADCQ DX, R10

	// | a0 * b3
	MOVQ 24(SI), AX
	MULQ CX
	ADDQ AX, R10
	ADCQ DX, R11

	// | a0 * b4
	MOVQ 32(SI), AX
	MULQ CX
	ADDQ AX, R11
	ADCQ DX, R12

	// | a0 * b5
	MOVQ 40(SI), AX
	MULQ CX
	ADDQ AX, R12
	ADCQ DX, R13

	// |

/* i1                                   */

	// | a1 @ CX
	MOVQ 8(DI), CX
	MOVQ $0x00, BX

	// | a1 * b0
	MOVQ (SI), AX
	MULQ CX
	ADDQ AX, R8
	ADCQ DX, R9
	ADCQ $0x00, R10
	ADCQ $0x00, BX
	MOVQ R8, 8(SP)
	MOVQ $0x00, R8

	// | a1 * b1
	MOVQ 8(SI), AX
	MULQ CX
	ADDQ AX, R9
	ADCQ DX, R10
	ADCQ BX, R11
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a1 * b2
	MOVQ 16(SI), AX
	MULQ CX
	ADDQ AX, R10
	ADCQ DX, R11
	ADCQ BX, R12
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a1 * b3
	MOVQ 24(SI), AX
	MULQ CX
	ADDQ AX, R11
	ADCQ DX, R12
	ADCQ BX, R13
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a1 * b4
	MOVQ 32(SI), AX
	MULQ CX
	ADDQ AX, R12
	ADCQ DX, R13
	ADCQ BX, R14

	// | a1 * b5
	MOVQ 40(SI), AX
	MULQ CX
	ADDQ AX, R13
	ADCQ DX, R14

	// |

/* i2                                   */

	// | a2 @ CX
	MOVQ 16(DI), CX
	MOVQ $0x00, BX

	// | a2 * b0
	MOVQ (SI), AX
	MULQ CX
	ADDQ AX, R9
	ADCQ DX, R10
	ADCQ $0x00, R11
	ADCQ $0x00, BX
	MOVQ R9, 16(SP)
	MOVQ $0x00, R9

	// | a2 * b1
	MOVQ 8(SI), AX
	MULQ CX
	ADDQ AX, R10
	ADCQ DX, R11
	ADCQ BX, R12
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a2 * b2
	MOVQ 16(SI), AX
	MULQ CX
	ADDQ AX, R11
	ADCQ DX, R12
	ADCQ BX, R13
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a2 * b3
	MOVQ 24(SI), AX
	MULQ CX
	ADDQ AX, R12
	ADCQ DX, R13
	ADCQ BX, R14
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a2 * b4
	MOVQ 32(SI), AX
	MULQ CX
	ADDQ AX, R13
	ADCQ DX, R14
	ADCQ BX, R15

	// | a2 * b5
	MOVQ 40(SI), AX
	MULQ CX
	ADDQ AX, R14
	ADCQ DX, R15

	// |

/* i3                                   */

	// | a3 @ CX
	MOVQ 24(DI), CX
	MOVQ $0x00, BX

	// | a3 * b0
	MOVQ (SI), AX
	MULQ CX
	ADDQ AX, R10
	ADCQ DX, R11
	ADCQ $0x00, R12
	ADCQ $0x00, BX

	// | a3 * b1
	MOVQ 8(SI), AX
	MULQ CX
	ADDQ AX, R11
	ADCQ DX, R12
	ADCQ BX, R13
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a3 * b2
	MOVQ 16(SI), AX
	MULQ CX
	ADDQ AX, R12
	ADCQ DX, R13
	ADCQ BX, R14
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a3 * b3
	MOVQ 24(SI), AX
	MULQ CX
	ADDQ AX, R13
	ADCQ DX, R14
	ADCQ BX, R15
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a3 * b4
	MOVQ 32(SI), AX
	MULQ CX
	ADDQ AX, R14
	ADCQ DX, R15
	ADCQ BX, R8

	// | a3 * b5
	MOVQ 40(SI), AX
	MULQ CX
	ADDQ AX, R15
	ADCQ DX, R8

	// |

/* i4                                   */

	// | a4 @ CX
	MOVQ 32(DI), CX
	MOVQ $0x00, BX

	// | a4 * b0
	MOVQ (SI), AX
	MULQ CX
	ADDQ AX, R11
	ADCQ DX, R12
	ADCQ $0x00, R13
	ADCQ $0x00, BX

	// | a4 * b1
	MOVQ 8(SI), AX
	MULQ CX
	ADDQ AX, R12
	ADCQ DX, R13
	ADCQ BX, R14
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a4 * b2
	MOVQ 16(SI), AX
	MULQ CX
	ADDQ AX, R13
	ADCQ DX, R14
	ADCQ BX, R15
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a4 * b3
	MOVQ 24(SI), AX
	MULQ CX
	ADDQ AX, R14
	ADCQ DX, R15
	ADCQ BX, R8
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a4 * b4
	MOVQ 32(SI), AX
	MULQ CX
	ADDQ AX, R15
	ADCQ DX, R8
	ADCQ BX, R9

	// | a4 * b5
	MOVQ 40(SI), AX
	MULQ CX
	ADDQ AX, R8
	ADCQ DX, R9

	// |

/* i5                                   */

	// | a5 @ CX
	MOVQ 40(DI), CX
	MOVQ $0x00, BX

	// | a5 * b0
	MOVQ (SI), AX
	MULQ CX
	ADDQ AX, R12
	ADCQ DX, R13
	ADCQ $0x00, R14
	ADCQ $0x00, BX

	// | a5 * b1
	MOVQ 8(SI), AX
	MULQ CX
	ADDQ AX, R13
	ADCQ DX, R14
	ADCQ BX, R15
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a5 * b2
	MOVQ 16(SI), AX
	MULQ CX
	ADDQ AX, R14
	ADCQ DX, R15
	ADCQ BX, R8
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a5 * b3
	MOVQ 24(SI), AX
	MULQ CX
	ADDQ AX, R15
	ADCQ DX, R8
	ADCQ BX, R9
	MOVQ $0x00, BX
	ADCQ $0x00, BX

	// | a5 * b4
	MOVQ 32(SI), AX
	MULQ CX
	ADDQ AX, R8
	ADCQ DX, R9
	ADCQ $0x00, BX

	// | a5 * b5
	MOVQ 40(SI), AX
	MULQ CX
	ADDQ AX, R9
	ADCQ DX, BX

	// |

/* 			                                     */

	// |
	// | W
	// | 0   (SP)      | 1   8(SP)     | 2   16(SP)    | 3   R10       | 4   R11       | 5   R12
	// | 6   R13       | 7   R14       | 8   R15       | 9   R8        | 10  R9        | 11  BX


	MOVQ (SP), CX
	MOVQ 8(SP), DI
	MOVQ 16(SP), SI
	MOVQ BX, (SP)
	MOVQ R9, 8(SP)

	// |

/* montgomery reduction                    */

	// |

/* i0                                   */

	// |
	// | W
	// | 0   CX        | 1   DI        | 2   SI        | 3   R10       | 4   R11       | 5   R12
	// | 6   R13       | 7   R14       | 8   R15       | 9   R8        | 10  8(SP)     | 11  (SP)


	// | | u0 = w0 * inp
	MOVQ CX, AX
	MULQ ·inp+0(SB)
	MOVQ AX, R9
	MOVQ $0x00, BX

	// |

/*                                         */

	// | j0

	// | w0 @ CX
	MOVQ ·modulus+0(SB), AX
	MULQ R9
	ADDQ AX, CX
	ADCQ DX, BX

	// | j1

	// | w1 @ DI
	MOVQ ·modulus+8(SB), AX
	MULQ R9
	ADDQ AX, DI
	ADCQ $0x00, DX
	ADDQ BX, DI
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j2

	// | w2 @ SI
	MOVQ ·modulus+16(SB), AX
	MULQ R9
	ADDQ AX, SI
	ADCQ $0x00, DX
	ADDQ BX, SI
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j3

	// | w3 @ R10
	MOVQ ·modulus+24(SB), AX
	MULQ R9
	ADDQ AX, R10
	ADCQ $0x00, DX
	ADDQ BX, R10
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j4

	// | w4 @ R11
	MOVQ ·modulus+32(SB), AX
	MULQ R9
	ADDQ AX, R11
	ADCQ $0x00, DX
	ADDQ BX, R11
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j5

	// | w5 @ R12
	MOVQ ·modulus+40(SB), AX
	MULQ R9
	ADDQ AX, R12
	ADCQ $0x00, DX
	ADDQ BX, R12

	// | w6 @ R13
	ADCQ DX, R13
	ADCQ $0x00, CX

	// |

/* i1                                   */

	// |
	// | W
	// | 0   -         | 1   DI        | 2   SI        | 3   R10       | 4   R11       | 5   R12
	// | 6   R13       | 7   R14       | 8   R15       | 9   R8        | 10  8(SP)     | 11  (SP)


	// | | u1 = w1 * inp
	MOVQ DI, AX
	MULQ ·inp+0(SB)
	MOVQ AX, R9
	MOVQ $0x00, BX

	// |

/*                                         */

	// | j0

	// | w1 @ DI
	MOVQ ·modulus+0(SB), AX
	MULQ R9
	ADDQ AX, DI
	ADCQ DX, BX

	// | j1

	// | w2 @ SI
	MOVQ ·modulus+8(SB), AX
	MULQ R9
	ADDQ AX, SI
	ADCQ $0x00, DX
	ADDQ BX, SI
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j2

	// | w3 @ R10
	MOVQ ·modulus+16(SB), AX
	MULQ R9
	ADDQ AX, R10
	ADCQ $0x00, DX
	ADDQ BX, R10
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j3

	// | w4 @ R11
	MOVQ ·modulus+24(SB), AX
	MULQ R9
	ADDQ AX, R11
	ADCQ $0x00, DX
	ADDQ BX, R11
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j4

	// | w5 @ R12
	MOVQ ·modulus+32(SB), AX
	MULQ R9
	ADDQ AX, R12
	ADCQ $0x00, DX
	ADDQ BX, R12
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j5

	// | w6 @ R13
	MOVQ ·modulus+40(SB), AX
	MULQ R9
	ADDQ AX, R13
	ADCQ DX, CX
	ADDQ BX, R13

	// | w7 @ R14
	ADCQ CX, R14
	MOVQ $0x00, CX
	ADCQ $0x00, CX

	// |

/* i2                                   */

	// |
	// | W
	// | 0   -         | 1   -         | 2   SI        | 3   R10       | 4   R11       | 5   R12
	// | 6   R13       | 7   R14       | 8   R15       | 9   R8        | 10  8(SP)     | 11  (SP)


	// | | u2 = w2 * inp
	MOVQ SI, AX
	MULQ ·inp+0(SB)
	MOVQ AX, R9
	MOVQ $0x00, BX

	// |

/*                                         */

	// | j0

	// | w2 @ SI
	MOVQ ·modulus+0(SB), AX
	MULQ R9
	ADDQ AX, SI
	ADCQ DX, BX

	// | j1

	// | w3 @ R10
	MOVQ ·modulus+8(SB), AX
	MULQ R9
	ADDQ AX, R10
	ADCQ $0x00, DX
	ADDQ BX, R10
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j2

	// | w4 @ R11
	MOVQ ·modulus+16(SB), AX
	MULQ R9
	ADDQ AX, R11
	ADCQ $0x00, DX
	ADDQ BX, R11
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j3

	// | w5 @ R12
	MOVQ ·modulus+24(SB), AX
	MULQ R9
	ADDQ AX, R12
	ADCQ $0x00, DX
	ADDQ BX, R12
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j4

	// | w6 @ R13
	MOVQ ·modulus+32(SB), AX
	MULQ R9
	ADDQ AX, R13
	ADCQ $0x00, DX
	ADDQ BX, R13
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j5

	// | w7 @ R14
	MOVQ ·modulus+40(SB), AX
	MULQ R9
	ADDQ AX, R14
	ADCQ DX, CX
	ADDQ BX, R14

	// | w8 @ R15
	ADCQ CX, R15
	MOVQ $0x00, CX
	ADCQ $0x00, CX

	// |

/* i3                                   */

	// |
	// | W
	// | 0   -         | 1   -         | 2   -         | 3   R10       | 4   R11       | 5   R12
	// | 6   R13       | 7   R14       | 8   R15       | 9   R8        | 10  8(SP)     | 11  (SP)


	// | | u3 = w3 * inp
	MOVQ R10, AX
	MULQ ·inp+0(SB)
	MOVQ AX, R9
	MOVQ $0x00, BX

	// |

/*                                         */

	// | j0

	// | w3 @ R10
	MOVQ ·modulus+0(SB), AX
	MULQ R9
	ADDQ AX, R10
	ADCQ DX, BX

	// | j1

	// | w4 @ R11
	MOVQ ·modulus+8(SB), AX
	MULQ R9
	ADDQ AX, R11
	ADCQ $0x00, DX
	ADDQ BX, R11
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j2

	// | w5 @ R12
	MOVQ ·modulus+16(SB), AX
	MULQ R9
	ADDQ AX, R12
	ADCQ $0x00, DX
	ADDQ BX, R12
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j3

	// | w6 @ R13
	MOVQ ·modulus+24(SB), AX
	MULQ R9
	ADDQ AX, R13
	ADCQ $0x00, DX
	ADDQ BX, R13
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j4

	// | w7 @ R14
	MOVQ ·modulus+32(SB), AX
	MULQ R9
	ADDQ AX, R14
	ADCQ $0x00, DX
	ADDQ BX, R14
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j5

	// | w8 @ R15
	MOVQ ·modulus+40(SB), AX
	MULQ R9
	ADDQ AX, R15
	ADCQ DX, CX
	ADDQ BX, R15

	// | w9 @ R8
	ADCQ CX, R8
	MOVQ $0x00, CX
	ADCQ $0x00, CX

	// |

/* i4                                   */

	// |
	// | W
	// | 0   -         | 1   -         | 2   -         | 3   -         | 4   R11       | 5   R12
	// | 6   R13       | 7   R14       | 8   R15       | 9   R8        | 10  8(SP)     | 11  (SP)


	// | | u4 = w4 * inp
	MOVQ R11, AX
	MULQ ·inp+0(SB)
	MOVQ AX, R9
	MOVQ $0x00, BX

	// |

/*                                         */

	// | j0

	// | w4 @ R11
	MOVQ ·modulus+0(SB), AX
	MULQ R9
	ADDQ AX, R11
	ADCQ DX, BX

	// | j1

	// | w5 @ R12
	MOVQ ·modulus+8(SB), AX
	MULQ R9
	ADDQ AX, R12
	ADCQ $0x00, DX
	ADDQ BX, R12
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j2

	// | w6 @ R13
	MOVQ ·modulus+16(SB), AX
	MULQ R9
	ADDQ AX, R13
	ADCQ $0x00, DX
	ADDQ BX, R13
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j3

	// | w7 @ R14
	MOVQ ·modulus+24(SB), AX
	MULQ R9
	ADDQ AX, R14
	ADCQ $0x00, DX
	ADDQ BX, R14
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j4

	// | w8 @ R15
	MOVQ ·modulus+32(SB), AX
	MULQ R9
	ADDQ AX, R15
	ADCQ $0x00, DX
	ADDQ BX, R15
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j5

	// | w9 @ R8
	MOVQ ·modulus+40(SB), AX
	MULQ R9
	ADDQ AX, R8
	ADCQ DX, CX
	ADDQ BX, R8

	// | move to idle register
	MOVQ 8(SP), DI

	// | w10 @ DI
	ADCQ CX, DI
	MOVQ $0x00, CX
	ADCQ $0x00, CX

	// |

/* i5                                   */

	// |
	// | W
	// | 0   -         | 1   -         | 2   -         | 3   -         | 4   -         | 5   R12
	// | 6   R13       | 7   R14       | 8   R15       | 9   R8        | 10  DI        | 11  (SP)


	// | | u5 = w5 * inp
	MOVQ R12, AX
	MULQ ·inp+0(SB)
	MOVQ AX, R9
	MOVQ $0x00, BX

	// |

/*                                         */

	// | j0

	// | w5 @ R12
	MOVQ ·modulus+0(SB), AX
	MULQ R9
	ADDQ AX, R12
	ADCQ DX, BX

	// | j1

	// | w6 @ R13
	MOVQ ·modulus+8(SB), AX
	MULQ R9
	ADDQ AX, R13
	ADCQ $0x00, DX
	ADDQ BX, R13
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j2

	// | w7 @ R14
	MOVQ ·modulus+16(SB), AX
	MULQ R9
	ADDQ AX, R14
	ADCQ $0x00, DX
	ADDQ BX, R14
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j3

	// | w8 @ R15
	MOVQ ·modulus+24(SB), AX
	MULQ R9
	ADDQ AX, R15
	ADCQ $0x00, DX
	ADDQ BX, R15
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j4

	// | w9 @ R8
	MOVQ ·modulus+32(SB), AX
	MULQ R9
	ADDQ AX, R8
	ADCQ $0x00, DX
	ADDQ BX, R8
	MOVQ $0x00, BX
	ADCQ DX, BX

	// | j5

	// | w10 @ DI
	MOVQ ·modulus+40(SB), AX
	MULQ R9
	ADDQ AX, DI
	ADCQ DX, CX
	ADDQ BX, DI

	// | w11 @ CX
	ADCQ (SP), CX

	// |
	// | W montgomerry reduction ends
	// | 0   -         | 1   -         | 2   -         | 3   -         | 4   -         | 5   -
	// | 6   R13       | 7   R14       | 8   R15       | 9   R8        | 10  DI        | 11  CX


	// |


/* modular reduction                       */

	MOVQ R13, R10
	SUBQ ·modulus+0(SB), R10
	MOVQ R14, R11
	SBBQ ·modulus+8(SB), R11
	MOVQ R15, R12
	SBBQ ·modulus+16(SB), R12
	MOVQ R8, AX
	SBBQ ·modulus+24(SB), AX
	MOVQ DI, BX
	SBBQ ·modulus+32(SB), BX
	MOVQ CX, R9
	SBBQ ·modulus+40(SB), R9
	// |

/* out                                     */

	MOVQ    c+0(FP), SI
	CMOVQCC R10, R13
	MOVQ    R13, (SI)
	CMOVQCC R11, R14
	MOVQ    R14, 8(SI)
	CMOVQCC R12, R15
	MOVQ    R15, 16(SI)
	CMOVQCC AX, R8
	MOVQ    R8, 24(SI)
	CMOVQCC BX, DI
	MOVQ    DI, 32(SI)
	CMOVQCC R9, CX
	MOVQ    CX, 40(SI)
	RET

	// |

/* end                                     */


// multiplication
// c = a * b % p
TEXT ·mulADX(SB), NOSPLIT, $16-24
	// |

/* inputs                                  */

	MOVQ a+8(FP), DI
	MOVQ b+16(FP), SI
	XORQ AX, AX

	// |

/* i0                                   */

	// | a0 @ DX
	MOVQ (DI), DX

	// | a0 * b0
	MULXQ (SI), AX, CX
	MOVQ  AX, (SP)

	// | a0 * b1
	MULXQ 8(SI), AX, R8
	ADCXQ AX, CX

	// | a0 * b2
	MULXQ 16(SI), AX, R9
	ADCXQ AX, R8

	// | a0 * b3
	MULXQ 24(SI), AX, R10
	ADCXQ AX, R9

	// | a0 * b4
	MULXQ 32(SI), AX, R11
	ADCXQ AX, R10

	// | a0 * b5
	MULXQ 40(SI), AX, R12
	ADCXQ AX, R11
	ADCQ  $0x00, R12

	// |

/* i1                                   */

	// | a1 @ DX
	MOVQ 8(DI), DX
	XORQ R13, R13

	// | a1 * b0
	MULXQ (SI), AX, BX
	ADOXQ AX, CX
	ADCXQ BX, R8
	MOVQ  CX, 8(SP)

	// | a1 * b1
	MULXQ 8(SI), AX, BX
	ADOXQ AX, R8
	ADCXQ BX, R9

	// | a1 * b2
	MULXQ 16(SI), AX, BX
	ADOXQ AX, R9
	ADCXQ BX, R10

	// | a1 * b3
	MULXQ 24(SI), AX, BX
	ADOXQ AX, R10
	ADCXQ BX, R11

	// | a1 * b4
	MULXQ 32(SI), AX, BX
	ADOXQ AX, R11
	ADCXQ BX, R12

	// | a1 * b5
	MULXQ 40(SI), AX, BX
	ADOXQ AX, R12
	ADOXQ R13, R13
	ADCXQ BX, R13

	// |

/* i2                                   */

	// | a2 @ DX
	MOVQ 16(DI), DX
	XORQ R14, R14

	// | a2 * b0
	MULXQ (SI), AX, BX
	ADOXQ AX, R8
	ADCXQ BX, R9

	// | a2 * b1
	MULXQ 8(SI), AX, BX
	ADOXQ AX, R9
	ADCXQ BX, R10

	// | a2 * b2
	MULXQ 16(SI), AX, BX
	ADOXQ AX, R10
	ADCXQ BX, R11

	// | a2 * b3
	MULXQ 24(SI), AX, BX
	ADOXQ AX, R11
	ADCXQ BX, R12

	// | a2 * b4
	MULXQ 32(SI), AX, BX
	ADOXQ AX, R12
	ADCXQ BX, R13

	// | a2 * b5
	MULXQ 40(SI), AX, BX
	ADOXQ AX, R13
	ADOXQ R14, R14
	ADCXQ BX, R14

	// |

/* i3                                   */

	// | a3 @ DX
	MOVQ 24(DI), DX
	XORQ R15, R15

	// | a3 * b0
	MULXQ (SI), AX, BX
	ADOXQ AX, R9
	ADCXQ BX, R10

	// | a3 * b1
	MULXQ 8(SI), AX, BX
	ADOXQ AX, R10
	ADCXQ BX, R11

	// | a3 * b2
	MULXQ 16(SI), AX, BX
	ADOXQ AX, R11
	ADCXQ BX, R12

	// | a3 * b3
	MULXQ 24(SI), AX, BX
	ADOXQ AX, R12
	ADCXQ BX, R13

	// | a3 * b4
	MULXQ 32(SI), AX, BX
	ADOXQ AX, R13
	ADCXQ BX, R14

	// | a3 * b5
	MULXQ 40(SI), AX, BX
	ADOXQ AX, R14
	ADOXQ R15, R15
	ADCXQ BX, R15

	// |

/* i4                                   */

	// | a4 @ DX
	MOVQ 32(DI), DX
	XORQ CX, CX

	// | a4 * b0
	MULXQ (SI), AX, BX
	ADOXQ AX, R10
	ADCXQ BX, R11

	// | a4 * b1
	MULXQ 8(SI), AX, BX
	ADOXQ AX, R11
	ADCXQ BX, R12

	// | a4 * b2
	MULXQ 16(SI), AX, BX
	ADOXQ AX, R12
	ADCXQ BX, R13

	// | a4 * b3
	MULXQ 24(SI), AX, BX
	ADOXQ AX, R13
	ADCXQ BX, R14

	// | a4 * b4
	MULXQ 32(SI), AX, BX
	ADOXQ AX, R14
	ADCXQ BX, R15

	// | a4 * b5
	MULXQ 40(SI), AX, BX
	ADOXQ AX, R15
	ADOXQ CX, CX
	ADCXQ BX, CX

	// |

/* i5                                   */

	// | a5 @ DX
	MOVQ 40(DI), DX
	XORQ DI, DI

	// | a5 * b0
	MULXQ (SI), AX, BX
	ADOXQ AX, R11
	ADCXQ BX, R12

	// | a5 * b1
	MULXQ 8(SI), AX, BX
	ADOXQ AX, R12
	ADCXQ BX, R13

	// | a5 * b2
	MULXQ 16(SI), AX, BX
	ADOXQ AX, R13
	ADCXQ BX, R14

	// | a5 * b3
	MULXQ 24(SI), AX, BX
	ADOXQ AX, R14
	ADCXQ BX, R15

	// | a5 * b4
	MULXQ 32(SI), AX, BX
	ADOXQ AX, R15
	ADCXQ BX, CX

	// | a5 * b5
	MULXQ 40(SI), AX, BX
	ADOXQ AX, CX
	ADOXQ BX, DI
	ADCQ  $0x00, DI

	// |

/* 			                                     */

	// |
	// | W
	// | 0   (SP)      | 1   8(SP)     | 2   R8        | 3   R9        | 4   R10       | 5   R11
	// | 6   R12       | 7   R13       | 8   R14       | 9   R15       | 10  CX        | 11  DI


	MOVQ (SP), BX
	MOVQ 8(SP), SI
	MOVQ DI, (SP)

	// |
	// | W ready to mont
	// | 0   BX        | 1   SI        | 2   R8        | 3   R9        | 4   R10       | 5   R11
	// | 6   R12       | 7   R13       | 8   R14       | 9   R15       | 10  CX        | 11  (SP)


	// |

/* montgomery reduction                    */

	// | clear flags
	XORQ AX, AX

	// |

/* i0                                   */

	// |
	// | W
	// | 0   BX        | 1   SI        | 2   R8        | 3   R9        | 4   R10       | 5   R11
	// | 6   R12       | 7   R13       | 8   R14       | 9   R15       | 10  CX        | 11  (SP)


	// | | u0 = w0 * inp
	MOVQ  BX, DX
	MULXQ ·inp+0(SB), DX, DI

	// |

/*                                         */

	// | j0

	// | w0 @ BX
	MULXQ ·modulus+0(SB), AX, DI
	ADOXQ AX, BX
	ADCXQ DI, SI

	// | j1

	// | w1 @ SI
	MULXQ ·modulus+8(SB), AX, DI
	ADOXQ AX, SI
	ADCXQ DI, R8

	// | j2

	// | w2 @ R8
	MULXQ ·modulus+16(SB), AX, DI
	ADOXQ AX, R8
	ADCXQ DI, R9

	// | j3

	// | w3 @ R9
	MULXQ ·modulus+24(SB), AX, DI
	ADOXQ AX, R9
	ADCXQ DI, R10

	// | j4

	// | w4 @ R10
	MULXQ ·modulus+32(SB), AX, DI
	ADOXQ AX, R10
	ADCXQ DI, R11

	// | j5

	// | w5 @ R11
	MULXQ ·modulus+40(SB), AX, DI
	ADOXQ AX, R11
	ADCXQ DI, R12
	ADOXQ BX, R12
	ADCXQ BX, BX
	MOVQ  $0x00, AX
	ADOXQ AX, BX

	// | clear flags
	XORQ AX, AX

	// |

/* i1                                   */

	// |
	// | W
	// | 0   -         | 1   SI        | 2   R8        | 3   R9        | 4   R10       | 5   R11
	// | 6   R12       | 7   R13       | 8   R14       | 9   R15       | 10  CX        | 11  (SP)


	// | | u1 = w1 * inp
	MOVQ  SI, DX
	MULXQ ·inp+0(SB), DX, DI

	// |

/*                                         */

	// | j0

	// | w1 @ SI
	MULXQ ·modulus+0(SB), AX, DI
	ADOXQ AX, SI
	ADCXQ DI, R8

	// | j1

	// | w2 @ R8
	MULXQ ·modulus+8(SB), AX, DI
	ADOXQ AX, R8
	ADCXQ DI, R9

	// | j2

	// | w3 @ R9
	MULXQ ·modulus+16(SB), AX, DI
	ADOXQ AX, R9
	ADCXQ DI, R10

	// | j3

	// | w4 @ R10
	MULXQ ·modulus+24(SB), AX, DI
	ADOXQ AX, R10
	ADCXQ DI, R11

	// | j4

	// | w5 @ R11
	MULXQ ·modulus+32(SB), AX, DI
	ADOXQ AX, R11
	ADCXQ DI, R12

	// | j5

	// | w6 @ R12
	MULXQ ·modulus+40(SB), AX, DI
	ADOXQ AX, R12
	ADCXQ DI, R13
	ADOXQ BX, R13
	ADCXQ SI, SI
	MOVQ  $0x00, AX
	ADOXQ AX, SI

	// | clear flags
	XORQ AX, AX

	// |

/* i2                                   */

	// |
	// | W
	// | 0   -         | 1   -         | 2   R8        | 3   R9        | 4   R10       | 5   R11
	// | 6   R12       | 7   R13       | 8   R14       | 9   R15       | 10  CX        | 11  (SP)


	// | | u2 = w2 * inp
	MOVQ  R8, DX
	MULXQ ·inp+0(SB), DX, DI

	// |

/*                                         */

	// | j0

	// | w2 @ R8
	MULXQ ·modulus+0(SB), AX, DI
	ADOXQ AX, R8
	ADCXQ DI, R9

	// | j1

	// | w3 @ R9
	MULXQ ·modulus+8(SB), AX, DI
	ADOXQ AX, R9
	ADCXQ DI, R10

	// | j2

	// | w4 @ R10
	MULXQ ·modulus+16(SB), AX, DI
	ADOXQ AX, R10
	ADCXQ DI, R11

	// | j3

	// | w5 @ R11
	MULXQ ·modulus+24(SB), AX, DI
	ADOXQ AX, R11
	ADCXQ DI, R12

	// | j4

	// | w6 @ R12
	MULXQ ·modulus+32(SB), AX, DI
	ADOXQ AX, R12
	ADCXQ DI, R13

	// | j5

	// | w7 @ R13
	MULXQ ·modulus+40(SB), AX, DI
	ADOXQ AX, R13
	ADCXQ DI, R14
	ADOXQ SI, R14
	ADCXQ R8, R8
	MOVQ  $0x00, AX
	ADOXQ AX, R8

	// | clear flags
	XORQ AX, AX

	// |

/* i3                                   */

	// |
	// | W
	// | 0   -         | 1   -         | 2   -         | 3   R9        | 4   R10       | 5   R11
	// | 6   R12       | 7   R13       | 8   R14       | 9   R15       | 10  CX        | 11  (SP)


	// | | u3 = w3 * inp
	MOVQ  R9, DX
	MULXQ ·inp+0(SB), DX, DI

	// |

/*                                         */

	// | j0

	// | w3 @ R9
	MULXQ ·modulus+0(SB), AX, DI
	ADOXQ AX, R9
	ADCXQ DI, R10

	// | j1

	// | w4 @ R10
	MULXQ ·modulus+8(SB), AX, DI
	ADOXQ AX, R10
	ADCXQ DI, R11

	// | j2

	// | w5 @ R11
	MULXQ ·modulus+16(SB), AX, DI
	ADOXQ AX, R11
	ADCXQ DI, R12

	// | j3

	// | w6 @ R12
	MULXQ ·modulus+24(SB), AX, DI
	ADOXQ AX, R12
	ADCXQ DI, R13

	// | j4

	// | w7 @ R13
	MULXQ ·modulus+32(SB), AX, DI
	ADOXQ AX, R13
	ADCXQ DI, R14

	// | j5

	// | w8 @ R14
	MULXQ ·modulus+40(SB), AX, DI
	ADOXQ AX, R14
	ADCXQ DI, R15
	ADOXQ R8, R15
	ADCXQ R9, R9
	MOVQ  $0x00, AX
	ADOXQ AX, R9

	// | clear flags
	XORQ AX, AX

	// |

/* i4                                   */

	// |
	// | W
	// | 0   -         | 1   -         | 2   -         | 3   -         | 4   R10       | 5   R11
	// | 6   R12       | 7   R13       | 8   R14       | 9   R15       | 10  CX        | 11  (SP)


	// | | u4 = w4 * inp
	MOVQ  R10, DX
	MULXQ ·inp+0(SB), DX, DI

	// |

/*                                         */

	// | j0

	// | w4 @ R10
	MULXQ ·modulus+0(SB), AX, DI
	ADOXQ AX, R10
	ADCXQ DI, R11

	// | j1

	// | w5 @ R11
	MULXQ ·modulus+8(SB), AX, DI
	ADOXQ AX, R11
	ADCXQ DI, R12

	// | j2

	// | w6 @ R12
	MULXQ ·modulus+16(SB), AX, DI
	ADOXQ AX, R12
	ADCXQ DI, R13

	// | j3

	// | w7 @ R13
	MULXQ ·modulus+24(SB), AX, DI
	ADOXQ AX, R13
	ADCXQ DI, R14

	// | j4

	// | w8 @ R14
	MULXQ ·modulus+32(SB), AX, DI
	ADOXQ AX, R14
	ADCXQ DI, R15

	// | j5

	// | w9 @ R15
	MULXQ ·modulus+40(SB), AX, DI
	ADOXQ AX, R15
	ADCXQ DI, CX
	ADOXQ R9, CX
	ADCXQ R10, R10
	MOVQ  $0x00, AX
	ADOXQ AX, R10

	// | clear flags
	XORQ AX, AX

	// |

/* i5                                   */

	// |
	// | W
	// | 0   -         | 1   -         | 2   -         | 3   -         | 4   -         | 5   R11
	// | 6   R12       | 7   R13       | 8   R14       | 9   R15       | 10  CX        | 11  (SP)


	// | | u5 = w5 * inp
	MOVQ  R11, DX
	MULXQ ·inp+0(SB), DX, DI

	// |

/*                                         */

	// | j0

	// | w5 @ R11
	MULXQ ·modulus+0(SB), AX, DI
	ADOXQ AX, R11
	ADCXQ DI, R12

	// | j1

	// | w6 @ R12
	MULXQ ·modulus+8(SB), AX, DI
	ADOXQ AX, R12
	ADCXQ DI, R13

	// | j2

	// | w7 @ R13
	MULXQ ·modulus+16(SB), AX, DI
	ADOXQ AX, R13
	ADCXQ DI, R14

	// | j3

	// | w8 @ R14
	MULXQ ·modulus+24(SB), AX, DI
	ADOXQ AX, R14
	ADCXQ DI, R15

	// | j4

	// | w9 @ R15
	MULXQ ·modulus+32(SB), AX, DI
	ADOXQ AX, R15
	ADCXQ DI, CX

	// | j5

	// | w10 @ CX
	MULXQ ·modulus+40(SB), AX, DI
	ADOXQ AX, CX

	// | w11 @ (SP)
	// | move to an idle register
	MOVQ  (SP), BX
	ADCXQ DI, BX
	ADOXQ R10, BX

	// |
	// | W montgomery reduction ends
	// | 0   -         | 1   -         | 2   -         | 3   -         | 4   -         | 5   -
	// | 6   R12       | 7   R13       | 8   R14       | 9   R15       | 10  CX        | 11  BX


	// |

/* modular reduction                       */

	MOVQ R12, AX
	SUBQ ·modulus+0(SB), AX
	MOVQ R13, DI
	SBBQ ·modulus+8(SB), DI
	MOVQ R14, SI
	SBBQ ·modulus+16(SB), SI
	MOVQ R15, R8
	SBBQ ·modulus+24(SB), R8
	MOVQ CX, R9
	SBBQ ·modulus+32(SB), R9
	MOVQ BX, R10
	SBBQ ·modulus+40(SB), R10

	// |

/* out                                     */

	MOVQ    c+0(FP), R11
	CMOVQCC AX, R12
	MOVQ    R12, (R11)
	CMOVQCC DI, R13
	MOVQ    R13, 8(R11)
	CMOVQCC SI, R14
	MOVQ    R14, 16(R11)
	CMOVQCC R8, R15
	MOVQ    R15, 24(R11)
	CMOVQCC R9, CX
	MOVQ    CX, 32(R11)
	CMOVQCC R10, BX
	MOVQ    BX, 40(R11)
	RET

	// |

/* end 																			*/
