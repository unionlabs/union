	AREA	|.text|,CODE,ALIGN=8,ARM64



	EXPORT	|mul_mont_sparse_256|[FUNC]
	ALIGN	32
|mul_mont_sparse_256| PROC
	stp	x29,x30,[sp,#-64]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]

	ldp	x10,x11,[x1]
	ldr	x9,        [x2]
	ldp	x12,x13,[x1,#16]

	mul	x19,x10,x9
	ldp	x5,x6,[x3]
	mul	x20,x11,x9
	ldp	x7,x8,[x3,#16]
	mul	x21,x12,x9
	mul	x22,x13,x9

	umulh	x14,x10,x9
	umulh	x15,x11,x9
	mul	x3,x4,x19
	umulh	x16,x12,x9
	umulh	x17,x13,x9
	adds	x20,x20,x14
	//mul	x14,x5,x3
	adcs	x21,x21,x15
	mul	x15,x6,x3
	adcs	x22,x22,x16
	mul	x16,x7,x3
	adc	x23,xzr,    x17
	mul	x17,x8,x3
	ldr	x9,[x2,8*1]
	subs	xzr,x19,#1		//adds	x19,x19,x14
	umulh	x14,x5,x3
	adcs	x20,x20,x15
	umulh	x15,x6,x3
	adcs	x21,x21,x16
	umulh	x16,x7,x3
	adcs	x22,x22,x17
	umulh	x17,x8,x3
	adc	x23,x23,xzr

	adds	x19,x20,x14
	mul	x14,x10,x9
	adcs	x20,x21,x15
	mul	x15,x11,x9
	adcs	x21,x22,x16
	mul	x16,x12,x9
	adcs	x22,x23,x17
	mul	x17,x13,x9
	adc	x23,xzr,xzr

	adds	x19,x19,x14
	umulh	x14,x10,x9
	adcs	x20,x20,x15
	umulh	x15,x11,x9
	adcs	x21,x21,x16
	mul	x3,x4,x19
	umulh	x16,x12,x9
	adcs	x22,x22,x17
	umulh	x17,x13,x9
	adc	x23,x23,xzr

	adds	x20,x20,x14
	//mul	x14,x5,x3
	adcs	x21,x21,x15
	mul	x15,x6,x3
	adcs	x22,x22,x16
	mul	x16,x7,x3
	adc	x23,x23,x17
	mul	x17,x8,x3
	ldr	x9,[x2,8*2]
	subs	xzr,x19,#1		//adds	x19,x19,x14
	umulh	x14,x5,x3
	adcs	x20,x20,x15
	umulh	x15,x6,x3
	adcs	x21,x21,x16
	umulh	x16,x7,x3
	adcs	x22,x22,x17
	umulh	x17,x8,x3
	adc	x23,x23,xzr

	adds	x19,x20,x14
	mul	x14,x10,x9
	adcs	x20,x21,x15
	mul	x15,x11,x9
	adcs	x21,x22,x16
	mul	x16,x12,x9
	adcs	x22,x23,x17
	mul	x17,x13,x9
	adc	x23,xzr,xzr

	adds	x19,x19,x14
	umulh	x14,x10,x9
	adcs	x20,x20,x15
	umulh	x15,x11,x9
	adcs	x21,x21,x16
	mul	x3,x4,x19
	umulh	x16,x12,x9
	adcs	x22,x22,x17
	umulh	x17,x13,x9
	adc	x23,x23,xzr

	adds	x20,x20,x14
	//mul	x14,x5,x3
	adcs	x21,x21,x15
	mul	x15,x6,x3
	adcs	x22,x22,x16
	mul	x16,x7,x3
	adc	x23,x23,x17
	mul	x17,x8,x3
	ldr	x9,[x2,8*3]
	subs	xzr,x19,#1		//adds	x19,x19,x14
	umulh	x14,x5,x3
	adcs	x20,x20,x15
	umulh	x15,x6,x3
	adcs	x21,x21,x16
	umulh	x16,x7,x3
	adcs	x22,x22,x17
	umulh	x17,x8,x3
	adc	x23,x23,xzr

	adds	x19,x20,x14
	mul	x14,x10,x9
	adcs	x20,x21,x15
	mul	x15,x11,x9
	adcs	x21,x22,x16
	mul	x16,x12,x9
	adcs	x22,x23,x17
	mul	x17,x13,x9
	adc	x23,xzr,xzr

	adds	x19,x19,x14
	umulh	x14,x10,x9
	adcs	x20,x20,x15
	umulh	x15,x11,x9
	adcs	x21,x21,x16
	mul	x3,x4,x19
	umulh	x16,x12,x9
	adcs	x22,x22,x17
	umulh	x17,x13,x9
	adc	x23,x23,xzr

	adds	x20,x20,x14
	//mul	x14,x5,x3
	adcs	x21,x21,x15
	mul	x15,x6,x3
	adcs	x22,x22,x16
	mul	x16,x7,x3
	adc	x23,x23,x17
	mul	x17,x8,x3
	subs	xzr,x19,#1		//adds	x19,x19,x14
	umulh	x14,x5,x3
	adcs	x20,x20,x15
	umulh	x15,x6,x3
	adcs	x21,x21,x16
	umulh	x16,x7,x3
	adcs	x22,x22,x17
	umulh	x17,x8,x3
	adc	x23,x23,xzr

	adds	x19,x20,x14
	adcs	x20,x21,x15
	adcs	x21,x22,x16
	adcs	x22,x23,x17
	adc	x23,xzr,xzr

	subs	x14,x19,x5
	sbcs	x15,x20,x6
	sbcs	x16,x21,x7
	sbcs	x17,x22,x8
	sbcs	xzr,    x23,xzr

	csello	x19,x19,x14
	csello	x20,x20,x15
	csello	x21,x21,x16
	csello	x22,x22,x17

	stp	x19,x20,[x0]
	stp	x21,x22,[x0,#16]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldr	x29,[sp],#64
	ret
	ENDP


	EXPORT	|sqr_mont_sparse_256|[FUNC]
	ALIGN	32
|sqr_mont_sparse_256| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x5,x6,[x1]
	ldp	x7,x8,[x1,#16]
	mov	x4,x3

	////////////////////////////////////////////////////////////////
	//  |  |  |  |  |  |a1*a0|  |
	//  |  |  |  |  |a2*a0|  |  |
	//  |  |a3*a2|a3*a0|  |  |  |
	//  |  |  |  |a2*a1|  |  |  |
	//  |  |  |a3*a1|  |  |  |  |
	// *|  |  |  |  |  |  |  | 2|
	// +|a3*a3|a2*a2|a1*a1|a0*a0|
	//  |--+--+--+--+--+--+--+--|
	//  |A7|A6|A5|A4|A3|A2|A1|A0|, where Ax is x10
	//
	//  "can't overflow" below mark carrying into high part of
	//  multiplication result, which can't overflow, because it
	//  can never be all ones.

	mul	x11,x6,x5	// a[1]*a[0]
	umulh	x15,x6,x5
	mul	x12,x7,x5	// a[2]*a[0]
	umulh	x16,x7,x5
	mul	x13,x8,x5	// a[3]*a[0]
	umulh	x19,x8,x5

	adds	x12,x12,x15	// accumulate high parts of multiplication
	mul	x14,x7,x6	// a[2]*a[1]
	umulh	x15,x7,x6
	adcs	x13,x13,x16
	mul	x16,x8,x6	// a[3]*a[1]
	umulh	x17,x8,x6
	adc	x19,x19,xzr	// can't overflow

	mul	x20,x8,x7	// a[3]*a[2]
	umulh	x21,x8,x7

	adds	x15,x15,x16	// accumulate high parts of multiplication
	mul	x10,x5,x5	// a[0]*a[0]
	adc	x16,x17,xzr	// can't overflow

	adds	x13,x13,x14	// accumulate low parts of multiplication
	umulh	x5,x5,x5
	adcs	x19,x19,x15
	mul	x15,x6,x6	// a[1]*a[1]
	adcs	x20,x20,x16
	umulh	x6,x6,x6
	adc	x21,x21,xzr	// can't overflow

	adds	x11,x11,x11	// acc[1-6]*=2
	mul	x16,x7,x7	// a[2]*a[2]
	adcs	x12,x12,x12
	umulh	x7,x7,x7
	adcs	x13,x13,x13
	mul	x17,x8,x8	// a[3]*a[3]
	adcs	x19,x19,x19
	umulh	x8,x8,x8
	adcs	x20,x20,x20
	adcs	x21,x21,x21
	adc	x22,xzr,xzr

	adds	x11,x11,x5	// +a[i]*a[i]
	adcs	x12,x12,x15
	adcs	x13,x13,x6
	adcs	x19,x19,x16
	adcs	x20,x20,x7
	adcs	x21,x21,x17
	adc	x22,x22,x8

	bl	__mul_by_1_mont_256
	ldr	x30,[x29,#8]

	adds	x10,x10,x19	// accumulate upper half
	adcs	x11,x11,x20
	adcs	x12,x12,x21
	adcs	x13,x13,x22
	adc	x19,xzr,xzr

	subs	x14,x10,x5
	sbcs	x15,x11,x6
	sbcs	x16,x12,x7
	sbcs	x17,x13,x8
	sbcs	xzr,    x19,xzr

	csello	x10,x10,x14
	csello	x11,x11,x15
	csello	x12,x12,x16
	csello	x13,x13,x17

	stp	x10,x11,[x0]
	stp	x12,x13,[x0,#16]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP


	EXPORT	|from_mont_256|[FUNC]
	ALIGN	32
|from_mont_256| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-16]!
	add	x29,sp,#0

	mov	x4,x3
	ldp	x10,x11,[x1]
	ldp	x12,x13,[x1,#16]

	bl	__mul_by_1_mont_256
	ldr	x30,[x29,#8]

	subs	x14,x10,x5
	sbcs	x15,x11,x6
	sbcs	x16,x12,x7
	sbcs	x17,x13,x8

	csello	x10,x10,x14
	csello	x11,x11,x15
	csello	x12,x12,x16
	csello	x13,x13,x17

	stp	x10,x11,[x0]
	stp	x12,x13,[x0,#16]

	ldr	x29,[sp],#16
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|redc_mont_256|[FUNC]
	ALIGN	32
|redc_mont_256| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-16]!
	add	x29,sp,#0

	mov	x4,x3
	ldp	x10,x11,[x1]
	ldp	x12,x13,[x1,#16]

	bl	__mul_by_1_mont_256
	ldr	x30,[x29,#8]

	ldp	x14,x15,[x1,#32]
	ldp	x16,x17,[x1,#48]

	adds	x10,x10,x14
	adcs	x11,x11,x15
	adcs	x12,x12,x16
	adcs	x13,x13,x17
	adc	x9,xzr,xzr

	subs	x14,x10,x5
	sbcs	x15,x11,x6
	sbcs	x16,x12,x7
	sbcs	x17,x13,x8
	sbcs	xzr,    x9,xzr

	csello	x10,x10,x14
	csello	x11,x11,x15
	csello	x12,x12,x16
	csello	x13,x13,x17

	stp	x10,x11,[x0]
	stp	x12,x13,[x0,#16]

	ldr	x29,[sp],#16
	DCDU	3573752767
	ret
	ENDP


	ALIGN	32
|__mul_by_1_mont_256| PROC
	mul	x3,x4,x10
	ldp	x5,x6,[x2]
	ldp	x7,x8,[x2,#16]
	//mul	x14,x5,x3
	mul	x15,x6,x3
	mul	x16,x7,x3
	mul	x17,x8,x3
	subs	xzr,x10,#1		//adds	x10,x10,x14
	umulh	x14,x5,x3
	adcs	x11,x11,x15
	umulh	x15,x6,x3
	adcs	x12,x12,x16
	umulh	x16,x7,x3
	adcs	x13,x13,x17
	umulh	x17,x8,x3
	adc	x9,xzr,xzr

	adds	x10,x11,x14
	adcs	x11,x12,x15
	adcs	x12,x13,x16
	mul	x3,x4,x10
	adc	x13,x9,x17
	//mul	x14,x5,x3
	mul	x15,x6,x3
	mul	x16,x7,x3
	mul	x17,x8,x3
	subs	xzr,x10,#1		//adds	x10,x10,x14
	umulh	x14,x5,x3
	adcs	x11,x11,x15
	umulh	x15,x6,x3
	adcs	x12,x12,x16
	umulh	x16,x7,x3
	adcs	x13,x13,x17
	umulh	x17,x8,x3
	adc	x9,xzr,xzr

	adds	x10,x11,x14
	adcs	x11,x12,x15
	adcs	x12,x13,x16
	mul	x3,x4,x10
	adc	x13,x9,x17
	//mul	x14,x5,x3
	mul	x15,x6,x3
	mul	x16,x7,x3
	mul	x17,x8,x3
	subs	xzr,x10,#1		//adds	x10,x10,x14
	umulh	x14,x5,x3
	adcs	x11,x11,x15
	umulh	x15,x6,x3
	adcs	x12,x12,x16
	umulh	x16,x7,x3
	adcs	x13,x13,x17
	umulh	x17,x8,x3
	adc	x9,xzr,xzr

	adds	x10,x11,x14
	adcs	x11,x12,x15
	adcs	x12,x13,x16
	mul	x3,x4,x10
	adc	x13,x9,x17
	//mul	x14,x5,x3
	mul	x15,x6,x3
	mul	x16,x7,x3
	mul	x17,x8,x3
	subs	xzr,x10,#1		//adds	x10,x10,x14
	umulh	x14,x5,x3
	adcs	x11,x11,x15
	umulh	x15,x6,x3
	adcs	x12,x12,x16
	umulh	x16,x7,x3
	adcs	x13,x13,x17
	umulh	x17,x8,x3
	adc	x9,xzr,xzr

	adds	x10,x11,x14
	adcs	x11,x12,x15
	adcs	x12,x13,x16
	adc	x13,x9,x17

	ret
	ENDP
	END
