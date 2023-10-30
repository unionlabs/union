// This file is generated from a similarly-named Perl script in the BoringSSL
// source tree. Do not edit by hand.

#if !defined(__has_feature)
#define __has_feature(x) 0
#endif
#if __has_feature(memory_sanitizer) && !defined(OPENSSL_NO_ASM)
#define OPENSSL_NO_ASM
#endif

#if !defined(OPENSSL_NO_ASM)
#include <GFp/arm_arch.h>

#if __ARM_MAX_ARCH__>=7
.text


.code	32
#undef	__thumb2__
.align	5
Lrcon:
.long	0x01,0x01,0x01,0x01
.long	0x0c0f0e0d,0x0c0f0e0d,0x0c0f0e0d,0x0c0f0e0d	@ rotate-n-splat
.long	0x1b,0x1b,0x1b,0x1b

.text

.globl	_GFp_aes_hw_set_encrypt_key
.private_extern	_GFp_aes_hw_set_encrypt_key
#ifdef __thumb2__
.thumb_func	_GFp_aes_hw_set_encrypt_key
#endif
.align	5
_GFp_aes_hw_set_encrypt_key:
Lenc_key:
	mov	r3,#-1
	cmp	r0,#0
	beq	Lenc_key_abort
	cmp	r2,#0
	beq	Lenc_key_abort
	mov	r3,#-2
	cmp	r1,#128
	blt	Lenc_key_abort
	cmp	r1,#256
	bgt	Lenc_key_abort
	tst	r1,#0x3f
	bne	Lenc_key_abort

	adr	r3,Lrcon
	cmp	r1,#192

	veor	q0,q0,q0
	vld1.8	{q3},[r0]!
	mov	r1,#8		@ reuse r1
	vld1.32	{q1,q2},[r3]!

	blt	Loop128
	@ 192-bit key support was removed.
	b	L256

.align	4
Loop128:
	vtbl.8	d20,{q3},d4
	vtbl.8	d21,{q3},d5
	vext.8	q9,q0,q3,#12
	vst1.32	{q3},[r2]!
.byte	0x00,0x43,0xf0,0xf3	@ aese q10,q0
	subs	r1,r1,#1

	veor	q3,q3,q9
	vext.8	q9,q0,q9,#12
	veor	q3,q3,q9
	vext.8	q9,q0,q9,#12
	veor	q10,q10,q1
	veor	q3,q3,q9
	vshl.u8	q1,q1,#1
	veor	q3,q3,q10
	bne	Loop128

	vld1.32	{q1},[r3]

	vtbl.8	d20,{q3},d4
	vtbl.8	d21,{q3},d5
	vext.8	q9,q0,q3,#12
	vst1.32	{q3},[r2]!
.byte	0x00,0x43,0xf0,0xf3	@ aese q10,q0

	veor	q3,q3,q9
	vext.8	q9,q0,q9,#12
	veor	q3,q3,q9
	vext.8	q9,q0,q9,#12
	veor	q10,q10,q1
	veor	q3,q3,q9
	vshl.u8	q1,q1,#1
	veor	q3,q3,q10

	vtbl.8	d20,{q3},d4
	vtbl.8	d21,{q3},d5
	vext.8	q9,q0,q3,#12
	vst1.32	{q3},[r2]!
.byte	0x00,0x43,0xf0,0xf3	@ aese q10,q0

	veor	q3,q3,q9
	vext.8	q9,q0,q9,#12
	veor	q3,q3,q9
	vext.8	q9,q0,q9,#12
	veor	q10,q10,q1
	veor	q3,q3,q9
	veor	q3,q3,q10
	vst1.32	{q3},[r2]
	add	r2,r2,#0x50

	mov	r12,#10
	b	Ldone

@ 192-bit key support was removed.

.align	4
L256:
	vld1.8	{q8},[r0]
	mov	r1,#7
	mov	r12,#14
	vst1.32	{q3},[r2]!

Loop256:
	vtbl.8	d20,{q8},d4
	vtbl.8	d21,{q8},d5
	vext.8	q9,q0,q3,#12
	vst1.32	{q8},[r2]!
.byte	0x00,0x43,0xf0,0xf3	@ aese q10,q0
	subs	r1,r1,#1

	veor	q3,q3,q9
	vext.8	q9,q0,q9,#12
	veor	q3,q3,q9
	vext.8	q9,q0,q9,#12
	veor	q10,q10,q1
	veor	q3,q3,q9
	vshl.u8	q1,q1,#1
	veor	q3,q3,q10
	vst1.32	{q3},[r2]!
	beq	Ldone

	vdup.32	q10,d7[1]
	vext.8	q9,q0,q8,#12
.byte	0x00,0x43,0xf0,0xf3	@ aese q10,q0

	veor	q8,q8,q9
	vext.8	q9,q0,q9,#12
	veor	q8,q8,q9
	vext.8	q9,q0,q9,#12
	veor	q8,q8,q9

	veor	q8,q8,q10
	b	Loop256

Ldone:
	str	r12,[r2]
	mov	r3,#0

Lenc_key_abort:
	mov	r0,r3			@ return value

	bx	lr

.globl	_GFp_aes_hw_encrypt
.private_extern	_GFp_aes_hw_encrypt
#ifdef __thumb2__
.thumb_func	_GFp_aes_hw_encrypt
#endif
.align	5
_GFp_aes_hw_encrypt:
	AARCH64_VALID_CALL_TARGET
	ldr	r3,[r2,#240]
	vld1.32	{q0},[r2]!
	vld1.8	{q2},[r0]
	sub	r3,r3,#2
	vld1.32	{q1},[r2]!

Loop_enc:
.byte	0x00,0x43,0xb0,0xf3	@ aese q2,q0
.byte	0x84,0x43,0xb0,0xf3	@ aesmc q2,q2
	vld1.32	{q0},[r2]!
	subs	r3,r3,#2
.byte	0x02,0x43,0xb0,0xf3	@ aese q2,q1
.byte	0x84,0x43,0xb0,0xf3	@ aesmc q2,q2
	vld1.32	{q1},[r2]!
	bgt	Loop_enc

.byte	0x00,0x43,0xb0,0xf3	@ aese q2,q0
.byte	0x84,0x43,0xb0,0xf3	@ aesmc q2,q2
	vld1.32	{q0},[r2]
.byte	0x02,0x43,0xb0,0xf3	@ aese q2,q1
	veor	q2,q2,q0

	vst1.8	{q2},[r1]
	bx	lr

.globl	_GFp_aes_hw_decrypt
.private_extern	_GFp_aes_hw_decrypt
#ifdef __thumb2__
.thumb_func	_GFp_aes_hw_decrypt
#endif
.align	5
_GFp_aes_hw_decrypt:
	AARCH64_VALID_CALL_TARGET
	ldr	r3,[r2,#240]
	vld1.32	{q0},[r2]!
	vld1.8	{q2},[r0]
	sub	r3,r3,#2
	vld1.32	{q1},[r2]!

Loop_dec:
.byte	0x40,0x43,0xb0,0xf3	@ aesd q2,q0
.byte	0xc4,0x43,0xb0,0xf3	@ aesimc q2,q2
	vld1.32	{q0},[r2]!
	subs	r3,r3,#2
.byte	0x42,0x43,0xb0,0xf3	@ aesd q2,q1
.byte	0xc4,0x43,0xb0,0xf3	@ aesimc q2,q2
	vld1.32	{q1},[r2]!
	bgt	Loop_dec

.byte	0x40,0x43,0xb0,0xf3	@ aesd q2,q0
.byte	0xc4,0x43,0xb0,0xf3	@ aesimc q2,q2
	vld1.32	{q0},[r2]
.byte	0x42,0x43,0xb0,0xf3	@ aesd q2,q1
	veor	q2,q2,q0

	vst1.8	{q2},[r1]
	bx	lr

.globl	_GFp_aes_hw_ctr32_encrypt_blocks
.private_extern	_GFp_aes_hw_ctr32_encrypt_blocks
#ifdef __thumb2__
.thumb_func	_GFp_aes_hw_ctr32_encrypt_blocks
#endif
.align	5
_GFp_aes_hw_ctr32_encrypt_blocks:
	mov	ip,sp
	stmdb	sp!,{r4,r5,r6,r7,r8,r9,r10,lr}
	vstmdb	sp!,{d8,d9,d10,d11,d12,d13,d14,d15}            @ ABI specification says so
	ldr	r4, [ip]		@ load remaining arg
	ldr	r5,[r3,#240]

	ldr	r8, [r4, #12]
	vld1.32	{q0},[r4]

	vld1.32	{q8,q9},[r3]		@ load key schedule...
	sub	r5,r5,#4
	mov	r12,#16
	cmp	r2,#2
	add	r7,r3,r5,lsl#4	@ pointer to last 5 round keys
	sub	r5,r5,#2
	vld1.32	{q12,q13},[r7]!
	vld1.32	{q14,q15},[r7]!
	vld1.32	{q7},[r7]
	add	r7,r3,#32
	mov	r6,r5
	movlo	r12,#0

	@ ARM Cortex-A57 and Cortex-A72 cores running in 32-bit mode are
	@ affected by silicon errata #1742098 [0] and #1655431 [1],
	@ respectively, where the second instruction of an aese/aesmc
	@ instruction pair may execute twice if an interrupt is taken right
	@ after the first instruction consumes an input register of which a
	@ single 32-bit lane has been updated the last time it was modified.
	@ 
	@ This function uses a counter in one 32-bit lane. The 
	@ could write to q1 and q10 directly, but that trips this bugs.
	@ We write to q6 and copy to the final register as a workaround.
	@ 
	@ [0] ARM-EPM-049219 v23 Cortex-A57 MPCore Software Developers Errata Notice
	@ [1] ARM-EPM-012079 v11.0 Cortex-A72 MPCore Software Developers Errata Notice
#ifndef __ARMEB__
	rev	r8, r8
#endif
	add	r10, r8, #1
	vorr	q6,q0,q0
	rev	r10, r10
	vmov.32	d13[1],r10
	add	r8, r8, #2
	vorr	q1,q6,q6
	bls	Lctr32_tail
	rev	r12, r8
	vmov.32	d13[1],r12
	sub	r2,r2,#3		@ bias
	vorr	q10,q6,q6
	b	Loop3x_ctr32

.align	4
Loop3x_ctr32:
.byte	0x20,0x03,0xb0,0xf3	@ aese q0,q8
.byte	0x80,0x03,0xb0,0xf3	@ aesmc q0,q0
.byte	0x20,0x23,0xb0,0xf3	@ aese q1,q8
.byte	0x82,0x23,0xb0,0xf3	@ aesmc q1,q1
.byte	0x20,0x43,0xf0,0xf3	@ aese q10,q8
.byte	0xa4,0x43,0xf0,0xf3	@ aesmc q10,q10
	vld1.32	{q8},[r7]!
	subs	r6,r6,#2
.byte	0x22,0x03,0xb0,0xf3	@ aese q0,q9
.byte	0x80,0x03,0xb0,0xf3	@ aesmc q0,q0
.byte	0x22,0x23,0xb0,0xf3	@ aese q1,q9
.byte	0x82,0x23,0xb0,0xf3	@ aesmc q1,q1
.byte	0x22,0x43,0xf0,0xf3	@ aese q10,q9
.byte	0xa4,0x43,0xf0,0xf3	@ aesmc q10,q10
	vld1.32	{q9},[r7]!
	bgt	Loop3x_ctr32

.byte	0x20,0x03,0xb0,0xf3	@ aese q0,q8
.byte	0x80,0x83,0xb0,0xf3	@ aesmc q4,q0
.byte	0x20,0x23,0xb0,0xf3	@ aese q1,q8
.byte	0x82,0xa3,0xb0,0xf3	@ aesmc q5,q1
	vld1.8	{q2},[r0]!
	add	r9,r8,#1
.byte	0x20,0x43,0xf0,0xf3	@ aese q10,q8
.byte	0xa4,0x43,0xf0,0xf3	@ aesmc q10,q10
	vld1.8	{q3},[r0]!
	rev	r9,r9
.byte	0x22,0x83,0xb0,0xf3	@ aese q4,q9
.byte	0x88,0x83,0xb0,0xf3	@ aesmc q4,q4
.byte	0x22,0xa3,0xb0,0xf3	@ aese q5,q9
.byte	0x8a,0xa3,0xb0,0xf3	@ aesmc q5,q5
	vld1.8	{q11},[r0]!
	mov	r7,r3
.byte	0x22,0x43,0xf0,0xf3	@ aese q10,q9
.byte	0xa4,0x23,0xf0,0xf3	@ aesmc q9,q10
.byte	0x28,0x83,0xb0,0xf3	@ aese q4,q12
.byte	0x88,0x83,0xb0,0xf3	@ aesmc q4,q4
.byte	0x28,0xa3,0xb0,0xf3	@ aese q5,q12
.byte	0x8a,0xa3,0xb0,0xf3	@ aesmc q5,q5
	veor	q2,q2,q7
	add	r10,r8,#2
.byte	0x28,0x23,0xf0,0xf3	@ aese q9,q12
.byte	0xa2,0x23,0xf0,0xf3	@ aesmc q9,q9
	veor	q3,q3,q7
	add	r8,r8,#3
.byte	0x2a,0x83,0xb0,0xf3	@ aese q4,q13
.byte	0x88,0x83,0xb0,0xf3	@ aesmc q4,q4
.byte	0x2a,0xa3,0xb0,0xf3	@ aese q5,q13
.byte	0x8a,0xa3,0xb0,0xf3	@ aesmc q5,q5
	 @ Note the logic to update q0, q1, and q1 is written to work
	 @ around a bug in ARM Cortex-A57 and Cortex-A72 cores running in
	 @ 32-bit mode. See the comment above.
	veor	q11,q11,q7
	vmov.32	d13[1], r9
.byte	0x2a,0x23,0xf0,0xf3	@ aese q9,q13
.byte	0xa2,0x23,0xf0,0xf3	@ aesmc q9,q9
	vorr	q0,q6,q6
	rev	r10,r10
.byte	0x2c,0x83,0xb0,0xf3	@ aese q4,q14
.byte	0x88,0x83,0xb0,0xf3	@ aesmc q4,q4
	vmov.32	d13[1], r10
	rev	r12,r8
.byte	0x2c,0xa3,0xb0,0xf3	@ aese q5,q14
.byte	0x8a,0xa3,0xb0,0xf3	@ aesmc q5,q5
	vorr	q1,q6,q6
	vmov.32	d13[1], r12
.byte	0x2c,0x23,0xf0,0xf3	@ aese q9,q14
.byte	0xa2,0x23,0xf0,0xf3	@ aesmc q9,q9
	vorr	q10,q6,q6
	subs	r2,r2,#3
.byte	0x2e,0x83,0xb0,0xf3	@ aese q4,q15
.byte	0x2e,0xa3,0xb0,0xf3	@ aese q5,q15
.byte	0x2e,0x23,0xf0,0xf3	@ aese q9,q15

	veor	q2,q2,q4
	vld1.32	{q8},[r7]!	@ re-pre-load rndkey[0]
	vst1.8	{q2},[r1]!
	veor	q3,q3,q5
	mov	r6,r5
	vst1.8	{q3},[r1]!
	veor	q11,q11,q9
	vld1.32	{q9},[r7]!	@ re-pre-load rndkey[1]
	vst1.8	{q11},[r1]!
	bhs	Loop3x_ctr32

	adds	r2,r2,#3
	beq	Lctr32_done
	cmp	r2,#1
	mov	r12,#16
	moveq	r12,#0

Lctr32_tail:
.byte	0x20,0x03,0xb0,0xf3	@ aese q0,q8
.byte	0x80,0x03,0xb0,0xf3	@ aesmc q0,q0
.byte	0x20,0x23,0xb0,0xf3	@ aese q1,q8
.byte	0x82,0x23,0xb0,0xf3	@ aesmc q1,q1
	vld1.32	{q8},[r7]!
	subs	r6,r6,#2
.byte	0x22,0x03,0xb0,0xf3	@ aese q0,q9
.byte	0x80,0x03,0xb0,0xf3	@ aesmc q0,q0
.byte	0x22,0x23,0xb0,0xf3	@ aese q1,q9
.byte	0x82,0x23,0xb0,0xf3	@ aesmc q1,q1
	vld1.32	{q9},[r7]!
	bgt	Lctr32_tail

.byte	0x20,0x03,0xb0,0xf3	@ aese q0,q8
.byte	0x80,0x03,0xb0,0xf3	@ aesmc q0,q0
.byte	0x20,0x23,0xb0,0xf3	@ aese q1,q8
.byte	0x82,0x23,0xb0,0xf3	@ aesmc q1,q1
.byte	0x22,0x03,0xb0,0xf3	@ aese q0,q9
.byte	0x80,0x03,0xb0,0xf3	@ aesmc q0,q0
.byte	0x22,0x23,0xb0,0xf3	@ aese q1,q9
.byte	0x82,0x23,0xb0,0xf3	@ aesmc q1,q1
	vld1.8	{q2},[r0],r12
.byte	0x28,0x03,0xb0,0xf3	@ aese q0,q12
.byte	0x80,0x03,0xb0,0xf3	@ aesmc q0,q0
.byte	0x28,0x23,0xb0,0xf3	@ aese q1,q12
.byte	0x82,0x23,0xb0,0xf3	@ aesmc q1,q1
	vld1.8	{q3},[r0]
.byte	0x2a,0x03,0xb0,0xf3	@ aese q0,q13
.byte	0x80,0x03,0xb0,0xf3	@ aesmc q0,q0
.byte	0x2a,0x23,0xb0,0xf3	@ aese q1,q13
.byte	0x82,0x23,0xb0,0xf3	@ aesmc q1,q1
	veor	q2,q2,q7
.byte	0x2c,0x03,0xb0,0xf3	@ aese q0,q14
.byte	0x80,0x03,0xb0,0xf3	@ aesmc q0,q0
.byte	0x2c,0x23,0xb0,0xf3	@ aese q1,q14
.byte	0x82,0x23,0xb0,0xf3	@ aesmc q1,q1
	veor	q3,q3,q7
.byte	0x2e,0x03,0xb0,0xf3	@ aese q0,q15
.byte	0x2e,0x23,0xb0,0xf3	@ aese q1,q15

	cmp	r2,#1
	veor	q2,q2,q0
	veor	q3,q3,q1
	vst1.8	{q2},[r1]!
	beq	Lctr32_done
	vst1.8	{q3},[r1]

Lctr32_done:
	vldmia	sp!,{d8,d9,d10,d11,d12,d13,d14,d15}
	ldmia	sp!,{r4,r5,r6,r7,r8,r9,r10,pc}

#endif
#endif  // !OPENSSL_NO_ASM
