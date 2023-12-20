#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
#
# As for "sparse" in subroutine names, see commentary in the
# asm/mulx_mont_256-x86_64.pl module.

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

# common argument layout
($r_ptr,$a_ptr,$b_org,$n_ptr,$n0) = ("%rdi","%rsi","%rdx","%rcx","%r8");
$b_ptr = "%rbx";

{ ############################################################## 256 bits
my @acc=map("%r$_",(9..15));

{ ############################################################## mulq
my ($hi, $a0) = ("%rbp", $r_ptr);

$code.=<<___;
.text

.globl	mul_mont_sparse_256
.hidden	mul_mont_sparse_256
.type	mul_mont_sparse_256,\@function,5,"unwind"
.align	32
mul_mont_sparse_256:
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
	push	$r_ptr
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	8*0($b_org), %rax
	mov	8*0($a_ptr), @acc[4]
	mov	8*1($a_ptr), @acc[5]
	mov	8*2($a_ptr), @acc[3]
	mov	8*3($a_ptr), $hi
	mov	$b_org, $b_ptr		# evacuate from %rdx

	mov	%rax, @acc[6]
	mulq	@acc[4]			# a[0]*b[0]
	mov	%rax, @acc[0]
	mov	@acc[6], %rax
	mov	%rdx, @acc[1]
	call	__mulq_mont_sparse_256

	mov	8(%rsp),%r15
.cfi_restore	%r15
	mov	16(%rsp),%r14
.cfi_restore	%r14
	mov	24(%rsp),%r13
.cfi_restore	%r13
	mov	32(%rsp),%r12
.cfi_restore	%r12
	mov	40(%rsp),%rbx
.cfi_restore	%rbx
	mov	48(%rsp),%rbp
.cfi_restore	%rbp
	lea	56(%rsp),%rsp
.cfi_adjust_cfa_offset	-56
.cfi_epilogue
	ret
.cfi_endproc
.size	mul_mont_sparse_256,.-mul_mont_sparse_256

.globl	sqr_mont_sparse_256
.hidden	sqr_mont_sparse_256
.type	sqr_mont_sparse_256,\@function,4,"unwind"
.align	32
sqr_mont_sparse_256:
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
	push	$r_ptr
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	8*0($a_ptr), %rax
	mov	$n_ptr, $n0
	mov	8*1($a_ptr), @acc[5]
	mov	$b_org, $n_ptr
	mov	8*2($a_ptr), @acc[3]
	lea	($a_ptr), $b_ptr
	mov	8*3($a_ptr), $hi

	mov	%rax, @acc[6]
	mulq	%rax			# a[0]*a[0]
	mov	%rax, @acc[0]
	mov	@acc[6], %rax
	mov	%rdx, @acc[1]
	call	__mulq_mont_sparse_256

	mov	8(%rsp),%r15
.cfi_restore	%r15
	mov	16(%rsp),%r14
.cfi_restore	%r14
	mov	24(%rsp),%r13
.cfi_restore	%r13
	mov	32(%rsp),%r12
.cfi_restore	%r12
	mov	40(%rsp),%rbx
.cfi_restore	%rbx
	mov	48(%rsp),%rbp
.cfi_restore	%rbp
	lea	56(%rsp),%rsp
.cfi_adjust_cfa_offset	-56
.cfi_epilogue
	ret
.cfi_endproc
.size	sqr_mont_sparse_256,.-sqr_mont_sparse_256
___
{
my @acc=@acc;
$code.=<<___;
.type	__mulq_mont_sparse_256,\@abi-omnipotent
.align	32
__mulq_mont_sparse_256:
	mulq	@acc[5]			# a[1]*b[0]
	add	%rax, @acc[1]
	mov	@acc[6], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[2]

	mulq	@acc[3]			# a[2]*b[0]
	add	%rax, @acc[2]
	mov	@acc[6], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[3]

	mulq	$hi			# a[3]*b[0]
	add	%rax, @acc[3]
	 mov	8($b_ptr), %rax
	adc	\$0, %rdx
	xor	@acc[5], @acc[5]
	mov	%rdx, @acc[4]

___
for (my $i=1; $i<4; $i++) {
my $b_next = $i<3 ? 8*($i+1)."($b_ptr)" : @acc[1];
$code.=<<___;
	mov	@acc[0], $a0
	imulq	$n0, @acc[0]

	################################# Multiply by b[$i]
	mov	%rax, @acc[6]
	mulq	8*0($a_ptr)
	add	%rax, @acc[1]
	mov	@acc[6], %rax
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*1($a_ptr)
	add	%rax, @acc[2]
	mov	@acc[6], %rax
	adc	\$0, %rdx
	add	$hi, @acc[2]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*2($a_ptr)
	add	%rax, @acc[3]
	mov	@acc[6], %rax
	adc	\$0, %rdx
	add	$hi, @acc[3]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*3($a_ptr)
	add	%rax, @acc[4]
	 mov	@acc[0], %rax
	adc	\$0, %rdx
	add	$hi, @acc[4]
	adc	%rdx, @acc[5]		# can't overflow
	xor	@acc[6], @acc[6]

	################################# reduction
	mulq	8*0($n_ptr)
	add	%rax, $a0		# guaranteed to be zero
	mov	@acc[0], %rax
	adc	%rdx, $a0

	mulq	8*1($n_ptr)
	add	%rax, @acc[1]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	add	$a0, @acc[1]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*2($n_ptr)
	add	%rax, @acc[2]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	add	$hi, @acc[2]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*3($n_ptr)
	add	%rax, @acc[3]
	 mov	$b_next, %rax
	adc	\$0, %rdx
	add	$hi, @acc[3]
	adc	\$0, %rdx
	add	%rdx, @acc[4]
	adc	\$0, @acc[5]
	adc	\$0, @acc[6]
___
    push(@acc,shift(@acc));
}
$code.=<<___;
	imulq	$n0, %rax
	mov	8(%rsp), $a_ptr		# restore $r_ptr

	################################# last reduction
	mov	%rax, @acc[6]
	mulq	8*0($n_ptr)
	add	%rax, @acc[0]		# guaranteed to be zero
	mov	@acc[6], %rax
	adc	%rdx, @acc[0]

	mulq	8*1($n_ptr)
	add	%rax, @acc[1]
	mov	@acc[6], %rax
	adc	\$0, %rdx
	add	@acc[0], @acc[1]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*2($n_ptr)
	add	%rax, @acc[2]
	mov	@acc[6], %rax
	adc	\$0, %rdx
	add	$hi, @acc[2]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*3($n_ptr)
	 mov	@acc[2], $b_ptr
	add	$hi, @acc[3]
	adc	\$0, %rdx
	add	%rax, @acc[3]
	 mov	@acc[1], %rax
	adc	\$0, %rdx
	add	%rdx, @acc[4]
	adc	\$0, @acc[5]

	#################################
	# Branch-less conditional subtraction of modulus

	 mov	@acc[3], @acc[0]
	sub	8*0($n_ptr), @acc[1]
	sbb	8*1($n_ptr), @acc[2]
	sbb	8*2($n_ptr), @acc[3]
	 mov	@acc[4], $hi
	sbb	8*3($n_ptr), @acc[4]
	sbb	\$0, @acc[5]

	cmovc	%rax, @acc[1]
	cmovc	$b_ptr, @acc[2]
	cmovc	@acc[0], @acc[3]
	mov	@acc[1], 8*0($a_ptr)
	cmovc	$hi, @acc[4]
	mov	@acc[2], 8*1($a_ptr)
	mov	@acc[3], 8*2($a_ptr)
	mov	@acc[4], 8*3($a_ptr)

	ret
.cfi_endproc
.size	__mulq_mont_sparse_256,.-__mulq_mont_sparse_256
___
} }
{ my ($n_ptr, $n0)=($b_ptr, $n_ptr);	# arguments are "shifted"

$code.=<<___;
.globl	from_mont_256
.hidden	from_mont_256
.type	from_mont_256,\@function,4,"unwind"
.align	32
from_mont_256:
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
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	$b_org, $n_ptr
	call	__mulq_by_1_mont_256

	#################################
	# Branch-less conditional acc[0:3] - modulus

	#mov	@acc[4], %rax		# __mulq_by_1_mont_256 does it
	mov	@acc[5], @acc[1]
	mov	@acc[6], @acc[2]
	mov	@acc[0], @acc[3]

	sub	8*0($n_ptr), @acc[4]
	sbb	8*1($n_ptr), @acc[5]
	sbb	8*2($n_ptr), @acc[6]
	sbb	8*3($n_ptr), @acc[0]

	cmovnc	@acc[4], %rax
	cmovnc	@acc[5], @acc[1]
	cmovnc	@acc[6], @acc[2]
	mov	%rax,    8*0($r_ptr)
	cmovnc	@acc[0], @acc[3]
	mov	@acc[1], 8*1($r_ptr)
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)

	mov	8(%rsp),%r15
.cfi_restore	%r15
	mov	16(%rsp),%r14
.cfi_restore	%r14
	mov	24(%rsp),%r13
.cfi_restore	%r13
	mov	32(%rsp),%r12
.cfi_restore	%r12
	mov	40(%rsp),%rbx
.cfi_restore	%rbx
	mov	48(%rsp),%rbp
.cfi_restore	%rbp
	lea	56(%rsp),%rsp
.cfi_adjust_cfa_offset	-56
.cfi_epilogue
	ret
.cfi_endproc
.size	from_mont_256,.-from_mont_256

.globl	redc_mont_256
.hidden	redc_mont_256
.type	redc_mont_256,\@function,4,"unwind"
.align	32
redc_mont_256:
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
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	$b_org, $n_ptr
	call	__mulq_by_1_mont_256

	add	8*4($a_ptr), @acc[4]	# accumulate upper half
	adc	8*5($a_ptr), @acc[5]
	mov	@acc[4], %rax
	adc	8*6($a_ptr), @acc[6]
	mov	@acc[5], @acc[1]
	adc	8*7($a_ptr), @acc[0]
	sbb	$a_ptr, $a_ptr

	#################################
	# Branch-less conditional acc[0:4] - modulus

	mov	@acc[6], @acc[2]
	sub	8*0($n_ptr), @acc[4]
	sbb	8*1($n_ptr), @acc[5]
	sbb	8*2($n_ptr), @acc[6]
	mov	@acc[0], @acc[3]
	sbb	8*3($n_ptr), @acc[0]
	sbb	\$0, $a_ptr

	cmovnc	@acc[4], %rax 
	cmovnc	@acc[5], @acc[1]
	cmovnc	@acc[6], @acc[2]
	mov	%rax,    8*0($r_ptr)
	cmovnc	@acc[0], @acc[3]
	mov	@acc[1], 8*1($r_ptr)
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)

	mov	8(%rsp),%r15
.cfi_restore	%r15
	mov	16(%rsp),%r14
.cfi_restore	%r14
	mov	24(%rsp),%r13
.cfi_restore	%r13
	mov	32(%rsp),%r12
.cfi_restore	%r12
	mov	40(%rsp),%rbx
.cfi_restore	%rbx
	mov	48(%rsp),%rbp
.cfi_restore	%rbp
	lea	56(%rsp),%rsp
.cfi_adjust_cfa_offset	-56
.cfi_epilogue
	ret
.cfi_endproc
.size	redc_mont_256,.-redc_mont_256
___
{
my @acc=@acc;

$code.=<<___;
.type	__mulq_by_1_mont_256,\@abi-omnipotent
.align	32
__mulq_by_1_mont_256:
	mov	8*0($a_ptr), %rax
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]

	mov	%rax, @acc[4]
	imulq	$n0, %rax
	mov	%rax, @acc[0]
___
for (my $i=0; $i<4; $i++) {
my $hi = @acc[4];
$code.=<<___;
	################################# reduction $i
	mulq	8*0($n_ptr)
	add	%rax, @acc[4]		# guaranteed to be zero
	mov	@acc[0], %rax
	adc	%rdx, @acc[4]

	mulq	8*1($n_ptr)
	add	%rax, @acc[1]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	add	@acc[4], @acc[1]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*2($n_ptr)
___
$code.=<<___	if ($i<3);
	 mov	@acc[1], @acc[5]
	 imulq	$n0, @acc[1]
___
$code.=<<___;
	add	%rax, @acc[2]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	add	$hi, @acc[2]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*3($n_ptr)
	add	%rax, @acc[3]
	mov	@acc[1], %rax
	adc	\$0, %rdx
	add	$hi, @acc[3]
	adc	\$0, %rdx
	mov	%rdx, @acc[4]
___
    push(@acc,shift(@acc));
}
$code.=<<___;
	ret
.size	__mulq_by_1_mont_256,.-__mulq_by_1_mont_256
___
} } }

print $code;
close STDOUT;
