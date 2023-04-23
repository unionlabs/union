// Copyright (c) 2017 Minio Inc. All rights reserved.
// Use of this source code is governed by a license that can be
// found in the LICENSE file.

// +build amd64,!gccgo,!appengine,!nacl,!noasm

#include "textflag.h"

DATA ·consAVX2<>+0x00(SB)/8, $0xdbe6d5d5fe4cce2f
DATA ·consAVX2<>+0x08(SB)/8, $0xa4093822299f31d0
DATA ·consAVX2<>+0x10(SB)/8, $0x13198a2e03707344
DATA ·consAVX2<>+0x18(SB)/8, $0x243f6a8885a308d3
DATA ·consAVX2<>+0x20(SB)/8, $0x3bd39e10cb0ef593
DATA ·consAVX2<>+0x28(SB)/8, $0xc0acf169b5f18a8c
DATA ·consAVX2<>+0x30(SB)/8, $0xbe5466cf34e90c6c
DATA ·consAVX2<>+0x38(SB)/8, $0x452821e638d01377
GLOBL ·consAVX2<>(SB), (NOPTR+RODATA), $64

DATA ·zipperMergeAVX2<>+0x00(SB)/8, $0xf010e05020c03
DATA ·zipperMergeAVX2<>+0x08(SB)/8, $0x70806090d0a040b
DATA ·zipperMergeAVX2<>+0x10(SB)/8, $0xf010e05020c03
DATA ·zipperMergeAVX2<>+0x18(SB)/8, $0x70806090d0a040b
GLOBL ·zipperMergeAVX2<>(SB), (NOPTR+RODATA), $32

#define REDUCE_MOD(x0, x1, x2, x3, tmp0, tmp1, y0, y1) \
	MOVQ $0x3FFFFFFFFFFFFFFF, tmp0 \
	ANDQ tmp0, x3                  \
	MOVQ x2, y0                    \
	MOVQ x3, y1                    \
	                               \
	MOVQ x2, tmp0                  \
	MOVQ x3, tmp1                  \
	SHLQ $1, tmp1                  \
	SHRQ $63, tmp0                 \
	MOVQ tmp1, x3                  \
	ORQ  tmp0, x3                  \
	                               \
	SHLQ $1, x2                    \
	                               \
	MOVQ y0, tmp0                  \
	MOVQ y1, tmp1                  \
	SHLQ $2, tmp1                  \
	SHRQ $62, tmp0                 \
	MOVQ tmp1, y1                  \
	ORQ  tmp0, y1                  \
	                               \
	SHLQ $2, y0                    \
	                               \
	XORQ x0, y0                    \
	XORQ x2, y0                    \
	XORQ x1, y1                    \
	XORQ x3, y1

#define UPDATE(msg) \
	VPADDQ  msg, Y2, Y2                               \
	VPADDQ  Y3, Y2, Y2                                \
	                                                  \
	VPSRLQ  $32, Y1, Y0                               \
	BYTE    $0xC5; BYTE $0xFD; BYTE $0xF4; BYTE $0xC2 \ // VPMULUDQ Y2, Y0, Y0
	VPXOR   Y0, Y3, Y3                                \
	                                                  \
	VPADDQ  Y4, Y1, Y1                                \
	                                                  \
	VPSRLQ  $32, Y2, Y0                               \
	BYTE    $0xC5; BYTE $0xFD; BYTE $0xF4; BYTE $0xC1 \ // VPMULUDQ Y1, Y0, Y0
	VPXOR   Y0, Y4, Y4                                \
	                                                  \
	VPSHUFB Y5, Y2, Y0                                \
	VPADDQ  Y0, Y1, Y1                                \
	                                                  \
	VPSHUFB Y5, Y1, Y0                                \
	VPADDQ  Y0, Y2, Y2

// func initializeAVX2(state *[16]uint64, key []byte)
TEXT ·initializeAVX2(SB), 4, $0-32
	MOVQ state+0(FP), AX
	MOVQ key_base+8(FP), BX
	MOVQ $·consAVX2<>(SB), CX

	VMOVDQU 0(BX), Y1
	VPSHUFD $177, Y1, Y2

	VMOVDQU 0(CX), Y3
	VMOVDQU 32(CX), Y4

	VPXOR Y3, Y1, Y1
	VPXOR Y4, Y2, Y2

	VMOVDQU Y1, 0(AX)
	VMOVDQU Y2, 32(AX)
	VMOVDQU Y3, 64(AX)
	VMOVDQU Y4, 96(AX)
	VZEROUPPER
	RET

// func updateAVX2(state *[16]uint64, msg []byte)
TEXT ·updateAVX2(SB), 4, $0-32
	MOVQ state+0(FP), AX
	MOVQ msg_base+8(FP), BX
	MOVQ msg_len+16(FP), CX

	CMPQ CX, $32
	JB   DONE

	VMOVDQU 0(AX), Y1
	VMOVDQU 32(AX), Y2
	VMOVDQU 64(AX), Y3
	VMOVDQU 96(AX), Y4

	VMOVDQU ·zipperMergeAVX2<>(SB), Y5

LOOP:
	VMOVDQU 0(BX), Y0
	UPDATE(Y0)

	ADDQ $32, BX
	SUBQ $32, CX
	JA   LOOP

	VMOVDQU Y1, 0(AX)
	VMOVDQU Y2, 32(AX)
	VMOVDQU Y3, 64(AX)
	VMOVDQU Y4, 96(AX)
	VZEROUPPER

DONE:
	RET

// func finalizeAVX2(out []byte, state *[16]uint64)
TEXT ·finalizeAVX2(SB), 4, $0-32
	MOVQ state+24(FP), AX
	MOVQ out_base+0(FP), BX
	MOVQ out_len+8(FP), CX

	VMOVDQU 0(AX), Y1
	VMOVDQU 32(AX), Y2
	VMOVDQU 64(AX), Y3
	VMOVDQU 96(AX), Y4

	VMOVDQU ·zipperMergeAVX2<>(SB), Y5

	VPERM2I128 $1, Y1, Y1, Y0
	VPSHUFD    $177, Y0, Y0
	UPDATE(Y0)

	VPERM2I128 $1, Y1, Y1, Y0
	VPSHUFD    $177, Y0, Y0
	UPDATE(Y0)

	VPERM2I128 $1, Y1, Y1, Y0
	VPSHUFD    $177, Y0, Y0
	UPDATE(Y0)

	VPERM2I128 $1, Y1, Y1, Y0
	VPSHUFD    $177, Y0, Y0
	UPDATE(Y0)

	CMPQ CX, $8
	JE   skipUpdate // Just 4 rounds for 64-bit checksum

	VPERM2I128 $1, Y1, Y1, Y0
	VPSHUFD    $177, Y0, Y0
	UPDATE(Y0)

	VPERM2I128 $1, Y1, Y1, Y0
	VPSHUFD    $177, Y0, Y0
	UPDATE(Y0)

	CMPQ CX, $16
	JE   skipUpdate // 6 rounds for 128-bit checksum

	VPERM2I128 $1, Y1, Y1, Y0
	VPSHUFD    $177, Y0, Y0
	UPDATE(Y0)

	VPERM2I128 $1, Y1, Y1, Y0
	VPSHUFD    $177, Y0, Y0
	UPDATE(Y0)

	VPERM2I128 $1, Y1, Y1, Y0
	VPSHUFD    $177, Y0, Y0
	UPDATE(Y0)

	VPERM2I128 $1, Y1, Y1, Y0
	VPSHUFD    $177, Y0, Y0
	UPDATE(Y0)

skipUpdate:
	VMOVDQU Y1, 0(AX)
	VMOVDQU Y2, 32(AX)
	VMOVDQU Y3, 64(AX)
	VMOVDQU Y4, 96(AX)
	VZEROUPPER

	CMPQ CX, $8
	JE   hash64
	CMPQ CX, $16
	JE   hash128

	// 256-bit checksum
	MOVQ 0*8(AX), R8
	MOVQ 1*8(AX), R9
	MOVQ 4*8(AX), R10
	MOVQ 5*8(AX), R11
	ADDQ 8*8(AX), R8
	ADDQ 9*8(AX), R9
	ADDQ 12*8(AX), R10
	ADDQ 13*8(AX), R11

	REDUCE_MOD(R8, R9, R10, R11, R12, R13, R14, R15)
	MOVQ R14, 0(BX)
	MOVQ R15, 8(BX)

	MOVQ 2*8(AX), R8
	MOVQ 3*8(AX), R9
	MOVQ 6*8(AX), R10
	MOVQ 7*8(AX), R11
	ADDQ 10*8(AX), R8
	ADDQ 11*8(AX), R9
	ADDQ 14*8(AX), R10
	ADDQ 15*8(AX), R11

	REDUCE_MOD(R8, R9, R10, R11, R12, R13, R14, R15)
	MOVQ R14, 16(BX)
	MOVQ R15, 24(BX)
	RET

hash128:
	MOVQ 0*8(AX), R8
	MOVQ 1*8(AX), R9
	ADDQ 6*8(AX), R8
	ADDQ 7*8(AX), R9
	ADDQ 8*8(AX), R8
	ADDQ 9*8(AX), R9
	ADDQ 14*8(AX), R8
	ADDQ 15*8(AX), R9
	MOVQ R8, 0(BX)
	MOVQ R9, 8(BX)
	RET

hash64:
	MOVQ 0*8(AX), DX
	ADDQ 4*8(AX), DX
	ADDQ 8*8(AX), DX
	ADDQ 12*8(AX), DX
	MOVQ DX, 0(BX)
	RET

