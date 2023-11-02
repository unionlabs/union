//
// Copyright Supranational LLC
// Licensed under the Apache License, Version 2.0, see LICENSE for details.
// SPDX-License-Identifier: Apache-2.0
//
// ====================================================================
// Written by Andy Polyakov, @dot-asm, initially for the OpenSSL
// project.
// ====================================================================
//
// sha256_block procedure for ARMv8.
//
// This module is stripped of scalar code paths, with raionale that all
// known processors are NEON-capable.
//
// See original module at CRYPTOGAMS for further details.

.text

.align	6

LK256:
.long	0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5
.long	0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5
.long	0xd807aa98,0x12835b01,0x243185be,0x550c7dc3
.long	0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174
.long	0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc
.long	0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da
.long	0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7
.long	0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967
.long	0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13
.long	0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85
.long	0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3
.long	0xd192e819,0xd6990624,0xf40e3585,0x106aa070
.long	0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5
.long	0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3
.long	0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208
.long	0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2
.long	0	//terminator

.byte	83,72,65,50,53,54,32,98,108,111,99,107,32,116,114,97,110,115,102,111,114,109,32,102,111,114,32,65,82,77,118,56,44,32,67,82,89,80,84,79,71,65,77,83,32,98,121,32,64,100,111,116,45,97,115,109,0
.align	2
.align	2
.globl	_blst_sha256_block_armv8

.align	6
_blst_sha256_block_armv8:
Lv8_entry:
	stp	x29,x30,[sp,#-16]!
	add	x29,sp,#0

	ld1	{v0.4s,v1.4s},[x0]
	adr	x3,LK256

Loop_hw:
	ld1	{v4.16b,v5.16b,v6.16b,v7.16b},[x1],#64
	sub	x2,x2,#1
	ld1	{v16.4s},[x3],#16
	rev32	v4.16b,v4.16b
	rev32	v5.16b,v5.16b
	rev32	v6.16b,v6.16b
	rev32	v7.16b,v7.16b
	orr	v18.16b,v0.16b,v0.16b		// offload
	orr	v19.16b,v1.16b,v1.16b
	ld1	{v17.4s},[x3],#16
	add	v16.4s,v16.4s,v4.4s
.long	0x5e2828a4	//sha256su0 v4.16b,v5.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e104020	//sha256h v0.16b,v1.16b,v16.4s
.long	0x5e105041	//sha256h2 v1.16b,v2.16b,v16.4s
.long	0x5e0760c4	//sha256su1 v4.16b,v6.16b,v7.16b
	ld1	{v16.4s},[x3],#16
	add	v17.4s,v17.4s,v5.4s
.long	0x5e2828c5	//sha256su0 v5.16b,v6.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e114020	//sha256h v0.16b,v1.16b,v17.4s
.long	0x5e115041	//sha256h2 v1.16b,v2.16b,v17.4s
.long	0x5e0460e5	//sha256su1 v5.16b,v7.16b,v4.16b
	ld1	{v17.4s},[x3],#16
	add	v16.4s,v16.4s,v6.4s
.long	0x5e2828e6	//sha256su0 v6.16b,v7.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e104020	//sha256h v0.16b,v1.16b,v16.4s
.long	0x5e105041	//sha256h2 v1.16b,v2.16b,v16.4s
.long	0x5e056086	//sha256su1 v6.16b,v4.16b,v5.16b
	ld1	{v16.4s},[x3],#16
	add	v17.4s,v17.4s,v7.4s
.long	0x5e282887	//sha256su0 v7.16b,v4.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e114020	//sha256h v0.16b,v1.16b,v17.4s
.long	0x5e115041	//sha256h2 v1.16b,v2.16b,v17.4s
.long	0x5e0660a7	//sha256su1 v7.16b,v5.16b,v6.16b
	ld1	{v17.4s},[x3],#16
	add	v16.4s,v16.4s,v4.4s
.long	0x5e2828a4	//sha256su0 v4.16b,v5.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e104020	//sha256h v0.16b,v1.16b,v16.4s
.long	0x5e105041	//sha256h2 v1.16b,v2.16b,v16.4s
.long	0x5e0760c4	//sha256su1 v4.16b,v6.16b,v7.16b
	ld1	{v16.4s},[x3],#16
	add	v17.4s,v17.4s,v5.4s
.long	0x5e2828c5	//sha256su0 v5.16b,v6.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e114020	//sha256h v0.16b,v1.16b,v17.4s
.long	0x5e115041	//sha256h2 v1.16b,v2.16b,v17.4s
.long	0x5e0460e5	//sha256su1 v5.16b,v7.16b,v4.16b
	ld1	{v17.4s},[x3],#16
	add	v16.4s,v16.4s,v6.4s
.long	0x5e2828e6	//sha256su0 v6.16b,v7.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e104020	//sha256h v0.16b,v1.16b,v16.4s
.long	0x5e105041	//sha256h2 v1.16b,v2.16b,v16.4s
.long	0x5e056086	//sha256su1 v6.16b,v4.16b,v5.16b
	ld1	{v16.4s},[x3],#16
	add	v17.4s,v17.4s,v7.4s
.long	0x5e282887	//sha256su0 v7.16b,v4.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e114020	//sha256h v0.16b,v1.16b,v17.4s
.long	0x5e115041	//sha256h2 v1.16b,v2.16b,v17.4s
.long	0x5e0660a7	//sha256su1 v7.16b,v5.16b,v6.16b
	ld1	{v17.4s},[x3],#16
	add	v16.4s,v16.4s,v4.4s
.long	0x5e2828a4	//sha256su0 v4.16b,v5.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e104020	//sha256h v0.16b,v1.16b,v16.4s
.long	0x5e105041	//sha256h2 v1.16b,v2.16b,v16.4s
.long	0x5e0760c4	//sha256su1 v4.16b,v6.16b,v7.16b
	ld1	{v16.4s},[x3],#16
	add	v17.4s,v17.4s,v5.4s
.long	0x5e2828c5	//sha256su0 v5.16b,v6.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e114020	//sha256h v0.16b,v1.16b,v17.4s
.long	0x5e115041	//sha256h2 v1.16b,v2.16b,v17.4s
.long	0x5e0460e5	//sha256su1 v5.16b,v7.16b,v4.16b
	ld1	{v17.4s},[x3],#16
	add	v16.4s,v16.4s,v6.4s
.long	0x5e2828e6	//sha256su0 v6.16b,v7.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e104020	//sha256h v0.16b,v1.16b,v16.4s
.long	0x5e105041	//sha256h2 v1.16b,v2.16b,v16.4s
.long	0x5e056086	//sha256su1 v6.16b,v4.16b,v5.16b
	ld1	{v16.4s},[x3],#16
	add	v17.4s,v17.4s,v7.4s
.long	0x5e282887	//sha256su0 v7.16b,v4.16b
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e114020	//sha256h v0.16b,v1.16b,v17.4s
.long	0x5e115041	//sha256h2 v1.16b,v2.16b,v17.4s
.long	0x5e0660a7	//sha256su1 v7.16b,v5.16b,v6.16b
	ld1	{v17.4s},[x3],#16
	add	v16.4s,v16.4s,v4.4s
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e104020	//sha256h v0.16b,v1.16b,v16.4s
.long	0x5e105041	//sha256h2 v1.16b,v2.16b,v16.4s

	ld1	{v16.4s},[x3],#16
	add	v17.4s,v17.4s,v5.4s
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e114020	//sha256h v0.16b,v1.16b,v17.4s
.long	0x5e115041	//sha256h2 v1.16b,v2.16b,v17.4s

	ld1	{v17.4s},[x3]
	add	v16.4s,v16.4s,v6.4s
	sub	x3,x3,#64*4-16	// rewind
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e104020	//sha256h v0.16b,v1.16b,v16.4s
.long	0x5e105041	//sha256h2 v1.16b,v2.16b,v16.4s

	add	v17.4s,v17.4s,v7.4s
	orr	v2.16b,v0.16b,v0.16b
.long	0x5e114020	//sha256h v0.16b,v1.16b,v17.4s
.long	0x5e115041	//sha256h2 v1.16b,v2.16b,v17.4s

	add	v0.4s,v0.4s,v18.4s
	add	v1.4s,v1.4s,v19.4s

	cbnz	x2,Loop_hw

	st1	{v0.4s,v1.4s},[x0]

	ldr	x29,[sp],#16
	ret

.globl	_blst_sha256_block_data_order

.align	4
_blst_sha256_block_data_order:
	stp	x29, x30, [sp, #-16]!
	mov	x29, sp
	sub	sp,sp,#16*4

	adr	x16,LK256
	add	x2,x1,x2,lsl#6	// len to point at the end of inp

	ld1	{v0.16b},[x1], #16
	ld1	{v1.16b},[x1], #16
	ld1	{v2.16b},[x1], #16
	ld1	{v3.16b},[x1], #16
	ld1	{v4.4s},[x16], #16
	ld1	{v5.4s},[x16], #16
	ld1	{v6.4s},[x16], #16
	ld1	{v7.4s},[x16], #16
	rev32	v0.16b,v0.16b		// yes, even on
	rev32	v1.16b,v1.16b		// big-endian
	rev32	v2.16b,v2.16b
	rev32	v3.16b,v3.16b
	mov	x17,sp
	add	v4.4s,v4.4s,v0.4s
	add	v5.4s,v5.4s,v1.4s
	add	v6.4s,v6.4s,v2.4s
	st1	{v4.4s,v5.4s},[x17], #32
	add	v7.4s,v7.4s,v3.4s
	st1	{v6.4s,v7.4s},[x17]
	sub	x17,x17,#32

	ldp	w3,w4,[x0]
	ldp	w5,w6,[x0,#8]
	ldp	w7,w8,[x0,#16]
	ldp	w9,w10,[x0,#24]
	ldr	w12,[sp,#0]
	mov	w13,wzr
	eor	w14,w4,w5
	mov	w15,wzr
	b	L_00_48

.align	4
L_00_48:
	ext	v4.16b,v0.16b,v1.16b,#4
	add	w10,w10,w12
	add	w3,w3,w15
	and	w12,w8,w7
	bic	w15,w9,w7
	ext	v7.16b,v2.16b,v3.16b,#4
	eor	w11,w7,w7,ror#5
	add	w3,w3,w13
	mov	d19,v3.d[1]
	orr	w12,w12,w15
	eor	w11,w11,w7,ror#19
	ushr	v6.4s,v4.4s,#7
	eor	w15,w3,w3,ror#11
	ushr	v5.4s,v4.4s,#3
	add	w10,w10,w12
	add	v0.4s,v0.4s,v7.4s
	ror	w11,w11,#6
	sli	v6.4s,v4.4s,#25
	eor	w13,w3,w4
	eor	w15,w15,w3,ror#20
	ushr	v7.4s,v4.4s,#18
	add	w10,w10,w11
	ldr	w12,[sp,#4]
	and	w14,w14,w13
	eor	v5.16b,v5.16b,v6.16b
	ror	w15,w15,#2
	add	w6,w6,w10
	sli	v7.4s,v4.4s,#14
	eor	w14,w14,w4
	ushr	v16.4s,v19.4s,#17
	add	w9,w9,w12
	add	w10,w10,w15
	and	w12,w7,w6
	eor	v5.16b,v5.16b,v7.16b
	bic	w15,w8,w6
	eor	w11,w6,w6,ror#5
	sli	v16.4s,v19.4s,#15
	add	w10,w10,w14
	orr	w12,w12,w15
	ushr	v17.4s,v19.4s,#10
	eor	w11,w11,w6,ror#19
	eor	w15,w10,w10,ror#11
	ushr	v7.4s,v19.4s,#19
	add	w9,w9,w12
	ror	w11,w11,#6
	add	v0.4s,v0.4s,v5.4s
	eor	w14,w10,w3
	eor	w15,w15,w10,ror#20
	sli	v7.4s,v19.4s,#13
	add	w9,w9,w11
	ldr	w12,[sp,#8]
	and	w13,w13,w14
	eor	v17.16b,v17.16b,v16.16b
	ror	w15,w15,#2
	add	w5,w5,w9
	eor	w13,w13,w3
	eor	v17.16b,v17.16b,v7.16b
	add	w8,w8,w12
	add	w9,w9,w15
	and	w12,w6,w5
	add	v0.4s,v0.4s,v17.4s
	bic	w15,w7,w5
	eor	w11,w5,w5,ror#5
	add	w9,w9,w13
	ushr	v18.4s,v0.4s,#17
	orr	w12,w12,w15
	ushr	v19.4s,v0.4s,#10
	eor	w11,w11,w5,ror#19
	eor	w15,w9,w9,ror#11
	sli	v18.4s,v0.4s,#15
	add	w8,w8,w12
	ushr	v17.4s,v0.4s,#19
	ror	w11,w11,#6
	eor	w13,w9,w10
	eor	v19.16b,v19.16b,v18.16b
	eor	w15,w15,w9,ror#20
	add	w8,w8,w11
	sli	v17.4s,v0.4s,#13
	ldr	w12,[sp,#12]
	and	w14,w14,w13
	ror	w15,w15,#2
	ld1	{v4.4s},[x16], #16
	add	w4,w4,w8
	eor	v19.16b,v19.16b,v17.16b
	eor	w14,w14,w10
	eor	v17.16b,v17.16b,v17.16b
	add	w7,w7,w12
	add	w8,w8,w15
	and	w12,w5,w4
	mov	v17.d[1],v19.d[0]
	bic	w15,w6,w4
	eor	w11,w4,w4,ror#5
	add	w8,w8,w14
	add	v0.4s,v0.4s,v17.4s
	orr	w12,w12,w15
	eor	w11,w11,w4,ror#19
	eor	w15,w8,w8,ror#11
	add	v4.4s,v4.4s,v0.4s
	add	w7,w7,w12
	ror	w11,w11,#6
	eor	w14,w8,w9
	eor	w15,w15,w8,ror#20
	add	w7,w7,w11
	ldr	w12,[sp,#16]
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w3,w3,w7
	eor	w13,w13,w9
	st1	{v4.4s},[x17], #16
	ext	v4.16b,v1.16b,v2.16b,#4
	add	w6,w6,w12
	add	w7,w7,w15
	and	w12,w4,w3
	bic	w15,w5,w3
	ext	v7.16b,v3.16b,v0.16b,#4
	eor	w11,w3,w3,ror#5
	add	w7,w7,w13
	mov	d19,v0.d[1]
	orr	w12,w12,w15
	eor	w11,w11,w3,ror#19
	ushr	v6.4s,v4.4s,#7
	eor	w15,w7,w7,ror#11
	ushr	v5.4s,v4.4s,#3
	add	w6,w6,w12
	add	v1.4s,v1.4s,v7.4s
	ror	w11,w11,#6
	sli	v6.4s,v4.4s,#25
	eor	w13,w7,w8
	eor	w15,w15,w7,ror#20
	ushr	v7.4s,v4.4s,#18
	add	w6,w6,w11
	ldr	w12,[sp,#20]
	and	w14,w14,w13
	eor	v5.16b,v5.16b,v6.16b
	ror	w15,w15,#2
	add	w10,w10,w6
	sli	v7.4s,v4.4s,#14
	eor	w14,w14,w8
	ushr	v16.4s,v19.4s,#17
	add	w5,w5,w12
	add	w6,w6,w15
	and	w12,w3,w10
	eor	v5.16b,v5.16b,v7.16b
	bic	w15,w4,w10
	eor	w11,w10,w10,ror#5
	sli	v16.4s,v19.4s,#15
	add	w6,w6,w14
	orr	w12,w12,w15
	ushr	v17.4s,v19.4s,#10
	eor	w11,w11,w10,ror#19
	eor	w15,w6,w6,ror#11
	ushr	v7.4s,v19.4s,#19
	add	w5,w5,w12
	ror	w11,w11,#6
	add	v1.4s,v1.4s,v5.4s
	eor	w14,w6,w7
	eor	w15,w15,w6,ror#20
	sli	v7.4s,v19.4s,#13
	add	w5,w5,w11
	ldr	w12,[sp,#24]
	and	w13,w13,w14
	eor	v17.16b,v17.16b,v16.16b
	ror	w15,w15,#2
	add	w9,w9,w5
	eor	w13,w13,w7
	eor	v17.16b,v17.16b,v7.16b
	add	w4,w4,w12
	add	w5,w5,w15
	and	w12,w10,w9
	add	v1.4s,v1.4s,v17.4s
	bic	w15,w3,w9
	eor	w11,w9,w9,ror#5
	add	w5,w5,w13
	ushr	v18.4s,v1.4s,#17
	orr	w12,w12,w15
	ushr	v19.4s,v1.4s,#10
	eor	w11,w11,w9,ror#19
	eor	w15,w5,w5,ror#11
	sli	v18.4s,v1.4s,#15
	add	w4,w4,w12
	ushr	v17.4s,v1.4s,#19
	ror	w11,w11,#6
	eor	w13,w5,w6
	eor	v19.16b,v19.16b,v18.16b
	eor	w15,w15,w5,ror#20
	add	w4,w4,w11
	sli	v17.4s,v1.4s,#13
	ldr	w12,[sp,#28]
	and	w14,w14,w13
	ror	w15,w15,#2
	ld1	{v4.4s},[x16], #16
	add	w8,w8,w4
	eor	v19.16b,v19.16b,v17.16b
	eor	w14,w14,w6
	eor	v17.16b,v17.16b,v17.16b
	add	w3,w3,w12
	add	w4,w4,w15
	and	w12,w9,w8
	mov	v17.d[1],v19.d[0]
	bic	w15,w10,w8
	eor	w11,w8,w8,ror#5
	add	w4,w4,w14
	add	v1.4s,v1.4s,v17.4s
	orr	w12,w12,w15
	eor	w11,w11,w8,ror#19
	eor	w15,w4,w4,ror#11
	add	v4.4s,v4.4s,v1.4s
	add	w3,w3,w12
	ror	w11,w11,#6
	eor	w14,w4,w5
	eor	w15,w15,w4,ror#20
	add	w3,w3,w11
	ldr	w12,[sp,#32]
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w7,w7,w3
	eor	w13,w13,w5
	st1	{v4.4s},[x17], #16
	ext	v4.16b,v2.16b,v3.16b,#4
	add	w10,w10,w12
	add	w3,w3,w15
	and	w12,w8,w7
	bic	w15,w9,w7
	ext	v7.16b,v0.16b,v1.16b,#4
	eor	w11,w7,w7,ror#5
	add	w3,w3,w13
	mov	d19,v1.d[1]
	orr	w12,w12,w15
	eor	w11,w11,w7,ror#19
	ushr	v6.4s,v4.4s,#7
	eor	w15,w3,w3,ror#11
	ushr	v5.4s,v4.4s,#3
	add	w10,w10,w12
	add	v2.4s,v2.4s,v7.4s
	ror	w11,w11,#6
	sli	v6.4s,v4.4s,#25
	eor	w13,w3,w4
	eor	w15,w15,w3,ror#20
	ushr	v7.4s,v4.4s,#18
	add	w10,w10,w11
	ldr	w12,[sp,#36]
	and	w14,w14,w13
	eor	v5.16b,v5.16b,v6.16b
	ror	w15,w15,#2
	add	w6,w6,w10
	sli	v7.4s,v4.4s,#14
	eor	w14,w14,w4
	ushr	v16.4s,v19.4s,#17
	add	w9,w9,w12
	add	w10,w10,w15
	and	w12,w7,w6
	eor	v5.16b,v5.16b,v7.16b
	bic	w15,w8,w6
	eor	w11,w6,w6,ror#5
	sli	v16.4s,v19.4s,#15
	add	w10,w10,w14
	orr	w12,w12,w15
	ushr	v17.4s,v19.4s,#10
	eor	w11,w11,w6,ror#19
	eor	w15,w10,w10,ror#11
	ushr	v7.4s,v19.4s,#19
	add	w9,w9,w12
	ror	w11,w11,#6
	add	v2.4s,v2.4s,v5.4s
	eor	w14,w10,w3
	eor	w15,w15,w10,ror#20
	sli	v7.4s,v19.4s,#13
	add	w9,w9,w11
	ldr	w12,[sp,#40]
	and	w13,w13,w14
	eor	v17.16b,v17.16b,v16.16b
	ror	w15,w15,#2
	add	w5,w5,w9
	eor	w13,w13,w3
	eor	v17.16b,v17.16b,v7.16b
	add	w8,w8,w12
	add	w9,w9,w15
	and	w12,w6,w5
	add	v2.4s,v2.4s,v17.4s
	bic	w15,w7,w5
	eor	w11,w5,w5,ror#5
	add	w9,w9,w13
	ushr	v18.4s,v2.4s,#17
	orr	w12,w12,w15
	ushr	v19.4s,v2.4s,#10
	eor	w11,w11,w5,ror#19
	eor	w15,w9,w9,ror#11
	sli	v18.4s,v2.4s,#15
	add	w8,w8,w12
	ushr	v17.4s,v2.4s,#19
	ror	w11,w11,#6
	eor	w13,w9,w10
	eor	v19.16b,v19.16b,v18.16b
	eor	w15,w15,w9,ror#20
	add	w8,w8,w11
	sli	v17.4s,v2.4s,#13
	ldr	w12,[sp,#44]
	and	w14,w14,w13
	ror	w15,w15,#2
	ld1	{v4.4s},[x16], #16
	add	w4,w4,w8
	eor	v19.16b,v19.16b,v17.16b
	eor	w14,w14,w10
	eor	v17.16b,v17.16b,v17.16b
	add	w7,w7,w12
	add	w8,w8,w15
	and	w12,w5,w4
	mov	v17.d[1],v19.d[0]
	bic	w15,w6,w4
	eor	w11,w4,w4,ror#5
	add	w8,w8,w14
	add	v2.4s,v2.4s,v17.4s
	orr	w12,w12,w15
	eor	w11,w11,w4,ror#19
	eor	w15,w8,w8,ror#11
	add	v4.4s,v4.4s,v2.4s
	add	w7,w7,w12
	ror	w11,w11,#6
	eor	w14,w8,w9
	eor	w15,w15,w8,ror#20
	add	w7,w7,w11
	ldr	w12,[sp,#48]
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w3,w3,w7
	eor	w13,w13,w9
	st1	{v4.4s},[x17], #16
	ext	v4.16b,v3.16b,v0.16b,#4
	add	w6,w6,w12
	add	w7,w7,w15
	and	w12,w4,w3
	bic	w15,w5,w3
	ext	v7.16b,v1.16b,v2.16b,#4
	eor	w11,w3,w3,ror#5
	add	w7,w7,w13
	mov	d19,v2.d[1]
	orr	w12,w12,w15
	eor	w11,w11,w3,ror#19
	ushr	v6.4s,v4.4s,#7
	eor	w15,w7,w7,ror#11
	ushr	v5.4s,v4.4s,#3
	add	w6,w6,w12
	add	v3.4s,v3.4s,v7.4s
	ror	w11,w11,#6
	sli	v6.4s,v4.4s,#25
	eor	w13,w7,w8
	eor	w15,w15,w7,ror#20
	ushr	v7.4s,v4.4s,#18
	add	w6,w6,w11
	ldr	w12,[sp,#52]
	and	w14,w14,w13
	eor	v5.16b,v5.16b,v6.16b
	ror	w15,w15,#2
	add	w10,w10,w6
	sli	v7.4s,v4.4s,#14
	eor	w14,w14,w8
	ushr	v16.4s,v19.4s,#17
	add	w5,w5,w12
	add	w6,w6,w15
	and	w12,w3,w10
	eor	v5.16b,v5.16b,v7.16b
	bic	w15,w4,w10
	eor	w11,w10,w10,ror#5
	sli	v16.4s,v19.4s,#15
	add	w6,w6,w14
	orr	w12,w12,w15
	ushr	v17.4s,v19.4s,#10
	eor	w11,w11,w10,ror#19
	eor	w15,w6,w6,ror#11
	ushr	v7.4s,v19.4s,#19
	add	w5,w5,w12
	ror	w11,w11,#6
	add	v3.4s,v3.4s,v5.4s
	eor	w14,w6,w7
	eor	w15,w15,w6,ror#20
	sli	v7.4s,v19.4s,#13
	add	w5,w5,w11
	ldr	w12,[sp,#56]
	and	w13,w13,w14
	eor	v17.16b,v17.16b,v16.16b
	ror	w15,w15,#2
	add	w9,w9,w5
	eor	w13,w13,w7
	eor	v17.16b,v17.16b,v7.16b
	add	w4,w4,w12
	add	w5,w5,w15
	and	w12,w10,w9
	add	v3.4s,v3.4s,v17.4s
	bic	w15,w3,w9
	eor	w11,w9,w9,ror#5
	add	w5,w5,w13
	ushr	v18.4s,v3.4s,#17
	orr	w12,w12,w15
	ushr	v19.4s,v3.4s,#10
	eor	w11,w11,w9,ror#19
	eor	w15,w5,w5,ror#11
	sli	v18.4s,v3.4s,#15
	add	w4,w4,w12
	ushr	v17.4s,v3.4s,#19
	ror	w11,w11,#6
	eor	w13,w5,w6
	eor	v19.16b,v19.16b,v18.16b
	eor	w15,w15,w5,ror#20
	add	w4,w4,w11
	sli	v17.4s,v3.4s,#13
	ldr	w12,[sp,#60]
	and	w14,w14,w13
	ror	w15,w15,#2
	ld1	{v4.4s},[x16], #16
	add	w8,w8,w4
	eor	v19.16b,v19.16b,v17.16b
	eor	w14,w14,w6
	eor	v17.16b,v17.16b,v17.16b
	add	w3,w3,w12
	add	w4,w4,w15
	and	w12,w9,w8
	mov	v17.d[1],v19.d[0]
	bic	w15,w10,w8
	eor	w11,w8,w8,ror#5
	add	w4,w4,w14
	add	v3.4s,v3.4s,v17.4s
	orr	w12,w12,w15
	eor	w11,w11,w8,ror#19
	eor	w15,w4,w4,ror#11
	add	v4.4s,v4.4s,v3.4s
	add	w3,w3,w12
	ror	w11,w11,#6
	eor	w14,w4,w5
	eor	w15,w15,w4,ror#20
	add	w3,w3,w11
	ldr	w12,[x16]
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w7,w7,w3
	eor	w13,w13,w5
	st1	{v4.4s},[x17], #16
	cmp	w12,#0				// check for K256 terminator
	ldr	w12,[sp,#0]
	sub	x17,x17,#64
	bne	L_00_48

	sub	x16,x16,#256		// rewind x16
	cmp	x1,x2
	mov	x17, #64
	csel	x17, x17, xzr, eq
	sub	x1,x1,x17			// avoid SEGV
	mov	x17,sp
	add	w10,w10,w12
	add	w3,w3,w15
	and	w12,w8,w7
	ld1	{v0.16b},[x1],#16
	bic	w15,w9,w7
	eor	w11,w7,w7,ror#5
	ld1	{v4.4s},[x16],#16
	add	w3,w3,w13
	orr	w12,w12,w15
	eor	w11,w11,w7,ror#19
	eor	w15,w3,w3,ror#11
	rev32	v0.16b,v0.16b
	add	w10,w10,w12
	ror	w11,w11,#6
	eor	w13,w3,w4
	eor	w15,w15,w3,ror#20
	add	v4.4s,v4.4s,v0.4s
	add	w10,w10,w11
	ldr	w12,[sp,#4]
	and	w14,w14,w13
	ror	w15,w15,#2
	add	w6,w6,w10
	eor	w14,w14,w4
	add	w9,w9,w12
	add	w10,w10,w15
	and	w12,w7,w6
	bic	w15,w8,w6
	eor	w11,w6,w6,ror#5
	add	w10,w10,w14
	orr	w12,w12,w15
	eor	w11,w11,w6,ror#19
	eor	w15,w10,w10,ror#11
	add	w9,w9,w12
	ror	w11,w11,#6
	eor	w14,w10,w3
	eor	w15,w15,w10,ror#20
	add	w9,w9,w11
	ldr	w12,[sp,#8]
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w5,w5,w9
	eor	w13,w13,w3
	add	w8,w8,w12
	add	w9,w9,w15
	and	w12,w6,w5
	bic	w15,w7,w5
	eor	w11,w5,w5,ror#5
	add	w9,w9,w13
	orr	w12,w12,w15
	eor	w11,w11,w5,ror#19
	eor	w15,w9,w9,ror#11
	add	w8,w8,w12
	ror	w11,w11,#6
	eor	w13,w9,w10
	eor	w15,w15,w9,ror#20
	add	w8,w8,w11
	ldr	w12,[sp,#12]
	and	w14,w14,w13
	ror	w15,w15,#2
	add	w4,w4,w8
	eor	w14,w14,w10
	add	w7,w7,w12
	add	w8,w8,w15
	and	w12,w5,w4
	bic	w15,w6,w4
	eor	w11,w4,w4,ror#5
	add	w8,w8,w14
	orr	w12,w12,w15
	eor	w11,w11,w4,ror#19
	eor	w15,w8,w8,ror#11
	add	w7,w7,w12
	ror	w11,w11,#6
	eor	w14,w8,w9
	eor	w15,w15,w8,ror#20
	add	w7,w7,w11
	ldr	w12,[sp,#16]
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w3,w3,w7
	eor	w13,w13,w9
	st1	{v4.4s},[x17], #16
	add	w6,w6,w12
	add	w7,w7,w15
	and	w12,w4,w3
	ld1	{v1.16b},[x1],#16
	bic	w15,w5,w3
	eor	w11,w3,w3,ror#5
	ld1	{v4.4s},[x16],#16
	add	w7,w7,w13
	orr	w12,w12,w15
	eor	w11,w11,w3,ror#19
	eor	w15,w7,w7,ror#11
	rev32	v1.16b,v1.16b
	add	w6,w6,w12
	ror	w11,w11,#6
	eor	w13,w7,w8
	eor	w15,w15,w7,ror#20
	add	v4.4s,v4.4s,v1.4s
	add	w6,w6,w11
	ldr	w12,[sp,#20]
	and	w14,w14,w13
	ror	w15,w15,#2
	add	w10,w10,w6
	eor	w14,w14,w8
	add	w5,w5,w12
	add	w6,w6,w15
	and	w12,w3,w10
	bic	w15,w4,w10
	eor	w11,w10,w10,ror#5
	add	w6,w6,w14
	orr	w12,w12,w15
	eor	w11,w11,w10,ror#19
	eor	w15,w6,w6,ror#11
	add	w5,w5,w12
	ror	w11,w11,#6
	eor	w14,w6,w7
	eor	w15,w15,w6,ror#20
	add	w5,w5,w11
	ldr	w12,[sp,#24]
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w9,w9,w5
	eor	w13,w13,w7
	add	w4,w4,w12
	add	w5,w5,w15
	and	w12,w10,w9
	bic	w15,w3,w9
	eor	w11,w9,w9,ror#5
	add	w5,w5,w13
	orr	w12,w12,w15
	eor	w11,w11,w9,ror#19
	eor	w15,w5,w5,ror#11
	add	w4,w4,w12
	ror	w11,w11,#6
	eor	w13,w5,w6
	eor	w15,w15,w5,ror#20
	add	w4,w4,w11
	ldr	w12,[sp,#28]
	and	w14,w14,w13
	ror	w15,w15,#2
	add	w8,w8,w4
	eor	w14,w14,w6
	add	w3,w3,w12
	add	w4,w4,w15
	and	w12,w9,w8
	bic	w15,w10,w8
	eor	w11,w8,w8,ror#5
	add	w4,w4,w14
	orr	w12,w12,w15
	eor	w11,w11,w8,ror#19
	eor	w15,w4,w4,ror#11
	add	w3,w3,w12
	ror	w11,w11,#6
	eor	w14,w4,w5
	eor	w15,w15,w4,ror#20
	add	w3,w3,w11
	ldr	w12,[sp,#32]
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w7,w7,w3
	eor	w13,w13,w5
	st1	{v4.4s},[x17], #16
	add	w10,w10,w12
	add	w3,w3,w15
	and	w12,w8,w7
	ld1	{v2.16b},[x1],#16
	bic	w15,w9,w7
	eor	w11,w7,w7,ror#5
	ld1	{v4.4s},[x16],#16
	add	w3,w3,w13
	orr	w12,w12,w15
	eor	w11,w11,w7,ror#19
	eor	w15,w3,w3,ror#11
	rev32	v2.16b,v2.16b
	add	w10,w10,w12
	ror	w11,w11,#6
	eor	w13,w3,w4
	eor	w15,w15,w3,ror#20
	add	v4.4s,v4.4s,v2.4s
	add	w10,w10,w11
	ldr	w12,[sp,#36]
	and	w14,w14,w13
	ror	w15,w15,#2
	add	w6,w6,w10
	eor	w14,w14,w4
	add	w9,w9,w12
	add	w10,w10,w15
	and	w12,w7,w6
	bic	w15,w8,w6
	eor	w11,w6,w6,ror#5
	add	w10,w10,w14
	orr	w12,w12,w15
	eor	w11,w11,w6,ror#19
	eor	w15,w10,w10,ror#11
	add	w9,w9,w12
	ror	w11,w11,#6
	eor	w14,w10,w3
	eor	w15,w15,w10,ror#20
	add	w9,w9,w11
	ldr	w12,[sp,#40]
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w5,w5,w9
	eor	w13,w13,w3
	add	w8,w8,w12
	add	w9,w9,w15
	and	w12,w6,w5
	bic	w15,w7,w5
	eor	w11,w5,w5,ror#5
	add	w9,w9,w13
	orr	w12,w12,w15
	eor	w11,w11,w5,ror#19
	eor	w15,w9,w9,ror#11
	add	w8,w8,w12
	ror	w11,w11,#6
	eor	w13,w9,w10
	eor	w15,w15,w9,ror#20
	add	w8,w8,w11
	ldr	w12,[sp,#44]
	and	w14,w14,w13
	ror	w15,w15,#2
	add	w4,w4,w8
	eor	w14,w14,w10
	add	w7,w7,w12
	add	w8,w8,w15
	and	w12,w5,w4
	bic	w15,w6,w4
	eor	w11,w4,w4,ror#5
	add	w8,w8,w14
	orr	w12,w12,w15
	eor	w11,w11,w4,ror#19
	eor	w15,w8,w8,ror#11
	add	w7,w7,w12
	ror	w11,w11,#6
	eor	w14,w8,w9
	eor	w15,w15,w8,ror#20
	add	w7,w7,w11
	ldr	w12,[sp,#48]
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w3,w3,w7
	eor	w13,w13,w9
	st1	{v4.4s},[x17], #16
	add	w6,w6,w12
	add	w7,w7,w15
	and	w12,w4,w3
	ld1	{v3.16b},[x1],#16
	bic	w15,w5,w3
	eor	w11,w3,w3,ror#5
	ld1	{v4.4s},[x16],#16
	add	w7,w7,w13
	orr	w12,w12,w15
	eor	w11,w11,w3,ror#19
	eor	w15,w7,w7,ror#11
	rev32	v3.16b,v3.16b
	add	w6,w6,w12
	ror	w11,w11,#6
	eor	w13,w7,w8
	eor	w15,w15,w7,ror#20
	add	v4.4s,v4.4s,v3.4s
	add	w6,w6,w11
	ldr	w12,[sp,#52]
	and	w14,w14,w13
	ror	w15,w15,#2
	add	w10,w10,w6
	eor	w14,w14,w8
	add	w5,w5,w12
	add	w6,w6,w15
	and	w12,w3,w10
	bic	w15,w4,w10
	eor	w11,w10,w10,ror#5
	add	w6,w6,w14
	orr	w12,w12,w15
	eor	w11,w11,w10,ror#19
	eor	w15,w6,w6,ror#11
	add	w5,w5,w12
	ror	w11,w11,#6
	eor	w14,w6,w7
	eor	w15,w15,w6,ror#20
	add	w5,w5,w11
	ldr	w12,[sp,#56]
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w9,w9,w5
	eor	w13,w13,w7
	add	w4,w4,w12
	add	w5,w5,w15
	and	w12,w10,w9
	bic	w15,w3,w9
	eor	w11,w9,w9,ror#5
	add	w5,w5,w13
	orr	w12,w12,w15
	eor	w11,w11,w9,ror#19
	eor	w15,w5,w5,ror#11
	add	w4,w4,w12
	ror	w11,w11,#6
	eor	w13,w5,w6
	eor	w15,w15,w5,ror#20
	add	w4,w4,w11
	ldr	w12,[sp,#60]
	and	w14,w14,w13
	ror	w15,w15,#2
	add	w8,w8,w4
	eor	w14,w14,w6
	add	w3,w3,w12
	add	w4,w4,w15
	and	w12,w9,w8
	bic	w15,w10,w8
	eor	w11,w8,w8,ror#5
	add	w4,w4,w14
	orr	w12,w12,w15
	eor	w11,w11,w8,ror#19
	eor	w15,w4,w4,ror#11
	add	w3,w3,w12
	ror	w11,w11,#6
	eor	w14,w4,w5
	eor	w15,w15,w4,ror#20
	add	w3,w3,w11
	and	w13,w13,w14
	ror	w15,w15,#2
	add	w7,w7,w3
	eor	w13,w13,w5
	st1	{v4.4s},[x17], #16
	add	w3,w3,w15			// h+=Sigma0(a) from the past
	ldp	w11,w12,[x0,#0]
	add	w3,w3,w13			// h+=Maj(a,b,c) from the past
	ldp	w13,w14,[x0,#8]
	add	w3,w3,w11			// accumulate
	add	w4,w4,w12
	ldp	w11,w12,[x0,#16]
	add	w5,w5,w13
	add	w6,w6,w14
	ldp	w13,w14,[x0,#24]
	add	w7,w7,w11
	add	w8,w8,w12
	ldr	w12,[sp,#0]
	stp	w3,w4,[x0,#0]
	add	w9,w9,w13
	mov	w13,wzr
	stp	w5,w6,[x0,#8]
	add	w10,w10,w14
	stp	w7,w8,[x0,#16]
	eor	w14,w4,w5
	stp	w9,w10,[x0,#24]
	mov	w15,wzr
	mov	x17,sp
	b.ne	L_00_48

	ldr	x29,[x29]
	add	sp,sp,#16*4+16
	ret

.globl	_blst_sha256_emit
.private_extern	_blst_sha256_emit

.align	4
_blst_sha256_emit:
	ldp	x4,x5,[x1]
	ldp	x6,x7,[x1,#16]
#ifndef	__AARCH64EB__
	rev	x4,x4
	rev	x5,x5
	rev	x6,x6
	rev	x7,x7
#endif
	str	w4,[x0,#4]
	lsr	x4,x4,#32
	str	w5,[x0,#12]
	lsr	x5,x5,#32
	str	w6,[x0,#20]
	lsr	x6,x6,#32
	str	w7,[x0,#28]
	lsr	x7,x7,#32
	str	w4,[x0,#0]
	str	w5,[x0,#8]
	str	w6,[x0,#16]
	str	w7,[x0,#24]
	ret


.globl	_blst_sha256_bcopy
.private_extern	_blst_sha256_bcopy

.align	4
_blst_sha256_bcopy:
Loop_bcopy:
	ldrb	w3,[x1],#1
	sub	x2,x2,#1
	strb	w3,[x0],#1
	cbnz	x2,Loop_bcopy
	ret


.globl	_blst_sha256_hcopy
.private_extern	_blst_sha256_hcopy

.align	4
_blst_sha256_hcopy:
	ldp	x4,x5,[x1]
	ldp	x6,x7,[x1,#16]
	stp	x4,x5,[x0]
	stp	x6,x7,[x0,#16]
	ret

