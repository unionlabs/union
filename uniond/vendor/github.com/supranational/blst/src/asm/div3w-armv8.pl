#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0

$flavour = shift;
$output  = shift;

if ($flavour && $flavour ne "void") {
    $0 =~ m/(.*[\/\\])[^\/\\]+$/; $dir=$1;
    ( $xlate="${dir}arm-xlate.pl" and -f $xlate ) or
    ( $xlate="${dir}../../perlasm/arm-xlate.pl" and -f $xlate) or
    die "can't locate arm-xlate.pl";

    open STDOUT,"| \"$^X\" $xlate $flavour $output";
} else {
    open STDOUT,">$output";
}

$code.=<<___;
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
___
{
my ($div_rem, $divisor, $quot) = map("x$_",(0..2));
my @div = map("x$_",(3..4));
my @acc = map("x$_",(5..7));
my @t = map("x$_",(8..11));

$code.=<<___;
.globl	quot_rem_128
.type	quot_rem_128,%function
.align	5
quot_rem_128:
	ldp	@div[0],@div[1],[$divisor]

	mul	@acc[0],@div[0],$quot	// divisor[0:1} * quotient
	umulh	@acc[1],@div[0],$quot
	mul	@t[3],  @div[1],$quot
	umulh	@acc[2],@div[1],$quot

	ldp	@t[0],@t[1],[$div_rem]	// load 3 limbs of the dividend
	ldr	@t[2],[$div_rem,#16]

	adds	@acc[1],@acc[1],@t[3]
	adc	@acc[2],@acc[2],xzr

	subs	@t[0],@t[0],@acc[0]	// dividend - divisor * quotient
	sbcs	@t[1],@t[1],@acc[1]
	sbcs	@t[2],@t[2],@acc[2]
	sbc	@acc[0],xzr,xzr		// borrow -> mask

	add	$quot,$quot,@acc[0]	// if borrowed, adjust the quotient ...
	and	@div[0],@div[0],@acc[0]
	and	@div[1],@div[1],@acc[0]
	adds	@t[0],@t[0],@div[0]	// ... and add divisor
	adc	@t[1],@t[1],@div[1]

	stp	@t[0],@t[1],[$div_rem]	// save 2 limbs of the remainder
	str	$quot,[$div_rem,#16]	// and one limb of the quotient

	mov	x0,$quot		// return adjusted quotient

	ret
.size	quot_rem_128,.-quot_rem_128

.globl	quot_rem_64
.type	quot_rem_64,%function
.align	5
quot_rem_64:
	ldr	@div[0],[$divisor]
	ldr	@t[0],[$div_rem]	// load 1 limb of the dividend

	mul	@acc[0],@div[0],$quot	// divisor * quotient

	sub	@t[0],@t[0],@acc[0]	// dividend - divisor * quotient

	stp	@t[0],$quot,[$div_rem]	// save remainder and quotient

	mov	x0,$quot		// return quotient

	ret
.size	quot_rem_64,.-quot_rem_64
___
}

print $code;
close STDOUT;
