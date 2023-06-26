#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0

$flavour = shift;
$output  = shift;
if ($flavour =~ /\./) { $output = $flavour; undef $flavour; }

$win64=0; $win64=1 if ($flavour =~ /[nm]asm|mingw64/ || $output =~ /\.asm$/);

$0 =~ m/(.*[\/\\])[^\/\\]+$/; $dir=$1;
( $xlate="${dir}x86_64-xlate.pl" and -f $xlate ) or
( $xlate="${dir}../../perlasm/x86_64-xlate.pl" and -f $xlate) or
die "can't locate x86_64-xlate.pl";

open STDOUT,"| \"$^X\" \"$xlate\" $flavour \"$output\""
    or die "can't call $xlate: $!";

$c_ref=<<'___';
/*
 * |div_top| points at two most significant limbs of the dividend, |d_hi|
 * and |d_lo| are two most significant limbs of the divisor. If divisor
 * is only one limb, it is to be passed in |d_hi| with zero in |d_lo|.
 * The divisor is required to be "bitwise left-aligned," and dividend's
 * top limbs to be not larger than the divisor's. The latter limitation
 * can be problematic in the first iteration of multi-precision division,
 * where in most general case the condition would have to be "smaller."
 * The subroutine considers four limbs, two of which are "overlapping,"
 * hence the name... Another way to look at it is to think of the pair
 * of the dividend's limbs being suffixed with a zero:
 *   +-------+-------+-------+
 * R |       |       |   0   |
 *   +-------+-------+-------+
 *           +-------+-------+
 * D         |       |       |
 *           +-------+-------+
 */
limb_t div_3_limbs(const limb_t *div_top, limb_t d_lo, limb_t d_hi)
{
    llimb_t R = ((llimb_t)div_top[1] << LIMB_BITS) | div_top[0];
    llimb_t D = ((llimb_t)d_hi << LIMB_BITS) | d_lo;
    limb_t Q = 0, mask;
    size_t i;

    for (i = 0; i < LIMB_BITS; i++) {
        Q <<= 1;
        mask = (R >= D);
        Q |= mask;
        R -= (D & ((llimb_t)0 - mask));
        D >>= 1;
    }

    mask = 0 - (Q >> (LIMB_BITS - 1));   /* does it overflow? */

    Q <<= 1;
    Q |= (R >= D);

    return (Q | mask);
}
___

$code.=<<___;
.text

.globl	div_3_limbs
.hidden	div_3_limbs
.type	div_3_limbs,\@function,3
.align	32
div_3_limbs:
	mov	(%rdi),%r8		# load R.lo
	mov	8(%rdi),%r9		# load R.hi
	xor	%rax,%rax		# Q = 0
	mov	\$64,%ecx		# loop counter

.Loop:
	 mov	%r8,%r10		# put aside R
	sub	%rsi,%r8		# R -= D
	 mov	%r9,%r11
	sbb	%rdx,%r9
	lea	1(%rax,%rax),%rax	# Q <<= 1 + speculative bit
	 mov	%rdx,%rdi
	cmovc	%r10,%r8		# restore R if R - D borrowed
	cmovc	%r11,%r9
	sbb	\$0,%rax		# subtract speculative bit
	 shl	\$63,%rdi
	 shr	\$1,%rsi
	 shr	\$1,%rdx
	 or	%rdi,%rsi		# D >>= 1
	sub	\$1,%ecx
	jnz	.Loop

	lea	1(%rax,%rax),%rcx	# Q <<= 1 + speculative bit
	sar	\$63,%rax		# top bit -> mask

	sub	%rsi,%r8		# R -= D
	sbb	%rdx,%r9
	sbb	\$0,%rcx		# subtract speculative bit

	or	%rcx,%rax		# all ones if overflow

	ret
.size	div_3_limbs,.-div_3_limbs
___
########################################################################
# Calculate remainder and adjust the quotient, which can be off-by-one.
# Then save quotient in limb next to top limb of the remainder. There is
# place, because the remainder/next-iteration-dividend gets shorter by
# one limb.
{
my ($div_rem, $divisor, $quotient) = ("%rdi", "%rsi", "%rcx");
my @acc = ("%r8", "%r9", "%rdx");
my @tmp = ("%r10", "%r11", "%rax");

$code.=<<___;
.globl	quot_rem_128
.hidden	quot_rem_128
.type	quot_rem_128,\@function,3
.align	32
quot_rem_128:
	mov	%rdx, %rax
	mov	%rdx, $quotient

	mulq	0($divisor)		# divisor[0:1] * quotient
	mov	%rax, @acc[0]
	mov	$quotient, %rax
	mov	%rdx, @acc[1]

	mulq	8($divisor)
	add	%rax, @acc[1]
	adc	\$0, %rdx		# %rdx is @acc[2]

	mov	0($div_rem), @tmp[0]	# load 3 limbs of the dividend
	mov	8($div_rem), @tmp[1]
	mov	16($div_rem), @tmp[2]

	sub	@acc[0], @tmp[0]	# dividend - divisor * quotient
	sbb	@acc[1], @tmp[1]
	sbb	@acc[2], @tmp[2]
	sbb	@acc[0], @acc[0]	# borrow -> mask

	add	@acc[0], $quotient	# if borrowed, adjust the quotient ...
	mov	@acc[0], @acc[1]
	and	0($divisor), @acc[0]
	and	8($divisor), @acc[1]
	add	@acc[0], @tmp[0]	# ... and add divisor
	adc	@acc[1], @tmp[1]

	mov	@tmp[0], 0($div_rem)	# save 2 limbs of the remainder ...
	mov	@tmp[1], 8($div_rem)
	mov	$quotient, 16($div_rem)	# ... and 1 limb of the quotient

	mov	$quotient, %rax		# return adjusted quotient

	ret
.size	quot_rem_128,.-quot_rem_128

########################################################################
# Unlike 128-bit case above, quotient is exact. As result just one limb
# of the dividend is sufficient to calculate the remainder...

.globl	quot_rem_64
.hidden	quot_rem_64
.type	quot_rem_64,\@function,3
.align	32
quot_rem_64:
	mov	%rdx, %rax		# return quotient
	imulq	0($divisor), %rdx	# divisor[0] * quotient

	mov	0($div_rem), @tmp[0]	# load 1 limb of the dividend

	sub	%rdx, @tmp[0]		# dividend - divisor * quotient

	mov	@tmp[0], 0($div_rem)	# save 1 limb of the remainder ...
	mov	%rax, 8($div_rem)	# ... and 1 limb of the quotient

	ret
.size	quot_rem_64,.-quot_rem_64
___
}

print $code;
close STDOUT;
