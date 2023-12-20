#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
#
# Both constant-time and fast Euclidean inversion as suggested in
# https://eprint.iacr.org/2020/972. Performance is >4x better than
# modulus-specific FLT addition chain...
#
# void ct_inverse_mod_383(vec768 ret, const vec384 inp, const vec384 mod);
#
$python_ref.=<<'___';
def ct_inverse_mod_383(inp, mod):
    a, u = inp, 1
    b, v = mod, 0

    k = 31
    mask = (1 << k) - 1

    for i in range(0, 766 // k):
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

        # __smulx_383_n_shift_by_31
        a, b = (a*f0 + b*g0) >> k, (a*f1 + b*g1) >> k
        if a < 0:
            a, f0, g0 = -a, -f0, -g0
        if b < 0:
            b, f1, g1 = -b, -f1, -g1

        # __smulx_767x63
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
if ($flavour =~ /\./) { $output = $flavour; undef $flavour; }

$win64=0; $win64=1 if ($flavour =~ /[nm]asm|mingw64/ || $output =~ /\.asm$/);

$0 =~ m/(.*[\/\\])[^\/\\]+$/; $dir=$1;
( $xlate="${dir}x86_64-xlate.pl" and -f $xlate ) or
( $xlate="${dir}../../perlasm/x86_64-xlate.pl" and -f $xlate) or
die "can't locate x86_64-xlate.pl";

open STDOUT,"| \"$^X\" \"$xlate\" $flavour \"$output\""
    or die "can't call $xlate: $!";

my ($out_ptr, $in_ptr, $n_ptr, $nx_ptr) = ("%rdi", "%rsi", "%rdx", "%rcx");
my @acc=(map("%r$_",(8..15)), "%rbx", "%rbp", $in_ptr, $out_ptr);
my ($f0, $g0, $f1, $g1) = ("%rdx","%rcx","%r12","%r13");
my $cnt = "%edi";

$frame = 8*11+2*512;

$code.=<<___;
.text

.globl	ctx_inverse_mod_383
.type	ctx_inverse_mod_383,\@function,4,"unwind"
.align	32
ctx_inverse_mod_383:
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

	lea	8*11+511(%rsp), %rax	# find closest 512-byte-aligned spot
	and	\$-512, %rax		# in the frame...
	mov	$out_ptr, 8*4(%rsp)
	mov	$nx_ptr, 8*5(%rsp)

	mov	8*0($in_ptr), @acc[0]	# load input
	mov	8*1($in_ptr), @acc[1]
	mov	8*2($in_ptr), @acc[2]
	mov	8*3($in_ptr), @acc[3]
	mov	8*4($in_ptr), @acc[4]
	mov	8*5($in_ptr), @acc[5]

	mov	8*0($n_ptr), @acc[6]	# load modulus
	mov	8*1($n_ptr), @acc[7]
	mov	8*2($n_ptr), @acc[8]
	mov	8*3($n_ptr), @acc[9]
	mov	8*4($n_ptr), @acc[10]
	mov	8*5($n_ptr), @acc[11]

	mov	@acc[0], 8*0(%rax)	# copy input to |a|
	mov	@acc[1], 8*1(%rax)
	mov	@acc[2], 8*2(%rax)
	mov	@acc[3], 8*3(%rax)
	mov	@acc[4], 8*4(%rax)
	mov	@acc[5], 8*5(%rax)

	mov	@acc[6], 8*6(%rax)	# copy modulus to |b|
	mov	@acc[7], 8*7(%rax)
	mov	@acc[8], 8*8(%rax)
	mov	@acc[9], 8*9(%rax)
	mov	@acc[10], 8*10(%rax)
	mov	%rax, $in_ptr
	mov	@acc[11], 8*11(%rax)

	################################# first iteration
	mov	\$31, $cnt
	call	__ab_approximation_31
	#mov	$f0, 8*7(%rsp)
	#mov	$g0, 8*8(%rsp)
	mov	$f1, 8*9(%rsp)
	mov	$g1, 8*10(%rsp)

	mov	\$256, $out_ptr
	xor	$in_ptr, $out_ptr	# pointer to destination |a|b|u|v|
	call	__smulx_383_n_shift_by_31
	#mov	$f0, 8*7(%rsp)		# corrected |f0|
	#mov	$g0, 8*8(%rsp)		# corrected |g0|
	mov	$f0, 8*12($out_ptr)	# initialize |u| with |f0|

	mov	8*9(%rsp), $f0		# |f1|
	mov	8*10(%rsp), $g0		# |g1|
	lea	8*6($out_ptr), $out_ptr	# pointer to destination |b|
	call	__smulx_383_n_shift_by_31
	#mov	$f0, 8*9(%rsp)		# corrected |f1|
	#mov	$g0, 8*10(%rsp)		# corrected |g1|
	mov	$f0, 8*12($out_ptr)	# initialize |v| with |f1|

	################################# second iteration
	xor	\$256, $in_ptr		# flip-flop pointer to source |a|b|u|v|
	mov	\$31, $cnt
	call	__ab_approximation_31
	#mov	$f0, 8*7(%rsp)
	#mov	$g0, 8*8(%rsp)
	mov	$f1, 8*9(%rsp)
	mov	$g1, 8*10(%rsp)

	mov	\$256, $out_ptr
	xor	$in_ptr, $out_ptr	# pointer to destination |a|b|u|v|
	call	__smulx_383_n_shift_by_31
	mov	$f0, 8*7(%rsp)		# corrected |f0|
	mov	$g0, 8*8(%rsp)		# corrected |g0|

	mov	8*9(%rsp), $f0		# |f1|
	mov	8*10(%rsp), $g0		# |g1|
	lea	8*6($out_ptr), $out_ptr	# pointer to destination |b|
	call	__smulx_383_n_shift_by_31
	#mov	$f0, 8*9(%rsp)		# corrected |f1|
	#mov	$g0, 8*10(%rsp)		# corrected |g1|

	mov	8*12($in_ptr), %rax	# |u|
	mov	8*18($in_ptr), @acc[3]	# |v|
	mov	$f0, %rbx
	mov	%rax, @acc[2]
	imulq	8*7(%rsp)		# |u|*|f0|
	mov	%rax, @acc[0]
	mov	@acc[3], %rax
	mov	%rdx, @acc[1]
	imulq	8*8(%rsp)		# |v|*|g0|
	add	%rax, @acc[0]
	adc	%rdx, @acc[1]
	mov	@acc[0], 8*6($out_ptr)	# destination |u|
	mov	@acc[1], 8*7($out_ptr)
	sar	\$63, @acc[1]		# sign extension
	mov	@acc[1], 8*8($out_ptr)
	mov	@acc[1], 8*9($out_ptr)
	mov	@acc[1], 8*10($out_ptr)
	mov	@acc[1], 8*11($out_ptr)
	lea	8*12($in_ptr), $in_ptr	# make in_ptr "rewindable" with xor

	mov	@acc[2], %rax
	imulq	%rbx			# |u|*|f1|
	mov	%rax, @acc[0]
	mov	@acc[3], %rax
	mov	%rdx, @acc[1]
	imulq	%rcx			# |v|*|g1|
	add	%rax, @acc[0]
	adc	%rdx, @acc[1]
	mov	@acc[0], 8*12($out_ptr)	# destination |v|
	mov	@acc[1], 8*13($out_ptr)
	sar	\$63, @acc[1]		# sign extension
	mov	@acc[1], 8*14($out_ptr)
	mov	@acc[1], 8*15($out_ptr)
	mov	@acc[1], 8*16($out_ptr)
	mov	@acc[1], 8*17($out_ptr)
___
for($i=2; $i<23; $i++) {
my $smul_n_shift = $i<19 ? "__smulx_383_n_shift_by_31"
                         : "__smulx_191_n_shift_by_31";
my $smul_767x63  = $i>11 ? "__smulx_767x63"
                         : "__smulx_383x63";
$code.=<<___;
	xor	\$256+8*12, $in_ptr	# flip-flop pointer to source |a|b|u|v|
	mov	\$31, $cnt
	call	__ab_approximation_31
	#mov	$f0, 8*7(%rsp)
	#mov	$g0, 8*8(%rsp)
	mov	$f1, 8*9(%rsp)
	mov	$g1, 8*10(%rsp)

	mov	\$256, $out_ptr
	xor	$in_ptr, $out_ptr	# pointer to destination |a|b|u|v|
	call	$smul_n_shift
	mov	$f0, 8*7(%rsp)		# corrected |f0|
	mov	$g0, 8*8(%rsp)		# corrected |g0|

	mov	8*9(%rsp), $f0		# |f1|
	mov	8*10(%rsp), $g0		# |g1|
	lea	8*6($out_ptr), $out_ptr	# pointer to destination |b|
	call	$smul_n_shift
	mov	$f0, 8*9(%rsp)		# corrected |f1|
	mov	$g0, 8*10(%rsp)		# corrected |g1|

	mov	8*7(%rsp), $f0		# |f0|
	mov	8*8(%rsp), $g0		# |g0|
	lea	8*12($in_ptr), $in_ptr	# pointer to source |u|v|
	lea	8*6($out_ptr), $out_ptr	# pointer to destination |u|
	call	__smulx_383x63

	mov	8*9(%rsp), $f0		# |f1|
	mov	8*10(%rsp), $g0		# |g1|
	lea	8*6($out_ptr),$out_ptr	# pointer to destination |v|
	call	$smul_767x63
___
$code.=<<___	if ($i==11);
	sar	\$63, @acc[5]		# sign extension
	mov	@acc[5], 8*6($out_ptr)
	mov	@acc[5], 8*7($out_ptr)
	mov	@acc[5], 8*8($out_ptr)
	mov	@acc[5], 8*9($out_ptr)
	mov	@acc[5], 8*10($out_ptr)
	mov	@acc[5], 8*11($out_ptr)
___
}
$code.=<<___;
	################################# two[!] last iterations in one go
	xor	\$256+8*12, $in_ptr	# flip-flop pointer to source |a|b|u|v|
	mov	\$53, $cnt		# 31 + 766 % 31
	#call	__ab_approximation_31	# |a| and |b| are exact, just load
	mov	8*0($in_ptr), @acc[0]	# |a_lo|
	#xor	@acc[1],      @acc[1]	# |a_hi|
	mov	8*6($in_ptr), @acc[2]	# |b_lo|
	#xor	@acc[3],      @acc[3]	# |b_hi|
	call	__inner_loop_62
	#mov	$f0, 8*7(%rsp)
	#mov	$g0, 8*8(%rsp)
	#mov	$f1, 8*9(%rsp)
	#mov	$g1, 8*10(%rsp)

	#mov	8*7(%rsp), $f0		# |f0|
	#mov	8*8(%rsp), $g0		# |g0|
	lea	8*12($in_ptr), $in_ptr	# pointer to source |u|v|
	#lea	8*6($out_ptr), $out_ptr	# pointer to destination |u|
	#call	__smulx_383x63

	#mov	8*9(%rsp), $f0		# |f1|
	#mov	8*10(%rsp), $g0		# |g1|
	mov	$f1, $f0
	mov	$g1, $g0
	mov	8*4(%rsp), $out_ptr	# original out_ptr
	call	__smulx_767x63

	mov	8*5(%rsp), $in_ptr	# original n_ptr
	mov	%rax, %rdx		# top limb of the result
	sar	\$63, %rax		# result's sign as mask

	mov	%rax, @acc[0]		# mask |modulus|
	mov	%rax, @acc[1]
	mov	%rax, @acc[2]
	and	8*0($in_ptr), @acc[0]
	and	8*1($in_ptr), @acc[1]
	mov	%rax, @acc[3]
	and	8*2($in_ptr), @acc[2]
	and	8*3($in_ptr), @acc[3]
	mov	%rax, @acc[4]
	and	8*4($in_ptr), @acc[4]
	and	8*5($in_ptr), %rax

	add	@acc[0], @acc[6]	# conditionally add |modulus|<<384
	adc	@acc[1], @acc[7]
	adc	@acc[2], @acc[8]
	adc	@acc[3], @acc[9]
	adc	@acc[4], %rcx
	adc	%rax,    %rdx

	mov	@acc[6], 8*6($out_ptr)	# store absolute value
	mov	@acc[7], 8*7($out_ptr)
	mov	@acc[8], 8*8($out_ptr)
	mov	@acc[9], 8*9($out_ptr)
	mov	%rcx,    8*10($out_ptr)
	mov	%rdx,    8*11($out_ptr)

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
.size	ctx_inverse_mod_383,.-ctx_inverse_mod_383
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
{
my ($out_ptr, $in_ptr, $f0, $g0) = ("%rdi", "%rsi", "%rdx", "%rcx");
my @acc = map("%r$_",(8..15),"bx","bp","cx","di");
my $fx = @acc[9];

$code.=<<___;
.type	__smulx_767x63,\@abi-omnipotent
.align	32
__smulx_767x63:
	mov	8*0($in_ptr), @acc[0]	# load |u|
	mov	8*1($in_ptr), @acc[1]
	mov	8*2($in_ptr), @acc[2]
	mov	8*3($in_ptr), @acc[3]
	mov	8*4($in_ptr), @acc[4]
	mov	8*5($in_ptr), @acc[5]

	mov	$f0, %rax
	sar	\$63, %rax		# |f0|'s sign as mask
	xor	$fx, $fx		# overrides in_ptr
	sub	%rax, $fx		# |f0|'s sign as bit

	mov	$out_ptr, 8*1(%rsp)
	mov	$in_ptr,  8*2(%rsp)
	lea	8*6($in_ptr), $in_ptr	# pointer to |v|

	xor	%rax, $f0		# conditionally negate |f0|
	add	$fx, $f0

	xor	%rax, @acc[0]		# conditionally negate |u|
	xor	%rax, @acc[1]
	xor	%rax, @acc[2]
	xor	%rax, @acc[3]
	xor	%rax, @acc[4]
	xor	@acc[5], %rax
	add	$fx, @acc[0]
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]
	adc	\$0, @acc[4]
	adc	\$0, %rax

	mulx	@acc[0], @acc[0], $fx	# |u|*|f0|
	mulx	@acc[1], @acc[1], @acc[5]
	add	$fx, @acc[1]
___
for(my ($a,$b) = ($fx, @acc[5]), $i=2; $i<5; $i++) {
$code.=<<___;
	mulx	@acc[$i], @acc[$i], $a
	adc	$b, @acc[$i]
___
    ($a, $b) = ($b, $a);
}
$code.=<<___;
	adc	\$0, $fx
	imulq	%rdx
	add	$fx, %rax
	adc	\$0, %rdx

	mov	@acc[0], 8*0($out_ptr)	# offload |u|*|f0|
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)
	mov	@acc[4], 8*4($out_ptr)
	mov	%rax,    8*5($out_ptr)
	mov	%rdx,    8*6($out_ptr)
	sar	\$63, %rdx		# sign extension
	mov	%rdx, 8*7($out_ptr)
___
{
my $fx=$in_ptr;
$code.=<<___;
	mov	$g0, $f0		# load |g0|
	mov	$g0, %rax

	mov	8*0($in_ptr), @acc[0]	# load |v|
	mov	8*1($in_ptr), @acc[1]
	mov	8*2($in_ptr), @acc[2]
	mov	8*3($in_ptr), @acc[3]
	mov	8*4($in_ptr), @acc[4]
	mov	8*5($in_ptr), @acc[5]
	mov	8*6($in_ptr), @acc[6]
	mov	8*7($in_ptr), @acc[7]
	mov	8*8($in_ptr), @acc[8]
	mov	8*9($in_ptr), @acc[9]
	mov	8*10($in_ptr), @acc[10]
	mov	8*11($in_ptr), @acc[11]

	sar	\$63, %rax		# |g0|'s sign as mask
	xor	$fx, $fx		# overrides in_ptr
	sub	%rax, $fx		# |g0|'s sign as bit

	xor	%rax, $f0		# conditionally negate |g0|
	add	$fx, $f0

	xor	%rax, @acc[0]		# conditionally negate |v|
	xor	%rax, @acc[1]
	xor	%rax, @acc[2]
	xor	%rax, @acc[3]
	xor	%rax, @acc[4]
	xor	%rax, @acc[5]
	xor	%rax, @acc[6]
	xor	%rax, @acc[7]
	xor	%rax, @acc[8]
	xor	%rax, @acc[9]
	xor	%rax, @acc[10]
	xor	%rax, @acc[11]
	add	$fx, @acc[0]
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]
	adc	\$0, @acc[4]
	adc	\$0, @acc[5]
	adc	\$0, @acc[6]
	adc	\$0, @acc[7]
	adc	\$0, @acc[8]
	adc	\$0, @acc[9]
	adc	\$0, @acc[10]
	adc	\$0, @acc[11]

	mulx	@acc[0], @acc[0], %rax	# |v|*|g0|
	mulx	@acc[1], @acc[1], $fx
	add	%rax, @acc[1]
___
for(my ($a,$b) = ("%rax", $fx), $i=2; $i<11; $i++) {
$code.=<<___;
	mulx	@acc[$i], @acc[$i], $a
	adc	$b, @acc[$i]
___
    ($a, $b) = ($b, $a);
}
$code.=<<___;
	mulx	@acc[11], @acc[11], $fx
	mov	8*1(%rsp), %rdx		# out_ptr
	mov	8*2(%rsp), $in_ptr	# restore original in_ptr
	adc	@acc[11], %rax

	add	8*0(%rdx), @acc[0]	# accumulate |u|*|f0|
	adc	8*1(%rdx), @acc[1]
	adc	8*2(%rdx), @acc[2]
	adc	8*3(%rdx), @acc[3]
	adc	8*4(%rdx), @acc[4]
	adc	8*5(%rdx), @acc[5]
	adc	8*6(%rdx), @acc[6]
	mov	8*7(%rdx), @acc[11]	# sign extension
	adc	@acc[11], @acc[7]
	adc	@acc[11], @acc[8]
	adc	@acc[11], @acc[9]
	adc	@acc[11], @acc[10]
	adc	@acc[11], %rax

	mov	%rdx, $out_ptr		# restore original out_ptr

	mov	@acc[0], 8*0(%rdx)
	mov	@acc[1], 8*1(%rdx)
	mov	@acc[2], 8*2(%rdx)
	mov	@acc[3], 8*3(%rdx)
	mov	@acc[4], 8*4(%rdx)
	mov	@acc[5], 8*5(%rdx)
	mov	@acc[6], 8*6(%rdx)
	mov	@acc[7], 8*7(%rdx)
	mov	@acc[8], 8*8(%rdx)
	mov	@acc[9], 8*9(%rdx)
	mov	@acc[10], 8*10(%rdx)
	mov	%rax,     8*11(%rdx)

	ret
.size	__smulx_767x63,.-__smulx_767x63
___
}
$code.=<<___;
.type	__smulx_383x63,\@abi-omnipotent
.align	32
__smulx_383x63:
___
for($j=0; $j<2; $j++) {
my $k = 8*6*$j;
$code.=<<___;
	mov	$k+8*0($in_ptr), @acc[0] # load |u| (or |v|)
	mov	$k+8*1($in_ptr), @acc[1]
	mov	$k+8*2($in_ptr), @acc[2]
	mov	$k+8*3($in_ptr), @acc[3]
	mov	$k+8*4($in_ptr), @acc[4]
	mov	$k+8*5($in_ptr), @acc[5]

	mov	$f0, $fx
	sar	\$63, $fx		# |f0|'s sign as mask (or |g0|'s)
	xor	%rax, %rax
	sub	$fx, %rax		# |f0|'s sign as bit (or |g0|'s)

	xor	$fx, $f0		# conditionally negate |f0|
	add	%rax, $f0

	xor	$fx, @acc[0]		# conditionally negate |u| (or |v|)
	xor	$fx, @acc[1]
	xor	$fx, @acc[2]
	xor	$fx, @acc[3]
	xor	$fx, @acc[4]
	xor	$fx, @acc[5]
	add	%rax, @acc[0]
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]
	adc	\$0, @acc[4]
	adc	\$0, @acc[5]

	mulx	@acc[0], @acc[0], $fx	# |u|*|f0| (or |v|*|g0|)
	mulx	@acc[1], @acc[1], %rax
	add	$fx, @acc[1]
___
for(my ($a,$b) = ($fx, "%rax"), $i=2; $i<5; $i++) {
$code.=<<___;
	mulx	@acc[$i], @acc[$i], $a
	adc	$b, @acc[$i]
___
    ($a, $b) = ($b, $a);
}
$code.=<<___	if ($j==0);
	mulx	@acc[$i], @acc[$i], %rax
	mov	$g0, $f0
	adc	$fx, @acc[$i]

	mov	@acc[0], 8*0($out_ptr)	# offload |u|*|f0|
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)
	mov	@acc[4], 8*4($out_ptr)
	mov	@acc[5], 8*5($out_ptr)
___
}
$code.=<<___;
	mulx	@acc[$i], @acc[$i], %rax
	adc	$fx, @acc[$i]

	add	8*0($out_ptr), @acc[0]	# accumulate |u|*|f0|
	adc	8*1($out_ptr), @acc[1]
	adc	8*2($out_ptr), @acc[2]
	adc	8*3($out_ptr), @acc[3]
	adc	8*4($out_ptr), @acc[4]
	adc	8*5($out_ptr), @acc[5]

	mov	@acc[0], 8*0($out_ptr)
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)
	mov	@acc[4], 8*4($out_ptr)
	mov	@acc[5], 8*5($out_ptr)

	ret
.size	__smulx_383x63,.-__smulx_383x63
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
.type	__smulx_383_n_shift_by_31,\@abi-omnipotent
.align	32
__smulx_383_n_shift_by_31:
	mov	$f0, @acc[8]
	xor	@acc[6], @acc[6]
___
my $f0 = @acc[8];
for($j=0; $j<2; $j++) {
my $k = 8*6*$j;
$code.=<<___;
	mov	$k+8*0($in_ptr), @acc[0] # load |a| (or |b|)
	mov	$k+8*1($in_ptr), @acc[1]
	mov	$k+8*2($in_ptr), @acc[2]
	mov	$k+8*3($in_ptr), @acc[3]
	mov	$k+8*4($in_ptr), @acc[4]
	mov	$k+8*5($in_ptr), @acc[5]

	mov	%rdx, %rax
	sar	\$63, %rax		# |f0|'s sign as mask (or |g0|'s)
	xor	$fx, $fx
	sub	%rax, $fx		# |f0|'s sign as bit (or |g0|'s)

	xor	%rax, %rdx		# conditionally negate |f0| (or |g0|)
	add	$fx, %rdx

	xor	%rax, @acc[0]		# conditionally negate |a| (or |b|)
	xor	%rax, @acc[1]
	xor	%rax, @acc[2]
	xor	%rax, @acc[3]
	xor	%rax, @acc[4]
	xor	@acc[5], %rax
	add	$fx, @acc[0]
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]
	adc	\$0, @acc[4]
	adc	\$0, %rax

	mulx	@acc[0], @acc[0], $fx	# |a|*|f0| (or |b|*|g0|)
	mulx	@acc[1], @acc[1], @acc[5]
	add	$fx, @acc[1]
___
for(my ($a,$b) = ($fx, @acc[5]), $i=2; $i<5; $i++) {
$code.=<<___;
	mulx	@acc[$i], @acc[$i], $a
	adc	$b, @acc[$i]
___
    ($a, $b) = ($b, $a);
}
$code.=<<___	if ($j==0);
	adc	\$0, $fx
	imulq	%rdx
	add	$fx, %rax
	adc	%rdx, @acc[6]

	mov	$g0, %rdx

	mov	@acc[0], 8*0($out_ptr)
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)
	mov	@acc[4], 8*4($out_ptr)
	mov	%rax,    8*5($out_ptr)
___
}
$code.=<<___;
	adc	\$0, $fx
	imulq	%rdx
	add	$fx, %rax
	adc	\$0, %rdx

	add	8*0($out_ptr), @acc[0]
	adc	8*1($out_ptr), @acc[1]
	adc	8*2($out_ptr), @acc[2]
	adc	8*3($out_ptr), @acc[3]
	adc	8*4($out_ptr), @acc[4]
	adc	8*5($out_ptr), %rax
	adc	%rdx,          @acc[6]
	mov	$f0, %rdx

	shrd	\$31, @acc[1], @acc[0]
	shrd	\$31, @acc[2], @acc[1]
	shrd	\$31, @acc[3], @acc[2]
	shrd	\$31, @acc[4], @acc[3]
	shrd	\$31, %rax,    @acc[4]
	shrd	\$31, @acc[6], %rax

	sar	\$63, @acc[6]		# sign as mask
	xor	$fx, $fx
	sub	@acc[6], $fx		# sign as bit

	xor	@acc[6], @acc[0]	# conditionally negate the result
	xor	@acc[6], @acc[1]
	xor	@acc[6], @acc[2]
	xor	@acc[6], @acc[3]
	xor	@acc[6], @acc[4]
	xor	@acc[6], %rax
	add	$fx, @acc[0]
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]
	adc	\$0, @acc[4]
	adc	\$0, %rax

	mov	@acc[0], 8*0($out_ptr)
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)
	mov	@acc[4], 8*4($out_ptr)
	mov	%rax,    8*5($out_ptr)

	xor	@acc[6], %rdx		# conditionally negate |f0|
	xor	@acc[6], $g0		# conditionally negate |g0|
	add	$fx, %rdx
	add	$fx, $g0

	ret
.size	__smulx_383_n_shift_by_31,.-__smulx_383_n_shift_by_31
___
} {
$code.=<<___;
.type	__smulx_191_n_shift_by_31,\@abi-omnipotent
.align	32
__smulx_191_n_shift_by_31:
	mov	$f0, @acc[8]
___
my $f0 = @acc[8];
for($j=0; $j<2; $j++) {
my $k = 8*6*$j;
my @acc=@acc;
   @acc=@acc[3..5] if ($j);
$code.=<<___;
	mov	$k+8*0($in_ptr), @acc[0] # load |a| (or |b|)
	mov	$k+8*1($in_ptr), @acc[1]
	mov	$k+8*2($in_ptr), @acc[2]

	mov	%rdx, %rax
	sar	\$63, %rax		# |f0|'s sign as mask (or |g0|'s)
	xor	$fx, $fx
	sub	%rax, $fx		# |f0|'s sign as bit (or |g0|'s)

	xor	%rax, %rdx		# conditionally negate |f0| (or |g0|)
	add	$fx, %rdx

	xor	%rax, @acc[0]		# conditionally negate |a| (or |b|)
	xor	%rax, @acc[1]
	xor	@acc[2], %rax
	add	$fx, @acc[0]
	adc	\$0, @acc[1]
	adc	\$0, %rax

	mulx	@acc[0], @acc[0], $fx	# |a|*|f0| (or |b|*|g0|)
	mulx	@acc[1], @acc[1], @acc[2]
	add	$fx, @acc[1]
	adc	\$0, @acc[2]
	imulq	%rdx
	add	%rax, @acc[2]
	adc	\$0, %rdx
___
$code.=<<___	if ($j==0);
	mov	%rdx, @acc[6]
	mov	$g0, %rdx
___
}
$code.=<<___;
	add	@acc[0], @acc[3]
	adc	@acc[1], @acc[4]
	adc	@acc[2], @acc[5]
	adc	%rdx,    @acc[6]
	mov	$f0, %rdx

	shrd	\$31, @acc[4], @acc[3]
	shrd	\$31, @acc[5], @acc[4]
	shrd	\$31, @acc[6], @acc[5]

	sar	\$63, @acc[6]		# sign as mask
	xor	$fx, $fx
	sub	@acc[6], $fx		# sign as bit

	xor	@acc[6], @acc[3]	# conditionally negate the result
	xor	@acc[6], @acc[4]
	xor	@acc[6], @acc[5]
	add	$fx, @acc[3]
	adc	\$0, @acc[4]
	adc	\$0, @acc[5]

	mov	@acc[3], 8*0($out_ptr)
	mov	@acc[4], 8*1($out_ptr)
	mov	@acc[5], 8*2($out_ptr)

	xor	@acc[6], %rdx		# conditionally negate |f0|
	xor	@acc[6], $g0		# conditionally negate |g0|
	add	$fx, %rdx
	add	$fx, $g0

	ret
.size	__smulx_191_n_shift_by_31,.-__smulx_191_n_shift_by_31
___
} }

{
my ($a_lo, $a_hi, $b_lo, $b_hi) = map("%r$_",(8..11));
my ($t0, $t1, $t2, $t3, $t4) = ("%rax","%rbx","%rbp","%r14","%r15");
my ($fg0, $fg1, $bias) = ($g0, $g1, $t4);
my ($a_, $b_) = ($a_lo, $b_lo);
{
my @a = ($a_lo, $t1, $a_hi);
my @b = ($b_lo, $t2, $b_hi);

$code.=<<___;
.type	__ab_approximation_31,\@abi-omnipotent
.align	32
__ab_approximation_31:
	mov	8*5($in_ptr), @a[2]	# load |a| in reverse order
	mov	8*11($in_ptr), @b[2]	# load |b| in reverse order
	mov	8*4($in_ptr), @a[1]
	mov	8*10($in_ptr), @b[1]
	mov	8*3($in_ptr), @a[0]
	mov	8*9($in_ptr), @b[0]

	mov	@a[2], $t0
	or	@b[2], $t0		# check top-most limbs, ...
	cmovz	@a[1], @a[2]
	cmovz	@b[1], @b[2]
	cmovz	@a[0], @a[1]
	mov	8*2($in_ptr), @a[0]
	cmovz	@b[0], @b[1]
	mov	8*8($in_ptr), @b[0]

	mov	@a[2], $t0
	or	@b[2], $t0		# ... ones before top-most, ...
	cmovz	@a[1], @a[2]
	cmovz	@b[1], @b[2]
	cmovz	@a[0], @a[1]
	mov	8*1($in_ptr), @a[0]
	cmovz	@b[0], @b[1]
	mov	8*7($in_ptr), @b[0]

	mov	@a[2], $t0
	or	@b[2], $t0		# ... and ones before that ...
	cmovz	@a[1], @a[2]
	cmovz	@b[1], @b[2]
	cmovz	@a[0], @a[1]
	mov	8*0($in_ptr), @a[0]
	cmovz	@b[0], @b[1]
	mov	8*6($in_ptr), @b[0]

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
	andn	@a[2], %rax, @a[2]
	andn	@b[2], %rax, @b[2]
	or	@a[2], @a[0]
	or	@b[2], @b[0]

	jmp	__inner_loop_31

	ret
.size	__ab_approximation_31,.-__ab_approximation_31
___
}
$code.=<<___;
.type	__inner_loop_31,\@abi-omnipotent
.align	32
__inner_loop_31:		################# by Thomas Pornin
	mov	\$0x7FFFFFFF80000000, $fg0	# |f0|=1, |g0|=0
	mov	\$0x800000007FFFFFFF, $fg1	# |f1|=0, |g1|=1
	mov	\$0x7FFFFFFF7FFFFFFF, $bias

.Loop_31:
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
	jnz	.Loop_31

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
.size	__inner_loop_31,.-__inner_loop_31

.type	__inner_loop_62,\@abi-omnipotent
.align	32
__inner_loop_62:
	mov	\$1, $f0	# |f0|=1
	xor	$g0, $g0	# |g0|=0
	xor	$f1, $f1	# |f1|=0
	mov	\$1, $g1	# |g1|=1

.Loop_62:
	xor	$t0, $t0
	test	\$1, $a_lo	# if |a_| is odd, then we'll be subtracting |b_|
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
	test	\$1, $t2	# if |a_| was odd, then we'll be subtracting...
	cmovnz	$f1, $t0
	cmovnz	$g1, $t1
	add	$f1, $f1	# |f1|<<=1
	add	$g1, $g1	# |g1|<<=1
	sub	$t0, $f0	# |f0|-=|f1| (or |f0-=0| if |a_| was even)
	sub	$t1, $g0	# |g0|-=|g1| (or |g0-=0| ...)
	sub	\$1, $cnt
	jnz	.Loop_62

	ret
.size	__inner_loop_62,.-__inner_loop_62
___
}

print $code;
close STDOUT;
