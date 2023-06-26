#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
#
# Both constant-time and fast Euclidean inversion as suggested in
# https://eprint.iacr.org/2020/972. ~4.600 cycles on Apple M1, ~8.900 -
# on Cortex-A57.
#
# void ct_inverse_mod_256(vec512 ret, const vec256 inp, const vec256 mod,
#                                                       const vec256 modx);
#
$python_ref.=<<'___';
def ct_inverse_mod_256(inp, mod):
    a, u = inp, 1
    b, v = mod, 0

    k = 31
    mask = (1 << k) - 1

    for i in range(0, 512 // k - 1):
        # __ab_approximation_31
        n = max(a.bit_length(), b.bit_length())
        if n < 64:
            a_, b_ = a, b
        else:
            a_ = (a & mask) | ((a >> (n-k-2)) << k)
            b_ = (b & mask) | ((b >> (n-k-2)) << k)

        # __inner_loop_31
        f0, g0, f1, g1 = 1, 0, 0, 1
        for j in range(0, k):
            if a_ & 1:
                if a_ < b_:
                    a_, b_, f0, g0, f1, g1 = b_, a_, f1, g1, f0, g0
                a_, f0, g0 = a_-b_, f0-f1, g0-g1
            a_, f1, g1 = a_ >> 1, f1 << 1, g1 << 1

        # __smul_256_n_shift_by_31
        a, b = (a*f0 + b*g0) >> k, (a*f1 + b*g1) >> k
        if a < 0:
            a, f0, g0 = -a, -f0, -g0
        if b < 0:
            b, f1, g1 = -b, -f1, -g1

        # __smul_512x63
        u, v = u*f0 + v*g0, u*f1 + v*g1

    if 512 % k + k:
        f0, g0, f1, g1 = 1, 0, 0, 1
        for j in range(0, 512 % k + k):
            if a & 1:
                if a < b:
                    a, b, f0, g0, f1, g1 = b, a, f1, g1, f0, g0
                a, f0, g0 = a-b, f0-f1, g0-g1
            a, f1, g1 = a >> 1, f1 << 1, g1 << 1

        v = u*f1 + v*g1

    mod <<= 512 - mod.bit_length()  # align to the left
    if v < 0:
        v += mod
    if v < 0:
        v += mod
    elif v == 1<<512
        v -= mod

    return v & (2**512 - 1) # to be reduced % mod
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
my @acc=map("x$_",(4..11));
my ($f0, $g0, $f1, $g1, $f_, $g_) = map("x$_",(12..17));
my $cnt = $n_ptr;
my @t = map("x$_",(19..26));
my ($a_lo, $b_lo) = @acc[3,7];

$frame = 16+2*512;

$code.=<<___;
.text

.globl	ct_inverse_mod_256
.type	ct_inverse_mod_256, %function
.align	5
ct_inverse_mod_256:
	paciasp
	stp	x29, x30, [sp,#-80]!
	add	x29, sp, #0
	stp	x19, x20, [sp,#16]
	stp	x21, x22, [sp,#32]
	stp	x23, x24, [sp,#48]
	stp	x25, x26, [sp,#64]
	sub	sp, sp, #$frame

	ldp	@acc[0], @acc[1], [$in_ptr,#8*0]
	ldp	@acc[2], @acc[3], [$in_ptr,#8*2]

	add	$in_ptr, sp, #16+511	// find closest 512-byte-aligned spot
	and	$in_ptr, $in_ptr, #-512	// in the frame...
	str	$out_ptr, [sp]

	ldp	@acc[4], @acc[5], [$n_ptr,#8*0]
	ldp	@acc[6], @acc[7], [$n_ptr,#8*2]

	stp	@acc[0], @acc[1], [$in_ptr,#8*0]	// copy input to |a|
	stp	@acc[2], @acc[3], [$in_ptr,#8*2]
	stp	@acc[4], @acc[5], [$in_ptr,#8*4]	// copy modulus to |b|
	stp	@acc[6], @acc[7], [$in_ptr,#8*6]

	////////////////////////////////////////// first iteration
	bl	.Lab_approximation_31_256_loaded

	eor	$out_ptr, $in_ptr, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	str	$f0,[$out_ptr,#8*8]		// initialize |u| with |f0|

	mov	$f0, $f1			// |f1|
	mov	$g0, $g1			// |g1|
	add	$out_ptr, $out_ptr, #8*4	// pointer to dst |b|
	bl	__smul_256_n_shift_by_31
	str	$f0, [$out_ptr,#8*9]		// initialize |v| with |f1|

	////////////////////////////////////////// second iteration
	eor	$in_ptr, $in_ptr, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	$out_ptr, $in_ptr, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	$f_, $f0			// corrected |f0|
	mov	$g_, $g0			// corrected |g0|

	mov	$f0, $f1			// |f1|
	mov	$g0, $g1			// |g1|
	add	$out_ptr, $out_ptr, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	ldr	@acc[4], [$in_ptr,#8*8]		// |u|
	ldr	@acc[5], [$in_ptr,#8*13]	// |v|
	madd	@acc[0], $f_, @acc[4], xzr	// |u|*|f0|
	madd	@acc[0], $g_, @acc[5], @acc[0]	// |v|*|g0|
	str	@acc[0], [$out_ptr,#8*4]
	asr	@acc[1], @acc[0], #63		// sign extenstion
	stp	@acc[1], @acc[1], [$out_ptr,#8*5]
	stp	@acc[1], @acc[1], [$out_ptr,#8*7]

	madd	@acc[0], $f0, @acc[4], xzr	// |u|*|f1|
	madd	@acc[0], $g0, @acc[5], @acc[0]	// |v|*|g1|
	str	@acc[0], [$out_ptr,#8*9]
	asr	@acc[1], @acc[0], #63		// sign extenstion
	stp	@acc[1], @acc[1], [$out_ptr,#8*10]
	stp	@acc[1], @acc[1], [$out_ptr,#8*12]
___
for($i=2; $i<15; $i++) {
$code.=<<___;
	eor	$in_ptr, $in_ptr, #256		// flip-flop src |a|b|u|v|
	bl	__ab_approximation_31_256

	eor	$out_ptr, $in_ptr, #256		// pointer to dst |a|b|u|v|
	bl	__smul_256_n_shift_by_31
	mov	$f_, $f0			// corrected |f0|
	mov	$g_, $g0			// corrected |g0|

	mov	$f0, $f1			// |f1|
	mov	$g0, $g1			// |g1|
	add	$out_ptr, $out_ptr, #8*4	// pointer to destination |b|
	bl	__smul_256_n_shift_by_31

	add	$out_ptr, $out_ptr, #8*4	// pointer to destination |u|
	bl	__smul_256x63
	adc	@t[3], @t[3], @t[4]
	str	@t[3], [$out_ptr,#8*4]

	mov	$f_, $f0			// corrected |f1|
	mov	$g_, $g0			// corrected |g1|
	add	$out_ptr, $out_ptr, #8*5	// pointer to destination |v|
	bl	__smul_256x63
___
$code.=<<___	if ($i>7);
	bl	__smul_512x63_tail
___
$code.=<<___	if ($i<=7);
	adc	@t[3], @t[3], @t[4]
	stp	@t[3], @t[3], [$out_ptr,#8*4]
	stp	@t[3], @t[3], [$out_ptr,#8*6]
___
}
$code.=<<___;
	////////////////////////////////////////// two[!] last iterations
	eor	$in_ptr, $in_ptr, #256		// flip-flop src |a|b|u|v|
	mov	$cnt, #47			// 31 + 512 % 31
	//bl	__ab_approximation_62_256	// |a| and |b| are exact,
	ldr	$a_lo, [$in_ptr,#8*0]		// just load
	ldr	$b_lo, [$in_ptr,#8*4]
	bl	__inner_loop_62_256

	mov	$f_, $f1
	mov	$g_, $g1
	ldr	$out_ptr, [sp]			// original out_ptr
	bl	__smul_256x63
	bl	__smul_512x63_tail
	ldr	x30, [x29,#8]

	smulh	@t[1], @acc[3], $g_		// figure out top-most limb
	ldp	@acc[4], @acc[5], [$nx_ptr,#8*0]
	adc	@t[4], @t[4], @t[6]
	ldp	@acc[6], @acc[7], [$nx_ptr,#8*2]

	add	@t[1], @t[1], @t[4]		// @t[1] is 1, 0 or -1
	asr	@t[0], @t[1], #63		// sign as mask

	and	@t[4],   @acc[4], @t[0]		// add mod<<256 conditionally
	and	@t[5],   @acc[5], @t[0]
	adds	@acc[0], @acc[0], @t[4]
	and	@t[6],   @acc[6], @t[0]
	adcs	@acc[1], @acc[1], @t[5]
	and	@t[7],   @acc[7], @t[0]
	adcs	@acc[2], @acc[2], @t[6]
	adcs	@acc[3], @t[3],   @t[7]
	adc	@t[1], @t[1], xzr		// @t[1] is 1, 0 or -1

	neg	@t[0], @t[1]
	orr	@t[1], @t[1], @t[0]		// excess bit or sign as mask
	asr	@t[0], @t[0], #63		// excess bit as mask

	and	@acc[4], @acc[4], @t[1]		// mask |mod|
	and	@acc[5], @acc[5], @t[1]
	and	@acc[6], @acc[6], @t[1]
	and	@acc[7], @acc[7], @t[1]

	eor	@acc[4], @acc[4], @t[0]		// conditionally negate |mod|
	eor	@acc[5], @acc[5], @t[0]
	adds	@acc[4], @acc[4], @t[0], lsr#63
	eor	@acc[6], @acc[6], @t[0]
	adcs	@acc[5], @acc[5], xzr
	eor	@acc[7], @acc[7], @t[0]
	adcs	@acc[6], @acc[6], xzr
	adc	@acc[7], @acc[7], xzr

	adds	@acc[0], @acc[0], @acc[4]	// final adjustment for |mod|<<256
	adcs	@acc[1], @acc[1], @acc[5]
	adcs	@acc[2], @acc[2], @acc[6]
	stp	@acc[0], @acc[1], [$out_ptr,#8*4]
	adc	@acc[3], @acc[3], @acc[7]
	stp	@acc[2], @acc[3], [$out_ptr,#8*6]

	add	sp, sp, #$frame
	ldp	x19, x20, [x29,#16]
	ldp	x21, x22, [x29,#32]
	ldp	x23, x24, [x29,#48]
	ldp	x25, x26, [x29,#64]
	ldr	x29, [sp],#80
	autiasp
	ret
.size	ct_inverse_mod_256,.-ct_inverse_mod_256

////////////////////////////////////////////////////////////////////////
.type	__smul_256x63, %function
.align	5
__smul_256x63:
___
for($j=0; $j<2; $j++) {
my $f_ = $f_;   $f_ = $g_          if ($j);
my @acc = @acc; @acc = @acc[4..7]  if ($j);
my $k = 8*8+8*5*$j;
$code.=<<___;
	ldp	@acc[0], @acc[1], [$in_ptr,#8*0+$k]	// load |u| (or |v|)
	asr	$f1, $f_, #63		// |f_|'s sign as mask (or |g_|'s)
	ldp	@acc[2], @acc[3], [$in_ptr,#8*2+$k]
	eor	$f_, $f_, $f1		// conditionally negate |f_| (or |g_|)
	ldr	@t[3+$j], [$in_ptr,#8*4+$k]

	eor	@acc[0], @acc[0], $f1	// conditionally negate |u| (or |v|)
	sub	$f_, $f_, $f1
	eor	@acc[1], @acc[1], $f1
	adds	@acc[0], @acc[0], $f1, lsr#63
	eor	@acc[2], @acc[2], $f1
	adcs	@acc[1], @acc[1], xzr
	eor	@acc[3], @acc[3], $f1
	adcs	@acc[2], @acc[2], xzr
	eor	@t[3+$j], @t[3+$j], $f1
	 umulh	@t[0], @acc[0], $f_
	adcs	@acc[3], @acc[3], xzr
	 umulh	@t[1], @acc[1], $f_
	adcs	@t[3+$j], @t[3+$j], xzr
	 umulh	@t[2], @acc[2], $f_
___
$code.=<<___	if ($j!=0);
	adc	$g1, xzr, xzr		// used in __smul_512x63_tail
___
$code.=<<___;
	mul	@acc[0], @acc[0], $f_
	 cmp	$f_, #0
	mul	@acc[1], @acc[1], $f_
	 csel	@t[3+$j], @t[3+$j], xzr, ne
	mul	@acc[2], @acc[2], $f_
	adds	@acc[1], @acc[1], @t[0]
	mul	@t[5+$j], @acc[3], $f_
	adcs	@acc[2], @acc[2], @t[1]
	adcs	@t[5+$j], @t[5+$j], @t[2]
___
$code.=<<___	if ($j==0);
	adc	@t[7], xzr, xzr
___
}
$code.=<<___;
	adc	@t[7], @t[7], xzr

	adds	@acc[0], @acc[0], @acc[4]
	adcs	@acc[1], @acc[1], @acc[5]
	adcs	@acc[2], @acc[2], @acc[6]
	stp	@acc[0], @acc[1], [$out_ptr,#8*0]
	adcs	@t[5],   @t[5],   @t[6]
	stp	@acc[2], @t[5], [$out_ptr,#8*2]

	ret
.size	__smul_256x63,.-__smul_256x63

.type	__smul_512x63_tail, %function
.align	5
__smul_512x63_tail:
	umulh	@t[5], @acc[3], $f_
	ldp	@acc[1], @acc[2], [$in_ptr,#8*18]	// load rest of |v|
	adc	@t[7], @t[7], xzr
	ldr	@acc[3], [$in_ptr,#8*20]
	and	@t[3], @t[3], $f_

	umulh	@acc[7], @acc[7], $g_	// resume |v|*|g1| chain

	sub	@t[5], @t[5], @t[3]	// tie up |u|*|f1| chain
	asr	@t[6], @t[5], #63

	eor	@acc[1], @acc[1], $f1	// conditionally negate rest of |v|
	eor	@acc[2], @acc[2], $f1
	adds	@acc[1], @acc[1], $g1
	eor	@acc[3], @acc[3], $f1
	adcs	@acc[2], @acc[2], xzr
	 umulh	@t[0], @t[4],   $g_
	adc	@acc[3], @acc[3], xzr
	 umulh	@t[1], @acc[1], $g_
	add	@acc[7], @acc[7], @t[7]
	 umulh	@t[2], @acc[2], $g_

	mul	@acc[0], @t[4],   $g_
	mul	@acc[1], @acc[1], $g_
	adds	@acc[0], @acc[0], @acc[7]
	mul	@acc[2], @acc[2], $g_
	adcs	@acc[1], @acc[1], @t[0]
	mul	@t[3],   @acc[3], $g_
	adcs	@acc[2], @acc[2], @t[1]
	adcs	@t[3],   @t[3],   @t[2]
	adc	@t[4], xzr, xzr		// used in the final step

	adds	@acc[0], @acc[0], @t[5]
	adcs	@acc[1], @acc[1], @t[6]
	adcs	@acc[2], @acc[2], @t[6]
	stp	@acc[0], @acc[1], [$out_ptr,#8*4]
	adcs	@t[3],   @t[3],   @t[6]	// carry is used in the final step
	stp	@acc[2], @t[3],   [$out_ptr,#8*6]

	ret
.size	__smul_512x63_tail,.-__smul_512x63_tail

.type	__smul_256_n_shift_by_31, %function
.align	5
__smul_256_n_shift_by_31:
___
for($j=0; $j<2; $j++) {
my $f0 = $f0;   $f0 = $g0           if ($j);
my @acc = @acc; @acc = @acc[4..7]   if ($j);
my $k = 8*4*$j;
$code.=<<___;
	ldp	@acc[0], @acc[1], [$in_ptr,#8*0+$k]	// load |a| (or |b|)
	asr	@t[5], $f0, #63		// |f0|'s sign as mask (or |g0|'s)
	ldp	@acc[2], @acc[3], [$in_ptr,#8*2+$k]
	eor	@t[6], $f0, @t[5]	// conditionally negate |f0| (or |g0|)

	eor	@acc[0], @acc[0], @t[5]	// conditionally negate |a| (or |b|)
	sub	@t[6], @t[6], @t[5]
	eor	@acc[1], @acc[1], @t[5]
	adds	@acc[0], @acc[0], @t[5], lsr#63
	eor	@acc[2], @acc[2], @t[5]
	adcs	@acc[1], @acc[1], xzr
	eor	@acc[3], @acc[3], @t[5]
	 umulh	@t[0], @acc[0], @t[6]
	adcs	@acc[2], @acc[2], xzr
	 umulh	@t[1], @acc[1], @t[6]
	adc	@acc[3], @acc[3], xzr
	 umulh	@t[2], @acc[2], @t[6]
	and	@t[5], @t[5], @t[6]
	 umulh	@t[3+$j], @acc[3], @t[6]
	neg	@t[5], @t[5]

	mul	@acc[0], @acc[0], @t[6]
	mul	@acc[1], @acc[1], @t[6]
	mul	@acc[2], @acc[2], @t[6]
	adds	@acc[1], @acc[1], @t[0]
	mul	@acc[3], @acc[3], @t[6]
	adcs	@acc[2], @acc[2], @t[1]
	adcs	@acc[3], @acc[3], @t[2]
	adc	@t[3+$j], @t[3+$j], @t[5]
___
}
$code.=<<___;
	adds	@acc[0], @acc[0], @acc[4]
	adcs	@acc[1], @acc[1], @acc[5]
	adcs	@acc[2], @acc[2], @acc[6]
	adcs	@acc[3], @acc[3], @acc[7]
	adc	@acc[4], @t[3],   @t[4]

	extr	@acc[0], @acc[1], @acc[0], #31
	extr	@acc[1], @acc[2], @acc[1], #31
	extr	@acc[2], @acc[3], @acc[2], #31
	asr	@t[4], @acc[4], #63	// result's sign as mask
	extr	@acc[3], @acc[4], @acc[3], #31

	eor	@acc[0], @acc[0], @t[4]	// ensure the result is positive
	eor	@acc[1], @acc[1], @t[4]
	adds	@acc[0], @acc[0], @t[4], lsr#63
	eor	@acc[2], @acc[2], @t[4]
	adcs	@acc[1], @acc[1], xzr
	eor	@acc[3], @acc[3], @t[4]
	adcs	@acc[2], @acc[2], xzr
	stp	@acc[0], @acc[1], [$out_ptr,#8*0]
	adc	@acc[3], @acc[3], xzr
	stp	@acc[2], @acc[3], [$out_ptr,#8*2]

	eor	$f0, $f0, @t[4]		// adjust |f/g| accordingly
	eor	$g0, $g0, @t[4]
	sub	$f0, $f0, @t[4]
	sub	$g0, $g0, @t[4]

	ret
.size	__smul_256_n_shift_by_31,.-__smul_256_n_shift_by_31
___

{
my @a = @acc[0..3];
my @b = @acc[4..7];
my ($fg0, $fg1, $bias) = ($g0, $g1, @t[4]);

$code.=<<___;
.type	__ab_approximation_31_256, %function
.align	4
__ab_approximation_31_256:
	ldp	@a[2], @a[3], [$in_ptr,#8*2]
	ldp	@b[2], @b[3], [$in_ptr,#8*6]
	ldp	@a[0], @a[1], [$in_ptr,#8*0]
	ldp	@b[0], @b[1], [$in_ptr,#8*4]

.Lab_approximation_31_256_loaded:
	orr	@t[0], @a[3], @b[3]	// check top-most limbs, ...
	cmp	@t[0], #0
	csel	@a[3], @a[3], @a[2], ne
	csel	@b[3], @b[3], @b[2], ne
	csel	@a[2], @a[2], @a[1], ne
	orr	@t[0], @a[3], @b[3]	// and ones before top-most, ...
	csel	@b[2], @b[2], @b[1], ne

	cmp	@t[0], #0
	csel	@a[3], @a[3], @a[2], ne
	csel	@b[3], @b[3], @b[2], ne
	csel	@a[2], @a[2], @a[0], ne
	orr	@t[0], @a[3], @b[3]	// and one more, ...
	csel	@b[2], @b[2], @b[0], ne

	clz	@t[0], @t[0]
	cmp	@t[0], #64
	csel	@t[0], @t[0], xzr, ne
	csel	@a[3], @a[3], @a[2], ne
	csel	@b[3], @b[3], @b[2], ne
	neg	@t[1], @t[0]

	lslv	@a[3], @a[3], @t[0]	// align high limbs to the left
	lslv	@b[3], @b[3], @t[0]
	lsrv	@a[2], @a[2], @t[1]
	lsrv	@b[2], @b[2], @t[1]
	and	@a[2], @a[2], @t[1], asr#6
	and	@b[2], @b[2], @t[1], asr#6
	orr	$a_lo, @a[3], @a[2]
	orr	$b_lo, @b[3], @b[2]

	bfxil	$a_lo, @a[0], #0, #31
	bfxil	$b_lo, @b[0], #0, #31

	b	__inner_loop_31_256
	ret
.size	__ab_approximation_31_256,.-__ab_approximation_31_256

.type	__inner_loop_31_256, %function
.align	4
__inner_loop_31_256:
	mov	$cnt, #31
	mov	$fg0, #0x7FFFFFFF80000000	// |f0|=1, |g0|=0
	mov	$fg1, #0x800000007FFFFFFF	// |f1|=0, |g1|=1
	mov	$bias,#0x7FFFFFFF7FFFFFFF

.Loop_31_256:
	sbfx	@t[3], $a_lo, #0, #1	// if |a_| is odd, then we'll be subtracting
	sub	$cnt, $cnt, #1
	and	@t[0], $b_lo, @t[3]
	sub	@t[1], $b_lo, $a_lo	// |b_|-|a_|
	subs	@t[2], $a_lo, @t[0]	// |a_|-|b_| (or |a_|-0 if |a_| was even)
	mov	@t[0], $fg1
	csel	$b_lo, $b_lo, $a_lo, hs	// |b_| = |a_|
	csel	$a_lo, @t[2], @t[1], hs	// borrow means |a_|<|b_|, replace with |b_|-|a_|
	csel	$fg1, $fg1, $fg0,    hs	// exchange |fg0| and |fg1|
	csel	$fg0, $fg0, @t[0],   hs
	lsr	$a_lo, $a_lo, #1
	and	@t[0], $fg1, @t[3]
	and	@t[1], $bias, @t[3]
	sub	$fg0, $fg0, @t[0]	// |f0|-=|f1| (or |f0-=0| if |a_| was even)
	add	$fg1, $fg1, $fg1	// |f1|<<=1
	add	$fg0, $fg0, @t[1]
	sub	$fg1, $fg1, $bias
	cbnz	$cnt, .Loop_31_256

	mov	$bias, #0x7FFFFFFF
	ubfx	$f0, $fg0, #0, #32
	ubfx	$g0, $fg0, #32, #32
	ubfx	$f1, $fg1, #0, #32
	ubfx	$g1, $fg1, #32, #32
	sub	$f0, $f0, $bias		// remove bias
	sub	$g0, $g0, $bias
	sub	$f1, $f1, $bias
	sub	$g1, $g1, $bias

	ret
.size	__inner_loop_31_256,.-__inner_loop_31_256

.type	__inner_loop_62_256, %function
.align	4
__inner_loop_62_256:
	mov	$f0, #1		// |f0|=1
	mov	$g0, #0		// |g0|=0
	mov	$f1, #0		// |f1|=0
	mov	$g1, #1		// |g1|=1

.Loop_62_256:
	sbfx	@t[3], $a_lo, #0, #1	// if |a_| is odd, then we'll be subtracting
	sub	$cnt, $cnt, #1
	and	@t[0], $b_lo, @t[3]
	sub	@t[1], $b_lo, $a_lo	// |b_|-|a_|
	subs	@t[2], $a_lo, @t[0]	// |a_|-|b_| (or |a_|-0 if |a_| was even)
	mov	@t[0], $f0
	csel	$b_lo, $b_lo, $a_lo, hs	// |b_| = |a_|
	csel	$a_lo, @t[2], @t[1], hs	// borrow means |a_|<|b_|, replace with |b_|-|a_|
	mov	@t[1], $g0
	csel	$f0, $f0, $f1,       hs	// exchange |f0| and |f1|
	csel	$f1, $f1, @t[0],     hs
	csel	$g0, $g0, $g1,       hs	// exchange |g0| and |g1|
	csel	$g1, $g1, @t[1],     hs
	lsr	$a_lo, $a_lo, #1
	and	@t[0], $f1, @t[3]
	and	@t[1], $g1, @t[3]
	add	$f1, $f1, $f1		// |f1|<<=1
	add	$g1, $g1, $g1		// |g1|<<=1
	sub	$f0, $f0, @t[0]		// |f0|-=|f1| (or |f0-=0| if |a_| was even)
	sub	$g0, $g0, @t[1]		// |g0|-=|g1| (or |g0-=0| ...)
	cbnz	$cnt, .Loop_62_256

	ret
.size	__inner_loop_62_256,.-__inner_loop_62_256
___
}

foreach(split("\n",$code)) {
    s/\b(smaddl\s+x[0-9]+,\s)x([0-9]+,\s+)x([0-9]+)/$1w$2w$3/;
    print $_,"\n";
}
close STDOUT;
