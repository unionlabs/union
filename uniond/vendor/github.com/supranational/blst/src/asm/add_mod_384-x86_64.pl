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

# common argument layout
($r_ptr,$a_ptr,$b_org,$n_ptr,$n0) = ("%rdi","%rsi","%rdx","%rcx","%r8");
$b_ptr = "%rbx";

{ ############################################################## 384 bits add
my @acc=map("%r$_",(8..15, "ax", "bx", "bp"));
   push(@acc, $a_ptr);

$code.=<<___;
.text

.globl	add_mod_384
.hidden	add_mod_384
.type	add_mod_384,\@function,4,"unwind"
.align	32
add_mod_384:
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

	call	__add_mod_384

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
.size	add_mod_384,.-add_mod_384

.type	__add_mod_384,\@abi-omnipotent
.align	32
__add_mod_384:
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]

__add_mod_384_a_is_loaded:
	add	8*0($b_org), @acc[0]
	adc	8*1($b_org), @acc[1]
	adc	8*2($b_org), @acc[2]
	 mov	@acc[0], @acc[6]
	adc	8*3($b_org), @acc[3]
	 mov	@acc[1], @acc[7]
	adc	8*4($b_org), @acc[4]
	 mov	@acc[2], @acc[8]
	adc	8*5($b_org), @acc[5]
	 mov	@acc[3], @acc[9]
	sbb	$b_org, $b_org

	sub	8*0($n_ptr), @acc[0]
	sbb	8*1($n_ptr), @acc[1]
	 mov	@acc[4], @acc[10]
	sbb	8*2($n_ptr), @acc[2]
	sbb	8*3($n_ptr), @acc[3]
	sbb	8*4($n_ptr), @acc[4]
	 mov	@acc[5], @acc[11]
	sbb	8*5($n_ptr), @acc[5]
	sbb	\$0, $b_org

	cmovc	@acc[6],  @acc[0]
	cmovc	@acc[7],  @acc[1]
	cmovc	@acc[8],  @acc[2]
	mov	@acc[0], 8*0($r_ptr)
	cmovc	@acc[9],  @acc[3]
	mov	@acc[1], 8*1($r_ptr)
	cmovc	@acc[10], @acc[4]
	mov	@acc[2], 8*2($r_ptr)
	cmovc	@acc[11], @acc[5]
	mov	@acc[3], 8*3($r_ptr)
	mov	@acc[4], 8*4($r_ptr)
	mov	@acc[5], 8*5($r_ptr)

	ret
.size	__add_mod_384,.-__add_mod_384

.globl	add_mod_384x
.hidden	add_mod_384x
.type	add_mod_384x,\@function,4,"unwind"
.align	32
add_mod_384x:
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
	sub	\$24, %rsp
.cfi_adjust_cfa_offset	24
.cfi_end_prologue

	mov	$a_ptr, 8*0(%rsp)
	mov	$b_org, 8*1(%rsp)
	lea	48($a_ptr), $a_ptr	# a->im
	lea	48($b_org), $b_org	# b->im
	lea	48($r_ptr), $r_ptr	# ret->im
	call	__add_mod_384		# add_mod_384(ret->im, a->im, b->im, mod);

	mov	8*0(%rsp), $a_ptr	# a->re
	mov	8*1(%rsp), $b_org	# b->re
	lea	-48($r_ptr), $r_ptr	# ret->re
	call	__add_mod_384		# add_mod_384(ret->re, a->re, b->re, mod);

	mov	24+8*0(%rsp),%r15
.cfi_restore	%r15
	mov	24+8*1(%rsp),%r14
.cfi_restore	%r14
	mov	24+8*2(%rsp),%r13
.cfi_restore	%r13
	mov	24+8*3(%rsp),%r12
.cfi_restore	%r12
	mov	24+8*4(%rsp),%rbx
.cfi_restore	%rbx
	mov	24+8*5(%rsp),%rbp
.cfi_restore	%rbp
	lea	24+8*6(%rsp),%rsp
.cfi_adjust_cfa_offset	-24-8*6
.cfi_epilogue
	ret
.cfi_endproc
.size	add_mod_384x,.-add_mod_384x

########################################################################
.globl	rshift_mod_384
.hidden	rshift_mod_384
.type	rshift_mod_384,\@function,4,"unwind"
.align	32
rshift_mod_384:
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

	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]

.Loop_rshift_mod_384:
	call	__rshift_mod_384
	dec	%edx
	jnz	.Loop_rshift_mod_384

	mov	@acc[0], 8*0($r_ptr)
	mov	@acc[1], 8*1($r_ptr)
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)
	mov	@acc[4], 8*4($r_ptr)
	mov	@acc[5], 8*5($r_ptr)

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
.size	rshift_mod_384,.-rshift_mod_384

.type	__rshift_mod_384,\@abi-omnipotent
.align	32
__rshift_mod_384:
	mov	\$1, @acc[11]
	mov	8*0($n_ptr), @acc[6]
	and	@acc[0], @acc[11]
	mov	8*1($n_ptr), @acc[7]
	neg	@acc[11]
	mov	8*2($n_ptr), @acc[8]
	and	@acc[11], @acc[6]
	mov	8*3($n_ptr), @acc[9]
	and	@acc[11], @acc[7]
	mov	8*4($n_ptr), @acc[10]
	and	@acc[11], @acc[8]
	and	@acc[11], @acc[9]
	and	@acc[11], @acc[10]
	and	8*5($n_ptr), @acc[11]

	add	@acc[0], @acc[6]
	adc	@acc[1], @acc[7]
	adc	@acc[2], @acc[8]
	adc	@acc[3], @acc[9]
	adc	@acc[4], @acc[10]
	adc	@acc[5], @acc[11]
	sbb	@acc[5], @acc[5]

	shr	\$1, @acc[6]
	mov	@acc[7], @acc[0]
	shr	\$1, @acc[7]
	mov	@acc[8], @acc[1]
	shr	\$1, @acc[8]
	mov	@acc[9], @acc[2]
	shr	\$1, @acc[9]
	mov	@acc[10], @acc[3]
	shr	\$1, @acc[10]
	mov	@acc[11], @acc[4]
	shr	\$1, @acc[11]
	shl	\$63, @acc[0]
	shl	\$63, @acc[1]
	or	@acc[6], @acc[0]
	shl	\$63, @acc[2]
	or	@acc[7], @acc[1]
	shl	\$63, @acc[3]
	or	@acc[8], @acc[2]
	shl	\$63, @acc[4]
	or	@acc[9], @acc[3]
	shl	\$63, @acc[5]
	or	@acc[10], @acc[4]
	or	@acc[11], @acc[5]

	ret
.size	__rshift_mod_384,.-__rshift_mod_384

.globl	div_by_2_mod_384
.hidden	div_by_2_mod_384
.type	div_by_2_mod_384,\@function,3,"unwind"
.align	32
div_by_2_mod_384:
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

	mov	8*0($a_ptr), @acc[0]
	mov	$b_org, $n_ptr
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]

	call	__rshift_mod_384

	mov	@acc[0], 8*0($r_ptr)
	mov	@acc[1], 8*1($r_ptr)
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)
	mov	@acc[4], 8*4($r_ptr)
	mov	@acc[5], 8*5($r_ptr)

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
.size	div_by_2_mod_384,.-div_by_2_mod_384

########################################################################
.globl	lshift_mod_384
.hidden	lshift_mod_384
.type	lshift_mod_384,\@function,4,"unwind"
.align	32
lshift_mod_384:
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

	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]

.Loop_lshift_mod_384:
	add	@acc[0], @acc[0]
	adc	@acc[1], @acc[1]
	adc	@acc[2], @acc[2]
	 mov	@acc[0], @acc[6]
	adc	@acc[3], @acc[3]
	 mov	@acc[1], @acc[7]
	adc	@acc[4], @acc[4]
	 mov	@acc[2], @acc[8]
	adc	@acc[5], @acc[5]
	 mov	@acc[3], @acc[9]
	sbb	$r_ptr, $r_ptr

	sub	8*0($n_ptr), @acc[0]
	sbb	8*1($n_ptr), @acc[1]
	 mov	@acc[4], @acc[10]
	sbb	8*2($n_ptr), @acc[2]
	sbb	8*3($n_ptr), @acc[3]
	sbb	8*4($n_ptr), @acc[4]
	 mov	@acc[5], @acc[11]
	sbb	8*5($n_ptr), @acc[5]
	sbb	\$0, $r_ptr

	mov	(%rsp), $r_ptr
	cmovc	@acc[6],  @acc[0]
	cmovc	@acc[7],  @acc[1]
	cmovc	@acc[8],  @acc[2]
	cmovc	@acc[9],  @acc[3]
	cmovc	@acc[10], @acc[4]
	cmovc	@acc[11], @acc[5]

	dec	%edx
	jnz	.Loop_lshift_mod_384

	mov	@acc[0], 8*0($r_ptr)
	mov	@acc[1], 8*1($r_ptr)
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)
	mov	@acc[4], 8*4($r_ptr)
	mov	@acc[5], 8*5($r_ptr)

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
.size	lshift_mod_384,.-lshift_mod_384

.type	__lshift_mod_384,\@abi-omnipotent
.align	32
__lshift_mod_384:
	add	@acc[0], @acc[0]
	adc	@acc[1], @acc[1]
	adc	@acc[2], @acc[2]
	 mov	@acc[0], @acc[6]
	adc	@acc[3], @acc[3]
	 mov	@acc[1], @acc[7]
	adc	@acc[4], @acc[4]
	 mov	@acc[2], @acc[8]
	adc	@acc[5], @acc[5]
	 mov	@acc[3], @acc[9]
	sbb	$b_org, $b_org

	sub	8*0($n_ptr), @acc[0]
	sbb	8*1($n_ptr), @acc[1]
	 mov	@acc[4], @acc[10]
	sbb	8*2($n_ptr), @acc[2]
	sbb	8*3($n_ptr), @acc[3]
	sbb	8*4($n_ptr), @acc[4]
	 mov	@acc[5], @acc[11]
	sbb	8*5($n_ptr), @acc[5]
	sbb	\$0, $b_org

	cmovc	@acc[6],  @acc[0]
	cmovc	@acc[7],  @acc[1]
	cmovc	@acc[8],  @acc[2]
	cmovc	@acc[9],  @acc[3]
	cmovc	@acc[10], @acc[4]
	cmovc	@acc[11], @acc[5]

	ret
.size	__lshift_mod_384,.-__lshift_mod_384

########################################################################
.globl	mul_by_3_mod_384
.hidden	mul_by_3_mod_384
.type	mul_by_3_mod_384,\@function,3,"unwind"
.align	32
mul_by_3_mod_384:
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
	push	$a_ptr
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]
	mov	$b_org, $n_ptr

	call	__lshift_mod_384

	mov	(%rsp), $b_org
	call	__add_mod_384_a_is_loaded

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
.size	mul_by_3_mod_384,.-mul_by_3_mod_384

.globl	mul_by_8_mod_384
.hidden	mul_by_8_mod_384
.type	mul_by_8_mod_384,\@function,3,"unwind"
.align	32
mul_by_8_mod_384:
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

	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]
	mov	$b_org, $n_ptr

	call	__lshift_mod_384
	call	__lshift_mod_384
	call	__lshift_mod_384

	mov	@acc[0], 8*0($r_ptr)
	mov	@acc[1], 8*1($r_ptr)
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)
	mov	@acc[4], 8*4($r_ptr)
	mov	@acc[5], 8*5($r_ptr)

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
.size	mul_by_8_mod_384,.-mul_by_8_mod_384

########################################################################
.globl	mul_by_3_mod_384x
.hidden	mul_by_3_mod_384x
.type	mul_by_3_mod_384x,\@function,3,"unwind"
.align	32
mul_by_3_mod_384x:
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
	push	$a_ptr
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]
	mov	$b_org, $n_ptr

	call	__lshift_mod_384

	mov	(%rsp), $b_org
	call	__add_mod_384_a_is_loaded

	mov	(%rsp), $a_ptr
	lea	8*6($r_ptr), $r_ptr

	mov	8*6($a_ptr), @acc[0]
	mov	8*7($a_ptr), @acc[1]
	mov	8*8($a_ptr), @acc[2]
	mov	8*9($a_ptr), @acc[3]
	mov	8*10($a_ptr), @acc[4]
	mov	8*11($a_ptr), @acc[5]

	call	__lshift_mod_384

	mov	\$8*6, $b_org
	add	(%rsp), $b_org
	call	__add_mod_384_a_is_loaded

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
.size	mul_by_3_mod_384x,.-mul_by_3_mod_384x

.globl	mul_by_8_mod_384x
.hidden	mul_by_8_mod_384x
.type	mul_by_8_mod_384x,\@function,3,"unwind"
.align	32
mul_by_8_mod_384x:
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
	push	$a_ptr
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]
	mov	$b_org, $n_ptr

	call	__lshift_mod_384
	call	__lshift_mod_384
	call	__lshift_mod_384

	mov	(%rsp), $a_ptr
	mov	@acc[0], 8*0($r_ptr)
	mov	@acc[1], 8*1($r_ptr)
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)
	mov	@acc[4], 8*4($r_ptr)
	mov	@acc[5], 8*5($r_ptr)

	mov	48+8*0($a_ptr), @acc[0]
	mov	48+8*1($a_ptr), @acc[1]
	mov	48+8*2($a_ptr), @acc[2]
	mov	48+8*3($a_ptr), @acc[3]
	mov	48+8*4($a_ptr), @acc[4]
	mov	48+8*5($a_ptr), @acc[5]

	call	__lshift_mod_384
	call	__lshift_mod_384
	call	__lshift_mod_384

	mov	@acc[0], 48+8*0($r_ptr)
	mov	@acc[1], 48+8*1($r_ptr)
	mov	@acc[2], 48+8*2($r_ptr)
	mov	@acc[3], 48+8*3($r_ptr)
	mov	@acc[4], 48+8*4($r_ptr)
	mov	@acc[5], 48+8*5($r_ptr)

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
.size	mul_by_8_mod_384x,.-mul_by_8_mod_384x

########################################################################
.globl	cneg_mod_384
.hidden	cneg_mod_384
.type	cneg_mod_384,\@function,4,"unwind"
.align	32
cneg_mod_384:
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
	push	$b_org			# condition flag
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	8*0($a_ptr), $b_org	# load a[0:5]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	$b_org, @acc[0]
	mov	8*3($a_ptr), @acc[3]
	or	@acc[1], $b_org
	mov	8*4($a_ptr), @acc[4]
	or	@acc[2], $b_org
	mov	8*5($a_ptr), @acc[5]
	or	@acc[3], $b_org
	mov	\$-1, @acc[11]
	or	@acc[4], $b_org
	or	@acc[5], $b_org

	mov	8*0($n_ptr), @acc[6]	# load n[0:5]
	cmovnz	@acc[11], $b_org	# mask = a[0:5] ? -1 : 0
	mov	8*1($n_ptr), @acc[7]
	mov	8*2($n_ptr), @acc[8]
	and	$b_org, @acc[6]		# n[0:5] &= mask
	mov	8*3($n_ptr), @acc[9]
	and	$b_org, @acc[7]
	mov	8*4($n_ptr), @acc[10]
	and	$b_org, @acc[8]
	mov	8*5($n_ptr), @acc[11]
	and	$b_org, @acc[9]
	mov	0(%rsp), $n_ptr		# restore condition flag
	and	$b_org, @acc[10]
	and	$b_org, @acc[11]

	sub	@acc[0], @acc[6]	# a[0:5] ? n[0:5]-a[0:5] : 0-0
	sbb	@acc[1], @acc[7]
	sbb	@acc[2], @acc[8]
	sbb	@acc[3], @acc[9]
	sbb	@acc[4], @acc[10]
	sbb	@acc[5], @acc[11]

	or	$n_ptr, $n_ptr		# check condition flag

	cmovz	@acc[0], @acc[6]	# flag ? n[0:5]-a[0:5] : a[0:5]
	cmovz	@acc[1], @acc[7]
	cmovz	@acc[2], @acc[8]
	mov	@acc[6], 8*0($r_ptr)
	cmovz	@acc[3], @acc[9]
	mov	@acc[7], 8*1($r_ptr)
	cmovz	@acc[4], @acc[10]
	mov	@acc[8], 8*2($r_ptr)
	cmovz	@acc[5], @acc[11]
	mov	@acc[9], 8*3($r_ptr)
	mov	@acc[10], 8*4($r_ptr)
	mov	@acc[11], 8*5($r_ptr)

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
.size	cneg_mod_384,.-cneg_mod_384

########################################################################
.globl	sub_mod_384
.hidden	sub_mod_384
.type	sub_mod_384,\@function,4,"unwind"
.align	32
sub_mod_384:
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

	call	__sub_mod_384

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
.size	sub_mod_384,.-sub_mod_384

.type	__sub_mod_384,\@abi-omnipotent
.align	32
__sub_mod_384:
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]

	sub	8*0($b_org), @acc[0]
	 mov	8*0($n_ptr), @acc[6]
	sbb	8*1($b_org), @acc[1]
	 mov	8*1($n_ptr), @acc[7]
	sbb	8*2($b_org), @acc[2]
	 mov	8*2($n_ptr), @acc[8]
	sbb	8*3($b_org), @acc[3]
	 mov	8*3($n_ptr), @acc[9]
	sbb	8*4($b_org), @acc[4]
	 mov	8*4($n_ptr), @acc[10]
	sbb	8*5($b_org), @acc[5]
	 mov	8*5($n_ptr), @acc[11]
	sbb	$b_org, $b_org

	and	$b_org, @acc[6]
	and	$b_org, @acc[7]
	and	$b_org, @acc[8]
	and	$b_org, @acc[9]
	and	$b_org, @acc[10]
	and	$b_org, @acc[11]

	add	@acc[6], @acc[0]
	adc	@acc[7], @acc[1]
	mov	@acc[0], 8*0($r_ptr)
	adc	@acc[8], @acc[2]
	mov	@acc[1], 8*1($r_ptr)
	adc	@acc[9], @acc[3]
	mov	@acc[2], 8*2($r_ptr)
	adc	@acc[10], @acc[4]
	mov	@acc[3], 8*3($r_ptr)
	adc	@acc[11], @acc[5]
	mov	@acc[4], 8*4($r_ptr)
	mov	@acc[5], 8*5($r_ptr)

	ret
.size	__sub_mod_384,.-__sub_mod_384

.globl	sub_mod_384x
.hidden	sub_mod_384x
.type	sub_mod_384x,\@function,4,"unwind"
.align	32
sub_mod_384x:
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
	sub	\$24, %rsp
.cfi_adjust_cfa_offset	24
.cfi_end_prologue

	mov	$a_ptr, 8*0(%rsp)
	mov	$b_org, 8*1(%rsp)
	lea	48($a_ptr), $a_ptr	# a->im
	lea	48($b_org), $b_org	# b->im
	lea	48($r_ptr), $r_ptr	# ret->im
	call	__sub_mod_384		# sub_mod_384(ret->im, a->im, b->im, mod);

	mov	8*0(%rsp), $a_ptr	# a->re
	mov	8*1(%rsp), $b_org	# b->re
	lea	-48($r_ptr), $r_ptr	# ret->re
	call	__sub_mod_384		# sub_mod_384(ret->re, a->re, b->re, mod);

	mov	24+8*0(%rsp),%r15
.cfi_restore	%r15
	mov	24+8*1(%rsp),%r14
.cfi_restore	%r14
	mov	24+8*2(%rsp),%r13
.cfi_restore	%r13
	mov	24+8*3(%rsp),%r12
.cfi_restore	%r12
	mov	24+8*4(%rsp),%rbx
.cfi_restore	%rbx
	mov	24+8*5(%rsp),%rbp
.cfi_restore	%rbp
	lea	24+8*6(%rsp),%rsp
.cfi_adjust_cfa_offset	-24-8*6
.cfi_epilogue
	ret
.cfi_endproc
.size	sub_mod_384x,.-sub_mod_384x
___
}
{ ###################################################### ret = a * (1 + i)
my ($r_ptr,$a_ptr,$n_ptr) = ("%rdi","%rsi","%rdx");
my @acc=map("%r$_",(8..15, "ax", "bx", "cx", "bp"));

$code.=<<___;
.globl	mul_by_1_plus_i_mod_384x
.hidden	mul_by_1_plus_i_mod_384x
.type	mul_by_1_plus_i_mod_384x,\@function,3,"unwind"
.align	32
mul_by_1_plus_i_mod_384x:
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
	sub	\$56, %rsp
.cfi_adjust_cfa_offset	56
.cfi_end_prologue

	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]

	mov	@acc[0], @acc[6]
	add	8*6($a_ptr), @acc[0]	# a->re + a->im
	mov	@acc[1], @acc[7]
	adc	8*7($a_ptr), @acc[1]
	mov	@acc[2], @acc[8]
	adc	8*8($a_ptr), @acc[2]
	mov	@acc[3], @acc[9]
	adc	8*9($a_ptr), @acc[3]
	mov	@acc[4], @acc[10]
	adc	8*10($a_ptr), @acc[4]
	mov	@acc[5], @acc[11]
	adc	8*11($a_ptr), @acc[5]
	mov	$r_ptr, 8*6(%rsp)	# offload r_ptr
	sbb	$r_ptr, $r_ptr

	sub	8*6($a_ptr), @acc[6]	# a->re - a->im
	sbb	8*7($a_ptr), @acc[7]
	sbb	8*8($a_ptr), @acc[8]
	sbb	8*9($a_ptr), @acc[9]
	sbb	8*10($a_ptr), @acc[10]
	sbb	8*11($a_ptr), @acc[11]
	sbb	$a_ptr, $a_ptr

	mov	@acc[0], 8*0(%rsp)	# offload a->re + a->im [without carry]
	 mov	8*0($n_ptr), @acc[0]
	mov	@acc[1], 8*1(%rsp)
	 mov	8*1($n_ptr), @acc[1]
	mov	@acc[2], 8*2(%rsp)
	 mov	8*2($n_ptr), @acc[2]
	mov	@acc[3], 8*3(%rsp)
	 mov	8*3($n_ptr), @acc[3]
	mov	@acc[4], 8*4(%rsp)
	 and	$a_ptr, @acc[0]
	 mov	8*4($n_ptr), @acc[4]
	mov	@acc[5], 8*5(%rsp)
	 and	$a_ptr, @acc[1]
	 mov	8*5($n_ptr), @acc[5]
	 and	$a_ptr, @acc[2]
	 and	$a_ptr, @acc[3]
	 and	$a_ptr, @acc[4]
	 and	$a_ptr, @acc[5]
	mov	8*6(%rsp), $a_ptr	# restore r_ptr

	add	@acc[0], @acc[6]
	 mov	8*0(%rsp), @acc[0]	# restore a->re + a->im
	adc	@acc[1], @acc[7]
	 mov	8*1(%rsp), @acc[1]
	adc	@acc[2], @acc[8]
	 mov	8*2(%rsp), @acc[2]
	adc	@acc[3], @acc[9]
	 mov	8*3(%rsp), @acc[3]
	adc	@acc[4], @acc[10]
	 mov	8*4(%rsp), @acc[4]
	adc	@acc[5], @acc[11]
	 mov	8*5(%rsp), @acc[5]

	mov	@acc[6], 8*0($a_ptr)	# ret->re = a->re - a->im
	 mov	@acc[0], @acc[6]
	mov	@acc[7], 8*1($a_ptr)
	mov	@acc[8], 8*2($a_ptr)
	 mov	@acc[1], @acc[7]
	mov	@acc[9], 8*3($a_ptr)
	mov	@acc[10], 8*4($a_ptr)
	 mov	@acc[2], @acc[8]
	mov	@acc[11], 8*5($a_ptr)

	sub	8*0($n_ptr), @acc[0]
	 mov	@acc[3], @acc[9]
	sbb	8*1($n_ptr), @acc[1]
	sbb	8*2($n_ptr), @acc[2]
	 mov	@acc[4], @acc[10]
	sbb	8*3($n_ptr), @acc[3]
	sbb	8*4($n_ptr), @acc[4]
	 mov	@acc[5], @acc[11]
	sbb	8*5($n_ptr), @acc[5]
	sbb	\$0, $r_ptr

	cmovc	@acc[6], @acc[0]
	cmovc	@acc[7], @acc[1]
	cmovc	@acc[8], @acc[2]
	mov	@acc[0], 8*6($a_ptr)	# ret->im = a->re + a->im
	cmovc	@acc[9], @acc[3]
	mov	@acc[1], 8*7($a_ptr)
	cmovc	@acc[10], @acc[4]
	mov	@acc[2], 8*8($a_ptr)
	cmovc	@acc[11], @acc[5]
	mov	@acc[3], 8*9($a_ptr)
	mov	@acc[4], 8*10($a_ptr)
	mov	@acc[5], 8*11($a_ptr)

	mov	56+8*0(%rsp),%r15
.cfi_restore	%r15
	mov	56+8*1(%rsp),%r14
.cfi_restore	%r14
	mov	56+8*2(%rsp),%r13
.cfi_restore	%r13
	mov	56+8*3(%rsp),%r12
.cfi_restore	%r12
	mov	56+8*4(%rsp),%rbx
.cfi_restore	%rbx
	mov	56+8*5(%rsp),%rbp
.cfi_restore	%rbp
	lea	56+8*6(%rsp),%rsp
.cfi_adjust_cfa_offset	-56-8*6
.cfi_epilogue
	ret
.cfi_endproc
.size	mul_by_1_plus_i_mod_384x,.-mul_by_1_plus_i_mod_384x
___
}
{ ######################################################
my ($r_ptr,$n_ptr) = ("%rdi","%rsi");
my @acc=map("%r$_",(8..11, "cx", "dx", "bx", "bp"));

$code.=<<___;
.globl	sgn0_pty_mod_384
.hidden	sgn0_pty_mod_384
.type	sgn0_pty_mod_384,\@function,2,"unwind"
.align	32
sgn0_pty_mod_384:
.cfi_startproc
.cfi_end_prologue
	mov	8*0($r_ptr), @acc[0]
	mov	8*1($r_ptr), @acc[1]
	mov	8*2($r_ptr), @acc[2]
	mov	8*3($r_ptr), @acc[3]
	mov	8*4($r_ptr), @acc[4]
	mov	8*5($r_ptr), @acc[5]

	xor	%rax, %rax
	mov	@acc[0], $r_ptr
	add	@acc[0], @acc[0]
	adc	@acc[1], @acc[1]
	adc	@acc[2], @acc[2]
	adc	@acc[3], @acc[3]
	adc	@acc[4], @acc[4]
	adc	@acc[5], @acc[5]
	adc	\$0, %rax

	sub	8*0($n_ptr), @acc[0]
	sbb	8*1($n_ptr), @acc[1]
	sbb	8*2($n_ptr), @acc[2]
	sbb	8*3($n_ptr), @acc[3]
	sbb	8*4($n_ptr), @acc[4]
	sbb	8*5($n_ptr), @acc[5]
	sbb	\$0, %rax

	not	%rax			# 2*x > p, which means "negative"
	and	\$1, $r_ptr
	and	\$2, %rax
	or	$r_ptr, %rax		# pack sign and parity

.cfi_epilogue
	ret
.cfi_endproc
.size	sgn0_pty_mod_384,.-sgn0_pty_mod_384

.globl	sgn0_pty_mod_384x
.hidden	sgn0_pty_mod_384x
.type	sgn0_pty_mod_384x,\@function,2,"unwind"
.align	32
sgn0_pty_mod_384x:
.cfi_startproc
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	8*6($r_ptr), @acc[0]	# sgn0(a->im)
	mov	8*7($r_ptr), @acc[1]
	mov	8*8($r_ptr), @acc[2]
	mov	8*9($r_ptr), @acc[3]
	mov	8*10($r_ptr), @acc[4]
	mov	8*11($r_ptr), @acc[5]

	mov	@acc[0], @acc[6]
	or	@acc[1], @acc[0]
	or	@acc[2], @acc[0]
	or	@acc[3], @acc[0]
	or	@acc[4], @acc[0]
	or	@acc[5], @acc[0]

	lea	0($r_ptr), %rax		# sgn0(a->re)
	xor	$r_ptr, $r_ptr
	mov	@acc[6], @acc[7]
	add	@acc[6], @acc[6]
	adc	@acc[1], @acc[1]
	adc	@acc[2], @acc[2]
	adc	@acc[3], @acc[3]
	adc	@acc[4], @acc[4]
	adc	@acc[5], @acc[5]
	adc	\$0, $r_ptr

	sub	8*0($n_ptr), @acc[6]
	sbb	8*1($n_ptr), @acc[1]
	sbb	8*2($n_ptr), @acc[2]
	sbb	8*3($n_ptr), @acc[3]
	sbb	8*4($n_ptr), @acc[4]
	sbb	8*5($n_ptr), @acc[5]
	sbb	\$0, $r_ptr

	mov	@acc[0], 0(%rsp)	# a->im is zero or not
	not	$r_ptr			# 2*x > p, which means "negative"
	and	\$1, @acc[7]
	and	\$2, $r_ptr
	or	@acc[7], $r_ptr		# pack sign and parity

	mov	8*0(%rax), @acc[0]
	mov	8*1(%rax), @acc[1]
	mov	8*2(%rax), @acc[2]
	mov	8*3(%rax), @acc[3]
	mov	8*4(%rax), @acc[4]
	mov	8*5(%rax), @acc[5]

	mov	@acc[0], @acc[6]
	or	@acc[1], @acc[0]
	or	@acc[2], @acc[0]
	or	@acc[3], @acc[0]
	or	@acc[4], @acc[0]
	or	@acc[5], @acc[0]

	xor	%rax, %rax
	mov	@acc[6], @acc[7]
	add	@acc[6], @acc[6]
	adc	@acc[1], @acc[1]
	adc	@acc[2], @acc[2]
	adc	@acc[3], @acc[3]
	adc	@acc[4], @acc[4]
	adc	@acc[5], @acc[5]
	adc	\$0, %rax

	sub	8*0($n_ptr), @acc[6]
	sbb	8*1($n_ptr), @acc[1]
	sbb	8*2($n_ptr), @acc[2]
	sbb	8*3($n_ptr), @acc[3]
	sbb	8*4($n_ptr), @acc[4]
	sbb	8*5($n_ptr), @acc[5]
	sbb	\$0, %rax

	mov	0(%rsp), @acc[6]

	not	%rax			# 2*x > p, which means "negative"

	test	@acc[0], @acc[0]
	cmovz	$r_ptr, @acc[7]		# a->re==0? prty(a->im) : prty(a->re)

	test	@acc[6], @acc[6]
	cmovnz	$r_ptr, %rax		# a->im!=0? sgn0(a->im) : sgn0(a->re)

	and	\$1, @acc[7]
	and	\$2, %rax
	or	@acc[7], %rax		# pack sign and parity

	mov	8(%rsp), %rbx
.cfi_restore	%rbx
	mov	16(%rsp), %rbp
.cfi_restore	%rbp
	lea	24(%rsp), %rsp
.cfi_adjust_cfa_offset	-24
.cfi_epilogue
	ret
.cfi_endproc
.size	sgn0_pty_mod_384x,.-sgn0_pty_mod_384x
___
}
if (0) {
my $inp = $win64 ? "%rcx" : "%rdi";
$code.=<<___;
.globl	nbits_384
.hidden	nbits_384
.type	nbits_384,\@abi-omnipotent
.align	32
nbits_384:
	mov	8*5($inp), %r8
	mov	8*4($inp), %r9
	mov	8*3($inp), %r10
	mov	8*2($inp), %r11
	mov	\$-1, %rdx
	mov	\$127, %eax
	bsr	%r8, %r8
	cmovnz	%rdx,%r9
	cmovz	%rax,%r8
	bsr	%r9, %r9
	cmovnz	%rdx,%r10
	cmovz	%rax,%r9
	xor	\$63,%r8
	bsr	%r10, %r10
	cmovnz	%rdx, %r11
	cmovz	%rax, %r10
	xor	\$63,%r9
	add	%r8, %r9
	mov	8*1($inp), %r8
	bsr	%r11, %r11
	cmovnz	%rdx, %r8
	cmovz	%rax, %r11
	xor	\$63, %r10
	add	%r9, %r10
	mov	8*0($inp), %r9
	bsr	%r8, %r8
	cmovnz	%rdx, %r9
	cmovz	%rax, %r8
	xor	\$63, %r11
	add	%r10, %r11
	bsr	%r9, %r9
	cmovz	%rax, %r9
	xor	\$63, %r8
	add	%r11, %r8
	xor	\$63, %r9
	add	%r8, %r9
	mov	\$384, %eax
	sub	%r9, %rax
	ret
.size	nbits_384,.-nbits_384
___
}

if (1) {
my ($out, $inp1, $inp2, $select) = $win64 ? ("%rcx", "%rdx", "%r8", "%r9d")
                                          : ("%rdi", "%rsi", "%rdx", "%ecx");

sub vec_select {
my $sz = shift;
my $half = $sz/2;
my ($xmm0,$xmm1,$xmm2,$xmm3)=map("%xmm$_",(0..3));

$code.=<<___;
.globl	vec_select_$sz
.hidden	vec_select_$sz
.type	vec_select_$sz,\@abi-omnipotent
.align	32
vec_select_$sz:
	movd	$select, %xmm5
	pxor	%xmm4,%xmm4
	pshufd	\$0,%xmm5,%xmm5		# broadcast
	movdqu	($inp1),$xmm0
	lea	$half($inp1),$inp1
	pcmpeqd	%xmm4,%xmm5
	movdqu	($inp2),$xmm1
	lea	$half($inp2),$inp2
	pcmpeqd	%xmm5,%xmm4
	lea	$half($out),$out
___
for($i=0; $i<$sz-16; $i+=16) {
$code.=<<___;
	pand	%xmm4,$xmm0
	movdqu	$i+16-$half($inp1),$xmm2
	pand	%xmm5,$xmm1
	movdqu	$i+16-$half($inp2),$xmm3
	por	$xmm1,$xmm0
	movdqu	$xmm0,$i-$half($out)
___
	($xmm0,$xmm1,$xmm2,$xmm3)=($xmm2,$xmm3,$xmm0,$xmm1);
}
$code.=<<___;
	pand	%xmm4,$xmm0
	pand	%xmm5,$xmm1
	por	$xmm1,$xmm0
	movdqu	$xmm0,$i-$half($out)
	ret
.size	vec_select_$sz,.-vec_select_$sz
___
}
vec_select(32);
vec_select(48);
vec_select(96);
vec_select(192);
vec_select(144);
vec_select(288);
}

{
my ($inp, $end) = $win64 ? ("%rcx", "%rdx") : ("%rdi", "%rsi");

$code.=<<___;
.globl	vec_prefetch
.hidden	vec_prefetch
.type	vec_prefetch,\@abi-omnipotent
.align	32
vec_prefetch:
	leaq		-1($inp,$end), $end
	mov		\$64, %rax
	xor		%r8, %r8
	prefetchnta	($inp)
	lea		($inp,%rax), $inp
	cmp		$end, $inp
	cmova		$end, $inp
	cmova		%r8, %rax
	prefetchnta	($inp)
	lea		($inp,%rax), $inp
	cmp		$end, $inp
	cmova		$end, $inp
	cmova		%r8, %rax
	prefetchnta	($inp)
	lea		($inp,%rax), $inp
	cmp		$end, $inp
	cmova		$end, $inp
	cmova		%r8, %rax
	prefetchnta	($inp)
	lea		($inp,%rax), $inp
	cmp		$end, $inp
	cmova		$end, $inp
	cmova		%r8, %rax
	prefetchnta	($inp)
	lea		($inp,%rax), $inp
	cmp		$end, $inp
	cmova		$end, $inp
	cmova		%r8, %rax
	prefetchnta	($inp)
	lea		($inp,%rax), $inp
	cmp		$end, $inp
	cmova		$end, $inp
	prefetchnta	($inp)
	ret
.size	vec_prefetch,.-vec_prefetch
___
my $len = $win64 ? "%edx" : "%esi";

$code.=<<___;
.globl	vec_is_zero_16x
.hidden	vec_is_zero_16x
.type	vec_is_zero_16x,\@abi-omnipotent
.align	32
vec_is_zero_16x:
	shr		\$4, $len
	movdqu		($inp), %xmm0
	lea		16($inp), $inp

.Loop_is_zero:
	dec		$len
	jz		.Loop_is_zero_done
	movdqu		($inp), %xmm1
	lea		16($inp), $inp
	por		%xmm1, %xmm0
	jmp		.Loop_is_zero

.Loop_is_zero_done:
	pshufd		\$0x4e, %xmm0, %xmm1
	por		%xmm1, %xmm0
	movq		%xmm0, %rax
	inc		$len			# now it's 1
	test		%rax, %rax
	cmovnz		$len, %eax
	xor		\$1, %eax
	ret
.size	vec_is_zero_16x,.-vec_is_zero_16x
___
}
{
my ($inp1, $inp2, $len) = $win64 ? ("%rcx", "%rdx", "%r8d")
                                 : ("%rdi", "%rsi", "%edx");
$code.=<<___;
.globl	vec_is_equal_16x
.hidden	vec_is_equal_16x
.type	vec_is_equal_16x,\@abi-omnipotent
.align	32
vec_is_equal_16x:
	shr		\$4, $len
	movdqu		($inp1), %xmm0
	movdqu		($inp2), %xmm1
	sub		$inp1, $inp2
	lea		16($inp1), $inp1
	pxor		%xmm1, %xmm0

.Loop_is_equal:
	dec		$len
	jz		.Loop_is_equal_done
	movdqu		($inp1), %xmm1
	movdqu		($inp1,$inp2), %xmm2
	lea		16($inp1), $inp1
	pxor		%xmm2, %xmm1
	por		%xmm1, %xmm0
	jmp		.Loop_is_equal

.Loop_is_equal_done:
	pshufd		\$0x4e, %xmm0, %xmm1
	por		%xmm1, %xmm0
	movq		%xmm0, %rax
	inc		$len			# now it's 1
	test		%rax, %rax
	cmovnz		$len, %eax
	xor		\$1, %eax
	ret
.size	vec_is_equal_16x,.-vec_is_equal_16x
___
}
print $code;
close STDOUT;
