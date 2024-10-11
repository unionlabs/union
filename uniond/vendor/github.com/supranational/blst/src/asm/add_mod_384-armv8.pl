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

($r_ptr,$a_ptr,$b_ptr,$n_ptr) = map("x$_", 0..3);

@mod=map("x$_",(4..9));
@a=map("x$_",(10..15));
@b=map("x$_",(16,17,19..22));
$carry=$n_ptr;

$code.=<<___;
.text

.globl	add_mod_384
.hidden	add_mod_384
.type	add_mod_384,%function
.align	5
add_mod_384:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

	bl	__add_mod_384
	ldr	c30,[csp,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$r_ptr]
	stp	@a[2],@a[3],[$r_ptr,#16]
	stp	@a[4],@a[5],[$r_ptr,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	add_mod_384,.-add_mod_384

.type	__add_mod_384,%function
.align	5
__add_mod_384:
	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@b[0],@b[1],[$b_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@b[2],@b[3],[$b_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]
	ldp	@b[4],@b[5],[$b_ptr,#32]

__add_mod_384_ab_are_loaded:
	adds	@a[0],@a[0],@b[0]
	adcs	@a[1],@a[1],@b[1]
	adcs	@a[2],@a[2],@b[2]
	adcs	@a[3],@a[3],@b[3]
	adcs	@a[4],@a[4],@b[4]
	adcs	@a[5],@a[5],@b[5]
	adc	$carry,xzr,xzr

	subs	@b[0],@a[0],@mod[0]
	sbcs	@b[1],@a[1],@mod[1]
	sbcs	@b[2],@a[2],@mod[2]
	sbcs	@b[3],@a[3],@mod[3]
	sbcs	@b[4],@a[4],@mod[4]
	sbcs	@b[5],@a[5],@mod[5]
	sbcs	xzr,$carry,xzr

	csel	@a[0],@a[0],@b[0],lo
	csel	@a[1],@a[1],@b[1],lo
	csel	@a[2],@a[2],@b[2],lo
	csel	@a[3],@a[3],@b[3],lo
	csel	@a[4],@a[4],@b[4],lo
	csel	@a[5],@a[5],@b[5],lo

	ret
.size	__add_mod_384,.-__add_mod_384

.globl	add_mod_384x
.hidden	add_mod_384x
.type	add_mod_384x,%function
.align	5
add_mod_384x:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

	bl	__add_mod_384

	stp	@a[0],@a[1],[$r_ptr]
	cadd	$a_ptr,$a_ptr,#48
	stp	@a[2],@a[3],[$r_ptr,#16]
	cadd	$b_ptr,$b_ptr,#48
	stp	@a[4],@a[5],[$r_ptr,#32]

	bl	__add_mod_384
	ldr	c30,[csp,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$r_ptr,#48]
	stp	@a[2],@a[3],[$r_ptr,#64]
	stp	@a[4],@a[5],[$r_ptr,#80]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	add_mod_384x,.-add_mod_384x

.globl	rshift_mod_384
.hidden	rshift_mod_384
.type	rshift_mod_384,%function
.align	5
rshift_mod_384:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

.Loop_rshift_mod_384:
	sub	$b_ptr,$b_ptr,#1
	bl	__rshift_mod_384
	cbnz	$b_ptr,.Loop_rshift_mod_384

	ldr	c30,[csp,#__SIZEOF_POINTER__]
	stp	@a[0],@a[1],[$r_ptr]
	stp	@a[2],@a[3],[$r_ptr,#16]
	stp	@a[4],@a[5],[$r_ptr,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	rshift_mod_384,.-rshift_mod_384

.type	__rshift_mod_384,%function
.align	5
__rshift_mod_384:
	sbfx	@b[5],@a[0],#0,#1
	 and	@b[0],@b[5],@mod[0]
	 and	@b[1],@b[5],@mod[1]
	adds	@a[0],@a[0],@b[0]
	 and	@b[2],@b[5],@mod[2]
	adcs	@a[1],@a[1],@b[1]
	 and	@b[3],@b[5],@mod[3]
	adcs	@a[2],@a[2],@b[2]
	 and	@b[4],@b[5],@mod[4]
	adcs	@a[3],@a[3],@b[3]
	 and	@b[5],@b[5],@mod[5]
	adcs	@a[4],@a[4],@b[4]
	 extr	@a[0],@a[1],@a[0],#1	// a[0:5] >>= 1
	adcs	@a[5],@a[5],@b[5]
	 extr	@a[1],@a[2],@a[1],#1
	adc	@b[5],xzr,xzr
	 extr	@a[2],@a[3],@a[2],#1
	 extr	@a[3],@a[4],@a[3],#1
	 extr	@a[4],@a[5],@a[4],#1
	 extr	@a[5],@b[5],@a[5],#1
	ret
.size	__rshift_mod_384,.-__rshift_mod_384

.globl	div_by_2_mod_384
.hidden	div_by_2_mod_384
.type	div_by_2_mod_384,%function
.align	5
div_by_2_mod_384:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	ldp	@mod[0],@mod[1],[$b_ptr]
	ldp	@mod[2],@mod[3],[$b_ptr,#16]
	ldp	@mod[4],@mod[5],[$b_ptr,#32]

	bl	__rshift_mod_384

	ldr	c30,[csp,#__SIZEOF_POINTER__]
	stp	@a[0],@a[1],[$r_ptr]
	stp	@a[2],@a[3],[$r_ptr,#16]
	stp	@a[4],@a[5],[$r_ptr,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	div_by_2_mod_384,.-div_by_2_mod_384

.globl	lshift_mod_384
.hidden	lshift_mod_384
.type	lshift_mod_384,%function
.align	5
lshift_mod_384:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

.Loop_lshift_mod_384:
	sub	$b_ptr,$b_ptr,#1
	bl	__lshift_mod_384
	cbnz	$b_ptr,.Loop_lshift_mod_384

	ldr	c30,[csp,#__SIZEOF_POINTER__]
	stp	@a[0],@a[1],[$r_ptr]
	stp	@a[2],@a[3],[$r_ptr,#16]
	stp	@a[4],@a[5],[$r_ptr,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	lshift_mod_384,.-lshift_mod_384

.type	__lshift_mod_384,%function
.align	5
__lshift_mod_384:
	adds	@a[0],@a[0],@a[0]
	adcs	@a[1],@a[1],@a[1]
	adcs	@a[2],@a[2],@a[2]
	adcs	@a[3],@a[3],@a[3]
	adcs	@a[4],@a[4],@a[4]
	adcs	@a[5],@a[5],@a[5]
	adc	$carry,xzr,xzr

	subs	@b[0],@a[0],@mod[0]
	sbcs	@b[1],@a[1],@mod[1]
	sbcs	@b[2],@a[2],@mod[2]
	sbcs	@b[3],@a[3],@mod[3]
	sbcs	@b[4],@a[4],@mod[4]
	sbcs	@b[5],@a[5],@mod[5]
	sbcs	xzr,$carry,xzr

	csel	@a[0],@a[0],@b[0],lo
	csel	@a[1],@a[1],@b[1],lo
	csel	@a[2],@a[2],@b[2],lo
	csel	@a[3],@a[3],@b[3],lo
	csel	@a[4],@a[4],@b[4],lo
	csel	@a[5],@a[5],@b[5],lo

	ret
.size	__lshift_mod_384,.-__lshift_mod_384

.globl	mul_by_3_mod_384
.hidden	mul_by_3_mod_384
.type	mul_by_3_mod_384,%function
.align	5
mul_by_3_mod_384:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	ldp	@mod[0],@mod[1],[$b_ptr]
	ldp	@mod[2],@mod[3],[$b_ptr,#16]
	ldp	@mod[4],@mod[5],[$b_ptr,#32]

	bl	__lshift_mod_384

	ldp	@b[0],@b[1],[$a_ptr]
	ldp	@b[2],@b[3],[$a_ptr,#16]
	ldp	@b[4],@b[5],[$a_ptr,#32]

	bl	__add_mod_384_ab_are_loaded
	ldr	c30,[csp,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$r_ptr]
	stp	@a[2],@a[3],[$r_ptr,#16]
	stp	@a[4],@a[5],[$r_ptr,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	mul_by_3_mod_384,.-mul_by_3_mod_384

.globl	mul_by_8_mod_384
.hidden	mul_by_8_mod_384
.type	mul_by_8_mod_384,%function
.align	5
mul_by_8_mod_384:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	ldp	@mod[0],@mod[1],[$b_ptr]
	ldp	@mod[2],@mod[3],[$b_ptr,#16]
	ldp	@mod[4],@mod[5],[$b_ptr,#32]

	bl	__lshift_mod_384
	bl	__lshift_mod_384
	bl	__lshift_mod_384
	ldr	c30,[csp,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$r_ptr]
	stp	@a[2],@a[3],[$r_ptr,#16]
	stp	@a[4],@a[5],[$r_ptr,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	mul_by_8_mod_384,.-mul_by_8_mod_384

.globl	mul_by_3_mod_384x
.hidden	mul_by_3_mod_384x
.type	mul_by_3_mod_384x,%function
.align	5
mul_by_3_mod_384x:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	ldp	@mod[0],@mod[1],[$b_ptr]
	ldp	@mod[2],@mod[3],[$b_ptr,#16]
	ldp	@mod[4],@mod[5],[$b_ptr,#32]

	bl	__lshift_mod_384

	ldp	@b[0],@b[1],[$a_ptr]
	ldp	@b[2],@b[3],[$a_ptr,#16]
	ldp	@b[4],@b[5],[$a_ptr,#32]

	bl	__add_mod_384_ab_are_loaded

	stp	@a[0],@a[1],[$r_ptr]
	ldp	@a[0],@a[1],[$a_ptr,#48]
	stp	@a[2],@a[3],[$r_ptr,#16]
	ldp	@a[2],@a[3],[$a_ptr,#64]
	stp	@a[4],@a[5],[$r_ptr,#32]
	ldp	@a[4],@a[5],[$a_ptr,#80]

	bl	__lshift_mod_384

	ldp	@b[0],@b[1],[$a_ptr,#48]
	ldp	@b[2],@b[3],[$a_ptr,#64]
	ldp	@b[4],@b[5],[$a_ptr,#80]

	bl	__add_mod_384_ab_are_loaded
	ldr	c30,[csp,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$r_ptr,#48]
	stp	@a[2],@a[3],[$r_ptr,#64]
	stp	@a[4],@a[5],[$r_ptr,#80]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	mul_by_3_mod_384x,.-mul_by_3_mod_384x

.globl	mul_by_8_mod_384x
.hidden	mul_by_8_mod_384x
.type	mul_by_8_mod_384x,%function
.align	5
mul_by_8_mod_384x:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]

	ldp	@mod[0],@mod[1],[$b_ptr]
	ldp	@mod[2],@mod[3],[$b_ptr,#16]
	ldp	@mod[4],@mod[5],[$b_ptr,#32]

	bl	__lshift_mod_384
	bl	__lshift_mod_384
	bl	__lshift_mod_384

	stp	@a[0],@a[1],[$r_ptr]
	ldp	@a[0],@a[1],[$a_ptr,#48]
	stp	@a[2],@a[3],[$r_ptr,#16]
	ldp	@a[2],@a[3],[$a_ptr,#64]
	stp	@a[4],@a[5],[$r_ptr,#32]
	ldp	@a[4],@a[5],[$a_ptr,#80]

	bl	__lshift_mod_384
	bl	__lshift_mod_384
	bl	__lshift_mod_384
	ldr	c30,[csp,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$r_ptr,#48]
	stp	@a[2],@a[3],[$r_ptr,#64]
	stp	@a[4],@a[5],[$r_ptr,#80]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	mul_by_8_mod_384x,.-mul_by_8_mod_384x

.globl	cneg_mod_384
.hidden	cneg_mod_384
.type	cneg_mod_384,%function
.align	5
cneg_mod_384:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]

	subs	@b[0],@mod[0],@a[0]
	ldp	@a[4],@a[5],[$a_ptr,#32]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]
	 orr	$carry,@a[0],@a[1]
	sbcs	@b[1],@mod[1],@a[1]
	 orr	$carry,$carry,@a[2]
	sbcs	@b[2],@mod[2],@a[2]
	 orr	$carry,$carry,@a[3]
	sbcs	@b[3],@mod[3],@a[3]
	 orr	$carry,$carry,@a[4]
	sbcs	@b[4],@mod[4],@a[4]
	 orr	$carry,$carry,@a[5]
	sbc	@b[5],@mod[5],@a[5]

	cmp	$carry,#0
	csetm	$carry,ne
	ands	$b_ptr,$b_ptr,$carry

	csel	@a[0],@a[0],@b[0],eq
	csel	@a[1],@a[1],@b[1],eq
	csel	@a[2],@a[2],@b[2],eq
	csel	@a[3],@a[3],@b[3],eq
	stp	@a[0],@a[1],[$r_ptr]
	csel	@a[4],@a[4],@b[4],eq
	stp	@a[2],@a[3],[$r_ptr,#16]
	csel	@a[5],@a[5],@b[5],eq
	stp	@a[4],@a[5],[$r_ptr,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	cneg_mod_384,.-cneg_mod_384

.globl	sub_mod_384
.hidden	sub_mod_384
.type	sub_mod_384,%function
.align	5
sub_mod_384:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

	bl	__sub_mod_384
	ldr	c30,[csp,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$r_ptr]
	stp	@a[2],@a[3],[$r_ptr,#16]
	stp	@a[4],@a[5],[$r_ptr,#32]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	sub_mod_384,.-sub_mod_384

.type	__sub_mod_384,%function
.align	5
__sub_mod_384:
	ldp	@a[0],@a[1],[$a_ptr]
	ldp	@b[0],@b[1],[$b_ptr]
	ldp	@a[2],@a[3],[$a_ptr,#16]
	ldp	@b[2],@b[3],[$b_ptr,#16]
	ldp	@a[4],@a[5],[$a_ptr,#32]
	ldp	@b[4],@b[5],[$b_ptr,#32]

	subs	@a[0],@a[0],@b[0]
	sbcs	@a[1],@a[1],@b[1]
	sbcs	@a[2],@a[2],@b[2]
	sbcs	@a[3],@a[3],@b[3]
	sbcs	@a[4],@a[4],@b[4]
	sbcs	@a[5],@a[5],@b[5]
	sbc	$carry,xzr,xzr

	 and	@b[0],@mod[0],$carry
	 and	@b[1],@mod[1],$carry
	adds	@a[0],@a[0],@b[0]
	 and	@b[2],@mod[2],$carry
	adcs	@a[1],@a[1],@b[1]
	 and	@b[3],@mod[3],$carry
	adcs	@a[2],@a[2],@b[2]
	 and	@b[4],@mod[4],$carry
	adcs	@a[3],@a[3],@b[3]
	 and	@b[5],@mod[5],$carry
	adcs	@a[4],@a[4],@b[4]
	adc	@a[5],@a[5],@b[5]

	ret
.size	__sub_mod_384,.-__sub_mod_384

.globl	sub_mod_384x
.hidden	sub_mod_384x
.type	sub_mod_384x,%function
.align	5
sub_mod_384x:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@mod[0],@mod[1],[$n_ptr]
	ldp	@mod[2],@mod[3],[$n_ptr,#16]
	ldp	@mod[4],@mod[5],[$n_ptr,#32]

	bl	__sub_mod_384

	stp	@a[0],@a[1],[$r_ptr]
	cadd	$a_ptr,$a_ptr,#48
	stp	@a[2],@a[3],[$r_ptr,#16]
	cadd	$b_ptr,$b_ptr,#48
	stp	@a[4],@a[5],[$r_ptr,#32]

	bl	__sub_mod_384
	ldr	c30,[csp,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$r_ptr,#48]
	stp	@a[2],@a[3],[$r_ptr,#64]
	stp	@a[4],@a[5],[$r_ptr,#80]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	sub_mod_384x,.-sub_mod_384x

.globl	mul_by_1_plus_i_mod_384x
.hidden	mul_by_1_plus_i_mod_384x
.type	mul_by_1_plus_i_mod_384x,%function
.align	5
mul_by_1_plus_i_mod_384x:
	paciasp
	stp	c29,c30,[csp,#-6*__SIZEOF_POINTER__]!
	add	c29,csp,#0
	stp	c19,c20,[csp,#2*__SIZEOF_POINTER__]
	stp	c21,c22,[csp,#4*__SIZEOF_POINTER__]

	ldp	@mod[0],@mod[1],[$b_ptr]
	ldp	@mod[2],@mod[3],[$b_ptr,#16]
	ldp	@mod[4],@mod[5],[$b_ptr,#32]
	cadd	$b_ptr,$a_ptr,#48

	bl	__sub_mod_384			// a->re - a->im

	ldp	@b[0],@b[1],[$a_ptr]
	ldp	@b[2],@b[3],[$a_ptr,#16]
	ldp	@b[4],@b[5],[$a_ptr,#32]
	stp	@a[0],@a[1],[$r_ptr]
	ldp	@a[0],@a[1],[$a_ptr,#48]
	stp	@a[2],@a[3],[$r_ptr,#16]
	ldp	@a[2],@a[3],[$a_ptr,#64]
	stp	@a[4],@a[5],[$r_ptr,#32]
	ldp	@a[4],@a[5],[$a_ptr,#80]

	bl	__add_mod_384_ab_are_loaded	// a->re + a->im
	ldr	c30,[csp,#__SIZEOF_POINTER__]

	stp	@a[0],@a[1],[$r_ptr,#48]
	stp	@a[2],@a[3],[$r_ptr,#64]
	stp	@a[4],@a[5],[$r_ptr,#80]

	ldp	c19,c20,[c29,#2*__SIZEOF_POINTER__]
	ldp	c21,c22,[c29,#4*__SIZEOF_POINTER__]
	ldr	c29,[csp],#6*__SIZEOF_POINTER__
	autiasp
	ret
.size	mul_by_1_plus_i_mod_384x,.-mul_by_1_plus_i_mod_384x

.globl	sgn0_pty_mod_384
.hidden	sgn0_pty_mod_384
.type	sgn0_pty_mod_384,%function
.align	5
sgn0_pty_mod_384:
	ldp	@a[0],@a[1],[$r_ptr]
	ldp	@a[2],@a[3],[$r_ptr,#16]
	ldp	@a[4],@a[5],[$r_ptr,#32]

	ldp	@mod[0],@mod[1],[$a_ptr]
	ldp	@mod[2],@mod[3],[$a_ptr,#16]
	ldp	@mod[4],@mod[5],[$a_ptr,#32]

	and	$r_ptr,@a[0],#1
	adds	@a[0],@a[0],@a[0]
	adcs	@a[1],@a[1],@a[1]
	adcs	@a[2],@a[2],@a[2]
	adcs	@a[3],@a[3],@a[3]
	adcs	@a[4],@a[4],@a[4]
	adcs	@a[5],@a[5],@a[5]
	adc	$carry,xzr,xzr

	subs	@a[0],@a[0],@mod[0]
	sbcs	@a[1],@a[1],@mod[1]
	sbcs	@a[2],@a[2],@mod[2]
	sbcs	@a[3],@a[3],@mod[3]
	sbcs	@a[4],@a[4],@mod[4]
	sbcs	@a[5],@a[5],@mod[5]
	sbc	$carry,$carry,xzr

	mvn	$carry,$carry
	and	$carry,$carry,#2
	orr	$r_ptr,$r_ptr,$carry

	ret
.size	sgn0_pty_mod_384,.-sgn0_pty_mod_384

.globl	sgn0_pty_mod_384x
.hidden	sgn0_pty_mod_384x
.type	sgn0_pty_mod_384x,%function
.align	5
sgn0_pty_mod_384x:
	ldp	@a[0],@a[1],[$r_ptr]
	ldp	@a[2],@a[3],[$r_ptr,#16]
	ldp	@a[4],@a[5],[$r_ptr,#32]

	ldp	@mod[0],@mod[1],[$a_ptr]
	ldp	@mod[2],@mod[3],[$a_ptr,#16]
	ldp	@mod[4],@mod[5],[$a_ptr,#32]

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
	adc	@b[0],xzr,xzr

	subs	@a[0],@a[0],@mod[0]
	sbcs	@a[1],@a[1],@mod[1]
	sbcs	@a[2],@a[2],@mod[2]
	sbcs	@a[3],@a[3],@mod[3]
	sbcs	@a[4],@a[4],@mod[4]
	sbcs	@a[5],@a[5],@mod[5]
	sbc	@b[0],@b[0],xzr

	ldp	@a[0],@a[1],[$r_ptr,#48]
	ldp	@a[2],@a[3],[$r_ptr,#64]
	ldp	@a[4],@a[5],[$r_ptr,#80]

	mvn	@b[0],@b[0]
	and	@b[0],@b[0],#2
	orr	$b_ptr,$b_ptr,@b[0]

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
	adc	@b[0],xzr,xzr

	subs	@a[0],@a[0],@mod[0]
	sbcs	@a[1],@a[1],@mod[1]
	sbcs	@a[2],@a[2],@mod[2]
	sbcs	@a[3],@a[3],@mod[3]
	sbcs	@a[4],@a[4],@mod[4]
	sbcs	@a[5],@a[5],@mod[5]
	sbc	@b[0],@b[0],xzr

	mvn	@b[0],@b[0]
	and	@b[0],@b[0],#2
	orr	$r_ptr,$r_ptr,@b[0]

	cmp	$n_ptr,#0
	csel	$n_ptr,$r_ptr,$b_ptr,eq	// a->re==0? prty(a->im) : prty(a->re)

	cmp	$a_ptr,#0
	csel	$a_ptr,$r_ptr,$b_ptr,ne	// a->im!=0? sgn0(a->im) : sgn0(a->re)

	and	$n_ptr,$n_ptr,#1
	and	$a_ptr,$a_ptr,#2
	orr	$r_ptr,$a_ptr,$n_ptr	// pack sign and parity

	ret
.size	sgn0_pty_mod_384x,.-sgn0_pty_mod_384x
___
if (1) {
sub vec_select {
my $sz = shift;
my @v=map("v$_",(0..5,16..21));

$code.=<<___;
.globl	vec_select_$sz
.hidden	vec_select_$sz
.type	vec_select_$sz,%function
.align	5
vec_select_$sz:
	dup	v6.2d, $n_ptr
	ld1	{@v[0].2d, @v[1].2d, @v[2].2d}, [$a_ptr],#48
	cmeq	v6.2d, v6.2d, #0
	ld1	{@v[3].2d, @v[4].2d, @v[5].2d}, [$b_ptr],#48
___
for($i=0; $i<$sz-48; $i+=48) {
$code.=<<___;
	bit	@v[0].16b, @v[3].16b, v6.16b
	ld1	{@v[6].2d, @v[7].2d, @v[8].2d}, [$a_ptr],#48
	bit	@v[1].16b, @v[4].16b, v6.16b
	ld1	{@v[9].2d, @v[10].2d, @v[11].2d}, [$b_ptr],#48
	bit	@v[2].16b, @v[5].16b, v6.16b
	st1	{@v[0].2d, @v[1].2d, @v[2].2d}, [$r_ptr],#48
___
	@v = @v[6..11,0..5];
}
$code.=<<___;
	bit	@v[0].16b, @v[3].16b, v6.16b
	bit	@v[1].16b, @v[4].16b, v6.16b
	bit	@v[2].16b, @v[5].16b, v6.16b
	st1	{@v[0].2d, @v[1].2d, @v[2].2d}, [$r_ptr]
	ret
.size	vec_select_$sz,.-vec_select_$sz
___
}
vec_select(32);
vec_select(48);
vec_select(96);
vec_select(192);
vec_select(144);
vec_select(288);
}

{
my ($inp, $end, $step) = map("x$_", (0..2));

$code.=<<___;
.globl	vec_prefetch
.hidden	vec_prefetch
.type	vec_prefetch,%function
.align	5
vec_prefetch:
	add	$end, $end, $inp
	sub	$end, $end, #1
	mov	$step, #64
	prfm	pldl1keep, [$inp]
	add	$inp, $inp, $step
	cmp	$inp, $end
	csel	$inp, $end, $inp, hi
	csel	$step, xzr, $step, hi
	prfm	pldl1keep, [$inp]
	add	$inp, $inp, $step
	cmp	$inp, $end
	csel	$inp, $end, $inp, hi
	csel	$step, xzr, $step, hi
	prfm	pldl1keep, [$inp]
	add	$inp, $inp, $step
	cmp	$inp, $end
	csel	$inp, $end, $inp, hi
	csel	$step, xzr, $step, hi
	prfm	pldl1keep, [$inp]
	add	$inp, $inp, $step
	cmp	$inp, $end
	csel	$inp, $end, $inp, hi
	csel	$step, xzr, $step, hi
	prfm	pldl1keep, [$inp]
	add	$inp, $inp, $step
	cmp	$inp, $end
	csel	$inp, $end, $inp, hi
	csel	$step, xzr, $step, hi
	prfm	pldl1keep, [$inp]
	add	$inp, $inp, $step
	cmp	$inp, $end
	csel	$inp, $end, $inp, hi
	prfm	pldl1keep, [$inp]
	ret
.size	vec_prefetch,.-vec_prefetch
___
my $len = $end;

$code.=<<___;
.globl	vec_is_zero_16x
.hidden	vec_is_zero_16x
.type	vec_is_zero_16x,%function
.align	5
vec_is_zero_16x:
	ld1	{v0.2d}, [$inp], #16
	lsr	$len, $len, #4
	sub	$len, $len, #1
	cbz	$len, .Loop_is_zero_done

.Loop_is_zero:
	ld1	{v1.2d}, [$inp], #16
	orr	v0.16b, v0.16b, v1.16b
	sub	$len, $len, #1
	cbnz	$len, .Loop_is_zero

.Loop_is_zero_done:
	dup	v1.2d, v0.2d[1]
	orr	v0.16b, v0.16b, v1.16b
	umov	x1, v0.2d[0]
	mov	x0, #1
	cmp	x1, #0
	csel	x0, x0, xzr, eq
	ret
.size	vec_is_zero_16x,.-vec_is_zero_16x
___
}
{
my ($inp1, $inp2, $len) = map("x$_", (0..2));

$code.=<<___;
.globl	vec_is_equal_16x
.hidden	vec_is_equal_16x
.type	vec_is_equal_16x,%function
.align	5
vec_is_equal_16x:
	ld1	{v0.2d}, [$inp1], #16
	ld1	{v1.2d}, [$inp2], #16
	lsr	$len, $len, #4
	eor	v0.16b, v0.16b, v1.16b

.Loop_is_equal:
	sub	$len, $len, #1
	cbz	$len, .Loop_is_equal_done
	ld1	{v1.2d}, [$inp1], #16
	ld1	{v2.2d}, [$inp2], #16
	eor	v1.16b, v1.16b, v2.16b
	orr	v0.16b, v0.16b, v1.16b
	b	.Loop_is_equal
	nop

.Loop_is_equal_done:
	dup	v1.2d, v0.2d[1]
	orr	v0.16b, v0.16b, v1.16b
	umov	x1, v0.2d[0]
	mov	x0, #1
	cmp	x1, #0
	csel	x0, x0, xzr, eq
	ret
.size	vec_is_equal_16x,.-vec_is_equal_16x
___
}

print $code;

close STDOUT;
