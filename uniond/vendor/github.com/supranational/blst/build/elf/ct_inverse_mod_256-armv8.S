.text

.globl	ct_inverse_mod_256
.type	ct_inverse_mod_256, %function
.align	5
ct_inverse_mod_256:
	.inst	0xd503233f
	stp	x29, x30, [sp,#-80]!
	add	x29, sp, #0
	stp	x19, x20, [sp,#16]
	stp	x21, x22, [sp,#32]
	stp	x23, x24, [sp,#48]
	stp	x25, x26, [sp,#64]
	sub	sp, sp, #1040

	ldp	x4, x5, [x1,#8*0]
	ldp	x6, x7, [x1,#8*2]

	add	x1, sp, #16+511	// find closest 512-byte-aligned spot
	and	x1, x1, #-512	// in the frame...
	str	x0, [sp]

	ldp	x8, x9, [x2,#8*0]
	ldp	x10, x11, [x2,#8*2]

	stp	x4, x5, [x1,#8*0]	// copy input to |a|
	stp	x6, x7, [x1,#8*2]
	stp	x8, x9, [x1,#8*4]	// copy modulus to |b|
	stp	x10, x11, [x1,#8*6]

	////////////////////////////////////////// first iteration
	bl	.Lab_approximation_31_256_loaded

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	str	x12,[x0,#8*8]		// initialize |u| with |f0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to dst |b|
	bl	__smul_256_n_shift_by_31
	str	x12, [x0,#8*9]		// initialize |v| with |f1|

	////////////////////////////////////////// second iteration
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	ldr	x8, [x1,#8*8]		// |u|
	ldr	x9, [x1,#8*13]	// |v|
	madd	x4, x16, x8, xzr	// |u|*|f0|
	madd	x4, x17, x9, x4	// |v|*|g0|
	str	x4, [x0,#8*4]
	asr	x5, x4, #63		// sign extenstion
	stp	x5, x5, [x0,#8*5]
	stp	x5, x5, [x0,#8*7]

	madd	x4, x12, x8, xzr	// |u|*|f1|
	madd	x4, x13, x9, x4	// |v|*|g1|
	str	x4, [x0,#8*9]
	asr	x5, x4, #63		// sign extenstion
	stp	x5, x5, [x0,#8*10]
	stp	x5, x5, [x0,#8*12]
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	x16, x12			// corrected |f0|
	mov	x17, x13			// corrected |g0|

	mov	x12, x14			// |f1|
	mov	x13, x15			// |g1|
	add	x0, x0, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	x0, x0, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12			// corrected |f1|
	mov	x17, x13			// corrected |g1|
	add	x0, x0, #8*5	// pointer to destination |v|
	bl	__smul_256x63
	bl	__smul_512x63_tail
	////////////////////////////////////////// two[!] last iterations
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #47			// 31 + 512 % 31
	//bl	__ab_approximation_62_256	// |a| and |b| are exact,
	ldr	x7, [x1,#8*0]		// just load
	ldr	x11, [x1,#8*4]
	bl	__inner_loop_62_256

	mov	x16, x14
	mov	x17, x15
	ldr	x0, [sp]			// original out_ptr
	bl	__smul_256x63
	bl	__smul_512x63_tail
	ldr	x30, [x29,#8]

	smulh	x20, x7, x17		// figure out top-most limb
	ldp	x8, x9, [x3,#8*0]
	adc	x23, x23, x25
	ldp	x10, x11, [x3,#8*2]

	add	x20, x20, x23		// x20 is 1, 0 or -1
	asr	x19, x20, #63		// sign as mask

	and	x23,   x8, x19		// add mod<<256 conditionally
	and	x24,   x9, x19
	adds	x4, x4, x23
	and	x25,   x10, x19
	adcs	x5, x5, x24
	and	x26,   x11, x19
	adcs	x6, x6, x25
	adcs	x7, x22,   x26
	adc	x20, x20, xzr		// x20 is 1, 0 or -1

	neg	x19, x20
	orr	x20, x20, x19		// excess bit or sign as mask
	asr	x19, x19, #63		// excess bit as mask

	and	x8, x8, x20		// mask |mod|
	and	x9, x9, x20
	and	x10, x10, x20
	and	x11, x11, x20

	eor	x8, x8, x19		// conditionally negate |mod|
	eor	x9, x9, x19
	adds	x8, x8, x19, lsr#63
	eor	x10, x10, x19
	adcs	x9, x9, xzr
	eor	x11, x11, x19
	adcs	x10, x10, xzr
	adc	x11, x11, xzr

	adds	x4, x4, x8	// final adjustment for |mod|<<256
	adcs	x5, x5, x9
	adcs	x6, x6, x10
	stp	x4, x5, [x0,#8*4]
	adc	x7, x7, x11
	stp	x6, x7, [x0,#8*6]

	add	sp, sp, #1040
	ldp	x19, x20, [x29,#16]
	ldp	x21, x22, [x29,#32]
	ldp	x23, x24, [x29,#48]
	ldp	x25, x26, [x29,#64]
	ldr	x29, [sp],#80
	.inst	0xd50323bf
	ret
.size	ct_inverse_mod_256,.-ct_inverse_mod_256

////////////////////////////////////////////////////////////////////////
.type	__smul_256x63, %function
.align	5
__smul_256x63:
	ldp	x4, x5, [x1,#8*0+64]	// load |u| (or |v|)
	asr	x14, x16, #63		// |f_|'s sign as mask (or |g_|'s)
	ldp	x6, x7, [x1,#8*2+64]
	eor	x16, x16, x14		// conditionally negate |f_| (or |g_|)
	ldr	x22, [x1,#8*4+64]

	eor	x4, x4, x14	// conditionally negate |u| (or |v|)
	sub	x16, x16, x14
	eor	x5, x5, x14
	adds	x4, x4, x14, lsr#63
	eor	x6, x6, x14
	adcs	x5, x5, xzr
	eor	x7, x7, x14
	adcs	x6, x6, xzr
	eor	x22, x22, x14
	umulh	x19, x4, x16
	adcs	x7, x7, xzr
	umulh	x20, x5, x16
	adcs	x22, x22, xzr
	umulh	x21, x6, x16
	mul	x4, x4, x16
	cmp	x16, #0
	mul	x5, x5, x16
	csel	x22, x22, xzr, ne
	mul	x6, x6, x16
	adds	x5, x5, x19
	mul	x24, x7, x16
	adcs	x6, x6, x20
	adcs	x24, x24, x21
	adc	x26, xzr, xzr
	ldp	x8, x9, [x1,#8*0+104]	// load |u| (or |v|)
	asr	x14, x17, #63		// |f_|'s sign as mask (or |g_|'s)
	ldp	x10, x11, [x1,#8*2+104]
	eor	x17, x17, x14		// conditionally negate |f_| (or |g_|)
	ldr	x23, [x1,#8*4+104]

	eor	x8, x8, x14	// conditionally negate |u| (or |v|)
	sub	x17, x17, x14
	eor	x9, x9, x14
	adds	x8, x8, x14, lsr#63
	eor	x10, x10, x14
	adcs	x9, x9, xzr
	eor	x11, x11, x14
	adcs	x10, x10, xzr
	eor	x23, x23, x14
	umulh	x19, x8, x17
	adcs	x11, x11, xzr
	umulh	x20, x9, x17
	adcs	x23, x23, xzr
	umulh	x21, x10, x17
	adc	x15, xzr, xzr		// used in __smul_512x63_tail
	mul	x8, x8, x17
	cmp	x17, #0
	mul	x9, x9, x17
	csel	x23, x23, xzr, ne
	mul	x10, x10, x17
	adds	x9, x9, x19
	mul	x25, x11, x17
	adcs	x10, x10, x20
	adcs	x25, x25, x21
	adc	x26, x26, xzr

	adds	x4, x4, x8
	adcs	x5, x5, x9
	adcs	x6, x6, x10
	stp	x4, x5, [x0,#8*0]
	adcs	x24,   x24,   x25
	stp	x6, x24, [x0,#8*2]

	ret
.size	__smul_256x63,.-__smul_256x63

.type	__smul_512x63_tail, %function
.align	5
__smul_512x63_tail:
	umulh	x24, x7, x16
	ldp	x5, x6, [x1,#8*18]	// load rest of |v|
	adc	x26, x26, xzr
	ldr	x7, [x1,#8*20]
	and	x22, x22, x16

	umulh	x11, x11, x17	// resume |v|*|g1| chain

	sub	x24, x24, x22	// tie up |u|*|f1| chain
	asr	x25, x24, #63

	eor	x5, x5, x14	// conditionally negate rest of |v|
	eor	x6, x6, x14
	adds	x5, x5, x15
	eor	x7, x7, x14
	adcs	x6, x6, xzr
	umulh	x19, x23,   x17
	adc	x7, x7, xzr
	umulh	x20, x5, x17
	add	x11, x11, x26
	umulh	x21, x6, x17

	mul	x4, x23,   x17
	mul	x5, x5, x17
	adds	x4, x4, x11
	mul	x6, x6, x17
	adcs	x5, x5, x19
	mul	x22,   x7, x17
	adcs	x6, x6, x20
	adcs	x22,   x22,   x21
	adc	x23, xzr, xzr		// used in the final step

	adds	x4, x4, x24
	adcs	x5, x5, x25
	adcs	x6, x6, x25
	stp	x4, x5, [x0,#8*4]
	adcs	x22,   x22,   x25	// carry is used in the final step
	stp	x6, x22,   [x0,#8*6]

	ret
.size	__smul_512x63_tail,.-__smul_512x63_tail

.type	__smul_256_n_shift_by_31, %function
.align	5
__smul_256_n_shift_by_31:
	ldp	x4, x5, [x1,#8*0+0]	// load |a| (or |b|)
	asr	x24, x12, #63		// |f0|'s sign as mask (or |g0|'s)
	ldp	x6, x7, [x1,#8*2+0]
	eor	x25, x12, x24	// conditionally negate |f0| (or |g0|)

	eor	x4, x4, x24	// conditionally negate |a| (or |b|)
	sub	x25, x25, x24
	eor	x5, x5, x24
	adds	x4, x4, x24, lsr#63
	eor	x6, x6, x24
	adcs	x5, x5, xzr
	eor	x7, x7, x24
	umulh	x19, x4, x25
	adcs	x6, x6, xzr
	umulh	x20, x5, x25
	adc	x7, x7, xzr
	umulh	x21, x6, x25
	and	x24, x24, x25
	umulh	x22, x7, x25
	neg	x24, x24

	mul	x4, x4, x25
	mul	x5, x5, x25
	mul	x6, x6, x25
	adds	x5, x5, x19
	mul	x7, x7, x25
	adcs	x6, x6, x20
	adcs	x7, x7, x21
	adc	x22, x22, x24
	ldp	x8, x9, [x1,#8*0+32]	// load |a| (or |b|)
	asr	x24, x13, #63		// |f0|'s sign as mask (or |g0|'s)
	ldp	x10, x11, [x1,#8*2+32]
	eor	x25, x13, x24	// conditionally negate |f0| (or |g0|)

	eor	x8, x8, x24	// conditionally negate |a| (or |b|)
	sub	x25, x25, x24
	eor	x9, x9, x24
	adds	x8, x8, x24, lsr#63
	eor	x10, x10, x24
	adcs	x9, x9, xzr
	eor	x11, x11, x24
	umulh	x19, x8, x25
	adcs	x10, x10, xzr
	umulh	x20, x9, x25
	adc	x11, x11, xzr
	umulh	x21, x10, x25
	and	x24, x24, x25
	umulh	x23, x11, x25
	neg	x24, x24

	mul	x8, x8, x25
	mul	x9, x9, x25
	mul	x10, x10, x25
	adds	x9, x9, x19
	mul	x11, x11, x25
	adcs	x10, x10, x20
	adcs	x11, x11, x21
	adc	x23, x23, x24
	adds	x4, x4, x8
	adcs	x5, x5, x9
	adcs	x6, x6, x10
	adcs	x7, x7, x11
	adc	x8, x22,   x23

	extr	x4, x5, x4, #31
	extr	x5, x6, x5, #31
	extr	x6, x7, x6, #31
	asr	x23, x8, #63	// result's sign as mask
	extr	x7, x8, x7, #31

	eor	x4, x4, x23	// ensure the result is positive
	eor	x5, x5, x23
	adds	x4, x4, x23, lsr#63
	eor	x6, x6, x23
	adcs	x5, x5, xzr
	eor	x7, x7, x23
	adcs	x6, x6, xzr
	stp	x4, x5, [x0,#8*0]
	adc	x7, x7, xzr
	stp	x6, x7, [x0,#8*2]

	eor	x12, x12, x23		// adjust |f/g| accordingly
	eor	x13, x13, x23
	sub	x12, x12, x23
	sub	x13, x13, x23

	ret
.size	__smul_256_n_shift_by_31,.-__smul_256_n_shift_by_31
.type	__ab_approximation_31_256, %function
.align	4
__ab_approximation_31_256:
	ldp	x6, x7, [x1,#8*2]
	ldp	x10, x11, [x1,#8*6]
	ldp	x4, x5, [x1,#8*0]
	ldp	x8, x9, [x1,#8*4]

.Lab_approximation_31_256_loaded:
	orr	x19, x7, x11	// check top-most limbs, ...
	cmp	x19, #0
	csel	x7, x7, x6, ne
	csel	x11, x11, x10, ne
	csel	x6, x6, x5, ne
	orr	x19, x7, x11	// and ones before top-most, ...
	csel	x10, x10, x9, ne

	cmp	x19, #0
	csel	x7, x7, x6, ne
	csel	x11, x11, x10, ne
	csel	x6, x6, x4, ne
	orr	x19, x7, x11	// and one more, ...
	csel	x10, x10, x8, ne

	clz	x19, x19
	cmp	x19, #64
	csel	x19, x19, xzr, ne
	csel	x7, x7, x6, ne
	csel	x11, x11, x10, ne
	neg	x20, x19

	lslv	x7, x7, x19	// align high limbs to the left
	lslv	x11, x11, x19
	lsrv	x6, x6, x20
	lsrv	x10, x10, x20
	and	x6, x6, x20, asr#6
	and	x10, x10, x20, asr#6
	orr	x7, x7, x6
	orr	x11, x11, x10

	bfxil	x7, x4, #0, #31
	bfxil	x11, x8, #0, #31

	b	__inner_loop_31_256
	ret
.size	__ab_approximation_31_256,.-__ab_approximation_31_256

.type	__inner_loop_31_256, %function
.align	4
__inner_loop_31_256:
	mov	x2, #31
	mov	x13, #0x7FFFFFFF80000000	// |f0|=1, |g0|=0
	mov	x15, #0x800000007FFFFFFF	// |f1|=0, |g1|=1
	mov	x23,#0x7FFFFFFF7FFFFFFF

.Loop_31_256:
	sbfx	x22, x7, #0, #1	// if |a_| is odd, then we'll be subtracting
	sub	x2, x2, #1
	and	x19, x11, x22
	sub	x20, x11, x7	// |b_|-|a_|
	subs	x21, x7, x19	// |a_|-|b_| (or |a_|-0 if |a_| was even)
	mov	x19, x15
	csel	x11, x11, x7, hs	// |b_| = |a_|
	csel	x7, x21, x20, hs	// borrow means |a_|<|b_|, replace with |b_|-|a_|
	csel	x15, x15, x13,    hs	// exchange |fg0| and |fg1|
	csel	x13, x13, x19,   hs
	lsr	x7, x7, #1
	and	x19, x15, x22
	and	x20, x23, x22
	sub	x13, x13, x19	// |f0|-=|f1| (or |f0-=0| if |a_| was even)
	add	x15, x15, x15	// |f1|<<=1
	add	x13, x13, x20
	sub	x15, x15, x23
	cbnz	x2, .Loop_31_256

	mov	x23, #0x7FFFFFFF
	ubfx	x12, x13, #0, #32
	ubfx	x13, x13, #32, #32
	ubfx	x14, x15, #0, #32
	ubfx	x15, x15, #32, #32
	sub	x12, x12, x23		// remove bias
	sub	x13, x13, x23
	sub	x14, x14, x23
	sub	x15, x15, x23

	ret
.size	__inner_loop_31_256,.-__inner_loop_31_256

.type	__inner_loop_62_256, %function
.align	4
__inner_loop_62_256:
	mov	x12, #1		// |f0|=1
	mov	x13, #0		// |g0|=0
	mov	x14, #0		// |f1|=0
	mov	x15, #1		// |g1|=1

.Loop_62_256:
	sbfx	x22, x7, #0, #1	// if |a_| is odd, then we'll be subtracting
	sub	x2, x2, #1
	and	x19, x11, x22
	sub	x20, x11, x7	// |b_|-|a_|
	subs	x21, x7, x19	// |a_|-|b_| (or |a_|-0 if |a_| was even)
	mov	x19, x12
	csel	x11, x11, x7, hs	// |b_| = |a_|
	csel	x7, x21, x20, hs	// borrow means |a_|<|b_|, replace with |b_|-|a_|
	mov	x20, x13
	csel	x12, x12, x14,       hs	// exchange |f0| and |f1|
	csel	x14, x14, x19,     hs
	csel	x13, x13, x15,       hs	// exchange |g0| and |g1|
	csel	x15, x15, x20,     hs
	lsr	x7, x7, #1
	and	x19, x14, x22
	and	x20, x15, x22
	add	x14, x14, x14		// |f1|<<=1
	add	x15, x15, x15		// |g1|<<=1
	sub	x12, x12, x19		// |f0|-=|f1| (or |f0-=0| if |a_| was even)
	sub	x13, x13, x20		// |g0|-=|g1| (or |g0-=0| ...)
	cbnz	x2, .Loop_62_256

	ret
.size	__inner_loop_62_256,.-__inner_loop_62_256
