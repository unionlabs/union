.text

.globl	_ct_inverse_mod_383

.align	5
_ct_inverse_mod_383:
.long	3573752639
	stp	x29, x30, [sp,#-128]!
	add	x29, sp, #0
	stp	x19, x20, [sp,#16]
	stp	x21, x22, [sp,#32]
	stp	x23, x24, [sp,#48]
	stp	x25, x26, [sp,#64]
	stp	x27, x28, [sp,#80]
	sub	sp, sp, #1040

	ldp	x22,   x4, [x1,#8*0]
	ldp	x5, x6, [x1,#8*2]
	ldp	x7, x8, [x1,#8*4]

	add	x1, sp, #16+511	// find closest 512-byte-aligned spot
	and	x1, x1, #-512	// in the frame...
	stp	x0, x3, [sp]

	ldp	x9, x10, [x2,#8*0]
	ldp	x11, x12, [x2,#8*2]
	ldp	x13, x14, [x2,#8*4]

	stp	x22,   x4, [x1,#8*0]	// copy input to |a|
	stp	x5, x6, [x1,#8*2]
	stp	x7, x8, [x1,#8*4]
	stp	x9, x10, [x1,#8*6]	// copy modulus to |b|
	stp	x11, x12, [x1,#8*8]
	stp	x13, x14, [x1,#8*10]

	////////////////////////////////////////// first iteration
	mov	x2, #62
	bl	Lab_approximation_62_loaded

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	str	x15,[x0,#8*12]		// initialize |u| with |f0|

	mov	x15, x17			// |f1|
	mov	x16, x19			// |g1|
	add	x0, x0, #8*6	// pointer to dst |b|
	bl	__smul_383_n_shift_by_62
	str	x15, [x0,#8*12]		// initialize |v| with |f1|

	////////////////////////////////////////// second iteration
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	x20, x15			// corrected |f0|
	mov	x21, x16			// corrected |g0|

	mov	x15, x17			// |f1|
	mov	x16, x19			// |g1|
	add	x0, x0, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	ldr	x7, [x1,#8*12]	// |u|
	ldr	x8, [x1,#8*18]	// |v|
	mul	x3, x20, x7		// |u|*|f0|
	smulh	x4, x20, x7
	mul	x5, x21, x8		// |v|*|g0|
	smulh	x6, x21, x8
	adds	x3, x3, x5
	adc	x4, x4, x6
	stp	x3, x4, [x0,#8*6]
	asr	x5, x4, #63		// sign extenstion
	stp	x5, x5, [x0,#8*8]
	stp	x5, x5, [x0,#8*10]

	mul	x3, x15, x7		// |u|*|f1|
	smulh	x4, x15, x7
	mul	x5, x16, x8		// |v|*|g1|
	smulh	x6, x16, x8
	adds	x3, x3, x5
	adc	x4, x4, x6
	stp	x3, x4, [x0,#8*12]
	asr	x5, x4, #63		// sign extenstion
	stp	x5, x5, [x0,#8*14]
	stp	x5, x5, [x0,#8*16]
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	x20, x15			// corrected |f0|
	mov	x21, x16			// corrected |g0|

	mov	x15, x17			// |f1|
	mov	x16, x19			// |g1|
	add	x0, x0, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	add	x0, x0, #8*6	// pointer to destination |u|
	bl	__smul_383x63

	mov	x20, x15			// corrected |f1|
	mov	x21, x16			// corrected |g1|
	add	x0, x0, #8*6	// pointer to destination |v|
	bl	__smul_383x63
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	x20, x15			// corrected |f0|
	mov	x21, x16			// corrected |g0|

	mov	x15, x17			// |f1|
	mov	x16, x19			// |g1|
	add	x0, x0, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	add	x0, x0, #8*6	// pointer to destination |u|
	bl	__smul_383x63

	mov	x20, x15			// corrected |f1|
	mov	x21, x16			// corrected |g1|
	add	x0, x0, #8*6	// pointer to destination |v|
	bl	__smul_383x63
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	x20, x15			// corrected |f0|
	mov	x21, x16			// corrected |g0|

	mov	x15, x17			// |f1|
	mov	x16, x19			// |g1|
	add	x0, x0, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	add	x0, x0, #8*6	// pointer to destination |u|
	bl	__smul_383x63

	mov	x20, x15			// corrected |f1|
	mov	x21, x16			// corrected |g1|
	add	x0, x0, #8*6	// pointer to destination |v|
	bl	__smul_383x63
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	x20, x15			// corrected |f0|
	mov	x21, x16			// corrected |g0|

	mov	x15, x17			// |f1|
	mov	x16, x19			// |g1|
	add	x0, x0, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	add	x0, x0, #8*6	// pointer to destination |u|
	bl	__smul_383x63

	mov	x20, x15			// corrected |f1|
	mov	x21, x16			// corrected |g1|
	add	x0, x0, #8*6	// pointer to destination |v|
	bl	__smul_383x63
	asr	x27, x27, #63		// sign extension
	stp	x27, x27, [x0,#8*6]
	stp	x27, x27, [x0,#8*8]
	stp	x27, x27, [x0,#8*10]
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	x20, x15			// corrected |f0|
	mov	x21, x16			// corrected |g0|

	mov	x15, x17			// |f1|
	mov	x16, x19			// |g1|
	add	x0, x0, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	add	x0, x0, #8*6	// pointer to destination |u|
	bl	__smul_383x63

	mov	x20, x15			// corrected |f1|
	mov	x21, x16			// corrected |g1|
	add	x0, x0, #8*6	// pointer to destination |v|
	bl	__smul_383x63
	bl	__smul_767x63_tail
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	x20, x15			// corrected |f0|
	mov	x21, x16			// corrected |g0|

	mov	x15, x17			// |f1|
	mov	x16, x19			// |g1|
	add	x0, x0, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	add	x0, x0, #8*6	// pointer to destination |u|
	bl	__smul_383x63

	mov	x20, x15			// corrected |f1|
	mov	x21, x16			// corrected |g1|
	add	x0, x0, #8*6	// pointer to destination |v|
	bl	__smul_383x63
	bl	__smul_767x63_tail
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	x20, x15			// corrected |f0|
	mov	x21, x16			// corrected |g0|

	mov	x15, x17			// |f1|
	mov	x16, x19			// |g1|
	add	x0, x0, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	add	x0, x0, #8*6	// pointer to destination |u|
	bl	__smul_383x63

	mov	x20, x15			// corrected |f1|
	mov	x21, x16			// corrected |g1|
	add	x0, x0, #8*6	// pointer to destination |v|
	bl	__smul_383x63
	bl	__smul_767x63_tail
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	x20, x15			// corrected |f0|
	mov	x21, x16			// corrected |g0|

	mov	x15, x17			// |f1|
	mov	x16, x19			// |g1|
	add	x0, x0, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	add	x0, x0, #8*6	// pointer to destination |u|
	bl	__smul_383x63

	mov	x20, x15			// corrected |f1|
	mov	x21, x16			// corrected |g1|
	add	x0, x0, #8*6	// pointer to destination |v|
	bl	__smul_383x63
	bl	__smul_767x63_tail
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	x20, x15			// corrected |f0|
	mov	x21, x16			// corrected |g0|

	mov	x15, x17			// |f1|
	mov	x16, x19			// |g1|
	add	x0, x0, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	add	x0, x0, #8*6	// pointer to destination |u|
	bl	__smul_383x63

	mov	x20, x15			// corrected |f1|
	mov	x21, x16			// corrected |g1|
	add	x0, x0, #8*6	// pointer to destination |v|
	bl	__smul_383x63
	bl	__smul_767x63_tail
	////////////////////////////////////////// iteration before last
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #62
	//bl	__ab_approximation_62		// |a| and |b| are exact,
	ldp	x3, x8, [x1,#8*0]	// just load
	ldp	x9, x14, [x1,#8*6]
	bl	__inner_loop_62

	eor	x0, x1, #256		// pointer to dst |a|b|u|v|
	str	x3, [x0,#8*0]
	str	x9, [x0,#8*6]

	mov	x20, x15			// exact |f0|
	mov	x21, x16			// exact |g0|
	mov	x15, x17
	mov	x16, x19
	add	x0, x0, #8*12	// pointer to dst |u|
	bl	__smul_383x63

	mov	x20, x15			// exact |f1|
	mov	x21, x16			// exact |g1|
	add	x0, x0, #8*6	// pointer to dst |v|
	bl	__smul_383x63
	bl	__smul_767x63_tail

	////////////////////////////////////////// last iteration
	eor	x1, x1, #256		// flip-flop src |a|b|u|v|
	mov	x2, #22			// 766 % 62
	//bl	__ab_approximation_62		// |a| and |b| are exact,
	ldr	x3, [x1,#8*0]		// just load
	eor	x8, x8, x8
	ldr	x9, [x1,#8*6]
	eor	x14, x14, x14
	bl	__inner_loop_62

	mov	x20, x17
	mov	x21, x19
	ldp	x0, x15, [sp]		// original out_ptr and n_ptr
	bl	__smul_383x63
	bl	__smul_767x63_tail
	ldr	x30, [x29,#8]

	asr	x22, x8, #63		// sign as mask
	ldp	x9, x10, [x15,#8*0]
	ldp	x11, x12, [x15,#8*2]
	ldp	x13, x14, [x15,#8*4]

	and	x9, x9, x22		// add mod<<384 conditionally
	and	x10, x10, x22
	adds	x3, x3, x9
	and	x11, x11, x22
	adcs	x4, x4, x10
	and	x12, x12, x22
	adcs	x5, x5, x11
	and	x13, x13, x22
	adcs	x6, x6, x12
	and	x14, x14, x22
	stp	x3, x4, [x0,#8*6]
	adcs	x7, x7, x13
	stp	x5, x6, [x0,#8*8]
	adc	x8, x8, x14
	stp	x7, x8, [x0,#8*10]

	add	sp, sp, #1040
	ldp	x19, x20, [x29,#16]
	ldp	x21, x22, [x29,#32]
	ldp	x23, x24, [x29,#48]
	ldp	x25, x26, [x29,#64]
	ldp	x27, x28, [x29,#80]
	ldr	x29, [sp],#128
.long	3573752767
	ret


////////////////////////////////////////////////////////////////////////
// see corresponding commentary in ctx_inverse_mod_384-x86_64...

.align	5
__smul_383x63:
	ldp	x3, x4, [x1,#8*0+96]	// load |u| (or |v|)
	asr	x17, x20, #63		// |f_|'s sign as mask (or |g_|'s)
	ldp	x5, x6, [x1,#8*2+96]
	eor	x20, x20, x17		// conditionally negate |f_| (or |g_|)
	ldp	x7, x8, [x1,#8*4+96]

	eor	x3, x3, x17	// conditionally negate |u| (or |v|)
	sub	x20, x20, x17
	eor	x4, x4, x17
	adds	x3, x3, x17, lsr#63
	eor	x5, x5, x17
	adcs	x4, x4, xzr
	eor	x6, x6, x17
	adcs	x5, x5, xzr
	eor	x7, x7, x17
	adcs	x6, x6, xzr
	umulh	x22, x3, x20
	eor	x8, x8, x17
	umulh	x23, x4, x20
	adcs	x7, x7, xzr
	umulh	x24, x5, x20
	adcs	x8, x8, xzr
	umulh	x25, x6, x20
	umulh	x26, x7, x20
	mul	x3, x3, x20
	mul	x4, x4, x20
	mul	x5, x5, x20
	adds	x4, x4, x22
	mul	x6, x6, x20
	adcs	x5, x5, x23
	mul	x7, x7, x20
	adcs	x6, x6, x24
	mul	x27,x8, x20
	adcs	x7, x7, x25
	adcs	x27,x27,x26
	adc	x2, xzr, xzr
	ldp	x9, x10, [x1,#8*0+144]	// load |u| (or |v|)
	asr	x17, x21, #63		// |f_|'s sign as mask (or |g_|'s)
	ldp	x11, x12, [x1,#8*2+144]
	eor	x21, x21, x17		// conditionally negate |f_| (or |g_|)
	ldp	x13, x14, [x1,#8*4+144]

	eor	x9, x9, x17	// conditionally negate |u| (or |v|)
	sub	x21, x21, x17
	eor	x10, x10, x17
	adds	x9, x9, x17, lsr#63
	eor	x11, x11, x17
	adcs	x10, x10, xzr
	eor	x12, x12, x17
	adcs	x11, x11, xzr
	eor	x13, x13, x17
	adcs	x12, x12, xzr
	umulh	x22, x9, x21
	eor	x14, x14, x17
	umulh	x23, x10, x21
	adcs	x13, x13, xzr
	umulh	x24, x11, x21
	adcs	x14, x14, xzr
	umulh	x25, x12, x21
	adc	x19, xzr, xzr		// used in __smul_767x63_tail
	umulh	x26, x13, x21
	mul	x9, x9, x21
	mul	x10, x10, x21
	mul	x11, x11, x21
	adds	x10, x10, x22
	mul	x12, x12, x21
	adcs	x11, x11, x23
	mul	x13, x13, x21
	adcs	x12, x12, x24
	mul	x28,x14, x21
	adcs	x13, x13, x25
	adcs	x28,x28,x26
	adc	x2, x2, xzr

	adds	x3, x3, x9
	adcs	x4, x4, x10
	adcs	x5, x5, x11
	adcs	x6, x6, x12
	stp	x3, x4, [x0,#8*0]
	adcs	x7, x7, x13
	stp	x5, x6, [x0,#8*2]
	adcs	x27,   x27,   x28
	stp	x7, x27,   [x0,#8*4]
	adc	x28,   x2,   xzr	// used in __smul_767x63_tail

	ret



.align	5
__smul_767x63_tail:
	smulh	x27,   x8, x20
	ldp	x3, x4, [x1,#8*24]	// load rest of |v|
	umulh	x14,x14, x21
	ldp	x5, x6, [x1,#8*26]
	ldp	x7, x8, [x1,#8*28]

	eor	x3, x3, x17	// conditionally negate rest of |v|
	eor	x4, x4, x17
	eor	x5, x5, x17
	adds	x3, x3, x19
	eor	x6, x6, x17
	adcs	x4, x4, xzr
	eor	x7, x7, x17
	adcs	x5, x5, xzr
	eor	x8, x8, x17
	adcs	x6, x6, xzr
	umulh	x22, x3, x21
	adcs	x7, x7, xzr
	umulh	x23, x4, x21
	adc	x8, x8, xzr

	umulh	x24, x5, x21
	add	x14, x14, x28
	umulh	x25, x6, x21
	asr	x28, x27, #63
	umulh	x26, x7, x21
	mul	x3, x3, x21
	mul	x4, x4, x21
	mul	x5, x5, x21
	adds	x3, x3, x14
	mul	x6, x6, x21
	adcs	x4, x4, x22
	mul	x7, x7, x21
	adcs	x5, x5, x23
	mul	x8, x8, x21
	adcs	x6, x6, x24
	adcs	x7, x7, x25
	adc	x8, x8, x26

	adds	x3, x3, x27
	adcs	x4, x4, x28
	adcs	x5, x5, x28
	adcs	x6, x6, x28
	stp	x3, x4, [x0,#8*6]
	adcs	x7, x7, x28
	stp	x5, x6, [x0,#8*8]
	adc	x8, x8, x28
	stp	x7, x8, [x0,#8*10]

	ret



.align	5
__smul_383_n_shift_by_62:
	ldp	x3, x4, [x1,#8*0+0]	// load |a| (or |b|)
	asr	x28, x15, #63		// |f0|'s sign as mask (or |g0|'s)
	ldp	x5, x6, [x1,#8*2+0]
	eor	x2, x15, x28	// conditionally negate |f0| (or |g0|)
	ldp	x7, x8, [x1,#8*4+0]

	eor	x3, x3, x28	// conditionally negate |a| (or |b|)
	sub	x2, x2, x28
	eor	x4, x4, x28
	adds	x3, x3, x28, lsr#63
	eor	x5, x5, x28
	adcs	x4, x4, xzr
	eor	x6, x6, x28
	adcs	x5, x5, xzr
	eor	x7, x7, x28
	umulh	x22, x3, x2
	adcs	x6, x6, xzr
	umulh	x23, x4, x2
	eor	x8, x8, x28
	umulh	x24, x5, x2
	adcs	x7, x7, xzr
	umulh	x25, x6, x2
	adc	x8, x8, xzr

	umulh	x26, x7, x2
	smulh	x27, x8, x2
	mul	x3, x3, x2
	mul	x4, x4, x2
	mul	x5, x5, x2
	adds	x4, x4, x22
	mul	x6, x6, x2
	adcs	x5, x5, x23
	mul	x7, x7, x2
	adcs	x6, x6, x24
	mul	x8, x8, x2
	adcs	x7, x7, x25
	adcs	x8, x8 ,x26
	adc	x27, x27, xzr
	ldp	x9, x10, [x1,#8*0+48]	// load |a| (or |b|)
	asr	x28, x16, #63		// |f0|'s sign as mask (or |g0|'s)
	ldp	x11, x12, [x1,#8*2+48]
	eor	x2, x16, x28	// conditionally negate |f0| (or |g0|)
	ldp	x13, x14, [x1,#8*4+48]

	eor	x9, x9, x28	// conditionally negate |a| (or |b|)
	sub	x2, x2, x28
	eor	x10, x10, x28
	adds	x9, x9, x28, lsr#63
	eor	x11, x11, x28
	adcs	x10, x10, xzr
	eor	x12, x12, x28
	adcs	x11, x11, xzr
	eor	x13, x13, x28
	umulh	x22, x9, x2
	adcs	x12, x12, xzr
	umulh	x23, x10, x2
	eor	x14, x14, x28
	umulh	x24, x11, x2
	adcs	x13, x13, xzr
	umulh	x25, x12, x2
	adc	x14, x14, xzr

	umulh	x26, x13, x2
	smulh	x28, x14, x2
	mul	x9, x9, x2
	mul	x10, x10, x2
	mul	x11, x11, x2
	adds	x10, x10, x22
	mul	x12, x12, x2
	adcs	x11, x11, x23
	mul	x13, x13, x2
	adcs	x12, x12, x24
	mul	x14, x14, x2
	adcs	x13, x13, x25
	adcs	x14, x14 ,x26
	adc	x28, x28, xzr
	adds	x3, x3, x9
	adcs	x4, x4, x10
	adcs	x5, x5, x11
	adcs	x6, x6, x12
	adcs	x7, x7, x13
	adcs	x8, x8, x14
	adc	x9, x27,   x28

	extr	x3, x4, x3, #62
	extr	x4, x5, x4, #62
	extr	x5, x6, x5, #62
	asr	x28, x9, #63
	extr	x6, x7, x6, #62
	extr	x7, x8, x7, #62
	extr	x8, x9, x8, #62

	eor	x3, x3, x28
	eor	x4, x4, x28
	adds	x3, x3, x28, lsr#63
	eor	x5, x5, x28
	adcs	x4, x4, xzr
	eor	x6, x6, x28
	adcs	x5, x5, xzr
	eor	x7, x7, x28
	adcs	x6, x6, xzr
	eor	x8, x8, x28
	stp	x3, x4, [x0,#8*0]
	adcs	x7, x7, xzr
	stp	x5, x6, [x0,#8*2]
	adc	x8, x8, xzr
	stp	x7, x8, [x0,#8*4]

	eor	x15, x15, x28
	eor	x16, x16, x28
	sub	x15, x15, x28
	sub	x16, x16, x28

	ret


.align	4
__ab_approximation_62:
	ldp	x7, x8, [x1,#8*4]
	ldp	x13, x14, [x1,#8*10]
	ldp	x5, x6, [x1,#8*2]
	ldp	x11, x12, [x1,#8*8]

Lab_approximation_62_loaded:
	orr	x22, x8, x14	// check top-most limbs, ...
	cmp	x22, #0
	csel	x8, x8, x7, ne
	csel	x14, x14, x13, ne
	csel	x7, x7, x6, ne
	orr	x22, x8, x14	// ... ones before top-most, ...
	csel	x13, x13, x12, ne

	ldp	x3, x4, [x1,#8*0]
	ldp	x9, x10, [x1,#8*6]

	cmp	x22, #0
	csel	x8, x8, x7, ne
	csel	x14, x14, x13, ne
	csel	x7, x7, x5, ne
	orr	x22, x8, x14	// ... and ones before that ...
	csel	x13, x13, x11, ne

	cmp	x22, #0
	csel	x8, x8, x7, ne
	csel	x14, x14, x13, ne
	csel	x7, x7, x4, ne
	orr	x22, x8, x14
	csel	x13, x13, x10, ne

	clz	x22, x22
	cmp	x22, #64
	csel	x22, x22, xzr, ne
	csel	x8, x8, x7, ne
	csel	x14, x14, x13, ne
	neg	x23, x22

	lslv	x8, x8, x22	// align high limbs to the left
	lslv	x14, x14, x22
	lsrv	x7, x7, x23
	lsrv	x13, x13, x23
	and	x7, x7, x23, asr#6
	and	x13, x13, x23, asr#6
	orr	x8, x8, x7
	orr	x14, x14, x13

	b	__inner_loop_62
	ret


.align	4
__inner_loop_62:
	mov	x15, #1		// |f0|=1
	mov	x16, #0		// |g0|=0
	mov	x17, #0		// |f1|=0
	mov	x19, #1		// |g1|=1

Loop_62:
	sbfx	x28, x3, #0, #1	// if |a_| is odd, then we'll be subtracting
	sub	x2, x2, #1
	subs	x24, x9, x3	// |b_|-|a_|
	and	x22, x9, x28
	sbc	x25, x14, x8
	and	x23, x14, x28
	subs	x26, x3, x22	// |a_|-|b_| (or |a_|-0 if |a_| was even)
	mov	x22, x15
	sbcs	x27, x8, x23
	mov	x23, x16
	csel	x9, x9, x3, hs	// |b_| = |a_|
	csel	x14, x14, x8, hs
	csel	x3, x26, x24, hs	// borrow means |a_|<|b_|, replace with |b_|-|a_|
	csel	x8, x27, x25, hs
	csel	x15, x15, x17,       hs	// exchange |f0| and |f1|
	csel	x17, x17, x22,     hs
	csel	x16, x16, x19,       hs	// exchange |g0| and |g1|
	csel	x19, x19, x23,     hs
	extr	x3, x8, x3, #1
	lsr	x8, x8, #1
	and	x22, x17, x28
	and	x23, x19, x28
	add	x17, x17, x17		// |f1|<<=1
	add	x19, x19, x19		// |g1|<<=1
	sub	x15, x15, x22		// |f0|-=|f1| (or |f0-=0| if |a_| was even)
	sub	x16, x16, x23		// |g0|-=|g1| (or |g0-=0| ...)
	cbnz	x2, Loop_62

	ret

