#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
#
# Both constant-time and fast quadratic residue test as suggested in
# https://eprint.iacr.org/2020/972. Performance is >12x better [on
# Cortex cores] than modulus-specific Legendre symbol addition chain...
#
# bool ct_is_square_mod_384(const vec384 inp, const vec384 mod);
#
$python_ref.=<<'___';
def ct_is_square_mod_384(inp, mod):
    a = inp
    b = mod
    L = 0   # only least significant bit, adding 1 makes up for sign change

    k = 30
    w = 32
    mask = (1 << w) - 1

    for i in range(0, 768 // k - 1):
        # __ab_approximation_30
        n = max(a.bit_length(), b.bit_length())
        if n < 64:
            a_, b_ = a, b
        else:
            a_ = (a & mask) | ((a >> (n-w)) << w)
            b_ = (b & mask) | ((b >> (n-w)) << w)

        # __inner_loop_30
        f0, g0, f1, g1 = 1, 0, 0, 1
        for j in range(0, k):
            if a_ & 1:
                if a_ < b_:
                    a_, b_, f0, g0, f1, g1 = b_, a_, f1, g1, f0, g0
                    L += (a_ & b_) >> 1 # |a| and |b| are both odd, second bits
                                        # tell the whole story
                a_, f0, g0 = a_-b_, f0-f1, g0-g1
            a_, f1, g1 = a_ >> 1, f1 << 1, g1 << 1
            L += (b_ + 2) >> 2          # if |b|%8 is 3 or 5 [out of 1,3,5,7]

        # __smulq_384_n_shift_by_30
        a, b = (a*f0 + b*g0) >> k, (a*f1 + b*g1) >> k
        if b < 0:
            b = -b
        if a < 0:
            a = -a
            L += (b % 4) >> 1           # |b| is always odd, the second bit
                                        # tells the whole story

    if True:
        for j in range(0, 768 % k + k):
            if a & 1:
                if a < b:
                    a, b = b, a
                    L += (a & b) >> 1   # |a| and |b| are both odd, second bits
                                        # tell the whole story
                a = a-b
            a = a >> 1
            L += (b + 2) >> 2           # if |b|%8 is 3 or 5 [out of 1,3,5,7]

    return (L & 1) ^ 1
___

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

my ($in_ptr, $out_ptr, $L) = map("x$_", (0..2));
my @acc=map("x$_",(3..14));
my ($cnt, $f0, $g0, $f1, $g1) = map("x$_",(15..17,19..20));
my @t = map("x$_",(21..28));
my ($a_, $b_) = @acc[5,11];

$frame = 2*256;

$code.=<<___;
.text

.globl	ct_is_square_mod_384
.hidden	ct_is_square_mod_384
.type	ct_is_square_mod_384, %function
.align	5
ct_is_square_mod_384:
	paciasp
	stp	c29, c30, [csp,#-16*__SIZEOF_POINTER__]!
	add	c29, csp, #0
	stp	c19, c20, [csp,#2*__SIZEOF_POINTER__]
	stp	c21, c22, [csp,#4*__SIZEOF_POINTER__]
	stp	c23, c24, [csp,#6*__SIZEOF_POINTER__]
	stp	c25, c26, [csp,#8*__SIZEOF_POINTER__]
	stp	c27, c28, [csp,#10*__SIZEOF_POINTER__]
	sub	csp, csp, #$frame

	ldp	@acc[0], @acc[1], [x0,#8*0]		// load input
	ldp	@acc[2], @acc[3], [x0,#8*2]
	ldp	@acc[4], @acc[5], [x0,#8*4]

	add	$in_ptr, sp, #255	// find closest 256-byte-aligned spot
	and	$in_ptr, $in_ptr, #-256	// in the frame...
#ifdef	__CHERI_PURE_CAPABILITY__
	scvalue $in_ptr, csp, $in_ptr
#endif

	ldp	@acc[6], @acc[7], [x1,#8*0]		// load modulus
	ldp	@acc[8], @acc[9], [x1,#8*2]
	ldp	@acc[10], @acc[11], [x1,#8*4]

	stp	@acc[0], @acc[1], [$in_ptr,#8*6]	// copy input to |a|
	stp	@acc[2], @acc[3], [$in_ptr,#8*8]
	stp	@acc[4], @acc[5], [$in_ptr,#8*10]
	stp	@acc[6], @acc[7], [$in_ptr,#8*0]	// copy modulus to |b|
	stp	@acc[8], @acc[9], [$in_ptr,#8*2]
	stp	@acc[10], @acc[11], [$in_ptr,#8*4]

	eor	$L, $L, $L			// init the Legendre symbol
	mov	$cnt, #24			// 24 is 768/30-1
	b	.Loop_is_square

.align	4
.Loop_is_square:
	bl	__ab_approximation_30
	sub	$cnt, $cnt, #1

	eor	$out_ptr, $in_ptr, #128		// pointer to dst |b|
#ifdef	__CHERI_PURE_CAPABILITY__
	scvalue $out_ptr, csp, $out_ptr
#endif
	bl	__smul_384_n_shift_by_30

	mov	$f1, $f0			// |f0|
	mov	$g1, $g0			// |g0|
	cadd	$out_ptr, $out_ptr, #8*6	// pointer to dst |a|
	bl	__smul_384_n_shift_by_30

	ldp	@acc[6], @acc[7], [$out_ptr,#-8*6]
	eor	$in_ptr, $in_ptr, #128		// flip-flop src |a|b|
#ifdef	__CHERI_PURE_CAPABILITY__
	scvalue $in_ptr, csp, $in_ptr
#endif
	and	@t[6], @t[6], @acc[6]		// if |a| was negative,
	add	$L, $L, @t[6], lsr#1		// adjust |L|

	cbnz	$cnt, .Loop_is_square

	////////////////////////////////////////// last iteration
	//bl	__ab_approximation_30		// |a| and |b| are exact,
	//ldr	$a_, [$in_ptr,#8*6]		// and loaded
	//ldr	$b_, [$in_ptr,#8*0]
	mov	$cnt, #48			// 48 is 768%30 + 30
	bl	__inner_loop_48
	ldr	c30, [c29,#__SIZEOF_POINTER__]

	and	x0, $L, #1
	eor	x0, x0, #1

	add	csp, csp, #$frame
	ldp	c19, c20, [c29,#2*__SIZEOF_POINTER__]
	ldp	c21, c22, [c29,#4*__SIZEOF_POINTER__]
	ldp	c23, c24, [c29,#6*__SIZEOF_POINTER__]
	ldp	c25, c26, [c29,#8*__SIZEOF_POINTER__]
	ldp	c27, c28, [c29,#10*__SIZEOF_POINTER__]
	ldr	c29, [csp],#16*__SIZEOF_POINTER__
	autiasp
	ret
.size	ct_is_square_mod_384,.-ct_is_square_mod_384

.type	__smul_384_n_shift_by_30, %function
.align	5
__smul_384_n_shift_by_30:
___
for($j=0; $j<2; $j++) {
my $fx = $g1;   $fx = $f1           if ($j);
my @acc = @acc; @acc = @acc[6..11]  if ($j);
my $k = 8*6*$j;
$code.=<<___;
	ldp	@acc[0], @acc[1], [$in_ptr,#8*0+$k]	// load |b| (or |a|)
	asr	@t[6], $fx, #63		// |g1|'s sign as mask (or |f1|'s)
	ldp	@acc[2], @acc[3], [$in_ptr,#8*2+$k]
	eor	$fx, $fx, @t[6]		// conditionally negate |g1| (or |f1|)
	ldp	@acc[4], @acc[5], [$in_ptr,#8*4+$k]

	eor	@acc[0], @acc[0], @t[6]	// conditionally negate |b| (or |a|)
	sub	$fx, $fx, @t[6]
	eor	@acc[1], @acc[1], @t[6]
	adds	@acc[0], @acc[0], @t[6], lsr#63
	eor	@acc[2], @acc[2], @t[6]
	adcs	@acc[1], @acc[1], xzr
	eor	@acc[3], @acc[3], @t[6]
	adcs	@acc[2], @acc[2], xzr
	eor	@acc[4], @acc[4], @t[6]
	 umulh	@t[0], @acc[0], $fx
	adcs	@acc[3], @acc[3], xzr
	 umulh	@t[1], @acc[1], $fx
	eor	@acc[5], @acc[5], @t[6]
	 umulh	@t[2], @acc[2], $fx
	adcs	@acc[4], @acc[4], xzr
	 umulh	@t[3], @acc[3], $fx
	adc	@acc[5], @acc[5], xzr

	umulh	@t[4], @acc[4], $fx
	and	@t[7], $fx, @t[6]
	umulh	@t[5+$j], @acc[5], $fx
	neg	@t[7], @t[7]
	mul	@acc[0], @acc[0], $fx
	mul	@acc[1], @acc[1], $fx
	mul	@acc[2], @acc[2], $fx
	adds	@acc[1], @acc[1], @t[0]
	mul	@acc[3], @acc[3], $fx
	adcs	@acc[2], @acc[2], @t[1]
	mul	@acc[4], @acc[4], $fx
	adcs	@acc[3], @acc[3], @t[2]
	mul	@acc[5], @acc[5], $fx
	adcs	@acc[4], @acc[4], @t[3]
	adcs	@acc[5], @acc[5] ,@t[4]
	adc	@t[5+$j], @t[5+$j], @t[7]
___
}
$code.=<<___;
	adds	@acc[0], @acc[0], @acc[6]
	adcs	@acc[1], @acc[1], @acc[7]
	adcs	@acc[2], @acc[2], @acc[8]
	adcs	@acc[3], @acc[3], @acc[9]
	adcs	@acc[4], @acc[4], @acc[10]
	adcs	@acc[5], @acc[5], @acc[11]
	adc	@acc[6], @t[5],   @t[6]

	extr	@acc[0], @acc[1], @acc[0], #30
	extr	@acc[1], @acc[2], @acc[1], #30
	extr	@acc[2], @acc[3], @acc[2], #30
	asr	@t[6], @acc[6], #63
	extr	@acc[3], @acc[4], @acc[3], #30
	extr	@acc[4], @acc[5], @acc[4], #30
	extr	@acc[5], @acc[6], @acc[5], #30

	eor	@acc[0], @acc[0], @t[6]
	eor	@acc[1], @acc[1], @t[6]
	adds	@acc[0], @acc[0], @t[6], lsr#63
	eor	@acc[2], @acc[2], @t[6]
	adcs	@acc[1], @acc[1], xzr
	eor	@acc[3], @acc[3], @t[6]
	adcs	@acc[2], @acc[2], xzr
	eor	@acc[4], @acc[4], @t[6]
	adcs	@acc[3], @acc[3], xzr
	eor	@acc[5], @acc[5], @t[6]
	stp	@acc[0], @acc[1], [$out_ptr,#8*0]
	adcs	@acc[4], @acc[4], xzr
	stp	@acc[2], @acc[3], [$out_ptr,#8*2]
	adc	@acc[5], @acc[5], xzr
	stp	@acc[4], @acc[5], [$out_ptr,#8*4]

	ret
.size	__smul_384_n_shift_by_30,.-__smul_384_n_shift_by_30
___

{
my @a = @acc[0..5];
my @b = @acc[6..11];
my ($fg0, $fg1, $bias, $cnt) = ($g0, $g1, @t[6], @t[7]);

$code.=<<___;
.type	__ab_approximation_30, %function
.align	4
__ab_approximation_30:
	ldp	@b[4], @b[5], [$in_ptr,#8*4]	// |a| is still in registers
	ldp	@b[2], @b[3], [$in_ptr,#8*2]

	orr	@t[0], @a[5], @b[5]	// check top-most limbs, ...
	cmp	@t[0], #0
	csel	@a[5], @a[5], @a[4], ne
	csel	@b[5], @b[5], @b[4], ne
	csel	@a[4], @a[4], @a[3], ne
	orr	@t[0], @a[5], @b[5]	// ... ones before top-most, ...
	csel	@b[4], @b[4], @b[3], ne

	cmp	@t[0], #0
	csel	@a[5], @a[5], @a[4], ne
	csel	@b[5], @b[5], @b[4], ne
	csel	@a[4], @a[4], @a[2], ne
	orr	@t[0], @a[5], @b[5]	// ... and ones before that ...
	csel	@b[4], @b[4], @b[2], ne

	cmp	@t[0], #0
	csel	@a[5], @a[5], @a[4], ne
	csel	@b[5], @b[5], @b[4], ne
	csel	@a[4], @a[4], @a[1], ne
	orr	@t[0], @a[5], @b[5]	// and one more, ...
	csel	@b[4], @b[4], @b[1], ne

	cmp	@t[0], #0
	csel	@a[5], @a[5], @a[4], ne
	csel	@b[5], @b[5], @b[4], ne
	csel	@a[4], @a[4], @a[0], ne
	orr	@t[0], @a[5], @b[5]
	csel	@b[4], @b[4], @b[0], ne

	clz	@t[0], @t[0]
	cmp	@t[0], #64
	csel	@t[0], @t[0], xzr, ne
	csel	@a[5], @a[5], @a[4], ne
	csel	@b[5], @b[5], @b[4], ne
	neg	@t[1], @t[0]

	lslv	@a[5], @a[5], @t[0]	// align high limbs to the left
	lslv	@b[5], @b[5], @t[0]
	lsrv	@a[4], @a[4], @t[1]
	lsrv	@b[4], @b[4], @t[1]
	and	@a[4], @a[4], @t[1], asr#6
	and	@b[4], @b[4], @t[1], asr#6
	orr	$a_, @a[5], @a[4]
	orr	$b_, @b[5], @b[4]

	bfxil	$a_, @a[0], #0, #32
	bfxil	$b_, @b[0], #0, #32

	b	__inner_loop_30
	ret
.size	__ab_approximation_30,.-__ab_approximation_30

.type	__inner_loop_30, %function
.align	4
__inner_loop_30:
	mov	$cnt, #30
	mov	$fg0, #0x7FFFFFFF80000000	// |f0|=1, |g0|=0
	mov	$fg1, #0x800000007FFFFFFF	// |f1|=0, |g1|=1
	mov	$bias,#0x7FFFFFFF7FFFFFFF

.Loop_30:
	sbfx	@t[3], $a_, #0, #1	// if |a_| is odd, then we'll be subtracting
	 and	@t[4], $a_, $b_
	sub	$cnt, $cnt, #1
	and	@t[0], $b_, @t[3]

	sub	@t[1], $b_, $a_		// |b_|-|a_|
	subs	@t[2], $a_, @t[0]	// |a_|-|b_| (or |a_|-0 if |a_| was even)
	 add	@t[4], $L, @t[4], lsr#1	// L + (a_ & b_) >> 1
	mov	@t[0], $fg1
	csel	$b_, $b_, $a_, hs	// |b_| = |a_|
	csel	$a_, @t[2], @t[1], hs	// borrow means |a_|<|b_|, replace with |b_|-|a_|
	csel	$fg1, $fg1, $fg0,  hs	// exchange |fg0| and |fg1|
	csel	$fg0, $fg0, @t[0], hs
	 csel	$L,   $L,   @t[4], hs
	lsr	$a_, $a_, #1
	and	@t[0], $fg1, @t[3]
	and	@t[1], $bias, @t[3]
	 add	$t[2], $b_, #2
	sub	$fg0, $fg0, @t[0]	// |f0|-=|f1| (or |f0-=0| if |a_| was even)
	add	$fg1, $fg1, $fg1	// |f1|<<=1
	 add	$L, $L, $t[2], lsr#2	// "negate" |L| if |b|%8 is 3 or 5
	add	$fg0, $fg0, @t[1]
	sub	$fg1, $fg1, $bias

	cbnz	$cnt, .Loop_30

	mov	$bias, #0x7FFFFFFF
	ubfx	$f0, $fg0, #0, #32
	ubfx	$g0, $fg0, #32, #32
	ubfx	$f1, $fg1, #0, #32
	ubfx	$g1, $fg1, #32, #32
	sub	$f0, $f0, $bias		// remove the bias
	sub	$g0, $g0, $bias
	sub	$f1, $f1, $bias
	sub	$g1, $g1, $bias

	ret
.size	__inner_loop_30,.-__inner_loop_30
___
}

{
my ($a_, $b_) = (@acc[0], @acc[6]);
$code.=<<___;
.type	__inner_loop_48, %function
.align	4
__inner_loop_48:
.Loop_48:
	sbfx	@t[3], $a_, #0, #1	// if |a_| is odd, then we'll be subtracting
	 and	@t[4], $a_, $b_
	sub	$cnt, $cnt, #1
	and	@t[0], $b_, @t[3]
	sub	@t[1], $b_, $a_		// |b_|-|a_|
	subs	@t[2], $a_, @t[0]	// |a_|-|b_| (or |a_|-0 if |a_| was even)
	 add	@t[4], $L, @t[4], lsr#1
	csel	$b_, $b_, $a_, hs	// |b_| = |a_|
	csel	$a_, @t[2], @t[1], hs	// borrow means |a_|<|b_|, replace with |b_|-|a_|
	 csel	$L,   $L,   @t[4], hs
	 add	$t[2], $b_, #2
	lsr	$a_, $a_, #1
	 add	$L, $L, $t[2], lsr#2	// "negate" |L| if |b|%8 is 3 or 5

	cbnz	$cnt, .Loop_48

	ret
.size	__inner_loop_48,.-__inner_loop_48
___
}

print $code;
close STDOUT;
