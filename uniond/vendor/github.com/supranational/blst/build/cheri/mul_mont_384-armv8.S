.text

.globl	add_mod_384x384
.type	add_mod_384x384,%function
.align	5
add_mod_384x384:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-8*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]

	ldp	x5,x6,[c3]
	ldp	x7,x8,[c3,#16]
	ldp	x9,x10,[c3,#32]

	bl	__add_mod_384x384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldr	c29,[csp],#8*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	add_mod_384x384,.-add_mod_384x384

.type	__add_mod_384x384,%function
.align	5
__add_mod_384x384:
	ldp	x11,  x12,  [c1]
	ldp	x19,x20,[c2]
	ldp	x13,  x14,  [c1,#16]
	adds	x11,x11,x19
	ldp	x21,x22,[c2,#16]
	adcs	x12,x12,x20
	ldp	x15,  x16,  [c1,#32]
	adcs	x13,x13,x21
	ldp	x23,x24,[c2,#32]
	adcs	x14,x14,x22
	stp	x11,  x12,  [c0]
	adcs	x15,x15,x23
	ldp	x11,  x12,  [c1,#48]
	adcs	x16,x16,x24

	ldp	x19,x20,[c2,#48]
	stp	x13,  x14,  [c0,#16]
	ldp	x13,  x14,  [c1,#64]
	ldp	x21,x22,[c2,#64]

	adcs	x11,x11,x19
	stp	x15,  x16,  [c0,#32]
	adcs	x12,x12,x20
	ldp	x15,  x16,  [c1,#80]
	adcs	x13,x13,x21
	ldp	x23,x24,[c2,#80]
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

	csel	x11,x11,x19,lo
	csel	x12,x12,x20,lo
	csel	x13,x13,x21,lo
	csel	x14,x14,x22,lo
	stp	x11,x12,[c0,#48]
	csel	x15,x15,x23,lo
	stp	x13,x14,[c0,#64]
	csel	x16,x16,x24,lo
	stp	x15,x16,[c0,#80]

	ret
.size	__add_mod_384x384,.-__add_mod_384x384

.globl	sub_mod_384x384
.type	sub_mod_384x384,%function
.align	5
sub_mod_384x384:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-8*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]

	ldp	x5,x6,[c3]
	ldp	x7,x8,[c3,#16]
	ldp	x9,x10,[c3,#32]

	bl	__sub_mod_384x384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldr	c29,[csp],#8*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	sub_mod_384x384,.-sub_mod_384x384

.type	__sub_mod_384x384,%function
.align	5
__sub_mod_384x384:
	ldp	x11,  x12,  [c1]
	ldp	x19,x20,[c2]
	ldp	x13,  x14,  [c1,#16]
	subs	x11,x11,x19
	ldp	x21,x22,[c2,#16]
	sbcs	x12,x12,x20
	ldp	x15,  x16,  [c1,#32]
	sbcs	x13,x13,x21
	ldp	x23,x24,[c2,#32]
	sbcs	x14,x14,x22
	stp	x11,  x12,  [c0]
	sbcs	x15,x15,x23
	ldp	x11,  x12,  [c1,#48]
	sbcs	x16,x16,x24

	ldp	x19,x20,[c2,#48]
	stp	x13,  x14,  [c0,#16]
	ldp	x13,  x14,  [c1,#64]
	ldp	x21,x22,[c2,#64]

	sbcs	x11,x11,x19
	stp	x15,  x16,  [c0,#32]
	sbcs	x12,x12,x20
	ldp	x15,  x16,  [c1,#80]
	sbcs	x13,x13,x21
	ldp	x23,x24,[c2,#80]
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
	stp	x11,x12,[c0,#48]
	adc	x16,x16,x24
	stp	x13,x14,[c0,#64]
	stp	x15,x16,[c0,#80]

	ret
.size	__sub_mod_384x384,.-__sub_mod_384x384

.type	__add_mod_384,%function
.align	5
__add_mod_384:
	ldp	x11,  x12,  [c1]
	ldp	x19,x20,[c2]
	ldp	x13,  x14,  [c1,#16]
	adds	x11,x11,x19
	ldp	x21,x22,[c2,#16]
	adcs	x12,x12,x20
	ldp	x15,  x16,  [c1,#32]
	adcs	x13,x13,x21
	ldp	x23,x24,[c2,#32]
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

	csel	x11,x11,x19,lo
	csel	x12,x12,x20,lo
	csel	x13,x13,x21,lo
	csel	x14,x14,x22,lo
	csel	x15,x15,x23,lo
	stp	x11,x12,[c0]
	csel	x16,x16,x24,lo
	stp	x13,x14,[c0,#16]
	stp	x15,x16,[c0,#32]

	ret
.size	__add_mod_384,.-__add_mod_384

.type	__sub_mod_384,%function
.align	5
__sub_mod_384:
	ldp	x11,  x12,  [c1]
	ldp	x19,x20,[c2]
	ldp	x13,  x14,  [c1,#16]
	subs	x11,x11,x19
	ldp	x21,x22,[c2,#16]
	sbcs	x12,x12,x20
	ldp	x15,  x16,  [c1,#32]
	sbcs	x13,x13,x21
	ldp	x23,x24,[c2,#32]
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
	stp	x11,x12,[c0]
	adc	x16,x16,x24
	stp	x13,x14,[c0,#16]
	stp	x15,x16,[c0,#32]

	ret
.size	__sub_mod_384,.-__sub_mod_384

.globl	mul_mont_384x
.hidden	mul_mont_384x
.type	mul_mont_384x,%function
.align	5
mul_mont_384x:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	sub	csp,csp,#288		// space for 3 768-bit vectors

	mov	c26,c0		// save r_ptr
	mov	c27,c1		// save b_ptr
	mov	c28,c2		// save b_ptr

	add	c0,csp,#0
	bl	__mul_384

	add	c1,c1,#48
	add	c2,c2,#48
	add	c0,csp,#96
	bl	__mul_384

	ldp	x5,x6,[c3]
	ldp	x7,x8,[c3,#16]
	ldp	x9,x10,[c3,#32]

	sub	c2,c1,#48
	add	c0,csp,#240
	bl	__add_mod_384

	add	c1,c28,#0
	add	c2,c28,#48
	add	c0,csp,#192
	bl	__add_mod_384

	add	c1,c0,#0
	add	c2,c0,#48
	bl	__mul_384		// mul_384(t2, a->re+a->im, b->re+b->im)

	ldp	x5,x6,[c3]
	ldp	x7,x8,[c3,#16]
	ldp	x9,x10,[c3,#32]

	mov	c1,c0
	add	c2,csp,#0
	bl	__sub_mod_384x384

	add	c2,csp,#96
	bl	__sub_mod_384x384	// t2 = t2-t0-t1

	add	c1,csp,#0
	add	c2,csp,#96
	add	c0,csp,#0
	bl	__sub_mod_384x384	// t0 = t0-t1

	add	c1,csp,#0
	add	c0,c26,#0
	bl	__mul_by_1_mont_384
	bl	__redc_tail_mont_384

	add	c1,csp,#192
	add	c0,c0,#48
	bl	__mul_by_1_mont_384
	bl	__redc_tail_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	add	csp,csp,#288
	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	mul_mont_384x,.-mul_mont_384x

.globl	sqr_mont_384x
.hidden	sqr_mont_384x
.type	sqr_mont_384x,%function
.align	5
sqr_mont_384x:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	stp	c3,c0,[csp,#12*__SIZEOF_POINTER__]	// __mul_mont_384 wants them there
	sub	csp,csp,#96		// space for 2 384-bit vectors
	mov	x4,x3		// adjust for missing b_ptr

	ldp	x5,x6,[c2]
	ldp	x7,x8,[c2,#16]
	ldp	x9,x10,[c2,#32]

	add	c2,c1,#48
	add	c0,csp,#0
	bl	__add_mod_384		// t0 = a->re + a->im

	add	c0,csp,#48
	bl	__sub_mod_384		// t1 = a->re - a->im

	ldp	x11,x12,[c1]
	ldr	x17,        [c2]
	ldp	x13,x14,[c1,#16]
	ldp	x15,x16,[c1,#32]

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

	csel	x19,x11,x19,lo
	csel	x20,x12,x20,lo
	csel	x21,x13,x21,lo
	ldp	x11,x12,[csp]
	csel	x22,x14,x22,lo
	ldr	x17,        [csp,#48]
	csel	x23,x15,x23,lo
	ldp	x13,x14,[csp,#16]
	csel	x24,x16,x24,lo
	ldp	x15,x16,[csp,#32]

	stp	x19,x20,[c2,#48]
	stp	x21,x22,[c2,#64]
	stp	x23,x24,[c2,#80]

	add	c2,csp,#48
	bl	__mul_mont_384		// mul_mont_384(ret->re, t0, t1)
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	stp	x11,x12,[c2]
	stp	x13,x14,[c2,#16]
	stp	x15,x16,[c2,#32]

	add	csp,csp,#96
	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	sqr_mont_384x,.-sqr_mont_384x

.globl	mul_mont_384
.hidden	mul_mont_384
.type	mul_mont_384,%function
.align	5
mul_mont_384:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	stp	c4,c0,[csp,#12*__SIZEOF_POINTER__]	// __mul_mont_384 wants them there

	ldp	x11,x12,[c1]
	ldr	x17,        [c2]
	ldp	x13,x14,[c1,#16]
	ldp	x15,x16,[c1,#32]

	ldp	x5,x6,[c3]
	ldp	x7,x8,[c3,#16]
	ldp	x9,x10,[c3,#32]

	bl	__mul_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	stp	x11,x12,[c2]
	stp	x13,x14,[c2,#16]
	stp	x15,x16,[c2,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	mul_mont_384,.-mul_mont_384

.type	__mul_mont_384,%function
.align	5
__mul_mont_384:
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
	ldr	x17,[c2,8*1]

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
	ldr	x4,[c29,#12*__SIZEOF_POINTER__]

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
	ldr	x17,[c2,8*2]

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
	ldr	x4,[c29,#12*__SIZEOF_POINTER__]

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
	ldr	x17,[c2,8*3]

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
	ldr	x4,[c29,#12*__SIZEOF_POINTER__]

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
	ldr	x17,[c2,8*4]

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
	ldr	x4,[c29,#12*__SIZEOF_POINTER__]

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
	ldr	x17,[c2,8*5]

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
	ldr	x4,[c29,#12*__SIZEOF_POINTER__]

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
	ldp	c4,c2,[c29,#12*__SIZEOF_POINTER__]	// pull r_ptr
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

	csel	x11,x19,x26,lo
	csel	x12,x20,x27,lo
	csel	x13,x21,x28,lo
	csel	x14,x22,x0,lo
	csel	x15,x23,x1,lo
	csel	x16,x24,x3,lo
	ret
.size	__mul_mont_384,.-__mul_mont_384

.globl	sqr_mont_384
.hidden	sqr_mont_384
.type	sqr_mont_384,%function
.align	5
sqr_mont_384:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	sub	csp,csp,#96		// space for 768-bit vector
	mov	c4,c3		// adjust for missing b_ptr

	mov	c3,c0		// save r_ptr
	mov	c0,csp

	ldp	x11,x12,[c1]
	ldp	x13,x14,[c1,#16]
	ldp	x15,x16,[c1,#32]

	bl	__sqr_384

	ldp	x5,x6,[c2]
	ldp	x7,x8,[c2,#16]
	ldp	x9,x10,[c2,#32]

	mov	c1,csp
	mov	c0,c3		// restore r_ptr
	bl	__mul_by_1_mont_384
	bl	__redc_tail_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	add	csp,csp,#96
	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	sqr_mont_384,.-sqr_mont_384

.globl	sqr_n_mul_mont_383
.hidden	sqr_n_mul_mont_383
.type	sqr_n_mul_mont_383,%function
.align	5
sqr_n_mul_mont_383:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	stp	c4,c0,[csp,#12*__SIZEOF_POINTER__]	// __mul_mont_384 wants them there
	sub	csp,csp,#96		// space for 768-bit vector
	mov	c17,c5			// save b_ptr

	ldp	x11,x12,[c1]
	ldp	x13,x14,[c1,#16]
	ldp	x15,x16,[c1,#32]
	mov	c0,csp
.Loop_sqr_383:
	bl	__sqr_384
	sub	x2,x2,#1	// counter

	ldp	x5,x6,[c3]
	ldp	x7,x8,[c3,#16]
	ldp	x9,x10,[c3,#32]

	mov	c1,csp
	bl	__mul_by_1_mont_384

	ldp	x19,x20,[c1,#48]
	ldp	x21,x22,[c1,#64]
	ldp	x23,x24,[c1,#80]

	adds	x11,x11,x19	// just accumulate upper half
	adcs	x12,x12,x20
	adcs	x13,x13,x21
	adcs	x14,x14,x22
	adcs	x15,x15,x23
	adc	x16,x16,x24

	cbnz	x2,.Loop_sqr_383

	mov	c2,c17
	ldr	x17,[c17]
	bl	__mul_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	stp	x11,x12,[c2]
	stp	x13,x14,[c2,#16]
	stp	x15,x16,[c2,#32]

	add	csp,csp,#96
	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	sqr_n_mul_mont_383,.-sqr_n_mul_mont_383
.type	__sqr_384,%function
.align	5
__sqr_384:
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
	stp	x5,x19,[c0]
	adcs	x26,x26,x9
	stp	x20,x21,[c0,#16]
	adcs	x27,x27,x15
	stp	x22,x23,[c0,#32]
	adcs	x28,x28,x10
	stp	x24,x25,[c0,#48]
	adc	x16,x16,x1
	stp	x26,x27,[c0,#64]
	stp	x28,x16,[c0,#80]

	ret
.size	__sqr_384,.-__sqr_384
.globl	sqr_384
.hidden	sqr_384
.type	sqr_384,%function
.align	5
sqr_384:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]

	ldp	x11,x12,[c1]
	ldp	x13,x14,[c1,#16]
	ldp	x15,x16,[c1,#32]

	bl	__sqr_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	sqr_384,.-sqr_384

.globl	redc_mont_384
.hidden	redc_mont_384
.type	redc_mont_384,%function
.align	5
redc_mont_384:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	mov	x4,x3		// adjust for missing b_ptr

	ldp	x5,x6,[c2]
	ldp	x7,x8,[c2,#16]
	ldp	x9,x10,[c2,#32]

	bl	__mul_by_1_mont_384
	bl	__redc_tail_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	redc_mont_384,.-redc_mont_384

.globl	from_mont_384
.hidden	from_mont_384
.type	from_mont_384,%function
.align	5
from_mont_384:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	mov	x4,x3		// adjust for missing b_ptr

	ldp	x5,x6,[c2]
	ldp	x7,x8,[c2,#16]
	ldp	x9,x10,[c2,#32]

	bl	__mul_by_1_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	subs	x19,x11,x5
	sbcs	x20,x12,x6
	sbcs	x21,x13,x7
	sbcs	x22,x14,x8
	sbcs	x23,x15,x9
	sbcs	x24,x16,x10

	csel	x11,x11,x19,lo
	csel	x12,x12,x20,lo
	csel	x13,x13,x21,lo
	csel	x14,x14,x22,lo
	csel	x15,x15,x23,lo
	csel	x16,x16,x24,lo

	stp	x11,x12,[c0]
	stp	x13,x14,[c0,#16]
	stp	x15,x16,[c0,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	from_mont_384,.-from_mont_384

.type	__mul_by_1_mont_384,%function
.align	5
__mul_by_1_mont_384:
	ldp	x11,x12,[c1]
	ldp	x13,x14,[c1,#16]
	mul	x26,x4,x11
	ldp	x15,x16,[c1,#32]

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
.size	__mul_by_1_mont_384,.-__mul_by_1_mont_384

.type	__redc_tail_mont_384,%function
.align	5
__redc_tail_mont_384:
	ldp	x19,x20,[c1,#48]
	ldp	x21,x22,[c1,#64]
	ldp	x23,x24,[c1,#80]

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

	csel	x11,x11,x19,lo
	csel	x12,x12,x20,lo
	csel	x13,x13,x21,lo
	csel	x14,x14,x22,lo
	csel	x15,x15,x23,lo
	csel	x16,x16,x24,lo

	stp	x11,x12,[c0]
	stp	x13,x14,[c0,#16]
	stp	x15,x16,[c0,#32]

	ret
.size	__redc_tail_mont_384,.-__redc_tail_mont_384

.globl	mul_384
.hidden	mul_384
.type	mul_384,%function
.align	5
mul_384:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]

	bl	__mul_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	mul_384,.-mul_384

.type	__mul_384,%function
.align	5
__mul_384:
	ldp	x11,x12,[c1]
	ldr	x17,        [c2]
	ldp	x13,x14,[c1,#16]
	ldp	x15,x16,[c1,#32]

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
	ldr	x17,[c2,8*1]

	str	x19,[c0]
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
	ldr	x17,[c2,#8*(1+1)]
	adc	x25,xzr,xzr

	str	x19,[c0,8*1]
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
	ldr	x17,[c2,#8*(2+1)]
	adc	x25,xzr,xzr

	str	x19,[c0,8*2]
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
	ldr	x17,[c2,#8*(3+1)]
	adc	x25,xzr,xzr

	str	x19,[c0,8*3]
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
	ldr	x17,[c2,#8*(4+1)]
	adc	x25,xzr,xzr

	str	x19,[c0,8*4]
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

	str	x19,[c0,8*5]
	adds	x19,x20,x5
	adcs	x20,x21,x6
	adcs	x21,x22,x7
	adcs	x22,x23,x8
	adcs	x23,x24,x9
	adc	x24,x25,x10

	stp	x19,x20,[c0,#48]
	stp	x21,x22,[c0,#64]
	stp	x23,x24,[c0,#80]

	ret
.size	__mul_384,.-__mul_384

.globl	mul_382x
.hidden	mul_382x
.type	mul_382x,%function
.align	5
mul_382x:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	sub	csp,csp,#96		// space for two 384-bit vectors

	ldp	x11,x12,[c1]
	mov	c26,c0		// save r_ptr
	ldp	x19,x20,[c1,#48]
	mov	c27,c1		// save a_ptr
	ldp	x13,x14,[c1,#16]
	mov	c28,c2		// save b_ptr
	ldp	x21,x22,[c1,#64]
	ldp	x15,x16,[c1,#32]
	adds	x5,x11,x19	// t0 = a->re + a->im
	ldp	x23,x24,[c1,#80]
	adcs	x6,x12,x20
	ldp	x11,x12,[c2]
	adcs	x7,x13,x21
	ldp	x19,x20,[c2,#48]
	adcs	x8,x14,x22
	ldp	x13,x14,[c2,#16]
	adcs	x9,x15,x23
	ldp	x21,x22,[c2,#64]
	adc	x10,x16,x24
	ldp	x15,x16,[c2,#32]

	stp	x5,x6,[csp]
	adds	x5,x11,x19	// t1 = b->re + b->im
	ldp	x23,x24,[c2,#80]
	adcs	x6,x12,x20
	stp	x7,x8,[csp,#16]
	adcs	x7,x13,x21
	adcs	x8,x14,x22
	stp	x9,x10,[csp,#32]
	adcs	x9,x15,x23
	stp	x5,x6,[csp,#48]
	adc	x10,x16,x24
	stp	x7,x8,[csp,#64]
	stp	x9,x10,[csp,#80]

	bl	__mul_384		// mul_384(ret->re, a->re, b->re)

	add	c1,csp,#0
	add	c2,csp,#48
	add	c0,c26,#96
	bl	__mul_384

	add	c1,c27,#48
	add	c2,c28,#48
	add	c0,csp,#0
	bl	__mul_384

	ldp	x5,x6,[c3]
	ldp	x7,x8,[c3,#16]
	ldp	x9,x10,[c3,#32]

	add	c1,c26,#96
	add	c2,csp,#0
	add	c0,c26,#96
	bl	__sub_mod_384x384

	add	c2,c26,#0
	bl	__sub_mod_384x384

	add	c1,c26,#0
	add	c2,csp,#0
	add	c0,c26,#0
	bl	__sub_mod_384x384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	add	csp,csp,#96
	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	mul_382x,.-mul_382x

.globl	sqr_382x
.hidden	sqr_382x
.type	sqr_382x,%function
.align	5
sqr_382x:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]

	ldp	x11,x12,[c1]
	ldp	x19,x20,[c1,#48]
	ldp	x13,x14,[c1,#16]
	adds	x5,x11,x19	// t0 = a->re + a->im
	ldp	x21,x22,[c1,#64]
	adcs	x6,x12,x20
	ldp	x15,x16,[c1,#32]
	adcs	x7,x13,x21
	ldp	x23,x24,[c1,#80]
	adcs	x8,x14,x22
	stp	x5,x6,[c0]
	adcs	x9,x15,x23
	ldp	x5,x6,[c2]
	adc	x10,x16,x24
	stp	x7,x8,[c0,#16]

	subs	x11,x11,x19	// t1 = a->re - a->im
	ldp	x7,x8,[c2,#16]
	sbcs	x12,x12,x20
	stp	x9,x10,[c0,#32]
	sbcs	x13,x13,x21
	ldp	x9,x10,[c2,#32]
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
	stp	x11,x12,[c0,#48]
	adc	x16,x16,x24
	stp	x13,x14,[c0,#64]
	stp	x15,x16,[c0,#80]

	mov	c4,c1		// save a_ptr
	add	c1,c0,#0
	add	c2,c0,#48
	bl	__mul_384

	add	c1,c4,#0
	add	c2,c4,#48
	add	c0,c0,#96
	bl	__mul_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	x11,x12,[c0]
	ldp	x13,x14,[c0,#16]
	adds	x11,x11,x11	// add with itself
	ldp	x15,x16,[c0,#32]
	adcs	x12,x12,x12
	adcs	x13,x13,x13
	adcs	x14,x14,x14
	adcs	x15,x15,x15
	adcs	x16,x16,x16
	adcs	x19,x19,x19
	adcs	x20,x20,x20
	stp	x11,x12,[c0]
	adcs	x21,x21,x21
	stp	x13,x14,[c0,#16]
	adcs	x22,x22,x22
	stp	x15,x16,[c0,#32]
	adcs	x23,x23,x23
	stp	x19,x20,[c0,#48]
	adc	x24,x24,x24
	stp	x21,x22,[c0,#64]
	stp	x23,x24,[c0,#80]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	sqr_382x,.-sqr_382x

.globl	sqr_mont_382x
.hidden	sqr_mont_382x
.type	sqr_mont_382x,%function
.align	5
sqr_mont_382x:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	stp	c3,c0,[csp,#12*__SIZEOF_POINTER__]	// __mul_mont_384 wants them there
	sub	csp,csp,#112		// space for two 384-bit vectors + word
	mov	x4,x3		// adjust for missing b_ptr

	ldp	x11,x12,[c1]
	ldp	x13,x14,[c1,#16]
	ldp	x15,x16,[c1,#32]

	ldp	x17,x20,[c1,#48]
	ldp	x21,x22,[c1,#64]
	ldp	x23,x24,[c1,#80]

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

	stp	x5,x6,[csp]
	stp	x7,x8,[csp,#16]
	stp	x9,x10,[csp,#32]
	stp	x19,x20,[csp,#48]
	stp	x21,x22,[csp,#64]
	stp	x23,x24,[csp,#80]
	str	x25,[csp,#96]

	ldp	x5,x6,[c2]
	ldp	x7,x8,[c2,#16]
	ldp	x9,x10,[c2,#32]

	add	c2,c1,#48
	bl	__mul_mont_383_nonred	// mul_mont_384(ret->im, a->re, a->im)

	adds	x19,x11,x11	// add with itself
	adcs	x20,x12,x12
	adcs	x21,x13,x13
	adcs	x22,x14,x14
	adcs	x23,x15,x15
	adc	x24,x16,x16

	stp	x19,x20,[c2,#48]
	stp	x21,x22,[c2,#64]
	stp	x23,x24,[c2,#80]

	ldp	x11,x12,[csp]
	ldr	x17,[csp,#48]
	ldp	x13,x14,[csp,#16]
	ldp	x15,x16,[csp,#32]

	add	c2,csp,#48
	bl	__mul_mont_383_nonred	// mul_mont_384(ret->im, t0, t1)
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldr	x25,[csp,#96]	// account for sign from a->re - a->im
	ldp	x19,x20,[csp]
	ldp	x21,x22,[csp,#16]
	ldp	x23,x24,[csp,#32]

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

	stp	x11,x12,[c2]
	stp	x13,x14,[c2,#16]
	stp	x15,x16,[c2,#32]

	add	csp,csp,#112
	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	sqr_mont_382x,.-sqr_mont_382x

.type	__mul_mont_383_nonred,%function
.align	5
__mul_mont_383_nonred:
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
	ldr	x17,[c2,8*1]
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

	ldr	x4,[c29,#12*__SIZEOF_POINTER__]
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
	ldr	x17,[c2,8*2]
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

	ldr	x4,[c29,#12*__SIZEOF_POINTER__]
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
	ldr	x17,[c2,8*3]
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

	ldr	x4,[c29,#12*__SIZEOF_POINTER__]
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
	ldr	x17,[c2,8*4]
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

	ldr	x4,[c29,#12*__SIZEOF_POINTER__]
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
	ldr	x17,[c2,8*5]
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

	ldr	x4,[c29,#12*__SIZEOF_POINTER__]
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
	ldp	c4,c2,[c29,#12*__SIZEOF_POINTER__]		// pull r_ptr

	adds	x11,x20,x26
	adcs	x12,x21,x27
	adcs	x13,x22,x28
	adcs	x14,x23,x0
	adcs	x15,x24,x1
	adcs	x16,x25,x3

	ret
.size	__mul_mont_383_nonred,.-__mul_mont_383_nonred

.globl	sgn0_pty_mont_384
.hidden	sgn0_pty_mont_384
.type	sgn0_pty_mont_384,%function
.align	5
sgn0_pty_mont_384:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]

	mov	x4,x2
	ldp	x5,x6,[c1]
	ldp	x7,x8,[c1,#16]
	ldp	x9,x10,[c1,#32]
	mov	c1,c0

	bl	__mul_by_1_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

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

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	sgn0_pty_mont_384,.-sgn0_pty_mont_384

.globl	sgn0_pty_mont_384x
.hidden	sgn0_pty_mont_384x
.type	sgn0_pty_mont_384x,%function
.align	5
sgn0_pty_mont_384x:
	.inst	0xd503233f
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]

	mov	x4,x2
	ldp	x5,x6,[c1]
	ldp	x7,x8,[c1,#16]
	ldp	x9,x10,[c1,#32]
	mov	c1,c0

	bl	__mul_by_1_mont_384
	add	c1,c1,#48

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
	ldr	c30,[c29,#__SIZEOF_POINTER__]

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
	csel	x3,x0,x2,eq	// a->re==0? prty(a->im) : prty(a->re)

	cmp	x1,#0
	csel	x1,x0,x2,ne	// a->im!=0? sgn0(a->im) : sgn0(a->re)

	and	x3,x3,#1
	and	x1,x1,#2
	orr	x0,x1,x3		// pack sign and parity

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	.inst	0xd50323bf
	ret
.size	sgn0_pty_mont_384x,.-sgn0_pty_mont_384x
