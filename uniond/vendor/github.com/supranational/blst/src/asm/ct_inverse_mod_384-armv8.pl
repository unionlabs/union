#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
#
# Both constant-time and fast Euclidean inversion as suggested in
# https://eprint.iacr.org/2020/972. Performance is >12x better [on
# Cortex cores] than modulus-specific FLT addition chain...
#
# void ct_inverse_mod_383(vec768 ret, const vec384 inp, const vec384 mod);
#
$python_ref.=<<'___';
def ct_inverse_mod_383(inp, mod):
    a, u = inp, 1
    b, v = mod, 0

    k = 62
    w = 64
    mask = (1 << w) - 1

    for i in range(0, 766 // k):
        # __ab_approximation_62
        n = max(a.bit_length(), b.bit_length())
        if n < 128:
            a_, b_ = a, b
        else:
            a_ = (a & mask) | ((a >> (n-w)) << w)
            b_ = (b & mask) | ((b >> (n-w)) << w)

        # __inner_loop_62
        f0, g0, f1, g1 = 1, 0, 0, 1
        for j in range(0, k):
            if a_ & 1:
                if a_ < b_:
                    a_, b_, f0, g0, f1, g1 = b_, a_, f1, g1, f0, g0
                a_, f0, g0 = a_-b_, f0-f1, g0-g1
            a_, f1, g1 = a_ >> 1, f1 << 1, g1 << 1

        # __smul_383_n_shift_by_62
        a, b = (a*f0 + b*g0) >> k, (a*f1 + b*g1) >> k
        if a < 0:
            a, f0, g0 = -a, -f0, -g0
        if b < 0:
            b, f1, g1 = -b, -f1, -g1

        # __smul_767x63
        u, v = u*f0 + v*g0, u*f1 + v*g1

    if 766 % k:
        f0, g0, f1, g1 = 1, 0, 0, 1
        for j in range(0, 766 % k):
            if a & 1:
                if a < b:
                    a, b, f0, g0, f1, g1 = b, a, f1, g1, f0, g0
                a, f0, g0 = a-b, f0-f1, g0-g1
            a, f1, g1 = a >> 1, f1 << 1, g1 << 1

        v = u*f1 + v*g1

    if v < 0:
        v += mod << (768 - mod.bit_length())    # left aligned

    return v & (2**768 - 1) # to be reduced % mod
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

my ($out_ptr, $in_ptr, $n_ptr, $nx_ptr) = map("x$_", (0..3));
my @acc=map("x$_",(3..14));
my ($f0, $g0, $f1, $g1, $f_, $g_) = map("x$_",(15..17,19..21));
my $cnt = $n_ptr;
my @t = map("x$_",(22..28,2));
my ($a_lo, $a_hi, $b_lo, $b_hi) = @acc[0,5,6,11];

$frame = 16+2*512;

$code.=<<___;
.text

.globl	ct_inverse_mod_383
.type	ct_inverse_mod_383, %function
.align	5
ct_inverse_mod_383:
	paciasp
	stp	x29, x30, [sp,#-128]!
	add	x29, sp, #0
	stp	x19, x20, [sp,#16]
	stp	x21, x22, [sp,#32]
	stp	x23, x24, [sp,#48]
	stp	x25, x26, [sp,#64]
	stp	x27, x28, [sp,#80]
	sub	sp, sp, #$frame

	ldp	@t[0],   @acc[1], [$in_ptr,#8*0]
	ldp	@acc[2], @acc[3], [$in_ptr,#8*2]
	ldp	@acc[4], @acc[5], [$in_ptr,#8*4]

	add	$in_ptr, sp, #16+511	// find closest 512-byte-aligned spot
	and	$in_ptr, $in_ptr, #-512	// in the frame...
	stp	$out_ptr, $nx_ptr, [sp]

	ldp	@acc[6], @acc[7], [$n_ptr,#8*0]
	ldp	@acc[8], @acc[9], [$n_ptr,#8*2]
	ldp	@acc[10], @acc[11], [$n_ptr,#8*4]

	stp	@t[0],   @acc[1], [$in_ptr,#8*0]	// copy input to |a|
	stp	@acc[2], @acc[3], [$in_ptr,#8*2]
	stp	@acc[4], @acc[5], [$in_ptr,#8*4]
	stp	@acc[6], @acc[7], [$in_ptr,#8*6]	// copy modulus to |b|
	stp	@acc[8], @acc[9], [$in_ptr,#8*8]
	stp	@acc[10], @acc[11], [$in_ptr,#8*10]

	////////////////////////////////////////// first iteration
	mov	$cnt, #62
	bl	.Lab_approximation_62_loaded

	eor	$out_ptr, $in_ptr, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	str	$f0,[$out_ptr,#8*12]		// initialize |u| with |f0|

	mov	$f0, $f1			// |f1|
	mov	$g0, $g1			// |g1|
	add	$out_ptr, $out_ptr, #8*6	// pointer to dst |b|
	bl	__smul_383_n_shift_by_62
	str	$f0, [$out_ptr,#8*12]		// initialize |v| with |f1|

	////////////////////////////////////////// second iteration
	eor	$in_ptr, $in_ptr, #256		// flip-flop src |a|b|u|v|
	mov	$cnt, #62
	bl	__ab_approximation_62

	eor	$out_ptr, $in_ptr, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	$f_, $f0			// corrected |f0|
	mov	$g_, $g0			// corrected |g0|

	mov	$f0, $f1			// |f1|
	mov	$g0, $g1			// |g1|
	add	$out_ptr, $out_ptr, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	ldr	@acc[4], [$in_ptr,#8*12]	// |u|
	ldr	@acc[5], [$in_ptr,#8*18]	// |v|
	mul	@acc[0], $f_, @acc[4]		// |u|*|f0|
	smulh	@acc[1], $f_, @acc[4]
	mul	@acc[2], $g_, @acc[5]		// |v|*|g0|
	smulh	@acc[3], $g_, @acc[5]
	adds	@acc[0], @acc[0], @acc[2]
	adc	@acc[1], @acc[1], @acc[3]
	stp	@acc[0], @acc[1], [$out_ptr,#8*6]
	asr	@acc[2], @acc[1], #63		// sign extenstion
	stp	@acc[2], @acc[2], [$out_ptr,#8*8]
	stp	@acc[2], @acc[2], [$out_ptr,#8*10]

	mul	@acc[0], $f0, @acc[4]		// |u|*|f1|
	smulh	@acc[1], $f0, @acc[4]
	mul	@acc[2], $g0, @acc[5]		// |v|*|g1|
	smulh	@acc[3], $g0, @acc[5]
	adds	@acc[0], @acc[0], @acc[2]
	adc	@acc[1], @acc[1], @acc[3]
	stp	@acc[0], @acc[1], [$out_ptr,#8*12]
	asr	@acc[2], @acc[1], #63		// sign extenstion
	stp	@acc[2], @acc[2], [$out_ptr,#8*14]
	stp	@acc[2], @acc[2], [$out_ptr,#8*16]
___
for($i=2; $i<11; $i++) {
$code.=<<___;
	eor	$in_ptr, $in_ptr, #256		// flip-flop src |a|b|u|v|
	mov	$cnt, #62
	bl	__ab_approximation_62

	eor	$out_ptr, $in_ptr, #256		// pointer to dst |a|b|u|v|
	bl	__smul_383_n_shift_by_62
	mov	$f_, $f0			// corrected |f0|
	mov	$g_, $g0			// corrected |g0|

	mov	$f0, $f1			// |f1|
	mov	$g0, $g1			// |g1|
	add	$out_ptr, $out_ptr, #8*6	// pointer to destination |b|
	bl	__smul_383_n_shift_by_62

	add	$out_ptr, $out_ptr, #8*6	// pointer to destination |u|
	bl	__smul_383x63

	mov	$f_, $f0			// corrected |f1|
	mov	$g_, $g0			// corrected |g1|
	add	$out_ptr, $out_ptr, #8*6	// pointer to destination |v|
	bl	__smul_383x63
___
$code.=<<___	if ($i>5);
	bl	__smul_767x63_tail
___
$code.=<<___	if ($i==5);
	asr	@t[5], @t[5], #63		// sign extension
	stp	@t[5], @t[5], [$out_ptr,#8*6]
	stp	@t[5], @t[5], [$out_ptr,#8*8]
	stp	@t[5], @t[5], [$out_ptr,#8*10]
___
}
$code.=<<___;
	////////////////////////////////////////// iteration before last
	eor	$in_ptr, $in_ptr, #256		// flip-flop src |a|b|u|v|
	mov	$cnt, #62
	//bl	__ab_approximation_62		// |a| and |b| are exact,
	ldp	$a_lo, $a_hi, [$in_ptr,#8*0]	// just load
	ldp	$b_lo, $b_hi, [$in_ptr,#8*6]
	bl	__inner_loop_62

	eor	$out_ptr, $in_ptr, #256		// pointer to dst |a|b|u|v|
	str	$a_lo, [$out_ptr,#8*0]
	str	$b_lo, [$out_ptr,#8*6]

	mov	$f_, $f0			// exact |f0|
	mov	$g_, $g0			// exact |g0|
	mov	$f0, $f1
	mov	$g0, $g1
	add	$out_ptr, $out_ptr, #8*12	// pointer to dst |u|
	bl	__smul_383x63

	mov	$f_, $f0			// exact |f1|
	mov	$g_, $g0			// exact |g1|
	add	$out_ptr, $out_ptr, #8*6	// pointer to dst |v|
	bl	__smul_383x63
	bl	__smul_767x63_tail

	////////////////////////////////////////// last iteration
	eor	$in_ptr, $in_ptr, #256		// flip-flop src |a|b|u|v|
	mov	$cnt, #22			// 766 % 62
	//bl	__ab_approximation_62		// |a| and |b| are exact,
	ldr	$a_lo, [$in_ptr,#8*0]		// just load
	eor	$a_hi, $a_hi, $a_hi
	ldr	$b_lo, [$in_ptr,#8*6]
	eor	$b_hi, $b_hi, $b_hi
	bl	__inner_loop_62

	mov	$f_, $f1
	mov	$g_, $g1
	ldp	$out_ptr, $f0, [sp]		// original out_ptr and n_ptr
	bl	__smul_383x63
	bl	__smul_767x63_tail
	ldr	x30, [x29,#8]

	asr	@t[0], @acc[5], #63		// sign as mask
	ldp	@acc[6], @acc[7], [$f0,#8*0]
	ldp	@acc[8], @acc[9], [$f0,#8*2]
	ldp	@acc[10], @acc[11], [$f0,#8*4]

	and	@acc[6], @acc[6], @t[0]		// add mod<<384 conditionally
	and	@acc[7], @acc[7], @t[0]
	adds	@acc[0], @acc[0], @acc[6]
	and	@acc[8], @acc[8], @t[0]
	adcs	@acc[1], @acc[1], @acc[7]
	and	@acc[9], @acc[9], @t[0]
	adcs	@acc[2], @acc[2], @acc[8]
	and	@acc[10], @acc[10], @t[0]
	adcs	@acc[3], @acc[3], @acc[9]
	and	@acc[11], @acc[11], @t[0]
	stp	@acc[0], @acc[1], [$out_ptr,#8*6]
	adcs	@acc[4], @acc[4], @acc[10]
	stp	@acc[2], @acc[3], [$out_ptr,#8*8]
	adc	@acc[5], @acc[5], @acc[11]
	stp	@acc[4], @acc[5], [$out_ptr,#8*10]

	add	sp, sp, #$frame
	ldp	x19, x20, [x29,#16]
	ldp	x21, x22, [x29,#32]
	ldp	x23, x24, [x29,#48]
	ldp	x25, x26, [x29,#64]
	ldp	x27, x28, [x29,#80]
	ldr	x29, [sp],#128
	autiasp
	ret
.size	ct_inverse_mod_383,.-ct_inverse_mod_383

////////////////////////////////////////////////////////////////////////
// see corresponding commentary in ctx_inverse_mod_384-x86_64...
.type	__smul_383x63, %function
.align	5
__smul_383x63:
___
for($j=0; $j<2; $j++) {
my $f_ = $f_;   $f_ = $g_          if ($j);
my @acc = @acc; @acc = @acc[6..11] if ($j);
my $k = 8*12+8*6*$j;
$code.=<<___;
	ldp	@acc[0], @acc[1], [$in_ptr,#8*0+$k]	// load |u| (or |v|)
	asr	$f1, $f_, #63		// |f_|'s sign as mask (or |g_|'s)
	ldp	@acc[2], @acc[3], [$in_ptr,#8*2+$k]
	eor	$f_, $f_, $f1		// conditionally negate |f_| (or |g_|)
	ldp	@acc[4], @acc[5], [$in_ptr,#8*4+$k]

	eor	@acc[0], @acc[0], $f1	// conditionally negate |u| (or |v|)
	sub	$f_, $f_, $f1
	eor	@acc[1], @acc[1], $f1
	adds	@acc[0], @acc[0], $f1, lsr#63
	eor	@acc[2], @acc[2], $f1
	adcs	@acc[1], @acc[1], xzr
	eor	@acc[3], @acc[3], $f1
	adcs	@acc[2], @acc[2], xzr
	eor	@acc[4], @acc[4], $f1
	adcs	@acc[3], @acc[3], xzr
	 umulh	@t[0], @acc[0], $f_
	eor	@acc[5], @acc[5], $f1
	 umulh	@t[1], @acc[1], $f_
	adcs	@acc[4], @acc[4], xzr
	 umulh	@t[2], @acc[2], $f_
	adcs	@acc[5], @acc[5], xzr
	 umulh	@t[3], @acc[3], $f_
___
$code.=<<___	if ($j);
	adc	$g1, xzr, xzr		// used in __smul_767x63_tail
___
$code.=<<___;
	umulh	@t[4], @acc[4], $f_
	mul	@acc[0], @acc[0], $f_
	mul	@acc[1], @acc[1], $f_
	mul	@acc[2], @acc[2], $f_
	adds	@acc[1], @acc[1], @t[0]
	mul	@acc[3], @acc[3], $f_
	adcs	@acc[2], @acc[2], @t[1]
	mul	@acc[4], @acc[4], $f_
	adcs	@acc[3], @acc[3], @t[2]
	mul	@t[5+$j],@acc[5], $f_
	adcs	@acc[4], @acc[4], @t[3]
	adcs	@t[5+$j],@t[5+$j],@t[4]
___
$code.=<<___	if ($j==0);
	adc	@t[7], xzr, xzr
___
}
$code.=<<___;
	adc	@t[7], @t[7], xzr

	adds	@acc[0], @acc[0], @acc[6]
	adcs	@acc[1], @acc[1], @acc[7]
	adcs	@acc[2], @acc[2], @acc[8]
	adcs	@acc[3], @acc[3], @acc[9]
	stp	@acc[0], @acc[1], [$out_ptr,#8*0]
	adcs	@acc[4], @acc[4], @acc[10]
	stp	@acc[2], @acc[3], [$out_ptr,#8*2]
	adcs	@t[5],   @t[5],   @t[6]
	stp	@acc[4], @t[5],   [$out_ptr,#8*4]
	adc	@t[6],   @t[7],   xzr	// used in __smul_767x63_tail

	ret
.size	__smul_383x63,.-__smul_383x63

.type	__smul_767x63_tail, %function
.align	5
__smul_767x63_tail:
	smulh	@t[5],   @acc[5], $f_
	ldp	@acc[0], @acc[1], [$in_ptr,#8*24]	// load rest of |v|
	umulh	@acc[11],@acc[11], $g_
	ldp	@acc[2], @acc[3], [$in_ptr,#8*26]
	ldp	@acc[4], @acc[5], [$in_ptr,#8*28]

	eor	@acc[0], @acc[0], $f1	// conditionally negate rest of |v|
	eor	@acc[1], @acc[1], $f1
	eor	@acc[2], @acc[2], $f1
	adds	@acc[0], @acc[0], $g1
	eor	@acc[3], @acc[3], $f1
	adcs	@acc[1], @acc[1], xzr
	eor	@acc[4], @acc[4], $f1
	adcs	@acc[2], @acc[2], xzr
	eor	@acc[5], @acc[5], $f1
	adcs	@acc[3], @acc[3], xzr
	 umulh	@t[0], @acc[0], $g_
	adcs	@acc[4], @acc[4], xzr
	 umulh	@t[1], @acc[1], $g_
	adc	@acc[5], @acc[5], xzr

	umulh	@t[2], @acc[2], $g_
	 add	@acc[11], @acc[11], @t[6]
	umulh	@t[3], @acc[3], $g_
	 asr	@t[6], @t[5], #63
	umulh	@t[4], @acc[4], $g_
	mul	@acc[0], @acc[0], $g_
	mul	@acc[1], @acc[1], $g_
	mul	@acc[2], @acc[2], $g_
	adds	@acc[0], @acc[0], @acc[11]
	mul	@acc[3], @acc[3], $g_
	adcs	@acc[1], @acc[1], @t[0]
	mul	@acc[4], @acc[4], $g_
	adcs	@acc[2], @acc[2], @t[1]
	mul	@acc[5], @acc[5], $g_
	adcs	@acc[3], @acc[3], @t[2]
	adcs	@acc[4], @acc[4], @t[3]
	adc	@acc[5], @acc[5], @t[4]

	adds	@acc[0], @acc[0], @t[5]
	adcs	@acc[1], @acc[1], @t[6]
	adcs	@acc[2], @acc[2], @t[6]
	adcs	@acc[3], @acc[3], @t[6]
	stp	@acc[0], @acc[1], [$out_ptr,#8*6]
	adcs	@acc[4], @acc[4], @t[6]
	stp	@acc[2], @acc[3], [$out_ptr,#8*8]
	adc	@acc[5], @acc[5], @t[6]
	stp	@acc[4], @acc[5], [$out_ptr,#8*10]

	ret
.size	__smul_767x63_tail,.-__smul_767x63_tail

.type	__smul_383_n_shift_by_62, %function
.align	5
__smul_383_n_shift_by_62:
___
for($j=0; $j<2; $j++) {
my $f0 = $f0;   $f0 = $g0           if ($j);
my @acc = @acc; @acc = @acc[6..11]  if ($j);
my $k = 8*6*$j;
$code.=<<___;
	ldp	@acc[0], @acc[1], [$in_ptr,#8*0+$k]	// load |a| (or |b|)
	asr	@t[6], $f0, #63		// |f0|'s sign as mask (or |g0|'s)
	ldp	@acc[2], @acc[3], [$in_ptr,#8*2+$k]
	eor	@t[7], $f0, @t[6]	// conditionally negate |f0| (or |g0|)
	ldp	@acc[4], @acc[5], [$in_ptr,#8*4+$k]

	eor	@acc[0], @acc[0], @t[6]	// conditionally negate |a| (or |b|)
	sub	@t[7], @t[7], @t[6]
	eor	@acc[1], @acc[1], @t[6]
	adds	@acc[0], @acc[0], @t[6], lsr#63
	eor	@acc[2], @acc[2], @t[6]
	adcs	@acc[1], @acc[1], xzr
	eor	@acc[3], @acc[3], @t[6]
	adcs	@acc[2], @acc[2], xzr
	eor	@acc[4], @acc[4], @t[6]
	 umulh	@t[0], @acc[0], @t[7]
	adcs	@acc[3], @acc[3], xzr
	 umulh	@t[1], @acc[1], @t[7]
	eor	@acc[5], @acc[5], @t[6]
	 umulh	@t[2], @acc[2], @t[7]
	adcs	@acc[4], @acc[4], xzr
	 umulh	@t[3], @acc[3], @t[7]
	adc	@acc[5], @acc[5], xzr

	umulh	@t[4], @acc[4], @t[7]
	smulh	@t[5+$j], @acc[5], @t[7]
	mul	@acc[0], @acc[0], @t[7]
	mul	@acc[1], @acc[1], @t[7]
	mul	@acc[2], @acc[2], @t[7]
	adds	@acc[1], @acc[1], @t[0]
	mul	@acc[3], @acc[3], @t[7]
	adcs	@acc[2], @acc[2], @t[1]
	mul	@acc[4], @acc[4], @t[7]
	adcs	@acc[3], @acc[3], @t[2]
	mul	@acc[5], @acc[5], @t[7]
	adcs	@acc[4], @acc[4], @t[3]
	adcs	@acc[5], @acc[5] ,@t[4]
	adc	@t[5+$j], @t[5+$j], xzr
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

	extr	@acc[0], @acc[1], @acc[0], #62
	extr	@acc[1], @acc[2], @acc[1], #62
	extr	@acc[2], @acc[3], @acc[2], #62
	asr	@t[6], @acc[6], #63
	extr	@acc[3], @acc[4], @acc[3], #62
	extr	@acc[4], @acc[5], @acc[4], #62
	extr	@acc[5], @acc[6], @acc[5], #62

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

	eor	$f0, $f0, @t[6]
	eor	$g0, $g0, @t[6]
	sub	$f0, $f0, @t[6]
	sub	$g0, $g0, @t[6]

	ret
.size	__smul_383_n_shift_by_62,.-__smul_383_n_shift_by_62
___

{
my @a = @acc[0..5];
my @b = @acc[6..11];

$code.=<<___;
.type	__ab_approximation_62, %function
.align	4
__ab_approximation_62:
	ldp	@a[4], @a[5], [$in_ptr,#8*4]
	ldp	@b[4], @b[5], [$in_ptr,#8*10]
	ldp	@a[2], @a[3], [$in_ptr,#8*2]
	ldp	@b[2], @b[3], [$in_ptr,#8*8]

.Lab_approximation_62_loaded:
	orr	@t[0], @a[5], @b[5]	// check top-most limbs, ...
	cmp	@t[0], #0
	csel	@a[5], @a[5], @a[4], ne
	csel	@b[5], @b[5], @b[4], ne
	csel	@a[4], @a[4], @a[3], ne
	orr	@t[0], @a[5], @b[5]	// ... ones before top-most, ...
	csel	@b[4], @b[4], @b[3], ne

	ldp	@a[0], @a[1], [$in_ptr,#8*0]
	ldp	@b[0], @b[1], [$in_ptr,#8*6]

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
	orr	@t[0], @a[5], @b[5]
	csel	@b[4], @b[4], @b[1], ne

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
	orr	@a[5], @a[5], @a[4]
	orr	@b[5], @b[5], @b[4]

	b	__inner_loop_62
	ret
.size	__ab_approximation_62,.-__ab_approximation_62
___
}
$code.=<<___;
.type	__inner_loop_62, %function
.align	4
__inner_loop_62:
	mov	$f0, #1		// |f0|=1
	mov	$g0, #0		// |g0|=0
	mov	$f1, #0		// |f1|=0
	mov	$g1, #1		// |g1|=1

.Loop_62:
	sbfx	@t[6], $a_lo, #0, #1	// if |a_| is odd, then we'll be subtracting
	sub	$cnt, $cnt, #1
	subs	@t[2], $b_lo, $a_lo	// |b_|-|a_|
	and	@t[0], $b_lo, @t[6]
	sbc	@t[3], $b_hi, $a_hi
	and	@t[1], $b_hi, @t[6]
	subs	@t[4], $a_lo, @t[0]	// |a_|-|b_| (or |a_|-0 if |a_| was even)
	mov	@t[0], $f0
	sbcs	@t[5], $a_hi, @t[1]
	mov	@t[1], $g0
	csel	$b_lo, $b_lo, $a_lo, hs	// |b_| = |a_|
	csel	$b_hi, $b_hi, $a_hi, hs
	csel	$a_lo, @t[4], @t[2], hs	// borrow means |a_|<|b_|, replace with |b_|-|a_|
	csel	$a_hi, @t[5], @t[3], hs
	csel	$f0, $f0, $f1,       hs	// exchange |f0| and |f1|
	csel	$f1, $f1, @t[0],     hs
	csel	$g0, $g0, $g1,       hs	// exchange |g0| and |g1|
	csel	$g1, $g1, @t[1],     hs
	extr	$a_lo, $a_hi, $a_lo, #1
	lsr	$a_hi, $a_hi, #1
	and	@t[0], $f1, @t[6]
	and	@t[1], $g1, @t[6]
	add	$f1, $f1, $f1		// |f1|<<=1
	add	$g1, $g1, $g1		// |g1|<<=1
	sub	$f0, $f0, @t[0]		// |f0|-=|f1| (or |f0-=0| if |a_| was even)
	sub	$g0, $g0, @t[1]		// |g0|-=|g1| (or |g0-=0| ...)
	cbnz	$cnt, .Loop_62

	ret
.size	__inner_loop_62,.-__inner_loop_62
___

print $code;
close STDOUT;
