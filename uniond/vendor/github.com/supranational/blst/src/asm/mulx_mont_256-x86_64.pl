#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
#
# "Sparse" in subroutine names refers to most significant limb of the
# modulus. Though "sparse" is a bit of misnomer, because limitation is
# just not-all-ones. Or in other words not larger than 2^256-2^192-1.
# In general Montgomery multiplication algorithm can handle one of the
# inputs being non-reduced and capped by 1<<radix_width, 1<<256 in this
# case, rather than the modulus. Whether or not mul_mont_sparse_256, a
# *taylored* implementation of the algorithm, can handle such input can
# be circumstantial. For example, in most general case it depends on
# similar "bit sparsity" of individual limbs of the second, fully reduced
# multiplicand. If you can't make such assumption about the limbs, then
# non-reduced value shouldn't be larger than "same old" 2^256-2^192-1.
# This requirement can be met by conditionally subtracting "bitwise
# left-aligned" modulus. For example, if modulus is 200 bits wide, you
# would need to conditionally subtract the value of modulus<<56. Common
# source of non-reduced values is redc_mont_256 treating 512-bit inputs.
# Well, more specifically ones with upper half not smaller than modulus.
# Just in case, why limitation at all and not general-purpose 256-bit
# subroutines? Unlike the 384-bit case, accounting for additional carry
# has disproportionate impact on performance, especially in adcx/adox
# implementation.

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

{ ############################################################## 255 bits
my @acc=map("%r$_",(10..15));

{ ############################################################## mulq
my ($lo,$hi)=("%rbp","%r9");

$code.=<<___;
.text

.globl	mulx_mont_sparse_256
.hidden	mulx_mont_sparse_256
.type	mulx_mont_sparse_256,\@function,5,"unwind"
.align	32
mulx_mont_sparse_256:
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
	sub	\$8,%rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	$b_org, $b_ptr		# evacuate from %rdx
	mov	8*0($b_org), %rdx
	mov	8*0($a_ptr), @acc[4]
	mov	8*1($a_ptr), @acc[5]
	mov	8*2($a_ptr), $lo
	mov	8*3($a_ptr), $hi
	lea	-128($a_ptr), $a_ptr	# control u-op density
	lea	-128($n_ptr), $n_ptr	# control u-op density

	mulx	@acc[4], %rax, @acc[1]	# a[0]*b[0]
	call	__mulx_mont_sparse_256

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
.size	mulx_mont_sparse_256,.-mulx_mont_sparse_256

.globl	sqrx_mont_sparse_256
.hidden	sqrx_mont_sparse_256
.type	sqrx_mont_sparse_256,\@function,4,"unwind"
.align	32
sqrx_mont_sparse_256:
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
	sub	\$8,%rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	$a_ptr, $b_ptr
	mov	$n_ptr, $n0
	mov	$b_org, $n_ptr
	mov	8*0($a_ptr), %rdx
	mov	8*1($a_ptr), @acc[5]
	mov	8*2($a_ptr), $lo
	mov	8*3($a_ptr), $hi
	lea	-128($b_ptr), $a_ptr	# control u-op density
	lea	-128($n_ptr), $n_ptr	# control u-op density

	mulx	%rdx, %rax, @acc[1]	# a[0]*a[0]
	call	__mulx_mont_sparse_256

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
.size	sqrx_mont_sparse_256,.-sqrx_mont_sparse_256
___
{
my @acc=@acc;
$code.=<<___;
.type	__mulx_mont_sparse_256,\@abi-omnipotent
.align	32
__mulx_mont_sparse_256:
	mulx	@acc[5], @acc[5], @acc[2]
	mulx	$lo, $lo, @acc[3]
	add	@acc[5], @acc[1]
	mulx	$hi, $hi, @acc[4]
	 mov	8($b_ptr), %rdx
	adc	$lo, @acc[2]
	adc	$hi, @acc[3]
	adc	\$0, @acc[4]

___
for (my $i=1; $i<4; $i++) {
my $b_next = $i<3 ? 8*($i+1)."($b_ptr)" : "%rax";
my $a5 = $i==1 ? @acc[5] : $lo;
$code.=<<___;
	 mov	%rax, @acc[0]
	 imulq	$n0, %rax

	################################# Multiply by b[$i]
	xor	$a5, $a5		# [@acc[5]=0,] cf=0, of=0
	mulx	8*0+128($a_ptr), $lo, $hi
	adox	$lo, @acc[1]
	adcx	$hi, @acc[2]

	mulx	8*1+128($a_ptr), $lo, $hi
	adox	$lo, @acc[2]
	adcx	$hi, @acc[3]

	mulx	8*2+128($a_ptr), $lo, $hi
	adox	$lo, @acc[3]
	adcx	$hi, @acc[4]

	mulx	8*3+128($a_ptr), $lo, $hi
	 mov	%rax, %rdx
	adox	$lo, @acc[4]
	adcx	@acc[5], $hi 		# cf=0
	adox	$hi, @acc[5]		# of=0

	################################# reduction
	mulx	8*0+128($n_ptr), $lo, %rax
	adcx	$lo, @acc[0]		# guaranteed to be zero
	adox	@acc[1], %rax

	mulx	8*1+128($n_ptr), $lo, $hi
	adcx	$lo, %rax		# @acc[1]
	adox	$hi, @acc[2]

	mulx	8*2+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[2]
	adox	$hi, @acc[3]

	mulx	8*3+128($n_ptr), $lo, $hi
	 mov	$b_next, %rdx
	adcx	$lo, @acc[3]
	adox	$hi, @acc[4]
	adcx	@acc[0], @acc[4]
	adox	@acc[0], @acc[5]
	adcx	@acc[0], @acc[5]
	adox	@acc[0], @acc[0]	# acc[5] in next iteration
	adc	\$0, @acc[0]		# cf=0, of=0
___
    push(@acc,shift(@acc));
}
$code.=<<___;
	imulq	$n0, %rdx

	################################# last reduction
	xor	$lo, $lo		# cf=0, of=0
	mulx	8*0+128($n_ptr), @acc[0], $hi
	adcx	%rax, @acc[0]		# guaranteed to be zero
	adox	$hi, @acc[1]

	mulx	8*1+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[1]
	adox	$hi, @acc[2]

	mulx	8*2+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[2]
	adox	$hi, @acc[3]

	mulx	8*3+128($n_ptr), $lo, $hi
	 mov	@acc[1], %rdx
	 lea	128($n_ptr), $n_ptr
	adcx	$lo, @acc[3]
	adox	$hi, @acc[4]
	 mov	@acc[2], %rax
	adcx	@acc[0], @acc[4]
	adox	@acc[0], @acc[5]
	adc	\$0, @acc[5]

	#################################
	# Branch-less conditional acc[1:5] - modulus

	 mov	@acc[3], $lo
	sub	8*0($n_ptr), @acc[1]
	sbb	8*1($n_ptr), @acc[2]
	sbb	8*2($n_ptr), @acc[3]
	 mov	@acc[4], $hi
	sbb	8*3($n_ptr), @acc[4]
	sbb	\$0, @acc[5]

	cmovc	%rdx, @acc[1]
	cmovc	%rax, @acc[2]
	cmovc	$lo,  @acc[3]
	mov	@acc[1], 8*0($r_ptr)
	cmovc	$hi,  @acc[4]
	mov	@acc[2], 8*1($r_ptr)
	mov	@acc[3], 8*2($r_ptr)
	mov	@acc[4], 8*3($r_ptr)

	ret
.size	__mulx_mont_sparse_256,.-__mulx_mont_sparse_256
___
} }
{ my ($n_ptr, $n0)=($b_ptr, $n_ptr);	# arguments are "shifted"

$code.=<<___;
.globl	fromx_mont_256
.hidden	fromx_mont_256
.type	fromx_mont_256,\@function,4,"unwind"
.align	32
fromx_mont_256:
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
	call	__mulx_by_1_mont_256

	#################################
	# Branch-less conditional acc[0:3] - modulus

	#mov	@acc[4], %rax		# __mulq_by_1_mont_256 does it
	mov	@acc[5], %rdx
	mov	@acc[0], @acc[2]
	mov	@acc[1], @acc[3]

	sub	8*0($n_ptr), @acc[4]
	sbb	8*1($n_ptr), @acc[5]
	sbb	8*2($n_ptr), @acc[0]
	sbb	8*3($n_ptr), @acc[1]

	cmovnc	@acc[4], %rax
	cmovnc	@acc[5], %rdx
	cmovnc	@acc[0], @acc[2]
	mov	%rax,    8*0($r_ptr)
	cmovnc	@acc[1], @acc[3]
	mov	%rdx,    8*1($r_ptr)
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
.size	fromx_mont_256,.-fromx_mont_256

.globl	redcx_mont_256
.hidden	redcx_mont_256
.type	redcx_mont_256,\@function,4,"unwind"
.align	32
redcx_mont_256:
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
	call	__mulx_by_1_mont_256

	add	8*4($a_ptr), @acc[4]	# accumulate upper half
	adc	8*5($a_ptr), @acc[5]
	mov	@acc[4], %rax
	adc	8*6($a_ptr), @acc[0]
	mov	@acc[5], %rdx
	adc	8*7($a_ptr), @acc[1]
	sbb	$a_ptr, $a_ptr

	#################################
	# Branch-less conditional acc[0:4] - modulus

	mov	@acc[0], @acc[2]
	sub	8*0($n_ptr), @acc[4]
	sbb	8*1($n_ptr), @acc[5]
	sbb	8*2($n_ptr), @acc[0]
	mov	@acc[1], @acc[3]
	sbb	8*3($n_ptr), @acc[1]
	sbb	\$0, $a_ptr

	cmovnc	@acc[4], %rax 
	cmovnc	@acc[5], %rdx
	cmovnc	@acc[0], @acc[2]
	mov	%rax,    8*0($r_ptr)
	cmovnc	@acc[1], @acc[3]
	mov	%rdx,    8*1($r_ptr)
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
.size	redcx_mont_256,.-redcx_mont_256
___
{
my @acc=@acc;

$code.=<<___;
.type	__mulx_by_1_mont_256,\@abi-omnipotent
.align	32
__mulx_by_1_mont_256:
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
.size	__mulx_by_1_mont_256,.-__mulx_by_1_mont_256
___
} } }

print $code;
close STDOUT;
