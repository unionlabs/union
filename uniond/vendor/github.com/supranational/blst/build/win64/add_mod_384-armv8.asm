	AREA	|.text|,CODE,ALIGN=8,ARM64



	EXPORT	|add_mod_384|[FUNC]
	ALIGN	32
|add_mod_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x4,x5,[x3]
	ldp	x6,x7,[x3,#16]
	ldp	x8,x9,[x3,#32]

	bl	__add_mod_384
	ldr	x30,[sp,#8]

	stp	x10,x11,[x0]
	stp	x12,x13,[x0,#16]
	stp	x14,x15,[x0,#32]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP


	ALIGN	32
|__add_mod_384| PROC
	ldp	x10,x11,[x1]
	ldp	x16,x17,[x2]
	ldp	x12,x13,[x1,#16]
	ldp	x19,x20,[x2,#16]
	ldp	x14,x15,[x1,#32]
	ldp	x21,x22,[x2,#32]

|__add_mod_384_ab_are_loaded|
	adds	x10,x10,x16
	adcs	x11,x11,x17
	adcs	x12,x12,x19
	adcs	x13,x13,x20
	adcs	x14,x14,x21
	adcs	x15,x15,x22
	adc	x3,xzr,xzr

	subs	x16,x10,x4
	sbcs	x17,x11,x5
	sbcs	x19,x12,x6
	sbcs	x20,x13,x7
	sbcs	x21,x14,x8
	sbcs	x22,x15,x9
	sbcs	xzr,x3,xzr

	csello	x10,x10,x16
	csello	x11,x11,x17
	csello	x12,x12,x19
	csello	x13,x13,x20
	csello	x14,x14,x21
	csello	x15,x15,x22

	ret
	ENDP



	EXPORT	|add_mod_384x|[FUNC]
	ALIGN	32
|add_mod_384x| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x4,x5,[x3]
	ldp	x6,x7,[x3,#16]
	ldp	x8,x9,[x3,#32]

	bl	__add_mod_384

	stp	x10,x11,[x0]
	add	x1,x1,#48
	stp	x12,x13,[x0,#16]
	add	x2,x2,#48
	stp	x14,x15,[x0,#32]

	bl	__add_mod_384
	ldr	x30,[sp,#8]

	stp	x10,x11,[x0,#48]
	stp	x12,x13,[x0,#64]
	stp	x14,x15,[x0,#80]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|rshift_mod_384|[FUNC]
	ALIGN	32
|rshift_mod_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x10,x11,[x1]
	ldp	x12,x13,[x1,#16]
	ldp	x14,x15,[x1,#32]

	ldp	x4,x5,[x3]
	ldp	x6,x7,[x3,#16]
	ldp	x8,x9,[x3,#32]

|$Loop_rshift_mod_384|
	sub	x2,x2,#1
	bl	__rshift_mod_384
	cbnz	x2,|$Loop_rshift_mod_384|

	ldr	x30,[sp,#8]
	stp	x10,x11,[x0]
	stp	x12,x13,[x0,#16]
	stp	x14,x15,[x0,#32]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP


	ALIGN	32
|__rshift_mod_384| PROC
	sbfx	x22,x10,#0,#1
	and	x16,x22,x4
	and	x17,x22,x5
	adds	x10,x10,x16
	and	x19,x22,x6
	adcs	x11,x11,x17
	and	x20,x22,x7
	adcs	x12,x12,x19
	and	x21,x22,x8
	adcs	x13,x13,x20
	and	x22,x22,x9
	adcs	x14,x14,x21
	extr	x10,x11,x10,#1	// a[0:5] >>= 1
	adcs	x15,x15,x22
	extr	x11,x12,x11,#1
	adc	x22,xzr,xzr
	extr	x12,x13,x12,#1
	extr	x13,x14,x13,#1
	extr	x14,x15,x14,#1
	extr	x15,x22,x15,#1
	ret
	ENDP



	EXPORT	|div_by_2_mod_384|[FUNC]
	ALIGN	32
|div_by_2_mod_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x10,x11,[x1]
	ldp	x12,x13,[x1,#16]
	ldp	x14,x15,[x1,#32]

	ldp	x4,x5,[x2]
	ldp	x6,x7,[x2,#16]
	ldp	x8,x9,[x2,#32]

	bl	__rshift_mod_384

	ldr	x30,[sp,#8]
	stp	x10,x11,[x0]
	stp	x12,x13,[x0,#16]
	stp	x14,x15,[x0,#32]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|lshift_mod_384|[FUNC]
	ALIGN	32
|lshift_mod_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x10,x11,[x1]
	ldp	x12,x13,[x1,#16]
	ldp	x14,x15,[x1,#32]

	ldp	x4,x5,[x3]
	ldp	x6,x7,[x3,#16]
	ldp	x8,x9,[x3,#32]

|$Loop_lshift_mod_384|
	sub	x2,x2,#1
	bl	__lshift_mod_384
	cbnz	x2,|$Loop_lshift_mod_384|

	ldr	x30,[sp,#8]
	stp	x10,x11,[x0]
	stp	x12,x13,[x0,#16]
	stp	x14,x15,[x0,#32]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP


	ALIGN	32
|__lshift_mod_384| PROC
	adds	x10,x10,x10
	adcs	x11,x11,x11
	adcs	x12,x12,x12
	adcs	x13,x13,x13
	adcs	x14,x14,x14
	adcs	x15,x15,x15
	adc	x3,xzr,xzr

	subs	x16,x10,x4
	sbcs	x17,x11,x5
	sbcs	x19,x12,x6
	sbcs	x20,x13,x7
	sbcs	x21,x14,x8
	sbcs	x22,x15,x9
	sbcs	xzr,x3,xzr

	csello	x10,x10,x16
	csello	x11,x11,x17
	csello	x12,x12,x19
	csello	x13,x13,x20
	csello	x14,x14,x21
	csello	x15,x15,x22

	ret
	ENDP



	EXPORT	|mul_by_3_mod_384|[FUNC]
	ALIGN	32
|mul_by_3_mod_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x10,x11,[x1]
	ldp	x12,x13,[x1,#16]
	ldp	x14,x15,[x1,#32]

	ldp	x4,x5,[x2]
	ldp	x6,x7,[x2,#16]
	ldp	x8,x9,[x2,#32]

	bl	__lshift_mod_384

	ldp	x16,x17,[x1]
	ldp	x19,x20,[x1,#16]
	ldp	x21,x22,[x1,#32]

	bl	__add_mod_384_ab_are_loaded
	ldr	x30,[sp,#8]

	stp	x10,x11,[x0]
	stp	x12,x13,[x0,#16]
	stp	x14,x15,[x0,#32]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|mul_by_8_mod_384|[FUNC]
	ALIGN	32
|mul_by_8_mod_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x10,x11,[x1]
	ldp	x12,x13,[x1,#16]
	ldp	x14,x15,[x1,#32]

	ldp	x4,x5,[x2]
	ldp	x6,x7,[x2,#16]
	ldp	x8,x9,[x2,#32]

	bl	__lshift_mod_384
	bl	__lshift_mod_384
	bl	__lshift_mod_384
	ldr	x30,[sp,#8]

	stp	x10,x11,[x0]
	stp	x12,x13,[x0,#16]
	stp	x14,x15,[x0,#32]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|mul_by_3_mod_384x|[FUNC]
	ALIGN	32
|mul_by_3_mod_384x| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x10,x11,[x1]
	ldp	x12,x13,[x1,#16]
	ldp	x14,x15,[x1,#32]

	ldp	x4,x5,[x2]
	ldp	x6,x7,[x2,#16]
	ldp	x8,x9,[x2,#32]

	bl	__lshift_mod_384

	ldp	x16,x17,[x1]
	ldp	x19,x20,[x1,#16]
	ldp	x21,x22,[x1,#32]

	bl	__add_mod_384_ab_are_loaded

	stp	x10,x11,[x0]
	ldp	x10,x11,[x1,#48]
	stp	x12,x13,[x0,#16]
	ldp	x12,x13,[x1,#64]
	stp	x14,x15,[x0,#32]
	ldp	x14,x15,[x1,#80]

	bl	__lshift_mod_384

	ldp	x16,x17,[x1,#48]
	ldp	x19,x20,[x1,#64]
	ldp	x21,x22,[x1,#80]

	bl	__add_mod_384_ab_are_loaded
	ldr	x30,[sp,#8]

	stp	x10,x11,[x0,#48]
	stp	x12,x13,[x0,#64]
	stp	x14,x15,[x0,#80]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|mul_by_8_mod_384x|[FUNC]
	ALIGN	32
|mul_by_8_mod_384x| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x10,x11,[x1]
	ldp	x12,x13,[x1,#16]
	ldp	x14,x15,[x1,#32]

	ldp	x4,x5,[x2]
	ldp	x6,x7,[x2,#16]
	ldp	x8,x9,[x2,#32]

	bl	__lshift_mod_384
	bl	__lshift_mod_384
	bl	__lshift_mod_384

	stp	x10,x11,[x0]
	ldp	x10,x11,[x1,#48]
	stp	x12,x13,[x0,#16]
	ldp	x12,x13,[x1,#64]
	stp	x14,x15,[x0,#32]
	ldp	x14,x15,[x1,#80]

	bl	__lshift_mod_384
	bl	__lshift_mod_384
	bl	__lshift_mod_384
	ldr	x30,[sp,#8]

	stp	x10,x11,[x0,#48]
	stp	x12,x13,[x0,#64]
	stp	x14,x15,[x0,#80]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|cneg_mod_384|[FUNC]
	ALIGN	32
|cneg_mod_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x10,x11,[x1]
	ldp	x4,x5,[x3]
	ldp	x12,x13,[x1,#16]
	ldp	x6,x7,[x3,#16]

	subs	x16,x4,x10
	ldp	x14,x15,[x1,#32]
	ldp	x8,x9,[x3,#32]
	orr	x3,x10,x11
	sbcs	x17,x5,x11
	orr	x3,x3,x12
	sbcs	x19,x6,x12
	orr	x3,x3,x13
	sbcs	x20,x7,x13
	orr	x3,x3,x14
	sbcs	x21,x8,x14
	orr	x3,x3,x15
	sbc	x22,x9,x15

	cmp	x3,#0
	csetmne	x3
	ands	x2,x2,x3

	cseleq	x10,x10,x16
	cseleq	x11,x11,x17
	cseleq	x12,x12,x19
	cseleq	x13,x13,x20
	stp	x10,x11,[x0]
	cseleq	x14,x14,x21
	stp	x12,x13,[x0,#16]
	cseleq	x15,x15,x22
	stp	x14,x15,[x0,#32]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|sub_mod_384|[FUNC]
	ALIGN	32
|sub_mod_384| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x4,x5,[x3]
	ldp	x6,x7,[x3,#16]
	ldp	x8,x9,[x3,#32]

	bl	__sub_mod_384
	ldr	x30,[sp,#8]

	stp	x10,x11,[x0]
	stp	x12,x13,[x0,#16]
	stp	x14,x15,[x0,#32]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP


	ALIGN	32
|__sub_mod_384| PROC
	ldp	x10,x11,[x1]
	ldp	x16,x17,[x2]
	ldp	x12,x13,[x1,#16]
	ldp	x19,x20,[x2,#16]
	ldp	x14,x15,[x1,#32]
	ldp	x21,x22,[x2,#32]

	subs	x10,x10,x16
	sbcs	x11,x11,x17
	sbcs	x12,x12,x19
	sbcs	x13,x13,x20
	sbcs	x14,x14,x21
	sbcs	x15,x15,x22
	sbc	x3,xzr,xzr

	and	x16,x4,x3
	and	x17,x5,x3
	adds	x10,x10,x16
	and	x19,x6,x3
	adcs	x11,x11,x17
	and	x20,x7,x3
	adcs	x12,x12,x19
	and	x21,x8,x3
	adcs	x13,x13,x20
	and	x22,x9,x3
	adcs	x14,x14,x21
	adc	x15,x15,x22

	ret
	ENDP



	EXPORT	|sub_mod_384x|[FUNC]
	ALIGN	32
|sub_mod_384x| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x4,x5,[x3]
	ldp	x6,x7,[x3,#16]
	ldp	x8,x9,[x3,#32]

	bl	__sub_mod_384

	stp	x10,x11,[x0]
	add	x1,x1,#48
	stp	x12,x13,[x0,#16]
	add	x2,x2,#48
	stp	x14,x15,[x0,#32]

	bl	__sub_mod_384
	ldr	x30,[sp,#8]

	stp	x10,x11,[x0,#48]
	stp	x12,x13,[x0,#64]
	stp	x14,x15,[x0,#80]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|mul_by_1_plus_i_mod_384x|[FUNC]
	ALIGN	32
|mul_by_1_plus_i_mod_384x| PROC
	DCDU	3573752639
	stp	x29,x30,[sp,#-48]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]

	ldp	x4,x5,[x2]
	ldp	x6,x7,[x2,#16]
	ldp	x8,x9,[x2,#32]
	add	x2,x1,#48

	bl	__sub_mod_384			// a->re - a->im

	ldp	x16,x17,[x1]
	ldp	x19,x20,[x1,#16]
	ldp	x21,x22,[x1,#32]
	stp	x10,x11,[x0]
	ldp	x10,x11,[x1,#48]
	stp	x12,x13,[x0,#16]
	ldp	x12,x13,[x1,#64]
	stp	x14,x15,[x0,#32]
	ldp	x14,x15,[x1,#80]

	bl	__add_mod_384_ab_are_loaded	// a->re + a->im
	ldr	x30,[sp,#8]

	stp	x10,x11,[x0,#48]
	stp	x12,x13,[x0,#64]
	stp	x14,x15,[x0,#80]

	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldr	x29,[sp],#48
	DCDU	3573752767
	ret
	ENDP



	EXPORT	|sgn0_pty_mod_384|[FUNC]
	ALIGN	32
|sgn0_pty_mod_384| PROC
	ldp	x10,x11,[x0]
	ldp	x12,x13,[x0,#16]
	ldp	x14,x15,[x0,#32]

	ldp	x4,x5,[x1]
	ldp	x6,x7,[x1,#16]
	ldp	x8,x9,[x1,#32]

	and	x0,x10,#1
	adds	x10,x10,x10
	adcs	x11,x11,x11
	adcs	x12,x12,x12
	adcs	x13,x13,x13
	adcs	x14,x14,x14
	adcs	x15,x15,x15
	adc	x3,xzr,xzr

	subs	x10,x10,x4
	sbcs	x11,x11,x5
	sbcs	x12,x12,x6
	sbcs	x13,x13,x7
	sbcs	x14,x14,x8
	sbcs	x15,x15,x9
	sbc	x3,x3,xzr

	mvn	x3,x3
	and	x3,x3,#2
	orr	x0,x0,x3

	ret
	ENDP



	EXPORT	|sgn0_pty_mod_384x|[FUNC]
	ALIGN	32
|sgn0_pty_mod_384x| PROC
	ldp	x10,x11,[x0]
	ldp	x12,x13,[x0,#16]
	ldp	x14,x15,[x0,#32]

	ldp	x4,x5,[x1]
	ldp	x6,x7,[x1,#16]
	ldp	x8,x9,[x1,#32]

	and	x2,x10,#1
	orr	x3,x10,x11
	adds	x10,x10,x10
	orr	x3,x3,x12
	adcs	x11,x11,x11
	orr	x3,x3,x13
	adcs	x12,x12,x12
	orr	x3,x3,x14
	adcs	x13,x13,x13
	orr	x3,x3,x15
	adcs	x14,x14,x14
	adcs	x15,x15,x15
	adc	x16,xzr,xzr

	subs	x10,x10,x4
	sbcs	x11,x11,x5
	sbcs	x12,x12,x6
	sbcs	x13,x13,x7
	sbcs	x14,x14,x8
	sbcs	x15,x15,x9
	sbc	x16,x16,xzr

	ldp	x10,x11,[x0,#48]
	ldp	x12,x13,[x0,#64]
	ldp	x14,x15,[x0,#80]

	mvn	x16,x16
	and	x16,x16,#2
	orr	x2,x2,x16

	and	x0,x10,#1
	orr	x1,x10,x11
	adds	x10,x10,x10
	orr	x1,x1,x12
	adcs	x11,x11,x11
	orr	x1,x1,x13
	adcs	x12,x12,x12
	orr	x1,x1,x14
	adcs	x13,x13,x13
	orr	x1,x1,x15
	adcs	x14,x14,x14
	adcs	x15,x15,x15
	adc	x16,xzr,xzr

	subs	x10,x10,x4
	sbcs	x11,x11,x5
	sbcs	x12,x12,x6
	sbcs	x13,x13,x7
	sbcs	x14,x14,x8
	sbcs	x15,x15,x9
	sbc	x16,x16,xzr

	mvn	x16,x16
	and	x16,x16,#2
	orr	x0,x0,x16

	cmp	x3,#0
	cseleq	x3,x0,x2

	cmp	x1,#0
	cselne	x1,x0,x2

	and	x3,x3,#1
	and	x1,x1,#2
	orr	x0,x1,x3	// pack sign and parity

	ret
	ENDP


	EXPORT	|vec_select_32|[FUNC]
	ALIGN	32
|vec_select_32| PROC
	dup	v6.2d, x3
	ld1	{v0.2d, v1.2d, v2.2d}, [x1],#48
	cmeq	v6.2d, v6.2d, #0
	ld1	{v3.2d, v4.2d, v5.2d}, [x2],#48
	bit	v0.16b, v3.16b, v6.16b
	bit	v1.16b, v4.16b, v6.16b
	bit	v2.16b, v5.16b, v6.16b
	st1	{v0.2d, v1.2d, v2.2d}, [x0]
	ret
	ENDP


	EXPORT	|vec_select_48|[FUNC]
	ALIGN	32
|vec_select_48| PROC
	dup	v6.2d, x3
	ld1	{v0.2d, v1.2d, v2.2d}, [x1],#48
	cmeq	v6.2d, v6.2d, #0
	ld1	{v3.2d, v4.2d, v5.2d}, [x2],#48
	bit	v0.16b, v3.16b, v6.16b
	bit	v1.16b, v4.16b, v6.16b
	bit	v2.16b, v5.16b, v6.16b
	st1	{v0.2d, v1.2d, v2.2d}, [x0]
	ret
	ENDP


	EXPORT	|vec_select_96|[FUNC]
	ALIGN	32
|vec_select_96| PROC
	dup	v6.2d, x3
	ld1	{v0.2d, v1.2d, v2.2d}, [x1],#48
	cmeq	v6.2d, v6.2d, #0
	ld1	{v3.2d, v4.2d, v5.2d}, [x2],#48
	bit	v0.16b, v3.16b, v6.16b
	ld1	{v16.2d, v17.2d, v18.2d}, [x1],#48
	bit	v1.16b, v4.16b, v6.16b
	ld1	{v19.2d, v20.2d, v21.2d}, [x2],#48
	bit	v2.16b, v5.16b, v6.16b
	st1	{v0.2d, v1.2d, v2.2d}, [x0],#48
	bit	v16.16b, v19.16b, v6.16b
	bit	v17.16b, v20.16b, v6.16b
	bit	v18.16b, v21.16b, v6.16b
	st1	{v16.2d, v17.2d, v18.2d}, [x0]
	ret
	ENDP


	EXPORT	|vec_select_192|[FUNC]
	ALIGN	32
|vec_select_192| PROC
	dup	v6.2d, x3
	ld1	{v0.2d, v1.2d, v2.2d}, [x1],#48
	cmeq	v6.2d, v6.2d, #0
	ld1	{v3.2d, v4.2d, v5.2d}, [x2],#48
	bit	v0.16b, v3.16b, v6.16b
	ld1	{v16.2d, v17.2d, v18.2d}, [x1],#48
	bit	v1.16b, v4.16b, v6.16b
	ld1	{v19.2d, v20.2d, v21.2d}, [x2],#48
	bit	v2.16b, v5.16b, v6.16b
	st1	{v0.2d, v1.2d, v2.2d}, [x0],#48
	bit	v16.16b, v19.16b, v6.16b
	ld1	{v0.2d, v1.2d, v2.2d}, [x1],#48
	bit	v17.16b, v20.16b, v6.16b
	ld1	{v3.2d, v4.2d, v5.2d}, [x2],#48
	bit	v18.16b, v21.16b, v6.16b
	st1	{v16.2d, v17.2d, v18.2d}, [x0],#48
	bit	v0.16b, v3.16b, v6.16b
	ld1	{v16.2d, v17.2d, v18.2d}, [x1],#48
	bit	v1.16b, v4.16b, v6.16b
	ld1	{v19.2d, v20.2d, v21.2d}, [x2],#48
	bit	v2.16b, v5.16b, v6.16b
	st1	{v0.2d, v1.2d, v2.2d}, [x0],#48
	bit	v16.16b, v19.16b, v6.16b
	bit	v17.16b, v20.16b, v6.16b
	bit	v18.16b, v21.16b, v6.16b
	st1	{v16.2d, v17.2d, v18.2d}, [x0]
	ret
	ENDP


	EXPORT	|vec_select_144|[FUNC]
	ALIGN	32
|vec_select_144| PROC
	dup	v6.2d, x3
	ld1	{v0.2d, v1.2d, v2.2d}, [x1],#48
	cmeq	v6.2d, v6.2d, #0
	ld1	{v3.2d, v4.2d, v5.2d}, [x2],#48
	bit	v0.16b, v3.16b, v6.16b
	ld1	{v16.2d, v17.2d, v18.2d}, [x1],#48
	bit	v1.16b, v4.16b, v6.16b
	ld1	{v19.2d, v20.2d, v21.2d}, [x2],#48
	bit	v2.16b, v5.16b, v6.16b
	st1	{v0.2d, v1.2d, v2.2d}, [x0],#48
	bit	v16.16b, v19.16b, v6.16b
	ld1	{v0.2d, v1.2d, v2.2d}, [x1],#48
	bit	v17.16b, v20.16b, v6.16b
	ld1	{v3.2d, v4.2d, v5.2d}, [x2],#48
	bit	v18.16b, v21.16b, v6.16b
	st1	{v16.2d, v17.2d, v18.2d}, [x0],#48
	bit	v0.16b, v3.16b, v6.16b
	bit	v1.16b, v4.16b, v6.16b
	bit	v2.16b, v5.16b, v6.16b
	st1	{v0.2d, v1.2d, v2.2d}, [x0]
	ret
	ENDP


	EXPORT	|vec_select_288|[FUNC]
	ALIGN	32
|vec_select_288| PROC
	dup	v6.2d, x3
	ld1	{v0.2d, v1.2d, v2.2d}, [x1],#48
	cmeq	v6.2d, v6.2d, #0
	ld1	{v3.2d, v4.2d, v5.2d}, [x2],#48
	bit	v0.16b, v3.16b, v6.16b
	ld1	{v16.2d, v17.2d, v18.2d}, [x1],#48
	bit	v1.16b, v4.16b, v6.16b
	ld1	{v19.2d, v20.2d, v21.2d}, [x2],#48
	bit	v2.16b, v5.16b, v6.16b
	st1	{v0.2d, v1.2d, v2.2d}, [x0],#48
	bit	v16.16b, v19.16b, v6.16b
	ld1	{v0.2d, v1.2d, v2.2d}, [x1],#48
	bit	v17.16b, v20.16b, v6.16b
	ld1	{v3.2d, v4.2d, v5.2d}, [x2],#48
	bit	v18.16b, v21.16b, v6.16b
	st1	{v16.2d, v17.2d, v18.2d}, [x0],#48
	bit	v0.16b, v3.16b, v6.16b
	ld1	{v16.2d, v17.2d, v18.2d}, [x1],#48
	bit	v1.16b, v4.16b, v6.16b
	ld1	{v19.2d, v20.2d, v21.2d}, [x2],#48
	bit	v2.16b, v5.16b, v6.16b
	st1	{v0.2d, v1.2d, v2.2d}, [x0],#48
	bit	v16.16b, v19.16b, v6.16b
	ld1	{v0.2d, v1.2d, v2.2d}, [x1],#48
	bit	v17.16b, v20.16b, v6.16b
	ld1	{v3.2d, v4.2d, v5.2d}, [x2],#48
	bit	v18.16b, v21.16b, v6.16b
	st1	{v16.2d, v17.2d, v18.2d}, [x0],#48
	bit	v0.16b, v3.16b, v6.16b
	ld1	{v16.2d, v17.2d, v18.2d}, [x1],#48
	bit	v1.16b, v4.16b, v6.16b
	ld1	{v19.2d, v20.2d, v21.2d}, [x2],#48
	bit	v2.16b, v5.16b, v6.16b
	st1	{v0.2d, v1.2d, v2.2d}, [x0],#48
	bit	v16.16b, v19.16b, v6.16b
	bit	v17.16b, v20.16b, v6.16b
	bit	v18.16b, v21.16b, v6.16b
	st1	{v16.2d, v17.2d, v18.2d}, [x0]
	ret
	ENDP


	EXPORT	|vec_prefetch|[FUNC]
	ALIGN	32
|vec_prefetch| PROC
	add	x1, x1, x0
	sub	x1, x1, #1
	mov	x2, #64
	prfm	pldl1keep, [x0]
	add	x0, x0, x2
	cmp	x0, x1
	cselhi	x0,x1,x0
	cselhi	x2,xzr,x2
	prfm	pldl1keep, [x0]
	add	x0, x0, x2
	cmp	x0, x1
	cselhi	x0,x1,x0
	cselhi	x2,xzr,x2
	prfm	pldl1keep, [x0]
	add	x0, x0, x2
	cmp	x0, x1
	cselhi	x0,x1,x0
	cselhi	x2,xzr,x2
	prfm	pldl1keep, [x0]
	add	x0, x0, x2
	cmp	x0, x1
	cselhi	x0,x1,x0
	cselhi	x2,xzr,x2
	prfm	pldl1keep, [x0]
	add	x0, x0, x2
	cmp	x0, x1
	cselhi	x0,x1,x0
	cselhi	x2,xzr,x2
	prfm	pldl1keep, [x0]
	add	x0, x0, x2
	cmp	x0, x1
	cselhi	x0,x1,x0
	prfm	pldl1keep, [x0]
	ret
	ENDP


	EXPORT	|vec_is_zero_16x|[FUNC]
	ALIGN	32
|vec_is_zero_16x| PROC
	ld1	{v0.2d}, [x0], #16
	lsr	x1, x1, #4
	sub	x1, x1, #1
	cbz	x1, |$Loop_is_zero_done|

|$Loop_is_zero|
	ld1	{v1.2d}, [x0], #16
	orr	v0.16b, v0.16b, v1.16b
	sub	x1, x1, #1
	cbnz	x1, |$Loop_is_zero|

|$Loop_is_zero_done|
	dup	v1.2d, v0.d[1]
	orr	v0.16b, v0.16b, v1.16b
	mov	x1, v0.d[0]
	mov	x0, #1
	cmp	x1, #0
	cseleq	x0,x0,xzr
	ret
	ENDP


	EXPORT	|vec_is_equal_16x|[FUNC]
	ALIGN	32
|vec_is_equal_16x| PROC
	ld1	{v0.2d}, [x0], #16
	ld1	{v1.2d}, [x1], #16
	lsr	x2, x2, #4
	eor	v0.16b, v0.16b, v1.16b

|$Loop_is_equal|
	sub	x2, x2, #1
	cbz	x2, |$Loop_is_equal_done|
	ld1	{v1.2d}, [x0], #16
	ld1	{v2.2d}, [x1], #16
	eor	v1.16b, v1.16b, v2.16b
	orr	v0.16b, v0.16b, v1.16b
	b	|$Loop_is_equal|
	nop

|$Loop_is_equal_done|
	dup	v1.2d, v0.d[1]
	orr	v0.16b, v0.16b, v1.16b
	mov	x1, v0.d[0]
	mov	x0, #1
	cmp	x1, #0
	cseleq	x0,x0,xzr
	ret
	ENDP
	END
