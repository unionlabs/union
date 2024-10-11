.text

.globl	mul_mont_sparse_256
.hidden	mul_mont_sparse_256
.type	mul_mont_sparse_256,%function
.align	5
mul_mont_sparse_256:
	stp	c29,c30,[csp,#-8*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]

	ldp	x10,x11,[c1]
	ldr	x9,        [c2]
	ldp	x12,x13,[c1,#16]

	mul	x19,x10,x9
	ldp	x5,x6,[c3]
	mul	x20,x11,x9
	ldp	x7,x8,[c3,#16]
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
	ldr	x9,[c2,8*1]
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
	ldr	x9,[c2,8*2]
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
	ldr	x9,[c2,8*3]
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

	csel	x19,x19,x14,lo
	csel	x20,x20,x15,lo
	csel	x21,x21,x16,lo
	csel	x22,x22,x17,lo

	stp	x19,x20,[c0]
	stp	x21,x22,[c0,#16]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldr	c29,[csp],#8*__SIZEOF_POINTER__
	ret
.size	mul_mont_sparse_256,.-mul_mont_sparse_256
.globl	sqr_mont_sparse_256
.hidden	sqr_mont_sparse_256
.type	sqr_mont_sparse_256,%function
.align	5
sqr_mont_sparse_256:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	x5,x6,[c1]
	ldp	x7,x8,[c1,#16]
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
	ldr	c30,[c29,#__SIZEOF_POINTER__]

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

	csel	x10,x10,x14,lo
	csel	x11,x11,x15,lo
	csel	x12,x12,x16,lo
	csel	x13,x13,x17,lo

	stp	x10,x11,[c0]
	stp	x12,x13,[c0,#16]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	sqr_mont_sparse_256,.-sqr_mont_sparse_256
.globl	from_mont_256
.hidden	from_mont_256
.type	from_mont_256,%function
.align	5
from_mont_256:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-2*__SIZEOF_POINTER__]!
	add	c29,csp,#0

	mov	x4,x3
	ldp	x10,x11,[c1]
	ldp	x12,x13,[c1,#16]

	bl	__mul_by_1_mont_256
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	subs	x14,x10,x5
	sbcs	x15,x11,x6
	sbcs	x16,x12,x7
	sbcs	x17,x13,x8

	csel	x10,x10,x14,lo
	csel	x11,x11,x15,lo
	csel	x12,x12,x16,lo
	csel	x13,x13,x17,lo

	stp	x10,x11,[c0]
	stp	x12,x13,[c0,#16]

	ldr	c29,[csp],#2*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	from_mont_256,.-from_mont_256

.globl	redc_mont_256
.hidden	redc_mont_256
.type	redc_mont_256,%function
.align	5
redc_mont_256:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-2*__SIZEOF_POINTER__]!
	add	c29,csp,#0

	mov	x4,x3
	ldp	x10,x11,[c1]
	ldp	x12,x13,[c1,#16]

	bl	__mul_by_1_mont_256
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	x14,x15,[c1,#32]
	ldp	x16,x17,[c1,#48]

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

	csel	x10,x10,x14,lo
	csel	x11,x11,x15,lo
	csel	x12,x12,x16,lo
	csel	x13,x13,x17,lo

	stp	x10,x11,[c0]
	stp	x12,x13,[c0,#16]

	ldr	c29,[csp],#2*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	redc_mont_256,.-redc_mont_256

.type	__mul_by_1_mont_256,%function
.align	5
__mul_by_1_mont_256:
	mul	x3,x4,x10
	ldp	x5,x6,[c2]
	ldp	x7,x8,[c2,#16]
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
.size	__mul_by_1_mont_256,.-__mul_by_1_mont_256
