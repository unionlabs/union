	AREA	|.text|,CODE,ALIGN=8,ARM64



	EXPORT	|add_mod_256|[FUNC]
	ALIGN	32
|add_mod_256| PROC
	ldp	x8,x9,[x1]
	ldp	x12,x13,[x2]

	ldp	x10,x11,[x1,#16]
	adds	x8,x8,x12
	ldp	x14,x15,[x2,#16]
	adcs	x9,x9,x13
	ldp	x4,x5,[x3]
	adcs	x10,x10,x14
	ldp	x6,x7,[x3,#16]
	adcs	x11,x11,x15
	adc	x3,xzr,xzr

	subs	x16,x8,x4
	sbcs	x17,x9,x5
	sbcs	x1,x10,x6
	sbcs	x2,x11,x7
	sbcs	xzr,x3,xzr

	csello	x8,x8,x16
	csello	x9,x9,x17
	csello	x10,x10,x1
	stp	x8,x9,[x0]
	csello	x11,x11,x2
	stp	x10,x11,[x0,#16]

	ret
	ENDP



	EXPORT	|mul_by_3_mod_256|[FUNC]
	ALIGN	32
|mul_by_3_mod_256| PROC
	ldp	x12,x13,[x1]
	ldp	x14,x15,[x1,#16]

	adds	x8,x12,x12
	ldp	x4,x5,[x2]
	adcs	x9,x13,x13
	ldp	x6,x7,[x2,#16]
	adcs	x10,x14,x14
	adcs	x11,x15,x15
	adc	x3,xzr,xzr

	subs	x16,x8,x4
	sbcs	x17,x9,x5
	sbcs	x1,x10,x6
	sbcs	x2,x11,x7
	sbcs	xzr,x3,xzr

	csello	x8,x8,x16
	csello	x9,x9,x17
	csello	x10,x10,x1
	csello	x11,x11,x2

	adds	x8,x8,x12
	adcs	x9,x9,x13
	adcs	x10,x10,x14
	adcs	x11,x11,x15
	adc	x3,xzr,xzr

	subs	x16,x8,x4
	sbcs	x17,x9,x5
	sbcs	x1,x10,x6
	sbcs	x2,x11,x7
	sbcs	xzr,x3,xzr

	csello	x8,x8,x16
	csello	x9,x9,x17
	csello	x10,x10,x1
	stp	x8,x9,[x0]
	csello	x11,x11,x2
	stp	x10,x11,[x0,#16]

	ret
	ENDP



	EXPORT	|lshift_mod_256|[FUNC]
	ALIGN	32
|lshift_mod_256| PROC
	ldp	x8,x9,[x1]
	ldp	x10,x11,[x1,#16]

	ldp	x4,x5,[x3]
	ldp	x6,x7,[x3,#16]

|$Loop_lshift_mod_256|
	adds	x8,x8,x8
	sub	x2,x2,#1
	adcs	x9,x9,x9
	adcs	x10,x10,x10
	adcs	x11,x11,x11
	adc	x3,xzr,xzr

	subs	x12,x8,x4
	sbcs	x13,x9,x5
	sbcs	x14,x10,x6
	sbcs	x15,x11,x7
	sbcs	xzr,x3,xzr

	csello	x8,x8,x12
	csello	x9,x9,x13
	csello	x10,x10,x14
	csello	x11,x11,x15

	cbnz	x2,|$Loop_lshift_mod_256|

	stp	x8,x9,[x0]
	stp	x10,x11,[x0,#16]

	ret
	ENDP



	EXPORT	|rshift_mod_256|[FUNC]
	ALIGN	32
|rshift_mod_256| PROC
	ldp	x8,x9,[x1]
	ldp	x10,x11,[x1,#16]

	ldp	x4,x5,[x3]
	ldp	x6,x7,[x3,#16]

|$Loop_rshift|
	adds	x12,x8,x4
	sub	x2,x2,#1
	adcs	x13,x9,x5
	adcs	x14,x10,x6
	adcs	x15,x11,x7
	adc	x3,xzr,xzr
	tst	x8,#1

	cselne	x12,x12,x8
	cselne	x13,x13,x9
	cselne	x14,x14,x10
	cselne	x15,x15,x11
	cselne	x3,x3,xzr

	extr	x8,x13,x12,#1
	extr	x9,x14,x13,#1
	extr	x10,x15,x14,#1
	extr	x11,x3,x15,#1

	cbnz	x2,|$Loop_rshift|

	stp	x8,x9,[x0]
	stp	x10,x11,[x0,#16]

	ret
	ENDP



	EXPORT	|cneg_mod_256|[FUNC]
	ALIGN	32
|cneg_mod_256| PROC
	ldp	x8,x9,[x1]
	ldp	x4,x5,[x3]

	ldp	x10,x11,[x1,#16]
	subs	x12,x4,x8
	ldp	x6,x7,[x3,#16]
	orr	x4,x8,x9
	sbcs	x13,x5,x9
	orr	x5,x10,x11
	sbcs	x14,x6,x10
	orr	x3,x4,x5
	sbc	x15,x7,x11

	cmp	x3,#0
	csetmne	x3
	ands	x2,x2,x3

	cseleq	x8,x8,x12
	cseleq	x9,x9,x13
	cseleq	x10,x10,x14
	stp	x8,x9,[x0]
	cseleq	x11,x11,x15
	stp	x10,x11,[x0,#16]

	ret
	ENDP



	EXPORT	|sub_mod_256|[FUNC]
	ALIGN	32
|sub_mod_256| PROC
	ldp	x8,x9,[x1]
	ldp	x12,x13,[x2]

	ldp	x10,x11,[x1,#16]
	subs	x8,x8,x12
	ldp	x14,x15,[x2,#16]
	sbcs	x9,x9,x13
	ldp	x4,x5,[x3]
	sbcs	x10,x10,x14
	ldp	x6,x7,[x3,#16]
	sbcs	x11,x11,x15
	sbc	x3,xzr,xzr

	and	x4,x4,x3
	and	x5,x5,x3
	adds	x8,x8,x4
	and	x6,x6,x3
	adcs	x9,x9,x5
	and	x7,x7,x3
	adcs	x10,x10,x6
	stp	x8,x9,[x0]
	adc	x11,x11,x7
	stp	x10,x11,[x0,#16]

	ret
	ENDP



	EXPORT	|check_mod_256|[FUNC]
	ALIGN	32
|check_mod_256| PROC
	ldp	x8,x9,[x0]
	ldp	x10,x11,[x0,#16]
	ldp	x4,x5,[x1]
	ldp	x6,x7,[x1,#16]

#ifdef	__AARCH64EB__
	rev	x8,x8
	rev	x9,x9
	rev	x10,x10
	rev	x11,x11
#endif

	subs	xzr,x8,x4
	sbcs	xzr,x9,x5
	orr	x8,x8,x9
	sbcs	xzr,x10,x6
	orr	x8,x8,x10
	sbcs	xzr,x11,x7
	orr	x8,x8,x11
	sbc	x1,xzr,xzr

	cmp	x8,#0
	mov	x0,#1
	cselne	x0,x0,xzr
	and	x0,x0,x1

	ret
	ENDP



	EXPORT	|add_n_check_mod_256|[FUNC]
	ALIGN	32
|add_n_check_mod_256| PROC
	ldp	x8,x9,[x1]
	ldp	x12,x13,[x2]
	ldp	x10,x11,[x1,#16]
	ldp	x14,x15,[x2,#16]

#ifdef	__AARCH64EB__
	rev	x8,x8
	rev	x12,x12
	rev	x9,x9
	rev	x13,x13
	rev	x10,x10
	rev	x14,x14
	rev	x11,x11
	rev	x15,x15
#endif

	adds	x8,x8,x12
	ldp	x4,x5,[x3]
	adcs	x9,x9,x13
	ldp	x6,x7,[x3,#16]
	adcs	x10,x10,x14
	adcs	x11,x11,x15
	adc	x3,xzr,xzr

	subs	x16,x8,x4
	sbcs	x17,x9,x5
	sbcs	x1,x10,x6
	sbcs	x2,x11,x7
	sbcs	xzr,x3,xzr

	csello	x8,x8,x16
	csello	x9,x9,x17
	csello	x10,x10,x1
	csello	x11,x11,x2

	orr	x16, x8, x9
	orr	x17, x10, x11
	orr	x16, x16, x17

#ifdef	__AARCH64EB__
	rev	x8,x8
	rev	x9,x9
	rev	x10,x10
	rev	x11,x11
#endif

	stp	x8,x9,[x0]
	stp	x10,x11,[x0,#16]

	mov	x17, #1
	cmp	x16, #0
	cselne	x0,x17,xzr

	ret
	ENDP



	EXPORT	|sub_n_check_mod_256|[FUNC]
	ALIGN	32
|sub_n_check_mod_256| PROC
	ldp	x8,x9,[x1]
	ldp	x12,x13,[x2]
	ldp	x10,x11,[x1,#16]
	ldp	x14,x15,[x2,#16]

#ifdef	__AARCH64EB__
	rev	x8,x8
	rev	x12,x12
	rev	x9,x9
	rev	x13,x13
	rev	x10,x10
	rev	x14,x14
	rev	x11,x11
	rev	x15,x15
#endif

	subs	x8,x8,x12
	sbcs	x9,x9,x13
	ldp	x4,x5,[x3]
	sbcs	x10,x10,x14
	ldp	x6,x7,[x3,#16]
	sbcs	x11,x11,x15
	sbc	x3,xzr,xzr

	and	x4,x4,x3
	and	x5,x5,x3
	adds	x8,x8,x4
	and	x6,x6,x3
	adcs	x9,x9,x5
	and	x7,x7,x3
	adcs	x10,x10,x6
	adc	x11,x11,x7

	orr	x16, x8, x9
	orr	x17, x10, x11
	orr	x16, x16, x17

#ifdef	__AARCH64EB__
	rev	x8,x8
	rev	x9,x9
	rev	x10,x10
	rev	x11,x11
#endif

	stp	x8,x9,[x0]
	stp	x10,x11,[x0,#16]

	mov	x17, #1
	cmp	x16, #0
	cselne	x0,x17,xzr

	ret
	ENDP
	END
