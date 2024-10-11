#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
#
# Both constant-time and fast Euclidean inversion as suggested in
# https://eprint.iacr.org/2020/972. Performance is >5x better than
# modulus-specific FLT addition chain...
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

        # __smulq_383_n_shift_by_62
        a, b = (a*f0 + b*g0) >> k, (a*f1 + b*g1) >> k
        if a < 0:
            a, f0, g0 = -a, -f0, -g0
        if b < 0:
            b, f1, g1 = -b, -f1, -g1

        # __smulq_767x63
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

$code.=<<___ if ($flavour =~ /masm/);
.extern	ct_inverse_mod_383\$1
___

my ($out_ptr, $in_ptr, $n_ptr, $nx_ptr) = ("%rdi", "%rsi", "%rdx", "%rcx");
my @acc=(map("%r$_",(8..15)), "%rbx", "%rbp", $in_ptr, $out_ptr);
my ($f0, $g0, $f1, $g1) = ("%rdx","%rcx","%r12","%r13");
my $cnt = "%edi";

$frame = 8*11+2*512;

$code.=<<___;
.comm	__blst_platform_cap,4
.text

.globl	ct_inverse_mod_383
.hidden	ct_inverse_mod_383
.type	ct_inverse_mod_383,\@function,4,"unwind"
.align	32
ct_inverse_mod_383:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	ct_inverse_mod_383\$1
#endif
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
	mov	%rax, $in_ptr		# pointer to source |a|b|1|0|
	mov	@acc[11], 8*11(%rax)

	################################# first iteration
	mov	\$62, $cnt
	call	__ab_approximation_62
	#mov	$f0, 8*7(%rsp)
	#mov	$g0, 8*8(%rsp)
	mov	$f1, 8*9(%rsp)
	mov	$g1, 8*10(%rsp)

	mov	\$256, $out_ptr
	xor	$in_ptr, $out_ptr	# pointer to destination |a|b|u|v|
	call	__smulq_383_n_shift_by_62
	#mov	$f0, 8*7(%rsp)		# corrected |f0|
	#mov	$g0, 8*8(%rsp)		# corrected |g0|
	mov	$f0, 8*12($out_ptr)	# initialize |u| with |f0|

	mov	8*9(%rsp), $f0		# |f1|
	mov	8*10(%rsp), $g0		# |g1|
	lea	8*6($out_ptr), $out_ptr	# pointer to destination |b|
	call	__smulq_383_n_shift_by_62
	#mov	$f0, 8*9(%rsp)		# corrected |f1|
	#mov	$g0, 8*10(%rsp)		# corrected |g1|
	mov	$f0, 8*12($out_ptr)	# initialize |v| with |f1|

	################################# second iteration
	xor	\$256, $in_ptr		# flip-flop pointer to source |a|b|u|v|
	mov	\$62, $cnt
	call	__ab_approximation_62
	#mov	$f0, 8*7(%rsp)
	#mov	$g0, 8*8(%rsp)
	mov	$f1, 8*9(%rsp)
	mov	$g1, 8*10(%rsp)

	mov	\$256, $out_ptr
	xor	$in_ptr, $out_ptr	# pointer to destination |a|b|u|v|
	call	__smulq_383_n_shift_by_62
	mov	$f0, 8*7(%rsp)		# corrected |f0|
	mov	$g0, 8*8(%rsp)		# corrected |g0|

	mov	8*9(%rsp), $f0		# |f1|
	mov	8*10(%rsp), $g0		# |g1|
	lea	8*6($out_ptr), $out_ptr	# pointer to destination |b|
	call	__smulq_383_n_shift_by_62
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
	lea	8*12($in_ptr),$in_ptr	# make in_ptr "rewindable" with xor

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
for($i=2; $i<11; $i++) {
my $smul_767x63  = $i>5 ? "__smulq_767x63"
                        : "__smulq_383x63";
$code.=<<___;
	xor	\$256+8*12, $in_ptr	# flip-flop pointer to source |a|b|u|v|
	mov	\$62, $cnt
	call	__ab_approximation_62
	#mov	$f0, 8*7(%rsp)
	#mov	$g0, 8*8(%rsp)
	mov	$f1, 8*9(%rsp)
	mov	$g1, 8*10(%rsp)

	mov	\$256, $out_ptr
	xor	$in_ptr, $out_ptr	# pointer to destination |a|b|u|v|
	call	__smulq_383_n_shift_by_62
	mov	$f0, 8*7(%rsp)		# corrected |f0|
	mov	$g0, 8*8(%rsp)		# corrected |g0|

	mov	8*9(%rsp), $f0		# |f1|
	mov	8*10(%rsp), $g0		# |g1|
	lea	8*6($out_ptr), $out_ptr	# pointer to destination |b|
	call	__smulq_383_n_shift_by_62
	mov	$f0, 8*9(%rsp)		# corrected |f1|
	mov	$g0, 8*10(%rsp)		# corrected |g1|

	mov	8*7(%rsp), $f0		# |f0|
	mov	8*8(%rsp), $g0		# |g0|
	lea	8*12($in_ptr), $in_ptr	# pointer to source |u|v|
	lea	8*6($out_ptr), $out_ptr	# pointer to destination |u|
	call	__smulq_383x63

	mov	8*9(%rsp), $f0		# |f1|
	mov	8*10(%rsp), $g0		# |g1|
	lea	8*6($out_ptr),$out_ptr	# pointer to destination |v|
	call	$smul_767x63
___
$code.=<<___	if ($i==5);
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
	################################# iteration before last
	xor	\$256+8*12, $in_ptr	# flip-flop pointer to source |a|b|u|v|
	mov	\$62, $cnt
	#call	__ab_approximation_62	# |a| and |b| are exact, just load
	mov	8*0($in_ptr), @acc[0]	# |a_lo|
	mov	8*1($in_ptr), @acc[1]	# |a_hi|
	mov	8*6($in_ptr), @acc[2]	# |b_lo|
	mov	8*7($in_ptr), @acc[3]	# |b_hi|
	call	__inner_loop_62
	#mov	$f0, 8*7(%rsp)
	#mov	$g0, 8*8(%rsp)
	mov	$f1, 8*9(%rsp)
	mov	$g1, 8*10(%rsp)

	mov	\$256, $out_ptr
	xor	$in_ptr, $out_ptr	# pointer to destination |a|b|u|v|
	mov	@acc[0], 8*0($out_ptr)
	mov	@acc[2], 8*6($out_ptr)

	#mov	8*7(%rsp), $f0		# |f0|
	#mov	8*8(%rsp), $g0		# |g0|
	lea	8*12($in_ptr), $in_ptr	# pointer to source |u|v|
	lea	8*12($out_ptr),$out_ptr	# pointer to destination |u|
	call	__smulq_383x63

	mov	8*9(%rsp), $f0		# |f1|
	mov	8*10(%rsp), $g0		# |g1|
	lea	8*6($out_ptr),$out_ptr	# pointer to destination |v|
	call	__smulq_767x63

	################################# last iteration
	xor	\$256+8*12, $in_ptr	# flip-flop pointer to source |a|b|u|v|
	mov	\$22, $cnt		# 766 % 62
	#call	__ab_approximation_62	# |a| and |b| are exact, just load
	mov	8*0($in_ptr), @acc[0]	# |a_lo|
	xor	@acc[1],      @acc[1]	# |a_hi|
	mov	8*6($in_ptr), @acc[2]	# |b_lo|
	xor	@acc[3],   @acc[3]	# |b_hi|
	call	__inner_loop_62
	#mov	$f0, 8*7(%rsp)
	#mov	$g0, 8*8(%rsp)
	#mov	$f1, 8*9(%rsp)
	#mov	$g1, 8*10(%rsp)

	#mov	8*7(%rsp), $f0		# |f0|
	#mov	8*8(%rsp), $g0		# |g0|
	lea	8*12($in_ptr), $in_ptr	# pointer to source |u|v|
	#lea	8*6($out_ptr), $out_ptr	# pointer to destination |u|
	#call	__smulq_383x63

	#mov	8*9(%rsp), $f0		# |f1|
	#mov	8*10(%rsp), $g0		# |g1|
	mov	$f1, $f0
	mov	$g1, $g0
	mov	8*4(%rsp), $out_ptr	# original out_ptr
	call	__smulq_767x63

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
.size	ct_inverse_mod_383,.-ct_inverse_mod_383
___
########################################################################
# see corresponding commentary in ctx_inverse_mod_384-x86_64...
{
my ($out_ptr, $in_ptr, $f0, $g0) = ("%rdi", "%rsi", "%rdx", "%rcx");
my @acc = map("%r$_",(8..15),"bx","bp","cx","di");
my $fx = @acc[9];

$code.=<<___;
.type	__smulq_767x63,\@abi-omnipotent
.align	32
__smulq_767x63:
	mov	8*0($in_ptr), @acc[0]	# load |u|
	mov	8*1($in_ptr), @acc[1]
	mov	8*2($in_ptr), @acc[2]
	mov	8*3($in_ptr), @acc[3]
	mov	8*4($in_ptr), @acc[4]
	mov	8*5($in_ptr), @acc[5]

	mov	$f0, $fx
	sar	\$63, $f0		# |f0|'s sign as mask
	xor	%rax, %rax
	sub	$f0, %rax		# |f0|'s sign as bit

	mov	$out_ptr, 8*1(%rsp)
	mov	$in_ptr, 8*2(%rsp)
	lea	8*6($in_ptr), $in_ptr	# pointer to |v|

	xor	$f0, $fx		# conditionally negate |f0|
	add	%rax, $fx

	xor	$f0, @acc[0]		# conditionally negate |u|
	xor	$f0, @acc[1]
	xor	$f0, @acc[2]
	xor	$f0, @acc[3]
	xor	$f0, @acc[4]
	xor	$f0, @acc[5]
	add	@acc[0], %rax
	adc	\$0, @acc[1]
	adc	\$0, @acc[2]
	adc	\$0, @acc[3]
	adc	\$0, @acc[4]
	adc	\$0, @acc[5]

	mulq	$fx			# |u|*|f0|
	mov	%rax, 8*0($out_ptr)	# offload |u|*|f0|
	mov	@acc[1], %rax
	mov	%rdx, @acc[1]
___
for($i=1; $i<5; $i++) {
$code.=<<___;
	mulq	$fx
	add	%rax, @acc[$i]
	mov	@acc[$i+1], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[$i+1]
	mov	@acc[$i], 8*$i($out_ptr)
___
}
$code.=<<___;
	imulq	$fx
	add	%rax, @acc[$i]
	adc	\$0, %rdx

	mov	@acc[5], 8*5($out_ptr)
	mov	%rdx, 8*6($out_ptr)
	sar	\$63, %rdx		# sign extension
	mov	%rdx, 8*7($out_ptr)
___
{
my $fx=$in_ptr;
$code.=<<___;
	mov	$g0, $f0		# load |g0|

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

	mov	$f0, $fx		# overrides in_ptr
	sar	\$63, $f0		# |g0|'s sign as mask
	xor	%rax, %rax
	sub	$f0, %rax		# |g0|'s sign as bit

	xor	$f0, $fx		# conditionally negate |g0|
	add	%rax, $fx

	xor	$f0, @acc[0]		# conditionally negate |v|
	xor	$f0, @acc[1]
	xor	$f0, @acc[2]
	xor	$f0, @acc[3]
	xor	$f0, @acc[4]
	xor	$f0, @acc[5]
	xor	$f0, @acc[6]
	xor	$f0, @acc[7]
	xor	$f0, @acc[8]
	xor	$f0, @acc[9]
	xor	$f0, @acc[10]
	xor	$f0, @acc[11]
	add	@acc[0], %rax
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

	mulq	$fx			# |v|*|g0|
	mov	%rax, @acc[0]
	mov	@acc[1], %rax
	mov	%rdx, @acc[1]
___
for($i=1; $i<11; $i++) {
$code.=<<___;
	mulq	$fx
	add	%rax, @acc[$i]
	mov	@acc[$i+1], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[$i+1]
___
}
$code.=<<___;
	mov	8*1(%rsp), %rdx		# out_ptr
	imulq	$fx, %rax
	mov	8*2(%rsp), $in_ptr	# restore original in_ptr
	add	@acc[11], %rax

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
.size	__smulq_767x63,.-__smulq_767x63
___
}
$code.=<<___;
.type	__smulq_383x63,\@abi-omnipotent
.align	32
__smulq_383x63:
___
for($j=0; $j<2; $j++) {
$code.=<<___;
	mov	8*0($in_ptr), @acc[0]	# load |u| (or |v|)
	mov	8*1($in_ptr), @acc[1]
	mov	8*2($in_ptr), @acc[2]
	mov	8*3($in_ptr), @acc[3]
	mov	8*4($in_ptr), @acc[4]
	mov	8*5($in_ptr), @acc[5]

	mov	%rdx, $fx
	sar	\$63, %rdx		# |f0|'s sign as mask (or |g0|'s)
	xor	%rax, %rax
	sub	%rdx, %rax		# |f0|'s sign as bit (or |g0|'s)

	xor	%rdx, $fx		# conditionally negate |f0|
	add	%rax, $fx

	xor	%rdx, @acc[0]		# conditionally negate |u| (or |v|)
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

	mulq	$fx			# |u|*|f0| (or |v|*|g0|)
	mov	%rax, @acc[0]
	mov	@acc[1], %rax
	mov	%rdx, @acc[1]
___
for($i=1; $i<5; $i++) {
$code.=<<___;
	mulq	$fx
	add	%rax, @acc[$i]
	mov	@acc[$i+1], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[$i+1]
___
}
$code.=<<___	if ($j==0);
	imulq	$fx, %rax
	add	%rax, @acc[$i]

	lea	8*6($in_ptr), $in_ptr	# pointer to |v|
	mov	$g0, %rdx

	mov	@acc[0], 8*0($out_ptr)	# offload |u|*|f0|
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)
	mov	@acc[4], 8*4($out_ptr)
	mov	@acc[5], 8*5($out_ptr)
___
}
$code.=<<___;
	imulq	$fx, %rax
	add	%rax, @acc[$i]

	lea	-8*6($in_ptr), $in_ptr	# restore original in_ptr

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
.size	__smulq_383x63,.-__smulq_383x63
___
{
$code.=<<___;
.type	__smulq_383_n_shift_by_62,\@abi-omnipotent
.align	32
__smulq_383_n_shift_by_62:
	mov	$f0, @acc[8]
___
my $f0 = @acc[8];
for($j=0; $j<2; $j++) {
$code.=<<___;
	mov	8*0($in_ptr), @acc[0]	# load |a| (or |b|)
	mov	8*1($in_ptr), @acc[1]
	mov	8*2($in_ptr), @acc[2]
	mov	8*3($in_ptr), @acc[3]
	mov	8*4($in_ptr), @acc[4]
	mov	8*5($in_ptr), @acc[5]

	mov	%rdx, $fx
	sar	\$63, %rdx		# |f0|'s sign as mask (or |g0|'s)
	xor	%rax, %rax
	sub	%rdx, %rax		# |f0|'s sign as bit (or |g0|'s)

	xor	%rdx, $fx		# conditionally negate |f0| (or |g0|)
	add	%rax, $fx

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

	mulq	$fx			# |a|*|f0| (or |b|*|g0|)
	mov	%rax, @acc[0]
	mov	@acc[1], %rax
	mov	%rdx, @acc[1]
___
for($i=1; $i<5; $i++) {
$code.=<<___;
	mulq	$fx
	add	%rax, @acc[$i]
	mov	@acc[$i+1], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[$i+1]
___
}
$code.=<<___	if ($j==0);
	imulq	$fx
	add	%rax, @acc[$i]
	adc	\$0, %rdx

	lea	8*6($in_ptr), $in_ptr	# pointer to |b|
	mov	%rdx, @acc[6]
	mov	$g0, %rdx

	mov	@acc[0], 8*0($out_ptr)
	mov	@acc[1], 8*1($out_ptr)
	mov	@acc[2], 8*2($out_ptr)
	mov	@acc[3], 8*3($out_ptr)
	mov	@acc[4], 8*4($out_ptr)
	mov	@acc[5], 8*5($out_ptr)
___
}
$code.=<<___;
	imulq	$fx
	add	%rax, @acc[$i]
	adc	\$0, %rdx

	lea	-8*6($in_ptr), $in_ptr	# restore original in_ptr

	add	8*0($out_ptr), @acc[0]
	adc	8*1($out_ptr), @acc[1]
	adc	8*2($out_ptr), @acc[2]
	adc	8*3($out_ptr), @acc[3]
	adc	8*4($out_ptr), @acc[4]
	adc	8*5($out_ptr), @acc[5]
	adc	%rdx,          @acc[6]
	mov	$f0, %rdx

	shrd	\$62, @acc[1], @acc[0]
	shrd	\$62, @acc[2], @acc[1]
	shrd	\$62, @acc[3], @acc[2]
	shrd	\$62, @acc[4], @acc[3]
	shrd	\$62, @acc[5], @acc[4]
	shrd	\$62, @acc[6], @acc[5]

	sar	\$63, @acc[6]		# sign as mask
	xor	$fx, $fx
	sub	@acc[6], $fx		# sign as bit

	xor	@acc[6], @acc[0]	# conditionally negate the result
	xor	@acc[6], @acc[1]
	xor	@acc[6], @acc[2]
	xor	@acc[6], @acc[3]
	xor	@acc[6], @acc[4]
	xor	@acc[6], @acc[5]
	add	$fx, @acc[0]
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

	xor	@acc[6], %rdx		# conditionally negate |f0|
	xor	@acc[6], $g0		# conditionally negate |g0|
	add	$fx, %rdx
	add	$fx, $g0

	ret	# __SGX_LVI_HARDENING_CLOBBER__=@acc[0]
.size	__smulq_383_n_shift_by_62,.-__smulq_383_n_shift_by_62
___
} }

{
my ($a_lo, $a_hi, $b_lo, $b_hi) = map("%r$_",(8..11));
my ($t0, $t1, $t2, $t3, $t4, $t5) = ("%rax","%rbx","%rbp","%r14","%r15","%rsi");
{
my @a = ($a_lo, $t1, $a_hi);
my @b = ($b_lo, $t2, $b_hi);

$code.=<<___;
.type	__ab_approximation_62,\@abi-omnipotent
.align	32
__ab_approximation_62:
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
	cmovz	@b[0], @b[1]
	mov	8*2($in_ptr), @a[0]
	mov	8*8($in_ptr), @b[0]

	mov	@a[2], $t0
	or	@b[2], $t0		# ... ones before top-most, ...
	cmovz	@a[1], @a[2]
	cmovz	@b[1], @b[2]
	cmovz	@a[0], @a[1]
	cmovz	@b[0], @b[1]
	mov	8*1($in_ptr), @a[0]
	mov	8*7($in_ptr), @b[0]

	mov	@a[2], $t0
	or	@b[2], $t0		# ... and ones before that ...
	cmovz	@a[1], @a[2]
	cmovz	@b[1], @b[2]
	cmovz	@a[0], @a[1]
	cmovz	@b[0], @b[1]
	mov	8*0($in_ptr), @a[0]
	mov	8*6($in_ptr), @b[0]

	mov	@a[2], $t0
	or	@b[2], $t0
	bsr	$t0, %rcx
	lea	1(%rcx), %rcx
	cmovz	@a[1], @a[2]
	cmovz	@b[1], @b[2]
	cmovz	$t0, %rcx
	neg	%rcx
	#and	\$63, %rcx		# debugging artefact

	shldq	%cl, @a[1], @a[2]	# align second limb to the left
	shldq	%cl, @b[1], @b[2]

	jmp	__inner_loop_62

	ret
.size	__ab_approximation_62,.-__ab_approximation_62
___
}
$code.=<<___;
.type	__inner_loop_62,\@abi-omnipotent
.align	8
.long	0
__inner_loop_62:
	mov	\$1, $f0	# |f0|=1
	xor	$g0, $g0	# |g0|=0
	xor	$f1, $f1	# |f1|=0
	mov	\$1, $g1	# |g1|=1
	mov	$in_ptr, 8(%rsp)

.Loop_62:
	xor	$t0, $t0
	xor	$t1, $t1
	test	\$1, $a_lo	# if |a_| is odd, then we'll be subtracting |b_|
	mov	$b_lo, $t2
	mov	$b_hi, $t3
	cmovnz	$b_lo, $t0
	cmovnz	$b_hi, $t1
	sub	$a_lo, $t2	# |b_|-|a_|
	sbb	$a_hi, $t3
	mov	$a_lo, $t4
	mov	$a_hi, $t5
	sub	$t0, $a_lo	# |a_|-|b_| (or |a_|-0 if |a_| was even)
	sbb	$t1, $a_hi
	cmovc	$t2, $a_lo	# borrow means |a_|<|b_|, replace with |b_|-|a_|
	cmovc	$t3, $a_hi
	cmovc	$t4, $b_lo	# |b_| = |a_|
	cmovc	$t5, $b_hi
	mov	$f0, $t0	# exchange |f0| and |f1|
	cmovc	$f1, $f0
	cmovc	$t0, $f1
	mov	$g0, $t1	# exchange |g0| and |g1|
	cmovc	$g1, $g0
	cmovc	$t1, $g1
	xor	$t0, $t0
	xor	$t1, $t1
	shrd	\$1, $a_hi, $a_lo
	shr	\$1, $a_hi
	test	\$1, $t4	# if |a_| was odd, then we'll be subtracting...
	cmovnz	$f1, $t0
	cmovnz	$g1, $t1
	add	$f1, $f1	# |f1|<<=1
	add	$g1, $g1	# |g1|<<=1
	sub	$t0, $f0	# |f0|-=|f1| (or |f0-=0| if |a_| was even)
	sub	$t1, $g0	# |g0|-=|g1| (or |g0-=0| ...)
	sub	\$1, $cnt
	jnz	.Loop_62

	mov	8(%rsp), $in_ptr
	ret	# __SGX_LVI_HARDENING_CLOBBER__=$t0
.size	__inner_loop_62,.-__inner_loop_62
___
}

print $code;
close STDOUT;
