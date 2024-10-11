 GBLA __SIZEOF_POINTER__
__SIZEOF_POINTER__ SETA 64/8
	AREA	|.text|,CODE,ALIGN=8,ARM64



	EXPORT	|ct_inverse_mod_256|[FUNC]
	ALIGN	32
|ct_inverse_mod_256| PROC
	DCDU	3573752639
	stp	x29, x30, [sp,#-10*__SIZEOF_POINTER__]!
	add	x29, sp, #0
	stp	x19, x20, [sp,#2*__SIZEOF_POINTER__]
	stp	x21, x22, [sp,#4*__SIZEOF_POINTER__]
	stp	x23, x24, [sp,#6*__SIZEOF_POINTER__]
	stp	x25, x26, [sp,#8*__SIZEOF_POINTER__]
	sub	sp, sp, #1040

	ldp	x4, x5, [x1,#8*0]
	ldp	x6, x7, [x1,#8*2]

 if :def:	__CHERI_PURE_CAPABILITY__
	add	x1,sp,#16+511
	alignd	c1,c1,#9
 else
	add	x1, sp, #16+511
	and	x1, x1, #-512
 endif
	str	x0, [sp]

	ldp	x8, x9, [x2,#8*0]
	ldp	x10, x11, [x2,#8*2]

	stp	x4, x5, [x1,#8*0]
	stp	x6, x7, [x1,#8*2]
	stp	x8, x9, [x1,#8*4]
	stp	x10, x11, [x1,#8*6]


	bl	|$Lab_approximation_31_256_loaded|

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	str	x12,[x0,#8*8]

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31
	str	x12, [x0,#8*9]


	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	ldr	x8, [x1,#8*8]
	ldr	x9, [x1,#8*13]
	madd	x4, x16, x8, xzr
	madd	x4, x17, x9, x4
	str	x4, [x0,#8*4]
	asr	x5, x4, #63
	stp	x5, x5, [x0,#8*5]
	stp	x5, x5, [x0,#8*7]

	madd	x4, x12, x8, xzr
	madd	x4, x13, x9, x4
	str	x4, [x0,#8*9]
	asr	x5, x4, #63
	stp	x5, x5, [x0,#8*10]
	stp	x5, x5, [x0,#8*12]
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	adc	x22, x22, x23
	stp	x22, x22, [x0,#8*4]
	stp	x22, x22, [x0,#8*6]
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	bl	__smul_512x63_tail
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__ab_approximation_31_256

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_256_n_shift_by_31
	mov	x16, x12
	mov	x17, x13

	mov	x12, x14
	mov	x13, x15
	add	x0,x0,#8*4
	bl	__smul_256_n_shift_by_31

	add	x0,x0,#8*4
	bl	__smul_256x63
	adc	x22, x22, x23
	str	x22, [x0,#8*4]

	mov	x16, x12
	mov	x17, x13
	add	x0,x0,#8*5
	bl	__smul_256x63
	bl	__smul_512x63_tail

	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #47

	ldr	x7, [x1,#8*0]
	ldr	x11, [x1,#8*4]
	bl	__inner_loop_62_256

	mov	x16, x14
	mov	x17, x15
	ldr	x0, [sp]
	bl	__smul_256x63
	bl	__smul_512x63_tail
	ldr	x30, [x29,#__SIZEOF_POINTER__]

	smulh	x20, x7, x17
	ldp	x8, x9, [x3,#8*0]
	adc	x23, x23, x25
	ldp	x10, x11, [x3,#8*2]

	add	x20, x20, x23
	asr	x19, x20, #63

	and	x23,   x8, x19
	and	x24,   x9, x19
	adds	x4, x4, x23
	and	x25,   x10, x19
	adcs	x5, x5, x24
	and	x26,   x11, x19
	adcs	x6, x6, x25
	adcs	x7, x22,   x26
	adc	x20, x20, xzr

	neg	x19, x20
	orr	x20, x20, x19
	asr	x19, x19, #63

	and	x8, x8, x20
	and	x9, x9, x20
	and	x10, x10, x20
	and	x11, x11, x20

	eor	x8, x8, x19
	eor	x9, x9, x19
	adds	x8, x8, x19, lsr#63
	eor	x10, x10, x19
	adcs	x9, x9, xzr
	eor	x11, x11, x19
	adcs	x10, x10, xzr
	adc	x11, x11, xzr

	adds	x4, x4, x8
	adcs	x5, x5, x9
	adcs	x6, x6, x10
	stp	x4, x5, [x0,#8*4]
	adc	x7, x7, x11
	stp	x6, x7, [x0,#8*6]

	add	sp, sp, #1040
	ldp	x19, x20, [x29,#2*__SIZEOF_POINTER__]
	ldp	x21, x22, [x29,#4*__SIZEOF_POINTER__]
	ldp	x23, x24, [x29,#6*__SIZEOF_POINTER__]
	ldp	x25, x26, [x29,#8*__SIZEOF_POINTER__]
	ldr	x29, [sp],#10*__SIZEOF_POINTER__
	DCDU	3573752767
	ret
	ENDP



	ALIGN	32
|__smul_256x63| PROC
	ldp	x4, x5, [x1,#8*0+64]
	asr	x14, x16, #63
	ldp	x6, x7, [x1,#8*2+64]
	eor	x16, x16, x14
	ldr	x22, [x1,#8*4+64]

	eor	x4, x4, x14
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
	cselne	x22,x22,xzr
	mul	x6, x6, x16
	adds	x5, x5, x19
	mul	x24, x7, x16
	adcs	x6, x6, x20
	adcs	x24, x24, x21
	adc	x26, xzr, xzr
	ldp	x8, x9, [x1,#8*0+104]
	asr	x14, x17, #63
	ldp	x10, x11, [x1,#8*2+104]
	eor	x17, x17, x14
	ldr	x23, [x1,#8*4+104]

	eor	x8, x8, x14
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
	adc	x15, xzr, xzr
	mul	x8, x8, x17
	cmp	x17, #0
	mul	x9, x9, x17
	cselne	x23,x23,xzr
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
	ENDP


	ALIGN	32
|__smul_512x63_tail| PROC
	umulh	x24, x7, x16
	ldp	x5, x6, [x1,#8*18]
	adc	x26, x26, xzr
	ldr	x7, [x1,#8*20]
	and	x22, x22, x16

	umulh	x11, x11, x17

	sub	x24, x24, x22
	asr	x25, x24, #63

	eor	x5, x5, x14
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
	adc	x23, xzr, xzr

	adds	x4, x4, x24
	adcs	x5, x5, x25
	adcs	x6, x6, x25
	stp	x4, x5, [x0,#8*4]
	adcs	x22,   x22,   x25
	stp	x6, x22,   [x0,#8*6]

	ret
	ENDP


	ALIGN	32
|__smul_256_n_shift_by_31| PROC
	ldp	x4, x5, [x1,#8*0+0]
	asr	x24, x12, #63
	ldp	x6, x7, [x1,#8*2+0]
	eor	x25, x12, x24

	eor	x4, x4, x24
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
	ldp	x8, x9, [x1,#8*0+32]
	asr	x24, x13, #63
	ldp	x10, x11, [x1,#8*2+32]
	eor	x25, x13, x24

	eor	x8, x8, x24
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
	asr	x23, x8, #63
	extr	x7, x8, x7, #31

	eor	x4, x4, x23
	eor	x5, x5, x23
	adds	x4, x4, x23, lsr#63
	eor	x6, x6, x23
	adcs	x5, x5, xzr
	eor	x7, x7, x23
	adcs	x6, x6, xzr
	stp	x4, x5, [x0,#8*0]
	adc	x7, x7, xzr
	stp	x6, x7, [x0,#8*2]

	eor	x12, x12, x23
	eor	x13, x13, x23
	sub	x12, x12, x23
	sub	x13, x13, x23

	ret
	ENDP

	ALIGN	16
|__ab_approximation_31_256| PROC
	ldp	x6, x7, [x1,#8*2]
	ldp	x10, x11, [x1,#8*6]
	ldp	x4, x5, [x1,#8*0]
	ldp	x8, x9, [x1,#8*4]

|$Lab_approximation_31_256_loaded|
	orr	x19, x7, x11
	cmp	x19, #0
	cselne	x7,x7,x6
	cselne	x11,x11,x10
	cselne	x6,x6,x5
	orr	x19, x7, x11
	cselne	x10,x10,x9

	cmp	x19, #0
	cselne	x7,x7,x6
	cselne	x11,x11,x10
	cselne	x6,x6,x4
	orr	x19, x7, x11
	cselne	x10,x10,x8

	clz	x19, x19
	cmp	x19, #64
	cselne	x19,x19,xzr
	cselne	x7,x7,x6
	cselne	x11,x11,x10
	neg	x20, x19

	lslv	x7, x7, x19
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
	ENDP


	ALIGN	16
|__inner_loop_31_256| PROC
	mov	x2, #31
	mov	x13, #0x7FFFFFFF80000000
	mov	x15, #0x800000007FFFFFFF
	mov	x23,#0x7FFFFFFF7FFFFFFF

|$Loop_31_256|
	sbfx	x22, x7, #0, #1
	sub	x2, x2, #1
	and	x19, x11, x22
	sub	x20, x11, x7
	subs	x21, x7, x19
	mov	x19, x15
	cselhs	x11,x11,x7
	cselhs	x7,x21,x20
	cselhs	x15,x15,x13
	cselhs	x13,x13,x19
	lsr	x7, x7, #1
	and	x19, x15, x22
	and	x20, x23, x22
	sub	x13, x13, x19
	add	x15, x15, x15
	add	x13, x13, x20
	sub	x15, x15, x23
	cbnz	x2, |$Loop_31_256|

	mov	x23, #0x7FFFFFFF
	ubfx	x12, x13, #0, #32
	ubfx	x13, x13, #32, #32
	ubfx	x14, x15, #0, #32
	ubfx	x15, x15, #32, #32
	sub	x12, x12, x23
	sub	x13, x13, x23
	sub	x14, x14, x23
	sub	x15, x15, x23

	ret
	ENDP


	ALIGN	16
|__inner_loop_62_256| PROC
	mov	x12, #1
	mov	x13, #0
	mov	x14, #0
	mov	x15, #1

|$Loop_62_256|
	sbfx	x22, x7, #0, #1
	sub	x2, x2, #1
	and	x19, x11, x22
	sub	x20, x11, x7
	subs	x21, x7, x19
	mov	x19, x12
	cselhs	x11,x11,x7
	cselhs	x7,x21,x20
	mov	x20, x13
	cselhs	x12,x12,x14
	cselhs	x14,x14,x19
	cselhs	x13,x13,x15
	cselhs	x15,x15,x20
	lsr	x7, x7, #1
	and	x19, x14, x22
	and	x20, x15, x22
	add	x14, x14, x14
	add	x15, x15, x15
	sub	x12, x12, x19
	sub	x13, x13, x20
	cbnz	x2, |$Loop_62_256|

	ret
	ENDP
	END
