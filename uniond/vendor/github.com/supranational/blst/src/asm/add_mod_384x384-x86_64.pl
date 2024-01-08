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

# common accumulator layout
@acc=map("%r$_",(8..15));

############################################################ 384x384 add/sub
# Double-width addition/subtraction modulo n<<384, as opposite to
# naively expected modulo n*n. It works because n<<384 is the actual
# input boundary condition for Montgomery reduction, not n*n.
# Just in case, this is duplicated, but only one module is
# supposed to be linked...
{
my @acc=(@acc,"%rax","%rbx","%rbp",$a_ptr);	# all registers are affected
						# except for $n_ptr and $r_ptr
$code.=<<___;
.text

.type	__add_mod_384x384,\@abi-omnipotent
.align	32
__add_mod_384x384:
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]
	mov	8*6($a_ptr), @acc[6]

	add	8*0($b_org), @acc[0]
	mov	8*7($a_ptr), @acc[7]
	adc	8*1($b_org), @acc[1]
	mov	8*8($a_ptr), @acc[8]
	adc	8*2($b_org), @acc[2]
	mov	8*9($a_ptr), @acc[9]
	adc	8*3($b_org), @acc[3]
	mov	8*10($a_ptr), @acc[10]
	adc	8*4($b_org), @acc[4]
	mov	8*11($a_ptr), @acc[11]
	adc	8*5($b_org), @acc[5]
	 mov	@acc[0], 8*0($r_ptr)
	adc	8*6($b_org), @acc[6]
	 mov	@acc[1], 8*1($r_ptr)
	adc	8*7($b_org), @acc[7]
	 mov	@acc[2], 8*2($r_ptr)
	adc	8*8($b_org), @acc[8]
	 mov	@acc[4], 8*4($r_ptr)
	 mov	@acc[6], @acc[0]
	adc	8*9($b_org), @acc[9]
	 mov	@acc[3], 8*3($r_ptr)
	 mov	@acc[7], @acc[1]
	adc	8*10($b_org), @acc[10]
	 mov	@acc[5], 8*5($r_ptr)
	 mov	@acc[8], @acc[2]
	adc	8*11($b_org), @acc[11]
	 mov	@acc[9], @acc[3]
	sbb	$b_org, $b_org

	sub	8*0($n_ptr), @acc[6]
	sbb	8*1($n_ptr), @acc[7]
	 mov	@acc[10], @acc[4]
	sbb	8*2($n_ptr), @acc[8]
	sbb	8*3($n_ptr), @acc[9]
	sbb	8*4($n_ptr), @acc[10]
	 mov	@acc[11], @acc[5]
	sbb	8*5($n_ptr), @acc[11]
	sbb	\$0, $b_org

	cmovc	@acc[0], @acc[6]
	cmovc	@acc[1], @acc[7]
	cmovc	@acc[2], @acc[8]
	mov	@acc[6], 8*6($r_ptr)
	cmovc	@acc[3], @acc[9]
	mov	@acc[7], 8*7($r_ptr)
	cmovc	@acc[4], @acc[10]
	mov	@acc[8], 8*8($r_ptr)
	cmovc	@acc[5], @acc[11]
	mov	@acc[9], 8*9($r_ptr)
	mov	@acc[10], 8*10($r_ptr)
	mov	@acc[11], 8*11($r_ptr)

	ret
.size	__add_mod_384x384,.-__add_mod_384x384

.type	__sub_mod_384x384,\@abi-omnipotent
.align	32
__sub_mod_384x384:
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]
	mov	8*6($a_ptr), @acc[6]

	sub	8*0($b_org), @acc[0]
	mov	8*7($a_ptr), @acc[7]
	sbb	8*1($b_org), @acc[1]
	mov	8*8($a_ptr), @acc[8]
	sbb	8*2($b_org), @acc[2]
	mov	8*9($a_ptr), @acc[9]
	sbb	8*3($b_org), @acc[3]
	mov	8*10($a_ptr), @acc[10]
	sbb	8*4($b_org), @acc[4]
	mov	8*11($a_ptr), @acc[11]
	sbb	8*5($b_org), @acc[5]
	 mov	@acc[0], 8*0($r_ptr)
	sbb	8*6($b_org), @acc[6]
	 mov	8*0($n_ptr), @acc[0]
	 mov	@acc[1], 8*1($r_ptr)
	sbb	8*7($b_org), @acc[7]
	 mov	8*1($n_ptr), @acc[1]
	 mov	@acc[2], 8*2($r_ptr)
	sbb	8*8($b_org), @acc[8]
	 mov	8*2($n_ptr), @acc[2]
	 mov	@acc[3], 8*3($r_ptr)
	sbb	8*9($b_org), @acc[9]
	 mov	8*3($n_ptr), @acc[3]
	 mov	@acc[4], 8*4($r_ptr)
	sbb	8*10($b_org), @acc[10]
	 mov	8*4($n_ptr), @acc[4]
	 mov	@acc[5], 8*5($r_ptr)
	sbb	8*11($b_org), @acc[11]
	 mov	8*5($n_ptr), @acc[5]
	sbb	$b_org, $b_org

	and	$b_org, @acc[0]
	and	$b_org, @acc[1]
	and	$b_org, @acc[2]
	and	$b_org, @acc[3]
	and	$b_org, @acc[4]
	and	$b_org, @acc[5]

	add	@acc[0], @acc[6]
	adc	@acc[1], @acc[7]
	mov	@acc[6], 8*6($r_ptr)
	adc	@acc[2], @acc[8]
	mov	@acc[7], 8*7($r_ptr)
	adc	@acc[3], @acc[9]
	mov	@acc[8], 8*8($r_ptr)
	adc	@acc[4], @acc[10]
	mov	@acc[9], 8*9($r_ptr)
	adc	@acc[5], @acc[11]
	mov	@acc[10], 8*10($r_ptr)
	mov	@acc[11], 8*11($r_ptr)

	ret
.size	__sub_mod_384x384,.-__sub_mod_384x384

.globl	add_mod_384x384
.hidden	add_mod_384x384
.type	add_mod_384x384,\@function,4,"unwind"
.align	32
add_mod_384x384:
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

	call	__add_mod_384x384

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
.size	add_mod_384x384,.-add_mod_384x384

.globl	sub_mod_384x384
.hidden	sub_mod_384x384
.type	sub_mod_384x384,\@function,4,"unwind"
.align	32
sub_mod_384x384:
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

	call	__sub_mod_384x384

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
.size	sub_mod_384x384,.-sub_mod_384x384
___
}

print $code;
close STDOUT;
