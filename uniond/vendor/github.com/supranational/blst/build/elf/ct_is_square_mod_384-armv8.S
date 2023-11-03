.text

.globl	ct_is_square_mod_384
.type	ct_is_square_mod_384, %function
.align	5
ct_is_square_mod_384:
	.inst	0xd503233f
	stp	x29, x30, [sp,#-128]!
	add	x29, sp, #0
	stp	x19, x20, [sp,#16]
	stp	x21, x22, [sp,#32]
	stp	x23, x24, [sp,#48]
	stp	x25, x26, [sp,#64]
	stp	x27, x28, [sp,#80]
	sub	sp, sp, #512

	ldp	x3, x4, [x0,#8*0]		// load input
	ldp	x5, x6, [x0,#8*2]
	ldp	x7, x8, [x0,#8*4]

	add	x0, sp, #255	// find closest 256-byte-aligned spot
	and	x0, x0, #-256	// in the frame...

	ldp	x9, x10, [x1,#8*0]		// load modulus
	ldp	x11, x12, [x1,#8*2]
	ldp	x13, x14, [x1,#8*4]

	stp	x3, x4, [x0,#8*6]	// copy input to |a|
	stp	x5, x6, [x0,#8*8]
	stp	x7, x8, [x0,#8*10]
	stp	x9, x10, [x0,#8*0]	// copy modulus to |b|
	stp	x11, x12, [x0,#8*2]
	stp	x13, x14, [x0,#8*4]

	eor	x2, x2, x2			// init the .Legendre symbol
	mov	x15, #24			// 24 is 768/30-1
	b	.Loop_is_square

.align	4
.Loop_is_square:
	bl	__ab_approximation_30
	sub	x15, x15, #1

	eor	x1, x0, #128		// pointer to dst |b|
	bl	__smul_384_n_shift_by_30

	mov	x19, x16			// |f0|
	mov	x20, x17			// |g0|
	add	x1, x1, #8*6	// pointer to dst |a|
	bl	__smul_384_n_shift_by_30

	ldp	x9, x10, [x1,#-8*6]
	eor	x0, x0, #128		// flip-flop src |a|b|
	and	x27, x27, x9		// if |a| was negative,
	add	x2, x2, x27, lsr#1		// adjust |L|

	cbnz	x15, .Loop_is_square

	////////////////////////////////////////// last iteration
	//bl	__ab_approximation_30		// |a| and |b| are exact,
	//ldr	x8, [x0,#8*6]		// and loaded
	//ldr	x14, [x0,#8*0]
	mov	x15, #48			// 48 is 768%30 + 30
	bl	__inner_loop_48
	ldr	x30, [x29,#8]

	and	x0, x2, #1
	eor	x0, x0, #1

	add	sp, sp, #512
	ldp	x19, x20, [x29,#16]
	ldp	x21, x22, [x29,#32]
	ldp	x23, x24, [x29,#48]
	ldp	x25, x26, [x29,#64]
	ldp	x27, x28, [x29,#80]
	ldr	x29, [sp],#128
	.inst	0xd50323bf
	ret
.size	ct_is_square_mod_384,.-ct_is_square_mod_384

.type	__smul_384_n_shift_by_30, %function
.align	5
__smul_384_n_shift_by_30:
	ldp	x3, x4, [x0,#8*0+0]	// load |b| (or |a|)
	asr	x27, x20, #63		// |g1|'s sign as mask (or |f1|'s)
	ldp	x5, x6, [x0,#8*2+0]
	eor	x20, x20, x27		// conditionally negate |g1| (or |f1|)
	ldp	x7, x8, [x0,#8*4+0]

	eor	x3, x3, x27	// conditionally negate |b| (or |a|)
	sub	x20, x20, x27
	eor	x4, x4, x27
	adds	x3, x3, x27, lsr#63
	eor	x5, x5, x27
	adcs	x4, x4, xzr
	eor	x6, x6, x27
	adcs	x5, x5, xzr
	eor	x7, x7, x27
	umulh	x21, x3, x20
	adcs	x6, x6, xzr
	umulh	x22, x4, x20
	eor	x8, x8, x27
	umulh	x23, x5, x20
	adcs	x7, x7, xzr
	umulh	x24, x6, x20
	adc	x8, x8, xzr

	umulh	x25, x7, x20
	and	x28, x20, x27
	umulh	x26, x8, x20
	neg	x28, x28
	mul	x3, x3, x20
	mul	x4, x4, x20
	mul	x5, x5, x20
	adds	x4, x4, x21
	mul	x6, x6, x20
	adcs	x5, x5, x22
	mul	x7, x7, x20
	adcs	x6, x6, x23
	mul	x8, x8, x20
	adcs	x7, x7, x24
	adcs	x8, x8 ,x25
	adc	x26, x26, x28
	ldp	x9, x10, [x0,#8*0+48]	// load |b| (or |a|)
	asr	x27, x19, #63		// |g1|'s sign as mask (or |f1|'s)
	ldp	x11, x12, [x0,#8*2+48]
	eor	x19, x19, x27		// conditionally negate |g1| (or |f1|)
	ldp	x13, x14, [x0,#8*4+48]

	eor	x9, x9, x27	// conditionally negate |b| (or |a|)
	sub	x19, x19, x27
	eor	x10, x10, x27
	adds	x9, x9, x27, lsr#63
	eor	x11, x11, x27
	adcs	x10, x10, xzr
	eor	x12, x12, x27
	adcs	x11, x11, xzr
	eor	x13, x13, x27
	umulh	x21, x9, x19
	adcs	x12, x12, xzr
	umulh	x22, x10, x19
	eor	x14, x14, x27
	umulh	x23, x11, x19
	adcs	x13, x13, xzr
	umulh	x24, x12, x19
	adc	x14, x14, xzr

	umulh	x25, x13, x19
	and	x28, x19, x27
	umulh	x27, x14, x19
	neg	x28, x28
	mul	x9, x9, x19
	mul	x10, x10, x19
	mul	x11, x11, x19
	adds	x10, x10, x21
	mul	x12, x12, x19
	adcs	x11, x11, x22
	mul	x13, x13, x19
	adcs	x12, x12, x23
	mul	x14, x14, x19
	adcs	x13, x13, x24
	adcs	x14, x14 ,x25
	adc	x27, x27, x28
	adds	x3, x3, x9
	adcs	x4, x4, x10
	adcs	x5, x5, x11
	adcs	x6, x6, x12
	adcs	x7, x7, x13
	adcs	x8, x8, x14
	adc	x9, x26,   x27

	extr	x3, x4, x3, #30
	extr	x4, x5, x4, #30
	extr	x5, x6, x5, #30
	asr	x27, x9, #63
	extr	x6, x7, x6, #30
	extr	x7, x8, x7, #30
	extr	x8, x9, x8, #30

	eor	x3, x3, x27
	eor	x4, x4, x27
	adds	x3, x3, x27, lsr#63
	eor	x5, x5, x27
	adcs	x4, x4, xzr
	eor	x6, x6, x27
	adcs	x5, x5, xzr
	eor	x7, x7, x27
	adcs	x6, x6, xzr
	eor	x8, x8, x27
	stp	x3, x4, [x1,#8*0]
	adcs	x7, x7, xzr
	stp	x5, x6, [x1,#8*2]
	adc	x8, x8, xzr
	stp	x7, x8, [x1,#8*4]

	ret
.size	__smul_384_n_shift_by_30,.-__smul_384_n_shift_by_30
.type	__ab_approximation_30, %function
.align	4
__ab_approximation_30:
	ldp	x13, x14, [x0,#8*4]	// |a| is still in registers
	ldp	x11, x12, [x0,#8*2]

	orr	x21, x8, x14	// check top-most limbs, ...
	cmp	x21, #0
	csel	x8, x8, x7, ne
	csel	x14, x14, x13, ne
	csel	x7, x7, x6, ne
	orr	x21, x8, x14	// ... ones before top-most, ...
	csel	x13, x13, x12, ne

	cmp	x21, #0
	csel	x8, x8, x7, ne
	csel	x14, x14, x13, ne
	csel	x7, x7, x5, ne
	orr	x21, x8, x14	// ... and ones before that ...
	csel	x13, x13, x11, ne

	cmp	x21, #0
	csel	x8, x8, x7, ne
	csel	x14, x14, x13, ne
	csel	x7, x7, x4, ne
	orr	x21, x8, x14	// and one more, ...
	csel	x13, x13, x10, ne

	cmp	x21, #0
	csel	x8, x8, x7, ne
	csel	x14, x14, x13, ne
	csel	x7, x7, x3, ne
	orr	x21, x8, x14
	csel	x13, x13, x9, ne

	clz	x21, x21
	cmp	x21, #64
	csel	x21, x21, xzr, ne
	csel	x8, x8, x7, ne
	csel	x14, x14, x13, ne
	neg	x22, x21

	lslv	x8, x8, x21	// align high limbs to the left
	lslv	x14, x14, x21
	lsrv	x7, x7, x22
	lsrv	x13, x13, x22
	and	x7, x7, x22, asr#6
	and	x13, x13, x22, asr#6
	orr	x8, x8, x7
	orr	x14, x14, x13

	bfxil	x8, x3, #0, #32
	bfxil	x14, x9, #0, #32

	b	__inner_loop_30
	ret
.size	__ab_approximation_30,.-__ab_approximation_30

.type	__inner_loop_30, %function
.align	4
__inner_loop_30:
	mov	x28, #30
	mov	x17, #0x7FFFFFFF80000000	// |f0|=1, |g0|=0
	mov	x20, #0x800000007FFFFFFF	// |f1|=0, |g1|=1
	mov	x27,#0x7FFFFFFF7FFFFFFF

.Loop_30:
	sbfx	x24, x8, #0, #1	// if |a_| is odd, then we'll be subtracting
	and	x25, x8, x14
	sub	x28, x28, #1
	and	x21, x14, x24

	sub	x22, x14, x8		// |b_|-|a_|
	subs	x23, x8, x21	// |a_|-|b_| (or |a_|-0 if |a_| was even)
	add	x25, x2, x25, lsr#1	// L + (a_ & b_) >> 1
	mov	x21, x20
	csel	x14, x14, x8, hs	// |b_| = |a_|
	csel	x8, x23, x22, hs	// borrow means |a_|<|b_|, replace with |b_|-|a_|
	csel	x20, x20, x17,  hs	// exchange |fg0| and |fg1|
	csel	x17, x17, x21, hs
	csel	x2,   x2,   x25, hs
	lsr	x8, x8, #1
	and	x21, x20, x24
	and	x22, x27, x24
	add	x23, x14, #2
	sub	x17, x17, x21	// |f0|-=|f1| (or |f0-=0| if |a_| was even)
	add	x20, x20, x20	// |f1|<<=1
	add	x2, x2, x23, lsr#2	// "negate" |L| if |b|%8 is 3 or 5
	add	x17, x17, x22
	sub	x20, x20, x27

	cbnz	x28, .Loop_30

	mov	x27, #0x7FFFFFFF
	ubfx	x16, x17, #0, #32
	ubfx	x17, x17, #32, #32
	ubfx	x19, x20, #0, #32
	ubfx	x20, x20, #32, #32
	sub	x16, x16, x27		// remove the bias
	sub	x17, x17, x27
	sub	x19, x19, x27
	sub	x20, x20, x27

	ret
.size	__inner_loop_30,.-__inner_loop_30
.type	__inner_loop_48, %function
.align	4
__inner_loop_48:
.Loop_48:
	sbfx	x24, x3, #0, #1	// if |a_| is odd, then we'll be subtracting
	and	x25, x3, x9
	sub	x15, x15, #1
	and	x21, x9, x24
	sub	x22, x9, x3		// |b_|-|a_|
	subs	x23, x3, x21	// |a_|-|b_| (or |a_|-0 if |a_| was even)
	add	x25, x2, x25, lsr#1
	csel	x9, x9, x3, hs	// |b_| = |a_|
	csel	x3, x23, x22, hs	// borrow means |a_|<|b_|, replace with |b_|-|a_|
	csel	x2,   x2,   x25, hs
	add	x23, x9, #2
	lsr	x3, x3, #1
	add	x2, x2, x23, lsr#2	// "negate" |L| if |b|%8 is 3 or 5

	cbnz	x15, .Loop_48

	ret
.size	__inner_loop_48,.-__inner_loop_48
