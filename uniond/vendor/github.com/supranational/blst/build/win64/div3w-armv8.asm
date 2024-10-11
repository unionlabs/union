 GBLA __SIZEOF_POINTER__
__SIZEOF_POINTER__ SETA 64/8
	AREA	|.text|,CODE,ALIGN=8,ARM64



	EXPORT	|div_3_limbs|[FUNC]
	ALIGN	32
|div_3_limbs| PROC
	ldp	x4,x5,[x0]
	eor	x0,x0,x0
	mov	x3,#64
	nop

|$Loop|
	subs	x6,x4,x1
	add	x0,x0,x0
	sbcs	x7,x5,x2
	add	x0,x0,#1
	csello	x4,x4,x6
	extr	x1,x2,x1,#1
	csello	x5,x5,x7
	lsr	x2,x2,#1
	sbc	x0,x0,xzr
	sub	x3,x3,#1
	cbnz	x3,|$Loop|

	asr	x3,x0,#63
	add	x0,x0,x0
	subs	x6,x4,x1
	add	x0,x0,#1
	sbcs	x7,x5,x2
	sbc	x0,x0,xzr

	orr	x0,x0,x3

	ret
	ENDP


	EXPORT	|quot_rem_128|[FUNC]
	ALIGN	32
|quot_rem_128| PROC
	ldp	x3,x4,[x1]

	mul	x5,x3,x2
	umulh	x6,x3,x2
	mul	x11,  x4,x2
	umulh	x7,x4,x2

	ldp	x8,x9,[x0]
	ldr	x10,[x0,#16]

	adds	x6,x6,x11
	adc	x7,x7,xzr

	subs	x8,x8,x5
	sbcs	x9,x9,x6
	sbcs	x10,x10,x7
	sbc	x5,xzr,xzr

	add	x2,x2,x5
	and	x3,x3,x5
	and	x4,x4,x5
	adds	x8,x8,x3
	adc	x9,x9,x4

	stp	x8,x9,[x0]
	str	x2,[x0,#16]

	mov	x0,x2

	ret
	ENDP



	EXPORT	|quot_rem_64|[FUNC]
	ALIGN	32
|quot_rem_64| PROC
	ldr	x3,[x1]
	ldr	x8,[x0]

	mul	x5,x3,x2

	sub	x8,x8,x5

	stp	x8,x2,[x0]

	mov	x0,x2

	ret
	ENDP
	END
