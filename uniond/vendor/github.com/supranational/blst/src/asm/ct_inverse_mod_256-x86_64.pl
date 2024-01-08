#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
#
# Both constant-time and fast Euclidean inversion as suggested in
# https://eprint.iacr.org/2020/972. ~5.300 cycles on Coffee Lake.
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

        # __smulq_256_n_shift_by_31
        a, b = (a*f0 + b*g0) >> k, (a*f1 + b*g1) >> k
        if a < 0:
            a, f0, g0 = -a, -f0, -g0
        if b < 0:
            b, f1, g1 = -b, -f1, -g1

        # __smulq_512x63
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
if ($flavour =~ /\./) { $output = $flavour; undef $flavour; }

$win64=0; $win64=1 if ($flavour =~ /[nm]asm|mingw64/ || $output =~ /\.asm$/);

$0 =~ m/(.*[\/\\])[^\/\\]+$/; $dir=$1;
( $xlate="${dir}x86_64-xlate.pl" and -f $xlate ) or
( $xlate="${dir}../../perlasm/x86_64-xlate.pl" and -f $xlate) or
die "can't locate x86_64-xlate.pl";

open STDOUT,"| \"$^X\" \"$xlate\" $flavour \"$output\""
    or die "can't call $xlate: $!";

my ($out_ptr, $in_ptr, $n_ptr, $nx_ptr) = ("%rdi", "%rsi", "%rdx", "%rcx");
my @acc = map("%r$_",(8..15));
my ($f0, $g0, $f1, $g1) = ("%rdx","%rcx","%r12","%r13");
my $cnt = "%edx";

$frame = 8*6+2*512;

$code.=<<___;
.text

.globl	ct_inverse_mod_256
.type	ct_inverse_mod_256,\@function,4,"unwind"
.align	32
ct_inverse_mod_256:
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

	lea	8*6+511(%rsp), %rax	# find closest 512-byte-aligned spot
	and	\$-512, %rax		# in the frame...
	mov	$out_ptr, 8*4(%rsp)
	mov	$nx_ptr,  8*5(%rsp)

	mov	8*0($in_ptr), @acc[0]	# load input
	mov	8*1($in_ptr), @acc[1]
	mov	8*2($in_ptr), @acc[2]
	mov	8*3($in_ptr), @acc[3]

	mov	8*0($n_ptr), @acc[4]	# load modulus
	mov	8*1($n_ptr), @acc[5]
	mov	8*2($n_ptr), @acc[6]
	mov	8*3($n_ptr), @acc[7]

	mov	@acc[0], 8*0(%rax)	# copy input to |a|
	mov	@acc[1], 8*1(%rax)
	mov	@acc[2], 8*2(%rax)
	mov	@acc[3], 8*3(%rax)

	mov	@acc[4], 8*4(%rax)	# copy modulus to |b|
	mov	@acc[5], 8*5(%rax)
	mov	@acc[6], 8*6(%rax)
	mov	@acc[7], 8*7(%rax)
	mov	%rax, $in_ptr

	################################# first iteration
	mov	\$31, $cnt
	call	__ab_approximation_31_256
	#mov	$f0, 8*0(%rsp)
	#mov	$g0, 8*1(%rsp)
	mov	$f1, 8*2(%rsp)
	mov	$g1, 8*3(%rsp)

	mov	\$256, $out_ptr
	xor	$in_ptr, $out_ptr	# pointer to destination |a|b|u|v|
	call	__smulq_256_n_shift_by_31
	#mov	$f0, 8*0(%rsp)		# corrected |f0|
	#mov	$g0, 8*1(%rsp)		# corrected |g0|
	mov	$f0, 8*8($out_ptr)	# initialize |u| with |f0|

	mov	8*2(%rsp), $f0		# |f1|
	mov	8*3(%rsp), $g0		# |g1|
	lea	8*4($out_ptr), $out_ptr	# pointer to destination |b|
	call	__smulq_256_n_shift_by_31
	#mov	$f0, 8*2(%rsp)		# corrected |f1|
	#mov	$g0, 8*3(%rsp)		# corrected |g1|
	mov	$f0, 8*9($out_ptr)	# initialize |v| with |f1|

	################################# second iteration
	xor	\$256, $in_ptr		# flip-flop pointer to source |a|b|u|v|
	mov	\$31, $cnt
	call	__ab_approximation_31_256
	#mov	$f0, 8*0(%rsp)
	#mov	$g0, 8*1(%rsp)
	mov	$f1, 8*2(%rsp)
	mov	$g1, 8*3(%rsp)

	mov	\$256, $out_ptr
	xor	$in_ptr, $out_ptr	# pointer to destination |a|b|u|v|
	call	__smulq_256_n_shift_by_31
	mov	$f0, 8*0(%rsp)		# corrected |f0|
	mov	$g0, 8*1(%rsp)		# corrected |g0|

	mov	8*2(%rsp), $f0		# |f1|
	mov	8*3(%rsp), $g0		# |g1|
	lea	8*4($out_ptr), $out_ptr	# pointer to destination |b|
	call	__smulq_256_n_shift_by_31
	#mov	$f0, 8*2(%rsp)		# corrected |f1|
	#mov	$g0, 8*3(%rsp)		# corrected |g1|

	mov	8*8($in_ptr),  @acc[0]	# |u|
	mov	8*13($in_ptr), @acc[4]	# |v|
	mov	@acc[0], @acc[1]
	imulq	8*0(%rsp), @acc[0]	# |u|*|f0|
	mov	@acc[4], @acc[5]
	imulq	8*1(%rsp), @acc[4]	# |v|*|g0|
	add	@acc[4], @acc[0]
	mov	@acc[0], 8*4($out_ptr)	# destination |u|
	sar	\$63, @acc[0]		# sign extension
	mov	@acc[0], 8*5($out_ptr)
	mov	@acc[0], 8*6($out_ptr)
	mov	@acc[0], 8*7($out_ptr)
	mov	@acc[0], 8*8($out_ptr)
	lea	8*8($in_ptr), $in_ptr	# make in_ptr "rewindable" with xor

	imulq	$f0, @acc[1]		# |u|*|f1|
	imulq	$g0, @acc[5]		# |v|*|g1|
	add	@acc[5], @acc[1]
	mov	@acc[1], 8*9($out_ptr)	# destination |v|
	sar	\$63, @acc[1]		# sign extension
	mov	@acc[1], 8*10($out_ptr)
	mov	@acc[1], 8*11($out_ptr)
	mov	@acc[1], 8*12($out_ptr)
	mov	@acc[1], 8*13($out_ptr)
___
for($i=2; $i<15; $i++) {
my $smul_512x63  = $i>8  ? "__smulq_512x63"
                         : "__smulq_256x63";
$code.=<<___;
	xor	\$256+8*8, $in_ptr	# flip-flop pointer to source |a|b|u|v|
	mov	\$31, $cnt
	call	__ab_approximation_31_256
	#mov	$f0, 8*0(%rsp)
	#mov	$g0, 8*1(%rsp)
	mov	$f1, 8*2(%rsp)
	mov	$g1, 8*3(%rsp)

	mov	\$256, $out_ptr
	xor	$in_ptr, $out_ptr	# pointer to destination |a|b|u|v|
	call	__smulq_256_n_shift_by_31
	mov	$f0, 8*0(%rsp)		# corrected |f0|
	mov	$g0, 8*1(%rsp)		# corrected |g0|

	mov	8*2(%rsp), $f0		# |f1|
	mov	8*3(%rsp), $g0		# |g1|
	lea	8*4($out_ptr), $out_ptr	# pointer to destination |b|
	call	__smulq_256_n_shift_by_31
	mov	$f0, 8*2(%rsp)		# corrected |f1|
	mov	$g0, 8*3(%rsp)		# corrected |g1|

	mov	8*0(%rsp), $f0		# |f0|
	mov	8*1(%rsp), $g0		# |g0|
	lea	8*8($in_ptr), $in_ptr	# pointer to source |u|v|
	lea	8*4($out_ptr), $out_ptr	# pointer to destination |u|
	call	__smulq_256x63

	mov	8*2(%rsp), $f0		# |f1|
	mov	8*3(%rsp), $g0		# |g1|
	lea	8*5($out_ptr),$out_ptr	# pointer to destination |v|
	call	$smul_512x63
___
$code.=<<___	if ($i==8);
	sar	\$63, %rbp		# sign extension
	mov	%rbp, 8*5($out_ptr)
	mov	%rbp, 8*6($out_ptr)
	mov	%rbp, 8*7($out_ptr)
___
}
$code.=<<___;
	################################# two[!] last iterations in one go
	xor	\$256+8*8, $in_ptr	# flip-flop pointer to source |a|b|u|v|
	mov	\$47, $cnt		# 31 + 512 % 31
	#call	__ab_approximation_31	# |a| and |b| are exact, just load
	mov	8*0($in_ptr), @acc[0]	# |a_lo|
	#xor	@acc[1],      @acc[1]	# |a_hi|
	mov	8*4($in_ptr), @acc[2]	# |b_lo|
	#xor	@acc[3],      @acc[3]	# |b_hi|
	call	__inner_loop_62_256
	#mov	$f0, 8*0(%rsp)
	#mov	$g0, 8*1(%rsp)
	#mov	$f1, 8*2(%rsp)
	#mov	$g1, 8*3(%rsp)

	#mov	8*0(%rsp), $f0		# |f0|
	#mov	8*1(%rsp), $g0		# |g0|
	lea	8*8($in_ptr), $in_ptr	# pointer to source |u|v|
	#lea	8*6($out_ptr), $out_ptr	# pointer to destination |u|
	#call	__smulq_256x63

	#mov	8*2(%rsp), $f0		# |f1|
	#mov	8*3(%rsp), $g0		# |g1|
	mov	$f1, $f0
	mov	$g1, $g0
	mov	8*4(%rsp), $out_ptr	# original |out_ptr|
	call	__smulq_512x63
	adc	%rbp, %rdx		# the excess limb of the result

	mov	8*5(%rsp), $in_ptr	# original |nx_ptr|
	mov	%rdx, %rax
	sar	\$63, %rdx		# result's sign as mask

	mov	%rdx, @acc[0]		# mask |modulus|
	mov	%rdx, @acc[1]
	and	8*0($in_ptr), @acc[0]
	mov	%rdx, @acc[2]
	and	8*1($in_ptr), @acc[1]
	and	8*2($in_ptr), @acc[2]
	and	8*3($in_ptr), %rdx

	add	@acc[0], @acc[4]	# conditionally add |modulus|<<256
	adc	@acc[1], @acc[5]
	adc	@acc[2], @acc[6]
	adc	%rdx,    @acc[7]
	adc	\$0,     %rax

	mov	%rax, %rdx
	neg	%rax
	or	%rax, %rdx		# excess bit or sign as mask
	sar	\$63, %rax		# excess bit as mask

	mov	%rdx, @acc[0]		# mask |modulus|
	mov	%rdx, @acc[1]
	and	8*0($in_ptr), @acc[0]
	mov	%rdx, @acc[2]
	and	8*1($in_ptr), @acc[1]
	and	8*2($in_ptr), @acc[2]
	and	8*3($in_ptr), %rdx

	xor	%rax, @acc[0]		# conditionally negate |modulus|
	xor	%rcx, %rcx
	xor	%rax, @acc[1]
	sub	%rax, %rcx
	xor	%rax, @acc[2]
	xor	%rax, %rdx
	add	%rcx, @acc[0]
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, %rdx

	add	@acc[0], @acc[4]	# final adjustment for |modulus|<<256
	adc	@acc[1], @acc[5]
	adc	@acc[2], @acc[6]
	adc	%rdx,    @acc[7]

	mov	@acc[4], 8*4($out_ptr)	# store absolute value
	mov	@acc[5], 8*5($out_ptr)
	mov	@acc[6], 8*6($out_ptr)
	mov	@acc[7], 8*7($out_ptr)

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
.size	ct_inverse_mod_256,.-ct_inverse_mod_256
___
########################################################################
# Signed |u|*|f?|+|v|*|g?| subroutines. "NNN" in "NNNx63" suffix refers
# to the maximum bit-length of the *result*, and "63" - to the maximum
# bit-length of the |f?| and |g?| single-limb multiplicands. However!
# The latter should not be taken literally, as they are always chosen so
# that "bad things" don't happen. For example, there comes a point when
# |v| grows beyond 383 bits, while |u| remains 383 bits wide. Yet, we
# always call __smul_383x63 to perform |u|*|f0|+|v|*|g0| step. This is
# because past that point |f0| is always 1 and |g0| is always 0. And,
# since |u| never grows beyond 383 bits, __smul_767x63 doesn't have to
# perform full-width |u|*|f1| multiplication, half-width one with sign
# extension is sufficient...
$code.=<<___;
.type	__smulq_512x63,\@abi-omnipotent
.align	32
__smulq_512x63:
	mov	8*0($in_ptr), @acc[0]	# load |u|
	mov	8*1($in_ptr), @acc[1]
	mov	8*2($in_ptr), @acc[2]
	mov	8*3($in_ptr), @acc[3]
	mov	8*4($in_ptr), %rbp	# sign limb

	mov	$f0, %rbx
	sar	\$63, $f0		# |f0|'s sign as mask
	xor	%rax, %rax
	sub	$f0, %rax		# |f0|'s sign as bit

	xor	$f0, %rbx		# conditionally negate |f0|
	add	%rax, %rbx

	xor	$f0, @acc[0]		# conditionally negate |u|
	xor	$f0, @acc[1]
	xor	$f0, @acc[2]
	xor	$f0, @acc[3]
	xor	$f0, %rbp
	add	@acc[0], %rax
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]
	adc	\$0, %rbp

	mulq	%rbx			# |u|*|f0|
	mov	%rax, 8*0($out_ptr)	# offload |u|*|f0|
	mov	@acc[1], %rax
	mov	%rdx, @acc[1]
___
for($i=1; $i<3; $i++) {
$code.=<<___;
	mulq	%rbx
	add	%rax, @acc[$i]
	mov	@acc[$i+1], %rax
	adc	\$0, %rdx
	mov	@acc[$i], 8*$i($out_ptr)
	mov	%rdx, @acc[$i+1]
___
}
$code.=<<___;
	and	%rbx, %rbp
	neg	%rbp
	mulq	%rbx
	add	%rax, @acc[3]
	adc	%rdx, %rbp
	mov	@acc[3], 8*3($out_ptr)

	mov	8*5($in_ptr), @acc[0]	# load |v|
	mov	8*6($in_ptr), @acc[1]
	mov	8*7($in_ptr), @acc[2]
	mov	8*8($in_ptr), @acc[3]
	mov	8*9($in_ptr), @acc[4]
	mov	8*10($in_ptr), @acc[5]
	mov	8*11($in_ptr), @acc[6]
	mov	8*12($in_ptr), @acc[7]

	mov	$g0, $f0
	sar	\$63, $f0		# |g0|'s sign as mask
	xor	%rax, %rax
	sub	$f0, %rax		# |g0|'s sign as bit

	xor	$f0, $g0		# conditionally negate |g0|
	add	%rax, $g0

	xor	$f0, @acc[0]		# conditionally negate |v|
	xor	$f0, @acc[1]
	xor	$f0, @acc[2]
	xor	$f0, @acc[3]
	xor	$f0, @acc[4]
	xor	$f0, @acc[5]
	xor	$f0, @acc[6]
	xor	$f0, @acc[7]
	add	@acc[0], %rax
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]
	adc	\$0, @acc[4]
	adc	\$0, @acc[5]
	adc	\$0, @acc[6]
	adc	\$0, @acc[7]

	mulq	$g0
	mov	%rax, @acc[0]
	mov	@acc[1], %rax
	mov	%rdx, @acc[1]
___
for($i=1; $i<7; $i++) {
$code.=<<___;
	mulq	$g0
	add	%rax, @acc[$i]
	mov	@acc[$i+1], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[$i+1]
___
}
$code.=<<___;
	imulq	$g0
	add	%rax, @acc[7]
	adc	\$0, %rdx		# used in the final step

	mov	%rbp, %rbx
	sar	\$63, %rbp		# sign extension

	add	8*0($out_ptr), @acc[0]	# accumulate |u|*|f0|
	adc	8*1($out_ptr), @acc[1]
	adc	8*2($out_ptr), @acc[2]
	adc	8*3($out_ptr), @acc[3]
	adc	%rbx, @acc[4]
	adc	%rbp, @acc[5]
	adc	%rbp, @acc[6]
	adc	%rbp, @acc[7]

	mov	@acc[0], 8*0($out_ptr)
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)
	mov	@acc[4], 8*4($out_ptr)
	mov	@acc[5], 8*5($out_ptr)
	mov	@acc[6], 8*6($out_ptr)
	mov	@acc[7], 8*7($out_ptr)

	ret
.size	__smulq_512x63,.-__smulq_512x63

.type	__smulq_256x63,\@abi-omnipotent
.align	32
__smulq_256x63:
___
for($j=0; $j<2; $j++) {
my $k = 8*5*$j;
my @acc=@acc;	@acc=@acc[4..7]	if($j);
my $top="%rbp";	$top=$g0	if($j);
$code.=<<___;
	mov	$k+8*0($in_ptr), @acc[0] # load |u| (or |v|)
	mov	$k+8*1($in_ptr), @acc[1]
	mov	$k+8*2($in_ptr), @acc[2]
	mov	$k+8*3($in_ptr), @acc[3]
	mov	$k+8*4($in_ptr), $top	# sign/excess limb

	mov	$f0, %rbx
	sar	\$63, $f0		# |f0|'s sign as mask (or |g0|'s)
	xor	%rax, %rax
	sub	$f0, %rax		# |f0|'s sign as bit (or |g0|'s)

	xor	$f0, %rbx		# conditionally negate |f0|
	add	%rax, %rbx

	xor	$f0, @acc[0]		# conditionally negate |u| (or |v|)
	xor	$f0, @acc[1]
	xor	$f0, @acc[2]
	xor	$f0, @acc[3]
	xor	$f0, $top
	add	@acc[0], %rax
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]
	adc	\$0, $top

	mulq	%rbx
	mov	%rax, @acc[0]
	mov	@acc[1], %rax
	mov	%rdx, @acc[1]
___
for($i=1; $i<3; $i++) {
$code.=<<___;
	mulq	%rbx
	add	%rax, @acc[$i]
	mov	@acc[$i+1], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[$i+1]
___
}
$code.=<<___;
	and	%rbx, $top
	neg	$top
	mulq	%rbx
	add	%rax, @acc[3]
	adc	%rdx, $top
___
$code.=<<___	if ($j==0);
	mov	$g0, $f0
___
}
$code.=<<___;
	add	@acc[4], @acc[0]	# accumulate |u|*|f0|
	adc	@acc[5], @acc[1]
	adc	@acc[6], @acc[2]
	adc	@acc[7], @acc[3]
	adc	%rcx, %rbp

	mov	@acc[0], 8*0($out_ptr)
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)
	mov	%rbp,    8*4($out_ptr)

	ret
.size	__smulq_256x63,.-__smulq_256x63
___
########################################################################
# Signed abs(|a|*|f?|+|b|*|g?|)>>k subroutines. "NNN" in the middle of
# the names refers to maximum bit-lengths of |a| and |b|. As already
# mentioned, |f?| and |g?| can be viewed as 63 bits wide, but are always
# chosen so that "bad things" don't happen. For example, so that the
# sum of the products doesn't overflow, and that the final result is
# never wider than inputs...
{
$code.=<<___;
.type	__smulq_256_n_shift_by_31,\@abi-omnipotent
.align	32
__smulq_256_n_shift_by_31:
	mov	$f0, 8*0($out_ptr)	# offload |f0|
	mov	$g0, 8*1($out_ptr)	# offload |g0|
	mov	$f0, %rbp
___
for($j=0; $j<2; $j++) {
my $k = 8*4*$j;
my @acc=@acc;	@acc=@acc[4..7] if ($j);
my $f0="%rbp";	$f0=$g0		if ($j);
$code.=<<___;
	mov	$k+8*0($in_ptr), @acc[0] # load |a| (or |b|)
	mov	$k+8*1($in_ptr), @acc[1]
	mov	$k+8*2($in_ptr), @acc[2]
	mov	$k+8*3($in_ptr), @acc[3]

	mov	$f0, %rbx
	sar	\$63, $f0		# |f0|'s sign as mask (or |g0|'s)
	xor	%rax, %rax
	sub	$f0, %rax		# |f0|'s sign as bit (or |g0|'s)

	xor	$f0, %rbx		# conditionally negate |f0| (or |g0|)
	add	%rax, %rbx

	xor	$f0, @acc[0]		# conditionally negate |a| (or |b|)
	xor	$f0, @acc[1]
	xor	$f0, @acc[2]
	xor	$f0, @acc[3]
	add	@acc[0], %rax
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]

	mulq	%rbx
	mov	%rax, @acc[0]
	mov	@acc[1], %rax
	and	%rbx, $f0
	neg	$f0
	mov	%rdx, @acc[1]
___
for($i=1; $i<3; $i++) {
$code.=<<___;
	mulq	%rbx
	add	%rax, @acc[$i]
	mov	@acc[$i+1], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[$i+1]
___
}
$code.=<<___;
	mulq	%rbx
	add	%rax, @acc[3]
	adc	%rdx, $f0
___
}
$code.=<<___;
	add	@acc[4], @acc[0]
	adc	@acc[5], @acc[1]
	adc	@acc[6], @acc[2]
	adc	@acc[7], @acc[3]
	adc	$g0, %rbp

	mov	8*0($out_ptr), $f0	# restore original |f0|
	mov	8*1($out_ptr), $g0	# restore original |g0|

	shrd	\$31, @acc[1], @acc[0]
	shrd	\$31, @acc[2], @acc[1]
	shrd	\$31, @acc[3], @acc[2]
	shrd	\$31, %rbp,    @acc[3]

	sar	\$63, %rbp		# sign as mask
	xor	%rax, %rax
	sub	%rbp, %rax		# sign as bit

	xor	%rbp, @acc[0]		# conditionally negate the result
	xor	%rbp, @acc[1]
	xor	%rbp, @acc[2]
	xor	%rbp, @acc[3]
	add	%rax, @acc[0]
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]

	mov	@acc[0], 8*0($out_ptr)
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)

	xor	%rbp, $f0		# conditionally negate |f0|
	xor	%rbp, $g0		# conditionally negate |g0|
	add	%rax, $f0
	add	%rax, $g0

	ret
.size	__smulq_256_n_shift_by_31,.-__smulq_256_n_shift_by_31
___
}

{
my ($a_lo, $a_hi, $b_lo, $b_hi) = map("%r$_",(8..11));
my ($t0, $t1, $t2, $t3, $t4) = ("%rax","%rbx","%rbp","%r14","%r15");
my ($fg0, $fg1, $bias) = ($g0, $g1, $t4);
my ($a_, $b_) = ($a_lo, $b_lo);
{
my @a = ($a_lo, $t1, $a_hi);
my @b = ($b_lo, $t2, $b_hi);

$code.=<<___;
.type	__ab_approximation_31_256,\@abi-omnipotent
.align	32
__ab_approximation_31_256:
	mov	8*3($in_ptr), @a[2]	# load |a| in reverse order
	mov	8*7($in_ptr), @b[2]	# load |b| in reverse order
	mov	8*2($in_ptr), @a[1]
	mov	8*6($in_ptr), @b[1]
	mov	8*1($in_ptr), @a[0]
	mov	8*5($in_ptr), @b[0]

	mov	@a[2], $t0
	or	@b[2], $t0		# check top-most limbs, ...
	cmovz	@a[1], @a[2]
	cmovz	@b[1], @b[2]
	cmovz	@a[0], @a[1]
	mov	8*0($in_ptr), @a[0]
	cmovz	@b[0], @b[1]
	mov	8*4($in_ptr), @b[0]

	mov	@a[2], $t0
	or	@b[2], $t0		# ... and ones before that ...
	cmovz	@a[1], @a[2]
	cmovz	@b[1], @b[2]
	cmovz	@a[0], @a[1]
	cmovz	@b[0], @b[1]

	mov	@a[2], $t0
	or	@b[2], $t0
	bsr	$t0, %rcx
	lea	1(%rcx), %rcx
	cmovz	@a[0], @a[2]
	cmovz	@b[0], @b[2]
	cmovz	$t0, %rcx
	neg	%rcx
	#and	\$63, %rcx		# debugging artefact

	shldq	%cl, @a[1], @a[2]	# align second limb to the left
	shldq	%cl, @b[1], @b[2]

	mov	\$0x7FFFFFFF, %eax
	and	%rax, @a[0]
	and	%rax, @b[0]
	not	%rax
	and	%rax, @a[2]
	and	%rax, @b[2]
	or	@a[2], @a[0]
	or	@b[2], @b[0]

	jmp	__inner_loop_31_256

	ret
.size	__ab_approximation_31_256,.-__ab_approximation_31_256
___
}
$code.=<<___;
.type	__inner_loop_31_256,\@abi-omnipotent
.align	32			# comment and punish Coffee Lake by up to 40%
__inner_loop_31_256:		################# by Thomas Pornin
	mov	\$0x7FFFFFFF80000000, $fg0	# |f0|=1, |g0|=0
	mov	\$0x800000007FFFFFFF, $fg1	# |f1|=0, |g1|=1
	mov	\$0x7FFFFFFF7FFFFFFF, $bias

.Loop_31_256:
	cmp	$b_, $a_		# if |a_|<|b_|, swap the variables
	mov	$a_, $t0
	mov	$b_, $t1
	mov	$fg0, $t2
	mov	$fg1, $t3
	cmovb	$b_, $a_
	cmovb	$t0, $b_
	cmovb	$fg1, $fg0
	cmovb	$t2, $fg1

	sub	$b_, $a_		# |a_|-|b_|
	sub	$fg1, $fg0		# |f0|-|f1|, |g0|-|g1|
	add	$bias, $fg0

	test	\$1, $t0		# if |a_| was even, roll back 
	cmovz	$t0, $a_
	cmovz	$t1, $b_
	cmovz	$t2, $fg0
	cmovz	$t3, $fg1

	shr	\$1, $a_		# |a_|>>=1
	add	$fg1, $fg1		# |f1|<<=1, |g1|<<=1
	sub	$bias, $fg1
	sub	\$1, $cnt
	jnz	.Loop_31_256

	shr	\$32, $bias
	mov	%ecx, %edx		# $fg0, $f0
	mov	${fg1}d, ${f1}d
	shr	\$32, $g0
	shr	\$32, $g1
	sub	$bias, $f0		# remove the bias
	sub	$bias, $g0
	sub	$bias, $f1
	sub	$bias, $g1

	ret
.size	__inner_loop_31_256,.-__inner_loop_31_256

.type	__inner_loop_62_256,\@abi-omnipotent
.align	32
__inner_loop_62_256:
	mov	$cnt, %r15d
	mov	\$1, $f0	# |f0|=1
	xor	$g0, $g0	# |g0|=0
	xor	$f1, $f1	# |f1|=0
	mov	$f0, $g1	# |g1|=1
	mov	$f0, %r14

.Loop_62_256:
	xor	$t0, $t0
	test	%r14, $a_lo	# if |a_| is odd, then we'll be subtracting |b_|
	mov	$b_lo, $t1
	cmovnz	$b_lo, $t0
	sub	$a_lo, $t1	# |b_|-|a_|
	mov	$a_lo, $t2
	sub	$t0, $a_lo	# |a_|-|b_| (or |a_|-0 if |a_| was even)
	cmovc	$t1, $a_lo	# borrow means |a_|<|b_|, replace with |b_|-|a_|
	cmovc	$t2, $b_lo	# |b_| = |a_|
	mov	$f0, $t0	# exchange |f0| and |f1|
	cmovc	$f1, $f0
	cmovc	$t0, $f1
	mov	$g0, $t1	# exchange |g0| and |g1|
	cmovc	$g1, $g0
	cmovc	$t1, $g1
	xor	$t0, $t0
	xor	$t1, $t1
	shr	\$1, $a_lo
	test	%r14, $t2	# if |a_| was odd, then we'll be subtracting...
	cmovnz	$f1, $t0
	cmovnz	$g1, $t1
	add	$f1, $f1	# |f1|<<=1
	add	$g1, $g1	# |g1|<<=1
	sub	$t0, $f0	# |f0|-=|f1| (or |f0-=0| if |a_| was even)
	sub	$t1, $g0	# |g0|-=|g1| (or |g0-=0| ...)
	sub	\$1, %r15d
	jnz	.Loop_62_256

	ret
.size	__inner_loop_62_256,.-__inner_loop_62_256
___
}

print $code;
close STDOUT;
