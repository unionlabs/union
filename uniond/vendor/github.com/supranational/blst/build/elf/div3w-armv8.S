.text

.globl	div_3_limbs
.type	div_3_limbs,%function
.align	5
div_3_limbs:
	ldp	x4,x5,[x0]	// load R
	eor	x0,x0,x0	// Q = 0
	mov	x3,#64		// loop counter
	nop

.Loop:
	subs	x6,x4,x1	// R - D
	add	x0,x0,x0	// Q <<= 1
	sbcs	x7,x5,x2
	add	x0,x0,#1	// Q + speculative bit
	csel	x4,x4,x6,lo	// select between R and R - D
	extr	x1,x2,x1,#1	// D >>= 1
	csel	x5,x5,x7,lo
	lsr	x2,x2,#1
	sbc	x0,x0,xzr	// subtract speculative bit
	sub	x3,x3,#1
	cbnz	x3,.Loop

	asr	x3,x0,#63	// top bit -> mask
	add	x0,x0,x0	// Q <<= 1
	subs	x6,x4,x1	// R - D
	add	x0,x0,#1	// Q + specilative bit
	sbcs	x7,x5,x2
	sbc	x0,x0,xzr	// subtract speculative bit

	orr	x0,x0,x3	// all ones if overflow

	ret
.size	div_3_limbs,.-div_3_limbs
.globl	quot_rem_128
.type	quot_rem_128,%function
.align	5
quot_rem_128:
	ldp	x3,x4,[x1]

	mul	x5,x3,x2	// divisor[0:1} * quotient
	umulh	x6,x3,x2
	mul	x11,  x4,x2
	umulh	x7,x4,x2

	ldp	x8,x9,[x0]	// load 3 limbs of the dividend
	ldr	x10,[x0,#16]

	adds	x6,x6,x11
	adc	x7,x7,xzr

	subs	x8,x8,x5	// dividend - divisor * quotient
	sbcs	x9,x9,x6
	sbcs	x10,x10,x7
	sbc	x5,xzr,xzr		// borrow -> mask

	add	x2,x2,x5	// if borrowed, adjust the quotient ...
	and	x3,x3,x5
	and	x4,x4,x5
	adds	x8,x8,x3	// ... and add divisor
	adc	x9,x9,x4

	stp	x8,x9,[x0]	// save 2 limbs of the remainder
	str	x2,[x0,#16]	// and one limb of the quotient

	mov	x0,x2		// return adjusted quotient

	ret
.size	quot_rem_128,.-quot_rem_128

.globl	quot_rem_64
.type	quot_rem_64,%function
.align	5
quot_rem_64:
	ldr	x3,[x1]
	ldr	x8,[x0]	// load 1 limb of the dividend

	mul	x5,x3,x2	// divisor * quotient

	sub	x8,x8,x5	// dividend - divisor * quotient

	stp	x8,x2,[x0]	// save remainder and quotient

	mov	x0,x2		// return quotient

	ret
.size	quot_rem_64,.-quot_rem_64
