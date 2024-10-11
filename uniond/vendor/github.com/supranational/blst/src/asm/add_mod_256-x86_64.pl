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
($r_ptr,$a_ptr,$b_org,$n_ptr) = ("%rdi","%rsi","%rdx","%rcx");
$b_ptr = "%rbx";

{ ############################################################## 256 bits add
my @acc=map("%r$_",(8..11, "ax", "si", "bx", "bp", 12));

$code.=<<___;
.text

.globl	add_mod_256
.hidden	add_mod_256
.type	add_mod_256,\@function,4,"unwind"
.align	32
add_mod_256:
.cfi_startproc
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]

.Loaded_a_add_mod_256:
	add	8*0($b_org), @acc[0]
	adc	8*1($b_org), @acc[1]
	 mov	@acc[0], @acc[4]
	adc	8*2($b_org), @acc[2]
	 mov	@acc[1], @acc[5]
	adc	8*3($b_org), @acc[3]
	sbb	$b_org, $b_org

	 mov	@acc[2], @acc[6]
	sub	8*0($n_ptr), @acc[0]
	sbb	8*1($n_ptr), @acc[1]
	sbb	8*2($n_ptr), @acc[2]
	 mov	@acc[3], @acc[7]
	sbb	8*3($n_ptr), @acc[3]
	sbb	\$0, $b_org

	cmovc	@acc[4], @acc[0]
	cmovc	@acc[5], @acc[1]
	mov	@acc[0], 8*0($r_ptr)
	cmovc	@acc[6], @acc[2]
	mov	@acc[1], 8*1($r_ptr)
	cmovc	@acc[7], @acc[3]
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)

	mov	8(%rsp),%rbx
.cfi_restore	%rbx
	mov	16(%rsp),%rbp
.cfi_restore	%rbp
	lea	24(%rsp),%rsp
.cfi_adjust_cfa_offset	-24
.cfi_epilogue
	ret
.cfi_endproc
.size	add_mod_256,.-add_mod_256

########################################################################
.globl	mul_by_3_mod_256
.hidden	mul_by_3_mod_256
.type	mul_by_3_mod_256,\@function,3,"unwind"
.align	32
mul_by_3_mod_256:
.cfi_startproc
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	push	%r12
.cfi_push	%r12
.cfi_end_prologue

	mov	$b_org,$n_ptr
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	$a_ptr,$b_org
	mov	8*3($a_ptr), @acc[3]

	call	__lshift_mod_256
	mov	0(%rsp),%r12
.cfi_restore	%r12
	jmp	.Loaded_a_add_mod_256

	mov	8(%rsp),%rbx
.cfi_restore	%rbx
	mov	16(%rsp),%rbp
.cfi_restore	%rbp
	lea	24(%rsp),%rsp
.cfi_adjust_cfa_offset	-24
.cfi_epilogue
	ret
.cfi_endproc
.size	mul_by_3_mod_256,.-mul_by_3_mod_256

.type	__lshift_mod_256,\@abi-omnipotent
.align	32
__lshift_mod_256:
	add	@acc[0], @acc[0]
	adc	@acc[1], @acc[1]
	 mov	@acc[0], @acc[4]
	adc	@acc[2], @acc[2]
	 mov	@acc[1], @acc[5]
	adc	@acc[3], @acc[3]
	sbb	@acc[8], @acc[8]

	 mov	@acc[2], @acc[6]
	sub	8*0($n_ptr), @acc[0]
	sbb	8*1($n_ptr), @acc[1]
	sbb	8*2($n_ptr), @acc[2]
	 mov	@acc[3], @acc[7]
	sbb	8*3($n_ptr), @acc[3]
	sbb	\$0, @acc[8]

	cmovc	@acc[4], @acc[0]
	cmovc	@acc[5], @acc[1]
	cmovc	@acc[6], @acc[2]
	cmovc	@acc[7], @acc[3]

	ret	# __SGX_LVI_HARDENING_CLOBBER__=@acc[4]
.size	__lshift_mod_256,.-__lshift_mod_256

########################################################################
.globl	lshift_mod_256
.hidden	lshift_mod_256
.type	lshift_mod_256,\@function,4,"unwind"
.align	32
lshift_mod_256:
.cfi_startproc
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	push	%r12
.cfi_push	%r12
.cfi_end_prologue

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]

.Loop_lshift_mod_256:
	call	__lshift_mod_256
	dec	%edx
	jnz	.Loop_lshift_mod_256

	mov	@acc[0], 8*0($r_ptr)
	mov	@acc[1], 8*1($r_ptr)
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)

	mov	0(%rsp),%r12
.cfi_restore	%r12
	mov	8(%rsp),%rbx
.cfi_restore	%rbx
	mov	16(%rsp),%rbp
.cfi_restore	%rbp
	lea	24(%rsp),%rsp
.cfi_adjust_cfa_offset	-24
.cfi_epilogue
	ret
.cfi_endproc
.size	lshift_mod_256,.-lshift_mod_256

########################################################################
.globl	rshift_mod_256
.hidden	rshift_mod_256
.type	rshift_mod_256,\@function,4,"unwind"
.align	32
rshift_mod_256:
.cfi_startproc
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	mov	8*0($a_ptr), @acc[7]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]

.Loop_rshift_mod_256:
	mov	@acc[7], @acc[0]
	and	\$1, @acc[7]
	mov	8*0($n_ptr), @acc[4]
	neg	@acc[7]
	mov	8*1($n_ptr), @acc[5]
	mov	8*2($n_ptr), @acc[6]

	and	@acc[7], @acc[4]
	and	@acc[7], @acc[5]
	and	@acc[7], @acc[6]
	and	8*3($n_ptr), @acc[7]

	add	@acc[4], @acc[0]
	adc	@acc[5], @acc[1]
	adc	@acc[6], @acc[2]
	adc	@acc[7], @acc[3]
	sbb	@acc[4], @acc[4]

	shr	\$1, @acc[0]
	mov	@acc[1], @acc[7]
	shr	\$1, @acc[1]
	mov	@acc[2], @acc[6]
	shr	\$1, @acc[2]
	mov	@acc[3], @acc[5]
	shr	\$1, @acc[3]

	shl	\$63, @acc[7]
	shl	\$63, @acc[6]
	or	@acc[0], @acc[7]
	shl	\$63, @acc[5]
	or	@acc[6], @acc[1]
	shl	\$63, @acc[4]
	or	@acc[5], @acc[2]
	or	@acc[4], @acc[3]

	dec	%edx
	jnz	.Loop_rshift_mod_256

	mov	@acc[7], 8*0($r_ptr)
	mov	@acc[1], 8*1($r_ptr)
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)

	mov	8(%rsp),%rbx
.cfi_restore	%rbx
	mov	16(%rsp),%rbp
.cfi_restore	%rbp
	lea	24(%rsp),%rsp
.cfi_adjust_cfa_offset	-24
.cfi_epilogue
	ret
.cfi_endproc
.size	rshift_mod_256,.-rshift_mod_256

########################################################################
.globl	cneg_mod_256
.hidden	cneg_mod_256
.type	cneg_mod_256,\@function,4,"unwind"
.align	32
cneg_mod_256:
.cfi_startproc
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	push	%r12
.cfi_push	%r12
.cfi_end_prologue

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	mov	8*0($a_ptr), @acc[8]	# load a[0:3]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	@acc[8], @acc[0]
	mov	8*3($a_ptr), @acc[3]
	or	@acc[1], @acc[8]
	or	@acc[2], @acc[8]
	or	@acc[3], @acc[8]
	mov	\$-1, @acc[7]

	mov	8*0($n_ptr), @acc[4]	# load n[0:3]
	cmovnz	@acc[7], @acc[8]	# mask = a[0:3] ? -1 : 0
	mov	8*1($n_ptr), @acc[5]
	mov	8*2($n_ptr), @acc[6]
	and	@acc[8], @acc[4]	# n[0:3] &= mask
	mov	8*3($n_ptr), @acc[7]
	and	@acc[8], @acc[5]
	and	@acc[8], @acc[6]
	and	@acc[8], @acc[7]

	sub	@acc[0], @acc[4]	# a[0:3] ? n[0:3]-a[0:3] : 0-0
	sbb	@acc[1], @acc[5]
	sbb	@acc[2], @acc[6]
	sbb	@acc[3], @acc[7]

	or	$b_org, $b_org		# check condition flag

	cmovz	@acc[0], @acc[4]	# flag ? n[0:3]-a[0:3] : a[0:3]
	cmovz	@acc[1], @acc[5]
	mov	@acc[4], 8*0($r_ptr)
	cmovz	@acc[2], @acc[6]
	mov	@acc[5], 8*1($r_ptr)
	cmovz	@acc[3], @acc[7]
	mov	@acc[6], 8*2($r_ptr)
	mov	@acc[7], 8*3($r_ptr)

	mov	0(%rsp),%r12
.cfi_restore	%r12
	mov	8(%rsp),%rbx
.cfi_restore	%rbx
	mov	16(%rsp),%rbp
.cfi_restore	%rbp
	lea	24(%rsp),%rsp
.cfi_adjust_cfa_offset	-24
.cfi_epilogue
	ret
.cfi_endproc
.size	cneg_mod_256,.-cneg_mod_256

########################################################################
.globl	sub_mod_256
.hidden	sub_mod_256
.type	sub_mod_256,\@function,4,"unwind"
.align	32
sub_mod_256:
.cfi_startproc
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]

	sub	8*0($b_org), @acc[0]
	 mov	8*0($n_ptr), @acc[4]
	sbb	8*1($b_org), @acc[1]
	 mov	8*1($n_ptr), @acc[5]
	sbb	8*2($b_org), @acc[2]
	 mov	8*2($n_ptr), @acc[6]
	sbb	8*3($b_org), @acc[3]
	 mov	8*3($n_ptr), @acc[7]
	sbb	$b_org, $b_org

	and	$b_org, @acc[4]
	and	$b_org, @acc[5]
	and	$b_org, @acc[6]
	and	$b_org, @acc[7]

	add	@acc[4], @acc[0]
	adc	@acc[5], @acc[1]
	mov	@acc[0], 8*0($r_ptr)
	adc	@acc[6], @acc[2]
	mov	@acc[1], 8*1($r_ptr)
	adc	@acc[7], @acc[3]
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)

	mov	8(%rsp),%rbx
.cfi_restore	%rbx
	mov	16(%rsp),%rbp
.cfi_restore	%rbp
	lea	24(%rsp),%rsp
.cfi_adjust_cfa_offset	-24
.cfi_epilogue
	ret
.cfi_endproc
.size	sub_mod_256,.-sub_mod_256

########################################################################
.globl	check_mod_256
.hidden	check_mod_256
.type	check_mod_256,\@function,2,"unwind"
.align	32
check_mod_256:
.cfi_startproc
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	mov	8*0($r_ptr), %rax
	mov	8*1($r_ptr), @acc[1]
	mov	8*2($r_ptr), @acc[2]
	mov	8*3($r_ptr), @acc[3]

	mov	%rax, @acc[0]		# see if it's zero
	or	@acc[1], %rax
	or	@acc[2], %rax
	or	@acc[3], %rax

	sub	8*0($a_ptr), @acc[0]	# does subtracting modulus borrow?
	sbb	8*1($a_ptr), @acc[1]
	sbb	8*2($a_ptr), @acc[2]
	sbb	8*3($a_ptr), @acc[3]
	sbb	$a_ptr, $a_ptr

	mov	\$1, %rdx
	cmp	\$0, %rax
	cmovne	%rdx, %rax
	and	$a_ptr, %rax
.cfi_epilogue
	ret
.cfi_endproc
.size	check_mod_256,.-check_mod_256

########################################################################
.globl	add_n_check_mod_256
.hidden	add_n_check_mod_256
.type	add_n_check_mod_256,\@function,4,"unwind"
.align	32
add_n_check_mod_256:
.cfi_startproc
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]

	add	8*0($b_org), @acc[0]
	adc	8*1($b_org), @acc[1]
	 mov	@acc[0], @acc[4]
	adc	8*2($b_org), @acc[2]
	 mov	@acc[1], @acc[5]
	adc	8*3($b_org), @acc[3]
	sbb	$b_org, $b_org

	 mov	@acc[2], @acc[6]
	sub	8*0($n_ptr), @acc[0]
	sbb	8*1($n_ptr), @acc[1]
	sbb	8*2($n_ptr), @acc[2]
	 mov	@acc[3], @acc[7]
	sbb	8*3($n_ptr), @acc[3]
	sbb	\$0, $b_org

	cmovc	@acc[4], @acc[0]
	cmovc	@acc[5], @acc[1]
	mov	@acc[0], 8*0($r_ptr)
	cmovc	@acc[6], @acc[2]
	mov	@acc[1], 8*1($r_ptr)
	cmovc	@acc[7], @acc[3]
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)

	or	@acc[1], @acc[0]
	or	@acc[3], @acc[2]
	or	@acc[2], @acc[0]
	mov	\$1, %rax
	cmovz	@acc[0], %rax

	mov	8(%rsp),%rbx
.cfi_restore	%rbx
	mov	16(%rsp),%rbp
.cfi_restore	%rbp
	lea	24(%rsp),%rsp
.cfi_adjust_cfa_offset	-24
.cfi_epilogue
	ret
.cfi_endproc
.size	add_n_check_mod_256,.-add_n_check_mod_256

########################################################################
.globl	sub_n_check_mod_256
.hidden	sub_n_check_mod_256
.type	sub_n_check_mod_256,\@function,4,"unwind"
.align	32
sub_n_check_mod_256:
.cfi_startproc
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]

	sub	8*0($b_org), @acc[0]
	 mov	8*0($n_ptr), @acc[4]
	sbb	8*1($b_org), @acc[1]
	 mov	8*1($n_ptr), @acc[5]
	sbb	8*2($b_org), @acc[2]
	 mov	8*2($n_ptr), @acc[6]
	sbb	8*3($b_org), @acc[3]
	 mov	8*3($n_ptr), @acc[7]
	sbb	$b_org, $b_org

	and	$b_org, @acc[4]
	and	$b_org, @acc[5]
	and	$b_org, @acc[6]
	and	$b_org, @acc[7]

	add	@acc[4], @acc[0]
	adc	@acc[5], @acc[1]
	mov	@acc[0], 8*0($r_ptr)
	adc	@acc[6], @acc[2]
	mov	@acc[1], 8*1($r_ptr)
	adc	@acc[7], @acc[3]
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)

	or	@acc[1], @acc[0]
	or	@acc[3], @acc[2]
	or	@acc[2], @acc[0]
	mov	\$1, %rax
	cmovz	@acc[0], %rax

	mov	8(%rsp),%rbx
.cfi_restore	%rbx
	mov	16(%rsp),%rbp
.cfi_restore	%rbp
	lea	24(%rsp),%rsp
.cfi_adjust_cfa_offset	-24
.cfi_epilogue
	ret
.cfi_endproc
.size	sub_n_check_mod_256,.-sub_n_check_mod_256
___
}

print $code;
close STDOUT;
