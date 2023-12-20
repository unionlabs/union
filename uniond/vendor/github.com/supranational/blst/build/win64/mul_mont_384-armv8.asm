	AREA	|.text|,CODE,ALIGN=8,ARM64


	EXPORT	|add_mod_384x384|[FUNC]
	ALIGN	32
|add_mod_384x384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-64]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]

	ldp	x5,x6,[x3]
	ldp	x7,x8,[x3,#16]
	ldp	x9,x10,[x3,#32]

	bl	__add_mod_384x384
	ldr	x30,[x29,#8]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldr	x29,[sp],#64
	DCDU	3573752767
	ret
	ENDP


	ALIGN	32
|__add_mod_384x384| PROC
	ldp	x11,  x12,  [x1]
	ldp	x19,x20,[x2]
	ldp	x13,  x14,  [x1,#16]
	adds	x11,x11,x19
	ldp	x21,x22,[x2,#16]
	adcs	x12,x12,x20
	ldp	x15,  x16,  [x1,#32]
	adcs	x13,x13,x21
	ldp	x23,x24,[x2,#32]
	adcs	x14,x14,x22
	stp	x11,  x12,  [x0]
	adcs	x15,x15,x23
	ldp	x11,  x12,  [x1,#48]
	adcs	x16,x16,x24

	ldp	x19,x20,[x2,#48]
	stp	x13,  x14,  [x0,#16]
	ldp	x13,  x14,  [x1,#64]
	ldp	x21,x22,[x2,#64]

	adcs	x11,x11,x19
	stp	x15,  x16,  [x0,#32]
	adcs	x12,x12,x20
	ldp	x15,  x16,  [x1,#80]
	adcs	x13,x13,x21
	ldp	x23,x24,[x2,#80]
	adcs	x14,x14,x22
	adcs	x15,x15,x23
	adcs	x16,x16,x24
	adc	x17,xzr,xzr

	subs	x19,x11,x5
	sbcs	x20,x12,x6
	sbcs	x21,x13,x7
	sbcs	x22,x14,x8
	sbcs	x23,x15,x9
	sbcs	x24,x16,x10
	sbcs	xzr,x17,xzr

	csello	x11,x11,x19
	csello	x12,x12,x20
	csello	x13,x13,x21
	csello	x14,x14,x22
	stp	x11,x12,[x0,#48]
	csello	x15,x15,x23
	stp	x13,x14,[x0,#64]
	csello	x16,x16,x24
	stp	x15,x16,[x0,#80]

	ret
	ENDP


	EXPORT	|sub_mod_384x384|[FUNC]
	ALIGN	32
|sub_mod_384x384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-64]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]

	ldp	x5,x6,[x3]
	ldp	x7,x8,[x3,#16]
	ldp	x9,x10,[x3,#32]

	bl	__sub_mod_384x384
	ldr	x30,[x29,#8]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldr	x29,[sp],#64
	DCDU	3573752767
	ret
	ENDP


	ALIGN	32
|__sub_mod_384x384| PROC
	ldp	x11,  x12,  [x1]
	ldp	x19,x20,[x2]
	ldp	x13,  x14,  [x1,#16]
	subs	x11,x11,x19
	ldp	x21,x22,[x2,#16]
	sbcs	x12,x12,x20
	ldp	x15,  x16,  [x1,#32]
	sbcs	x13,x13,x21
	ldp	x23,x24,[x2,#32]
	sbcs	x14,x14,x22
	stp	x11,  x12,  [x0]
	sbcs	x15,x15,x23
	ldp	x11,  x12,  [x1,#48]
	sbcs	x16,x16,x24

	ldp	x19,x20,[x2,#48]
	stp	x13,  x14,  [x0,#16]
	ldp	x13,  x14,  [x1,#64]
	ldp	x21,x22,[x2,#64]

	sbcs	x11,x11,x19
	stp	x15,  x16,  [x0,#32]
	sbcs	x12,x12,x20
	ldp	x15,  x16,  [x1,#80]
	sbcs	x13,x13,x21
	ldp	x23,x24,[x2,#80]
	sbcs	x14,x14,x22
	sbcs	x15,x15,x23
	sbcs	x16,x16,x24
	sbc	x17,xzr,xzr

	and	x19,x5,x17
	and	x20,x6,x17
	adds	x11,x11,x19
	and	x21,x7,x17
	adcs	x12,x12,x20
	and	x22,x8,x17
	adcs	x13,x13,x21
	and	x23,x9,x17
	adcs	x14,x14,x22
	and	x24,x10,x17
	adcs	x15,x15,x23
	stp	x11,x12,[x0,#48]
	adc	x16,x16,x24
	stp	x13,x14,[x0,#64]
	stp	x15,x16,[x0,#80]

	ret
	ENDP


	ALIGN	32
|__add_mod_384| PROC
	ldp	x11,  x12,  [x1]
	ldp	x19,x20,[x2]
	ldp	x13,  x14,  [x1,#16]
	adds	x11,x11,x19
	ldp	x21,x22,[x2,#16]
	adcs	x12,x12,x20
	ldp	x15,  x16,  [x1,#32]
	adcs	x13,x13,x21
	ldp	x23,x24,[x2,#32]
	adcs	x14,x14,x22
	adcs	x15,x15,x23
	adcs	x16,x16,x24
	adc	x17,xzr,xzr

	subs	x19,x11,x5
	sbcs	x20,x12,x6
	sbcs	x21,x13,x7
	sbcs	x22,x14,x8
	sbcs	x23,x15,x9
	sbcs	x24,x16,x10
	sbcs	xzr,x17,xzr

	csello	x11,x11,x19
	csello	x12,x12,x20
	csello	x13,x13,x21
	csello	x14,x14,x22
	csello	x15,x15,x23
	stp	x11,x12,[x0]
	csello	x16,x16,x24
	stp	x13,x14,[x0,#16]
	stp	x15,x16,[x0,#32]

	ret
	ENDP


	ALIGN	32
|__sub_mod_384| PROC
	ldp	x11,  x12,  [x1]
	ldp	x19,x20,[x2]
	ldp	x13,  x14,  [x1,#16]
	subs	x11,x11,x19
	ldp	x21,x22,[x2,#16]
	sbcs	x12,x12,x20
	ldp	x15,  x16,  [x1,#32]
	sbcs	x13,x13,x21
	ldp	x23,x24,[x2,#32]
	sbcs	x14,x14,x22
	sbcs	x15,x15,x23
	sbcs	x16,x16,x24
	sbc	x17,xzr,xzr

	and	x19,x5,x17
	and	x20,x6,x17
	adds	x11,x11,x19
	and	x21,x7,x17
	adcs	x12,x12,x20
	and	x22,x8,x17
	adcs	x13,x13,x21
	and	x23,x9,x17
	adcs	x14,x14,x22
	and	x24,x10,x17
	adcs	x15,x15,x23
	stp	x11,x12,[x0]
	adc	x16,x16,x24
	stp	x13,x14,[x0,#16]
	stp	x15,x16,[x0,#32]

	ret
	ENDP



	EXPORT	|mul_mont_384x|[FUNC]
	ALIGN	32
|mul_mont_384x| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]
	sub	sp,sp,#288		// space for 3 768-bit vectors

	mov	x26,x0		// save r_ptr
	mov	x27,x1		// save b_ptr
	mov	x28,x2		// save b_ptr

	sub	x0,sp,#0		// mul_384(t0, a->re, b->re)
	bl	__mul_384

	add	x1,x1,#48	// mul_384(t1, a->im, b->im)
	add	x2,x2,#48
	add	x0,sp,#96
	bl	__mul_384

	ldp	x5,x6,[x3]
	ldp	x7,x8,[x3,#16]
	ldp	x9,x10,[x3,#32]

	sub	x2,x1,#48
	add	x0,sp,#240
	bl	__add_mod_384

	add	x1,x28,#0
	add	x2,x28,#48
	add	x0,sp,#192		// t2
	bl	__add_mod_384

	add	x1,x0,#0
	add	x2,x0,#48
	bl	__mul_384		// mul_384(t2, a->re+a->im, b->re+b->im)

	ldp	x5,x6,[x3]
	ldp	x7,x8,[x3,#16]
	ldp	x9,x10,[x3,#32]

	mov	x1,x0
	add	x2,sp,#0
	bl	__sub_mod_384x384

	add	x2,sp,#96
	bl	__sub_mod_384x384	// t2 = t2-t0-t1

	add	x1,sp,#0
	add	x2,sp,#96
	add	x0,sp,#0
	bl	__sub_mod_384x384	// t0 = t0-t1

	add	x1,sp,#0		// ret->re = redc(t0)
	add	x0,x26,#0
	bl	__mul_by_1_mont_384
	bl	__redc_tail_mont_384

	add	x1,sp,#192		// ret->im = redc(t2)
	add	x0,x0,#48
	bl	__mul_by_1_mont_384
	bl	__redc_tail_mont_384
	ldr	x30,[x29,#8]

	add	sp,sp,#288
	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|sqr_mont_384x|[FUNC]
	ALIGN	32
|sqr_mont_384x| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]
	stp	x3,x0,[sp,#96]	// __mul_mont_384 wants them there
	sub	sp,sp,#96		// space for 2 384-bit vectors
	mov	x4,x3		// adjust for missing b_ptr

	ldp	x5,x6,[x2]
	ldp	x7,x8,[x2,#16]
	ldp	x9,x10,[x2,#32]

	add	x2,x1,#48
	add	x0,sp,#0
	bl	__add_mod_384		// t0 = a->re + a->im

	add	x0,sp,#48
	bl	__sub_mod_384		// t1 = a->re - a->im

	ldp	x11,x12,[x1]
	ldr	x17,        [x2]
	ldp	x13,x14,[x1,#16]
	ldp	x15,x16,[x1,#32]

	bl	__mul_mont_384		// mul_mont_384(ret->im, a->re, a->im)

	adds	x11,x11,x11	// add with itself
	adcs	x12,x12,x12
	adcs	x13,x13,x13
	adcs	x14,x14,x14
	adcs	x15,x15,x15
	adcs	x16,x16,x16
	adc	x25,xzr,xzr

	subs	x19,x11,x5
	sbcs	x20,x12,x6
	sbcs	x21,x13,x7
	sbcs	x22,x14,x8
	sbcs	x23,x15,x9
	sbcs	x24,x16,x10
	sbcs	xzr,x25,xzr

	csello	x19,x11,x19
	csello	x20,x12,x20
	csello	x21,x13,x21
	ldp	x11,x12,[sp]
	csello	x22,x14,x22
	ldr	x17,        [sp,#48]
	csello	x23,x15,x23
	ldp	x13,x14,[sp,#16]
	csello	x24,x16,x24
	ldp	x15,x16,[sp,#32]

	stp	x19,x20,[x2,#48]
	stp	x21,x22,[x2,#64]
	stp	x23,x24,[x2,#80]

	add	x2,sp,#48
	bl	__mul_mont_384		// mul_mont_384(ret->re, t0, t1)
	ldr	x30,[x29,#8]

	stp	x11,x12,[x2]
	stp	x13,x14,[x2,#16]
	stp	x15,x16,[x2,#32]

	add	sp,sp,#96
	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|mul_mont_384|[FUNC]
	ALIGN	32
|mul_mont_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]
	stp	x4,x0,[sp,#96]	// __mul_mont_384 wants them there

	ldp	x11,x12,[x1]
	ldr	x17,        [x2]
	ldp	x13,x14,[x1,#16]
	ldp	x15,x16,[x1,#32]

	ldp	x5,x6,[x3]
	ldp	x7,x8,[x3,#16]
	ldp	x9,x10,[x3,#32]

	bl	__mul_mont_384
	ldr	x30,[x29,#8]

	stp	x11,x12,[x2]
	stp	x13,x14,[x2,#16]
	stp	x15,x16,[x2,#32]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP


	ALIGN	32
|__mul_mont_384| PROC
	mul	x19,x11,x17
	mul	x20,x12,x17
	mul	x21,x13,x17
	mul	x22,x14,x17
	mul	x23,x15,x17
	mul	x24,x16,x17
	mul	x4,x4,x19

	umulh	x26,x11,x17
	umulh	x27,x12,x17
	umulh	x28,x13,x17
	umulh	x0,x14,x17
	umulh	x1,x15,x17
	umulh	x3,x16,x17

	adds	x20,x20,x26
	// mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adc	x25,xzr,    x3
	mul	x3,x10,x4
	mov	x17,xzr
	subs	xzr,x19,#1		// adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adcs	x25,x25,xzr
	adc	x4,x17,xzr
	ldr	x17,[x2,8*1]

	adds	x19,x20,x26
	mul	x26,x11,x17
	adcs	x20,x21,x27
	mul	x27,x12,x17
	adcs	x21,x22,x28
	mul	x28,x13,x17
	adcs	x22,x23,x0
	mul	x0,x14,x17
	adcs	x23,x24,x1
	mul	x1,x15,x17
	adcs	x24,x25,x3
	mul	x3,x16,x17
	adc	x25,x4,xzr
	ldr	x4,[x29,#96]

	adds	x19,x19,x26
	umulh	x26,x11,x17
	adcs	x20,x20,x27
	umulh	x27,x12,x17
	adcs	x21,x21,x28
	mul	x4,x4,x19
	umulh	x28,x13,x17
	adcs	x22,x22,x0
	umulh	x0,x14,x17
	adcs	x23,x23,x1
	umulh	x1,x15,x17
	adcs	x24,x24,x3
	umulh	x3,x16,x17
	adcs	x25,x25,xzr
	adc	x17,xzr,xzr

	adds	x20,x20,x26
	// mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adcs	x25,x25,x3
	mul	x3,x10,x4
	adc	x17,x17,xzr
	subs	xzr,x19,#1		// adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adcs	x25,x25,xzr
	adc	x4,x17,xzr
	ldr	x17,[x2,8*2]

	adds	x19,x20,x26
	mul	x26,x11,x17
	adcs	x20,x21,x27
	mul	x27,x12,x17
	adcs	x21,x22,x28
	mul	x28,x13,x17
	adcs	x22,x23,x0
	mul	x0,x14,x17
	adcs	x23,x24,x1
	mul	x1,x15,x17
	adcs	x24,x25,x3
	mul	x3,x16,x17
	adc	x25,x4,xzr
	ldr	x4,[x29,#96]

	adds	x19,x19,x26
	umulh	x26,x11,x17
	adcs	x20,x20,x27
	umulh	x27,x12,x17
	adcs	x21,x21,x28
	mul	x4,x4,x19
	umulh	x28,x13,x17
	adcs	x22,x22,x0
	umulh	x0,x14,x17
	adcs	x23,x23,x1
	umulh	x1,x15,x17
	adcs	x24,x24,x3
	umulh	x3,x16,x17
	adcs	x25,x25,xzr
	adc	x17,xzr,xzr

	adds	x20,x20,x26
	// mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adcs	x25,x25,x3
	mul	x3,x10,x4
	adc	x17,x17,xzr
	subs	xzr,x19,#1		// adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adcs	x25,x25,xzr
	adc	x4,x17,xzr
	ldr	x17,[x2,8*3]

	adds	x19,x20,x26
	mul	x26,x11,x17
	adcs	x20,x21,x27
	mul	x27,x12,x17
	adcs	x21,x22,x28
	mul	x28,x13,x17
	adcs	x22,x23,x0
	mul	x0,x14,x17
	adcs	x23,x24,x1
	mul	x1,x15,x17
	adcs	x24,x25,x3
	mul	x3,x16,x17
	adc	x25,x4,xzr
	ldr	x4,[x29,#96]

	adds	x19,x19,x26
	umulh	x26,x11,x17
	adcs	x20,x20,x27
	umulh	x27,x12,x17
	adcs	x21,x21,x28
	mul	x4,x4,x19
	umulh	x28,x13,x17
	adcs	x22,x22,x0
	umulh	x0,x14,x17
	adcs	x23,x23,x1
	umulh	x1,x15,x17
	adcs	x24,x24,x3
	umulh	x3,x16,x17
	adcs	x25,x25,xzr
	adc	x17,xzr,xzr

	adds	x20,x20,x26
	// mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adcs	x25,x25,x3
	mul	x3,x10,x4
	adc	x17,x17,xzr
	subs	xzr,x19,#1		// adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adcs	x25,x25,xzr
	adc	x4,x17,xzr
	ldr	x17,[x2,8*4]

	adds	x19,x20,x26
	mul	x26,x11,x17
	adcs	x20,x21,x27
	mul	x27,x12,x17
	adcs	x21,x22,x28
	mul	x28,x13,x17
	adcs	x22,x23,x0
	mul	x0,x14,x17
	adcs	x23,x24,x1
	mul	x1,x15,x17
	adcs	x24,x25,x3
	mul	x3,x16,x17
	adc	x25,x4,xzr
	ldr	x4,[x29,#96]

	adds	x19,x19,x26
	umulh	x26,x11,x17
	adcs	x20,x20,x27
	umulh	x27,x12,x17
	adcs	x21,x21,x28
	mul	x4,x4,x19
	umulh	x28,x13,x17
	adcs	x22,x22,x0
	umulh	x0,x14,x17
	adcs	x23,x23,x1
	umulh	x1,x15,x17
	adcs	x24,x24,x3
	umulh	x3,x16,x17
	adcs	x25,x25,xzr
	adc	x17,xzr,xzr

	adds	x20,x20,x26
	// mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adcs	x25,x25,x3
	mul	x3,x10,x4
	adc	x17,x17,xzr
	subs	xzr,x19,#1		// adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adcs	x25,x25,xzr
	adc	x4,x17,xzr
	ldr	x17,[x2,8*5]

	adds	x19,x20,x26
	mul	x26,x11,x17
	adcs	x20,x21,x27
	mul	x27,x12,x17
	adcs	x21,x22,x28
	mul	x28,x13,x17
	adcs	x22,x23,x0
	mul	x0,x14,x17
	adcs	x23,x24,x1
	mul	x1,x15,x17
	adcs	x24,x25,x3
	mul	x3,x16,x17
	adc	x25,x4,xzr
	ldr	x4,[x29,#96]

	adds	x19,x19,x26
	umulh	x26,x11,x17
	adcs	x20,x20,x27
	umulh	x27,x12,x17
	adcs	x21,x21,x28
	mul	x4,x4,x19
	umulh	x28,x13,x17
	adcs	x22,x22,x0
	umulh	x0,x14,x17
	adcs	x23,x23,x1
	umulh	x1,x15,x17
	adcs	x24,x24,x3
	umulh	x3,x16,x17
	adcs	x25,x25,xzr
	adc	x17,xzr,xzr

	adds	x20,x20,x26
	// mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adcs	x25,x25,x3
	mul	x3,x10,x4
	adc	x17,x17,xzr
	subs	xzr,x19,#1		// adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adcs	x25,x25,xzr
	ldp	x4,x2,[x29,#96]	// pull r_ptr
	adc	x17,x17,xzr

	adds	x19,x20,x26
	adcs	x20,x21,x27
	adcs	x21,x22,x28
	adcs	x22,x23,x0
	adcs	x23,x24,x1
	adcs	x24,x25,x3
	adc	x25,x17,xzr

	subs	x26,x19,x5
	sbcs	x27,x20,x6
	sbcs	x28,x21,x7
	sbcs	x0,x22,x8
	sbcs	x1,x23,x9
	sbcs	x3,x24,x10
	sbcs	xzr,    x25,xzr

	csello	x11,x19,x26
	csello	x12,x20,x27
	csello	x13,x21,x28
	csello	x14,x22,x0
	csello	x15,x23,x1
	csello	x16,x24,x3
	ret
	ENDP



	EXPORT	|sqr_mont_384|[FUNC]
	ALIGN	32
|sqr_mont_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]
	sub	sp,sp,#96		// space for 768-bit vector
	mov	x4,x3		// adjust for missing b_ptr

	mov	x3,x0		// save r_ptr
	mov	x0,sp

	ldp	x11,x12,[x1]
	ldp	x13,x14,[x1,#16]
	ldp	x15,x16,[x1,#32]

	bl	__sqr_384

	ldp	x5,x6,[x2]
	ldp	x7,x8,[x2,#16]
	ldp	x9,x10,[x2,#32]

	mov	x1,sp
	mov	x0,x3		// restore r_ptr
	bl	__mul_by_1_mont_384
	bl	__redc_tail_mont_384
	ldr	x30,[x29,#8]

	add	sp,sp,#96
	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|sqr_n_mul_mont_383|[FUNC]
	ALIGN	32
|sqr_n_mul_mont_383| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]
	stp	x4,x0,[sp,#96]	// __mul_mont_384 wants them there
	sub	sp,sp,#96		// space for 768-bit vector
	mov	x17,x5			// save b_ptr

	ldp	x11,x12,[x1]
	ldp	x13,x14,[x1,#16]
	ldp	x15,x16,[x1,#32]
	mov	x0,sp
|$Loop_sqr_383|
	bl	__sqr_384
	sub	x2,x2,#1	// counter

	ldp	x5,x6,[x3]
	ldp	x7,x8,[x3,#16]
	ldp	x9,x10,[x3,#32]

	mov	x1,sp
	bl	__mul_by_1_mont_384

	ldp	x19,x20,[x1,#48]
	ldp	x21,x22,[x1,#64]
	ldp	x23,x24,[x1,#80]

	adds	x11,x11,x19	// just accumulate upper half
	adcs	x12,x12,x20
	adcs	x13,x13,x21
	adcs	x14,x14,x22
	adcs	x15,x15,x23
	adc	x16,x16,x24

	cbnz	x2,|$Loop_sqr_383|

	mov	x2,x17
	ldr	x17,[x17]
	bl	__mul_mont_384
	ldr	x30,[x29,#8]

	stp	x11,x12,[x2]
	stp	x13,x14,[x2,#16]
	stp	x15,x16,[x2,#32]

	add	sp,sp,#96
	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP

	ALIGN	32
|__sqr_384| PROC
	mul	x19,x12,x11
	mul	x20,x13,x11
	mul	x21,x14,x11
	mul	x22,x15,x11
	mul	x23,x16,x11

	umulh	x6,x12,x11
	umulh	x7,x13,x11
	umulh	x8,x14,x11
	umulh	x9,x15,x11
	adds	x20,x20,x6
	umulh	x10,x16,x11
	adcs	x21,x21,x7
	mul	x7,x13,x12
	adcs	x22,x22,x8
	mul	x8,x14,x12
	adcs	x23,x23,x9
	mul	x9,x15,x12
	adc	x24,xzr,    x10
	mul	x10,x16,x12

	adds	x21,x21,x7
	umulh	x7,x13,x12
	adcs	x22,x22,x8
	umulh	x8,x14,x12
	adcs	x23,x23,x9
	umulh	x9,x15,x12
	adcs	x24,x24,x10
	umulh	x10,x16,x12
	adc	x25,xzr,xzr

	mul	x5,x11,x11
	adds	x22,x22,x7
	umulh	x11,  x11,x11
	adcs	x23,x23,x8
	mul	x8,x14,x13
	adcs	x24,x24,x9
	mul	x9,x15,x13
	adc	x25,x25,x10
	mul	x10,x16,x13

	adds	x23,x23,x8
	umulh	x8,x14,x13
	adcs	x24,x24,x9
	umulh	x9,x15,x13
	adcs	x25,x25,x10
	umulh	x10,x16,x13
	adc	x26,xzr,xzr

	mul	x6,x12,x12
	adds	x24,x24,x8
	umulh	x12,  x12,x12
	adcs	x25,x25,x9
	mul	x9,x15,x14
	adc	x26,x26,x10
	mul	x10,x16,x14

	adds	x25,x25,x9
	umulh	x9,x15,x14
	adcs	x26,x26,x10
	umulh	x10,x16,x14
	adc	x27,xzr,xzr
	mul	x7,x13,x13
	adds	x26,x26,x9
	umulh	x13,  x13,x13
	adc	x27,x27,x10
	mul	x8,x14,x14

	mul	x10,x16,x15
	umulh	x14,  x14,x14
	adds	x27,x27,x10
	umulh	x10,x16,x15
	mul	x9,x15,x15
	adc	x28,x10,xzr

	adds	x19,x19,x19
	adcs	x20,x20,x20
	adcs	x21,x21,x21
	adcs	x22,x22,x22
	adcs	x23,x23,x23
	adcs	x24,x24,x24
	adcs	x25,x25,x25
	adcs	x26,x26,x26
	umulh	x15,  x15,x15
	adcs	x27,x27,x27
	mul	x10,x16,x16
	adcs	x28,x28,x28
	umulh	x16,  x16,x16
	adc	x1,xzr,xzr

	adds	x19,x19,x11
	adcs	x20,x20,x6
	adcs	x21,x21,x12
	adcs	x22,x22,x7
	adcs	x23,x23,x13
	adcs	x24,x24,x8
	adcs	x25,x25,x14
	stp	x5,x19,[x0]
	adcs	x26,x26,x9
	stp	x20,x21,[x0,#16]
	adcs	x27,x27,x15
	stp	x22,x23,[x0,#32]
	adcs	x28,x28,x10
	stp	x24,x25,[x0,#48]
	adc	x16,x16,x1
	stp	x26,x27,[x0,#64]
	stp	x28,x16,[x0,#80]

	ret
	ENDP


	EXPORT	|sqr_384|[FUNC]
	ALIGN	32
|sqr_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]

	ldp	x11,x12,[x1]
	ldp	x13,x14,[x1,#16]
	ldp	x15,x16,[x1,#32]

	bl	__sqr_384
	ldr	x30,[x29,#8]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|redc_mont_384|[FUNC]
	ALIGN	32
|redc_mont_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]
	mov	x4,x3		// adjust for missing b_ptr

	ldp	x5,x6,[x2]
	ldp	x7,x8,[x2,#16]
	ldp	x9,x10,[x2,#32]

	bl	__mul_by_1_mont_384
	bl	__redc_tail_mont_384
	ldr	x30,[x29,#8]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|from_mont_384|[FUNC]
	ALIGN	32
|from_mont_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]
	mov	x4,x3		// adjust for missing b_ptr

	ldp	x5,x6,[x2]
	ldp	x7,x8,[x2,#16]
	ldp	x9,x10,[x2,#32]

	bl	__mul_by_1_mont_384
	ldr	x30,[x29,#8]

	subs	x19,x11,x5
	sbcs	x20,x12,x6
	sbcs	x21,x13,x7
	sbcs	x22,x14,x8
	sbcs	x23,x15,x9
	sbcs	x24,x16,x10

	csello	x11,x11,x19
	csello	x12,x12,x20
	csello	x13,x13,x21
	csello	x14,x14,x22
	csello	x15,x15,x23
	csello	x16,x16,x24

	stp	x11,x12,[x0]
	stp	x13,x14,[x0,#16]
	stp	x15,x16,[x0,#32]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP


	ALIGN	32
|__mul_by_1_mont_384| PROC
	ldp	x11,x12,[x1]
	ldp	x13,x14,[x1,#16]
	mul	x26,x4,x11
	ldp	x15,x16,[x1,#32]

	// mul	x19,x5,x26
	mul	x20,x6,x26
	mul	x21,x7,x26
	mul	x22,x8,x26
	mul	x23,x9,x26
	mul	x24,x10,x26
	subs	xzr,x11,#1		// adds	x19,x19,x11
	umulh	x11,x5,x26
	adcs	x20,x20,x12
	umulh	x12,x6,x26
	adcs	x21,x21,x13
	umulh	x13,x7,x26
	adcs	x22,x22,x14
	umulh	x14,x8,x26
	adcs	x23,x23,x15
	umulh	x15,x9,x26
	adcs	x24,x24,x16
	umulh	x16,x10,x26
	adc	x25,xzr,xzr
	adds	x11,x11,x20
	adcs	x12,x12,x21
	adcs	x13,x13,x22
	mul	x26,x4,x11
	adcs	x14,x14,x23
	adcs	x15,x15,x24
	adc	x16,x16,x25

	// mul	x19,x5,x26
	mul	x20,x6,x26
	mul	x21,x7,x26
	mul	x22,x8,x26
	mul	x23,x9,x26
	mul	x24,x10,x26
	subs	xzr,x11,#1		// adds	x19,x19,x11
	umulh	x11,x5,x26
	adcs	x20,x20,x12
	umulh	x12,x6,x26
	adcs	x21,x21,x13
	umulh	x13,x7,x26
	adcs	x22,x22,x14
	umulh	x14,x8,x26
	adcs	x23,x23,x15
	umulh	x15,x9,x26
	adcs	x24,x24,x16
	umulh	x16,x10,x26
	adc	x25,xzr,xzr
	adds	x11,x11,x20
	adcs	x12,x12,x21
	adcs	x13,x13,x22
	mul	x26,x4,x11
	adcs	x14,x14,x23
	adcs	x15,x15,x24
	adc	x16,x16,x25

	// mul	x19,x5,x26
	mul	x20,x6,x26
	mul	x21,x7,x26
	mul	x22,x8,x26
	mul	x23,x9,x26
	mul	x24,x10,x26
	subs	xzr,x11,#1		// adds	x19,x19,x11
	umulh	x11,x5,x26
	adcs	x20,x20,x12
	umulh	x12,x6,x26
	adcs	x21,x21,x13
	umulh	x13,x7,x26
	adcs	x22,x22,x14
	umulh	x14,x8,x26
	adcs	x23,x23,x15
	umulh	x15,x9,x26
	adcs	x24,x24,x16
	umulh	x16,x10,x26
	adc	x25,xzr,xzr
	adds	x11,x11,x20
	adcs	x12,x12,x21
	adcs	x13,x13,x22
	mul	x26,x4,x11
	adcs	x14,x14,x23
	adcs	x15,x15,x24
	adc	x16,x16,x25

	// mul	x19,x5,x26
	mul	x20,x6,x26
	mul	x21,x7,x26
	mul	x22,x8,x26
	mul	x23,x9,x26
	mul	x24,x10,x26
	subs	xzr,x11,#1		// adds	x19,x19,x11
	umulh	x11,x5,x26
	adcs	x20,x20,x12
	umulh	x12,x6,x26
	adcs	x21,x21,x13
	umulh	x13,x7,x26
	adcs	x22,x22,x14
	umulh	x14,x8,x26
	adcs	x23,x23,x15
	umulh	x15,x9,x26
	adcs	x24,x24,x16
	umulh	x16,x10,x26
	adc	x25,xzr,xzr
	adds	x11,x11,x20
	adcs	x12,x12,x21
	adcs	x13,x13,x22
	mul	x26,x4,x11
	adcs	x14,x14,x23
	adcs	x15,x15,x24
	adc	x16,x16,x25

	// mul	x19,x5,x26
	mul	x20,x6,x26
	mul	x21,x7,x26
	mul	x22,x8,x26
	mul	x23,x9,x26
	mul	x24,x10,x26
	subs	xzr,x11,#1		// adds	x19,x19,x11
	umulh	x11,x5,x26
	adcs	x20,x20,x12
	umulh	x12,x6,x26
	adcs	x21,x21,x13
	umulh	x13,x7,x26
	adcs	x22,x22,x14
	umulh	x14,x8,x26
	adcs	x23,x23,x15
	umulh	x15,x9,x26
	adcs	x24,x24,x16
	umulh	x16,x10,x26
	adc	x25,xzr,xzr
	adds	x11,x11,x20
	adcs	x12,x12,x21
	adcs	x13,x13,x22
	mul	x26,x4,x11
	adcs	x14,x14,x23
	adcs	x15,x15,x24
	adc	x16,x16,x25

	// mul	x19,x5,x26
	mul	x20,x6,x26
	mul	x21,x7,x26
	mul	x22,x8,x26
	mul	x23,x9,x26
	mul	x24,x10,x26
	subs	xzr,x11,#1		// adds	x19,x19,x11
	umulh	x11,x5,x26
	adcs	x20,x20,x12
	umulh	x12,x6,x26
	adcs	x21,x21,x13
	umulh	x13,x7,x26
	adcs	x22,x22,x14
	umulh	x14,x8,x26
	adcs	x23,x23,x15
	umulh	x15,x9,x26
	adcs	x24,x24,x16
	umulh	x16,x10,x26
	adc	x25,xzr,xzr
	adds	x11,x11,x20
	adcs	x12,x12,x21
	adcs	x13,x13,x22
	adcs	x14,x14,x23
	adcs	x15,x15,x24
	adc	x16,x16,x25

	ret
	ENDP


	ALIGN	32
|__redc_tail_mont_384| PROC
	ldp	x19,x20,[x1,#48]
	ldp	x21,x22,[x1,#64]
	ldp	x23,x24,[x1,#80]

	adds	x11,x11,x19	// accumulate upper half
	adcs	x12,x12,x20
	adcs	x13,x13,x21
	adcs	x14,x14,x22
	adcs	x15,x15,x23
	adcs	x16,x16,x24
	adc	x25,xzr,xzr

	subs	x19,x11,x5
	sbcs	x20,x12,x6
	sbcs	x21,x13,x7
	sbcs	x22,x14,x8
	sbcs	x23,x15,x9
	sbcs	x24,x16,x10
	sbcs	xzr,x25,xzr

	csello	x11,x11,x19
	csello	x12,x12,x20
	csello	x13,x13,x21
	csello	x14,x14,x22
	csello	x15,x15,x23
	csello	x16,x16,x24

	stp	x11,x12,[x0]
	stp	x13,x14,[x0,#16]
	stp	x15,x16,[x0,#32]

	ret
	ENDP



	EXPORT	|mul_384|[FUNC]
	ALIGN	32
|mul_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]

	bl	__mul_384
	ldr	x30,[x29,#8]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP


	ALIGN	32
|__mul_384| PROC
	ldp	x11,x12,[x1]
	ldr	x17,        [x2]
	ldp	x13,x14,[x1,#16]
	ldp	x15,x16,[x1,#32]

	mul	x19,x11,x17
	mul	x20,x12,x17
	mul	x21,x13,x17
	mul	x22,x14,x17
	mul	x23,x15,x17
	mul	x24,x16,x17

	umulh	x5,x11,x17
	umulh	x6,x12,x17
	umulh	x7,x13,x17
	umulh	x8,x14,x17
	umulh	x9,x15,x17
	umulh	x10,x16,x17
	ldr	x17,[x2,8*1]

	str	x19,[x0]
	adds	x19,x20,x5
	mul	x5,x11,x17
	adcs	x20,x21,x6
	mul	x6,x12,x17
	adcs	x21,x22,x7
	mul	x7,x13,x17
	adcs	x22,x23,x8
	mul	x8,x14,x17
	adcs	x23,x24,x9
	mul	x9,x15,x17
	adc	x24,xzr,    x10
	mul	x10,x16,x17
	adds	x19,x19,x5
	umulh	x5,x11,x17
	adcs	x20,x20,x6
	umulh	x6,x12,x17
	adcs	x21,x21,x7
	umulh	x7,x13,x17
	adcs	x22,x22,x8
	umulh	x8,x14,x17
	adcs	x23,x23,x9
	umulh	x9,x15,x17
	adcs	x24,x24,x10
	umulh	x10,x16,x17
	ldr	x17,[x2,#8*(1+1)]
	adc	x25,xzr,xzr

	str	x19,[x0,8*1]
	adds	x19,x20,x5
	mul	x5,x11,x17
	adcs	x20,x21,x6
	mul	x6,x12,x17
	adcs	x21,x22,x7
	mul	x7,x13,x17
	adcs	x22,x23,x8
	mul	x8,x14,x17
	adcs	x23,x24,x9
	mul	x9,x15,x17
	adc	x24,x25,x10
	mul	x10,x16,x17
	adds	x19,x19,x5
	umulh	x5,x11,x17
	adcs	x20,x20,x6
	umulh	x6,x12,x17
	adcs	x21,x21,x7
	umulh	x7,x13,x17
	adcs	x22,x22,x8
	umulh	x8,x14,x17
	adcs	x23,x23,x9
	umulh	x9,x15,x17
	adcs	x24,x24,x10
	umulh	x10,x16,x17
	ldr	x17,[x2,#8*(2+1)]
	adc	x25,xzr,xzr

	str	x19,[x0,8*2]
	adds	x19,x20,x5
	mul	x5,x11,x17
	adcs	x20,x21,x6
	mul	x6,x12,x17
	adcs	x21,x22,x7
	mul	x7,x13,x17
	adcs	x22,x23,x8
	mul	x8,x14,x17
	adcs	x23,x24,x9
	mul	x9,x15,x17
	adc	x24,x25,x10
	mul	x10,x16,x17
	adds	x19,x19,x5
	umulh	x5,x11,x17
	adcs	x20,x20,x6
	umulh	x6,x12,x17
	adcs	x21,x21,x7
	umulh	x7,x13,x17
	adcs	x22,x22,x8
	umulh	x8,x14,x17
	adcs	x23,x23,x9
	umulh	x9,x15,x17
	adcs	x24,x24,x10
	umulh	x10,x16,x17
	ldr	x17,[x2,#8*(3+1)]
	adc	x25,xzr,xzr

	str	x19,[x0,8*3]
	adds	x19,x20,x5
	mul	x5,x11,x17
	adcs	x20,x21,x6
	mul	x6,x12,x17
	adcs	x21,x22,x7
	mul	x7,x13,x17
	adcs	x22,x23,x8
	mul	x8,x14,x17
	adcs	x23,x24,x9
	mul	x9,x15,x17
	adc	x24,x25,x10
	mul	x10,x16,x17
	adds	x19,x19,x5
	umulh	x5,x11,x17
	adcs	x20,x20,x6
	umulh	x6,x12,x17
	adcs	x21,x21,x7
	umulh	x7,x13,x17
	adcs	x22,x22,x8
	umulh	x8,x14,x17
	adcs	x23,x23,x9
	umulh	x9,x15,x17
	adcs	x24,x24,x10
	umulh	x10,x16,x17
	ldr	x17,[x2,#8*(4+1)]
	adc	x25,xzr,xzr

	str	x19,[x0,8*4]
	adds	x19,x20,x5
	mul	x5,x11,x17
	adcs	x20,x21,x6
	mul	x6,x12,x17
	adcs	x21,x22,x7
	mul	x7,x13,x17
	adcs	x22,x23,x8
	mul	x8,x14,x17
	adcs	x23,x24,x9
	mul	x9,x15,x17
	adc	x24,x25,x10
	mul	x10,x16,x17
	adds	x19,x19,x5
	umulh	x5,x11,x17
	adcs	x20,x20,x6
	umulh	x6,x12,x17
	adcs	x21,x21,x7
	umulh	x7,x13,x17
	adcs	x22,x22,x8
	umulh	x8,x14,x17
	adcs	x23,x23,x9
	umulh	x9,x15,x17
	adcs	x24,x24,x10
	umulh	x10,x16,x17
	adc	x25,xzr,xzr

	str	x19,[x0,8*5]
	adds	x19,x20,x5
	adcs	x20,x21,x6
	adcs	x21,x22,x7
	adcs	x22,x23,x8
	adcs	x23,x24,x9
	adc	x24,x25,x10

	stp	x19,x20,[x0,#48]
	stp	x21,x22,[x0,#64]
	stp	x23,x24,[x0,#80]

	ret
	ENDP



	EXPORT	|mul_382x|[FUNC]
	ALIGN	32
|mul_382x| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]
	sub	sp,sp,#96		// space for two 384-bit vectors

	ldp	x11,x12,[x1]
	mov	x26,x0		// save r_ptr
	ldp	x19,x20,[x1,#48]
	mov	x27,x1		// save a_ptr
	ldp	x13,x14,[x1,#16]
	mov	x28,x2		// save b_ptr
	ldp	x21,x22,[x1,#64]
	ldp	x15,x16,[x1,#32]
	adds	x5,x11,x19	// t0 = a->re + a->im
	ldp	x23,x24,[x1,#80]
	adcs	x6,x12,x20
	ldp	x11,x12,[x2]
	adcs	x7,x13,x21
	ldp	x19,x20,[x2,#48]
	adcs	x8,x14,x22
	ldp	x13,x14,[x2,#16]
	adcs	x9,x15,x23
	ldp	x21,x22,[x2,#64]
	adc	x10,x16,x24
	ldp	x15,x16,[x2,#32]

	stp	x5,x6,[sp]
	adds	x5,x11,x19	// t1 = b->re + b->im
	ldp	x23,x24,[x2,#80]
	adcs	x6,x12,x20
	stp	x7,x8,[sp,#16]
	adcs	x7,x13,x21
	adcs	x8,x14,x22
	stp	x9,x10,[sp,#32]
	adcs	x9,x15,x23
	stp	x5,x6,[sp,#48]
	adc	x10,x16,x24
	stp	x7,x8,[sp,#64]
	stp	x9,x10,[sp,#80]

	bl	__mul_384		// mul_384(ret->re, a->re, b->re)

	add	x1,sp,#0		// mul_384(ret->im, t0, t1)
	add	x2,sp,#48
	add	x0,x26,#96
	bl	__mul_384

	add	x1,x27,#48	// mul_384(tx, a->im, b->im)
	add	x2,x28,#48
	add	x0,sp,#0
	bl	__mul_384

	ldp	x5,x6,[x3]
	ldp	x7,x8,[x3,#16]
	ldp	x9,x10,[x3,#32]

	add	x1,x26,#96	// ret->im -= tx
	add	x2,sp,#0
	add	x0,x26,#96
	bl	__sub_mod_384x384

	add	x2,x26,#0	// ret->im -= ret->re
	bl	__sub_mod_384x384

	add	x1,x26,#0	// ret->re -= tx
	add	x2,sp,#0
	add	x0,x26,#0
	bl	__sub_mod_384x384
	ldr	x30,[x29,#8]

	add	sp,sp,#96
	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|sqr_382x|[FUNC]
	ALIGN	32
|sqr_382x| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]

	ldp	x11,x12,[x1]
	ldp	x19,x20,[x1,#48]
	ldp	x13,x14,[x1,#16]
	adds	x5,x11,x19	// t0 = a->re + a->im
	ldp	x21,x22,[x1,#64]
	adcs	x6,x12,x20
	ldp	x15,x16,[x1,#32]
	adcs	x7,x13,x21
	ldp	x23,x24,[x1,#80]
	adcs	x8,x14,x22
	stp	x5,x6,[x0]
	adcs	x9,x15,x23
	ldp	x5,x6,[x2]
	adc	x10,x16,x24
	stp	x7,x8,[x0,#16]

	subs	x11,x11,x19	// t1 = a->re - a->im
	ldp	x7,x8,[x2,#16]
	sbcs	x12,x12,x20
	stp	x9,x10,[x0,#32]
	sbcs	x13,x13,x21
	ldp	x9,x10,[x2,#32]
	sbcs	x14,x14,x22
	sbcs	x15,x15,x23
	sbcs	x16,x16,x24
	sbc	x25,xzr,xzr

	and	x19,x5,x25
	and	x20,x6,x25
	adds	x11,x11,x19
	and	x21,x7,x25
	adcs	x12,x12,x20
	and	x22,x8,x25
	adcs	x13,x13,x21
	and	x23,x9,x25
	adcs	x14,x14,x22
	and	x24,x10,x25
	adcs	x15,x15,x23
	stp	x11,x12,[x0,#48]
	adc	x16,x16,x24
	stp	x13,x14,[x0,#64]
	stp	x15,x16,[x0,#80]

	mov	x4,x1		// save a_ptr
	add	x1,x0,#0	// mul_384(ret->re, t0, t1)
	add	x2,x0,#48
	bl	__mul_384

	add	x1,x4,#0		// mul_384(ret->im, a->re, a->im)
	add	x2,x4,#48
	add	x0,x0,#96
	bl	__mul_384
	ldr	x30,[x29,#8]

	ldp	x11,x12,[x0]
	ldp	x13,x14,[x0,#16]
	adds	x11,x11,x11	// add with itself
	ldp	x15,x16,[x0,#32]
	adcs	x12,x12,x12
	adcs	x13,x13,x13
	adcs	x14,x14,x14
	adcs	x15,x15,x15
	adcs	x16,x16,x16
	adcs	x19,x19,x19
	adcs	x20,x20,x20
	stp	x11,x12,[x0]
	adcs	x21,x21,x21
	stp	x13,x14,[x0,#16]
	adcs	x22,x22,x22
	stp	x15,x16,[x0,#32]
	adcs	x23,x23,x23
	stp	x19,x20,[x0,#48]
	adc	x24,x24,x24
	stp	x21,x22,[x0,#64]
	stp	x23,x24,[x0,#80]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|sqr_mont_382x|[FUNC]
	ALIGN	32
|sqr_mont_382x| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]
	stp	x3,x0,[sp,#96]	// __mul_mont_384 wants them there
	sub	sp,sp,#112		// space for two 384-bit vectors + word
	mov	x4,x3		// adjust for missing b_ptr

	ldp	x11,x12,[x1]
	ldp	x13,x14,[x1,#16]
	ldp	x15,x16,[x1,#32]

	ldp	x17,x20,[x1,#48]
	ldp	x21,x22,[x1,#64]
	ldp	x23,x24,[x1,#80]

	adds	x5,x11,x17	// t0 = a->re + a->im
	adcs	x6,x12,x20
	adcs	x7,x13,x21
	adcs	x8,x14,x22
	adcs	x9,x15,x23
	adc	x10,x16,x24

	subs	x19,x11,x17	// t1 = a->re - a->im
	sbcs	x20,x12,x20
	sbcs	x21,x13,x21
	sbcs	x22,x14,x22
	sbcs	x23,x15,x23
	sbcs	x24,x16,x24
	sbc	x25,xzr,xzr		// borrow flag as mask

	stp	x5,x6,[sp]
	stp	x7,x8,[sp,#16]
	stp	x9,x10,[sp,#32]
	stp	x19,x20,[sp,#48]
	stp	x21,x22,[sp,#64]
	stp	x23,x24,[sp,#80]
	str	x25,[sp,#96]

	ldp	x5,x6,[x2]
	ldp	x7,x8,[x2,#16]
	ldp	x9,x10,[x2,#32]

	add	x2,x1,#48
	bl	__mul_mont_383_nonred	// mul_mont_384(ret->im, a->re, a->im)

	adds	x19,x11,x11	// add with itself
	adcs	x20,x12,x12
	adcs	x21,x13,x13
	adcs	x22,x14,x14
	adcs	x23,x15,x15
	adc	x24,x16,x16

	stp	x19,x20,[x2,#48]
	stp	x21,x22,[x2,#64]
	stp	x23,x24,[x2,#80]

	ldp	x11,x12,[sp]
	ldr	x17,[sp,#48]
	ldp	x13,x14,[sp,#16]
	ldp	x15,x16,[sp,#32]

	add	x2,sp,#48
	bl	__mul_mont_383_nonred	// mul_mont_384(ret->im, t0, t1)
	ldr	x30,[x29,#8]

	ldr	x25,[sp,#96]	// account for sign from a->re - a->im
	ldp	x19,x20,[sp]
	ldp	x21,x22,[sp,#16]
	ldp	x23,x24,[sp,#32]

	and	x19,x19,x25
	and	x20,x20,x25
	and	x21,x21,x25
	and	x22,x22,x25
	and	x23,x23,x25
	and	x24,x24,x25

	subs	x11,x11,x19
	sbcs	x12,x12,x20
	sbcs	x13,x13,x21
	sbcs	x14,x14,x22
	sbcs	x15,x15,x23
	sbcs	x16,x16,x24
	sbc	x25,xzr,xzr

	and	x19,x5,x25
	and	x20,x6,x25
	and	x21,x7,x25
	and	x22,x8,x25
	and	x23,x9,x25
	and	x24,x10,x25

	adds	x11,x11,x19
	adcs	x12,x12,x20
	adcs	x13,x13,x21
	adcs	x14,x14,x22
	adcs	x15,x15,x23
	adc	x16,x16,x24

	stp	x11,x12,[x2]
	stp	x13,x14,[x2,#16]
	stp	x15,x16,[x2,#32]

	add	sp,sp,#112
	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP


	ALIGN	32
|__mul_mont_383_nonred| PROC
	mul	x19,x11,x17
	mul	x20,x12,x17
	mul	x21,x13,x17
	mul	x22,x14,x17
	mul	x23,x15,x17
	mul	x24,x16,x17
	mul	x4,x4,x19

	umulh	x26,x11,x17
	umulh	x27,x12,x17
	umulh	x28,x13,x17
	umulh	x0,x14,x17
	umulh	x1,x15,x17
	umulh	x3,x16,x17

	adds	x20,x20,x26
	mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adc	x25,xzr,    x3
	mul	x3,x10,x4
	ldr	x17,[x2,8*1]
	adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adc	x25,x25,xzr

	ldr	x4,[x29,#96]
	adds	x19,x20,x26
	mul	x26,x11,x17
	adcs	x20,x21,x27
	mul	x27,x12,x17
	adcs	x21,x22,x28
	mul	x28,x13,x17
	adcs	x22,x23,x0
	mul	x0,x14,x17
	adcs	x23,x24,x1
	mul	x1,x15,x17
	adcs	x24,x25,x3
	mul	x3,x16,x17
	adc	x25,xzr,xzr

	adds	x19,x19,x26
	umulh	x26,x11,x17
	adcs	x20,x20,x27
	umulh	x27,x12,x17
	adcs	x21,x21,x28
	mul	x4,x4,x19
	umulh	x28,x13,x17
	adcs	x22,x22,x0
	umulh	x0,x14,x17
	adcs	x23,x23,x1
	umulh	x1,x15,x17
	adcs	x24,x24,x3
	umulh	x3,x16,x17
	adc	x25,x25,xzr

	adds	x20,x20,x26
	mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adc	x25,x25,x3
	mul	x3,x10,x4
	ldr	x17,[x2,8*2]
	adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adc	x25,x25,xzr

	ldr	x4,[x29,#96]
	adds	x19,x20,x26
	mul	x26,x11,x17
	adcs	x20,x21,x27
	mul	x27,x12,x17
	adcs	x21,x22,x28
	mul	x28,x13,x17
	adcs	x22,x23,x0
	mul	x0,x14,x17
	adcs	x23,x24,x1
	mul	x1,x15,x17
	adcs	x24,x25,x3
	mul	x3,x16,x17
	adc	x25,xzr,xzr

	adds	x19,x19,x26
	umulh	x26,x11,x17
	adcs	x20,x20,x27
	umulh	x27,x12,x17
	adcs	x21,x21,x28
	mul	x4,x4,x19
	umulh	x28,x13,x17
	adcs	x22,x22,x0
	umulh	x0,x14,x17
	adcs	x23,x23,x1
	umulh	x1,x15,x17
	adcs	x24,x24,x3
	umulh	x3,x16,x17
	adc	x25,x25,xzr

	adds	x20,x20,x26
	mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adc	x25,x25,x3
	mul	x3,x10,x4
	ldr	x17,[x2,8*3]
	adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adc	x25,x25,xzr

	ldr	x4,[x29,#96]
	adds	x19,x20,x26
	mul	x26,x11,x17
	adcs	x20,x21,x27
	mul	x27,x12,x17
	adcs	x21,x22,x28
	mul	x28,x13,x17
	adcs	x22,x23,x0
	mul	x0,x14,x17
	adcs	x23,x24,x1
	mul	x1,x15,x17
	adcs	x24,x25,x3
	mul	x3,x16,x17
	adc	x25,xzr,xzr

	adds	x19,x19,x26
	umulh	x26,x11,x17
	adcs	x20,x20,x27
	umulh	x27,x12,x17
	adcs	x21,x21,x28
	mul	x4,x4,x19
	umulh	x28,x13,x17
	adcs	x22,x22,x0
	umulh	x0,x14,x17
	adcs	x23,x23,x1
	umulh	x1,x15,x17
	adcs	x24,x24,x3
	umulh	x3,x16,x17
	adc	x25,x25,xzr

	adds	x20,x20,x26
	mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adc	x25,x25,x3
	mul	x3,x10,x4
	ldr	x17,[x2,8*4]
	adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adc	x25,x25,xzr

	ldr	x4,[x29,#96]
	adds	x19,x20,x26
	mul	x26,x11,x17
	adcs	x20,x21,x27
	mul	x27,x12,x17
	adcs	x21,x22,x28
	mul	x28,x13,x17
	adcs	x22,x23,x0
	mul	x0,x14,x17
	adcs	x23,x24,x1
	mul	x1,x15,x17
	adcs	x24,x25,x3
	mul	x3,x16,x17
	adc	x25,xzr,xzr

	adds	x19,x19,x26
	umulh	x26,x11,x17
	adcs	x20,x20,x27
	umulh	x27,x12,x17
	adcs	x21,x21,x28
	mul	x4,x4,x19
	umulh	x28,x13,x17
	adcs	x22,x22,x0
	umulh	x0,x14,x17
	adcs	x23,x23,x1
	umulh	x1,x15,x17
	adcs	x24,x24,x3
	umulh	x3,x16,x17
	adc	x25,x25,xzr

	adds	x20,x20,x26
	mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adc	x25,x25,x3
	mul	x3,x10,x4
	ldr	x17,[x2,8*5]
	adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adc	x25,x25,xzr

	ldr	x4,[x29,#96]
	adds	x19,x20,x26
	mul	x26,x11,x17
	adcs	x20,x21,x27
	mul	x27,x12,x17
	adcs	x21,x22,x28
	mul	x28,x13,x17
	adcs	x22,x23,x0
	mul	x0,x14,x17
	adcs	x23,x24,x1
	mul	x1,x15,x17
	adcs	x24,x25,x3
	mul	x3,x16,x17
	adc	x25,xzr,xzr

	adds	x19,x19,x26
	umulh	x26,x11,x17
	adcs	x20,x20,x27
	umulh	x27,x12,x17
	adcs	x21,x21,x28
	mul	x4,x4,x19
	umulh	x28,x13,x17
	adcs	x22,x22,x0
	umulh	x0,x14,x17
	adcs	x23,x23,x1
	umulh	x1,x15,x17
	adcs	x24,x24,x3
	umulh	x3,x16,x17
	adc	x25,x25,xzr

	adds	x20,x20,x26
	mul	x26,x5,x4
	adcs	x21,x21,x27
	mul	x27,x6,x4
	adcs	x22,x22,x28
	mul	x28,x7,x4
	adcs	x23,x23,x0
	mul	x0,x8,x4
	adcs	x24,x24,x1
	mul	x1,x9,x4
	adc	x25,x25,x3
	mul	x3,x10,x4
	adds	x19,x19,x26
	umulh	x26,x5,x4
	adcs	x20,x20,x27
	umulh	x27,x6,x4
	adcs	x21,x21,x28
	umulh	x28,x7,x4
	adcs	x22,x22,x0
	umulh	x0,x8,x4
	adcs	x23,x23,x1
	umulh	x1,x9,x4
	adcs	x24,x24,x3
	umulh	x3,x10,x4
	adc	x25,x25,xzr
	ldp	x4,x2,[x29,#96]		// pull r_ptr

	adds	x11,x20,x26
	adcs	x12,x21,x27
	adcs	x13,x22,x28
	adcs	x14,x23,x0
	adcs	x15,x24,x1
	adcs	x16,x25,x3

	ret
	ENDP



	EXPORT	|sgn0_pty_mont_384|[FUNC]
	ALIGN	32
|sgn0_pty_mont_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]

	mov	x4,x2
	ldp	x5,x6,[x1]
	ldp	x7,x8,[x1,#16]
	ldp	x9,x10,[x1,#32]
	mov	x1,x0

	bl	__mul_by_1_mont_384
	ldr	x30,[x29,#8]

	and	x0,x11,#1
	adds	x11,x11,x11
	adcs	x12,x12,x12
	adcs	x13,x13,x13
	adcs	x14,x14,x14
	adcs	x15,x15,x15
	adcs	x16,x16,x16
	adc	x17,xzr,xzr

	subs	x11,x11,x5
	sbcs	x12,x12,x6
	sbcs	x13,x13,x7
	sbcs	x14,x14,x8
	sbcs	x15,x15,x9
	sbcs	x16,x16,x10
	sbc	x17,x17,xzr

	mvn	x17,x17
	and	x17,x17,#2
	orr	x0,x0,x17

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|sgn0_pty_mont_384x|[FUNC]
	ALIGN	32
|sgn0_pty_mont_384x| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-128]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	stp	x27,x28,[sp,#80]

	mov	x4,x2
	ldp	x5,x6,[x1]
	ldp	x7,x8,[x1,#16]
	ldp	x9,x10,[x1,#32]
	mov	x1,x0

	bl	__mul_by_1_mont_384
	add	x1,x1,#48

	and	x2,x11,#1
	orr	x3,x11,x12
	adds	x11,x11,x11
	orr	x3,x3,x13
	adcs	x12,x12,x12
	orr	x3,x3,x14
	adcs	x13,x13,x13
	orr	x3,x3,x15
	adcs	x14,x14,x14
	orr	x3,x3,x16
	adcs	x15,x15,x15
	adcs	x16,x16,x16
	adc	x17,xzr,xzr

	subs	x11,x11,x5
	sbcs	x12,x12,x6
	sbcs	x13,x13,x7
	sbcs	x14,x14,x8
	sbcs	x15,x15,x9
	sbcs	x16,x16,x10
	sbc	x17,x17,xzr

	mvn	x17,x17
	and	x17,x17,#2
	orr	x2,x2,x17

	bl	__mul_by_1_mont_384
	ldr	x30,[x29,#8]

	and	x0,x11,#1
	orr	x1,x11,x12
	adds	x11,x11,x11
	orr	x1,x1,x13
	adcs	x12,x12,x12
	orr	x1,x1,x14
	adcs	x13,x13,x13
	orr	x1,x1,x15
	adcs	x14,x14,x14
	orr	x1,x1,x16
	adcs	x15,x15,x15
	adcs	x16,x16,x16
	adc	x17,xzr,xzr

	subs	x11,x11,x5
	sbcs	x12,x12,x6
	sbcs	x13,x13,x7
	sbcs	x14,x14,x8
	sbcs	x15,x15,x9
	sbcs	x16,x16,x10
	sbc	x17,x17,xzr

	mvn	x17,x17
	and	x17,x17,#2
	orr	x0,x0,x17

	cmp	x3,#0
	cseleq	x3,x0,x2

	cmp	x1,#0
	cselne	x1,x0,x2

	and	x3,x3,#1
	and	x1,x1,#2
	orr	x0,x1,x3		// pack sign and parity

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x27,x28,[x29,#80]
	ldr	x29,[sp],#128
	DCDU	3573752767
	ret
	ENDP
	END
