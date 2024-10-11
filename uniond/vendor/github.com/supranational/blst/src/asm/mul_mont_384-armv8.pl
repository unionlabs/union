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

($r_ptr,$a_ptr,$b_ptr,$n_ptr,$n0) = map("x$_", 0..4);

@mod = map("x$_",(5..10));
@a   = map("x$_",(11..16));
$bi  = "x17";
@acc = map("x$_",(19..25));
@tmp = map("x$_",(26..28,0,1,3));

$code.=<<___;
.text

.globl	add_mod_384x384
.type	add_mod_384x384,%function
.align	5
add_mod_384x384:
	paciasp
	stp	c29,c30,[csp,#-8*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

	bl	__add_mod_384x384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldr	c29,[csp],#8*__SIZEOF_POINTER__
	autiasp
	ret
.size	add_mod_384x384,.-add_mod_384x384

.type	__add_mod_384x384,%function
.align	5
__add_mod_384x384:
	ldp	@a[0],  @a[1],  [$a_ptr]
	ldp	@acc[0],@acc[1],[$b_ptr]
	ldp	@a[2],  @a[3],  [$a_ptr,#16]
	adds	@a[0],@a[0],@acc[0]
	ldp	@acc[2],@acc[3],[$b_ptr,#16]
	adcs	@a[1],@a[1],@acc[1]
	ldp	@a[4],  @a[5],  [$a_ptr,#32]
	adcs	@a[2],@a[2],@acc[2]
	ldp	@acc[4],@acc[5],[$b_ptr,#32]
	adcs	@a[3],@a[3],@acc[3]
	 stp	@a[0],  @a[1],  [$r_ptr]
	adcs	@a[4],@a[4],@acc[4]
	 ldp	@a[0],  @a[1],  [$a_ptr,#48]
	adcs	@a[5],@a[5],@acc[5]

	 ldp	@acc[0],@acc[1],[$b_ptr,#48]
	 stp	@a[2],  @a[3],  [$r_ptr,#16]
	 ldp	@a[2],  @a[3],  [$a_ptr,#64]
	 ldp	@acc[2],@acc[3],[$b_ptr,#64]

	adcs	@a[0],@a[0],@acc[0]
	 stp	@a[4],  @a[5],  [$r_ptr,#32]
	adcs	@a[1],@a[1],@acc[1]
	 ldp	@a[4],  @a[5],  [$a_ptr,#80]
	adcs	@a[2],@a[2],@acc[2]
	 ldp	@acc[4],@acc[5],[$b_ptr,#80]
	adcs	@a[3],@a[3],@acc[3]
	adcs	@a[4],@a[4],@acc[4]
	adcs	@a[5],@a[5],@acc[5]
	adc	$bi,xzr,xzr

	subs	@acc[0],@a[0],@mod[0]
	sbcs	@acc[1],@a[1],@mod[1]
	sbcs	@acc[2],@a[2],@mod[2]
	sbcs	@acc[3],@a[3],@mod[3]
	sbcs	@acc[4],@a[4],@mod[4]
	sbcs	@acc[5],@a[5],@mod[5]
	sbcs	xzr,$bi,xzr

	csel	@a[0],@a[0],@acc[0],lo
	csel	@a[1],@a[1],@acc[1],lo
	csel	@a[2],@a[2],@acc[2],lo
	csel	@a[3],@a[3],@acc[3],lo
	stp	@a[0],@a[1],[$r_ptr,#48]
	csel	@a[4],@a[4],@acc[4],lo
	stp	@a[2],@a[3],[$r_ptr,#64]
	csel	@a[5],@a[5],@acc[5],lo
	stp	@a[4],@a[5],[$r_ptr,#80]

	ret
.size	__add_mod_384x384,.-__add_mod_384x384

.globl	sub_mod_384x384
.type	sub_mod_384x384,%function
.align	5
sub_mod_384x384:
	paciasp
	stp	c29,c30,[csp,#-8*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

	bl	__sub_mod_384x384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldr	c29,[csp],#8*__SIZEOF_POINTER__
	autiasp
	ret
.size	sub_mod_384x384,.-sub_mod_384x384

.type	__sub_mod_384x384,%function
.align	5
__sub_mod_384x384:
	ldp	@a[0],  @a[1],  [$a_ptr]
	ldp	@acc[0],@acc[1],[$b_ptr]
	ldp	@a[2],  @a[3],  [$a_ptr,#16]
	subs	@a[0],@a[0],@acc[0]
	ldp	@acc[2],@acc[3],[$b_ptr,#16]
	sbcs	@a[1],@a[1],@acc[1]
	ldp	@a[4],  @a[5],  [$a_ptr,#32]
	sbcs	@a[2],@a[2],@acc[2]
	ldp	@acc[4],@acc[5],[$b_ptr,#32]
	sbcs	@a[3],@a[3],@acc[3]
	 stp	@a[0],  @a[1],  [$r_ptr]
	sbcs	@a[4],@a[4],@acc[4]
	 ldp	@a[0],  @a[1],  [$a_ptr,#48]
	sbcs	@a[5],@a[5],@acc[5]

	 ldp	@acc[0],@acc[1],[$b_ptr,#48]
	 stp	@a[2],  @a[3],  [$r_ptr,#16]
	 ldp	@a[2],  @a[3],  [$a_ptr,#64]
	 ldp	@acc[2],@acc[3],[$b_ptr,#64]

	sbcs	@a[0],@a[0],@acc[0]
	 stp	@a[4],  @a[5],  [$r_ptr,#32]
	sbcs	@a[1],@a[1],@acc[1]
	 ldp	@a[4],  @a[5],  [$a_ptr,#80]
	sbcs	@a[2],@a[2],@acc[2]
	 ldp	@acc[4],@acc[5],[$b_ptr,#80]
	sbcs	@a[3],@a[3],@acc[3]
	sbcs	@a[4],@a[4],@acc[4]
	sbcs	@a[5],@a[5],@acc[5]
	sbc	$bi,xzr,xzr

	 and	@acc[0],@mod[0],$bi
	 and	@acc[1],@mod[1],$bi
	adds	@a[0],@a[0],@acc[0]
	 and	@acc[2],@mod[2],$bi
	adcs	@a[1],@a[1],@acc[1]
	 and	@acc[3],@mod[3],$bi
	adcs	@a[2],@a[2],@acc[2]
	 and	@acc[4],@mod[4],$bi
	adcs	@a[3],@a[3],@acc[3]
	 and	@acc[5],@mod[5],$bi
	adcs	@a[4],@a[4],@acc[4]
	stp	@a[0],@a[1],[$r_ptr,#48]
	adc	@a[5],@a[5],@acc[5]
	stp	@a[2],@a[3],[$r_ptr,#64]
	stp	@a[4],@a[5],[$r_ptr,#80]

	ret
.size	__sub_mod_384x384,.-__sub_mod_384x384

.type	__add_mod_384,%function
.align	5
__add_mod_384:
	ldp	@a[0],  @a[1],  [$a_ptr]
	ldp	@acc[0],@acc[1],[$b_ptr]
	ldp	@a[2],  @a[3],  [$a_ptr,#16]
	adds	@a[0],@a[0],@acc[0]
	ldp	@acc[2],@acc[3],[$b_ptr,#16]
	adcs	@a[1],@a[1],@acc[1]
	ldp	@a[4],  @a[5],  [$a_ptr,#32]
	adcs	@a[2],@a[2],@acc[2]
	ldp	@acc[4],@acc[5],[$b_ptr,#32]
	adcs	@a[3],@a[3],@acc[3]
	adcs	@a[4],@a[4],@acc[4]
	adcs	@a[5],@a[5],@acc[5]
	adc	$bi,xzr,xzr

	subs	@acc[0],@a[0],@mod[0]
	sbcs	@acc[1],@a[1],@mod[1]
	sbcs	@acc[2],@a[2],@mod[2]
	sbcs	@acc[3],@a[3],@mod[3]
	sbcs	@acc[4],@a[4],@mod[4]
	sbcs	@acc[5],@a[5],@mod[5]
	sbcs	xzr,$bi,xzr

	csel	@a[0],@a[0],@acc[0],lo
	csel	@a[1],@a[1],@acc[1],lo
	csel	@a[2],@a[2],@acc[2],lo
	csel	@a[3],@a[3],@acc[3],lo
	csel	@a[4],@a[4],@acc[4],lo
	stp	@a[0],@a[1],[$r_ptr]
	csel	@a[5],@a[5],@acc[5],lo
	stp	@a[2],@a[3],[$r_ptr,#16]
	stp	@a[4],@a[5],[$r_ptr,#32]

	ret
.size	__add_mod_384,.-__add_mod_384

.type	__sub_mod_384,%function
.align	5
__sub_mod_384:
	ldp	@a[0],  @a[1],  [$a_ptr]
	ldp	@acc[0],@acc[1],[$b_ptr]
	ldp	@a[2],  @a[3],  [$a_ptr,#16]
	subs	@a[0],@a[0],@acc[0]
	ldp	@acc[2],@acc[3],[$b_ptr,#16]
	sbcs	@a[1],@a[1],@acc[1]
	ldp	@a[4],  @a[5],  [$a_ptr,#32]
	sbcs	@a[2],@a[2],@acc[2]
	ldp	@acc[4],@acc[5],[$b_ptr,#32]
	sbcs	@a[3],@a[3],@acc[3]
	sbcs	@a[4],@a[4],@acc[4]
	sbcs	@a[5],@a[5],@acc[5]
	sbc	$bi,xzr,xzr

	 and	@acc[0],@mod[0],$bi
	 and	@acc[1],@mod[1],$bi
	adds	@a[0],@a[0],@acc[0]
	 and	@acc[2],@mod[2],$bi
	adcs	@a[1],@a[1],@acc[1]
	 and	@acc[3],@mod[3],$bi
	adcs	@a[2],@a[2],@acc[2]
	 and	@acc[4],@mod[4],$bi
	adcs	@a[3],@a[3],@acc[3]
	 and	@acc[5],@mod[5],$bi
	adcs	@a[4],@a[4],@acc[4]
	stp	@a[0],@a[1],[$r_ptr]
	adc	@a[5],@a[5],@acc[5]
	stp	@a[2],@a[3],[$r_ptr,#16]
	stp	@a[4],@a[5],[$r_ptr,#32]

	ret
.size	__sub_mod_384,.-__sub_mod_384

.globl	mul_mont_384x
.hidden	mul_mont_384x
.type	mul_mont_384x,%function
.align	5
mul_mont_384x:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	sub	csp,csp,#288		// space for 3 768-bit vectors

	cmov	@tmp[0],$r_ptr		// save r_ptr
	cmov	@tmp[1],$a_ptr		// save b_ptr
	cmov	@tmp[2],$b_ptr		// save b_ptr

	cadd	$r_ptr,sp,#0		// mul_384(t0, a->re, b->re)
	bl	__mul_384

	cadd	$a_ptr,$a_ptr,#48	// mul_384(t1, a->im, b->im)
	cadd	$b_ptr,$b_ptr,#48
	cadd	$r_ptr,sp,#96
	bl	__mul_384

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

	csub	$b_ptr,$a_ptr,#48
	cadd	$r_ptr,sp,#240
	bl	__add_mod_384

	cadd	$a_ptr,@tmp[2],#0
	cadd	$b_ptr,@tmp[2],#48
	cadd	$r_ptr,sp,#192		// t2
	bl	__add_mod_384

	cadd	$a_ptr,$r_ptr,#0
	cadd	$b_ptr,$r_ptr,#48
	bl	__mul_384		// mul_384(t2, a->re+a->im, b->re+b->im)

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

	cmov	$a_ptr,$r_ptr
	cadd	$b_ptr,sp,#0
	bl	__sub_mod_384x384

	cadd	$b_ptr,sp,#96
	bl	__sub_mod_384x384	// t2 = t2-t0-t1

	cadd	$a_ptr,sp,#0
	cadd	$b_ptr,sp,#96
	cadd	$r_ptr,sp,#0
	bl	__sub_mod_384x384	// t0 = t0-t1

	cadd	$a_ptr,sp,#0		// ret->re = redc(t0)
	cadd	$r_ptr,@tmp[0],#0
	bl	__mul_by_1_mont_384
	bl	__redc_tail_mont_384

	cadd	$a_ptr,sp,#192		// ret->im = redc(t2)
	cadd	$r_ptr,$r_ptr,#48
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
	autiasp
	ret
.size	mul_mont_384x,.-mul_mont_384x

.globl	sqr_mont_384x
.hidden	sqr_mont_384x
.type	sqr_mont_384x,%function
.align	5
sqr_mont_384x:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	stp	c3,c0,[csp,#12*__SIZEOF_POINTER__]	// __mul_mont_384 wants them there
	sub	csp,csp,#96		// space for 2 384-bit vectors
	mov	$n0,$n_ptr		// adjust for missing b_ptr

	ldp	@mod[0],@mod[1],[$b_ptr]
	ldp	@mod[2],@mod[3],[$b_ptr,#16]
	ldp	@mod[4],@mod[5],[$b_ptr,#32]

	cadd	$b_ptr,$a_ptr,#48
	cadd	$r_ptr,sp,#0
	bl	__add_mod_384		// t0 = a->re + a->im

	cadd	$r_ptr,sp,#48
	bl	__sub_mod_384		// t1 = a->re - a->im

	ldp	@a[0],@a[1],[$a_ptr]
	ldr	$bi,        [$b_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	bl	__mul_mont_384		// mul_mont_384(ret->im, a->re, a->im)

	adds	@a[0],@a[0],@a[0]	// add with itself
	adcs	@a[1],@a[1],@a[1]
	adcs	@a[2],@a[2],@a[2]
	adcs	@a[3],@a[3],@a[3]
	adcs	@a[4],@a[4],@a[4]
	adcs	@a[5],@a[5],@a[5]
	adc	@acc[6],xzr,xzr

	subs	@acc[0],@a[0],@mod[0]
	sbcs	@acc[1],@a[1],@mod[1]
	sbcs	@acc[2],@a[2],@mod[2]
	sbcs	@acc[3],@a[3],@mod[3]
	sbcs	@acc[4],@a[4],@mod[4]
	sbcs	@acc[5],@a[5],@mod[5]
	sbcs	xzr,@acc[6],xzr

	csel	@acc[0],@a[0],@acc[0],lo
	csel	@acc[1],@a[1],@acc[1],lo
	csel	@acc[2],@a[2],@acc[2],lo
	 ldp	@a[0],@a[1],[sp]
	csel	@acc[3],@a[3],@acc[3],lo
	 ldr	$bi,        [sp,#48]
	csel	@acc[4],@a[4],@acc[4],lo
	 ldp	@a[2],@a[3],[sp,#16]
	csel	@acc[5],@a[5],@acc[5],lo
	 ldp	@a[4],@a[5],[sp,#32]

	stp	@acc[0],@acc[1],[$b_ptr,#48]
	stp	@acc[2],@acc[3],[$b_ptr,#64]
	stp	@acc[4],@acc[5],[$b_ptr,#80]

	cadd	$b_ptr,sp,#48
	bl	__mul_mont_384		// mul_mont_384(ret->re, t0, t1)
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$b_ptr]
	stp	@a[2],@a[3],[$b_ptr,#16]
	stp	@a[4],@a[5],[$b_ptr,#32]

	add	csp,csp,#96
	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	sqr_mont_384x,.-sqr_mont_384x

.globl	mul_mont_384
.hidden	mul_mont_384
.type	mul_mont_384,%function
.align	5
mul_mont_384:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	stp	c4,c0,[csp,#12*__SIZEOF_POINTER__]	// __mul_mont_384 wants them there

	ldp	@a[0],@a[1],[$a_ptr]
	ldr	$bi,        [$b_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

	bl	__mul_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$b_ptr]
	stp	@a[2],@a[3],[$b_ptr,#16]
	stp	@a[4],@a[5],[$b_ptr,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	mul_mont_384,.-mul_mont_384

.type	__mul_mont_384,%function
.align	5
__mul_mont_384:
	mul	@acc[0],@a[0],$bi
	mul	@acc[1],@a[1],$bi
	mul	@acc[2],@a[2],$bi
	mul	@acc[3],@a[3],$bi
	mul	@acc[4],@a[4],$bi
	mul	@acc[5],@a[5],$bi
	mul	$n0,$n0,@acc[0]

	 umulh	@tmp[0],@a[0],$bi
	 umulh	@tmp[1],@a[1],$bi
	 umulh	@tmp[2],@a[2],$bi
	 umulh	@tmp[3],@a[3],$bi
	 umulh	@tmp[4],@a[4],$bi
	 umulh	@tmp[5],@a[5],$bi

	 adds	@acc[1],@acc[1],@tmp[0]
	// mul	@tmp[0],@mod[0],$n0
	 adcs	@acc[2],@acc[2],@tmp[1]
	mul	@tmp[1],@mod[1],$n0
	 adcs	@acc[3],@acc[3],@tmp[2]
	mul	@tmp[2],@mod[2],$n0
	 adcs	@acc[4],@acc[4],@tmp[3]
	mul	@tmp[3],@mod[3],$n0
	 adcs	@acc[5],@acc[5],@tmp[4]
	mul	@tmp[4],@mod[4],$n0
	 adc	@acc[6],xzr,    @tmp[5]
	mul	@tmp[5],@mod[5],$n0
	 mov	$bi,xzr
___
for ($i=1;$i<6;$i++) {
$code.=<<___;
	subs	xzr,@acc[0],#1		// adds	@acc[0],@acc[0],@tmp[0]
	 umulh	@tmp[0],@mod[0],$n0
	adcs	@acc[1],@acc[1],@tmp[1]
	 umulh	@tmp[1],@mod[1],$n0
	adcs	@acc[2],@acc[2],@tmp[2]
	 umulh	@tmp[2],@mod[2],$n0
	adcs	@acc[3],@acc[3],@tmp[3]
	 umulh	@tmp[3],@mod[3],$n0
	adcs	@acc[4],@acc[4],@tmp[4]
	 umulh	@tmp[4],@mod[4],$n0
	adcs	@acc[5],@acc[5],@tmp[5]
	 umulh	@tmp[5],@mod[5],$n0
	adcs	@acc[6],@acc[6],xzr
	adc	$n0,$bi,xzr
	ldr	$bi,[$b_ptr,8*$i]

	 adds	@acc[0],@acc[1],@tmp[0]
	mul	@tmp[0],@a[0],$bi
	 adcs	@acc[1],@acc[2],@tmp[1]
	mul	@tmp[1],@a[1],$bi
	 adcs	@acc[2],@acc[3],@tmp[2]
	mul	@tmp[2],@a[2],$bi
	 adcs	@acc[3],@acc[4],@tmp[3]
	mul	@tmp[3],@a[3],$bi
	 adcs	@acc[4],@acc[5],@tmp[4]
	mul	@tmp[4],@a[4],$bi
	 adcs	@acc[5],@acc[6],@tmp[5]
	mul	@tmp[5],@a[5],$bi
	 adc	@acc[6],$n0,xzr
	ldr	$n0,[x29,#12*__SIZEOF_POINTER__]

	adds	@acc[0],@acc[0],@tmp[0]
	 umulh	@tmp[0],@a[0],$bi
	adcs	@acc[1],@acc[1],@tmp[1]
	 umulh	@tmp[1],@a[1],$bi
	adcs	@acc[2],@acc[2],@tmp[2]
	mul	$n0,$n0,@acc[0]
	 umulh	@tmp[2],@a[2],$bi
	adcs	@acc[3],@acc[3],@tmp[3]
	 umulh	@tmp[3],@a[3],$bi
	adcs	@acc[4],@acc[4],@tmp[4]
	 umulh	@tmp[4],@a[4],$bi
	adcs	@acc[5],@acc[5],@tmp[5]
	 umulh	@tmp[5],@a[5],$bi
	adcs	@acc[6],@acc[6],xzr
	adc	$bi,xzr,xzr

	 adds	@acc[1],@acc[1],@tmp[0]
	// mul	@tmp[0],@mod[0],$n0
	 adcs	@acc[2],@acc[2],@tmp[1]
	mul	@tmp[1],@mod[1],$n0
	 adcs	@acc[3],@acc[3],@tmp[2]
	mul	@tmp[2],@mod[2],$n0
	 adcs	@acc[4],@acc[4],@tmp[3]
	mul	@tmp[3],@mod[3],$n0
	 adcs	@acc[5],@acc[5],@tmp[4]
	mul	@tmp[4],@mod[4],$n0
	 adcs	@acc[6],@acc[6],@tmp[5]
	mul	@tmp[5],@mod[5],$n0
	 adc	$bi,$bi,xzr
___
}
$code.=<<___;
	subs	xzr,@acc[0],#1		// adds	@acc[0],@acc[0],@tmp[0]
	 umulh	@tmp[0],@mod[0],$n0
	adcs	@acc[1],@acc[1],@tmp[1]
	 umulh	@tmp[1],@mod[1],$n0
	adcs	@acc[2],@acc[2],@tmp[2]
	 umulh	@tmp[2],@mod[2],$n0
	adcs	@acc[3],@acc[3],@tmp[3]
	 umulh	@tmp[3],@mod[3],$n0
	adcs	@acc[4],@acc[4],@tmp[4]
	 umulh	@tmp[4],@mod[4],$n0
	adcs	@acc[5],@acc[5],@tmp[5]
	 umulh	@tmp[5],@mod[5],$n0
	adcs	@acc[6],@acc[6],xzr
	 ldp	c4,c2,[c29,#12*__SIZEOF_POINTER__]	// pull r_ptr
	adc	$bi,$bi,xzr

	 adds	@acc[0],@acc[1],@tmp[0]
	 adcs	@acc[1],@acc[2],@tmp[1]
	 adcs	@acc[2],@acc[3],@tmp[2]
	 adcs	@acc[3],@acc[4],@tmp[3]
	 adcs	@acc[4],@acc[5],@tmp[4]
	 adcs	@acc[5],@acc[6],@tmp[5]
	 adc	@acc[6],$bi,xzr

	subs	@tmp[0],@acc[0],@mod[0]
	sbcs	@tmp[1],@acc[1],@mod[1]
	sbcs	@tmp[2],@acc[2],@mod[2]
	sbcs	@tmp[3],@acc[3],@mod[3]
	sbcs	@tmp[4],@acc[4],@mod[4]
	sbcs	@tmp[5],@acc[5],@mod[5]
	sbcs	xzr,    @acc[6],xzr

	csel	@a[0],@acc[0],@tmp[0],lo
	csel	@a[1],@acc[1],@tmp[1],lo
	csel	@a[2],@acc[2],@tmp[2],lo
	csel	@a[3],@acc[3],@tmp[3],lo
	csel	@a[4],@acc[4],@tmp[4],lo
	csel	@a[5],@acc[5],@tmp[5],lo
	ret
.size	__mul_mont_384,.-__mul_mont_384

.globl	sqr_mont_384
.hidden	sqr_mont_384
.type	sqr_mont_384,%function
.align	5
sqr_mont_384:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	sub	csp,csp,#96		// space for 768-bit vector
	cmov	$n0,$n_ptr		// adjust for missing b_ptr

	cmov	$n_ptr,$r_ptr		// save r_ptr
	cmov	$r_ptr,sp

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	bl	__sqr_384

	ldp	@mod[0],@mod[1],[$b_ptr]
	ldp	@mod[2],@mod[3],[$b_ptr,#16]
	ldp	@mod[4],@mod[5],[$b_ptr,#32]

	cmov	$a_ptr,sp
	cmov	$r_ptr,$n_ptr		// restore r_ptr
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
	autiasp
	ret
.size	sqr_mont_384,.-sqr_mont_384

.globl	sqr_n_mul_mont_383
.hidden	sqr_n_mul_mont_383
.type	sqr_n_mul_mont_383,%function
.align	5
sqr_n_mul_mont_383:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	stp	c4,c0,[csp,#12*__SIZEOF_POINTER__]	// __mul_mont_384 wants them there
	sub	csp,csp,#96		// space for 768-bit vector
	cmov	$bi,x5			// save b_ptr

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]
	cmov	$r_ptr,sp
.Loop_sqr_383:
	bl	__sqr_384
	sub	$b_ptr,$b_ptr,#1	// counter

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

	cmov	$a_ptr,sp
	bl	__mul_by_1_mont_384

	ldp	@acc[0],@acc[1],[$a_ptr,#48]
	ldp	@acc[2],@acc[3],[$a_ptr,#64]
	ldp	@acc[4],@acc[5],[$a_ptr,#80]

	adds	@a[0],@a[0],@acc[0]	// just accumulate upper half
	adcs	@a[1],@a[1],@acc[1]
	adcs	@a[2],@a[2],@acc[2]
	adcs	@a[3],@a[3],@acc[3]
	adcs	@a[4],@a[4],@acc[4]
	adc	@a[5],@a[5],@acc[5]

	cbnz	$b_ptr,.Loop_sqr_383

	cmov	$b_ptr,$bi
	ldr	$bi,[$bi]
	bl	__mul_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$b_ptr]
	stp	@a[2],@a[3],[$b_ptr,#16]
	stp	@a[4],@a[5],[$b_ptr,#32]

	add	csp,csp,#96
	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	sqr_n_mul_mont_383,.-sqr_n_mul_mont_383
___
{
my @acc=(@acc,@tmp[0..2]);

$code.=<<___;
.type	__sqr_384,%function
.align	5
__sqr_384:
	mul	@acc[0],@a[1],@a[0]
	mul	@acc[1],@a[2],@a[0]
	mul	@acc[2],@a[3],@a[0]
	mul	@acc[3],@a[4],@a[0]
	mul	@acc[4],@a[5],@a[0]

	 umulh	@mod[1],@a[1],@a[0]
	 umulh	@mod[2],@a[2],@a[0]
	 umulh	@mod[3],@a[3],@a[0]
	 umulh	@mod[4],@a[4],@a[0]
	 adds	@acc[1],@acc[1],@mod[1]
	 umulh	@mod[5],@a[5],@a[0]
	 adcs	@acc[2],@acc[2],@mod[2]
	mul	@mod[2],@a[2],@a[1]
	 adcs	@acc[3],@acc[3],@mod[3]
	mul	@mod[3],@a[3],@a[1]
	 adcs	@acc[4],@acc[4],@mod[4]
	mul	@mod[4],@a[4],@a[1]
	 adc	@acc[5],xzr,    @mod[5]
	mul	@mod[5],@a[5],@a[1]

	adds	@acc[2],@acc[2],@mod[2]
	 umulh	@mod[2],@a[2],@a[1]
	adcs	@acc[3],@acc[3],@mod[3]
	 umulh	@mod[3],@a[3],@a[1]
	adcs	@acc[4],@acc[4],@mod[4]
	 umulh	@mod[4],@a[4],@a[1]
	adcs	@acc[5],@acc[5],@mod[5]
	 umulh	@mod[5],@a[5],@a[1]
	adc	@acc[6],xzr,xzr

	  mul	@mod[0],@a[0],@a[0]
	 adds	@acc[3],@acc[3],@mod[2]
	  umulh	@a[0],  @a[0],@a[0]
	 adcs	@acc[4],@acc[4],@mod[3]
	mul	@mod[3],@a[3],@a[2]
	 adcs	@acc[5],@acc[5],@mod[4]
	mul	@mod[4],@a[4],@a[2]
	 adc	@acc[6],@acc[6],@mod[5]
	mul	@mod[5],@a[5],@a[2]

	adds	@acc[4],@acc[4],@mod[3]
	 umulh	@mod[3],@a[3],@a[2]
	adcs	@acc[5],@acc[5],@mod[4]
	 umulh	@mod[4],@a[4],@a[2]
	adcs	@acc[6],@acc[6],@mod[5]
	 umulh	@mod[5],@a[5],@a[2]
	adc	@acc[7],xzr,xzr

	  mul	@mod[1],@a[1],@a[1]
	 adds	@acc[5],@acc[5],@mod[3]
	  umulh	@a[1],  @a[1],@a[1]
	 adcs	@acc[6],@acc[6],@mod[4]
	mul	@mod[4],@a[4],@a[3]
	 adc	@acc[7],@acc[7],@mod[5]
	mul	@mod[5],@a[5],@a[3]

	adds	@acc[6],@acc[6],@mod[4]
	 umulh	@mod[4],@a[4],@a[3]
	adcs	@acc[7],@acc[7],@mod[5]
	 umulh	@mod[5],@a[5],@a[3]
	adc	@acc[8],xzr,xzr
	  mul	@mod[2],@a[2],@a[2]
	 adds	@acc[7],@acc[7],@mod[4]
	  umulh	@a[2],  @a[2],@a[2]
	 adc	@acc[8],@acc[8],@mod[5]
	  mul	@mod[3],@a[3],@a[3]

	mul	@mod[5],@a[5],@a[4]
	  umulh	@a[3],  @a[3],@a[3]
	adds	@acc[8],@acc[8],@mod[5]
	 umulh	@mod[5],@a[5],@a[4]
	  mul	@mod[4],@a[4],@a[4]
	adc	@acc[9],@mod[5],xzr

	adds	@acc[0],@acc[0],@acc[0]
	adcs	@acc[1],@acc[1],@acc[1]
	adcs	@acc[2],@acc[2],@acc[2]
	adcs	@acc[3],@acc[3],@acc[3]
	adcs	@acc[4],@acc[4],@acc[4]
	adcs	@acc[5],@acc[5],@acc[5]
	adcs	@acc[6],@acc[6],@acc[6]
	adcs	@acc[7],@acc[7],@acc[7]
	  umulh	@a[4],  @a[4],@a[4]
	adcs	@acc[8],@acc[8],@acc[8]
	  mul	@mod[5],@a[5],@a[5]
	adcs	@acc[9],@acc[9],@acc[9]
	  umulh	@a[5],  @a[5],@a[5]
	adc	$a_ptr,xzr,xzr

	adds	@acc[0],@acc[0],@a[0]
	adcs	@acc[1],@acc[1],@mod[1]
	adcs	@acc[2],@acc[2],@a[1]
	adcs	@acc[3],@acc[3],@mod[2]
	adcs	@acc[4],@acc[4],@a[2]
	adcs	@acc[5],@acc[5],@mod[3]
	adcs	@acc[6],@acc[6],@a[3]
	stp	@mod[0],@acc[0],[$r_ptr]
	adcs	@acc[7],@acc[7],@mod[4]
	stp	@acc[1],@acc[2],[$r_ptr,#16]
	adcs	@acc[8],@acc[8],@a[4]
	stp	@acc[3],@acc[4],[$r_ptr,#32]
	adcs	@acc[9],@acc[9],@mod[5]
	stp	@acc[5],@acc[6],[$r_ptr,#48]
	adc	@a[5],@a[5],$a_ptr
	stp	@acc[7],@acc[8],[$r_ptr,#64]
	stp	@acc[9],@a[5],[$r_ptr,#80]

	ret
.size	__sqr_384,.-__sqr_384
___
}
$code.=<<___;
.globl	sqr_384
.hidden	sqr_384
.type	sqr_384,%function
.align	5
sqr_384:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	bl	__sqr_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	sqr_384,.-sqr_384

.globl	redc_mont_384
.hidden	redc_mont_384
.type	redc_mont_384,%function
.align	5
redc_mont_384:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	mov	$n0,$n_ptr		// adjust for missing b_ptr

	ldp	@mod[0],@mod[1],[$b_ptr]
	ldp	@mod[2],@mod[3],[$b_ptr,#16]
	ldp	@mod[4],@mod[5],[$b_ptr,#32]

	bl	__mul_by_1_mont_384
	bl	__redc_tail_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	redc_mont_384,.-redc_mont_384

.globl	from_mont_384
.hidden	from_mont_384
.type	from_mont_384,%function
.align	5
from_mont_384:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	mov	$n0,$n_ptr		// adjust for missing b_ptr

	ldp	@mod[0],@mod[1],[$b_ptr]
	ldp	@mod[2],@mod[3],[$b_ptr,#16]
	ldp	@mod[4],@mod[5],[$b_ptr,#32]

	bl	__mul_by_1_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	subs	@acc[0],@a[0],@mod[0]
	sbcs	@acc[1],@a[1],@mod[1]
	sbcs	@acc[2],@a[2],@mod[2]
	sbcs	@acc[3],@a[3],@mod[3]
	sbcs	@acc[4],@a[4],@mod[4]
	sbcs	@acc[5],@a[5],@mod[5]

	csel	@a[0],@a[0],@acc[0],lo
	csel	@a[1],@a[1],@acc[1],lo
	csel	@a[2],@a[2],@acc[2],lo
	csel	@a[3],@a[3],@acc[3],lo
	csel	@a[4],@a[4],@acc[4],lo
	csel	@a[5],@a[5],@acc[5],lo

	stp	@a[0],@a[1],[$r_ptr]
	stp	@a[2],@a[3],[$r_ptr,#16]
	stp	@a[4],@a[5],[$r_ptr,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	from_mont_384,.-from_mont_384

.type	__mul_by_1_mont_384,%function
.align	5
__mul_by_1_mont_384:
	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	mul	@tmp[0],$n0,@a[0]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	// mul	@acc[0],@mod[0],@tmp[0]
	mul	@acc[1],@mod[1],@tmp[0]
	mul	@acc[2],@mod[2],@tmp[0]
	mul	@acc[3],@mod[3],@tmp[0]
	mul	@acc[4],@mod[4],@tmp[0]
	mul	@acc[5],@mod[5],@tmp[0]
	subs	xzr,@a[0],#1		// adds	@acc[0],@acc[0],@a[0]
	 umulh	@a[0],@mod[0],@tmp[0]
	adcs	@acc[1],@acc[1],@a[1]
	 umulh	@a[1],@mod[1],@tmp[0]
	adcs	@acc[2],@acc[2],@a[2]
	 umulh	@a[2],@mod[2],@tmp[0]
	adcs	@acc[3],@acc[3],@a[3]
	 umulh	@a[3],@mod[3],@tmp[0]
	adcs	@acc[4],@acc[4],@a[4]
	 umulh	@a[4],@mod[4],@tmp[0]
	adcs	@acc[5],@acc[5],@a[5]
	 umulh	@a[5],@mod[5],@tmp[0]
	adc	@acc[6],xzr,xzr
___
for ($i=1;$i<6;$i++) {
$code.=<<___;
	 adds	@a[0],@a[0],@acc[1]
	 adcs	@a[1],@a[1],@acc[2]
	 adcs	@a[2],@a[2],@acc[3]
	mul	@tmp[0],$n0,@a[0]
	 adcs	@a[3],@a[3],@acc[4]
	 adcs	@a[4],@a[4],@acc[5]
	 adc	@a[5],@a[5],@acc[6]

	// mul	@acc[0],@mod[0],@tmp[0]
	mul	@acc[1],@mod[1],@tmp[0]
	mul	@acc[2],@mod[2],@tmp[0]
	mul	@acc[3],@mod[3],@tmp[0]
	mul	@acc[4],@mod[4],@tmp[0]
	mul	@acc[5],@mod[5],@tmp[0]
	subs	xzr,@a[0],#1		// adds	@acc[0],@acc[0],@a[0]
	 umulh	@a[0],@mod[0],@tmp[0]
	adcs	@acc[1],@acc[1],@a[1]
	 umulh	@a[1],@mod[1],@tmp[0]
	adcs	@acc[2],@acc[2],@a[2]
	 umulh	@a[2],@mod[2],@tmp[0]
	adcs	@acc[3],@acc[3],@a[3]
	 umulh	@a[3],@mod[3],@tmp[0]
	adcs	@acc[4],@acc[4],@a[4]
	 umulh	@a[4],@mod[4],@tmp[0]
	adcs	@acc[5],@acc[5],@a[5]
	 umulh	@a[5],@mod[5],@tmp[0]
	adc	@acc[6],xzr,xzr
___
}
$code.=<<___;
	adds	@a[0],@a[0],@acc[1]
	adcs	@a[1],@a[1],@acc[2]
	adcs	@a[2],@a[2],@acc[3]
	adcs	@a[3],@a[3],@acc[4]
	adcs	@a[4],@a[4],@acc[5]
	adc	@a[5],@a[5],@acc[6]

	ret
.size	__mul_by_1_mont_384,.-__mul_by_1_mont_384

.type	__redc_tail_mont_384,%function
.align	5
__redc_tail_mont_384:
	ldp	@acc[0],@acc[1],[$a_ptr,#48]
	ldp	@acc[2],@acc[3],[$a_ptr,#64]
	ldp	@acc[4],@acc[5],[$a_ptr,#80]

	adds	@a[0],@a[0],@acc[0]	// accumulate upper half
	adcs	@a[1],@a[1],@acc[1]
	adcs	@a[2],@a[2],@acc[2]
	adcs	@a[3],@a[3],@acc[3]
	adcs	@a[4],@a[4],@acc[4]
	adcs	@a[5],@a[5],@acc[5]
	adc	@acc[6],xzr,xzr

	subs	@acc[0],@a[0],@mod[0]
	sbcs	@acc[1],@a[1],@mod[1]
	sbcs	@acc[2],@a[2],@mod[2]
	sbcs	@acc[3],@a[3],@mod[3]
	sbcs	@acc[4],@a[4],@mod[4]
	sbcs	@acc[5],@a[5],@mod[5]
	sbcs	xzr,@acc[6],xzr

	csel	@a[0],@a[0],@acc[0],lo
	csel	@a[1],@a[1],@acc[1],lo
	csel	@a[2],@a[2],@acc[2],lo
	csel	@a[3],@a[3],@acc[3],lo
	csel	@a[4],@a[4],@acc[4],lo
	csel	@a[5],@a[5],@acc[5],lo

	stp	@a[0],@a[1],[$r_ptr]
	stp	@a[2],@a[3],[$r_ptr,#16]
	stp	@a[4],@a[5],[$r_ptr,#32]

	ret
.size	__redc_tail_mont_384,.-__redc_tail_mont_384

.globl	mul_384
.hidden	mul_384
.type	mul_384,%function
.align	5
mul_384:
	paciasp
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
	autiasp
	ret
.size	mul_384,.-mul_384

.type	__mul_384,%function
.align	5
__mul_384:
	ldp	@a[0],@a[1],[$a_ptr]
	ldr	$bi,        [$b_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	mul	@acc[0],@a[0],$bi
	mul	@acc[1],@a[1],$bi
	mul	@acc[2],@a[2],$bi
	mul	@acc[3],@a[3],$bi
	mul	@acc[4],@a[4],$bi
	mul	@acc[5],@a[5],$bi

	 umulh	@mod[0],@a[0],$bi
	 umulh	@mod[1],@a[1],$bi
	 umulh	@mod[2],@a[2],$bi
	 umulh	@mod[3],@a[3],$bi
	 umulh	@mod[4],@a[4],$bi
	 umulh	@mod[5],@a[5],$bi
	ldr	$bi,[$b_ptr,8*1]

	str	@acc[0],[$r_ptr]
	 adds	@acc[0],@acc[1],@mod[0]
	mul	@mod[0],@a[0],$bi
	 adcs	@acc[1],@acc[2],@mod[1]
	mul	@mod[1],@a[1],$bi
	 adcs	@acc[2],@acc[3],@mod[2]
	mul	@mod[2],@a[2],$bi
	 adcs	@acc[3],@acc[4],@mod[3]
	mul	@mod[3],@a[3],$bi
	 adcs	@acc[4],@acc[5],@mod[4]
	mul	@mod[4],@a[4],$bi
	 adc	@acc[5],xzr,    @mod[5]
	mul	@mod[5],@a[5],$bi
___
for ($i=1;$i<5;$i++) {
$code.=<<___;
	adds	@acc[0],@acc[0],@mod[0]
	 umulh	@mod[0],@a[0],$bi
	adcs	@acc[1],@acc[1],@mod[1]
	 umulh	@mod[1],@a[1],$bi
	adcs	@acc[2],@acc[2],@mod[2]
	 umulh	@mod[2],@a[2],$bi
	adcs	@acc[3],@acc[3],@mod[3]
	 umulh	@mod[3],@a[3],$bi
	adcs	@acc[4],@acc[4],@mod[4]
	 umulh	@mod[4],@a[4],$bi
	adcs	@acc[5],@acc[5],@mod[5]
	 umulh	@mod[5],@a[5],$bi
	ldr	$bi,[$b_ptr,#8*($i+1)]
	adc	@acc[6],xzr,xzr

	str	@acc[0],[$r_ptr,8*$i]
	 adds	@acc[0],@acc[1],@mod[0]
	mul	@mod[0],@a[0],$bi
	 adcs	@acc[1],@acc[2],@mod[1]
	mul	@mod[1],@a[1],$bi
	 adcs	@acc[2],@acc[3],@mod[2]
	mul	@mod[2],@a[2],$bi
	 adcs	@acc[3],@acc[4],@mod[3]
	mul	@mod[3],@a[3],$bi
	 adcs	@acc[4],@acc[5],@mod[4]
	mul	@mod[4],@a[4],$bi
	 adc	@acc[5],@acc[6],@mod[5]
	mul	@mod[5],@a[5],$bi
___
}
$code.=<<___;
	adds	@acc[0],@acc[0],@mod[0]
	 umulh	@mod[0],@a[0],$bi
	adcs	@acc[1],@acc[1],@mod[1]
	 umulh	@mod[1],@a[1],$bi
	adcs	@acc[2],@acc[2],@mod[2]
	 umulh	@mod[2],@a[2],$bi
	adcs	@acc[3],@acc[3],@mod[3]
	 umulh	@mod[3],@a[3],$bi
	adcs	@acc[4],@acc[4],@mod[4]
	 umulh	@mod[4],@a[4],$bi
	adcs	@acc[5],@acc[5],@mod[5]
	 umulh	@mod[5],@a[5],$bi
	adc	@acc[6],xzr,xzr

	str	@acc[0],[$r_ptr,8*$i]
	 adds	@acc[0],@acc[1],@mod[0]
	 adcs	@acc[1],@acc[2],@mod[1]
	 adcs	@acc[2],@acc[3],@mod[2]
	 adcs	@acc[3],@acc[4],@mod[3]
	 adcs	@acc[4],@acc[5],@mod[4]
	 adc	@acc[5],@acc[6],@mod[5]

	stp	@acc[0],@acc[1],[$r_ptr,#48]
	stp	@acc[2],@acc[3],[$r_ptr,#64]
	stp	@acc[4],@acc[5],[$r_ptr,#80]

	ret
.size	__mul_384,.-__mul_384

.globl	mul_382x
.hidden	mul_382x
.type	mul_382x,%function
.align	5
mul_382x:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	sub	csp,csp,#96		// space for two 384-bit vectors

	ldp	@a[0],@a[1],[$a_ptr]
	cmov	@tmp[0],$r_ptr		// save r_ptr
	ldp	@acc[0],@acc[1],[$a_ptr,#48]
	cmov	@tmp[1],$a_ptr		// save a_ptr
	ldp	@a[2],@a[3],[$a_ptr,#16]
	cmov	@tmp[2],$b_ptr		// save b_ptr
	ldp	@acc[2],@acc[3],[$a_ptr,#64]
	ldp	@a[4],@a[5],[$a_ptr,#32]
	adds	@mod[0],$a[0],@acc[0]	// t0 = a->re + a->im
	ldp	@acc[4],@acc[5],[$a_ptr,#80]
	adcs	@mod[1],$a[1],@acc[1]
	 ldp	@a[0],@a[1],[$b_ptr]
	adcs	@mod[2],$a[2],@acc[2]
	 ldp	@acc[0],@acc[1],[$b_ptr,#48]
	adcs	@mod[3],$a[3],@acc[3]
	 ldp	@a[2],@a[3],[$b_ptr,#16]
	adcs	@mod[4],$a[4],@acc[4]
	 ldp	@acc[2],@acc[3],[$b_ptr,#64]
	adc	@mod[5],$a[5],@acc[5]
	 ldp	@a[4],@a[5],[$b_ptr,#32]

	stp	@mod[0],@mod[1],[sp]
	 adds	@mod[0],$a[0],@acc[0]	// t1 = b->re + b->im
	 ldp	@acc[4],@acc[5],[$b_ptr,#80]
	 adcs	@mod[1],$a[1],@acc[1]
	stp	@mod[2],@mod[3],[sp,#16]
	 adcs	@mod[2],$a[2],@acc[2]
	 adcs	@mod[3],$a[3],@acc[3]
	 stp	@mod[4],@mod[5],[sp,#32]
	 adcs	@mod[4],$a[4],@acc[4]
	 stp	@mod[0],@mod[1],[sp,#48]
	 adc	@mod[5],$a[5],@acc[5]
	 stp	@mod[2],@mod[3],[sp,#64]
	 stp	@mod[4],@mod[5],[sp,#80]

	bl	__mul_384		// mul_384(ret->re, a->re, b->re)

	cadd	$a_ptr,sp,#0		// mul_384(ret->im, t0, t1)
	cadd	$b_ptr,sp,#48
	cadd	$r_ptr,@tmp[0],#96
	bl	__mul_384

	cadd	$a_ptr,@tmp[1],#48	// mul_384(tx, a->im, b->im)
	cadd	$b_ptr,@tmp[2],#48
	cadd	$r_ptr,sp,#0
	bl	__mul_384

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

	cadd	$a_ptr,@tmp[0],#96	// ret->im -= tx
	cadd	$b_ptr,sp,#0
	cadd	$r_ptr,@tmp[0],#96
	bl	__sub_mod_384x384

	cadd	$b_ptr,@tmp[0],#0	// ret->im -= ret->re
	bl	__sub_mod_384x384

	cadd	$a_ptr,@tmp[0],#0	// ret->re -= tx
	cadd	$b_ptr,sp,#0
	cadd	$r_ptr,@tmp[0],#0
	bl	__sub_mod_384x384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	add	csp,csp,#96
	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	mul_382x,.-mul_382x

.globl	sqr_382x
.hidden	sqr_382x
.type	sqr_382x,%function
.align	5
sqr_382x:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@acc[0],@acc[1],[$a_ptr,#48]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	adds	@mod[0],$a[0],@acc[0]	// t0 = a->re + a->im
	ldp	@acc[2],@acc[3],[$a_ptr,#64]
	adcs	@mod[1],$a[1],@acc[1]
	ldp	@a[4],@a[5],[$a_ptr,#32]
	adcs	@mod[2],$a[2],@acc[2]
	ldp	@acc[4],@acc[5],[$a_ptr,#80]
	adcs	@mod[3],$a[3],@acc[3]
	stp	@mod[0],@mod[1],[$r_ptr]
	adcs	@mod[4],$a[4],@acc[4]
	 ldp	@mod[0],@mod[1],[$b_ptr]
	adc	@mod[5],$a[5],@acc[5]
	stp	@mod[2],@mod[3],[$r_ptr,#16]

	subs	@a[0],$a[0],@acc[0]	// t1 = a->re - a->im
	 ldp	@mod[2],@mod[3],[$b_ptr,#16]
	sbcs	@a[1],$a[1],@acc[1]
	stp	@mod[4],@mod[5],[$r_ptr,#32]
	sbcs	@a[2],$a[2],@acc[2]
	 ldp	@mod[4],@mod[5],[$b_ptr,#32]
	sbcs	@a[3],$a[3],@acc[3]
	sbcs	@a[4],$a[4],@acc[4]
	sbcs	@a[5],$a[5],@acc[5]
	sbc	@acc[6],xzr,xzr

	 and	@acc[0],@mod[0],@acc[6]
	 and	@acc[1],@mod[1],@acc[6]
	adds	@a[0],@a[0],@acc[0]
	 and	@acc[2],@mod[2],@acc[6]
	adcs	@a[1],@a[1],@acc[1]
	 and	@acc[3],@mod[3],@acc[6]
	adcs	@a[2],@a[2],@acc[2]
	 and	@acc[4],@mod[4],@acc[6]
	adcs	@a[3],@a[3],@acc[3]
	 and	@acc[5],@mod[5],@acc[6]
	adcs	@a[4],@a[4],@acc[4]
	stp	@a[0],@a[1],[$r_ptr,#48]
	adc	@a[5],@a[5],@acc[5]
	stp	@a[2],@a[3],[$r_ptr,#64]
	stp	@a[4],@a[5],[$r_ptr,#80]

	cmov	$n0,$a_ptr		// save a_ptr
	cadd	$a_ptr,$r_ptr,#0	// mul_384(ret->re, t0, t1)
	cadd	$b_ptr,$r_ptr,#48
	bl	__mul_384

	cadd	$a_ptr,$n0,#0		// mul_384(ret->im, a->re, a->im)
	cadd	$b_ptr,$n0,#48
	cadd	$r_ptr,$r_ptr,#96
	bl	__mul_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldp	@a[0],@a[1],[$r_ptr]
	ldp	@a[2],@a[3],[$r_ptr,#16]
	adds	@a[0],@a[0],@a[0]	// add with itself
	ldp	@a[4],@a[5],[$r_ptr,#32]
	adcs	@a[1],@a[1],@a[1]
	adcs	@a[2],@a[2],@a[2]
	adcs	@a[3],@a[3],@a[3]
	adcs	@a[4],@a[4],@a[4]
	adcs	@a[5],@a[5],@a[5]
	adcs	@acc[0],@acc[0],@acc[0]
	adcs	@acc[1],@acc[1],@acc[1]
	stp	@a[0],@a[1],[$r_ptr]
	adcs	@acc[2],@acc[2],@acc[2]
	stp	@a[2],@a[3],[$r_ptr,#16]
	adcs	@acc[3],@acc[3],@acc[3]
	stp	@a[4],@a[5],[$r_ptr,#32]
	adcs	@acc[4],@acc[4],@acc[4]
	stp	@acc[0],@acc[1],[$r_ptr,#48]
	adc	@acc[5],@acc[5],@acc[5]
	stp	@acc[2],@acc[3],[$r_ptr,#64]
	stp	@acc[4],@acc[5],[$r_ptr,#80]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	sqr_382x,.-sqr_382x

.globl	sqr_mont_382x
.hidden	sqr_mont_382x
.type	sqr_mont_382x,%function
.align	5
sqr_mont_382x:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]
	stp	c3,c0,[csp,#12*__SIZEOF_POINTER__]	// __mul_mont_384 wants them there
	sub	csp,csp,#112		// space for two 384-bit vectors + word
	mov	$n0,$n_ptr		// adjust for missing b_ptr

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	ldp	$bi,@acc[1],[$a_ptr,#48]
	ldp	@acc[2],@acc[3],[$a_ptr,#64]
	ldp	@acc[4],@acc[5],[$a_ptr,#80]

	adds	@mod[0],$a[0],$bi	// t0 = a->re + a->im
	adcs	@mod[1],$a[1],@acc[1]
	adcs	@mod[2],$a[2],@acc[2]
	adcs	@mod[3],$a[3],@acc[3]
	adcs	@mod[4],$a[4],@acc[4]
	adc	@mod[5],$a[5],@acc[5]

	subs	@acc[0],$a[0],$bi	// t1 = a->re - a->im
	sbcs	@acc[1],$a[1],@acc[1]
	sbcs	@acc[2],$a[2],@acc[2]
	sbcs	@acc[3],$a[3],@acc[3]
	sbcs	@acc[4],$a[4],@acc[4]
	sbcs	@acc[5],$a[5],@acc[5]
	sbc	@acc[6],xzr,xzr		// borrow flag as mask

	stp	@mod[0],@mod[1],[sp]
	stp	@mod[2],@mod[3],[sp,#16]
	stp	@mod[4],@mod[5],[sp,#32]
	stp	@acc[0],@acc[1],[sp,#48]
	stp	@acc[2],@acc[3],[sp,#64]
	stp	@acc[4],@acc[5],[sp,#80]
	str	@acc[6],[sp,#96]

	ldp	@mod[0],@mod[1],[$b_ptr]
	ldp	@mod[2],@mod[3],[$b_ptr,#16]
	ldp	@mod[4],@mod[5],[$b_ptr,#32]

	cadd	$b_ptr,$a_ptr,#48
	bl	__mul_mont_383_nonred	// mul_mont_384(ret->im, a->re, a->im)

	adds	@acc[0],@a[0],@a[0]	// add with itself
	adcs	@acc[1],@a[1],@a[1]
	adcs	@acc[2],@a[2],@a[2]
	adcs	@acc[3],@a[3],@a[3]
	adcs	@acc[4],@a[4],@a[4]
	adc	@acc[5],@a[5],@a[5]

	stp	@acc[0],@acc[1],[$b_ptr,#48]
	stp	@acc[2],@acc[3],[$b_ptr,#64]
	stp	@acc[4],@acc[5],[$b_ptr,#80]

	ldp	@a[0],@a[1],[sp]
	ldr	$bi,[sp,#48]
	ldp	@a[2],@a[3],[sp,#16]
	ldp	@a[4],@a[5],[sp,#32]

	cadd	$b_ptr,sp,#48
	bl	__mul_mont_383_nonred	// mul_mont_384(ret->im, t0, t1)
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	ldr	@acc[6],[sp,#96]	// account for sign from a->re - a->im
	ldp	@acc[0],@acc[1],[sp]
	ldp	@acc[2],@acc[3],[sp,#16]
	ldp	@acc[4],@acc[5],[sp,#32]

	and	@acc[0],@acc[0],@acc[6]
	and	@acc[1],@acc[1],@acc[6]
	and	@acc[2],@acc[2],@acc[6]
	and	@acc[3],@acc[3],@acc[6]
	and	@acc[4],@acc[4],@acc[6]
	and	@acc[5],@acc[5],@acc[6]

	subs	@a[0],@a[0],@acc[0]
	sbcs	@a[1],@a[1],@acc[1]
	sbcs	@a[2],@a[2],@acc[2]
	sbcs	@a[3],@a[3],@acc[3]
	sbcs	@a[4],@a[4],@acc[4]
	sbcs	@a[5],@a[5],@acc[5]
	sbc	@acc[6],xzr,xzr

	and	@acc[0],@mod[0],@acc[6]
	and	@acc[1],@mod[1],@acc[6]
	and	@acc[2],@mod[2],@acc[6]
	and	@acc[3],@mod[3],@acc[6]
	and	@acc[4],@mod[4],@acc[6]
	and	@acc[5],@mod[5],@acc[6]

	adds	@a[0],@a[0],@acc[0]
	adcs	@a[1],@a[1],@acc[1]
	adcs	@a[2],@a[2],@acc[2]
	adcs	@a[3],@a[3],@acc[3]
	adcs	@a[4],@a[4],@acc[4]
	adc	@a[5],@a[5],@acc[5]

	stp	@a[0],@a[1],[$b_ptr]
	stp	@a[2],@a[3],[$b_ptr,#16]
	stp	@a[4],@a[5],[$b_ptr,#32]

	add	csp,csp,#112
	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	sqr_mont_382x,.-sqr_mont_382x

.type	__mul_mont_383_nonred,%function
.align	5
__mul_mont_383_nonred:
	mul	@acc[0],@a[0],$bi
	mul	@acc[1],@a[1],$bi
	mul	@acc[2],@a[2],$bi
	mul	@acc[3],@a[3],$bi
	mul	@acc[4],@a[4],$bi
	mul	@acc[5],@a[5],$bi
	mul	$n0,$n0,@acc[0]

	 umulh	@tmp[0],@a[0],$bi
	 umulh	@tmp[1],@a[1],$bi
	 umulh	@tmp[2],@a[2],$bi
	 umulh	@tmp[3],@a[3],$bi
	 umulh	@tmp[4],@a[4],$bi
	 umulh	@tmp[5],@a[5],$bi

	 adds	@acc[1],@acc[1],@tmp[0]
	mul	@tmp[0],@mod[0],$n0
	 adcs	@acc[2],@acc[2],@tmp[1]
	mul	@tmp[1],@mod[1],$n0
	 adcs	@acc[3],@acc[3],@tmp[2]
	mul	@tmp[2],@mod[2],$n0
	 adcs	@acc[4],@acc[4],@tmp[3]
	mul	@tmp[3],@mod[3],$n0
	 adcs	@acc[5],@acc[5],@tmp[4]
	mul	@tmp[4],@mod[4],$n0
	 adc	@acc[6],xzr,    @tmp[5]
	mul	@tmp[5],@mod[5],$n0
___
for ($i=1;$i<6;$i++) {
$code.=<<___;
	ldr	$bi,[$b_ptr,8*$i]
	adds	@acc[0],@acc[0],@tmp[0]
	 umulh	@tmp[0],@mod[0],$n0
	adcs	@acc[1],@acc[1],@tmp[1]
	 umulh	@tmp[1],@mod[1],$n0
	adcs	@acc[2],@acc[2],@tmp[2]
	 umulh	@tmp[2],@mod[2],$n0
	adcs	@acc[3],@acc[3],@tmp[3]
	 umulh	@tmp[3],@mod[3],$n0
	adcs	@acc[4],@acc[4],@tmp[4]
	 umulh	@tmp[4],@mod[4],$n0
	adcs	@acc[5],@acc[5],@tmp[5]
	 umulh	@tmp[5],@mod[5],$n0
	adc	@acc[6],@acc[6],xzr

	ldr	$n0,[x29,#12*__SIZEOF_POINTER__]
	 adds	@acc[0],@acc[1],@tmp[0]
	mul	@tmp[0],@a[0],$bi
	 adcs	@acc[1],@acc[2],@tmp[1]
	mul	@tmp[1],@a[1],$bi
	 adcs	@acc[2],@acc[3],@tmp[2]
	mul	@tmp[2],@a[2],$bi
	 adcs	@acc[3],@acc[4],@tmp[3]
	mul	@tmp[3],@a[3],$bi
	 adcs	@acc[4],@acc[5],@tmp[4]
	mul	@tmp[4],@a[4],$bi
	 adcs	@acc[5],@acc[6],@tmp[5]
	mul	@tmp[5],@a[5],$bi
	 adc	@acc[6],xzr,xzr

	adds	@acc[0],@acc[0],@tmp[0]
	 umulh	@tmp[0],@a[0],$bi
	adcs	@acc[1],@acc[1],@tmp[1]
	 umulh	@tmp[1],@a[1],$bi
	adcs	@acc[2],@acc[2],@tmp[2]
	mul	$n0,$n0,@acc[0]
	 umulh	@tmp[2],@a[2],$bi
	adcs	@acc[3],@acc[3],@tmp[3]
	 umulh	@tmp[3],@a[3],$bi
	adcs	@acc[4],@acc[4],@tmp[4]
	 umulh	@tmp[4],@a[4],$bi
	adcs	@acc[5],@acc[5],@tmp[5]
	 umulh	@tmp[5],@a[5],$bi
	adc	@acc[6],@acc[6],xzr

	 adds	@acc[1],@acc[1],@tmp[0]
	mul	@tmp[0],@mod[0],$n0
	 adcs	@acc[2],@acc[2],@tmp[1]
	mul	@tmp[1],@mod[1],$n0
	 adcs	@acc[3],@acc[3],@tmp[2]
	mul	@tmp[2],@mod[2],$n0
	 adcs	@acc[4],@acc[4],@tmp[3]
	mul	@tmp[3],@mod[3],$n0
	 adcs	@acc[5],@acc[5],@tmp[4]
	mul	@tmp[4],@mod[4],$n0
	 adc	@acc[6],@acc[6],@tmp[5]
	mul	@tmp[5],@mod[5],$n0
___
}
$code.=<<___;
	adds	@acc[0],@acc[0],@tmp[0]
	 umulh	@tmp[0],@mod[0],$n0
	adcs	@acc[1],@acc[1],@tmp[1]
	 umulh	@tmp[1],@mod[1],$n0
	adcs	@acc[2],@acc[2],@tmp[2]
	 umulh	@tmp[2],@mod[2],$n0
	adcs	@acc[3],@acc[3],@tmp[3]
	 umulh	@tmp[3],@mod[3],$n0
	adcs	@acc[4],@acc[4],@tmp[4]
	 umulh	@tmp[4],@mod[4],$n0
	adcs	@acc[5],@acc[5],@tmp[5]
	 umulh	@tmp[5],@mod[5],$n0
	adc	@acc[6],@acc[6],xzr
	 ldp	c4,c2,[c29,#12*__SIZEOF_POINTER__]		// pull r_ptr

	 adds	@a[0],@acc[1],@tmp[0]
	 adcs	@a[1],@acc[2],@tmp[1]
	 adcs	@a[2],@acc[3],@tmp[2]
	 adcs	@a[3],@acc[4],@tmp[3]
	 adcs	@a[4],@acc[5],@tmp[4]
	 adcs	@a[5],@acc[6],@tmp[5]

	ret
.size	__mul_mont_383_nonred,.-__mul_mont_383_nonred

.globl	sgn0_pty_mont_384
.hidden	sgn0_pty_mont_384
.type	sgn0_pty_mont_384,%function
.align	5
sgn0_pty_mont_384:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]

	mov	$n0,$b_ptr
	ldp	@mod[0],@mod[1],[$a_ptr]
	ldp	@mod[2],@mod[3],[$a_ptr,#16]
	ldp	@mod[4],@mod[5],[$a_ptr,#32]
	cmov	$a_ptr,$r_ptr

	bl	__mul_by_1_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	and	$r_ptr,@a[0],#1
	adds	@a[0],@a[0],@a[0]
	adcs	@a[1],@a[1],@a[1]
	adcs	@a[2],@a[2],@a[2]
	adcs	@a[3],@a[3],@a[3]
	adcs	@a[4],@a[4],@a[4]
	adcs	@a[5],@a[5],@a[5]
	adc	$bi,xzr,xzr

	subs	@a[0],@a[0],@mod[0]
	sbcs	@a[1],@a[1],@mod[1]
	sbcs	@a[2],@a[2],@mod[2]
	sbcs	@a[3],@a[3],@mod[3]
	sbcs	@a[4],@a[4],@mod[4]
	sbcs	@a[5],@a[5],@mod[5]
	sbc	$bi,$bi,xzr

	mvn	$bi,$bi
	and	$bi,$bi,#2
	orr	$r_ptr,$r_ptr,$bi

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	sgn0_pty_mont_384,.-sgn0_pty_mont_384

.globl	sgn0_pty_mont_384x
.hidden	sgn0_pty_mont_384x
.type	sgn0_pty_mont_384x,%function
.align	5
sgn0_pty_mont_384x:
	paciasp
	stp	c29,c30,[csp,#-16*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]
	stp	c23,c24,[csp,#6*__SIZEOF_POINTER__]
	stp	c25,c26,[csp,#8*__SIZEOF_POINTER__]
	stp	c27,c28,[csp,#10*__SIZEOF_POINTER__]

	mov	$n0,$b_ptr
	ldp	@mod[0],@mod[1],[$a_ptr]
	ldp	@mod[2],@mod[3],[$a_ptr,#16]
	ldp	@mod[4],@mod[5],[$a_ptr,#32]
	cmov	$a_ptr,$r_ptr

	bl	__mul_by_1_mont_384
	cadd	$a_ptr,$a_ptr,#48

	and	$b_ptr,@a[0],#1
	 orr	$n_ptr,@a[0],@a[1]
	adds	@a[0],@a[0],@a[0]
	 orr	$n_ptr,$n_ptr,@a[2]
	adcs	@a[1],@a[1],@a[1]
	 orr	$n_ptr,$n_ptr,@a[3]
	adcs	@a[2],@a[2],@a[2]
	 orr	$n_ptr,$n_ptr,@a[4]
	adcs	@a[3],@a[3],@a[3]
	 orr	$n_ptr,$n_ptr,@a[5]
	adcs	@a[4],@a[4],@a[4]
	adcs	@a[5],@a[5],@a[5]
	adc	$bi,xzr,xzr

	subs	@a[0],@a[0],@mod[0]
	sbcs	@a[1],@a[1],@mod[1]
	sbcs	@a[2],@a[2],@mod[2]
	sbcs	@a[3],@a[3],@mod[3]
	sbcs	@a[4],@a[4],@mod[4]
	sbcs	@a[5],@a[5],@mod[5]
	sbc	$bi,$bi,xzr

	mvn	$bi,$bi
	and	$bi,$bi,#2
	orr	$b_ptr,$b_ptr,$bi

	bl	__mul_by_1_mont_384
	ldr	c30,[c29,#__SIZEOF_POINTER__]

	and	$r_ptr,@a[0],#1
	 orr	$a_ptr,@a[0],@a[1]
	adds	@a[0],@a[0],@a[0]
	 orr	$a_ptr,$a_ptr,@a[2]
	adcs	@a[1],@a[1],@a[1]
	 orr	$a_ptr,$a_ptr,@a[3]
	adcs	@a[2],@a[2],@a[2]
	 orr	$a_ptr,$a_ptr,@a[4]
	adcs	@a[3],@a[3],@a[3]
	 orr	$a_ptr,$a_ptr,@a[5]
	adcs	@a[4],@a[4],@a[4]
	adcs	@a[5],@a[5],@a[5]
	adc	$bi,xzr,xzr

	subs	@a[0],@a[0],@mod[0]
	sbcs	@a[1],@a[1],@mod[1]
	sbcs	@a[2],@a[2],@mod[2]
	sbcs	@a[3],@a[3],@mod[3]
	sbcs	@a[4],@a[4],@mod[4]
	sbcs	@a[5],@a[5],@mod[5]
	sbc	$bi,$bi,xzr

	mvn	$bi,$bi
	and	$bi,$bi,#2
	orr	$r_ptr,$r_ptr,$bi

	cmp	$n_ptr,#0
	csel	$n_ptr,$r_ptr,$b_ptr,eq	// a->re==0? prty(a->im) : prty(a->re)

	cmp	$a_ptr,#0
	csel	$a_ptr,$r_ptr,$b_ptr,ne	// a->im!=0? sgn0(a->im) : sgn0(a->re)

	and	$n_ptr,$n_ptr,#1
	and	$a_ptr,$a_ptr,#2
	orr	$r_ptr,$a_ptr,$n_ptr		// pack sign and parity

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldp	c23,c24,[c29,#6*__SIZEOF_POINTER__]
	ldp	c25,c26,[c29,#8*__SIZEOF_POINTER__]
	ldp	c27,c28,[c29,#10*__SIZEOF_POINTER__]
	ldr	c29,[csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	sgn0_pty_mont_384x,.-sgn0_pty_mont_384x
___

if (0) {
my @b = ($bi, @mod[0..4]);
my @comba = @acc[4..6];

$code.=<<___;
.type	__mul_384_comba,%function
.align	5
__mul_384_comba:
	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@b[0],@b[1],[$b_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]
	ldp	@b[2],@b[3],[$b_ptr,#16]
	ldp	@b[4],@b[5],[$b_ptr,#32]

	mul	@comba[0],@a[0],@b[0]
	umulh	@comba[1],@a[0],@b[0]
	 mul	@acc[0],@a[1],@b[0]
	 umulh	@acc[1],@a[1],@b[0]
	str	@comba[0],[$r_ptr]
___
	push(@comba,shift(@comba));
$code.=<<___;
	mul	@acc[2],@a[0],@b[1]
	umulh	@acc[3],@a[0],@b[1]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],xzr,      @acc[1]
	adc	@comba[2],xzr,xzr
	mul	@acc[0],@a[2],@b[0]
	umulh	@acc[1],@a[2],@b[0]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	str	@comba[0],[$r_ptr,#8]
___
	push(@comba,shift(@comba));
$code.=<<___;
	mul	@acc[2],@a[1],@b[1]
	umulh	@acc[3],@a[1],@b[1]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],xzr,xzr
	mul	@acc[0],@a[0],@b[2]
	umulh	@acc[1],@a[0],@b[2]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	 mul	@acc[2],@a[3],@b[0]
	 umulh	@acc[3],@a[3],@b[0]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	str	@comba[0],[$r_ptr,#16]
___
	push(@comba,shift(@comba));
$code.=<<___;
	mul	@acc[0],@a[2],@b[1]
	umulh	@acc[1],@a[2],@b[1]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],xzr,xzr
	mul	@acc[2],@a[1],@b[2]
	umulh	@acc[3],@a[1],@b[2]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	mul	@acc[0],@a[0],@b[3]
	umulh	@acc[1],@a[0],@b[3]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	 mul	@acc[2],@a[4],@b[0]
	 umulh	@acc[3],@a[4],@b[0]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	str	@comba[0],[$r_ptr,#24]
___
	push(@comba,shift(@comba));
$code.=<<___;
	mul	@acc[0],@a[3],@b[1]
	umulh	@acc[1],@a[3],@b[1]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],xzr,xzr
	mul	@acc[2],@a[2],@b[2]
	umulh	@acc[3],@a[2],@b[2]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	mul	@acc[0],@a[1],@b[3]
	umulh	@acc[1],@a[1],@b[3]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	mul	@acc[2],@a[0],@b[4]
	umulh	@acc[3],@a[0],@b[4]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	 mul	@acc[0],@a[5],@b[0]
	 umulh	@acc[1],@a[5],@b[0]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	str	@comba[0],[$r_ptr,#32]
___
	push(@comba,shift(@comba));
$code.=<<___;
	mul	@acc[2],@a[4],@b[1]
	umulh	@acc[3],@a[4],@b[1]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],xzr,xzr
	mul	@acc[0],@a[3],@b[2]
	umulh	@acc[1],@a[3],@b[2]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	mul	@acc[2],@a[2],@b[3]
	umulh	@acc[3],@a[2],@b[3]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	mul	@acc[0],@a[1],@b[4]
	umulh	@acc[1],@a[1],@b[4]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	mul	@acc[2],@a[0],@b[5]
	umulh	@acc[3],@a[0],@b[5]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	 mul	@acc[0],@a[5],@b[1]
	 umulh	@acc[1],@a[5],@b[1]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	str	@comba[0],[$r_ptr,#40]
___
	push(@comba,shift(@comba));
$code.=<<___;
	mul	@acc[2],@a[4],@b[2]
	umulh	@acc[3],@a[4],@b[2]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],xzr,xzr
	mul	@acc[0],@a[3],@b[3]
	umulh	@acc[1],@a[3],@b[3]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	mul	@acc[2],@a[2],@b[4]
	umulh	@acc[3],@a[2],@b[4]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	mul	@acc[0],@a[1],@b[5]
	umulh	@acc[1],@a[1],@b[5]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	 mul	@acc[2],@a[5],@b[2]
	 umulh	@acc[3],@a[5],@b[2]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	str	@comba[0],[$r_ptr,#48]
___
	push(@comba,shift(@comba));
$code.=<<___;
	mul	@acc[0],@a[4],@b[3]
	umulh	@acc[1],@a[4],@b[3]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],xzr,xzr
	mul	@acc[2],@a[3],@b[4]
	umulh	@acc[3],@a[3],@b[4]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	mul	@acc[0],@a[2],@b[5]
	umulh	@acc[1],@a[2],@b[5]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	 mul	@acc[2],@a[5],@b[3]
	 umulh	@acc[3],@a[5],@b[3]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	str	@comba[0],[$r_ptr,#56]
___
	push(@comba,shift(@comba));
$code.=<<___;
	mul	@acc[0],@a[4],@b[4]
	umulh	@acc[1],@a[4],@b[4]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],xzr,xzr
	mul	@acc[2],@a[3],@b[5]
	umulh	@acc[3],@a[3],@b[5]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],@comba[2],xzr
	 mul	@acc[0],@a[5],@b[4]
	 umulh	@acc[1],@a[5],@b[4]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	str	@comba[0],[$r_ptr,#64]
___
	push(@comba,shift(@comba));
$code.=<<___;
	mul	@acc[2],@a[4],@b[5]
	umulh	@acc[3],@a[4],@b[5]
	adds	@comba[0],@comba[0],@acc[0]
	adcs	@comba[1],@comba[1],@acc[1]
	adc	@comba[2],xzr,xzr
	 mul	@acc[0],@a[5],@b[5]
	 umulh	@acc[1],@a[5],@b[5]
	adds	@comba[0],@comba[0],@acc[2]
	adcs	@comba[1],@comba[1],@acc[3]
	adc	@comba[2],@comba[2],xzr
	str	@comba[0],[$r_ptr,#72]
___
	push(@comba,shift(@comba));
$code.=<<___;
	adds	@comba[0],@comba[0],@acc[0]
	adc	@comba[1],@comba[1],@acc[1]
	stp	@comba[0],@comba[1],[$r_ptr,#80]

	ret
.size	__mul_384_comba,.-__mul_384_comba
___
}
print $code;

close STDOUT;
