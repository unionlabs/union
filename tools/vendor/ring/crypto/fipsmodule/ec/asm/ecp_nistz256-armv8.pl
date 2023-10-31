#! /usr/bin/env perl
# Copyright 2015-2016 The OpenSSL Project Authors. All Rights Reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions
# are met:
#
# 1. Redistributions of source code must retain the above copyright
#    notice, this list of conditions and the following disclaimer. 
#
# 2. Redistributions in binary form must reproduce the above copyright
#    notice, this list of conditions and the following disclaimer in
#    the documentation and/or other materials provided with the
#    distribution.
#
# 3. All advertising materials mentioning features or use of this
#    software must display the following acknowledgment:
#    "This product includes software developed by the OpenSSL Project
#    for use in the OpenSSL Toolkit. (http://www.openssl.org/)"
#
# 4. The names "OpenSSL Toolkit" and "OpenSSL Project" must not be used to
#    endorse or promote products derived from this software without
#    prior written permission. For written permission, please contact
#    openssl-core@openssl.org.
#
# 5. Products derived from this software may not be called "OpenSSL"
#    nor may "OpenSSL" appear in their names without prior written
#    permission of the OpenSSL Project.
#
# 6. Redistributions of any form whatsoever must retain the following
#    acknowledgment:
#    "This product includes software developed by the OpenSSL Project
#    for use in the OpenSSL Toolkit (http://www.openssl.org/)"
#
# THIS SOFTWARE IS PROVIDED BY THE OpenSSL PROJECT ``AS IS'' AND ANY
# EXPRESSED OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
# IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
# PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL THE OpenSSL PROJECT OR
# ITS CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
# SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
# NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
# LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
# HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
# STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
# ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED
# OF THE POSSIBILITY OF SUCH DAMAGE.
# ====================================================================
#
# This product includes cryptographic software written by Eric Young
# (eay@cryptsoft.com).  This product includes software written by Tim
# Hudson (tjh@cryptsoft.com).


# ====================================================================
# Written by Andy Polyakov <appro@openssl.org> for the OpenSSL
# project. The module is, however, dual licensed under OpenSSL and
# CRYPTOGAMS licenses depending on where you obtain it. For further
# details see http://www.openssl.org/~appro/cryptogams/.
# ====================================================================
#
# ECP_NISTZ256 module for ARMv8.
#
# February 2015.
#
# Original ECP_NISTZ256 submission targeting x86_64 is detailed in
# http://eprint.iacr.org/2013/816.
#
#			with/without -DECP_NISTZ256_ASM
# Apple A7		+120-360%
# Cortex-A53		+120-400%
# Cortex-A57		+120-350%
# X-Gene		+200-330%
# Denver		+140-400%
#
# Ranges denote minimum and maximum improvement coefficients depending
# on benchmark. Lower coefficients are for ECDSA sign, server-side
# operation. Keep in mind that +400% means 5x improvement.

$flavour = shift;
while (($output=shift) && ($output!~/\w[\w\-]*\.\w+$/)) {}

$0 =~ m/(.*[\/\\])[^\/\\]+$/; $dir=$1;
( $xlate="${dir}arm-xlate.pl" and -f $xlate ) or
( $xlate="${dir}../../../perlasm/arm-xlate.pl" and -f $xlate) or
die "can't locate arm-xlate.pl";

open OUT,"| \"$^X\" $xlate $flavour $output";
*STDOUT=*OUT;

{
my ($rp,$ap,$bp,$bi,$a0,$a1,$a2,$a3,$t0,$t1,$t2,$t3,$poly1,$poly3,
    $acc0,$acc1,$acc2,$acc3,$acc4,$acc5) =
    map("x$_",(0..17,19,20));

my ($acc6,$acc7)=($ap,$bp);	# used in __ecp_nistz256_sqr_mont

$code.=<<___;
#include <GFp/arm_arch.h>

.text
.align	5
.Lpoly:
.quad	0xffffffffffffffff,0x00000000ffffffff,0x0000000000000000,0xffffffff00000001
.Lone_mont:
.quad	0x0000000000000001,0xffffffff00000000,0xffffffffffffffff,0x00000000fffffffe
.Lone:
.quad	1,0,0,0
.asciz	"ECP_NISTZ256 for ARMv8, CRYPTOGAMS by <appro\@openssl.org>"

// void	GFp_nistz256_mul_mont(BN_ULONG x0[4],const BN_ULONG x1[4],
//					     const BN_ULONG x2[4]);
.globl	GFp_nistz256_mul_mont
.type	GFp_nistz256_mul_mont,%function
.align	4
GFp_nistz256_mul_mont:
	stp	x29,x30,[sp,#-32]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]

	ldr	$bi,[$bp]		// bp[0]
	ldp	$a0,$a1,[$ap]
	ldp	$a2,$a3,[$ap,#16]
	ldr	$poly1,.Lpoly+8
	ldr	$poly3,.Lpoly+24

	bl	__ecp_nistz256_mul_mont

	ldp	x19,x20,[sp,#16]
	ldp	x29,x30,[sp],#32
	ret
.size	GFp_nistz256_mul_mont,.-GFp_nistz256_mul_mont

// void	GFp_nistz256_sqr_mont(BN_ULONG x0[4],const BN_ULONG x1[4]);
.globl	GFp_nistz256_sqr_mont
.type	GFp_nistz256_sqr_mont,%function
.align	4
GFp_nistz256_sqr_mont:
	stp	x29,x30,[sp,#-32]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]

	ldp	$a0,$a1,[$ap]
	ldp	$a2,$a3,[$ap,#16]
	ldr	$poly1,.Lpoly+8
	ldr	$poly3,.Lpoly+24

	bl	__ecp_nistz256_sqr_mont

	ldp	x19,x20,[sp,#16]
	ldp	x29,x30,[sp],#32
	ret
.size	GFp_nistz256_sqr_mont,.-GFp_nistz256_sqr_mont

// void	GFp_nistz256_add(BN_ULONG x0[4],const BN_ULONG x1[4],
//					const BN_ULONG x2[4]);
.globl	GFp_nistz256_add
.type	GFp_nistz256_add,%function
.align	4
GFp_nistz256_add:
	stp	x29,x30,[sp,#-16]!
	add	x29,sp,#0

	ldp	$acc0,$acc1,[$ap]
	ldp	$t0,$t1,[$bp]
	ldp	$acc2,$acc3,[$ap,#16]
	ldp	$t2,$t3,[$bp,#16]
	ldr	$poly1,.Lpoly+8
	ldr	$poly3,.Lpoly+24

	bl	__ecp_nistz256_add

	ldp	x29,x30,[sp],#16
	ret
.size	GFp_nistz256_add,.-GFp_nistz256_add

// void	GFp_nistz256_neg(BN_ULONG x0[4],const BN_ULONG x1[4]);
.globl	GFp_nistz256_neg
.type	GFp_nistz256_neg,%function
.align	4
GFp_nistz256_neg:
	stp	x29,x30,[sp,#-16]!
	add	x29,sp,#0

	mov	$bp,$ap
	mov	$acc0,xzr		// a = 0
	mov	$acc1,xzr
	mov	$acc2,xzr
	mov	$acc3,xzr
	ldr	$poly1,.Lpoly+8
	ldr	$poly3,.Lpoly+24

	bl	__ecp_nistz256_sub_from

	ldp	x29,x30,[sp],#16
	ret
.size	GFp_nistz256_neg,.-GFp_nistz256_neg

// note that __ecp_nistz256_mul_mont expects a[0-3] input pre-loaded
// to $a0-$a3 and b[0] - to $bi
.type	__ecp_nistz256_mul_mont,%function
.align	4
__ecp_nistz256_mul_mont:
	mul	$acc0,$a0,$bi		// a[0]*b[0]
	umulh	$t0,$a0,$bi

	mul	$acc1,$a1,$bi		// a[1]*b[0]
	umulh	$t1,$a1,$bi

	mul	$acc2,$a2,$bi		// a[2]*b[0]
	umulh	$t2,$a2,$bi

	mul	$acc3,$a3,$bi		// a[3]*b[0]
	umulh	$t3,$a3,$bi
	ldr	$bi,[$bp,#8]		// b[1]

	adds	$acc1,$acc1,$t0		// accumulate high parts of multiplication
	 lsl	$t0,$acc0,#32
	adcs	$acc2,$acc2,$t1
	 lsr	$t1,$acc0,#32
	adcs	$acc3,$acc3,$t2
	adc	$acc4,xzr,$t3
	mov	$acc5,xzr
___
for($i=1;$i<4;$i++) {
        # Reduction iteration is normally performed by accumulating
        # result of multiplication of modulus by "magic" digit [and
        # omitting least significant word, which is guaranteed to
        # be 0], but thanks to special form of modulus and "magic"
        # digit being equal to least significant word, it can be
        # performed with additions and subtractions alone. Indeed:
        #
        #            ffff0001.00000000.0000ffff.ffffffff
        # *                                     abcdefgh
        # + xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.abcdefgh
        #
        # Now observing that ff..ff*x = (2^n-1)*x = 2^n*x-x, we
        # rewrite above as:
        #
        #   xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.abcdefgh
        # + abcdefgh.abcdefgh.0000abcd.efgh0000.00000000
        # - 0000abcd.efgh0000.00000000.00000000.abcdefgh
        #
        # or marking redundant operations:
        #
        #   xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx.--------
        # + abcdefgh.abcdefgh.0000abcd.efgh0000.--------
        # - 0000abcd.efgh0000.--------.--------.--------

$code.=<<___;
	subs	$t2,$acc0,$t0		// "*0xffff0001"
	sbc	$t3,$acc0,$t1
	adds	$acc0,$acc1,$t0		// +=acc[0]<<96 and omit acc[0]
	 mul	$t0,$a0,$bi		// lo(a[0]*b[i])
	adcs	$acc1,$acc2,$t1
	 mul	$t1,$a1,$bi		// lo(a[1]*b[i])
	adcs	$acc2,$acc3,$t2		// +=acc[0]*0xffff0001
	 mul	$t2,$a2,$bi		// lo(a[2]*b[i])
	adcs	$acc3,$acc4,$t3
	 mul	$t3,$a3,$bi		// lo(a[3]*b[i])
	adc	$acc4,$acc5,xzr

	adds	$acc0,$acc0,$t0		// accumulate low parts of multiplication
	 umulh	$t0,$a0,$bi		// hi(a[0]*b[i])
	adcs	$acc1,$acc1,$t1
	 umulh	$t1,$a1,$bi		// hi(a[1]*b[i])
	adcs	$acc2,$acc2,$t2
	 umulh	$t2,$a2,$bi		// hi(a[2]*b[i])
	adcs	$acc3,$acc3,$t3
	 umulh	$t3,$a3,$bi		// hi(a[3]*b[i])
	adc	$acc4,$acc4,xzr
___
$code.=<<___	if ($i<3);
	ldr	$bi,[$bp,#8*($i+1)]	// b[$i+1]
___
$code.=<<___;
	adds	$acc1,$acc1,$t0		// accumulate high parts of multiplication
	 lsl	$t0,$acc0,#32
	adcs	$acc2,$acc2,$t1
	 lsr	$t1,$acc0,#32
	adcs	$acc3,$acc3,$t2
	adcs	$acc4,$acc4,$t3
	adc	$acc5,xzr,xzr
___
}
$code.=<<___;
	// last reduction
	subs	$t2,$acc0,$t0		// "*0xffff0001"
	sbc	$t3,$acc0,$t1
	adds	$acc0,$acc1,$t0		// +=acc[0]<<96 and omit acc[0]
	adcs	$acc1,$acc2,$t1
	adcs	$acc2,$acc3,$t2		// +=acc[0]*0xffff0001
	adcs	$acc3,$acc4,$t3
	adc	$acc4,$acc5,xzr

	adds	$t0,$acc0,#1		// subs	$t0,$acc0,#-1 // tmp = ret-modulus
	sbcs	$t1,$acc1,$poly1
	sbcs	$t2,$acc2,xzr
	sbcs	$t3,$acc3,$poly3
	sbcs	xzr,$acc4,xzr		// did it borrow?

	csel	$acc0,$acc0,$t0,lo	// ret = borrow ? ret : ret-modulus
	csel	$acc1,$acc1,$t1,lo
	csel	$acc2,$acc2,$t2,lo
	stp	$acc0,$acc1,[$rp]
	csel	$acc3,$acc3,$t3,lo
	stp	$acc2,$acc3,[$rp,#16]

	ret
.size	__ecp_nistz256_mul_mont,.-__ecp_nistz256_mul_mont

// note that __ecp_nistz256_sqr_mont expects a[0-3] input pre-loaded
// to $a0-$a3
.type	__ecp_nistz256_sqr_mont,%function
.align	4
__ecp_nistz256_sqr_mont:
	//  |  |  |  |  |  |a1*a0|  |
	//  |  |  |  |  |a2*a0|  |  |
	//  |  |a3*a2|a3*a0|  |  |  |
	//  |  |  |  |a2*a1|  |  |  |
	//  |  |  |a3*a1|  |  |  |  |
	// *|  |  |  |  |  |  |  | 2|
	// +|a3*a3|a2*a2|a1*a1|a0*a0|
	//  |--+--+--+--+--+--+--+--|
	//  |A7|A6|A5|A4|A3|A2|A1|A0|, where Ax is $accx, i.e. follow $accx
	//
	//  "can't overflow" below mark carrying into high part of
	//  multiplication result, which can't overflow, because it
	//  can never be all ones.

	mul	$acc1,$a1,$a0		// a[1]*a[0]
	umulh	$t1,$a1,$a0
	mul	$acc2,$a2,$a0		// a[2]*a[0]
	umulh	$t2,$a2,$a0
	mul	$acc3,$a3,$a0		// a[3]*a[0]
	umulh	$acc4,$a3,$a0

	adds	$acc2,$acc2,$t1		// accumulate high parts of multiplication
	 mul	$t0,$a2,$a1		// a[2]*a[1]
	 umulh	$t1,$a2,$a1
	adcs	$acc3,$acc3,$t2
	 mul	$t2,$a3,$a1		// a[3]*a[1]
	 umulh	$t3,$a3,$a1
	adc	$acc4,$acc4,xzr		// can't overflow

	mul	$acc5,$a3,$a2		// a[3]*a[2]
	umulh	$acc6,$a3,$a2

	adds	$t1,$t1,$t2		// accumulate high parts of multiplication
	 mul	$acc0,$a0,$a0		// a[0]*a[0]
	adc	$t2,$t3,xzr		// can't overflow

	adds	$acc3,$acc3,$t0		// accumulate low parts of multiplication
	 umulh	$a0,$a0,$a0
	adcs	$acc4,$acc4,$t1
	 mul	$t1,$a1,$a1		// a[1]*a[1]
	adcs	$acc5,$acc5,$t2
	 umulh	$a1,$a1,$a1
	adc	$acc6,$acc6,xzr		// can't overflow

	adds	$acc1,$acc1,$acc1	// acc[1-6]*=2
	 mul	$t2,$a2,$a2		// a[2]*a[2]
	adcs	$acc2,$acc2,$acc2
	 umulh	$a2,$a2,$a2
	adcs	$acc3,$acc3,$acc3
	 mul	$t3,$a3,$a3		// a[3]*a[3]
	adcs	$acc4,$acc4,$acc4
	 umulh	$a3,$a3,$a3
	adcs	$acc5,$acc5,$acc5
	adcs	$acc6,$acc6,$acc6
	adc	$acc7,xzr,xzr

	adds	$acc1,$acc1,$a0		// +a[i]*a[i]
	adcs	$acc2,$acc2,$t1
	adcs	$acc3,$acc3,$a1
	adcs	$acc4,$acc4,$t2
	adcs	$acc5,$acc5,$a2
	 lsl	$t0,$acc0,#32
	adcs	$acc6,$acc6,$t3
	 lsr	$t1,$acc0,#32
	adc	$acc7,$acc7,$a3
___
for($i=0;$i<3;$i++) {			# reductions, see commentary in
					# multiplication for details
$code.=<<___;
	subs	$t2,$acc0,$t0		// "*0xffff0001"
	sbc	$t3,$acc0,$t1
	adds	$acc0,$acc1,$t0		// +=acc[0]<<96 and omit acc[0]
	adcs	$acc1,$acc2,$t1
	 lsl	$t0,$acc0,#32
	adcs	$acc2,$acc3,$t2		// +=acc[0]*0xffff0001
	 lsr	$t1,$acc0,#32
	adc	$acc3,$t3,xzr		// can't overflow
___
}
$code.=<<___;
	subs	$t2,$acc0,$t0		// "*0xffff0001"
	sbc	$t3,$acc0,$t1
	adds	$acc0,$acc1,$t0		// +=acc[0]<<96 and omit acc[0]
	adcs	$acc1,$acc2,$t1
	adcs	$acc2,$acc3,$t2		// +=acc[0]*0xffff0001
	adc	$acc3,$t3,xzr		// can't overflow

	adds	$acc0,$acc0,$acc4	// accumulate upper half
	adcs	$acc1,$acc1,$acc5
	adcs	$acc2,$acc2,$acc6
	adcs	$acc3,$acc3,$acc7
	adc	$acc4,xzr,xzr

	adds	$t0,$acc0,#1		// subs	$t0,$acc0,#-1 // tmp = ret-modulus
	sbcs	$t1,$acc1,$poly1
	sbcs	$t2,$acc2,xzr
	sbcs	$t3,$acc3,$poly3
	sbcs	xzr,$acc4,xzr		// did it borrow?

	csel	$acc0,$acc0,$t0,lo	// ret = borrow ? ret : ret-modulus
	csel	$acc1,$acc1,$t1,lo
	csel	$acc2,$acc2,$t2,lo
	stp	$acc0,$acc1,[$rp]
	csel	$acc3,$acc3,$t3,lo
	stp	$acc2,$acc3,[$rp,#16]

	ret
.size	__ecp_nistz256_sqr_mont,.-__ecp_nistz256_sqr_mont

// Note that __ecp_nistz256_add expects both input vectors pre-loaded to
// $a0-$a3 and $t0-$t3. This is done because it's used in multiple
// contexts, e.g. in multiplication by 2 and 3...
.type	__ecp_nistz256_add,%function
.align	4
__ecp_nistz256_add:
	adds	$acc0,$acc0,$t0		// ret = a+b
	adcs	$acc1,$acc1,$t1
	adcs	$acc2,$acc2,$t2
	adcs	$acc3,$acc3,$t3
	adc	$ap,xzr,xzr		// zap $ap

	adds	$t0,$acc0,#1		// subs	$t0,$a0,#-1 // tmp = ret-modulus
	sbcs	$t1,$acc1,$poly1
	sbcs	$t2,$acc2,xzr
	sbcs	$t3,$acc3,$poly3
	sbcs	xzr,$ap,xzr		// did subtraction borrow?

	csel	$acc0,$acc0,$t0,lo	// ret = borrow ? ret : ret-modulus
	csel	$acc1,$acc1,$t1,lo
	csel	$acc2,$acc2,$t2,lo
	stp	$acc0,$acc1,[$rp]
	csel	$acc3,$acc3,$t3,lo
	stp	$acc2,$acc3,[$rp,#16]

	ret
.size	__ecp_nistz256_add,.-__ecp_nistz256_add

.type	__ecp_nistz256_sub_from,%function
.align	4
__ecp_nistz256_sub_from:
	ldp	$t0,$t1,[$bp]
	ldp	$t2,$t3,[$bp,#16]
	subs	$acc0,$acc0,$t0		// ret = a-b
	sbcs	$acc1,$acc1,$t1
	sbcs	$acc2,$acc2,$t2
	sbcs	$acc3,$acc3,$t3
	sbc	$ap,xzr,xzr		// zap $ap

	subs	$t0,$acc0,#1		// adds	$t0,$a0,#-1 // tmp = ret+modulus
	adcs	$t1,$acc1,$poly1
	adcs	$t2,$acc2,xzr
	adc	$t3,$acc3,$poly3
	cmp	$ap,xzr			// did subtraction borrow?

	csel	$acc0,$acc0,$t0,eq	// ret = borrow ? ret+modulus : ret
	csel	$acc1,$acc1,$t1,eq
	csel	$acc2,$acc2,$t2,eq
	stp	$acc0,$acc1,[$rp]
	csel	$acc3,$acc3,$t3,eq
	stp	$acc2,$acc3,[$rp,#16]

	ret
.size	__ecp_nistz256_sub_from,.-__ecp_nistz256_sub_from

.type	__ecp_nistz256_sub_morf,%function
.align	4
__ecp_nistz256_sub_morf:
	ldp	$t0,$t1,[$bp]
	ldp	$t2,$t3,[$bp,#16]
	subs	$acc0,$t0,$acc0		// ret = b-a
	sbcs	$acc1,$t1,$acc1
	sbcs	$acc2,$t2,$acc2
	sbcs	$acc3,$t3,$acc3
	sbc	$ap,xzr,xzr		// zap $ap

	subs	$t0,$acc0,#1		// adds	$t0,$a0,#-1 // tmp = ret+modulus
	adcs	$t1,$acc1,$poly1
	adcs	$t2,$acc2,xzr
	adc	$t3,$acc3,$poly3
	cmp	$ap,xzr			// did subtraction borrow?

	csel	$acc0,$acc0,$t0,eq	// ret = borrow ? ret+modulus : ret
	csel	$acc1,$acc1,$t1,eq
	csel	$acc2,$acc2,$t2,eq
	stp	$acc0,$acc1,[$rp]
	csel	$acc3,$acc3,$t3,eq
	stp	$acc2,$acc3,[$rp,#16]

	ret
.size	__ecp_nistz256_sub_morf,.-__ecp_nistz256_sub_morf

.type	__ecp_nistz256_div_by_2,%function
.align	4
__ecp_nistz256_div_by_2:
	subs	$t0,$acc0,#1		// adds	$t0,$a0,#-1 // tmp = a+modulus
	adcs	$t1,$acc1,$poly1
	adcs	$t2,$acc2,xzr
	adcs	$t3,$acc3,$poly3
	adc	$ap,xzr,xzr		// zap $ap
	tst	$acc0,#1		// is a even?

	csel	$acc0,$acc0,$t0,eq	// ret = even ? a : a+modulus
	csel	$acc1,$acc1,$t1,eq
	csel	$acc2,$acc2,$t2,eq
	csel	$acc3,$acc3,$t3,eq
	csel	$ap,xzr,$ap,eq

	lsr	$acc0,$acc0,#1		// ret >>= 1
	orr	$acc0,$acc0,$acc1,lsl#63
	lsr	$acc1,$acc1,#1
	orr	$acc1,$acc1,$acc2,lsl#63
	lsr	$acc2,$acc2,#1
	orr	$acc2,$acc2,$acc3,lsl#63
	lsr	$acc3,$acc3,#1
	stp	$acc0,$acc1,[$rp]
	orr	$acc3,$acc3,$ap,lsl#63
	stp	$acc2,$acc3,[$rp,#16]

	ret
.size	__ecp_nistz256_div_by_2,.-__ecp_nistz256_div_by_2
___
########################################################################
# following subroutines are "literal" implementation of those found in
# ecp_nistz256.c
#
########################################################################
# void GFp_nistz256_point_double(P256_POINT *out,const P256_POINT *inp);
#
{
my ($S,$M,$Zsqr,$tmp0)=map(32*$_,(0..3));
# above map() describes stack layout with 4 temporary
# 256-bit vectors on top.
my ($rp_real,$ap_real) = map("x$_",(21,22));

$code.=<<___;
.globl	GFp_nistz256_point_double
.type	GFp_nistz256_point_double,%function
.align	5
GFp_nistz256_point_double:
	stp	x29,x30,[sp,#-80]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	sub	sp,sp,#32*4

.Ldouble_shortcut:
	ldp	$acc0,$acc1,[$ap,#32]
	 mov	$rp_real,$rp
	ldp	$acc2,$acc3,[$ap,#48]
	 mov	$ap_real,$ap
	 ldr	$poly1,.Lpoly+8
	mov	$t0,$acc0
	 ldr	$poly3,.Lpoly+24
	mov	$t1,$acc1
	 ldp	$a0,$a1,[$ap_real,#64]	// forward load for p256_sqr_mont
	mov	$t2,$acc2
	mov	$t3,$acc3
	 ldp	$a2,$a3,[$ap_real,#64+16]
	add	$rp,sp,#$S
	bl	__ecp_nistz256_add	// p256_mul_by_2(S, in_y);

	add	$rp,sp,#$Zsqr
	bl	__ecp_nistz256_sqr_mont	// p256_sqr_mont(Zsqr, in_z);

	ldp	$t0,$t1,[$ap_real]
	ldp	$t2,$t3,[$ap_real,#16]
	mov	$a0,$acc0		// put Zsqr aside for p256_sub
	mov	$a1,$acc1
	mov	$a2,$acc2
	mov	$a3,$acc3
	add	$rp,sp,#$M
	bl	__ecp_nistz256_add	// p256_add(M, Zsqr, in_x);

	add	$bp,$ap_real,#0
	mov	$acc0,$a0		// restore Zsqr
	mov	$acc1,$a1
	 ldp	$a0,$a1,[sp,#$S]	// forward load for p256_sqr_mont
	mov	$acc2,$a2
	mov	$acc3,$a3
	 ldp	$a2,$a3,[sp,#$S+16]
	add	$rp,sp,#$Zsqr
	bl	__ecp_nistz256_sub_morf	// p256_sub(Zsqr, in_x, Zsqr);

	add	$rp,sp,#$S
	bl	__ecp_nistz256_sqr_mont	// p256_sqr_mont(S, S);

	ldr	$bi,[$ap_real,#32]
	ldp	$a0,$a1,[$ap_real,#64]
	ldp	$a2,$a3,[$ap_real,#64+16]
	add	$bp,$ap_real,#32
	add	$rp,sp,#$tmp0
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(tmp0, in_z, in_y);

	mov	$t0,$acc0
	mov	$t1,$acc1
	 ldp	$a0,$a1,[sp,#$S]	// forward load for p256_sqr_mont
	mov	$t2,$acc2
	mov	$t3,$acc3
	 ldp	$a2,$a3,[sp,#$S+16]
	add	$rp,$rp_real,#64
	bl	__ecp_nistz256_add	// p256_mul_by_2(res_z, tmp0);

	add	$rp,sp,#$tmp0
	bl	__ecp_nistz256_sqr_mont	// p256_sqr_mont(tmp0, S);

	 ldr	$bi,[sp,#$Zsqr]		// forward load for p256_mul_mont
	 ldp	$a0,$a1,[sp,#$M]
	 ldp	$a2,$a3,[sp,#$M+16]
	add	$rp,$rp_real,#32
	bl	__ecp_nistz256_div_by_2	// p256_div_by_2(res_y, tmp0);

	add	$bp,sp,#$Zsqr
	add	$rp,sp,#$M
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(M, M, Zsqr);

	mov	$t0,$acc0		// duplicate M
	mov	$t1,$acc1
	mov	$t2,$acc2
	mov	$t3,$acc3
	mov	$a0,$acc0		// put M aside
	mov	$a1,$acc1
	mov	$a2,$acc2
	mov	$a3,$acc3
	add	$rp,sp,#$M
	bl	__ecp_nistz256_add
	mov	$t0,$a0			// restore M
	mov	$t1,$a1
	 ldr	$bi,[$ap_real]		// forward load for p256_mul_mont
	mov	$t2,$a2
	 ldp	$a0,$a1,[sp,#$S]
	mov	$t3,$a3
	 ldp	$a2,$a3,[sp,#$S+16]
	bl	__ecp_nistz256_add	// p256_mul_by_3(M, M);

	add	$bp,$ap_real,#0
	add	$rp,sp,#$S
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(S, S, in_x);

	mov	$t0,$acc0
	mov	$t1,$acc1
	 ldp	$a0,$a1,[sp,#$M]	// forward load for p256_sqr_mont
	mov	$t2,$acc2
	mov	$t3,$acc3
	 ldp	$a2,$a3,[sp,#$M+16]
	add	$rp,sp,#$tmp0
	bl	__ecp_nistz256_add	// p256_mul_by_2(tmp0, S);

	add	$rp,$rp_real,#0
	bl	__ecp_nistz256_sqr_mont	// p256_sqr_mont(res_x, M);

	add	$bp,sp,#$tmp0
	bl	__ecp_nistz256_sub_from	// p256_sub(res_x, res_x, tmp0);

	add	$bp,sp,#$S
	add	$rp,sp,#$S
	bl	__ecp_nistz256_sub_morf	// p256_sub(S, S, res_x);

	ldr	$bi,[sp,#$M]
	mov	$a0,$acc0		// copy S
	mov	$a1,$acc1
	mov	$a2,$acc2
	mov	$a3,$acc3
	add	$bp,sp,#$M
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(S, S, M);

	add	$bp,$rp_real,#32
	add	$rp,$rp_real,#32
	bl	__ecp_nistz256_sub_from	// p256_sub(res_y, S, res_y);

	add	sp,x29,#0		// destroy frame
	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x29,x30,[sp],#80
	ret
.size	GFp_nistz256_point_double,.-GFp_nistz256_point_double
___
}

########################################################################
# void GFp_nistz256_point_add_affine(P256_POINT *out,const P256_POINT *in1,
#				     const P256_POINT_AFFINE *in2);
{
my ($res_x,$res_y,$res_z,
    $U2,$S2,$H,$R,$Hsqr,$Hcub,$Rsqr)=map(32*$_,(0..9));
my $Z1sqr = $S2;
# above map() describes stack layout with 10 temporary
# 256-bit vectors on top.
my ($rp_real,$ap_real,$bp_real,$in1infty,$in2infty,$temp)=map("x$_",(21..26));

$code.=<<___;
.globl	GFp_nistz256_point_add_affine
.type	GFp_nistz256_point_add_affine,%function
.align	5
GFp_nistz256_point_add_affine:
	stp	x29,x30,[sp,#-80]!
	add	x29,sp,#0
	stp	x19,x20,[sp,#16]
	stp	x21,x22,[sp,#32]
	stp	x23,x24,[sp,#48]
	stp	x25,x26,[sp,#64]
	sub	sp,sp,#32*10

	mov	$rp_real,$rp
	mov	$ap_real,$ap
	mov	$bp_real,$bp
	ldr	$poly1,.Lpoly+8
	ldr	$poly3,.Lpoly+24

	ldp	$a0,$a1,[$ap,#64]	// in1_z
	ldp	$a2,$a3,[$ap,#64+16]
	orr	$t0,$a0,$a1
	orr	$t2,$a2,$a3
	orr	$in1infty,$t0,$t2
	cmp	$in1infty,#0
	csetm	$in1infty,ne		// !in1infty

	ldp	$acc0,$acc1,[$bp]	// in2_x
	ldp	$acc2,$acc3,[$bp,#16]
	ldp	$t0,$t1,[$bp,#32]	// in2_y
	ldp	$t2,$t3,[$bp,#48]
	orr	$acc0,$acc0,$acc1
	orr	$acc2,$acc2,$acc3
	orr	$t0,$t0,$t1
	orr	$t2,$t2,$t3
	orr	$acc0,$acc0,$acc2
	orr	$t0,$t0,$t2
	orr	$in2infty,$acc0,$t0
	cmp	$in2infty,#0
	csetm	$in2infty,ne		// !in2infty

	add	$rp,sp,#$Z1sqr
	bl	__ecp_nistz256_sqr_mont	// p256_sqr_mont(Z1sqr, in1_z);

	mov	$a0,$acc0
	mov	$a1,$acc1
	mov	$a2,$acc2
	mov	$a3,$acc3
	ldr	$bi,[$bp_real]
	add	$bp,$bp_real,#0
	add	$rp,sp,#$U2
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(U2, Z1sqr, in2_x);

	add	$bp,$ap_real,#0
	 ldr	$bi,[$ap_real,#64]	// forward load for p256_mul_mont
	 ldp	$a0,$a1,[sp,#$Z1sqr]
	 ldp	$a2,$a3,[sp,#$Z1sqr+16]
	add	$rp,sp,#$H
	bl	__ecp_nistz256_sub_from	// p256_sub(H, U2, in1_x);

	add	$bp,$ap_real,#64
	add	$rp,sp,#$S2
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(S2, Z1sqr, in1_z);

	ldr	$bi,[$ap_real,#64]
	ldp	$a0,$a1,[sp,#$H]
	ldp	$a2,$a3,[sp,#$H+16]
	add	$bp,$ap_real,#64
	add	$rp,sp,#$res_z
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(res_z, H, in1_z);

	ldr	$bi,[$bp_real,#32]
	ldp	$a0,$a1,[sp,#$S2]
	ldp	$a2,$a3,[sp,#$S2+16]
	add	$bp,$bp_real,#32
	add	$rp,sp,#$S2
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(S2, S2, in2_y);

	add	$bp,$ap_real,#32
	 ldp	$a0,$a1,[sp,#$H]	// forward load for p256_sqr_mont
	 ldp	$a2,$a3,[sp,#$H+16]
	add	$rp,sp,#$R
	bl	__ecp_nistz256_sub_from	// p256_sub(R, S2, in1_y);

	add	$rp,sp,#$Hsqr
	bl	__ecp_nistz256_sqr_mont	// p256_sqr_mont(Hsqr, H);

	ldp	$a0,$a1,[sp,#$R]
	ldp	$a2,$a3,[sp,#$R+16]
	add	$rp,sp,#$Rsqr
	bl	__ecp_nistz256_sqr_mont	// p256_sqr_mont(Rsqr, R);

	ldr	$bi,[sp,#$H]
	ldp	$a0,$a1,[sp,#$Hsqr]
	ldp	$a2,$a3,[sp,#$Hsqr+16]
	add	$bp,sp,#$H
	add	$rp,sp,#$Hcub
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(Hcub, Hsqr, H);

	ldr	$bi,[$ap_real]
	ldp	$a0,$a1,[sp,#$Hsqr]
	ldp	$a2,$a3,[sp,#$Hsqr+16]
	add	$bp,$ap_real,#0
	add	$rp,sp,#$U2
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(U2, in1_x, Hsqr);

	mov	$t0,$acc0
	mov	$t1,$acc1
	mov	$t2,$acc2
	mov	$t3,$acc3
	add	$rp,sp,#$Hsqr
	bl	__ecp_nistz256_add	// p256_mul_by_2(Hsqr, U2);

	add	$bp,sp,#$Rsqr
	add	$rp,sp,#$res_x
	bl	__ecp_nistz256_sub_morf	// p256_sub(res_x, Rsqr, Hsqr);

	add	$bp,sp,#$Hcub
	bl	__ecp_nistz256_sub_from	//  p256_sub(res_x, res_x, Hcub);

	add	$bp,sp,#$U2
	 ldr	$bi,[$ap_real,#32]	// forward load for p256_mul_mont
	 ldp	$a0,$a1,[sp,#$Hcub]
	 ldp	$a2,$a3,[sp,#$Hcub+16]
	add	$rp,sp,#$res_y
	bl	__ecp_nistz256_sub_morf	// p256_sub(res_y, U2, res_x);

	add	$bp,$ap_real,#32
	add	$rp,sp,#$S2
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(S2, in1_y, Hcub);

	ldr	$bi,[sp,#$R]
	ldp	$a0,$a1,[sp,#$res_y]
	ldp	$a2,$a3,[sp,#$res_y+16]
	add	$bp,sp,#$R
	add	$rp,sp,#$res_y
	bl	__ecp_nistz256_mul_mont	// p256_mul_mont(res_y, res_y, R);

	add	$bp,sp,#$S2
	bl	__ecp_nistz256_sub_from	// p256_sub(res_y, res_y, S2);

	ldp	$a0,$a1,[sp,#$res_x]		// res
	ldp	$a2,$a3,[sp,#$res_x+16]
	ldp	$t0,$t1,[$bp_real]		// in2
	ldp	$t2,$t3,[$bp_real,#16]
___
for($i=0;$i<64;$i+=32) {		# conditional moves
$code.=<<___;
	ldp	$acc0,$acc1,[$ap_real,#$i]	// in1
	cmp	$in1infty,#0			// !$in1intfy, remember?
	ldp	$acc2,$acc3,[$ap_real,#$i+16]
	csel	$t0,$a0,$t0,ne
	csel	$t1,$a1,$t1,ne
	ldp	$a0,$a1,[sp,#$res_x+$i+32]	// res
	csel	$t2,$a2,$t2,ne
	csel	$t3,$a3,$t3,ne
	cmp	$in2infty,#0			// !$in2intfy, remember?
	ldp	$a2,$a3,[sp,#$res_x+$i+48]
	csel	$acc0,$t0,$acc0,ne
	csel	$acc1,$t1,$acc1,ne
	ldp	$t0,$t1,[$bp_real,#$i+32]	// in2
	csel	$acc2,$t2,$acc2,ne
	csel	$acc3,$t3,$acc3,ne
	ldp	$t2,$t3,[$bp_real,#$i+48]
	stp	$acc0,$acc1,[$rp_real,#$i]
	stp	$acc2,$acc3,[$rp_real,#$i+16]
___
$code.=<<___	if ($i == 0);
	adr	$bp_real,.Lone_mont-64
___
}
$code.=<<___;
	ldp	$acc0,$acc1,[$ap_real,#$i]	// in1
	cmp	$in1infty,#0			// !$in1intfy, remember?
	ldp	$acc2,$acc3,[$ap_real,#$i+16]
	csel	$t0,$a0,$t0,ne
	csel	$t1,$a1,$t1,ne
	csel	$t2,$a2,$t2,ne
	csel	$t3,$a3,$t3,ne
	cmp	$in2infty,#0			// !$in2intfy, remember?
	csel	$acc0,$t0,$acc0,ne
	csel	$acc1,$t1,$acc1,ne
	csel	$acc2,$t2,$acc2,ne
	csel	$acc3,$t3,$acc3,ne
	stp	$acc0,$acc1,[$rp_real,#$i]
	stp	$acc2,$acc3,[$rp_real,#$i+16]

	add	sp,x29,#0		// destroy frame
	ldp	x19,x20,[x29,#16]
	ldp	x21,x22,[x29,#32]
	ldp	x23,x24,[x29,#48]
	ldp	x25,x26,[x29,#64]
	ldp	x29,x30,[sp],#80
	ret
.size	GFp_nistz256_point_add_affine,.-GFp_nistz256_point_add_affine
___
}	}

foreach (split("\n",$code)) {
	s/\`([^\`]*)\`/eval $1/ge;

	print $_,"\n";
}
close STDOUT or die "error closing STDOUT";
