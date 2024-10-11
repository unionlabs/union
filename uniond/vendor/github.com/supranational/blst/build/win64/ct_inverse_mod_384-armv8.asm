 GBLA __SIZEOF_POINTER__
__SIZEOF_POINTER__ SETA 64/8
	AREA	|.text|,CODE,ALIGN=8,ARM64



	EXPORT	|ct_inverse_mod_383|[FUNC]
	ALIGN	32
|ct_inverse_mod_383| PROC
	DCDU	3573752639
	stp	x29, x30, [sp,#-16*__SIZEOF_POINTER__]!
	add	x29, sp, #0
	stp	x19, x20, [sp,#2*__SIZEOF_POINTER__]
	stp	x21, x22, [sp,#4*__SIZEOF_POINTER__]
	stp	x23, x24, [sp,#6*__SIZEOF_POINTER__]
	stp	x25, x26, [sp,#8*__SIZEOF_POINTER__]
	stp	x27, x28, [sp,#10*__SIZEOF_POINTER__]
	sub	sp, sp, #1056

	ldp	x22,   x4, [x1,#8*0]
	ldp	x5, x6, [x1,#8*2]
	ldp	x7, x8, [x1,#8*4]

 if :def:	__CHERI_PURE_CAPABILITY__
	add	x1,sp,#32+511
	alignd	c1,c1,#9
 else
	add	x1, sp, #32+511
	and	x1, x1, #-512
 endif
	stp	x0, x3, [sp]

	ldp	x9, x10, [x2,#8*0]
	ldp	x11, x12, [x2,#8*2]
	ldp	x13, x14, [x2,#8*4]

	stp	x22,   x4, [x1,#8*0]
	stp	x5, x6, [x1,#8*2]
	stp	x7, x8, [x1,#8*4]
	stp	x9, x10, [x1,#8*6]
	stp	x11, x12, [x1,#8*8]
	stp	x13, x14, [x1,#8*10]


	mov	x2, #62
	bl	|$Lab_approximation_62_loaded|

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_383_n_shift_by_62
	str	x15,[x0,#8*12]

	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*6
	bl	__smul_383_n_shift_by_62
	str	x15, [x0,#8*12]


	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_383_n_shift_by_62
	mov	x20, x15
	mov	x21, x16

	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*6
	bl	__smul_383_n_shift_by_62

	ldr	x7, [x1,#8*12]
	ldr	x8, [x1,#8*18]
	mul	x3, x20, x7
	smulh	x4, x20, x7
	mul	x5, x21, x8
	smulh	x6, x21, x8
	adds	x3, x3, x5
	adc	x4, x4, x6
	stp	x3, x4, [x0,#8*6]
	asr	x5, x4, #63
	stp	x5, x5, [x0,#8*8]
	stp	x5, x5, [x0,#8*10]

	mul	x3, x15, x7
	smulh	x4, x15, x7
	mul	x5, x16, x8
	smulh	x6, x16, x8
	adds	x3, x3, x5
	adc	x4, x4, x6
	stp	x3, x4, [x0,#8*12]
	asr	x5, x4, #63
	stp	x5, x5, [x0,#8*14]
	stp	x5, x5, [x0,#8*16]
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_383_n_shift_by_62
	mov	x20, x15
	mov	x21, x16

	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*6
	bl	__smul_383_n_shift_by_62

	add	x0,x0,#8*6
	bl	__smul_383x63

	mov	x20, x15
	mov	x21, x16
	add	x0,x0,#8*6
	bl	__smul_383x63
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_383_n_shift_by_62
	mov	x20, x15
	mov	x21, x16

	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*6
	bl	__smul_383_n_shift_by_62

	add	x0,x0,#8*6
	bl	__smul_383x63

	mov	x20, x15
	mov	x21, x16
	add	x0,x0,#8*6
	bl	__smul_383x63
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_383_n_shift_by_62
	mov	x20, x15
	mov	x21, x16

	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*6
	bl	__smul_383_n_shift_by_62

	add	x0,x0,#8*6
	bl	__smul_383x63

	mov	x20, x15
	mov	x21, x16
	add	x0,x0,#8*6
	bl	__smul_383x63
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_383_n_shift_by_62
	mov	x20, x15
	mov	x21, x16

	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*6
	bl	__smul_383_n_shift_by_62

	add	x0,x0,#8*6
	bl	__smul_383x63

	mov	x20, x15
	mov	x21, x16
	add	x0,x0,#8*6
	bl	__smul_383x63
	asr	x27, x27, #63
	stp	x27, x27, [x0,#8*6]
	stp	x27, x27, [x0,#8*8]
	stp	x27, x27, [x0,#8*10]
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_383_n_shift_by_62
	mov	x20, x15
	mov	x21, x16

	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*6
	bl	__smul_383_n_shift_by_62

	add	x0,x0,#8*6
	bl	__smul_383x63

	mov	x20, x15
	mov	x21, x16
	add	x0,x0,#8*6
	bl	__smul_383x63
	bl	__smul_767x63_tail
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_383_n_shift_by_62
	mov	x20, x15
	mov	x21, x16

	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*6
	bl	__smul_383_n_shift_by_62

	add	x0,x0,#8*6
	bl	__smul_383x63

	mov	x20, x15
	mov	x21, x16
	add	x0,x0,#8*6
	bl	__smul_383x63
	bl	__smul_767x63_tail
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_383_n_shift_by_62
	mov	x20, x15
	mov	x21, x16

	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*6
	bl	__smul_383_n_shift_by_62

	add	x0,x0,#8*6
	bl	__smul_383x63

	mov	x20, x15
	mov	x21, x16
	add	x0,x0,#8*6
	bl	__smul_383x63
	bl	__smul_767x63_tail
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_383_n_shift_by_62
	mov	x20, x15
	mov	x21, x16

	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*6
	bl	__smul_383_n_shift_by_62

	add	x0,x0,#8*6
	bl	__smul_383x63

	mov	x20, x15
	mov	x21, x16
	add	x0,x0,#8*6
	bl	__smul_383x63
	bl	__smul_767x63_tail
	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #62
	bl	__ab_approximation_62

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	bl	__smul_383_n_shift_by_62
	mov	x20, x15
	mov	x21, x16

	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*6
	bl	__smul_383_n_shift_by_62

	add	x0,x0,#8*6
	bl	__smul_383x63

	mov	x20, x15
	mov	x21, x16
	add	x0,x0,#8*6
	bl	__smul_383x63
	bl	__smul_767x63_tail

	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #62

	ldp	x3, x8, [x1,#8*0]
	ldp	x9, x14, [x1,#8*6]
	bl	__inner_loop_62

	eor	x0, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	str	x3, [x0,#8*0]
	str	x9, [x0,#8*6]

	mov	x20, x15
	mov	x21, x16
	mov	x15, x17
	mov	x16, x19
	add	x0,x0,#8*12
	bl	__smul_383x63

	mov	x20, x15
	mov	x21, x16
	add	x0,x0,#8*6
	bl	__smul_383x63
	bl	__smul_767x63_tail


	eor	x1, x1, #256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	mov	x2, #22

	ldr	x3, [x1,#8*0]
	eor	x8, x8, x8
	ldr	x9, [x1,#8*6]
	eor	x14, x14, x14
	bl	__inner_loop_62

	mov	x20, x17
	mov	x21, x19
	ldp	x0, x15, [sp]
	bl	__smul_383x63
	bl	__smul_767x63_tail
	ldr	x30, [x29,#__SIZEOF_POINTER__]

	asr	x22, x8, #63
	ldp	x9, x10, [x15,#8*0]
	ldp	x11, x12, [x15,#8*2]
	ldp	x13, x14, [x15,#8*4]

	and	x9, x9, x22
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

	add	sp, sp, #1056
	ldp	x19, x20, [x29,#2*__SIZEOF_POINTER__]
	ldp	x21, x22, [x29,#4*__SIZEOF_POINTER__]
	ldp	x23, x24, [x29,#6*__SIZEOF_POINTER__]
	ldp	x25, x26, [x29,#8*__SIZEOF_POINTER__]
	ldp	x27, x28, [x29,#10*__SIZEOF_POINTER__]
	ldr	x29, [sp],#16*__SIZEOF_POINTER__
	DCDU	3573752767
	ret
	ENDP




	ALIGN	32
|__smul_383x63| PROC
	ldp	x3, x4, [x1,#8*0+96]
	asr	x17, x20, #63
	ldp	x5, x6, [x1,#8*2+96]
	eor	x20, x20, x17
	ldp	x7, x8, [x1,#8*4+96]

	eor	x3, x3, x17
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
	ldp	x9, x10, [x1,#8*0+144]
	asr	x17, x21, #63
	ldp	x11, x12, [x1,#8*2+144]
	eor	x21, x21, x17
	ldp	x13, x14, [x1,#8*4+144]

	eor	x9, x9, x17
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
	adc	x19, xzr, xzr
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
	adc	x28,   x2,   xzr

	ret
	ENDP


	ALIGN	32
|__smul_767x63_tail| PROC
	smulh	x27,   x8, x20
	ldp	x3, x4, [x1,#8*24]
	umulh	x14,x14, x21
	ldp	x5, x6, [x1,#8*26]
	ldp	x7, x8, [x1,#8*28]

	eor	x3, x3, x17
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
	ENDP


	ALIGN	32
|__smul_383_n_shift_by_62| PROC
	ldp	x3, x4, [x1,#8*0+0]
	asr	x28, x15, #63
	ldp	x5, x6, [x1,#8*2+0]
	eor	x2, x15, x28
	ldp	x7, x8, [x1,#8*4+0]

	eor	x3, x3, x28
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
	ldp	x9, x10, [x1,#8*0+48]
	asr	x28, x16, #63
	ldp	x11, x12, [x1,#8*2+48]
	eor	x2, x16, x28
	ldp	x13, x14, [x1,#8*4+48]

	eor	x9, x9, x28
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
	ENDP

	ALIGN	16
|__ab_approximation_62| PROC
	ldp	x7, x8, [x1,#8*4]
	ldp	x13, x14, [x1,#8*10]
	ldp	x5, x6, [x1,#8*2]
	ldp	x11, x12, [x1,#8*8]

|$Lab_approximation_62_loaded|
	orr	x22, x8, x14
	cmp	x22, #0
	cselne	x8,x8,x7
	cselne	x14,x14,x13
	cselne	x7,x7,x6
	orr	x22, x8, x14
	cselne	x13,x13,x12

	ldp	x3, x4, [x1,#8*0]
	ldp	x9, x10, [x1,#8*6]

	cmp	x22, #0
	cselne	x8,x8,x7
	cselne	x14,x14,x13
	cselne	x7,x7,x5
	orr	x22, x8, x14
	cselne	x13,x13,x11

	cmp	x22, #0
	cselne	x8,x8,x7
	cselne	x14,x14,x13
	cselne	x7,x7,x4
	orr	x22, x8, x14
	cselne	x13,x13,x10

	clz	x22, x22
	cmp	x22, #64
	cselne	x22,x22,xzr
	cselne	x8,x8,x7
	cselne	x14,x14,x13
	neg	x23, x22

	lslv	x8, x8, x22
	lslv	x14, x14, x22
	lsrv	x7, x7, x23
	lsrv	x13, x13, x23
	and	x7, x7, x23, asr#6
	and	x13, x13, x23, asr#6
	orr	x8, x8, x7
	orr	x14, x14, x13

	b	__inner_loop_62
	ret
	ENDP

	ALIGN	16
|__inner_loop_62| PROC
	mov	x15, #1
	mov	x16, #0
	mov	x17, #0
	mov	x19, #1

|$Loop_62|
	sbfx	x28, x3, #0, #1
	sub	x2, x2, #1
	subs	x24, x9, x3
	and	x22, x9, x28
	sbc	x25, x14, x8
	and	x23, x14, x28
	subs	x26, x3, x22
	mov	x22, x15
	sbcs	x27, x8, x23
	mov	x23, x16
	cselhs	x9,x9,x3
	cselhs	x14,x14,x8
	cselhs	x3,x26,x24
	cselhs	x8,x27,x25
	cselhs	x15,x15,x17
	cselhs	x17,x17,x22
	cselhs	x16,x16,x19
	cselhs	x19,x19,x23
	extr	x3, x8, x3, #1
	lsr	x8, x8, #1
	and	x22, x17, x28
	and	x23, x19, x28
	add	x17, x17, x17
	add	x19, x19, x19
	sub	x15, x15, x22
	sub	x16, x16, x23
	cbnz	x2, |$Loop_62|

	ret
	ENDP
	END
