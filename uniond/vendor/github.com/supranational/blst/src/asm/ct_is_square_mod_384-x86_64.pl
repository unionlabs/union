#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
#
# Both constant-time and fast quadratic residue test as suggested in
# https://eprint.iacr.org/2020/972. Performance is >5x better than
# modulus-specific Legendre symbol addition chain...
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
if ($flavour =~ /\./) { $output = $flavour; undef $flavour; }

$win64=0; $win64=1 if ($flavour =~ /[nm]asm|mingw64/ || $output =~ /\.asm$/);

$0 =~ m/(.*[\/\\])[^\/\\]+$/; $dir=$1;
( $xlate="${dir}x86_64-xlate.pl" and -f $xlate ) or
( $xlate="${dir}../../perlasm/x86_64-xlate.pl" and -f $xlate) or
die "can't locate x86_64-xlate.pl";

open STDOUT,"| \"$^X\" \"$xlate\" $flavour \"$output\""
    or die "can't call $xlate: $!";

my ($out_ptr, $in_ptr) = ("%rdi", "%rsi");
my ($f0, $g0, $f1, $g1) = ("%rax", "%rbx", "%rdx","%rcx");
my @acc=map("%r$_",(8..15));
my $L = "%rbp";

$frame = 8*3+2*256;

$code.=<<___;
.text

.globl	ct_is_square_mod_384
.hidden	ct_is_square_mod_384
.type	ct_is_square_mod_384,\@function,2,"unwind"
.align	32
ct_is_square_mod_384:
.cfi_startproc
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	push	%r12
.cfi_push	%r12
	push	%r13
.cfi_push	%r13
	push	%r14
.cfi_push	%r14
	push	%r15
.cfi_push	%r15
	sub	\$$frame, %rsp
.cfi_adjust_cfa_offset	$frame
.cfi_end_prologue

	lea	8*3+255(%rsp), %rax	# find closest 256-byte-aligned spot
	and	\$-256, %rax		# in the frame...

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	mov	8*0(%rdi), @acc[0]	# load input
	mov	8*1(%rdi), @acc[1]
	mov	8*2(%rdi), @acc[2]
	mov	8*3(%rdi), @acc[3]
	mov	8*4(%rdi), @acc[4]
	mov	8*5(%rdi), @acc[5]

	mov	8*0(%rsi), @acc[6]	# load modulus
	mov	8*1(%rsi), @acc[7]
	mov	8*2(%rsi), %rbx
	mov	8*3(%rsi), %rcx
	mov	8*4(%rsi), %rdx
	mov	8*5(%rsi), %rdi
	mov	%rax, $in_ptr		# pointer to source |a|b|

	mov	@acc[0], 8*0(%rax)	# copy input to |a|
	mov	@acc[1], 8*1(%rax)
	mov	@acc[2], 8*2(%rax)
	mov	@acc[3], 8*3(%rax)
	mov	@acc[4], 8*4(%rax)
	mov	@acc[5], 8*5(%rax)

	mov	@acc[6], 8*6(%rax)	# copy modulus to |b|
	mov	@acc[7], 8*7(%rax)
	mov	%rbx,    8*8(%rax)
	mov	%rcx,    8*9(%rax)
	mov	%rdx,    8*10(%rax)
	mov	%rdi,    8*11(%rax)

	xor	$L, $L			# initialize the Legendre symbol
	mov	\$24, %ecx		# 24 is 768/30-1
	jmp	.Loop_is_square

.align	32
.Loop_is_square:
	mov	%ecx, 8*2(%rsp)		# offload loop counter

	call	__ab_approximation_30
	mov	$f0, 8*0(%rsp)		# offload |f0| and |g0|
	mov	$g0, 8*1(%rsp)

	mov	\$128+8*6, $out_ptr
	xor	$in_ptr, $out_ptr	# pointer to destination |b|
	call	__smulq_384_n_shift_by_30

	mov	8*0(%rsp), $f1		# pop |f0| and |g0|
	mov	8*1(%rsp), $g1
	lea	-8*6($out_ptr),$out_ptr	# pointer to destination |a|
	call	__smulq_384_n_shift_by_30

	mov	8*2(%rsp), %ecx		# re-load loop counter
	xor	\$128, $in_ptr		# flip-flop pointer to source |a|b|

	and	8*6($out_ptr), @acc[6]	# if |a| was negative, adjust |L|
	shr	\$1, @acc[6]
	add	@acc[6], $L

	sub	\$1, %ecx
	jnz	.Loop_is_square

	################################# last iteration
	#call	__ab_approximation_30	# |a| and |b| are exact, just load
	#mov	8*0($in_ptr), @acc[0]	# |a_|
	mov	8*6($in_ptr), @acc[1]	# |b_|
	call	__inner_loop_48		# 48 is 768%30+30

	mov	\$1, %rax
	and	$L,  %rax
	xor	\$1, %rax		# return value

	lea	$frame(%rsp), %r8	# size optimization
	mov	8*0(%r8),%r15
.cfi_restore	%r15
	mov	8*1(%r8),%r14
.cfi_restore	%r14
	mov	8*2(%r8),%r13
.cfi_restore	%r13
	mov	8*3(%r8),%r12
.cfi_restore	%r12
	mov	8*4(%r8),%rbx
.cfi_restore	%rbx
	mov	8*5(%r8),%rbp
.cfi_restore	%rbp
	lea	8*6(%r8),%rsp
.cfi_adjust_cfa_offset	-$frame-8*6
.cfi_epilogue
	ret
.cfi_endproc
.size	ct_is_square_mod_384,.-ct_is_square_mod_384

.type	__smulq_384_n_shift_by_30,\@abi-omnipotent
.align	32
__smulq_384_n_shift_by_30:
___
for($j=0; $j<2; $j++) {
$code.=<<___;
	mov	8*0($in_ptr), @acc[0]	# load |a| (or |b|)
	mov	8*1($in_ptr), @acc[1]
	mov	8*2($in_ptr), @acc[2]
	mov	8*3($in_ptr), @acc[3]
	mov	8*4($in_ptr), @acc[4]
	mov	8*5($in_ptr), @acc[5]

	mov	%rdx, %rbx		# |f1| (or |g1|)
	sar	\$63, %rdx		# |f1|'s sign as mask (or |g1|'s)
	xor	%rax, %rax
	sub	%rdx, %rax		# |f1|'s sign as bit (or |g1|'s)

	xor	%rdx, %rbx		# conditionally negate |f1| (or |g1|)
	add	%rax, %rbx

	xor	%rdx, @acc[0]		# conditionally negate |a| (or |b|)
	xor	%rdx, @acc[1]
	xor	%rdx, @acc[2]
	xor	%rdx, @acc[3]
	xor	%rdx, @acc[4]
	xor	%rdx, @acc[5]
	add	@acc[0], %rax
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]
	adc	\$0, @acc[4]
	adc	\$0, @acc[5]

	mov	%rdx, @acc[6+$j]
	and	%rbx, @acc[6+$j]
	mulq	%rbx			# |a|*|f1| (or |b|*|g1|)
	mov	%rax, @acc[0]
	mov	@acc[1], %rax
	mov	%rdx, @acc[1]
___
for($i=1; $i<5; $i++) {
$code.=<<___;
	mulq	%rbx
	add	%rax, @acc[$i]
	mov	@acc[$i+1], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[$i+1]
___
}
$code.=<<___;
	neg	@acc[6+$j]
	mulq	%rbx
	add	%rax, @acc[5]
	adc	%rdx, @acc[6+$j]
___
$code.=<<___	if ($j==0);
	lea	8*6($in_ptr), $in_ptr	# pointer to |b|
	mov	$g1, %rdx

	mov	@acc[0], 8*0($out_ptr)
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)
	mov	@acc[4], 8*4($out_ptr)
	mov	@acc[5], 8*5($out_ptr)
___
}
$code.=<<___;
	lea	-8*6($in_ptr), $in_ptr	# restore original in_ptr

	add	8*0($out_ptr), @acc[0]
	adc	8*1($out_ptr), @acc[1]
	adc	8*2($out_ptr), @acc[2]
	adc	8*3($out_ptr), @acc[3]
	adc	8*4($out_ptr), @acc[4]
	adc	8*5($out_ptr), @acc[5]
	adc	@acc[7],       @acc[6]

	shrd	\$30, @acc[1], @acc[0]
	shrd	\$30, @acc[2], @acc[1]
	shrd	\$30, @acc[3], @acc[2]
	shrd	\$30, @acc[4], @acc[3]
	shrd	\$30, @acc[5], @acc[4]
	shrd	\$30, @acc[6], @acc[5]

	sar	\$63, @acc[6]		# sign as mask
	xor	%rbx, %rbx
	sub	@acc[6], %rbx		# sign as bit

	xor	@acc[6], @acc[0]	# conditionally negate the result
	xor	@acc[6], @acc[1]
	xor	@acc[6], @acc[2]
	xor	@acc[6], @acc[3]
	xor	@acc[6], @acc[4]
	xor	@acc[6], @acc[5]
	add	%rbx, @acc[0]
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]
	adc	\$0, @acc[4]
	adc	\$0, @acc[5]

	mov	@acc[0], 8*0($out_ptr)
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)
	mov	@acc[4], 8*4($out_ptr)
	mov	@acc[5], 8*5($out_ptr)

	ret
.size	__smulq_384_n_shift_by_30,.-__smulq_384_n_shift_by_30
___
{
my ($a_, $b_) = @acc[0..1];
my ($t0, $t1, $t2, $t3, $t4, $t5) = map("%r$_",(10..15));
my ($fg0, $fg1, $bias) = ($g0, $g1, $t5);
my $cnt = "%edi";
{
my @a = @acc[0..5];
my @b = (@a[1..3], $t4, $t5, $g0);

$code.=<<___;
.type	__ab_approximation_30,\@abi-omnipotent
.align	32
__ab_approximation_30:
	mov	8*11($in_ptr), @b[5]	# load |b| in reverse order
	mov	8*10($in_ptr), @b[4]
	mov	8*9($in_ptr),  @b[3]

	mov	@a[5], %rax
	or	@b[5], %rax		# check top-most limbs, ...
	cmovz	@a[4], @a[5]
	cmovz	@b[4], @b[5]
	cmovz	@a[3], @a[4]
	mov	8*8($in_ptr), @b[2]
	cmovz	@b[3], @b[4]

	mov	@a[5], %rax
	or	@b[5], %rax		# ... ones before top-most, ...
	cmovz	@a[4], @a[5]
	cmovz	@b[4], @b[5]
	cmovz	@a[2], @a[4]
	mov	8*7($in_ptr), @b[1]
	cmovz	@b[2], @b[4]

	mov	@a[5], %rax
	or	@b[5], %rax		# ... and ones before that ...
	cmovz	@a[4], @a[5]
	cmovz	@b[4], @b[5]
	cmovz	@a[1], @a[4]
	mov	8*6($in_ptr), @b[0]
	cmovz	@b[1], @b[4]

	mov	@a[5], %rax
	or	@b[5], %rax		# ... and ones before that ...
	cmovz	@a[4], @a[5]
	cmovz	@b[4], @b[5]
	cmovz	@a[0], @a[4]
	cmovz	@b[0], @b[4]

	mov	@a[5], %rax
	or	@b[5], %rax
	bsr	%rax, %rcx
	lea	1(%rcx), %rcx
	cmovz	@a[0], @a[5]
	cmovz	@b[0], @b[5]
	cmovz	%rax, %rcx
	neg	%rcx
	#and	\$63, %rcx		# debugging artefact

	shldq	%cl, @a[4], @a[5]	# align second limb to the left
	shldq	%cl, @b[4], @b[5]

	mov	\$0xFFFFFFFF00000000, %rax
	mov	@a[0]d, ${a_}d
	mov	@b[0]d, ${b_}d
	and	%rax, @a[5]
	and	%rax, @b[5]
	or	@a[5], ${a_}
	or	@b[5], ${b_}

	jmp	__inner_loop_30

	ret
.size	__ab_approximation_30,.-__ab_approximation_30
___
}
$code.=<<___;
.type	__inner_loop_30,\@abi-omnipotent
.align	32
__inner_loop_30:		################# by Thomas Pornin
	mov	\$0x7FFFFFFF80000000, $fg0	# |f0|=1, |g0|=0
	mov	\$0x800000007FFFFFFF, $fg1	# |f1|=0, |g1|=1
	lea	-1($fg0), $bias			# 0x7FFFFFFF7FFFFFFF
	mov	\$30, $cnt

.Loop_30:
	 mov	$a_, %rax
	 and	$b_, %rax
	 shr	\$1, %rax		# (a_ & b_) >> 1

	cmp	$b_, $a_		# if |a_|<|b_|, swap the variables
	mov	$a_, $t0
	mov	$b_, $t1
	 lea	(%rax,$L), %rax		# pre-"negate" |L|
	mov	$fg0, $t2
	mov	$fg1, $t3
	 mov	$L,   $t4
	cmovb	$b_, $a_
	cmovb	$t0, $b_
	cmovb	$fg1, $fg0
	cmovb	$t2, $fg1
	 cmovb	%rax, $L

	sub	$b_, $a_		# |a_|-|b_|
	sub	$fg1, $fg0		# |f0|-|f1|, |g0|-|g1|
	add	$bias, $fg0

	test	\$1, $t0		# if |a_| was even, roll back 
	cmovz	$t0, $a_
	cmovz	$t1, $b_
	cmovz	$t2, $fg0
	cmovz	$t3, $fg1
	cmovz	$t4, $L

	 lea	2($b_), %rax
	shr	\$1, $a_		# |a_|>>=1
	 shr	\$2, %rax
	add	$fg1, $fg1		# |f1|<<=1, |g1|<<=1
	 lea	(%rax,$L), $L		# "negate" |L| if |b|%8 is 3 or 5
	sub	$bias, $fg1

	sub	\$1, $cnt
	jnz	.Loop_30

	shr	\$32, $bias
	mov	%ebx, %eax		# $fg0 -> $f0
	shr	\$32, $g0
	mov	%ecx, %edx		# $fg1 -> $f1
	shr	\$32, $g1
	sub	$bias, $f0		# remove the bias
	sub	$bias, $g0
	sub	$bias, $f1
	sub	$bias, $g1

	ret	# __SGX_LVI_HARDENING_CLOBBER__=$a_
.size	__inner_loop_30,.-__inner_loop_30

.type	__inner_loop_48,\@abi-omnipotent
.align	32
__inner_loop_48:
	mov	\$48, $cnt		# 48 is 768%30+30

.Loop_48:
	 mov	$a_, %rax
	 and	$b_, %rax
	 shr	\$1, %rax		# (a_ & b_) >> 1

	cmp	$b_, $a_		# if |a_|<|b_|, swap the variables
	mov	$a_, $t0
	mov	$b_, $t1
	 lea	(%rax,$L), %rax
	 mov	$L,  $t2
	cmovb	$b_, $a_
	cmovb	$t0, $b_
	 cmovb	%rax, $L

	sub	$b_, $a_		# |a_|-|b_|

	test	\$1, $t0		# if |a_| was even, roll back 
	cmovz	$t0, $a_
	cmovz	$t1, $b_
	cmovz	$t2, $L

	 lea	2($b_), %rax
	shr	\$1, $a_		# |a_|>>=1
	 shr	\$2, %rax
	 add	%rax, $L		# "negate" |L| if |b|%8 is 3 or 5

	sub	\$1, $cnt
	jnz	.Loop_48

	ret
.size	__inner_loop_48,.-__inner_loop_48
___
}

print $code;
close STDOUT;
