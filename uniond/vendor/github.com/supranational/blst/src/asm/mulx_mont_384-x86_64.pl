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

########################################################################
{ my @acc=(@acc,"%rax","%rbx","%rbp",$a_ptr);	# all registers are affected
						# except for $n_ptr and $r_ptr
$code.=<<___;
.text

########################################################################
# Double-width subtraction modulo n<<384, as opposite to naively
# expected modulo n*n. It works because n<<384 is the actual
# input boundary condition for Montgomery reduction, not n*n.
# Just in case, this is duplicated, but only one module is
# supposed to be linked...
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

.type	__add_mod_384,\@abi-omnipotent
.align	32
__add_mod_384:
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]

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

.type	__sub_mod_384,\@abi-omnipotent
.align	32
__sub_mod_384:
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]

__sub_mod_384_a_is_loaded:
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
___
}

########################################################################
# "Complex" multiplication and squaring. Use vanilla multiplication when
# possible to fold reductions. I.e. instead of mul_mont, mul_mont
# followed by add/sub_mod, it calls mul, mul, double-width add/sub_mod
# followed by *common* reduction... For single multiplication disjoint
# reduction is bad for performance for given vector length, yet overall
# it's a win here, because it's one reduction less.
{ my $frame = 5*8 +	# place for argument off-load +
	      3*768/8;	# place for 3 768-bit temporary vectors
$code.=<<___;
.globl	mulx_mont_384x
.hidden	mulx_mont_384x
.type	mulx_mont_384x,\@function,5,"unwind"
.align	32
mulx_mont_384x:
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

	mov	$b_org, $b_ptr
	mov	$r_ptr, 8*4(%rsp)	# offload arguments
	mov	$a_ptr, 8*3(%rsp)
	mov	$b_org, 8*2(%rsp)
	mov	$n_ptr, 8*1(%rsp)
	mov	$n0,    8*0(%rsp)

	################################# mul_384(t0, a->re, b->re);
	#lea	0($b_btr), $b_ptr	# b->re
	#lea	0($a_ptr), $a_ptr	# a->re
	lea	40(%rsp), $r_ptr	# t0
	call	__mulx_384

	################################# mul_384(t1, a->im, b->im);
	lea	48($b_ptr), $b_ptr	# b->im
	lea	128+48($a_ptr), $a_ptr	# a->im
	lea	96($r_ptr), $r_ptr	# t1
	call	__mulx_384

	################################# mul_384(t2, a->re+a->im, b->re+b->im);
	mov	8*1(%rsp), $n_ptr
	lea	($b_ptr), $a_ptr	# b->re
	lea	-48($b_ptr), $b_org	# b->im
	lea	40+192+48(%rsp), $r_ptr
	call	__add_mod_384

	mov	8*3(%rsp), $a_ptr	# a->re
	lea	48($a_ptr), $b_org	# a->im
	lea	-48($r_ptr), $r_ptr
	call	__add_mod_384

	lea	($r_ptr),$b_ptr
	lea	48($r_ptr),$a_ptr
	call	__mulx_384

	################################# t2=t2-t0-t1
	lea	($r_ptr), $a_ptr	# t2
	lea	40(%rsp), $b_org	# t0
	mov	8*1(%rsp), $n_ptr
	call	__sub_mod_384x384	# t2-t0

	lea	($r_ptr), $a_ptr	# t2
	lea	-96($r_ptr), $b_org	# t1
	call	__sub_mod_384x384	# t2-t0-t1

	################################# t0=t0-t1
	lea	40(%rsp), $a_ptr
	lea	40+96(%rsp), $b_org
	lea	40(%rsp), $r_ptr
	call	__sub_mod_384x384	# t0-t1

	lea	($n_ptr), $b_ptr	# n_ptr for redc_mont_384

	################################# redc_mont_384(ret->re, t0, mod, n0);
	lea	40(%rsp), $a_ptr	# t0
	mov	8*0(%rsp), %rcx		# n0 for redc_mont_384
	mov	8*4(%rsp), $r_ptr	# ret->re
	call	__mulx_by_1_mont_384
	call	__redc_tail_mont_384

	################################# redc_mont_384(ret->im, t2, mod, n0);
	lea	40+192(%rsp), $a_ptr	# t2
	mov	8*0(%rsp), %rcx		# n0 for redc_mont_384
	lea	48($r_ptr), $r_ptr	# ret->im
	call	__mulx_by_1_mont_384
	call	__redc_tail_mont_384

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
.size	mulx_mont_384x,.-mulx_mont_384x
___
}
{ my $frame = 4*8 +	# place for argument off-load +
	      2*384/8 +	# place for 2 384-bit temporary vectors
	      8;	# alignment
$code.=<<___;
.globl	sqrx_mont_384x
.hidden	sqrx_mont_384x
.type	sqrx_mont_384x,\@function,4,"unwind"
.align	32
sqrx_mont_384x:
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

	mov	$n_ptr, 8*0(%rsp)	# n0
	mov	$b_org, $n_ptr		# n_ptr
					# gap for __mulx_mont_384
	mov	$r_ptr, 8*2(%rsp)
	mov	$a_ptr, 8*3(%rsp)

	################################# add_mod_384(t0, a->re, a->im);
	lea	48($a_ptr), $b_org	# a->im
	lea	32(%rsp), $r_ptr	# t0
	call	__add_mod_384

	################################# sub_mod_384(t1, a->re, a->im);
	mov	8*3(%rsp), $a_ptr	# a->re
	lea	48($a_ptr), $b_org	# a->im
	lea	32+48(%rsp), $r_ptr	# t1
	call	__sub_mod_384

	################################# mul_mont_384(ret->im, a->re, a->im, mod, n0);
	mov	8*3(%rsp), $a_ptr	# a->re
	lea	48($a_ptr), $b_ptr	# a->im

	mov	48($a_ptr), %rdx
	mov	8*0($a_ptr), %r14	# @acc[6]
	mov	8*1($a_ptr), %r15	# @acc[7]
	mov	8*2($a_ptr), %rax	# @acc[8]
	mov	8*3($a_ptr), %r12	# @acc[4]
	mov	8*4($a_ptr), %rdi	# $lo
	mov	8*5($a_ptr), %rbp	# $hi
	lea	-128($a_ptr), $a_ptr	# control u-op density
	lea	-128($n_ptr), $n_ptr	# control u-op density

	mulx	%r14, %r8, %r9
	call	__mulx_mont_384
___
{
my @acc = map("%r$_","dx",15,"ax",12,"di","bp",	# output from __mulx_mont_384
                      8..11,13,14);
$code.=<<___;
	add	@acc[0], @acc[0]	# add with itself
	adc	@acc[1], @acc[1]
	adc	@acc[2], @acc[2]
	 mov	@acc[0], @acc[6]
	adc	@acc[3], @acc[3]
	 mov	@acc[1], @acc[7]
	adc	@acc[4], @acc[4]
	 mov	@acc[2], @acc[8]
	adc	@acc[5], @acc[5]
	 mov	@acc[3], @acc[9]
	sbb	$a_ptr, $a_ptr

	sub	8*0($n_ptr), @acc[0]
	sbb	8*1($n_ptr), @acc[1]
	 mov	@acc[4], @acc[10]
	sbb	8*2($n_ptr), @acc[2]
	sbb	8*3($n_ptr), @acc[3]
	sbb	8*4($n_ptr), @acc[4]
	 mov	@acc[5], @acc[11]
	sbb	8*5($n_ptr), @acc[5]
	sbb	\$0, $a_ptr

	cmovc	@acc[6],  @acc[0]
	cmovc	@acc[7],  @acc[1]
	cmovc	@acc[8],  @acc[2]
	mov	@acc[0], 8*6($b_ptr)	# ret->im
	cmovc	@acc[9],  @acc[3]
	mov	@acc[1], 8*7($b_ptr)
	cmovc	@acc[10], @acc[4]
	mov	@acc[2], 8*8($b_ptr)
	cmovc	@acc[11], @acc[5]
	mov	@acc[3], 8*9($b_ptr)
	mov	@acc[4], 8*10($b_ptr)
	mov	@acc[5], 8*11($b_ptr)
___
}
$code.=<<___;
	################################# mul_mont_384(ret->re, t0, t1, mod, n0);
	lea	32(%rsp), $a_ptr	# t0
	lea	32+48(%rsp), $b_ptr	# t1

	mov	32+48(%rsp), %rdx	# t1[0]
	mov	32+8*0(%rsp), %r14	# @acc[6]
	mov	32+8*1(%rsp), %r15	# @acc[7]
	mov	32+8*2(%rsp), %rax	# @acc[8]
	mov	32+8*3(%rsp), %r12	# @acc[4]
	mov	32+8*4(%rsp), %rdi	# $lo
	mov	32+8*5(%rsp), %rbp	# $hi
	lea	-128($a_ptr), $a_ptr	# control u-op density
	lea	-128($n_ptr), $n_ptr	# control u-op density

	mulx	%r14, %r8, %r9
	call	__mulx_mont_384

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
.size	sqrx_mont_384x,.-sqrx_mont_384x

.globl	mulx_382x
.hidden	mulx_382x
.type	mulx_382x,\@function,4,"unwind"
.align	32
mulx_382x:
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

	lea	96($r_ptr), $r_ptr	# ret->im
	mov	$a_ptr, 8*0(%rsp)
	mov	$b_org, 8*1(%rsp)
	mov	$r_ptr, 8*2(%rsp)	# offload ret->im
	mov	$n_ptr, 8*3(%rsp)

	################################# t0 = a->re + a->im
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]

	add	8*6($a_ptr), @acc[0]
	adc	8*7($a_ptr), @acc[1]
	adc	8*8($a_ptr), @acc[2]
	adc	8*9($a_ptr), @acc[3]
	adc	8*10($a_ptr), @acc[4]
	adc	8*11($a_ptr), @acc[5]

	mov	@acc[0], 32+8*0(%rsp)
	mov	@acc[1], 32+8*1(%rsp)
	mov	@acc[2], 32+8*2(%rsp)
	mov	@acc[3], 32+8*3(%rsp)
	mov	@acc[4], 32+8*4(%rsp)
	mov	@acc[5], 32+8*5(%rsp)

	################################# t1 = b->re + b->im
	mov	8*0($b_org), @acc[0]
	mov	8*1($b_org), @acc[1]
	mov	8*2($b_org), @acc[2]
	mov	8*3($b_org), @acc[3]
	mov	8*4($b_org), @acc[4]
	mov	8*5($b_org), @acc[5]

	add	8*6($b_org), @acc[0]
	adc	8*7($b_org), @acc[1]
	adc	8*8($b_org), @acc[2]
	adc	8*9($b_org), @acc[3]
	adc	8*10($b_org), @acc[4]
	adc	8*11($b_org), @acc[5]

	mov	@acc[0], 32+8*6(%rsp)
	mov	@acc[1], 32+8*7(%rsp)
	mov	@acc[2], 32+8*8(%rsp)
	mov	@acc[3], 32+8*9(%rsp)
	mov	@acc[4], 32+8*10(%rsp)
	mov	@acc[5], 32+8*11(%rsp)

	################################# mul_384(ret->im, t0, t1);
	lea	32+8*0(%rsp), $a_ptr	# t0
	lea	32+8*6(%rsp), $b_ptr	# t1
	call	__mulx_384

	################################# mul_384(ret->re, a->re, b->re);
	mov	8*0(%rsp), $a_ptr
	mov	8*1(%rsp), $b_ptr
	lea	-96($r_ptr), $r_ptr	# ret->re
	call	__mulx_384

	################################# mul_384(tx, a->im, b->im);
	lea	48+128($a_ptr), $a_ptr
	lea	48($b_ptr), $b_ptr
	lea	32(%rsp), $r_ptr
	call	__mulx_384

	################################# ret->im -= tx
	mov	8*2(%rsp), $a_ptr	# restore ret->im
	lea	32(%rsp), $b_org
	mov	8*3(%rsp), $n_ptr
	mov	$a_ptr, $r_ptr
	call	__sub_mod_384x384

	################################# ret->im -= ret->re
	lea	0($r_ptr), $a_ptr
	lea	-96($r_ptr), $b_org
	call	__sub_mod_384x384

	################################# ret->re -= tx
	lea	-96($r_ptr), $a_ptr
	lea	32(%rsp), $b_org
	lea	-96($r_ptr), $r_ptr
	call	__sub_mod_384x384

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
.size	mulx_382x,.-mulx_382x
___
}
{ my @acc=(@acc,"%rax","%rbx","%rbp",$b_org);	# all registers are affected
						# except for $n_ptr and $r_ptr
$code.=<<___;
.globl	sqrx_382x
.hidden	sqrx_382x
.type	sqrx_382x,\@function,3,"unwind"
.align	32
sqrx_382x:
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

	mov	$b_org, $n_ptr

	################################# t0 = a->re + a->im
	mov	8*0($a_ptr), @acc[6]
	mov	8*1($a_ptr), @acc[7]
	mov	8*2($a_ptr), @acc[8]
	mov	8*3($a_ptr), @acc[9]
	mov	8*4($a_ptr), @acc[10]
	mov	8*5($a_ptr), @acc[11]

	mov	@acc[6], @acc[0]
	add	8*6($a_ptr), @acc[6]
	mov	@acc[7], @acc[1]
	adc	8*7($a_ptr), @acc[7]
	mov	@acc[8], @acc[2]
	adc	8*8($a_ptr), @acc[8]
	mov	@acc[9], @acc[3]
	adc	8*9($a_ptr), @acc[9]
	mov	@acc[10], @acc[4]
	adc	8*10($a_ptr), @acc[10]
	mov	@acc[11], @acc[5]
	adc	8*11($a_ptr), @acc[11]

	mov	@acc[6], 8*0($r_ptr)
	mov	@acc[7], 8*1($r_ptr)
	mov	@acc[8], 8*2($r_ptr)
	mov	@acc[9], 8*3($r_ptr)
	mov	@acc[10], 8*4($r_ptr)
	mov	@acc[11], 8*5($r_ptr)

	################################# t1 = a->re - a->im
	lea	48($a_ptr), $b_org
	lea	48($r_ptr), $r_ptr
	call	__sub_mod_384_a_is_loaded

	################################# mul_384(ret->re, t0, t1);
	lea	($r_ptr), $a_ptr
	lea	-48($r_ptr), $b_ptr
	lea	-48($r_ptr), $r_ptr
	call	__mulx_384

	################################# mul_384(ret->im, a->re, a->im);
	mov	(%rsp), $a_ptr
	lea	48($a_ptr), $b_ptr
	lea	96($r_ptr), $r_ptr
	call	__mulx_384

	mov	8*0($r_ptr), @acc[0]	# double ret->im
	mov	8*1($r_ptr), @acc[1]
	mov	8*2($r_ptr), @acc[2]
	mov	8*3($r_ptr), @acc[3]
	mov	8*4($r_ptr), @acc[4]
	mov	8*5($r_ptr), @acc[5]
	mov	8*6($r_ptr), @acc[6]
	mov	8*7($r_ptr), @acc[7]
	mov	8*8($r_ptr), @acc[8]
	mov	8*9($r_ptr), @acc[9]
	mov	8*10($r_ptr), @acc[10]
	add	@acc[0], @acc[0]
	mov	8*11($r_ptr), @acc[11]
	adc	@acc[1], @acc[1]
	mov	@acc[0], 8*0($r_ptr)
	adc	@acc[2], @acc[2]
	mov	@acc[1], 8*1($r_ptr)
	adc	@acc[3], @acc[3]
	mov	@acc[2], 8*2($r_ptr)
	adc	@acc[4], @acc[4]
	mov	@acc[3], 8*3($r_ptr)
	adc	@acc[5], @acc[5]
	mov	@acc[4], 8*4($r_ptr)
	adc	@acc[6], @acc[6]
	mov	@acc[5], 8*5($r_ptr)
	adc	@acc[7], @acc[7]
	mov	@acc[6], 8*6($r_ptr)
	adc	@acc[8], @acc[8]
	mov	@acc[7], 8*7($r_ptr)
	adc	@acc[9], @acc[9]
	mov	@acc[8], 8*8($r_ptr)
	adc	@acc[10], @acc[10]
	mov	@acc[9], 8*9($r_ptr)
	adc	@acc[11], @acc[11]
	mov	@acc[10], 8*10($r_ptr)
	mov	@acc[11], 8*11($r_ptr)

	mov	8*1(%rsp),%r15
.cfi_restore	%r15
	mov	8*2(%rsp),%r14
.cfi_restore	%r14
	mov	8*3(%rsp),%r13
.cfi_restore	%r13
	mov	8*4(%rsp),%r12
.cfi_restore	%r12
	mov	8*5(%rsp),%rbx
.cfi_restore	%rbx
	mov	8*6(%rsp),%rbp
.cfi_restore	%rbp
	lea	8*7(%rsp),%rsp
.cfi_adjust_cfa_offset	-8*7
.cfi_epilogue
	ret
.cfi_endproc
.size	sqrx_382x,.-sqrx_382x
___
}
{ ########################################################## 384-bit mulx
my ($a0, $a1) = @acc[6..7];
my @acc = @acc[0..5];
my ($lo, $hi, $zr) = ("%rax", "%rcx", "%rbp");

$code.=<<___;
.globl	mulx_384
.hidden	mulx_384
.type	mulx_384,\@function,3,"unwind"
.align	32
mulx_384:
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
.cfi_end_prologue

	mov	$b_org, $b_ptr		# evacuate from %rdx
	call	__mulx_384

	mov	0(%rsp),%r15
.cfi_restore	%r15
	mov	8(%rsp),%r14
.cfi_restore	%r14
	mov	16(%rsp),%r13
.cfi_restore	%r13
	mov	24(%rsp),%r12
.cfi_restore	%r12
	mov	32(%rsp),%rbx
.cfi_restore	%rbx
	mov	40(%rsp),%rbp
.cfi_restore	%rbp
	lea	48(%rsp),%rsp
.cfi_adjust_cfa_offset	-48
.cfi_epilogue
	ret
.cfi_endproc
.size	mulx_384,.-mulx_384

.type	__mulx_384,\@abi-omnipotent
.align	32
__mulx_384:
	mov	8*0($b_ptr), %rdx
	mov	8*0($a_ptr), $a0
	mov	8*1($a_ptr), $a1
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]
	lea	-128($a_ptr), $a_ptr

	mulx	$a0, @acc[1], $hi
	xor	$zr, $zr

	mulx	$a1, @acc[0], $lo
	adcx	$hi, @acc[0]
	mov	@acc[1], 8*0($r_ptr)

	mulx	@acc[2], @acc[1], $hi
	adcx	$lo, @acc[1]

	mulx	@acc[3], @acc[2], $lo
	adcx	$hi, @acc[2]

	mulx	@acc[4], @acc[3], $hi
	adcx	$lo, @acc[3]

	mulx	@acc[5], @acc[4], @acc[5]
	mov	8*1($b_ptr), %rdx
	adcx	$hi, @acc[4]
	adcx	$zr, @acc[5]
___
for(my $i=1; $i<6; $i++) {
my $b_next = $i<5 ? 8*($i+1)."($b_ptr)" : "%rax";
$code.=<<___;
	mulx	$a0, $lo, $hi
	adcx	@acc[0], $lo
	adox	$hi, @acc[1]
	mov	$lo, 8*$i($r_ptr)

	mulx	$a1, @acc[0], $hi
	adcx	@acc[1], $acc[0]
	adox	$hi, @acc[2]

	mulx	128+8*2($a_ptr), @acc[1], $lo
	adcx	@acc[2], @acc[1]
	adox	$lo, @acc[3]

	mulx	128+8*3($a_ptr), @acc[2], $hi
	adcx	@acc[3], @acc[2]
	adox	$hi, @acc[4]

	mulx	128+8*4($a_ptr), @acc[3], $lo
	adcx	@acc[4], @acc[3]
	adox	@acc[5], $lo

	mulx	128+8*5($a_ptr), @acc[4], @acc[5]
	mov	$b_next, %rdx
	adcx	$lo, @acc[4]
	adox	$zr, @acc[5]
	adcx	$zr, @acc[5]
___
}
$code.=<<___;
	mov	@acc[0], 8*6($r_ptr)
	mov	@acc[1], 8*7($r_ptr)
	mov	@acc[2], 8*8($r_ptr)
	mov	@acc[3], 8*9($r_ptr)
	mov	@acc[4], 8*10($r_ptr)
	mov	@acc[5], 8*11($r_ptr)

	ret
.size	__mulx_384,.-__mulx_384
___
}
{ ########################################################## 384-bit sqrx
$code.=<<___;
.globl	sqrx_384
.hidden	sqrx_384
.type	sqrx_384,\@function,2,"unwind"
.align	32
sqrx_384:
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

	call	__sqrx_384

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
.size	sqrx_384,.-sqrx_384
___
if (0) {
# up to 5% slower than below variant
my @acc=map("%r$_",("no",8..15,"cx","bx"));
   push(@acc, $a_ptr);
my ($lo, $hi, $carry)=("%rax", "%rbp", "%rno");

$code.=<<___;
.type	__sqrx_384,\@abi-omnipotent
.align	32
__sqrx_384:
	mov	8*0($a_ptr), %rdx
	mov	8*1($a_ptr), @acc[7]
	mov	8*2($a_ptr), @acc[8]
	mov	8*3($a_ptr), @acc[9]
	mov	8*4($a_ptr), @acc[10]

	#########################################
	mulx	@acc[7], @acc[1], $lo		# a[1]*a[0]
	 mov	8*5($a_ptr), @acc[11]
	mulx	@acc[8], @acc[2], $hi		# a[2]*a[0]
	add	$lo, @acc[2]
	mulx	@acc[9], @acc[3], $lo		# a[3]*a[0]
	adc	$hi, @acc[3]
	mulx	@acc[10], @acc[4], $hi		# a[4]*a[0]
	adc	$lo, @acc[4]
	mulx	@acc[11], @acc[5], @acc[6]	# a[5]*a[0]
	adc	$hi, @acc[5]
	adc	\$0, @acc[6]

	mulx	%rdx, $lo, $hi			# a[0]*a[0]
	 mov	@acc[7], %rdx
	xor	@acc[7], @acc[7]
	add	@acc[1], @acc[1]		# double acc[1]
	adc	\$0, @acc[7]
	add	$hi, @acc[1]
	adc	\$0, @acc[7]
	mov	$lo, 8*0($r_ptr)
	mov	@acc[1], 8*1($r_ptr)
___
($carry, @acc[7]) = (@acc[7], @acc[1]);
$code.=<<___;
	#########################################
	xor	@acc[7], @acc[7]
	mulx	@acc[8], $lo, $hi		# a[2]*a[1]
	adcx	$lo, @acc[3]
	adox	$hi, @acc[4]

	mulx	@acc[9], $lo, $hi		# a[3]*a[1]
	adcx	$lo, @acc[4]
	adox	$hi, @acc[5]

	mulx	@acc[10], $lo, $hi		# a[4]*a[1]
	adcx	$lo, @acc[5]
	adox	$hi, @acc[6]

	mulx	@acc[11], $lo, $hi		# a[5]*a[1]
	adcx	$lo, @acc[6]
	adox	@acc[7], $hi
	adcx	$hi, @acc[7]

	mulx	%rdx, $lo, $hi			# a[1]*a[1]
	 mov	@acc[8], %rdx
	xor	@acc[8], @acc[8]
	adox	@acc[2], @acc[2]		# double acc[2:3]
	adcx	$carry, $lo			# can't carry
	adox	@acc[3], @acc[3]
	adcx	$lo, @acc[2]
	adox	@acc[8], @acc[8]
	adcx	$hi, @acc[3]
	adc	\$0, @acc[8]
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)
___
($carry,@acc[8])=(@acc[8],$carry);
$code.=<<___;
	#########################################
	xor	@acc[8], @acc[8]
	mulx	@acc[9], $lo, $hi		# a[3]*a[2]
	adcx	$lo, @acc[5]
	adox	$hi, @acc[6]

	mulx	@acc[10], $lo, $hi		# a[4]*a[2]
	adcx	$lo, @acc[6]
	adox	$hi, @acc[7]

	mulx	@acc[11], $lo, $hi		# a[5]*a[2]
	adcx	$lo, @acc[7]
	adox	@acc[8], $hi
	adcx	$hi, @acc[8]

	mulx	%rdx, $lo, $hi			# a[2]*a[2]
	 mov	@acc[9], %rdx
	xor	@acc[9], @acc[9]
	adox	@acc[4], @acc[4]		# double acc[4:5]
	adcx	$carry, $lo			# can't carry
	adox	@acc[5], @acc[5]
	adcx	$lo, @acc[4]
	adox	@acc[9], @acc[9]
	adcx	$hi, @acc[5]
	adc	\$0, $acc[9]
	mov	@acc[4], 8*4($r_ptr)
	mov	@acc[5], 8*5($r_ptr)
___
($carry,@acc[9])=(@acc[9],$carry);
$code.=<<___;
	#########################################
	xor	@acc[9], @acc[9]
	mulx	@acc[10], $lo, $hi		# a[4]*a[3]
	adcx	$lo, @acc[7]
	adox	$hi, @acc[8]

	mulx	@acc[11], $lo, $hi		# a[5]*a[3]
	adcx	$lo, @acc[8]
	adox	@acc[9], $hi
	adcx	$hi, @acc[9]

	mulx	%rdx, $lo, $hi
	 mov	@acc[10], %rdx
	xor	@acc[10], @acc[10]
	adox	@acc[6], @acc[6]		# double acc[6:7]
	adcx	$carry, $lo			# can't carry
	adox	@acc[7], @acc[7]
	adcx	$lo, @acc[6]
	adox	@acc[10], @acc[10]
	adcx	$hi, @acc[7]
	adc	\$0, $acc[10]
	mov	@acc[6], 8*6($r_ptr)
	mov	@acc[7], 8*7($r_ptr)
___
($carry,@acc[10])=(@acc[10],$carry);
$code.=<<___;
	#########################################
	mulx	@acc[11], $lo, @acc[10]		# a[5]*a[4]
	add	$lo, @acc[9]
	adc	\$0, @acc[10]

	mulx	%rdx, $lo, $hi			# a[4]*a[4]
	 mov	@acc[11], %rdx
	xor	@acc[11], @acc[11]
	adox	@acc[8], @acc[8]		# double acc[8:10]
	adcx	$carry, $lo			# can't carry
	adox	@acc[9], @acc[9]
	adcx	$lo, @acc[8]
	adox	@acc[10], @acc[10]
	adcx	$hi, @acc[9]
	adox	@acc[11], @acc[11]
	mov	@acc[8], 8*8($r_ptr)
	mov	@acc[9], 8*9($r_ptr)

	#########################################
	mulx	%rdx, $lo, $hi			# a[5]*a[5]
	adcx	$lo, @acc[10]
	adcx	$hi, @acc[11]

	mov	@acc[10], 8*10($r_ptr)
	mov	@acc[11], 8*11($r_ptr)

	ret
.size	__sqrx_384,.-__sqrx_384
___
} else {
my @acc=map("%r$_",("no",8..15,"cx","bx","bp"));
my ($lo, $hi)=($r_ptr, "%rax");

$code.=<<___;
.type	__sqrx_384,\@abi-omnipotent
.align	32
__sqrx_384:
	mov	8*0($a_ptr), %rdx
	mov	8*1($a_ptr), @acc[7]
	mov	8*2($a_ptr), @acc[8]
	mov	8*3($a_ptr), @acc[9]
	mov	8*4($a_ptr), @acc[10]

	#########################################
	mulx	@acc[7], @acc[1], $lo		# a[1]*a[0]
	 mov	8*5($a_ptr), @acc[11]
	mulx	@acc[8], @acc[2], $hi		# a[2]*a[0]
	add	$lo, @acc[2]
	mulx	@acc[9], @acc[3], $lo		# a[3]*a[0]
	adc	$hi, @acc[3]
	mulx	@acc[10], @acc[4], $hi		# a[4]*a[0]
	adc	$lo, @acc[4]
	mulx	@acc[11], @acc[5], @acc[6]	# a[5]*a[0]
	 mov	@acc[7], %rdx
	adc	$hi, @acc[5]
	adc	\$0, @acc[6]

	#########################################
	xor	@acc[7], @acc[7]
	mulx	@acc[8], $lo, $hi		# a[2]*a[1]
	adcx	$lo, @acc[3]
	adox	$hi, @acc[4]

	mulx	@acc[9], $lo, $hi		# a[3]*a[1]
	adcx	$lo, @acc[4]
	adox	$hi, @acc[5]

	mulx	@acc[10], $lo, $hi		# a[4]*a[1]
	adcx	$lo, @acc[5]
	adox	$hi, @acc[6]

	mulx	@acc[11], $lo, $hi		# a[5]*a[1]
	 mov	@acc[8], %rdx
	adcx	$lo, @acc[6]
	adox	@acc[7], $hi
	adcx	$hi, @acc[7]

	#########################################
	xor	@acc[8], @acc[8]
	mulx	@acc[9], $lo, $hi		# a[3]*a[2]
	adcx	$lo, @acc[5]
	adox	$hi, @acc[6]

	mulx	@acc[10], $lo, $hi		# a[4]*a[2]
	adcx	$lo, @acc[6]
	adox	$hi, @acc[7]

	mulx	@acc[11], $lo, $hi		# a[5]*a[2]
	 mov	@acc[9], %rdx
	adcx	$lo, @acc[7]
	adox	@acc[8], $hi
	adcx	$hi, @acc[8]

	#########################################
	xor	@acc[9], @acc[9]
	mulx	@acc[10], $lo, $hi		# a[4]*a[3]
	adcx	$lo, @acc[7]
	adox	$hi, @acc[8]

	mulx	@acc[11], $lo, $hi		# a[5]*a[3]
	 mov	@acc[10], %rdx
	adcx	$lo, @acc[8]
	adox	@acc[9], $hi
	adcx	$hi, @acc[9]

	#########################################
	mulx	@acc[11], $lo, @acc[10]		# a[5]*a[4]
	 mov	8*0($a_ptr), %rdx
	add	$lo, @acc[9]
	 mov	8(%rsp), $r_ptr			# restore $r_ptr
	adc	\$0, @acc[10]

	######################################### double acc[1:10]
	xor	@acc[11], @acc[11]
	adcx	@acc[1], @acc[1]
	adcx	@acc[2], @acc[2]
	adcx	@acc[3], @acc[3]
	adcx	@acc[4], @acc[4]
	adcx	@acc[5], @acc[5]

	######################################### accumulate a[i]*a[i]
	mulx	%rdx, %rdx, $hi 		# a[0]*a[0]
	mov	%rdx, 8*0($r_ptr)
	mov	8*1($a_ptr), %rdx
	adox	$hi, @acc[1]
	mov	@acc[1], 8*1($r_ptr)

	mulx	%rdx, @acc[1], $hi		# a[1]*a[1]
	mov	8*2($a_ptr), %rdx
	adox	@acc[1], @acc[2]
	adox	$hi,     @acc[3]
	mov	@acc[2], 8*2($r_ptr)
	mov	@acc[3], 8*3($r_ptr)

	mulx	%rdx, @acc[1], @acc[2]		# a[2]*a[2]
	mov	8*3($a_ptr), %rdx
	adox	@acc[1], @acc[4]
	adox	@acc[2], @acc[5]
	adcx	@acc[6], @acc[6]
	adcx	@acc[7], @acc[7]
	mov	@acc[4], 8*4($r_ptr)
	mov	@acc[5], 8*5($r_ptr)

	mulx	%rdx, @acc[1], @acc[2]		# a[3]*a[3]
	mov	8*4($a_ptr), %rdx
	adox	@acc[1], @acc[6]
	adox	@acc[2], @acc[7]
	adcx	@acc[8], @acc[8]
	adcx	@acc[9], @acc[9]
	mov	@acc[6], 8*6($r_ptr)
	mov	@acc[7], 8*7($r_ptr)

	mulx	%rdx, @acc[1], @acc[2]		# a[4]*a[4]
	mov	8*5($a_ptr), %rdx
	adox	@acc[1], @acc[8]
	adox	@acc[2], @acc[9]
	adcx	@acc[10], @acc[10]
	adcx	@acc[11], @acc[11]
	mov	@acc[8], 8*8($r_ptr)
	mov	@acc[9], 8*9($r_ptr)

	mulx	%rdx, @acc[1], @acc[2]		# a[5]*a[5]
	adox	@acc[1], @acc[10]
	adox	@acc[2], @acc[11]

	mov	@acc[10], 8*10($r_ptr)
	mov	@acc[11], 8*11($r_ptr)

	ret
.size	__sqrx_384,.-__sqrx_384
___
}

{ ########################################################## 384-bit redcx_mont
my ($n_ptr, $n0)=($b_ptr, $n_ptr);      # arguments are "shifted"
my ($lo, $hi) = ("%rax", "%rbp");

$code.=<<___;
########################################################################
# void redcx_mont_384(uint64_t ret[6], const uint64_t a[12],
#                     uint64_t m[6], uint64_t n0);
.globl	redcx_mont_384
.hidden	redcx_mont_384
.type	redcx_mont_384,\@function,4,"unwind"
.align	32
redcx_mont_384:
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
	call	__mulx_by_1_mont_384
	call	__redc_tail_mont_384

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
.size	redcx_mont_384,.-redcx_mont_384

########################################################################
# void fromx_mont_384(uint64_t ret[6], const uint64_t a[6],
#                    uint64_t m[6], uint64_t n0);
.globl	fromx_mont_384
.hidden	fromx_mont_384
.type	fromx_mont_384,\@function,4,"unwind"
.align	32
fromx_mont_384:
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
	call	__mulx_by_1_mont_384

	#################################
	# Branch-less conditional acc[0:6] - modulus

	mov	@acc[6], %rax
	mov	@acc[7], %rcx
	mov	@acc[0], %rdx
	mov	@acc[1], %rbp

	sub	8*0($n_ptr), @acc[6]
	sbb	8*1($n_ptr), @acc[7]
	mov	@acc[2], @acc[5]
	sbb	8*2($n_ptr), @acc[0]
	sbb	8*3($n_ptr), @acc[1]
	sbb	8*4($n_ptr), @acc[2]
	mov	@acc[3], $a_ptr
	sbb	8*5($n_ptr), @acc[3]

	cmovc	%rax, @acc[6]
	cmovc	%rcx, @acc[7]
	cmovc	%rdx, @acc[0]
	mov	@acc[6], 8*0($r_ptr)
	cmovc	%rbp, @acc[1]
	mov	@acc[7], 8*1($r_ptr)
	cmovc	@acc[5], @acc[2]
	mov	@acc[0], 8*2($r_ptr)
	cmovc	$a_ptr,  @acc[3]
	mov	@acc[1], 8*3($r_ptr)
	mov	@acc[2], 8*4($r_ptr)
	mov	@acc[3], 8*5($r_ptr)

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
.size	fromx_mont_384,.-fromx_mont_384
___
{ my @acc=@acc;				# will be rotated locally

$code.=<<___;
.type	__mulx_by_1_mont_384,\@abi-omnipotent
.align	32
__mulx_by_1_mont_384:
	mov	8*0($a_ptr), @acc[0]
	mov	$n0, %rdx
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]
___
for (my $i=0; $i<6; $i++) {
$code.=<<___;
	imulq	@acc[0], %rdx

	################################# reduction $i
	xor	@acc[6], @acc[6]	# @acc[6]=0, cf=0, of=0
	mulx	8*0($n_ptr), $lo, $hi
	adcx	$lo, @acc[0]		# guaranteed to be zero
	adox	$hi, @acc[1]

	mulx	8*1($n_ptr), $lo, $hi
	adcx	$lo, @acc[1]
	adox	$hi, @acc[2]

	mulx	8*2($n_ptr), $lo, $hi
	adcx	$lo, @acc[2]
	adox	$hi, @acc[3]

	mulx	8*3($n_ptr), $lo, $hi
	adcx	$lo, @acc[3]
	adox	$hi, @acc[4]

	mulx	8*4($n_ptr), $lo, $hi
	adcx	$lo, @acc[4]
	adox	$hi, @acc[5]

	mulx	8*5($n_ptr), $lo, $hi
	 mov	$n0, %rdx
	adcx	$lo, @acc[5]
	adox	@acc[6], $hi
	adcx	$hi, @acc[6]
___
    push(@acc,shift(@acc));
}
$code.=<<___;
	ret
.size	__mulx_by_1_mont_384,.-__mulx_by_1_mont_384

.type	__redc_tail_mont_384,\@abi-omnipotent
.align	32
__redc_tail_mont_384:
	add	8*6($a_ptr), @acc[0]	# accumulate upper half
	mov	@acc[0], %rax
	adc	8*7($a_ptr), @acc[1]
	adc	8*8($a_ptr), @acc[2]
	adc	8*9($a_ptr), @acc[3]
	mov	@acc[1], %rcx
	adc	8*10($a_ptr), @acc[4]
	adc	8*11($a_ptr), @acc[5]
	sbb	@acc[6], @acc[6]

	#################################
	# Branch-less conditional acc[0:6] - modulus

	mov	@acc[2], %rdx
	mov	@acc[3], %rbp

	sub	8*0($n_ptr), @acc[0]
	sbb	8*1($n_ptr), @acc[1]
	mov	@acc[4], @acc[7]
	sbb	8*2($n_ptr), @acc[2]
	sbb	8*3($n_ptr), @acc[3]
	sbb	8*4($n_ptr), @acc[4]
	mov	@acc[5], $a_ptr
	sbb	8*5($n_ptr), @acc[5]
	sbb	\$0, @acc[6]

	cmovc	%rax, @acc[0]
	cmovc	%rcx, @acc[1]
	cmovc	%rdx, @acc[2]
	mov	@acc[0], 8*0($r_ptr)
	cmovc	%rbp, @acc[3]
	mov	@acc[1], 8*1($r_ptr)
	cmovc	@acc[7], @acc[4]
	mov	@acc[2], 8*2($r_ptr)
	cmovc	$a_ptr,  @acc[5]
	mov	@acc[3], 8*3($r_ptr)
	mov	@acc[4], 8*4($r_ptr)
	mov	@acc[5], 8*5($r_ptr)

	ret
.size	__redc_tail_mont_384,.-__redc_tail_mont_384

.globl	sgn0x_pty_mont_384
.hidden	sgn0x_pty_mont_384
.type	sgn0x_pty_mont_384,\@function,3,"unwind"
.align	32
sgn0x_pty_mont_384:
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

	mov	$a_ptr, $n_ptr
	lea	0($r_ptr), $a_ptr
	mov	$b_org, $n0
	call	__mulx_by_1_mont_384

	xor	%rax, %rax
	mov	@acc[0], @acc[7]
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
	and	\$1, @acc[7]
	and	\$2, %rax
	or	@acc[7], %rax		# pack sign and parity

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
.size	sgn0x_pty_mont_384,.-sgn0x_pty_mont_384

.globl	sgn0x_pty_mont_384x
.hidden	sgn0x_pty_mont_384x
.type	sgn0x_pty_mont_384x,\@function,3,"unwind"
.align	32
sgn0x_pty_mont_384x:
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

	mov	$a_ptr, $n_ptr
	lea	48($r_ptr), $a_ptr	# sgn0(a->im)
	mov	$b_org, $n0
	call	__mulx_by_1_mont_384

	mov	@acc[0], @acc[6]
	or	@acc[1], @acc[0]
	or	@acc[2], @acc[0]
	or	@acc[3], @acc[0]
	or	@acc[4], @acc[0]
	or	@acc[5], @acc[0]

	lea	0($r_ptr), $a_ptr	# sgn0(a->re)
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

	call	__mulx_by_1_mont_384

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
.size	sgn0x_pty_mont_384x,.-sgn0x_pty_mont_384x
___
} }

{ ########################################################## mulx/sqrx_mont
my @acc = (@acc, "%rax");
my ($lo,$hi)=("%rdi","%rbp");

$code.=<<___;
.globl	mulx_mont_384
.hidden	mulx_mont_384
.type	mulx_mont_384,\@function,5,"unwind"
.align	32
mulx_mont_384:
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
	lea	-8*3(%rsp), %rsp
.cfi_adjust_cfa_offset	8*3
.cfi_end_prologue

	mov	$b_org, $b_ptr		# evacuate from %rdx
	mov	8*0($b_org), %rdx
	mov	8*0($a_ptr), @acc[6]
	mov	8*1($a_ptr), @acc[7]
	mov	8*2($a_ptr), @acc[8]
	mov	8*3($a_ptr), @acc[4]
	mov	$r_ptr, 8*2(%rsp)
	mov	8*4($a_ptr), $lo
	mov	8*5($a_ptr), $hi
	lea	-128($a_ptr), $a_ptr	# control u-op density
	lea	-128($n_ptr), $n_ptr	# control u-op density
	mov	$n0, (%rsp)

	mulx	@acc[6],@acc[0],@acc[1]	# a[0]*b[0]
	call	__mulx_mont_384

	mov	8*3(%rsp),%r15
.cfi_restore	%r15
	mov	8*4(%rsp),%r14
.cfi_restore	%r14
	mov	8*5(%rsp),%r13
.cfi_restore	%r13
	mov	8*6(%rsp),%r12
.cfi_restore	%r12
	mov	8*7(%rsp),%rbx
.cfi_restore	%rbx
	mov	8*8(%rsp),%rbp
.cfi_restore	%rbp
	lea	8*9(%rsp),%rsp
.cfi_adjust_cfa_offset	-8*9
.cfi_epilogue
	ret
.cfi_endproc
.size	mulx_mont_384,.-mulx_mont_384
___
{ my @acc=@acc;				# will be rotated locally

$code.=<<___;
.type	__mulx_mont_384,\@abi-omnipotent
.align	32
__mulx_mont_384:
.cfi_startproc
	mulx	@acc[7], @acc[6], @acc[2]
	mulx	@acc[8], @acc[7], @acc[3]
	add	@acc[6], @acc[1]
	mulx	@acc[4], @acc[8], @acc[4]
	adc	@acc[7], @acc[2]
	mulx	$lo, $lo, @acc[5]
	adc	@acc[8], @acc[3]
	mulx	$hi, $hi, @acc[6]
	 mov	8($b_ptr), %rdx
	adc	$lo, @acc[4]
	adc	$hi, @acc[5]
	adc	\$0, @acc[6]
	xor	@acc[7], @acc[7]

___
for (my $i=1; $i<6; $i++) {
my $tt = $i==1 ? @acc[7] : $hi;
my $b_next = $i<5 ? 8*($i+1)."($b_ptr)" : @acc[1];
$code.=<<___;
	 mov	@acc[0], 16(%rsp)
	 imulq	8(%rsp), @acc[0]

	################################# Multiply by b[$i]
	xor	@acc[8], @acc[8]	# @acc[8]=0, cf=0, of=0
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
	adox	$lo, @acc[4]
	adcx	$hi, @acc[5]

	mulx	8*4+128($a_ptr), $lo, $hi
	adox	$lo, @acc[5]
	adcx	$hi, @acc[6]

	mulx	8*5+128($a_ptr), $lo, $hi
	 mov	@acc[0], %rdx
	adox	$lo, @acc[6]
	adcx	$hi, @acc[7]		# cf=0
	adox	@acc[8], @acc[7]
	adox	@acc[8], @acc[8]

	################################# reduction
	xor	@acc[0], @acc[0]	# acc[0]=0, cf=0, of=0
	mulx	8*0+128($n_ptr), $lo, $hi
	adcx	16(%rsp), $lo		# guaranteed to be zero
	adox	$hi, @acc[1]

	mulx	8*1+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[1]
	adox	$hi, @acc[2]

	mulx	8*2+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[2]
	adox	$hi, @acc[3]

	mulx	8*3+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[3]
	adox	$hi, @acc[4]

	mulx	8*4+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[4]
	adox	$hi, @acc[5]

	mulx	8*5+128($n_ptr), $lo, $hi
	 mov	$b_next, %rdx
	adcx	$lo, @acc[5]
	adox	$hi, @acc[6]
	adcx	@acc[0], @acc[6]
	adox	@acc[0], @acc[7]
	adcx	@acc[0], @acc[7]
	adox	@acc[0], @acc[8]
	adcx	@acc[0], @acc[8]
___
    push(@acc,shift(@acc));
}
$code.=<<___;
	imulq	8(%rsp), %rdx
	mov	8*3(%rsp), $b_ptr	# restore $r_ptr

	################################# last reduction
	xor	@acc[8], @acc[8]	# @acc[8]=0, cf=0, of=0
	mulx	8*0+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[0]		# guaranteed to be zero
	adox	$hi, @acc[1]

	mulx	8*1+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[1]
	adox	$hi, @acc[2]

	mulx	8*2+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[2]
	adox	$hi, @acc[3]

	mulx	8*3+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[3]
	adox	$hi, @acc[4]
	 mov	@acc[2], @acc[0]

	mulx	8*4+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[4]
	adox	$hi, @acc[5]
	 mov	@acc[3], $a_ptr

	mulx	8*5+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[5]
	adox	$hi, @acc[6]
	 mov	@acc[1], %rdx
	adcx	@acc[8], @acc[6]
	adox	@acc[8], @acc[7]
	 lea	128($n_ptr), $n_ptr
	 mov	@acc[4], @acc[8]
	adc	\$0, @acc[7]

	#################################
	# Branch-less conditional acc[1:7] - modulus

	sub	8*0($n_ptr), @acc[1]
	sbb	8*1($n_ptr), @acc[2]
	 mov	@acc[5], $lo
	sbb	8*2($n_ptr), @acc[3]
	sbb	8*3($n_ptr), @acc[4]
	sbb	8*4($n_ptr), @acc[5]
	 mov	@acc[6], $hi
	sbb	8*5($n_ptr), @acc[6]
	sbb	\$0, @acc[7]

	cmovnc	@acc[1], %rdx
	cmovc	@acc[0], @acc[2]
	cmovc	$a_ptr, @acc[3]
	cmovnc	@acc[4], @acc[8]
	mov	%rdx, 8*0($b_ptr)
	cmovnc	@acc[5], $lo
	mov	@acc[2], 8*1($b_ptr)
	cmovnc	@acc[6], $hi
	mov	@acc[3], 8*2($b_ptr)
	mov	@acc[8], 8*3($b_ptr)
	mov	$lo, 8*4($b_ptr)
	mov	$hi, 8*5($b_ptr)

	ret
.cfi_endproc
.size	__mulx_mont_384,.-__mulx_mont_384
___
}
$code.=<<___;
.globl	sqrx_mont_384
.hidden	sqrx_mont_384
.type	sqrx_mont_384,\@function,4,"unwind"
.align	32
sqrx_mont_384:
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
	lea	-8*3(%rsp), %rsp
.cfi_adjust_cfa_offset	8*3
.cfi_end_prologue

	mov	$n_ptr, $n0		# n0
	lea	-128($b_org), $n_ptr	# control u-op density
	mov	8*0($a_ptr), %rdx
	mov	8*1($a_ptr), @acc[7]
	mov	8*2($a_ptr), @acc[8]
	mov	8*3($a_ptr), @acc[4]
	mov	$r_ptr, 8*2(%rsp)
	mov	8*4($a_ptr), $lo
	mov	8*5($a_ptr), $hi

	lea	($a_ptr), $b_ptr
	mov	$n0, (%rsp)		# n0
	lea	-128($a_ptr), $a_ptr	# control u-op density

	mulx	%rdx, @acc[0], @acc[1]	# a[0]*a[0]
	call	__mulx_mont_384		# as fast as dedicated squaring

	mov	8*3(%rsp),%r15
.cfi_restore	%r15
	mov	8*4(%rsp),%r14
.cfi_restore	%r14
	mov	8*5(%rsp),%r13
.cfi_restore	%r13
	mov	8*6(%rsp),%r12
.cfi_restore	%r12
	mov	8*7(%rsp),%rbx
.cfi_restore	%rbx
	mov	8*8(%rsp),%rbp
.cfi_restore	%rbp
	lea	8*9(%rsp),%rsp
.cfi_adjust_cfa_offset	-8*9
.cfi_epilogue
	ret
.cfi_endproc
.size	sqrx_mont_384,.-sqrx_mont_384

.globl	sqrx_n_mul_mont_384
.hidden	sqrx_n_mul_mont_384
.type	sqrx_n_mul_mont_384,\@function,6,"unwind"
.align	32
sqrx_n_mul_mont_384:
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
	lea	-8*5(%rsp), %rsp
.cfi_adjust_cfa_offset	8*5
.cfi_end_prologue

	mov	$b_org, @acc[2]		# loop counter
	mov	8*0($a_ptr), %rdx
	mov	8*1($a_ptr), @acc[7]
	mov	8*2($a_ptr), @acc[8]
	mov	$a_ptr, $b_ptr
	mov	8*3($a_ptr), @acc[4]
	mov	$r_ptr, 8*2(%rsp)	# to __mulx_mont_384
	mov	8*4($a_ptr), $lo
	mov	8*5($a_ptr), $hi

	mov	$n0, (%rsp)
	mov	%r9, 8*3(%rsp)		# 6th, multiplicand argument
	movq	8*0(%r9), %xmm2		# prefetch b[0]

.Loop_sqrx_384:
	movd	@acc[2]d, %xmm1
	lea	-128($b_ptr), $a_ptr	# control u-op density
	lea	-128($n_ptr), $n_ptr	# control u-op density

	mulx	%rdx, @acc[0], @acc[1]	# a[0]*a[0]
	call	__mulx_mont_384

	movd	%xmm1, @acc[2]d
	dec	@acc[2]d
	jnz	.Loop_sqrx_384

	mov	%rdx, @acc[6]
	movq	%xmm2, %rdx		# b[0]
	lea	-128($b_ptr), $a_ptr	# control u-op density
	mov	8*3(%rsp), $b_ptr	# 6th, multiplicand argument
	lea	-128($n_ptr), $n_ptr	# control u-op density

	mulx	@acc[6],@acc[0],@acc[1]	# a[0]*b[0]
	call	__mulx_mont_384

	mov	8*5(%rsp),%r15
.cfi_restore	%r15
	mov	8*6(%rsp),%r14
.cfi_restore	%r14
	mov	8*7(%rsp),%r13
.cfi_restore	%r13
	mov	8*8(%rsp),%r12
.cfi_restore	%r12
	mov	8*9(%rsp),%rbx
.cfi_restore	%rbx
	mov	8*10(%rsp),%rbp
.cfi_restore	%rbp
	lea	8*11(%rsp),%rsp
.cfi_adjust_cfa_offset	-8*11
.cfi_epilogue
	ret
.cfi_endproc
.size	sqrx_n_mul_mont_384,.-sqrx_n_mul_mont_384

.globl	sqrx_n_mul_mont_383
.hidden	sqrx_n_mul_mont_383
.type	sqrx_n_mul_mont_383,\@function,6,"unwind"
.align	32
sqrx_n_mul_mont_383:
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
	lea	-8*5(%rsp), %rsp
.cfi_adjust_cfa_offset	8*5
.cfi_end_prologue

	mov	$b_org, @acc[2]		# loop counter
	mov	8*0($a_ptr), %rdx
	mov	8*1($a_ptr), @acc[7]
	mov	8*2($a_ptr), @acc[8]
	mov	$a_ptr, $b_ptr
	mov	8*3($a_ptr), @acc[4]
	mov	$r_ptr, 8*2(%rsp)	# to __mulx_mont_383_nonred
	mov	8*4($a_ptr), $lo
	mov	8*5($a_ptr), $hi

	mov	$n0, (%rsp)
	mov	%r9, 8*3(%rsp)		# 6th, multiplicand argument
	movq	8*0(%r9), %xmm2		# prefetch b[0]
	lea	-128($n_ptr), $n_ptr	# control u-op density

.Loop_sqrx_383:
	movd	@acc[2]d, %xmm1
	lea	-128($b_ptr), $a_ptr	# control u-op density

	mulx	%rdx, @acc[0], @acc[1]	# a[0]*a[0]
	call	__mulx_mont_383_nonred	# omitting full reduction gives ~15%
					# in addition-chains
	movd	%xmm1, @acc[2]d
	dec	@acc[2]d
	jnz	.Loop_sqrx_383

	mov	%rdx, @acc[6]
	movq	%xmm2, %rdx		# b[0]
	lea	-128($b_ptr), $a_ptr	# control u-op density
	mov	8*3(%rsp), $b_ptr	# 6th, multiplicand argument

	mulx	@acc[6], @acc[0], @acc[1]	# a[0]*b[0]
	call	__mulx_mont_384

	mov	8*5(%rsp),%r15
.cfi_restore	%r15
	mov	8*6(%rsp),%r14
.cfi_restore	%r14
	mov	8*7(%rsp),%r13
.cfi_restore	%r13
	mov	8*8(%rsp),%r12
.cfi_restore	%r12
	mov	8*9(%rsp),%rbx
.cfi_restore	%rbx
	mov	8*10(%rsp),%rbp
.cfi_restore	%rbp
	lea	8*11(%rsp),%rsp
.cfi_adjust_cfa_offset	-8*11
.cfi_epilogue
	ret
.cfi_endproc
.size	sqrx_n_mul_mont_383,.-sqrx_n_mul_mont_383
___
{ my @acc=@acc;				# will be rotated locally

$code.=<<___;
.type	__mulx_mont_383_nonred,\@abi-omnipotent
.align	32
__mulx_mont_383_nonred:
.cfi_startproc
	mulx	@acc[7], @acc[6], @acc[2]
	mulx	@acc[8], @acc[7], @acc[3]
	add	@acc[6], @acc[1]
	mulx	@acc[4], @acc[8], @acc[4]
	adc	@acc[7], @acc[2]
	mulx	$lo, $lo, @acc[5]
	adc	@acc[8], @acc[3]
	mulx	$hi, $hi, @acc[6]
	 mov	8($b_ptr), %rdx
	adc	$lo, @acc[4]
	adc	$hi, @acc[5]
	adc	\$0, @acc[6]
___
for (my $i=1; $i<6; $i++) {
my $tt = $i==1 ? @acc[7] : $hi;
my $b_next = $i<5 ? 8*($i+1)."($b_ptr)" : @acc[1];
$code.=<<___;
	 mov	@acc[0], @acc[8]
	 imulq	8(%rsp), @acc[0]

	################################# Multiply by b[$i]
	xor	@acc[7], @acc[7]	# @acc[8]=0, cf=0, of=0
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
	adox	$lo, @acc[4]
	adcx	$hi, @acc[5]

	mulx	8*4+128($a_ptr), $lo, $hi
	adox	$lo, @acc[5]
	adcx	$hi, @acc[6]

	mulx	8*5+128($a_ptr), $lo, $hi
	 mov	@acc[0], %rdx
	adox	$lo, @acc[6]
	adcx	@acc[7], $hi
	adox	$hi, @acc[7]

	################################# reduction
	xor	@acc[0], @acc[0]	# acc[0]=0, cf=0, of=0
	mulx	8*0+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[8]		# guaranteed to be zero
	adox	$hi, @acc[1]

	mulx	8*1+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[1]
	adox	$hi, @acc[2]

	mulx	8*2+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[2]
	adox	$hi, @acc[3]

	mulx	8*3+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[3]
	adox	$hi, @acc[4]

	mulx	8*4+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[4]
	adox	$hi, @acc[5]

	mulx	8*5+128($n_ptr), $lo, $hi
	 mov	$b_next, %rdx
	adcx	$lo, @acc[5]
	adox	$hi, @acc[6]
	adcx	@acc[8], @acc[6]
	adox	@acc[8], @acc[7]
	adcx	@acc[8], @acc[7]
___
    push(@acc,shift(@acc));
}
$code.=<<___;
	imulq	8(%rsp), %rdx
	mov	8*3(%rsp), $b_ptr	# restore $r_ptr

	################################# last reduction
	xor	@acc[8], @acc[8]	# @acc[8]=0, cf=0, of=0
	mulx	8*0+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[0]		# guaranteed to be zero
	adox	$hi, @acc[1]

	mulx	8*1+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[1]
	adox	$hi, @acc[2]

	mulx	8*2+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[2]
	adox	$hi, @acc[3]

	mulx	8*3+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[3]
	adox	$hi, @acc[4]

	mulx	8*4+128($n_ptr), $lo, $hi
	adcx	$lo, @acc[4]
	adox	$hi, @acc[5]

	mulx	8*5+128($n_ptr), $lo, $hi
	 mov	@acc[1], %rdx
	adcx	$lo, @acc[5]
	adox	$hi, @acc[6]
	adc	\$0, @acc[6]
	 mov	@acc[4], @acc[8]

	mov	@acc[1], 8*0($b_ptr)
	mov	@acc[2], 8*1($b_ptr)
	mov	@acc[3], 8*2($b_ptr)
	 mov	@acc[5], $lo
	mov	@acc[4], 8*3($b_ptr)
	mov	@acc[5], 8*4($b_ptr)
	mov	@acc[6], 8*5($b_ptr)
	 mov	@acc[6], $hi

	ret
.cfi_endproc
.size	__mulx_mont_383_nonred,.-__mulx_mont_383_nonred
___
} } }
{ my $frame = 4*8 +	# place for argument off-load +
	      2*384/8 +	# place for 2 384-bit temporary vectors
	      8;	# align
my @acc = (@acc,"%rax","%rdx","%rbx","%rbp");

# omitting 3 reductions gives ~10% better performance in add-chains
$code.=<<___;
.globl	sqrx_mont_382x
.hidden	sqrx_mont_382x
.type	sqrx_mont_382x,\@function,4,"unwind"
.align	32
sqrx_mont_382x:
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

	mov	$n_ptr, 8*0(%rsp)	# n0
	mov	$b_org, $n_ptr		# n_ptr
	mov	$r_ptr, 8*2(%rsp)
	mov	$a_ptr, 8*3(%rsp)

	#################################
	mov	8*0($a_ptr), @acc[0]	# a->re
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

	sub	8*6($a_ptr), @acc[6]	# a->re - a->im
	sbb	8*7($a_ptr), @acc[7]
	sbb	8*8($a_ptr), @acc[8]
	sbb	8*9($a_ptr), @acc[9]
	sbb	8*10($a_ptr), @acc[10]
	sbb	8*11($a_ptr), @acc[11]
	sbb	$r_ptr, $r_ptr		# borrow flag as mask

	mov	@acc[0], 32+8*0(%rsp)	# t0
	mov	@acc[1], 32+8*1(%rsp)
	mov	@acc[2], 32+8*2(%rsp)
	mov	@acc[3], 32+8*3(%rsp)
	mov	@acc[4], 32+8*4(%rsp)
	mov	@acc[5], 32+8*5(%rsp)

	mov	@acc[6], 32+8*6(%rsp)	# t1
	mov	@acc[7], 32+8*7(%rsp)
	mov	@acc[8], 32+8*8(%rsp)
	mov	@acc[9], 32+8*9(%rsp)
	mov	@acc[10], 32+8*10(%rsp)
	mov	@acc[11], 32+8*11(%rsp)
	mov	$r_ptr,   32+8*12(%rsp)

	################################# mul_mont_384(ret->im, a->re, a->im, mod, n0);
	#mov	8*3(%rsp), $a_ptr	# a->re
	lea	48($a_ptr), $b_ptr	# a->im

	mov	48($a_ptr), %rdx
	mov	8*0($a_ptr), %r14	# @acc[6]
	mov	8*1($a_ptr), %r15	# @acc[7]
	mov	8*2($a_ptr), %rax	# @acc[8]
	mov	8*3($a_ptr), %r12	# @acc[4]
	mov	8*4($a_ptr), %rdi	# $lo
	mov	8*5($a_ptr), %rbp	# $hi
	lea	-128($a_ptr), $a_ptr	# control u-op density
	lea	-128($n_ptr), $n_ptr	# control u-op density

	mulx	%r14, %r8, %r9
	call	__mulx_mont_383_nonred
___
{
my @acc = map("%r$_","dx",15,"ax",12,"di","bp",	# output from __mulx_mont_384
                      8..11,13,14);
$code.=<<___;
	add	@acc[0], @acc[0]	# add with itself
	adc	@acc[1], @acc[1]
	adc	@acc[2], @acc[2]
	adc	@acc[3], @acc[3]
	adc	@acc[4], @acc[4]
	adc	@acc[5], @acc[5]

	mov	@acc[0],  8*6($b_ptr)	# ret->im
	mov	@acc[1],  8*7($b_ptr)
	mov	@acc[2],  8*8($b_ptr)
	mov	@acc[3],  8*9($b_ptr)
	mov	@acc[4],  8*10($b_ptr)
	mov	@acc[5],  8*11($b_ptr)
___
}
$code.=<<___;
	################################# mul_mont_384(ret->re, t0, t1, mod, n0);
	lea	32-128(%rsp), $a_ptr	# t0 [+u-op density]
	lea	32+8*6(%rsp), $b_ptr	# t1

	mov	32+8*6(%rsp), %rdx	# t1[0]
	mov	32+8*0(%rsp), %r14	# @acc[6]
	mov	32+8*1(%rsp), %r15	# @acc[7]
	mov	32+8*2(%rsp), %rax	# @acc[8]
	mov	32+8*3(%rsp), %r12	# @acc[4]
	mov	32+8*4(%rsp), %rdi	# $lo
	mov	32+8*5(%rsp), %rbp	# $hi
	#lea	-128($a_ptr), $a_ptr	# control u-op density
	#lea	-128($n_ptr), $n_ptr	# control u-op density

	mulx	%r14, %r8, %r9
	call	__mulx_mont_383_nonred
___
{
my @acc = map("%r$_","dx",15,"ax",12,"di","bp",	# output from __mulx_mont_384
                      8..11,13,14);
$code.=<<___;
	mov	32+8*12(%rsp), @acc[11]	# account for sign from a->re - a->im
	lea	128($n_ptr), $n_ptr
	mov	32+8*0(%rsp), @acc[6]
	and	@acc[11], @acc[6]
	mov	32+8*1(%rsp), @acc[7]
	and	@acc[11], @acc[7]
	mov	32+8*2(%rsp), @acc[8]
	and	@acc[11], @acc[8]
	mov	32+8*3(%rsp), @acc[9]
	and	@acc[11], @acc[9]
	mov	32+8*4(%rsp), @acc[10]
	and	@acc[11], @acc[10]
	and	32+8*5(%rsp), @acc[11]

	sub	@acc[6], @acc[0]
	mov	8*0($n_ptr), @acc[6]
	sbb	@acc[7], @acc[1]
	mov	8*1($n_ptr), @acc[7]
	sbb	@acc[8], @acc[2]
	mov	8*2($n_ptr), @acc[8]
	sbb	@acc[9], @acc[3]
	mov	8*3($n_ptr), @acc[9]
	sbb	@acc[10], @acc[4]
	mov	8*4($n_ptr), @acc[10]
	sbb	@acc[11], @acc[5]
	sbb	@acc[11], @acc[11]

	and	@acc[11], @acc[6]
	and	@acc[11], @acc[7]
	and	@acc[11], @acc[8]
	and	@acc[11], @acc[9]
	and	@acc[11], @acc[10]
	and	8*5($n_ptr), @acc[11]

	add	@acc[6], @acc[0]
	adc	@acc[7], @acc[1]
	adc	@acc[8], @acc[2]
	adc	@acc[9], @acc[3]
	adc	@acc[10], @acc[4]
	adc	@acc[11], @acc[5]

	mov	@acc[0],  8*0($b_ptr)	# ret->re
	mov	@acc[1],  8*1($b_ptr)
	mov	@acc[2],  8*2($b_ptr)
	mov	@acc[3],  8*3($b_ptr)
	mov	@acc[4],  8*4($b_ptr)
	mov	@acc[5],  8*5($b_ptr)
___
}
$code.=<<___;
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
.size	sqrx_mont_382x,.-sqrx_mont_382x
___
}

print $code;
close STDOUT;
