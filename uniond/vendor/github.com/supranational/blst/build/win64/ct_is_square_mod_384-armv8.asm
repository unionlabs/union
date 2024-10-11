 GBLA __SIZEOF_POINTER__
__SIZEOF_POINTER__ SETA 64/8
	AREA	|.text|,CODE,ALIGN=8,ARM64



	EXPORT	|ct_is_square_mod_384|[FUNC]
	ALIGN	32
|ct_is_square_mod_384| PROC
	DCDU	3573752639
	stp	x29, x30, [sp,#-16*__SIZEOF_POINTER__]!
	add	x29, sp, #0
	stp	x19, x20, [sp,#2*__SIZEOF_POINTER__]
	stp	x21, x22, [sp,#4*__SIZEOF_POINTER__]
	stp	x23, x24, [sp,#6*__SIZEOF_POINTER__]
	stp	x25, x26, [sp,#8*__SIZEOF_POINTER__]
	stp	x27, x28, [sp,#10*__SIZEOF_POINTER__]
	sub	sp, sp, #512

	ldp	x3, x4, [x0,#8*0]
	ldp	x5, x6, [x0,#8*2]
	ldp	x7, x8, [x0,#8*4]

	add	x0, sp, #255
	and	x0, x0, #-256
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif

	ldp	x9, x10, [x1,#8*0]
	ldp	x11, x12, [x1,#8*2]
	ldp	x13, x14, [x1,#8*4]

	stp	x3, x4, [x0,#8*6]
	stp	x5, x6, [x0,#8*8]
	stp	x7, x8, [x0,#8*10]
	stp	x9, x10, [x0,#8*0]
	stp	x11, x12, [x0,#8*2]
	stp	x13, x14, [x0,#8*4]

	eor	x2, x2, x2
	mov	x15, #24
	b	|$Loop_is_square|

	ALIGN	16
|$Loop_is_square|
	bl	__ab_approximation_30
	sub	x15, x15, #1

	eor	x1, x0, #128
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c1,csp,x1
 endif
	bl	__smul_384_n_shift_by_30

	mov	x19, x16
	mov	x20, x17
	add	x1,x1,#8*6
	bl	__smul_384_n_shift_by_30

	ldp	x9, x10, [x1,#-8*6]
	eor	x0, x0, #128
 if :def:	__CHERI_PURE_CAPABILITY__
	scvalue	c0,csp,x0
 endif
	and	x27, x27, x9
	add	x2, x2, x27, lsr#1

	cbnz	x15, |$Loop_is_square|





	mov	x15, #48
	bl	__inner_loop_48
	ldr	x30, [x29,#__SIZEOF_POINTER__]

	and	x0, x2, #1
	eor	x0, x0, #1

	add	sp, sp, #512
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
|__smul_384_n_shift_by_30| PROC
	ldp	x3, x4, [x0,#8*0+0]
	asr	x27, x20, #63
	ldp	x5, x6, [x0,#8*2+0]
	eor	x20, x20, x27
	ldp	x7, x8, [x0,#8*4+0]

	eor	x3, x3, x27
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
	ldp	x9, x10, [x0,#8*0+48]
	asr	x27, x19, #63
	ldp	x11, x12, [x0,#8*2+48]
	eor	x19, x19, x27
	ldp	x13, x14, [x0,#8*4+48]

	eor	x9, x9, x27
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
	ENDP

	ALIGN	16
|__ab_approximation_30| PROC
	ldp	x13, x14, [x0,#8*4]
	ldp	x11, x12, [x0,#8*2]

	orr	x21, x8, x14
	cmp	x21, #0
	cselne	x8,x8,x7
	cselne	x14,x14,x13
	cselne	x7,x7,x6
	orr	x21, x8, x14
	cselne	x13,x13,x12

	cmp	x21, #0
	cselne	x8,x8,x7
	cselne	x14,x14,x13
	cselne	x7,x7,x5
	orr	x21, x8, x14
	cselne	x13,x13,x11

	cmp	x21, #0
	cselne	x8,x8,x7
	cselne	x14,x14,x13
	cselne	x7,x7,x4
	orr	x21, x8, x14
	cselne	x13,x13,x10

	cmp	x21, #0
	cselne	x8,x8,x7
	cselne	x14,x14,x13
	cselne	x7,x7,x3
	orr	x21, x8, x14
	cselne	x13,x13,x9

	clz	x21, x21
	cmp	x21, #64
	cselne	x21,x21,xzr
	cselne	x8,x8,x7
	cselne	x14,x14,x13
	neg	x22, x21

	lslv	x8, x8, x21
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
	ENDP


	ALIGN	16
|__inner_loop_30| PROC
	mov	x28, #30
	mov	x17, #0x7FFFFFFF80000000
	mov	x20, #0x800000007FFFFFFF
	mov	x27,#0x7FFFFFFF7FFFFFFF

|$Loop_30|
	sbfx	x24, x8, #0, #1
	and	x25, x8, x14
	sub	x28, x28, #1
	and	x21, x14, x24

	sub	x22, x14, x8
	subs	x23, x8, x21
	add	x25, x2, x25, lsr#1
	mov	x21, x20
	cselhs	x14,x14,x8
	cselhs	x8,x23,x22
	cselhs	x20,x20,x17
	cselhs	x17,x17,x21
	cselhs	x2,x2,x25
	lsr	x8, x8, #1
	and	x21, x20, x24
	and	x22, x27, x24
	add	x23, x14, #2
	sub	x17, x17, x21
	add	x20, x20, x20
	add	x2, x2, x23, lsr#2
	add	x17, x17, x22
	sub	x20, x20, x27

	cbnz	x28, |$Loop_30|

	mov	x27, #0x7FFFFFFF
	ubfx	x16, x17, #0, #32
	ubfx	x17, x17, #32, #32
	ubfx	x19, x20, #0, #32
	ubfx	x20, x20, #32, #32
	sub	x16, x16, x27
	sub	x17, x17, x27
	sub	x19, x19, x27
	sub	x20, x20, x27

	ret
	ENDP

	ALIGN	16
|__inner_loop_48| PROC
|$Loop_48|
	sbfx	x24, x3, #0, #1
	and	x25, x3, x9
	sub	x15, x15, #1
	and	x21, x9, x24
	sub	x22, x9, x3
	subs	x23, x3, x21
	add	x25, x2, x25, lsr#1
	cselhs	x9,x9,x3
	cselhs	x3,x23,x22
	cselhs	x2,x2,x25
	add	x23, x9, #2
	lsr	x3, x3, #1
	add	x2, x2, x23, lsr#2

	cbnz	x15, |$Loop_48|

	ret
	ENDP
	END
