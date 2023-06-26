/*
MIT License

Copyright (c) 2021 Prysmatic Labs

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

This code is based on Intel's implementation found in
	https://github.com/intel/intel-ipsec-mb
Copied parts are
	Copyright (c) 2012-2021, Intel Corporation
*/

#include "textflag.h"

#define OUTPUT_PTR	R0
#define DATA_PTR	R1
#define NUM_BLKS	R2
#define last	R2

#define digest		R19
#define k256		R20
#define padding		R21

#define VR0	V0
#define VR1	V1
#define VR2	V2
#define VR3	V3
#define VTMP0	V4
#define VTMP1	V5
#define VTMP2	V6
#define VTMP3	V7
#define VTMP4	V17
#define VTMP5	V18
#define VTMP6	V19
#define KV0	V20
#define KV1	V21
#define KV2	V22
#define KV3	V23
#define KQ0	F20
#define KQ1	F21
#define KQ2	F22
#define KQ3	F23
#define VZ	V16

#define A_  R3
#define B_  R4
#define C_  R5
#define D_  R6
#define E_  R7
#define F_  R9
#define G_  R10
#define H_  R11
#define T1  R12
#define T2  R13
#define T3  R14
#define T4  R15
#define T5  R22

#define round1_sched(A, B, C, D, E, F, G, H, VV0, VV1, VV2, VV3) \
	VEXT	$4, VV3.B16, VV2.B16, VTMP0.B16; \
	RORW	$6, E, T1; \
	MOVWU	(RSP), T3; \
	RORW	$2, A, T2; \
	RORW	$13, A, T4; \
	VEXT	$4, VV1.B16, VV0.B16, VTMP1.B16; \
	EORW	T4, T2, T2; \
	ADDW	T3, H, H; \
	RORW	$11, E, T3; \
	VADD	VV0.S4, VTMP0.S4, VTMP0.S4; \
	EORW	T3, T1, T1; \
	RORW	$25, E, T3; \
	RORW	$22, A, T4; \
	VUSHR	$7, VTMP1.S4, VTMP2.S4; \
	EORW	T3, T1, T1; \
	EORW	T4, T2, T2; \
	EORW	G, F, T3; \
	VSHL	$(32-7), VTMP1.S4, VTMP3.S4; \
	EORW	C, A, T4; \
	ANDW	E, T3, T3; \
	ANDW	B, T4, T4; \
	EORW	G, T3, T3; \
	VUSHR	$18, VTMP1.S4, VTMP4.S4; \
	ADDW	T3, T1, T1; \
	ANDW	C, A, T3; \
	ADDW	T1, H, H; \
	VORR	VTMP2.B16, VTMP3.B16, VTMP3.B16; \
	EORW	T3, T4, T4; \
	ADDW	H, D, D; \
	ADDW	T4, T2, T2; \
	VUSHR	$3, VTMP1.S4, VTMP2.S4; \
	ADDW	T2, H, H

#define round2_sched(A, B, C, D, E, F, G, H, VV3) \
	MOVWU	4(RSP), T3; \
	RORW	$6, E, T1; \
	VSHL	$(32-18), VTMP1.S4, VTMP1.S4; \
	RORW	$2, A, T2; \
	RORW	$13, A, T4; \
	ADDW	T3, H, H; \
	VEOR	VTMP2.B16, VTMP3.B16, VTMP3.B16; \
	RORW	$11, E, T3; \
	EORW	T4, T2, T2; \
	EORW	T3, T1, T1; \
	VEOR	VTMP1.B16, VTMP4.B16, VTMP1.B16; \
	RORW	$25, E, T3; \
	RORW	$22, A, T4; \
	EORW	T3, T1, T1; \
	VZIP2	VV3.S4, VV3.S4, VTMP5.S4; \
	EORW	T4, T2, T2; \
	EORW	G, F, T3; \
	EORW	C, A, T4; \
	VEOR	VTMP1.B16, VTMP3.B16, VTMP1.B16; \
	ANDW	E, T3, T3; \
	ANDW	B, T4, T4; \
	EORW	G, T3, T3; \
	VUSHR	$10, VTMP5.S4, VTMP6.S4; \
	ADDW	T3, T1, T1; \
	ANDW	C, A, T3; \
	ADDW	T1, H, H; \
	VUSHR	$19, VTMP5.D2, VTMP3.D2; \
	EORW	T3, T4, T4; \
	ADDW	H, D, D; \
	ADDW	T4, T2, T2; \
	VUSHR	$17, VTMP5.D2, VTMP2.D2; \
	ADDW	T2, H, H

#define round3_sched(A, B, C, D, E, F, G, H) \
	MOVWU	8(RSP), T3; \
	RORW	$6, E, T1; \
	VEOR	VTMP6.B16, VTMP3.B16, VTMP3.B16; \
	RORW	$2, A, T2; \
	RORW	$13, A, T4; \
	ADDW	T3, H, H; \
	VADD	VTMP1.S4, VTMP0.S4, VTMP0.S4; \
	RORW	$11, E, T3; \
	EORW	T4, T2, T2; \
	EORW	T3, T1, T1; \
	VEOR	VTMP2.B16, VTMP3.B16, VTMP1.B16; \
	RORW	$25, E, T3; \
	RORW	$22, A, T4; \
	EORW	T3, T1, T1; \
	WORD	$0xea128a5; \
	EORW	T4, T2, T2; \
	EORW	G, F, T3; \
	EORW	C, A, T4; \
	VADD	VTMP1.S4, VTMP0.S4, VTMP0.S4; \
	ANDW	E, T3, T3; \
	ANDW	B, T4, T4; \
	EORW	G, T3, T3; \
	VZIP1	VTMP0.S4, VTMP0.S4, VTMP2.S4; \
	ADDW	T3, T1, T1; \
	ANDW	C, A, T3; \
	ADDW	T1, H, H; \
	EORW	T3, T4, T4; \
	ADDW	H, D, D; \
	ADDW	T4, T2, T2; \
	VUSHR	$10, VTMP2.S4, VTMP1.S4; \
	ADDW	T2, H, H

#define round4_sched(A, B, C, D, E, F, G, H, VV0) \
	MOVWU	12(RSP), T3; \
	RORW	$6, E, T1; \
	RORW	$2, A, T2; \
	VUSHR	$19, VTMP2.D2, VTMP3.D2; \
	RORW	$13, A, T4; \
	ADDW	T3, H, H; \
	RORW	$11, E, T3; \
	EORW	T4, T2, T2; \
	VUSHR	$17, VTMP2.D2, VTMP2.D2; \
	EORW	T3, T1, T1; \
	RORW	$25, E, T3; \
	RORW	$22, A, T4; \
	EORW	T3, T1, T1; \
	VEOR	VTMP3.B16, VTMP1.B16, VTMP1.B16; \
	EORW	T4, T2, T2; \
	EORW	G, F, T3; \
	EORW	C, A, T4; \
	VEOR	VTMP2.B16, VTMP1.B16, VTMP1.B16; \
	ANDW	E, T3, T3; \
	ANDW	B, T4, T4; \
	EORW	G, T3, T3; \
	VUZP1	VTMP1.S4, VZ.S4, VTMP1.S4; \
	ADDW	T3, T1, T1; \
	ANDW	C, A, T3; \
	ADDW	T1, H, H; \
	EORW	T3, T4, T4; \
	ADDW	H, D, D; \
	ADDW	T4, T2, T2; \
	VADD	VTMP0.S4, VTMP1.S4, VV0.S4; \
	ADDW	T2, H, H

#define four_rounds_sched(A, B, C, D, E, F, G, H, VV0, VV1, VV2, VV3) \
		    round1_sched(A, B, C, D, E, F, G, H, VV0, VV1, VV2, VV3); \
		    round2_sched(H, A, B, C, D, E, F, G, VV3); \
		    round3_sched(G, H, A, B, C, D, E, F); \
		    round4_sched(F, G, H, A, B, C, D, E, VV0)

#define one_round(A, B, C, D, E, F, G, H, ptr, offset) \
	MOVWU	offset(ptr), T3; \
	RORW	$6, E, T1; \
	RORW	$2, A, T2; \
	RORW	$13, A, T4; \
	ADDW	T3, H, H; \
	RORW	$11, E, T3; \
	EORW	T4, T2, T2; \
	EORW	T3, T1, T1; \
	RORW	$25, E, T3; \
	RORW	$22, A, T4; \
	EORW	T3, T1, T1; \
	EORW	T4, T2, T2; \
	EORW	G, F, T3; \
	EORW	C, A, T4; \
	ANDW	E, T3, T3; \
	ANDW	B, T4, T4; \
	EORW	G, T3, T3; \
	ADDW	T3, T1, T1; \
	ANDW	C, A, T3; \
	ADDW	T1, H, H; \
	EORW	T3, T4, T4; \
	ADDW	H, D, D; \
	ADDW	T4, T2, T2; \
	ADDW	T2, H, H

#define four_rounds(A, B, C, D, E, F, G, H, ptr, offset) \
		    one_round(A, B, C, D, E, F, G, H, ptr, offset); \
		    one_round(H, A, B, C, D, E, F, G, ptr, offset + 4); \
		    one_round(G, H, A, B, C, D, E, F, ptr, offset + 8); \
		    one_round(F, G, H, A, B, C, D, E, ptr, offset + 12)

// Definitions for ASIMD version
#define digest2              R6
#define post64               R7
#define postminus176         R9
#define post32               R10
#define postminus80          R11
#define M1		     V16
#define M2		     V17
#define M3		     V18
#define M4		     V19
#define MQ1                  F16
#define MQ2                  F17
#define MQ3                  F18
#define MQ4                  F19
#define NVR1		     V24
#define NVR2		     V25
#define NVR3		     V26
#define NVR4		     V27
#define QR2		     F25
#define QR4		     F27
#define TV1		     V28
#define TV2		     V29
#define TV3		     V30
#define TV4		     V31
#define TV5		     V20
#define TV6                  V21
#define TV7		     V22
#define TV8		     V23
#define TQ4		     F31
#define TQ5		     F20
#define TQ6                  F21
#define TQ7		     F22

#define round_4(A, B, C, D, E, F, G, H, MV, MQ, bicword, offset) \
	VUSHR	$6, E.S4, TV1.S4; \
	VSHL	$(32-6), E.S4, TV2.S4; \
	VUSHR	$11, E.S4, NVR2.S4; \
	VSHL	$(32-11), E.S4, NVR1.S4; \
	VAND	F.B16, E.B16, TV3.B16; \
	WORD	bicword; \
	VORR	TV2.B16, TV1.B16, TV1.B16; \
	VUSHR	$25, E.S4, TV2.S4; \
	FMOVQ	offset(k256), QR4; \
	VSHL	$(32-25), E.S4, NVR3.S4; \
	VORR	NVR1.B16, NVR2.B16, NVR1.B16; \
	VEOR	TV4.B16, TV3.B16, TV3.B16; \
	VORR	NVR3.B16, TV2.B16, TV2.B16; \
	VEOR	C.B16, A.B16, NVR3.B16; \
	VEOR	NVR1.B16, TV1.B16, TV1.B16; \
	VADD	NVR4.S4, MV.S4, TV4.S4; \
	VADD	TV3.S4, H.S4, H.S4; \
	VUSHR	$2, A.S4, TV3.S4; \
	VAND	B.B16, NVR3.B16, NVR3.B16; \
	VSHL	$(32-2), A.S4, NVR4.S4; \
	VEOR	TV2.B16, TV1.B16, TV1.B16; \
	VUSHR	$13, A.S4, TV2.S4; \
	VSHL	$(32-13), A.S4, NVR1.S4; \
	VADD	TV4.S4, H.S4, H.S4; \
	VORR	NVR4.B16, TV3.B16, TV3.B16; \
	VAND	C.B16, A.B16, NVR4.B16; \
	VUSHR	$22, A.S4, TV4.S4; \
	VSHL	$(32 - 22), A.S4, NVR2.S4 ; \
	VORR	NVR1.B16, TV2.B16, TV2.B16; \
	VADD	TV1.S4, H.S4, H.S4; \
	VEOR	NVR4.B16, NVR3.B16, NVR3.B16; \
	VORR	NVR2.B16, TV4.B16, TV4.B16; \
	VEOR	TV3.B16, TV2.B16, TV2.B16; \
	VADD	H.S4, D.S4, D.S4; \
	VADD	NVR3.S4, H.S4, H.S4; \
	VEOR	TV4.B16, TV2.B16, TV2.B16; \
	FMOVQ	MQ, offset(RSP); \
	VADD	TV2.S4, H.S4, H.S4

#define eight_4_roundsA(A, B, C, D, E, F, G, H, MV1, MV2, MV3, MV4, MQ1, MQ2, MQ3, MQ4, offset) \
                    round_4(A, B, C, D, E, F, G, H, MV1, MQ1, $0x4e641cdf, offset); \
                    round_4(H, A, B, C, D, E, F, G, MV2, MQ2, $0x4e631cbf, offset + 16); \
                    round_4(G, H, A, B, C, D, E, F, MV3, MQ3, $0x4e621c9f, offset + 32); \ 
                    round_4(F, G, H, A, B, C, D, E, MV4, MQ4, $0x4e611c7f, offset + 48)

#define eight_4_roundsB(A, B, C, D, E, F, G, H, MV1, MV2, MV3, MV4, MQ1, MQ2, MQ3, MQ4, offset) \
                    round_4(A, B, C, D, E, F, G, H, MV1, MQ1, $0x4e601c5f, offset); \
                    round_4(H, A, B, C, D, E, F, G, MV2, MQ2, $0x4e671c3f, offset + 16); \
                    round_4(G, H, A, B, C, D, E, F, MV3, MQ3, $0x4e661c1f, offset + 32); \ 
                    round_4(F, G, H, A, B, C, D, E, MV4, MQ4, $0x4e651cff, offset + 48)

#define round_4_and_sched(A, B, C, D, E, F, G, H, bicword, offset) \
	FLDPQ	(offset-256)(RSP), (TQ6, TQ5); \
	VUSHR	$6, E.S4, TV1.S4; \
	VSHL	$(32-6), E.S4, TV2.S4; \
	VUSHR	$11, E.S4, NVR2.S4; \
	VSHL	$(32-11), E.S4, NVR1.S4; \
	VAND	F.B16, E.B16, TV3.B16; \
	WORD	bicword; \
	VUSHR	$7, TV5.S4, M1.S4; \
        FMOVQ   (offset-32)(RSP), TQ7; \
	VSHL	$(32-7), TV5.S4, M2.S4; \
	VORR	TV2.B16, TV1.B16, TV1.B16; \
	VUSHR	$25, E.S4, TV2.S4; \
	VSHL	$(32-25), E.S4, NVR3.S4; \
	VORR	NVR1.B16, NVR2.B16, NVR1.B16; \
	VEOR	TV4.B16, TV3.B16, TV3.B16; \
	FMOVQ	offset(k256), QR4; \
	VORR	M2.B16, M1.B16, M1.B16; \
	VUSHR	$17, TV7.S4, M3.S4; \
	VSHL	$(32-17), TV7.S4, M4.S4; \
	VUSHR	$18, TV5.S4, M2.S4; \
	VSHL	$(32-18), TV5.S4, TV8.S4; \
	VORR	NVR3.B16, TV2.B16, TV2.B16; \
	VEOR	C.B16, A.B16, NVR3.B16; \
	VORR	M4.B16, M3.B16, M3.B16; \
        FMOVQ	(offset-112)(RSP), TQ4; \
	VUSHR	$19, TV7.S4, M4.S4; \
	VSHL	$(32-19), TV7.S4, NVR2.S4; \
	VORR	TV8.B16, M2.B16, M2.B16; \
	VUSHR	$3, TV5.S4, TV8.S4; \
	VORR	NVR2.B16, M4.B16, M4.B16; \
	VEOR	NVR1.B16, TV1.B16, TV1.B16; \
	VEOR	M2.B16, M1.B16, M1.B16; \
	VUSHR	$10, TV7.S4, M2.S4; \
	VEOR	M4.B16, M3.B16, M3.B16; \
	VADD	TV3.S4, H.S4, H.S4; \
	VEOR	TV8.B16, M1.B16, M1.B16; \
	VADD	TV4.S4, TV6.S4, TV6.S4; \
	VEOR	M2.B16, M3.B16, M3.B16; \
	VUSHR	$2, A.S4, TV3.S4; \
	VAND	B.B16, NVR3.B16, NVR3.B16; \
	VADD	TV6.S4, M1.S4, M1.S4; \
	VSHL	$(32-2), A.S4, TV6.S4; \
	VEOR	TV2.B16, TV1.B16, TV1.B16; \
	VUSHR	$13, A.S4, TV2.S4; \
	VADD	M3.S4, M1.S4, M1.S4; \
	VADD	TV1.S4, H.S4, H.S4; \
	VSHL	$(32-13), A.S4, NVR1.S4; \
	VORR	TV6.B16, TV3.B16, TV3.B16; \
	VADD	NVR4.S4, M1.S4, TV5.S4; \
	FMOVQ	MQ1, offset(RSP); \
	VAND	C.B16, A.B16, NVR4.B16; \
	VUSHR	$22, A.S4, TV4.S4; \
        VSHL    $(32-22), A.S4, NVR2.S4; \
	VADD	TV5.S4, H.S4, H.S4; \
	VORR	NVR1.B16, TV2.B16, TV2.B16; \
	VEOR	NVR4.B16, NVR3.B16, NVR3.B16; \
	VORR	NVR2.B16, TV4.B16, TV4.B16; \
	VEOR	TV3.B16, TV2.B16, TV2.B16; \
	VADD	H.S4, D.S4, D.S4; \
	VADD	NVR3.S4, H.S4, H.S4; \
	VEOR	TV4.B16, TV2.B16, TV2.B16; \
	VADD	TV2.S4, H.S4, H.S4

#define eight_4_rounds_and_sched(A, B, C, D, E, F, G, H, offset) \
                    round_4_and_sched(A, B, C, D, E, F, G, H, $0x4e641cdf, offset + 0*16); \
                    round_4_and_sched(H, A, B, C, D, E, F, G, $0x4e631cbf, offset + 1*16); \
                    round_4_and_sched(G, H, A, B, C, D, E, F, $0x4e621c9f, offset + 2*16); \
                    round_4_and_sched(F, G, H, A, B, C, D, E, $0x4e611c7f, offset + 3*16); \
                    round_4_and_sched(E, F, G, H, A, B, C, D, $0x4e601c5f, offset + 4*16); \
                    round_4_and_sched(D, E, F, G, H, A, B, C, $0x4e671c3f, offset + 5*16); \
                    round_4_and_sched(C, D, E, F, G, H, A, B, $0x4e661c1f, offset + 6*16); \
                    round_4_and_sched(B, C, D, E, F, G, H, A, $0x4e651cff, offset + 7*16)

#define round_4_padding(A, B, C, D, E, F, G, H, bicword, offset) \
	VUSHR	$6, E.S4, TV1.S4; \
	VSHL	$(32-6), E.S4, TV2.S4; \
	VUSHR	$11, E.S4, NVR2.S4; \
	VSHL	$(32-11), E.S4, NVR1.S4; \
	VAND	F.B16, E.B16, TV3.B16; \
	WORD	bicword; \
	VORR	TV2.B16, TV1.B16, TV1.B16; \
	VUSHR	$25, E.S4, TV2.S4; \
	VSHL	$(32-25), E.S4, NVR3.S4; \
	VORR	NVR1.B16, NVR2.B16, NVR1.B16; \
	VEOR	TV4.B16, TV3.B16, TV3.B16; \
	VORR	NVR3.B16, TV2.B16, TV2.B16; \
	VEOR	C.B16, A.B16, NVR3.B16; \
	VEOR	NVR1.B16, TV1.B16, TV1.B16; \
	VADD	TV3.S4, H.S4, H.S4; \
	VUSHR	$2, A.S4, TV3.S4; \
	FMOVQ	offset(padding), QR2; \
	VAND	B.B16, NVR3.B16, NVR3.B16; \
	VSHL	$(32-2), A.S4, NVR4.S4; \
	VEOR	TV2.B16, TV1.B16, TV1.B16; \
	VUSHR	$13, A.S4, TV2.S4; \
	VSHL	$(32-13), A.S4, NVR1.S4; \
	VADD	NVR2.S4, H.S4, H.S4; \
	VORR	NVR4.B16, TV3.B16, TV3.B16; \
	VAND	C.B16, A.B16, NVR4.B16; \
	VUSHR	$22, A.S4, TV4.S4; \
        VSHL	$(32-22), A.S4, NVR2.S4; \
	VORR	NVR1.B16, TV2.B16, TV2.B16; \
	VADD	TV1.S4, H.S4, H.S4; \
	VEOR	NVR4.B16, NVR3.B16, NVR3.B16; \
	VORR	NVR2.B16, TV4.B16, TV4.B16; \
	VEOR	TV3.B16, TV2.B16, TV2.B16; \
	VADD	H.S4, D.S4, D.S4; \
	VADD	NVR3.S4, H.S4, H.S4; \
	VEOR	TV4.B16, TV2.B16, TV2.B16; \
	VADD	TV2.S4, H.S4, H.S4

#define eight_4_rounds_padding(A, B, C, D, E, F, G, H, offset) \
                    round_4_padding(A, B, C, D, E, F, G, H, $0x4e641cdf, offset + 0*16); \
                    round_4_padding(H, A, B, C, D, E, F, G, $0x4e631cbf, offset + 1*16); \
                    round_4_padding(G, H, A, B, C, D, E, F, $0x4e621c9f, offset + 2*16); \
                    round_4_padding(F, G, H, A, B, C, D, E, $0x4e611c7f, offset + 3*16); \
                    round_4_padding(E, F, G, H, A, B, C, D, $0x4e601c5f, offset + 4*16); \
                    round_4_padding(D, E, F, G, H, A, B, C, $0x4e671c3f, offset + 5*16); \
                    round_4_padding(C, D, E, F, G, H, A, B, $0x4e661c1f, offset + 6*16); \
                    round_4_padding(B, C, D, E, F, G, H, A, $0x4e651cff, offset + 7*16)

// Definitions for SHA-2
#define check_shani R19

#define HASHUPDATE(word) \
	SHA256H	word, V3, V2; \
	SHA256H2	word, V8, V3; \
	VMOV	V2.B16, V8.B16

TEXT ·_hash(SB), 0, $1024-36
	MOVD digests+0(FP), OUTPUT_PTR
	MOVD p_base+8(FP), DATA_PTR
	MOVWU count+32(FP), NUM_BLKS

	MOVBU ·hasShani(SB), check_shani
	CBNZ  check_shani, shani

arm_x4:
	CMPW	$4, NUM_BLKS
	BLO	arm_x1

	MOVD	$_PADDING_4<>(SB), padding
	MOVD	$_K256_4<>(SB), k256
	MOVD	$_DIGEST_4<>(SB), digest
	ADD	$64, digest, digest2
	MOVD	$64, post64
	MOVD	$32, post32
	MOVD    $-80, postminus80
	MOVD	$-176, postminus176

arm_x4_loop:
	CMPW	$4, NUM_BLKS
	BLO	arm_x1
	VLD1    (digest), [V0.S4, V1.S4, V2.S4, V3.S4]
	VLD1	(digest2), [V4.S4, V5.S4, V6.S4, V7.S4]

	// First 16 rounds
	WORD   $0xde7a030
	WORD   $0xde7b030
	WORD   $0x4de7a030
	WORD   $0x4de9b030
	VREV32 M1.B16, M1.B16
	VREV32 M2.B16, M2.B16
	VREV32 M3.B16, M3.B16
	VREV32 M4.B16, M4.B16
	eight_4_roundsA(V0, V1, V2, V3, V4, V5, V6, V7, M1, M2, M3, M4, MQ1, MQ2, MQ3, MQ4, 0x00)

	WORD   $0xde7a030
	WORD   $0xde7b030
	WORD   $0x4de7a030
	WORD   $0x4de9b030
	VREV32 M1.B16, M1.B16
	VREV32 M2.B16, M2.B16
	VREV32 M3.B16, M3.B16
	VREV32 M4.B16, M4.B16
	eight_4_roundsB(V4, V5, V6, V7, V0, V1, V2, V3, M1, M2, M3, M4, MQ1, MQ2, MQ3, MQ4, 0x40)

	WORD   $0xde7a030
	WORD   $0xde7b030
	WORD   $0x4de7a030
	WORD   $0x4de9b030
	VREV32 M1.B16, M1.B16
	VREV32 M2.B16, M2.B16
	VREV32 M3.B16, M3.B16
	VREV32 M4.B16, M4.B16
	eight_4_roundsA(V0, V1, V2, V3, V4, V5, V6, V7, M1, M2, M3, M4, MQ1, MQ2, MQ3, MQ4, 0x80)

	WORD   $0xde7a030
	WORD   $0xde7b030
	WORD   $0x4de7a030
	WORD   $0x4de9b030
	VREV32 M1.B16, M1.B16
	VREV32 M2.B16, M2.B16
	VREV32 M3.B16, M3.B16
	VREV32 M4.B16, M4.B16
	eight_4_roundsB(V4, V5, V6, V7, V0, V1, V2, V3, M1, M2, M3, M4, MQ1, MQ2, MQ3, MQ4, 0xc0)

	eight_4_rounds_and_sched(V0, V1, V2, V3, V4, V5, V6, V7, 0x100)
	eight_4_rounds_and_sched(V0, V1, V2, V3, V4, V5, V6, V7, 0x180)
	eight_4_rounds_and_sched(V0, V1, V2, V3, V4, V5, V6, V7, 0x200)
	eight_4_rounds_and_sched(V0, V1, V2, V3, V4, V5, V6, V7, 0x280)
	eight_4_rounds_and_sched(V0, V1, V2, V3, V4, V5, V6, V7, 0x300)
	eight_4_rounds_and_sched(V0, V1, V2, V3, V4, V5, V6, V7, 0x380)


	// add previous digest
	VLD1	(digest), [M1.S4, M2.S4, M3.S4, M4.S4]
	VLD1	(digest2), [TV5.S4, TV6.S4, TV7.S4, TV8.S4]
	VADD	M1.S4, V0.S4, V0.S4
	VADD	M2.S4, V1.S4, V1.S4
	VADD	M3.S4, V2.S4, V2.S4
	VADD	M4.S4, V3.S4, V3.S4
	VADD	TV5.S4, V4.S4, V4.S4
	VADD	TV6.S4, V5.S4, V5.S4
	VADD	TV7.S4, V6.S4, V6.S4
	VADD	TV8.S4, V7.S4, V7.S4

	// save state
	VMOV	V0.B16, M1.B16
	VMOV	V1.B16, M2.B16
	VMOV	V2.B16, M3.B16
	VMOV	V3.B16, M4.B16
	VMOV	V4.B16, TV5.B16
	VMOV	V5.B16, TV6.B16
	VMOV	V6.B16, TV7.B16
	VMOV	V7.B16, TV8.B16

	// rounds with padding
	eight_4_rounds_padding(V0, V1, V2, V3, V4, V5, V6, V7, 0x000)
	eight_4_rounds_padding(V0, V1, V2, V3, V4, V5, V6, V7, 0x080)
	eight_4_rounds_padding(V0, V1, V2, V3, V4, V5, V6, V7, 0x100)
	eight_4_rounds_padding(V0, V1, V2, V3, V4, V5, V6, V7, 0x180)
	eight_4_rounds_padding(V0, V1, V2, V3, V4, V5, V6, V7, 0x200)
	eight_4_rounds_padding(V0, V1, V2, V3, V4, V5, V6, V7, 0x280)
	eight_4_rounds_padding(V0, V1, V2, V3, V4, V5, V6, V7, 0x300)
	eight_4_rounds_padding(V0, V1, V2, V3, V4, V5, V6, V7, 0x380)

	// add previous digest
	VADD	M1.S4, V0.S4, V0.S4
	VADD	M2.S4, V1.S4, V1.S4
	VADD	M3.S4, V2.S4, V2.S4
	VADD	M4.S4, V3.S4, V3.S4
	VADD	TV5.S4, V4.S4, V4.S4
	VADD	TV6.S4, V5.S4, V5.S4
	VADD	TV7.S4, V6.S4, V6.S4
	VADD	TV8.S4, V7.S4, V7.S4

	// change endianness transpose and store
	VREV32               V0.B16, V0.B16
	VREV32               V1.B16, V1.B16
	VREV32               V2.B16, V2.B16
	VREV32               V3.B16, V3.B16
	VREV32               V4.B16, V4.B16
	VREV32               V5.B16, V5.B16
	VREV32               V6.B16, V6.B16
	VREV32               V7.B16, V7.B16
	
	WORD $0xdaaa000
	WORD $0xdaab000
	WORD $0x4daaa000
	WORD $0x4dabb000
	WORD $0xdaaa004
	WORD $0xdaab004
	WORD $0x4daaa004
	WORD $0x4dbfb004

	ADD	$192, DATA_PTR, DATA_PTR
	SUBW	$4, NUM_BLKS, NUM_BLKS
	JMP	arm_x4_loop

arm_x1:
	VMOV	ZR, VZ.S4	// Golang guarantees this is zero
	MOVD	$_DIGEST_1<>(SB), digest
	MOVD	$_PADDING_1<>(SB), padding
	ADD	NUM_BLKS<<5, OUTPUT_PTR, last

arm_x1_loop:
	CMP	OUTPUT_PTR, last
	BEQ	epilog

	// Load one block
	VLD1.P	64(DATA_PTR), [VR0.S4, VR1.S4, VR2.S4, VR3.S4]	
	MOVD	$_K256_1<>(SB), k256

	// change endiannes
	VREV32		VR0.B16, VR0.B16
	VREV32		VR1.B16, VR1.B16
	VREV32		VR2.B16, VR2.B16
	VREV32		VR3.B16, VR3.B16

	// load initial digest
	LDPW	(digest), (A_, B_)
	LDPW	8(digest), (C_, D_)
	LDPW	16(digest), (E_, F_)
	LDPW	24(digest), (G_, H_)

	// First 48 rounds
	VLD1.P	64(k256), [KV0.S4, KV1.S4, KV2.S4, KV3.S4]
	VADD	VR0.S4, KV0.S4, KV0.S4
	FMOVQ	KQ0, (RSP)
	four_rounds_sched(A_, B_, C_, D_, E_, F_, G_, H_, VR0, VR1, VR2, VR3)
	
	VADD	VR1.S4, KV1.S4, KV1.S4
	FMOVQ	KQ1, (RSP)
	four_rounds_sched(E_, F_, G_, H_, A_, B_, C_, D_, VR1, VR2, VR3, VR0)

	VADD	VR2.S4, KV2.S4, KV2.S4
	FMOVQ	KQ2, (RSP)
	four_rounds_sched(A_, B_, C_, D_, E_, F_, G_, H_, VR2, VR3, VR0, VR1)

	VADD	VR3.S4, KV3.S4, KV3.S4
	FMOVQ	KQ3, (RSP)
	four_rounds_sched(E_, F_, G_, H_, A_, B_, C_, D_, VR3, VR0, VR1, VR2)

	VLD1.P	64(k256), [KV0.S4, KV1.S4, KV2.S4, KV3.S4]
	VADD	VR0.S4, KV0.S4, KV0.S4
	FMOVQ	KQ0, (RSP)
	four_rounds_sched(A_, B_, C_, D_, E_, F_, G_, H_, VR0, VR1, VR2, VR3)
	
	VADD	VR1.S4, KV1.S4, KV1.S4
	FMOVQ	KQ1, (RSP)
	four_rounds_sched(E_, F_, G_, H_, A_, B_, C_, D_, VR1, VR2, VR3, VR0)

	VADD	VR2.S4, KV2.S4, KV2.S4
	FMOVQ	KQ2, (RSP)
	four_rounds_sched(A_, B_, C_, D_, E_, F_, G_, H_, VR2, VR3, VR0, VR1)

	VADD	VR3.S4, KV3.S4, KV3.S4
	FMOVQ	KQ3, (RSP)
	four_rounds_sched(E_, F_, G_, H_, A_, B_, C_, D_, VR3, VR0, VR1, VR2)

	VLD1.P	64(k256), [KV0.S4, KV1.S4, KV2.S4, KV3.S4]
	VADD	VR0.S4, KV0.S4, KV0.S4
	FMOVQ	KQ0, (RSP)
	four_rounds_sched(A_, B_, C_, D_, E_, F_, G_, H_, VR0, VR1, VR2, VR3)
	
	VADD	VR1.S4, KV1.S4, KV1.S4
	FMOVQ	KQ1, (RSP)
	four_rounds_sched(E_, F_, G_, H_, A_, B_, C_, D_, VR1, VR2, VR3, VR0)

	VADD	VR2.S4, KV2.S4, KV2.S4
	FMOVQ	KQ2, (RSP)
	four_rounds_sched(A_, B_, C_, D_, E_, F_, G_, H_, VR2, VR3, VR0, VR1)

	VADD	VR3.S4, KV3.S4, KV3.S4
	FMOVQ	KQ3, (RSP)
	four_rounds_sched(E_, F_, G_, H_, A_, B_, C_, D_, VR3, VR0, VR1, VR2)

	// last 16 rounds
	VLD1.P	64(k256), [KV0.S4, KV1.S4, KV2.S4, KV3.S4]
	VADD	VR0.S4, KV0.S4, KV0.S4
	FMOVQ	KQ0, (RSP)
	four_rounds(A_, B_, C_, D_, E_, F_, G_, H_, RSP, 0)
	
	VADD	VR1.S4, KV1.S4, KV1.S4
	FMOVQ	KQ1, (RSP)
	four_rounds(E_, F_, G_, H_, A_, B_, C_, D_, RSP, 0)

	VADD	VR2.S4, KV2.S4, KV2.S4
	FMOVQ	KQ2, (RSP)
	four_rounds(A_, B_, C_, D_, E_, F_, G_, H_, RSP, 0)

	VADD	VR3.S4, KV3.S4, KV3.S4
	FMOVQ	KQ3, (RSP)
	four_rounds(E_, F_, G_, H_, A_, B_, C_, D_, RSP, 0)

	// rounds with padding
	LDPW	(digest), (T1, T2)
	LDPW	8(digest), (T3, T4)

	ADDW	T1, A_, A_
	ADDW	T2, B_, B_
	ADDW	T3, C_, C_
	ADDW	T4, D_, D_
	LDPW	16(digest), (T1, T2)
	STPW	(A_, B_), (RSP)
	STPW	(C_, D_), 8(RSP)
	LDPW	24(digest), (T3, T4)
	ADDW	T1, E_, E_
	ADDW	T2, F_, F_
	ADDW	T3, G_, G_
	STPW	(E_, F_), 16(RSP)
	ADDW	T4, H_, H_
	STPW	(G_, H_), 24(RSP)

	four_rounds(A_, B_, C_, D_, E_, F_, G_, H_, padding, 0x00)
	four_rounds(E_, F_, G_, H_, A_, B_, C_, D_, padding, 0x10)
	four_rounds(A_, B_, C_, D_, E_, F_, G_, H_, padding, 0x20)
	four_rounds(E_, F_, G_, H_, A_, B_, C_, D_, padding, 0x30)
	four_rounds(A_, B_, C_, D_, E_, F_, G_, H_, padding, 0x40)
	four_rounds(E_, F_, G_, H_, A_, B_, C_, D_, padding, 0x50)
	four_rounds(A_, B_, C_, D_, E_, F_, G_, H_, padding, 0x60)
	four_rounds(E_, F_, G_, H_, A_, B_, C_, D_, padding, 0x70)
	four_rounds(A_, B_, C_, D_, E_, F_, G_, H_, padding, 0x80)
	four_rounds(E_, F_, G_, H_, A_, B_, C_, D_, padding, 0x90)
	four_rounds(A_, B_, C_, D_, E_, F_, G_, H_, padding, 0xa0)
	four_rounds(E_, F_, G_, H_, A_, B_, C_, D_, padding, 0xb0)
	four_rounds(A_, B_, C_, D_, E_, F_, G_, H_, padding, 0xc0)
	four_rounds(E_, F_, G_, H_, A_, B_, C_, D_, padding, 0xd0)
	four_rounds(A_, B_, C_, D_, E_, F_, G_, H_, padding, 0xe0)
	four_rounds(E_, F_, G_, H_, A_, B_, C_, D_, padding, 0xf0)

	LDPW	(RSP), (T1, T2)
	LDPW	8(RSP), (T3, T4)
	ADDW	T1, A_, A_
	ADDW	T2, B_, B_
	REV32	A_, A_
	REV32	B_, B_
	ADDW	T3, C_, C_	
	ADDW	T4, D_, D_
	STPW.P	(A_, B_), 8(OUTPUT_PTR)
	LDPW	16(RSP), (T1, T2)

	REV32	C_, C_
	REV32	D_, D_
	STPW.P	(C_, D_), 8(OUTPUT_PTR)
	LDPW	24(RSP), (T3, T4)
	ADDW	T1, E_, E_
	ADDW	T2, F_, F_
	REV32	E_, E_
	REV32	F_, F_
	ADDW	T3, G_, G_
	ADDW	T4, H_, H_
	REV32	G_, G_
	REV32	H_, H_
	STPW.P	(E_, F_), 8(OUTPUT_PTR)
	STPW.P	(G_, H_), 8(OUTPUT_PTR)

	JMP	arm_x1_loop

shani:
	MOVD	$_DIGEST_1<>(SB), digest
	MOVD	$_PADDING_1<>(SB), padding
	MOVD	$_K256_1<>(SB), k256
	ADD	NUM_BLKS<<5, OUTPUT_PTR, last

	// load incoming digest
	VLD1	(digest), [V0.S4, V1.S4]

shani_loop:
	CMP	OUTPUT_PTR, last
	BEQ	epilog


	// load all K constants
	VLD1.P	64(k256), [V16.S4, V17.S4, V18.S4, V19.S4]
	VLD1.P	64(k256), [V20.S4, V21.S4, V22.S4, V23.S4]
	VLD1.P	64(k256), [V24.S4, V25.S4, V26.S4, V27.S4]
	VLD1	(k256), [V28.S4, V29.S4, V30.S4, V31.S4]
	SUB	$192, k256, k256

	// load one block
	VLD1.P	64(DATA_PTR), [V4.S4, V5.S4, V6.S4, V7.S4]
	VMOV	V0.B16, V2.B16
	VMOV	V1.B16, V3.B16
	VMOV	V2.B16, V8.B16

	// reverse endianness 
	VREV32	V4.B16, V4.B16
	VREV32	V5.B16, V5.B16
	VREV32	V6.B16, V6.B16
	VREV32	V7.B16, V7.B16

	VADD	V16.S4, V4.S4, V9.S4
	SHA256SU0	V5.S4, V4.S4
	HASHUPDATE(V9.S4)

	VADD	V17.S4, V5.S4, V9.S4
	SHA256SU0	V6.S4, V5.S4
	SHA256SU1	V7.S4, V6.S4, V4.S4
	HASHUPDATE(V9.S4)

	VADD	V18.S4, V6.S4, V9.S4
	SHA256SU0	V7.S4, V6.S4
	SHA256SU1	V4.S4, V7.S4, V5.S4
	HASHUPDATE(V9.S4)

	VADD	V19.S4, V7.S4, V9.S4
	SHA256SU0	V4.S4, V7.S4
	SHA256SU1	V5.S4, V4.S4, V6.S4
	HASHUPDATE(V9.S4)

	VADD	V20.S4, V4.S4, V9.S4
	SHA256SU0	V5.S4, V4.S4
	SHA256SU1	V6.S4, V5.S4, V7.S4
	HASHUPDATE(V9.S4)

	VADD	V21.S4, V5.S4, V9.S4
	SHA256SU0	V6.S4, V5.S4
	SHA256SU1	V7.S4, V6.S4, V4.S4
	HASHUPDATE(V9.S4)

	VADD	V22.S4, V6.S4, V9.S4
	SHA256SU0	V7.S4, V6.S4
	SHA256SU1	V4.S4, V7.S4, V5.S4
	HASHUPDATE(V9.S4)

	VADD	V23.S4, V7.S4, V9.S4
	SHA256SU0	V4.S4, V7.S4
	SHA256SU1	V5.S4, V4.S4, V6.S4
	HASHUPDATE(V9.S4)

	VADD	V24.S4, V4.S4, V9.S4
	SHA256SU0	V5.S4, V4.S4
	SHA256SU1	V6.S4, V5.S4, V7.S4
	HASHUPDATE(V9.S4)

	VADD	V25.S4, V5.S4, V9.S4
	SHA256SU0	V6.S4, V5.S4
	SHA256SU1	V7.S4, V6.S4, V4.S4
	HASHUPDATE(V9.S4)

	VADD	V26.S4, V6.S4, V9.S4
	SHA256SU0	V7.S4, V6.S4
	SHA256SU1	V4.S4, V7.S4, V5.S4
	HASHUPDATE(V9.S4)

	VADD	V27.S4, V7.S4, V9.S4
	SHA256SU0	V4.S4, V7.S4
	SHA256SU1	V5.S4, V4.S4, V6.S4
	HASHUPDATE(V9.S4)

	VADD	V28.S4, V4.S4, V9.S4
	HASHUPDATE(V9.S4)
	SHA256SU1	V6.S4, V5.S4, V7.S4

	VADD	V29.S4, V5.S4, V9.S4
	HASHUPDATE(V9.S4)

	VADD	V30.S4, V6.S4, V9.S4
	HASHUPDATE(V9.S4)

	VADD	V31.S4, V7.S4, V9.S4
	HASHUPDATE(V9.S4)

	
	// Add initial digest
	VADD	V2.S4, V0.S4, V2.S4
	VADD	V3.S4, V1.S4, V3.S4

	// Back it up
	VMOV	V2.B16, V10.B16
	VMOV	V3.B16, V11.B16

	// Rounds with padding

	// load prescheduled constants
	VLD1.P	64(padding), [V16.S4, V17.S4, V18.S4, V19.S4]
	VLD1.P	64(padding), [V20.S4, V21.S4, V22.S4, V23.S4]
	VMOV	V2.B16, V8.B16
	VLD1.P	64(padding), [V24.S4, V25.S4, V26.S4, V27.S4]
	VLD1	(padding), [V28.S4, V29.S4, V30.S4, V31.S4]
	SUB	$192, padding, padding 

	HASHUPDATE(V16.S4)
	HASHUPDATE(V17.S4)
	HASHUPDATE(V18.S4)
	HASHUPDATE(V19.S4)
	HASHUPDATE(V20.S4)
	HASHUPDATE(V21.S4)
	HASHUPDATE(V22.S4)
	HASHUPDATE(V23.S4)
	HASHUPDATE(V24.S4)
	HASHUPDATE(V25.S4)
	HASHUPDATE(V26.S4)
	HASHUPDATE(V27.S4)
	HASHUPDATE(V28.S4)
	HASHUPDATE(V29.S4)
	HASHUPDATE(V30.S4)
	HASHUPDATE(V31.S4)

	// add backed up digest
	VADD	V2.S4, V10.S4, V2.S4
	VADD	V3.S4, V11.S4, V3.S4


	VREV32	V2.B16, V2.B16
	VREV32  V3.B16, V3.B16

	VST1.P	[V2.S4, V3.S4], 32(OUTPUT_PTR)
	JMP	shani_loop

epilog:
	RET

// Data section
DATA _K256_1<>+0x00(SB)/4, 	$0x428a2f98
DATA _K256_1<>+0x04(SB)/4, 	$0x71374491
DATA _K256_1<>+0x08(SB)/4, 	$0xb5c0fbcf
DATA _K256_1<>+0x0c(SB)/4, 	$0xe9b5dba5
DATA _K256_1<>+0x10(SB)/4, 	$0x3956c25b
DATA _K256_1<>+0x14(SB)/4, 	$0x59f111f1
DATA _K256_1<>+0x18(SB)/4, 	$0x923f82a4
DATA _K256_1<>+0x1c(SB)/4, 	$0xab1c5ed5
DATA _K256_1<>+0x20(SB)/4, 	$0xd807aa98
DATA _K256_1<>+0x24(SB)/4, 	$0x12835b01
DATA _K256_1<>+0x28(SB)/4, 	$0x243185be
DATA _K256_1<>+0x2c(SB)/4, 	$0x550c7dc3
DATA _K256_1<>+0x30(SB)/4, 	$0x72be5d74
DATA _K256_1<>+0x34(SB)/4, 	$0x80deb1fe
DATA _K256_1<>+0x38(SB)/4, 	$0x9bdc06a7
DATA _K256_1<>+0x3c(SB)/4, 	$0xc19bf174
DATA _K256_1<>+0x40(SB)/4, 	$0xe49b69c1
DATA _K256_1<>+0x44(SB)/4, 	$0xefbe4786
DATA _K256_1<>+0x48(SB)/4, 	$0x0fc19dc6
DATA _K256_1<>+0x4c(SB)/4, 	$0x240ca1cc
DATA _K256_1<>+0x50(SB)/4, 	$0x2de92c6f
DATA _K256_1<>+0x54(SB)/4, 	$0x4a7484aa
DATA _K256_1<>+0x58(SB)/4, 	$0x5cb0a9dc
DATA _K256_1<>+0x5c(SB)/4, 	$0x76f988da
DATA _K256_1<>+0x60(SB)/4, 	$0x983e5152
DATA _K256_1<>+0x64(SB)/4, 	$0xa831c66d
DATA _K256_1<>+0x68(SB)/4, 	$0xb00327c8
DATA _K256_1<>+0x6c(SB)/4, 	$0xbf597fc7
DATA _K256_1<>+0x70(SB)/4, 	$0xc6e00bf3
DATA _K256_1<>+0x74(SB)/4, 	$0xd5a79147
DATA _K256_1<>+0x78(SB)/4, 	$0x06ca6351
DATA _K256_1<>+0x7c(SB)/4, 	$0x14292967
DATA _K256_1<>+0x80(SB)/4, 	$0x27b70a85
DATA _K256_1<>+0x84(SB)/4, 	$0x2e1b2138
DATA _K256_1<>+0x88(SB)/4, 	$0x4d2c6dfc
DATA _K256_1<>+0x8c(SB)/4, 	$0x53380d13
DATA _K256_1<>+0x90(SB)/4, 	$0x650a7354
DATA _K256_1<>+0x94(SB)/4, 	$0x766a0abb
DATA _K256_1<>+0x98(SB)/4, 	$0x81c2c92e
DATA _K256_1<>+0x9c(SB)/4, 	$0x92722c85
DATA _K256_1<>+0xa0(SB)/4, 	$0xa2bfe8a1
DATA _K256_1<>+0xa4(SB)/4, 	$0xa81a664b
DATA _K256_1<>+0xa8(SB)/4, 	$0xc24b8b70
DATA _K256_1<>+0xac(SB)/4, 	$0xc76c51a3
DATA _K256_1<>+0xb0(SB)/4, 	$0xd192e819
DATA _K256_1<>+0xb4(SB)/4, 	$0xd6990624
DATA _K256_1<>+0xb8(SB)/4, 	$0xf40e3585
DATA _K256_1<>+0xbc(SB)/4, 	$0x106aa070
DATA _K256_1<>+0xc0(SB)/4, 	$0x19a4c116
DATA _K256_1<>+0xc4(SB)/4, 	$0x1e376c08
DATA _K256_1<>+0xc8(SB)/4, 	$0x2748774c
DATA _K256_1<>+0xcc(SB)/4, 	$0x34b0bcb5
DATA _K256_1<>+0xd0(SB)/4, 	$0x391c0cb3
DATA _K256_1<>+0xd4(SB)/4, 	$0x4ed8aa4a
DATA _K256_1<>+0xd8(SB)/4, 	$0x5b9cca4f
DATA _K256_1<>+0xdc(SB)/4, 	$0x682e6ff3
DATA _K256_1<>+0xe0(SB)/4, 	$0x748f82ee
DATA _K256_1<>+0xe4(SB)/4, 	$0x78a5636f
DATA _K256_1<>+0xe8(SB)/4, 	$0x84c87814
DATA _K256_1<>+0xec(SB)/4, 	$0x8cc70208
DATA _K256_1<>+0xf0(SB)/4, 	$0x90befffa
DATA _K256_1<>+0xf4(SB)/4, 	$0xa4506ceb
DATA _K256_1<>+0xf8(SB)/4, 	$0xbef9a3f7
DATA _K256_1<>+0xfc(SB)/4, 	$0xc67178f2
GLOBL _K256_1<>(SB),(NOPTR+RODATA),$256

DATA _PADDING_1<>+0x00(SB)/4, $0xc28a2f98
DATA _PADDING_1<>+0x04(SB)/4, $0x71374491
DATA _PADDING_1<>+0x08(SB)/4, $0xb5c0fbcf
DATA _PADDING_1<>+0x0c(SB)/4, $0xe9b5dba5
DATA _PADDING_1<>+0x10(SB)/4, $0x3956c25b
DATA _PADDING_1<>+0x14(SB)/4, $0x59f111f1
DATA _PADDING_1<>+0x18(SB)/4, $0x923f82a4
DATA _PADDING_1<>+0x1c(SB)/4, $0xab1c5ed5
DATA _PADDING_1<>+0x20(SB)/4, $0xd807aa98
DATA _PADDING_1<>+0x24(SB)/4, $0x12835b01
DATA _PADDING_1<>+0x28(SB)/4, $0x243185be
DATA _PADDING_1<>+0x2c(SB)/4, $0x550c7dc3
DATA _PADDING_1<>+0x30(SB)/4, $0x72be5d74
DATA _PADDING_1<>+0x34(SB)/4, $0x80deb1fe
DATA _PADDING_1<>+0x38(SB)/4, $0x9bdc06a7
DATA _PADDING_1<>+0x3c(SB)/4, $0xc19bf374
DATA _PADDING_1<>+0x40(SB)/4, $0x649b69c1
DATA _PADDING_1<>+0x44(SB)/4, $0xf0fe4786
DATA _PADDING_1<>+0x48(SB)/4, $0x0fe1edc6
DATA _PADDING_1<>+0x4c(SB)/4, $0x240cf254
DATA _PADDING_1<>+0x50(SB)/4, $0x4fe9346f
DATA _PADDING_1<>+0x54(SB)/4, $0x6cc984be
DATA _PADDING_1<>+0x58(SB)/4, $0x61b9411e
DATA _PADDING_1<>+0x5c(SB)/4, $0x16f988fa
DATA _PADDING_1<>+0x60(SB)/4, $0xf2c65152
DATA _PADDING_1<>+0x64(SB)/4, $0xa88e5a6d
DATA _PADDING_1<>+0x68(SB)/4, $0xb019fc65
DATA _PADDING_1<>+0x6c(SB)/4, $0xb9d99ec7
DATA _PADDING_1<>+0x70(SB)/4, $0x9a1231c3
DATA _PADDING_1<>+0x74(SB)/4, $0xe70eeaa0
DATA _PADDING_1<>+0x78(SB)/4, $0xfdb1232b
DATA _PADDING_1<>+0x7c(SB)/4, $0xc7353eb0
DATA _PADDING_1<>+0x80(SB)/4, $0x3069bad5
DATA _PADDING_1<>+0x84(SB)/4, $0xcb976d5f
DATA _PADDING_1<>+0x88(SB)/4, $0x5a0f118f
DATA _PADDING_1<>+0x8c(SB)/4, $0xdc1eeefd
DATA _PADDING_1<>+0x90(SB)/4, $0x0a35b689
DATA _PADDING_1<>+0x94(SB)/4, $0xde0b7a04
DATA _PADDING_1<>+0x98(SB)/4, $0x58f4ca9d
DATA _PADDING_1<>+0x9c(SB)/4, $0xe15d5b16
DATA _PADDING_1<>+0xa0(SB)/4, $0x007f3e86
DATA _PADDING_1<>+0xa4(SB)/4, $0x37088980
DATA _PADDING_1<>+0xa8(SB)/4, $0xa507ea32
DATA _PADDING_1<>+0xac(SB)/4, $0x6fab9537
DATA _PADDING_1<>+0xb0(SB)/4, $0x17406110
DATA _PADDING_1<>+0xb4(SB)/4, $0x0d8cd6f1
DATA _PADDING_1<>+0xb8(SB)/4, $0xcdaa3b6d
DATA _PADDING_1<>+0xbc(SB)/4, $0xc0bbbe37
DATA _PADDING_1<>+0xc0(SB)/4, $0x83613bda
DATA _PADDING_1<>+0xc4(SB)/4, $0xdb48a363
DATA _PADDING_1<>+0xc8(SB)/4, $0x0b02e931
DATA _PADDING_1<>+0xcc(SB)/4, $0x6fd15ca7
DATA _PADDING_1<>+0xd0(SB)/4, $0x521afaca
DATA _PADDING_1<>+0xd4(SB)/4, $0x31338431
DATA _PADDING_1<>+0xd8(SB)/4, $0x6ed41a95
DATA _PADDING_1<>+0xdc(SB)/4, $0x6d437890
DATA _PADDING_1<>+0xe0(SB)/4, $0xc39c91f2
DATA _PADDING_1<>+0xe4(SB)/4, $0x9eccabbd
DATA _PADDING_1<>+0xe8(SB)/4, $0xb5c9a0e6
DATA _PADDING_1<>+0xec(SB)/4, $0x532fb63c
DATA _PADDING_1<>+0xf0(SB)/4, $0xd2c741c6
DATA _PADDING_1<>+0xf4(SB)/4, $0x07237ea3
DATA _PADDING_1<>+0xf8(SB)/4, $0xa4954b68
DATA _PADDING_1<>+0xfc(SB)/4, $0x4c191d76
GLOBL _PADDING_1<>(SB),(NOPTR+RODATA),$256

DATA _DIGEST_1<>+0(SB)/4, $0x6a09e667
DATA _DIGEST_1<>+4(SB)/4, $0xbb67ae85
DATA _DIGEST_1<>+8(SB)/4, $0x3c6ef372
DATA _DIGEST_1<>+12(SB)/4, $0xa54ff53a
DATA _DIGEST_1<>+16(SB)/4, $0x510e527f
DATA _DIGEST_1<>+20(SB)/4, $0x9b05688c
DATA _DIGEST_1<>+24(SB)/4, $0x1f83d9ab
DATA _DIGEST_1<>+28(SB)/4, $0x5be0cd19
GLOBL _DIGEST_1<>(SB),(NOPTR+RODATA),$32

DATA _DIGEST_4<>+0(SB)/8, $0x6a09e6676a09e667
DATA _DIGEST_4<>+8(SB)/8, $0x6a09e6676a09e667
DATA _DIGEST_4<>+16(SB)/8, $0xbb67ae85bb67ae85
DATA _DIGEST_4<>+24(SB)/8, $0xbb67ae85bb67ae85
DATA _DIGEST_4<>+32(SB)/8, $0x3c6ef3723c6ef372
DATA _DIGEST_4<>+40(SB)/8, $0x3c6ef3723c6ef372
DATA _DIGEST_4<>+48(SB)/8, $0xa54ff53aa54ff53a
DATA _DIGEST_4<>+56(SB)/8, $0xa54ff53aa54ff53a
DATA _DIGEST_4<>+64(SB)/8, $0x510e527f510e527f
DATA _DIGEST_4<>+72(SB)/8, $0x510e527f510e527f
DATA _DIGEST_4<>+80(SB)/8, $0x9b05688c9b05688c
DATA _DIGEST_4<>+88(SB)/8, $0x9b05688c9b05688c
DATA _DIGEST_4<>+96(SB)/8, $0x1f83d9ab1f83d9ab
DATA _DIGEST_4<>+104(SB)/8, $0x1f83d9ab1f83d9ab
DATA _DIGEST_4<>+112(SB)/8, $0x5be0cd195be0cd19
DATA _DIGEST_4<>+120(SB)/8, $0x5be0cd195be0cd19
GLOBL _DIGEST_4<>(SB),(NOPTR+RODATA),$128


DATA _PADDING_4<>+0(SB)/8, $0xc28a2f98c28a2f98
DATA _PADDING_4<>+8(SB)/8, $0xc28a2f98c28a2f98
DATA _PADDING_4<>+16(SB)/8, $0x7137449171374491
DATA _PADDING_4<>+24(SB)/8, $0x7137449171374491
DATA _PADDING_4<>+32(SB)/8, $0xb5c0fbcfb5c0fbcf
DATA _PADDING_4<>+40(SB)/8, $0xb5c0fbcfb5c0fbcf
DATA _PADDING_4<>+48(SB)/8, $0xe9b5dba5e9b5dba5
DATA _PADDING_4<>+56(SB)/8, $0xe9b5dba5e9b5dba5
DATA _PADDING_4<>+64(SB)/8, $0x3956c25b3956c25b
DATA _PADDING_4<>+72(SB)/8, $0x3956c25b3956c25b
DATA _PADDING_4<>+80(SB)/8, $0x59f111f159f111f1
DATA _PADDING_4<>+88(SB)/8, $0x59f111f159f111f1
DATA _PADDING_4<>+96(SB)/8, $0x923f82a4923f82a4
DATA _PADDING_4<>+104(SB)/8, $0x923f82a4923f82a4
DATA _PADDING_4<>+112(SB)/8, $0xab1c5ed5ab1c5ed5
DATA _PADDING_4<>+120(SB)/8, $0xab1c5ed5ab1c5ed5
DATA _PADDING_4<>+128(SB)/8, $0xd807aa98d807aa98
DATA _PADDING_4<>+136(SB)/8, $0xd807aa98d807aa98
DATA _PADDING_4<>+144(SB)/8, $0x12835b0112835b01
DATA _PADDING_4<>+152(SB)/8, $0x12835b0112835b01
DATA _PADDING_4<>+160(SB)/8, $0x243185be243185be
DATA _PADDING_4<>+168(SB)/8, $0x243185be243185be
DATA _PADDING_4<>+176(SB)/8, $0x550c7dc3550c7dc3
DATA _PADDING_4<>+184(SB)/8, $0x550c7dc3550c7dc3
DATA _PADDING_4<>+192(SB)/8, $0x72be5d7472be5d74
DATA _PADDING_4<>+200(SB)/8, $0x72be5d7472be5d74
DATA _PADDING_4<>+208(SB)/8, $0x80deb1fe80deb1fe
DATA _PADDING_4<>+216(SB)/8, $0x80deb1fe80deb1fe
DATA _PADDING_4<>+224(SB)/8, $0x9bdc06a79bdc06a7
DATA _PADDING_4<>+232(SB)/8, $0x9bdc06a79bdc06a7
DATA _PADDING_4<>+240(SB)/8, $0xc19bf374c19bf374
DATA _PADDING_4<>+248(SB)/8, $0xc19bf374c19bf374
DATA _PADDING_4<>+256(SB)/8, $0x649b69c1649b69c1
DATA _PADDING_4<>+264(SB)/8, $0x649b69c1649b69c1
DATA _PADDING_4<>+272(SB)/8, $0xf0fe4786f0fe4786
DATA _PADDING_4<>+280(SB)/8, $0xf0fe4786f0fe4786
DATA _PADDING_4<>+288(SB)/8, $0x0fe1edc60fe1edc6
DATA _PADDING_4<>+296(SB)/8, $0x0fe1edc60fe1edc6
DATA _PADDING_4<>+304(SB)/8, $0x240cf254240cf254
DATA _PADDING_4<>+312(SB)/8, $0x240cf254240cf254
DATA _PADDING_4<>+320(SB)/8, $0x4fe9346f4fe9346f
DATA _PADDING_4<>+328(SB)/8, $0x4fe9346f4fe9346f
DATA _PADDING_4<>+336(SB)/8, $0x6cc984be6cc984be
DATA _PADDING_4<>+344(SB)/8, $0x6cc984be6cc984be
DATA _PADDING_4<>+352(SB)/8, $0x61b9411e61b9411e
DATA _PADDING_4<>+360(SB)/8, $0x61b9411e61b9411e
DATA _PADDING_4<>+368(SB)/8, $0x16f988fa16f988fa
DATA _PADDING_4<>+376(SB)/8, $0x16f988fa16f988fa
DATA _PADDING_4<>+384(SB)/8, $0xf2c65152f2c65152
DATA _PADDING_4<>+392(SB)/8, $0xf2c65152f2c65152
DATA _PADDING_4<>+400(SB)/8, $0xa88e5a6da88e5a6d
DATA _PADDING_4<>+408(SB)/8, $0xa88e5a6da88e5a6d
DATA _PADDING_4<>+416(SB)/8, $0xb019fc65b019fc65
DATA _PADDING_4<>+424(SB)/8, $0xb019fc65b019fc65
DATA _PADDING_4<>+432(SB)/8, $0xb9d99ec7b9d99ec7
DATA _PADDING_4<>+440(SB)/8, $0xb9d99ec7b9d99ec7
DATA _PADDING_4<>+448(SB)/8, $0x9a1231c39a1231c3
DATA _PADDING_4<>+456(SB)/8, $0x9a1231c39a1231c3
DATA _PADDING_4<>+464(SB)/8, $0xe70eeaa0e70eeaa0
DATA _PADDING_4<>+472(SB)/8, $0xe70eeaa0e70eeaa0
DATA _PADDING_4<>+480(SB)/8, $0xfdb1232bfdb1232b
DATA _PADDING_4<>+488(SB)/8, $0xfdb1232bfdb1232b
DATA _PADDING_4<>+496(SB)/8, $0xc7353eb0c7353eb0
DATA _PADDING_4<>+504(SB)/8, $0xc7353eb0c7353eb0
DATA _PADDING_4<>+512(SB)/8, $0x3069bad53069bad5
DATA _PADDING_4<>+520(SB)/8, $0x3069bad53069bad5
DATA _PADDING_4<>+528(SB)/8, $0xcb976d5fcb976d5f
DATA _PADDING_4<>+536(SB)/8, $0xcb976d5fcb976d5f
DATA _PADDING_4<>+544(SB)/8, $0x5a0f118f5a0f118f
DATA _PADDING_4<>+552(SB)/8, $0x5a0f118f5a0f118f
DATA _PADDING_4<>+560(SB)/8, $0xdc1eeefddc1eeefd
DATA _PADDING_4<>+568(SB)/8, $0xdc1eeefddc1eeefd
DATA _PADDING_4<>+576(SB)/8, $0x0a35b6890a35b689
DATA _PADDING_4<>+584(SB)/8, $0x0a35b6890a35b689
DATA _PADDING_4<>+592(SB)/8, $0xde0b7a04de0b7a04
DATA _PADDING_4<>+600(SB)/8, $0xde0b7a04de0b7a04
DATA _PADDING_4<>+608(SB)/8, $0x58f4ca9d58f4ca9d
DATA _PADDING_4<>+616(SB)/8, $0x58f4ca9d58f4ca9d
DATA _PADDING_4<>+624(SB)/8, $0xe15d5b16e15d5b16
DATA _PADDING_4<>+632(SB)/8, $0xe15d5b16e15d5b16
DATA _PADDING_4<>+640(SB)/8, $0x007f3e86007f3e86
DATA _PADDING_4<>+648(SB)/8, $0x007f3e86007f3e86
DATA _PADDING_4<>+656(SB)/8, $0x3708898037088980
DATA _PADDING_4<>+664(SB)/8, $0x3708898037088980
DATA _PADDING_4<>+672(SB)/8, $0xa507ea32a507ea32
DATA _PADDING_4<>+680(SB)/8, $0xa507ea32a507ea32
DATA _PADDING_4<>+688(SB)/8, $0x6fab95376fab9537
DATA _PADDING_4<>+696(SB)/8, $0x6fab95376fab9537
DATA _PADDING_4<>+704(SB)/8, $0x1740611017406110
DATA _PADDING_4<>+712(SB)/8, $0x1740611017406110
DATA _PADDING_4<>+720(SB)/8, $0x0d8cd6f10d8cd6f1
DATA _PADDING_4<>+728(SB)/8, $0x0d8cd6f10d8cd6f1
DATA _PADDING_4<>+736(SB)/8, $0xcdaa3b6dcdaa3b6d
DATA _PADDING_4<>+744(SB)/8, $0xcdaa3b6dcdaa3b6d
DATA _PADDING_4<>+752(SB)/8, $0xc0bbbe37c0bbbe37
DATA _PADDING_4<>+760(SB)/8, $0xc0bbbe37c0bbbe37
DATA _PADDING_4<>+768(SB)/8, $0x83613bda83613bda
DATA _PADDING_4<>+776(SB)/8, $0x83613bda83613bda
DATA _PADDING_4<>+784(SB)/8, $0xdb48a363db48a363
DATA _PADDING_4<>+792(SB)/8, $0xdb48a363db48a363
DATA _PADDING_4<>+800(SB)/8, $0x0b02e9310b02e931
DATA _PADDING_4<>+808(SB)/8, $0x0b02e9310b02e931
DATA _PADDING_4<>+816(SB)/8, $0x6fd15ca76fd15ca7
DATA _PADDING_4<>+824(SB)/8, $0x6fd15ca76fd15ca7
DATA _PADDING_4<>+832(SB)/8, $0x521afaca521afaca
DATA _PADDING_4<>+840(SB)/8, $0x521afaca521afaca
DATA _PADDING_4<>+848(SB)/8, $0x3133843131338431
DATA _PADDING_4<>+856(SB)/8, $0x3133843131338431
DATA _PADDING_4<>+864(SB)/8, $0x6ed41a956ed41a95
DATA _PADDING_4<>+872(SB)/8, $0x6ed41a956ed41a95
DATA _PADDING_4<>+880(SB)/8, $0x6d4378906d437890
DATA _PADDING_4<>+888(SB)/8, $0x6d4378906d437890
DATA _PADDING_4<>+896(SB)/8, $0xc39c91f2c39c91f2
DATA _PADDING_4<>+904(SB)/8, $0xc39c91f2c39c91f2
DATA _PADDING_4<>+912(SB)/8, $0x9eccabbd9eccabbd
DATA _PADDING_4<>+920(SB)/8, $0x9eccabbd9eccabbd
DATA _PADDING_4<>+928(SB)/8, $0xb5c9a0e6b5c9a0e6
DATA _PADDING_4<>+936(SB)/8, $0xb5c9a0e6b5c9a0e6
DATA _PADDING_4<>+944(SB)/8, $0x532fb63c532fb63c
DATA _PADDING_4<>+952(SB)/8, $0x532fb63c532fb63c
DATA _PADDING_4<>+960(SB)/8, $0xd2c741c6d2c741c6
DATA _PADDING_4<>+968(SB)/8, $0xd2c741c6d2c741c6
DATA _PADDING_4<>+976(SB)/8, $0x07237ea307237ea3
DATA _PADDING_4<>+984(SB)/8, $0x07237ea307237ea3
DATA _PADDING_4<>+992(SB)/8, $0xa4954b68a4954b68
DATA _PADDING_4<>+1000(SB)/8, $0xa4954b68a4954b68
DATA _PADDING_4<>+1008(SB)/8, $0x4c191d764c191d76
DATA _PADDING_4<>+1016(SB)/8, $0x4c191d764c191d76
GLOBL _PADDING_4<>(SB),(NOPTR+RODATA),$1024

DATA _K256_4<>+0(SB)/8, $0x428a2f98428a2f98
DATA _K256_4<>+8(SB)/8, $0x428a2f98428a2f98
DATA _K256_4<>+16(SB)/8, $0x7137449171374491
DATA _K256_4<>+24(SB)/8, $0x7137449171374491
DATA _K256_4<>+32(SB)/8, $0xb5c0fbcfb5c0fbcf
DATA _K256_4<>+40(SB)/8, $0xb5c0fbcfb5c0fbcf
DATA _K256_4<>+48(SB)/8, $0xe9b5dba5e9b5dba5
DATA _K256_4<>+56(SB)/8, $0xe9b5dba5e9b5dba5
DATA _K256_4<>+64(SB)/8, $0x3956c25b3956c25b
DATA _K256_4<>+72(SB)/8, $0x3956c25b3956c25b
DATA _K256_4<>+80(SB)/8, $0x59f111f159f111f1
DATA _K256_4<>+88(SB)/8, $0x59f111f159f111f1
DATA _K256_4<>+96(SB)/8, $0x923f82a4923f82a4
DATA _K256_4<>+104(SB)/8, $0x923f82a4923f82a4
DATA _K256_4<>+112(SB)/8, $0xab1c5ed5ab1c5ed5
DATA _K256_4<>+120(SB)/8, $0xab1c5ed5ab1c5ed5
DATA _K256_4<>+128(SB)/8, $0xd807aa98d807aa98
DATA _K256_4<>+136(SB)/8, $0xd807aa98d807aa98
DATA _K256_4<>+144(SB)/8, $0x12835b0112835b01
DATA _K256_4<>+152(SB)/8, $0x12835b0112835b01
DATA _K256_4<>+160(SB)/8, $0x243185be243185be
DATA _K256_4<>+168(SB)/8, $0x243185be243185be
DATA _K256_4<>+176(SB)/8, $0x550c7dc3550c7dc3
DATA _K256_4<>+184(SB)/8, $0x550c7dc3550c7dc3
DATA _K256_4<>+192(SB)/8, $0x72be5d7472be5d74
DATA _K256_4<>+200(SB)/8, $0x72be5d7472be5d74
DATA _K256_4<>+208(SB)/8, $0x80deb1fe80deb1fe
DATA _K256_4<>+216(SB)/8, $0x80deb1fe80deb1fe
DATA _K256_4<>+224(SB)/8, $0x9bdc06a79bdc06a7
DATA _K256_4<>+232(SB)/8, $0x9bdc06a79bdc06a7
DATA _K256_4<>+240(SB)/8, $0xc19bf174c19bf174
DATA _K256_4<>+248(SB)/8, $0xc19bf174c19bf174
DATA _K256_4<>+256(SB)/8, $0xe49b69c1e49b69c1
DATA _K256_4<>+264(SB)/8, $0xe49b69c1e49b69c1
DATA _K256_4<>+272(SB)/8, $0xefbe4786efbe4786
DATA _K256_4<>+280(SB)/8, $0xefbe4786efbe4786
DATA _K256_4<>+288(SB)/8, $0x0fc19dc60fc19dc6
DATA _K256_4<>+296(SB)/8, $0x0fc19dc60fc19dc6
DATA _K256_4<>+304(SB)/8, $0x240ca1cc240ca1cc
DATA _K256_4<>+312(SB)/8, $0x240ca1cc240ca1cc
DATA _K256_4<>+320(SB)/8, $0x2de92c6f2de92c6f
DATA _K256_4<>+328(SB)/8, $0x2de92c6f2de92c6f
DATA _K256_4<>+336(SB)/8, $0x4a7484aa4a7484aa
DATA _K256_4<>+344(SB)/8, $0x4a7484aa4a7484aa
DATA _K256_4<>+352(SB)/8, $0x5cb0a9dc5cb0a9dc
DATA _K256_4<>+360(SB)/8, $0x5cb0a9dc5cb0a9dc
DATA _K256_4<>+368(SB)/8, $0x76f988da76f988da
DATA _K256_4<>+376(SB)/8, $0x76f988da76f988da
DATA _K256_4<>+384(SB)/8, $0x983e5152983e5152
DATA _K256_4<>+392(SB)/8, $0x983e5152983e5152
DATA _K256_4<>+400(SB)/8, $0xa831c66da831c66d
DATA _K256_4<>+408(SB)/8, $0xa831c66da831c66d
DATA _K256_4<>+416(SB)/8, $0xb00327c8b00327c8
DATA _K256_4<>+424(SB)/8, $0xb00327c8b00327c8
DATA _K256_4<>+432(SB)/8, $0xbf597fc7bf597fc7
DATA _K256_4<>+440(SB)/8, $0xbf597fc7bf597fc7
DATA _K256_4<>+448(SB)/8, $0xc6e00bf3c6e00bf3
DATA _K256_4<>+456(SB)/8, $0xc6e00bf3c6e00bf3
DATA _K256_4<>+464(SB)/8, $0xd5a79147d5a79147
DATA _K256_4<>+472(SB)/8, $0xd5a79147d5a79147
DATA _K256_4<>+480(SB)/8, $0x06ca635106ca6351
DATA _K256_4<>+488(SB)/8, $0x06ca635106ca6351
DATA _K256_4<>+496(SB)/8, $0x1429296714292967
DATA _K256_4<>+504(SB)/8, $0x1429296714292967
DATA _K256_4<>+512(SB)/8, $0x27b70a8527b70a85
DATA _K256_4<>+520(SB)/8, $0x27b70a8527b70a85
DATA _K256_4<>+528(SB)/8, $0x2e1b21382e1b2138
DATA _K256_4<>+536(SB)/8, $0x2e1b21382e1b2138
DATA _K256_4<>+544(SB)/8, $0x4d2c6dfc4d2c6dfc
DATA _K256_4<>+552(SB)/8, $0x4d2c6dfc4d2c6dfc
DATA _K256_4<>+560(SB)/8, $0x53380d1353380d13
DATA _K256_4<>+568(SB)/8, $0x53380d1353380d13
DATA _K256_4<>+576(SB)/8, $0x650a7354650a7354
DATA _K256_4<>+584(SB)/8, $0x650a7354650a7354
DATA _K256_4<>+592(SB)/8, $0x766a0abb766a0abb
DATA _K256_4<>+600(SB)/8, $0x766a0abb766a0abb
DATA _K256_4<>+608(SB)/8, $0x81c2c92e81c2c92e
DATA _K256_4<>+616(SB)/8, $0x81c2c92e81c2c92e
DATA _K256_4<>+624(SB)/8, $0x92722c8592722c85
DATA _K256_4<>+632(SB)/8, $0x92722c8592722c85
DATA _K256_4<>+640(SB)/8, $0xa2bfe8a1a2bfe8a1
DATA _K256_4<>+648(SB)/8, $0xa2bfe8a1a2bfe8a1
DATA _K256_4<>+656(SB)/8, $0xa81a664ba81a664b
DATA _K256_4<>+664(SB)/8, $0xa81a664ba81a664b
DATA _K256_4<>+672(SB)/8, $0xc24b8b70c24b8b70
DATA _K256_4<>+680(SB)/8, $0xc24b8b70c24b8b70
DATA _K256_4<>+688(SB)/8, $0xc76c51a3c76c51a3
DATA _K256_4<>+696(SB)/8, $0xc76c51a3c76c51a3
DATA _K256_4<>+704(SB)/8, $0xd192e819d192e819
DATA _K256_4<>+712(SB)/8, $0xd192e819d192e819
DATA _K256_4<>+720(SB)/8, $0xd6990624d6990624
DATA _K256_4<>+728(SB)/8, $0xd6990624d6990624
DATA _K256_4<>+736(SB)/8, $0xf40e3585f40e3585
DATA _K256_4<>+744(SB)/8, $0xf40e3585f40e3585
DATA _K256_4<>+752(SB)/8, $0x106aa070106aa070
DATA _K256_4<>+760(SB)/8, $0x106aa070106aa070
DATA _K256_4<>+768(SB)/8, $0x19a4c11619a4c116
DATA _K256_4<>+776(SB)/8, $0x19a4c11619a4c116
DATA _K256_4<>+784(SB)/8, $0x1e376c081e376c08
DATA _K256_4<>+792(SB)/8, $0x1e376c081e376c08
DATA _K256_4<>+800(SB)/8, $0x2748774c2748774c
DATA _K256_4<>+808(SB)/8, $0x2748774c2748774c
DATA _K256_4<>+816(SB)/8, $0x34b0bcb534b0bcb5
DATA _K256_4<>+824(SB)/8, $0x34b0bcb534b0bcb5
DATA _K256_4<>+832(SB)/8, $0x391c0cb3391c0cb3
DATA _K256_4<>+840(SB)/8, $0x391c0cb3391c0cb3
DATA _K256_4<>+848(SB)/8, $0x4ed8aa4a4ed8aa4a
DATA _K256_4<>+856(SB)/8, $0x4ed8aa4a4ed8aa4a
DATA _K256_4<>+864(SB)/8, $0x5b9cca4f5b9cca4f
DATA _K256_4<>+872(SB)/8, $0x5b9cca4f5b9cca4f
DATA _K256_4<>+880(SB)/8, $0x682e6ff3682e6ff3
DATA _K256_4<>+888(SB)/8, $0x682e6ff3682e6ff3
DATA _K256_4<>+896(SB)/8, $0x748f82ee748f82ee
DATA _K256_4<>+904(SB)/8, $0x748f82ee748f82ee
DATA _K256_4<>+912(SB)/8, $0x78a5636f78a5636f
DATA _K256_4<>+920(SB)/8, $0x78a5636f78a5636f
DATA _K256_4<>+928(SB)/8, $0x84c8781484c87814
DATA _K256_4<>+936(SB)/8, $0x84c8781484c87814
DATA _K256_4<>+944(SB)/8, $0x8cc702088cc70208
DATA _K256_4<>+952(SB)/8, $0x8cc702088cc70208
DATA _K256_4<>+960(SB)/8, $0x90befffa90befffa
DATA _K256_4<>+968(SB)/8, $0x90befffa90befffa
DATA _K256_4<>+976(SB)/8, $0xa4506ceba4506ceb
DATA _K256_4<>+984(SB)/8, $0xa4506ceba4506ceb
DATA _K256_4<>+992(SB)/8, $0xbef9a3f7bef9a3f7
DATA _K256_4<>+1000(SB)/8, $0xbef9a3f7bef9a3f7
DATA _K256_4<>+1008(SB)/8, $0xc67178f2c67178f2
DATA _K256_4<>+1016(SB)/8, $0xc67178f2c67178f2
GLOBL _K256_4<>(SB),(NOPTR+RODATA),$1024
