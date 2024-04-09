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

$code.=<<___ if ($flavour =~ /masm/);
.extern	mul_mont_384x\$1
.extern	sqr_mont_384x\$1
.extern	mul_382x\$1
.extern	sqr_382x\$1
.extern	mul_384\$1
.extern	sqr_384\$1
.extern	redc_mont_384\$1
.extern	from_mont_384\$1
.extern	sgn0_pty_mont_384\$1
.extern	sgn0_pty_mont_384x\$1
.extern	mul_mont_384\$1
.extern	sqr_mont_384\$1
.extern	sqr_n_mul_mont_384\$1
.extern	sqr_n_mul_mont_383\$1
.extern	sqr_mont_382x\$1
___

# common argument layout
($r_ptr,$a_ptr,$b_org,$n_ptr,$n0) = ("%rdi","%rsi","%rdx","%rcx","%r8");
$b_ptr = "%rbx";

# common accumulator layout
@acc=map("%r$_",(8..15));

########################################################################
{ my @acc=(@acc,"%rax","%rbx","%rbp",$a_ptr);	# all registers are affected
						# except for $n_ptr and $r_ptr
$code.=<<___;
.comm	__blst_platform_cap,4
.text

########################################################################
# Double-width subtraction modulo n<<384, as opposite to naively
# expected modulo n*n. It works because n<<384 is the actual
# input boundary condition for Montgomery reduction, not n*n.
# Just in case, this is duplicated, but only one module is
# supposed to be linked...
.type	__subq_mod_384x384,\@abi-omnipotent
.align	32
__subq_mod_384x384:
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
.size	__subq_mod_384x384,.-__subq_mod_384x384

.type	__addq_mod_384,\@abi-omnipotent
.align	32
__addq_mod_384:
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
.size	__addq_mod_384,.-__addq_mod_384

.type	__subq_mod_384,\@abi-omnipotent
.align	32
__subq_mod_384:
	mov	8*0($a_ptr), @acc[0]
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]

__subq_mod_384_a_is_loaded:
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
.size	__subq_mod_384,.-__subq_mod_384
___
}

########################################################################
# "Complex" multiplication and squaring. Use vanilla multiplication when
# possible to fold reductions. I.e. instead of mul_mont, mul_mont
# followed by add/sub_mod, it calls mul, mul, double-width add/sub_mod
# followed by *common* reduction...
{ my $frame = 5*8 +	# place for argument off-load +
	      3*768/8;	# place for 3 768-bit temporary vectors
$code.=<<___;
.globl	mul_mont_384x
.hidden	mul_mont_384x
.type	mul_mont_384x,\@function,5,"unwind"
.align	32
mul_mont_384x:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	mul_mont_384x\$1
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
	call	__mulq_384

	################################# mul_384(t1, a->im, b->im);
	lea	48($b_ptr), $b_ptr	# b->im
	lea	48($a_ptr), $a_ptr	# a->im
	lea	40+96(%rsp), $r_ptr	# t1
	call	__mulq_384

	################################# mul_384(t2, a->re+a->im, b->re+b->im);
	mov	8*1(%rsp), $n_ptr
	lea	-48($a_ptr), $b_org
	lea	40+192+48(%rsp), $r_ptr
	call	__addq_mod_384

	mov	8*2(%rsp), $a_ptr
	lea	48($a_ptr), $b_org
	lea	-48($r_ptr), $r_ptr
	call	__addq_mod_384

	lea	($r_ptr),$b_ptr
	lea	48($r_ptr),$a_ptr
	call	__mulq_384

	################################# t2=t2-t0-t1
	lea	($r_ptr), $a_ptr	# t2
	lea	40(%rsp), $b_org	# t0
	mov	8*1(%rsp), $n_ptr
	call	__subq_mod_384x384	# t2=t2-t0

	lea	($r_ptr), $a_ptr	# t2
	lea	-96($r_ptr), $b_org	# t1
	call	__subq_mod_384x384	# t2=t2-t1

	################################# t0=t0-t1
	lea	40(%rsp), $a_ptr
	lea	40+96(%rsp), $b_org
	lea	40(%rsp), $r_ptr
	call	__subq_mod_384x384	# t0-t1

	mov	$n_ptr, $b_ptr		# n_ptr for redc_mont_384

	################################# redc_mont_384(ret->re, t0, mod, n0);
	lea	40(%rsp), $a_ptr	# t0
	mov	8*0(%rsp), %rcx		# n0 for redc_mont_384
	mov	8*4(%rsp), $r_ptr	# ret->re
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

	################################# redc_mont_384(ret->im, t2, mod, n0);
	lea	40+192(%rsp), $a_ptr	# t2
	mov	8*0(%rsp), %rcx		# n0 for redc_mont_384
	lea	48($r_ptr), $r_ptr	# ret->im
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

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
.size	mul_mont_384x,.-mul_mont_384x
___
}
{ my $frame = 4*8 +	# place for argument off-load +
	      2*384/8 +	# place for 2 384-bit temporary vectors
	      8;	# align
$code.=<<___;
.globl	sqr_mont_384x
.hidden	sqr_mont_384x
.type	sqr_mont_384x,\@function,4,"unwind"
.align	32
sqr_mont_384x:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	sqr_mont_384x\$1
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

	mov	$n_ptr, 8*0(%rsp)	# n0
	mov	$b_org, $n_ptr		# n_ptr
	mov	$r_ptr, 8*1(%rsp)	# to __mulq_mont_384
	mov	$a_ptr, 8*2(%rsp)

	################################# add_mod_384(t0, a->re, a->im);
	lea	48($a_ptr), $b_org	# a->im
	lea	32(%rsp), $r_ptr	# t0
	call	__addq_mod_384

	################################# sub_mod_384(t1, a->re, a->im);
	mov	8*2(%rsp), $a_ptr	# a->re
	lea	48($a_ptr), $b_org	# a->im
	lea	32+48(%rsp), $r_ptr	# t1
	call	__subq_mod_384

	################################# mul_mont_384(ret->im, a->re, a->im, mod, n0);
	mov	8*2(%rsp), $a_ptr	# a->re
	lea	48($a_ptr), $b_ptr	# a->im

	mov	48($a_ptr), %rax	# a->im
	mov	8*0($a_ptr), @acc[6]	# a->re
	mov	8*1($a_ptr), @acc[7]
	mov	8*2($a_ptr), @acc[4]
	mov	8*3($a_ptr), @acc[5]

	call	__mulq_mont_384
___
{
my @acc = map("%r$_",14,15,8..11,	# output from __mulq_mont_384
                     12,13,"ax","bx","bp","si");
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
	mov	@acc[0],  8*6($r_ptr)	# ret->im
	cmovc	@acc[9],  @acc[3]
	mov	@acc[1],  8*7($r_ptr)
	cmovc	@acc[10], @acc[4]
	mov	@acc[2],  8*8($r_ptr)
	cmovc	@acc[11], @acc[5]
	mov	@acc[3],  8*9($r_ptr)
	mov	@acc[4],  8*10($r_ptr)
	mov	@acc[5],  8*11($r_ptr)
___
}
$code.=<<___;
	################################# mul_mont_384(ret->re, t0, t1, mod, n0);
	lea	32(%rsp), $a_ptr	# t0
	lea	32+48(%rsp), $b_ptr	# t1

	mov	32+48(%rsp), %rax	# t1[0]
	mov	32+8*0(%rsp), @acc[6]	# t0[0..3]
	mov	32+8*1(%rsp), @acc[7]
	mov	32+8*2(%rsp), @acc[4]
	mov	32+8*3(%rsp), @acc[5]

	call	__mulq_mont_384

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
.size	sqr_mont_384x,.-sqr_mont_384x

.globl	mul_382x
.hidden	mul_382x
.type	mul_382x,\@function,4,"unwind"
.align	32
mul_382x:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	mul_382x\$1
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
	call	__mulq_384

	################################# mul_384(ret->re, a->re, b->re);
	mov	8*0(%rsp), $a_ptr
	mov	8*1(%rsp), $b_ptr
	lea	-96($r_ptr), $r_ptr	# ret->re
	call	__mulq_384

	################################# mul_384(tx, a->im, b->im);
	lea	48($a_ptr), $a_ptr
	lea	48($b_ptr), $b_ptr
	lea	32(%rsp), $r_ptr
	call	__mulq_384

	################################# ret->im -= tx
	mov	8*2(%rsp), $a_ptr	# restore ret->im
	lea	32(%rsp), $b_org
	mov	8*3(%rsp), $n_ptr
	mov	$a_ptr, $r_ptr
	call	__subq_mod_384x384

	################################# ret->im -= ret->re
	lea	0($r_ptr), $a_ptr
	lea	-96($r_ptr), $b_org
	call	__subq_mod_384x384

	################################# ret->re -= tx
	lea	-96($r_ptr), $a_ptr
	lea	32(%rsp), $b_org
	lea	-96($r_ptr), $r_ptr
	call	__subq_mod_384x384

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
.size	mul_382x,.-mul_382x
___
}
{ my @acc=(@acc,"%rax","%rbx","%rbp",$b_org);	# all registers are affected
						# except for $n_ptr and $r_ptr
$code.=<<___;
.globl	sqr_382x
.hidden	sqr_382x
.type	sqr_382x,\@function,3,"unwind"
.align	32
sqr_382x:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	sqr_382x\$1
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
	call	__subq_mod_384_a_is_loaded

	################################# mul_384(ret->re, t0, t1);
	lea	($r_ptr), $a_ptr
	lea	-48($r_ptr), $b_ptr
	lea	-48($r_ptr), $r_ptr
	call	__mulq_384

	################################# mul_384(ret->im, a->re, a->im);
	mov	(%rsp), $a_ptr
	lea	48($a_ptr), $b_ptr
	lea	96($r_ptr), $r_ptr
	call	__mulq_384

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
.size	sqr_382x,.-sqr_382x
___
}
{ ########################################################## 384-bit mul
my @acc=map("%r$_",("cx",8..12));
my $bi = "%rbp";

$code.=<<___;
.globl	mul_384
.hidden	mul_384
.type	mul_384,\@function,3,"unwind"
.align	32
mul_384:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	mul_384\$1
#endif
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	push	%r12
.cfi_push	%r12
.cfi_end_prologue

	mov	$b_org, $b_ptr
	call	__mulq_384

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
.size	mul_384,.-mul_384

.type	__mulq_384,\@abi-omnipotent
.align	32
__mulq_384:
	mov	8*0($b_ptr), %rax

	mov	%rax, $bi
	mulq	8*0($a_ptr)
	mov	%rax, 8*0($r_ptr)
	mov	$bi, %rax
	mov	%rdx, @acc[0]

	mulq	8*1($a_ptr)
	add	%rax, @acc[0]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[1]

	mulq	8*2($a_ptr)
	add	%rax, @acc[1]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[2]

	mulq	8*3($a_ptr)
	add	%rax, @acc[2]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[3]

	mulq	8*4($a_ptr)
	add	%rax, @acc[3]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[4]

	mulq	8*5($a_ptr)
	add	%rax, @acc[4]
	mov	8*1($b_ptr), %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[5]
___
for(my $i=1; $i<6; $i++) {
my $b_next = $i<5 ? 8*($i+1)."($b_ptr)" : "%rax";
$code.=<<___;
	mov	%rax, $bi
	mulq	8*0($a_ptr)
	add	%rax, @acc[0]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	@acc[0], 8*$i($r_ptr)
	mov	%rdx, @acc[0]

	mulq	8*1($a_ptr)
	add	%rax, @acc[1]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[1], @acc[0]
	adc	\$0, %rdx
	mov	%rdx, @acc[1]

	mulq	8*2($a_ptr)
	add	%rax, @acc[2]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[2], @acc[1]
	adc	\$0, %rdx
	mov	%rdx, @acc[2]

	mulq	8*3($a_ptr)
	add	%rax, @acc[3]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[3], @acc[2]
	adc	\$0, %rdx
	mov	%rdx, @acc[3]

	mulq	8*4($a_ptr)
	add	%rax, @acc[4]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[4], @acc[3]
	adc	\$0, %rdx
	mov	%rdx, @acc[4]

	mulq	8*5($a_ptr)
	add	%rax, @acc[5]
	mov	$b_next, %rax
	adc	\$0, %rdx
	add	@acc[5], @acc[4]
	adc	\$0, %rdx
	mov	%rdx, @acc[5]
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
.size	__mulq_384,.-__mulq_384
___
}
if (0) { ##############################################################
my @b=map("%r$_",(10..15));
my @a=reverse(@b);
   @b[5]=$b_ptr;
my $bi = "%rbp";
my @comba=map("%r$_",("cx",8,9));
#                                                   a[0]*b[0]
#                                              a[1]*b[0]
#                                              a[0]*b[1]
#                                         a[2]*b[0]
#                                         a[1]*b[1]
#                                         a[0]*b[2]
#                                    a[3]*b[0]
#                                    a[2]*b[1]
#                                    a[1]*b[2]
#                                    a[0]*b[3]
#                               a[4]*b[0]
#                               a[3]*b[1]
#                               a[2]*b[2]
#                               a[1]*b[3]
#                               a[0]*b[4]
#                          a[5]*b[0]
#                          a[4]*b[1]
#                          a[3]*b[2]
#                          a[2]*b[3]
#                          a[1]*b[4]
#                          a[0]*b[5]
#                     a[5]*b[1]
#                     a[4]*b[2]
#                     a[3]*b[3]
#                     a[2]*b[4]
#                     a[1]*b[5]
#                a[5]*b[2]
#                a[4]*b[3]
#                a[3]*b[4]
#                a[2]*b[5]
#           a[5]*b[3]
#           a[4]*b[4]
#           a[3]*b[5]
#      a[5]*b[4]
#      a[4]*b[5]
# a[5]*b[5]
#
# 13% less instructions give +15% on Core2, +10% on Goldmont,
# -0% on Sandy Bridge, but -16% on Haswell:-(
# [for reference +5% on Skylake, +11% on Ryzen]

$code.=<<___;
.type	__mulq_comba_384,\@abi-omnipotent
.align	32
__mulq_comba_384:
	mov	8*0($b_ptr), %rax
	mov	8*0($a_ptr), @a[0]
	mov	8*1($a_ptr), @a[1]
	mov	8*1($b_ptr), @b[1]

	mov	%rax, @b[0]
	mulq	@a[0]			# a[0]*b[0]
	mov	%rax, 8*0($r_ptr)
	mov	@b[0], %rax
	mov	%rdx, @comba[0]

	#################################
	mov	8*2($a_ptr), @a[2]
	xor	@comba[2], @comba[2]
	mulq	@a[1]			# a[1]*b[0]
	add	%rax, @comba[0]
	mov	@b[1], %rax
	adc	\$0, %rdx
	mov	8*2($b_ptr), @b[2]
	mov	%rdx, @comba[1]

	mulq	@a[0]			# a[0]*b[1]
	add	%rax, @comba[0]
	mov	@b[0], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]
	mov	@comba[0], 8*1($r_ptr)
___
    push(@comba,shift(@comba));
$code.=<<___;
	xor	@comba[2], @comba[2]
	mulq	@a[2]			# a[2]*b[0]
	add	%rax, @comba[0]
	mov	@b[1], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	@a[1]			# a[1]*b[1]
	add	%rax, @comba[0]
	mov	@b[2], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	@a[0]			# a[0]*b[2]
	add	%rax, @comba[0]
	mov	@b[0], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]
	mov	@comba[0], 8*2($r_ptr)
___
    push(@comba,shift(@comba));
$code.=<<___;
	xor	@comba[2], @comba[2]
	mulq	8*3($a_ptr)		# a[3]*b[0]
	add	%rax, @comba[0]
	mov	@b[1], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	@a[2]			# a[2]*b[1]
	add	%rax, @comba[0]
	mov	@b[2], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	@a[1]			# a[1]*b[2]
	add	%rax, @comba[0]
	mov	8*3($b_ptr), %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mov	%rax, @b[3]
	mulq	@a[0]			# a[0]*b[3]
	add	%rax, @comba[0]
	mov	@b[0], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]
	mov	@comba[0], 8*3($r_ptr)
___
    push(@comba,shift(@comba));
$code.=<<___;
	xor	@comba[2], @comba[2]
	mulq	8*4($a_ptr)		# a[4]*b[0]
	add	%rax, @comba[0]
	mov	@b[1], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	8*3($a_ptr)		# a[3]*b[1]
	add	%rax, @comba[0]
	mov	@b[2], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	8*2($a_ptr)		# a[2]*b[2]
	add	%rax, @comba[0]
	mov	@b[3], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	@a[1]			# a[1]*b[3]
	add	%rax, @comba[0]
	mov	8*4($b_ptr), %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mov	%rax, @b[4]
	mulq	@a[0]			# a[0]*b[4]
	add	%rax, @comba[0]
	mov	@b[0], %rax
	adc	%rdx, @comba[1]
	mov	8*5($a_ptr), @a[5]
	adc	\$0, @comba[2]
	mov	@comba[0], 8*4($r_ptr)
___
    push(@comba,shift(@comba));
$code.=<<___;
	xor	@comba[2], @comba[2]
	mulq	@a[5]			# a[5]*b[0]
	add	%rax, @comba[0]
	mov	@b[1], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	8*4($a_ptr)		# a[4]*b[1]
	add	%rax, @comba[0]
	mov	@b[2], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	8*3($a_ptr)		# a[3]*b[2]
	add	%rax, @comba[0]
	mov	@b[3], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	8*2($a_ptr)		# a[2]*b[3]
	add	%rax, @comba[0]
	mov	@b[4], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	8*1($a_ptr)		# a[1]*b[4]
	add	%rax, @comba[0]
	mov	8*5($b_ptr), %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mov	%rax, @b[5]
	mulq	@a[0]			# a[0]*b[5]
	add	%rax, @comba[0]
	mov	@b[1], %rax
	adc	%rdx, @comba[1]
	mov	8*4($a_ptr), @a[4]
	adc	\$0, @comba[2]
	mov	@comba[0], 8*5($r_ptr)
___
    push(@comba,shift(@comba));
$code.=<<___;
	xor	@comba[2], @comba[2]
	mulq	@a[5]			# a[5]*b[1]
	add	%rax, @comba[0]
	mov	@b[2], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	@a[4]			# a[4]*b[2]
	add	%rax, @comba[0]
	mov	@b[3], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	8*3($a_ptr)		# a[3]*b[3]
	add	%rax, @comba[0]
	mov	@b[4], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	8*2($a_ptr)		# a[2]*b[4]
	add	%rax, @comba[0]
	mov	@b[5], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	8*1($a_ptr)		# a[1]*b[5]
	add	%rax, @comba[0]
	mov	$b[2], %rax
	adc	%rdx, @comba[1]
	mov	8*3($a_ptr), @a[3]
	adc	\$0, @comba[2]
	mov	@comba[0], 8*6($r_ptr)
___
    push(@comba,shift(@comba));
$code.=<<___;
	xor	@comba[2], @comba[2]
	mulq	@a[5]			# a[5]*b[2]
	add	%rax, @comba[0]
	mov	@b[3], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	@a[4]			# a[4]*b[3]
	add	%rax, @comba[0]
	mov	@b[4], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	@a[3]			# a[3]*b[4]
	add	%rax, @comba[0]
	mov	@b[5], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	8*2($a_ptr)		# a[2]*b[5]
	add	%rax, @comba[0]
	mov	@b[3], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]
	mov	@comba[0], 8*7($r_ptr)
___
    push(@comba,shift(@comba));
$code.=<<___;
	xor	@comba[2], @comba[2]
	mulq	@a[5]			# a[5]*b[3]
	add	%rax, @comba[0]
	mov	@b[4], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	@a[4]			# a[4]*b[4]
	add	%rax, @comba[0]
	mov	@b[5], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	@a[3]			# a[3]*b[5]
	add	%rax, @comba[0]
	mov	@b[4], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]
	mov	@comba[0], 8*8($r_ptr)
___
    push(@comba,shift(@comba));
$code.=<<___;
	xor	@comba[2], @comba[2]
	mulq	@a[5]			# a[5]*b[4]
	add	%rax, @comba[0]
	mov	@b[5], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]

	mulq	@a[4]			# a[4]*b[5]
	add	%rax, @comba[0]
	mov	@b[5], %rax
	adc	%rdx, @comba[1]
	adc	\$0, @comba[2]
	mov	@comba[0], 8*9($r_ptr)
___
    push(@comba,shift(@comba));
$code.=<<___;
	mulq	@a[5]			# a[5]*b[4]
	add	%rax, @comba[0]
	adc	%rdx, @comba[1]

	mov	@comba[0], 8*10($r_ptr)
	mov	@comba[1], 8*11($r_ptr)

	ret
.size	__mulq_comba_384,.-__mulq_comba_384
___
}
{ ########################################################## 384-bit sqr
my @acc=(@acc,"%rcx","%rbx","%rbp",$a_ptr);
my $hi;

$code.=<<___;
.globl	sqr_384
.hidden	sqr_384
.type	sqr_384,\@function,2,"unwind"
.align	32
sqr_384:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	sqr_384\$1
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
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	call	__sqrq_384

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
.size	sqr_384,.-sqr_384

.type	__sqrq_384,\@abi-omnipotent
.align	32
__sqrq_384:
	mov	8*0($a_ptr), %rax
	mov	8*1($a_ptr), @acc[7]
	mov	8*2($a_ptr), @acc[8]
	mov	8*3($a_ptr), @acc[9]

	#########################################
	mov	%rax, @acc[6]
	mulq	@acc[7]				# a[1]*a[0]
	mov	%rax, @acc[1]
	mov	@acc[6], %rax
	 mov	8*4($a_ptr), @acc[10]
	mov	%rdx, @acc[2]

	mulq	@acc[8]				# a[2]*a[0]
	add	%rax, @acc[2]
	mov	@acc[6], %rax
	adc	\$0, %rdx
	 mov	8*5($a_ptr), @acc[11]
	mov	%rdx, @acc[3]

	mulq	@acc[9]				# a[3]*a[0]
	add	%rax, @acc[3]
	mov	@acc[6], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[4]

	mulq	@acc[10]			# a[4]*a[0]
	add	%rax, @acc[4]
	mov	@acc[6], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[5]

	mulq	@acc[11]			# a[5]*a[0]
	add	%rax, @acc[5]
	mov	@acc[6], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[6]

	mulq	%rax				# a[0]*a[0]
	xor	@acc[0], @acc[0]
	mov	%rax, 8*0($r_ptr)
	 mov	@acc[7], %rax
	add	@acc[1], @acc[1]		# double acc[1]
	adc	\$0, @acc[0]
	add	%rdx, @acc[1]			# accumulate a[0]*a[0]
	adc	\$0, @acc[0]			# carries to a[1]*a[1]
	mov	@acc[1], 8*1($r_ptr)
___
$hi=@acc[1];
$code.=<<___;
	#########################################
	mulq	@acc[8]				# a[2]*a[1]
	add	%rax, @acc[3]
	mov	@acc[7], %rax
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	@acc[9]				# a[3]*a[1]
	add	%rax, @acc[4]
	mov	@acc[7], %rax
	adc	\$0, %rdx
	add	$hi, @acc[4]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	@acc[10]			# a[4]*a[1]
	add	%rax, @acc[5]
	mov	@acc[7], %rax
	adc	\$0, %rdx
	add	$hi, @acc[5]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	@acc[11]			# a[5]*a[1]
	add	%rax, @acc[6]
	mov	@acc[7], %rax
	adc	\$0, %rdx
	add	$hi, @acc[6]
	adc	\$0, %rdx
	mov	%rdx, @acc[7]

	mulq	%rax				# a[1]*a[1]
	xor	@acc[1], @acc[1]
	add	%rax, @acc[0]			# can't carry
	 mov	@acc[8], %rax
	add	@acc[2], @acc[2]		# double acc[2:3]
	adc	@acc[3], @acc[3]
	adc	\$0, @acc[1]
	add	@acc[0], @acc[2]		# accumulate a[1]*a[1]
	adc	%rdx, @acc[3]
	adc	\$0, @acc[1]			# carries to a[2]*a[2]
	mov	@acc[2], 8*2($r_ptr)
___
$hi=@acc[0];
$code.=<<___;
	#########################################
	mulq	@acc[9]				# a[3]*a[2]
	add	%rax, @acc[5]
	mov	@acc[8], %rax
	adc	\$0, %rdx
	 mov	@acc[3], 8*3($r_ptr)
	mov	%rdx, $hi

	mulq	@acc[10]			# a[4]*a[2]
	add	%rax, @acc[6]
	mov	@acc[8], %rax
	adc	\$0, %rdx
	add	$hi, @acc[6]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	@acc[11]			# a[5]*a[2]
	add	%rax, @acc[7]
	mov	@acc[8], %rax
	adc	\$0, %rdx
	add	$hi, @acc[7]
	adc	\$0, %rdx
	mov	%rdx, @acc[8]

	mulq	%rax				# a[2]*a[2]
	xor	@acc[3], @acc[3]
	add	%rax, @acc[1]			# can't carry
	 mov	@acc[9], %rax
	add	@acc[4], @acc[4]		# double acc[4:5]
	adc	@acc[5], @acc[5]
	adc	\$0, @acc[3]
	add	@acc[1], @acc[4]		# accumulate a[2]*a[2]
	adc	%rdx, @acc[5]
	adc	\$0, @acc[3]			# carries to a[3]*a[3]
	mov	@acc[4], 8*4($r_ptr)

	#########################################
	mulq	@acc[10]			# a[4]*a[3]
	add	%rax, @acc[7]
	mov	@acc[9], %rax
	adc	\$0, %rdx
	 mov	@acc[5], 8*5($r_ptr)
	mov	%rdx, $hi

	mulq	@acc[11]			# a[5]*a[3]
	add	%rax, @acc[8]
	mov	@acc[9], %rax
	adc	\$0, %rdx
	add	$hi, @acc[8]
	adc	\$0, %rdx
	mov	%rdx, @acc[9]

	mulq	%rax				# a[3]*a[3]
	xor	@acc[4], @acc[4]
	add	%rax, @acc[3]			# can't carry
	 mov	@acc[10], %rax
	add	@acc[6], @acc[6]		# double acc[6:7]
	adc	@acc[7], @acc[7]
	adc	\$0, @acc[4]
	add	@acc[3], @acc[6]		# accumulate a[3]*a[3]
	adc	%rdx, @acc[7]
	mov	@acc[6], 8*6($r_ptr)
	adc	\$0, @acc[4]			# carries to a[4]*a[4]
	mov	@acc[7], 8*7($r_ptr)

	#########################################
	mulq	@acc[11]			# a[5]*a[4]
	add	%rax, @acc[9]
	mov	@acc[10], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[10]

	mulq	%rax				# a[4]*a[4]
	xor	@acc[5], @acc[5]
	add	%rax, @acc[4]			# can't carry
	 mov	@acc[11], %rax
	add	@acc[8], @acc[8]		# double acc[8:9]
	adc	@acc[9], @acc[9]
	adc	\$0, @acc[5]
	add	@acc[4], @acc[8]		# accumulate a[4]*a[4]
	adc	%rdx, @acc[9]
	mov	@acc[8], 8*8($r_ptr)
	adc	\$0, @acc[5]			# carries to a[5]*a[5]
	mov	@acc[9], 8*9($r_ptr)

	#########################################
	mulq	%rax				# a[5]*a[5]
	add	@acc[5], %rax			# can't carry
	add	@acc[10], @acc[10]		# double acc[10]
	adc	\$0, %rdx
	add	@acc[10], %rax			# accumulate a[5]*a[5]
	adc	\$0, %rdx
	mov	%rax, 8*10($r_ptr)
	mov	%rdx, 8*11($r_ptr)

	ret
.size	__sqrq_384,.-__sqrq_384

.globl	sqr_mont_384
.hidden	sqr_mont_384
.type	sqr_mont_384,\@function,4,"unwind"
.align	32
sqr_mont_384:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	sqr_mont_384\$1
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
	sub	\$8*15, %rsp
.cfi_adjust_cfa_offset	8*15
.cfi_end_prologue

	mov	$n_ptr, 8*12(%rsp)	# n0
	mov	$b_org, 8*13(%rsp)	# n_ptr
	mov	$r_ptr, 8*14(%rsp)

	mov	%rsp, $r_ptr
	call	__sqrq_384

	lea	0(%rsp), $a_ptr
	mov	8*12(%rsp), %rcx	# n0 for mul_by_1
	mov	8*13(%rsp), $b_ptr	# n_ptr for mul_by_1
	mov	8*14(%rsp), $r_ptr
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

	lea	8*15(%rsp), %r8		# size optimization
	mov	8*15(%rsp), %r15
.cfi_restore	%r15
	mov	8*1(%r8), %r14
.cfi_restore	%r14
	mov	8*2(%r8), %r13
.cfi_restore	%r13
	mov	8*3(%r8), %r12
.cfi_restore	%r12
	mov	8*4(%r8), %rbx
.cfi_restore	%rbx
	mov	8*5(%r8), %rbp
.cfi_restore	%rbp
	lea	8*6(%r8), %rsp
.cfi_adjust_cfa_offset	-8*21
.cfi_epilogue
	ret
.cfi_endproc
.size	sqr_mont_384,.-sqr_mont_384
___
}
{ ########################################################## 384-bit redc_mont
my ($n_ptr, $n0)=($b_ptr, $n_ptr);	# arguments are "shifted"

$code.=<<___;
########################################################################
# void redc_mont_384(uint64_t ret[6], const uint64_t a[12],
#                    uint64_t m[6], uint64_t n0);
.globl	redc_mont_384
.hidden	redc_mont_384
.type	redc_mont_384,\@function,4,"unwind"
.align	32
redc_mont_384:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	redc_mont_384\$1
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
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	$b_org, $n_ptr
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

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
.size	redc_mont_384,.-redc_mont_384

########################################################################
# void from_mont_384(uint64_t ret[6], const uint64_t a[6],
#                    uint64_t m[6], uint64_t n0);
.globl	from_mont_384
.hidden	from_mont_384
.type	from_mont_384,\@function,4,"unwind"
.align	32
from_mont_384:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	from_mont_384\$1
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
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	$b_org, $n_ptr
	call	__mulq_by_1_mont_384

	#################################
	# Branch-less conditional acc[0:6] - modulus

	#mov	@acc[6], %rax		# __mulq_by_1_mont_384 does it
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
.size	from_mont_384,.-from_mont_384
___
{ my @acc=@acc;				# will be rotated locally

$code.=<<___;
.type	__mulq_by_1_mont_384,\@abi-omnipotent
.align	32
__mulq_by_1_mont_384:
	mov	8*0($a_ptr), %rax
	mov	8*1($a_ptr), @acc[1]
	mov	8*2($a_ptr), @acc[2]
	mov	8*3($a_ptr), @acc[3]
	mov	8*4($a_ptr), @acc[4]
	mov	8*5($a_ptr), @acc[5]

	mov	%rax, @acc[6]
	imulq	$n0, %rax
	mov	%rax, @acc[0]
___
for (my $i=0; $i<6; $i++) {
my $hi = @acc[6];
$code.=<<___;
	################################# reduction $i
	mulq	8*0($n_ptr)
	add	%rax, @acc[6]		# guaranteed to be zero
	mov	@acc[0], %rax
	adc	%rdx, @acc[6]

	mulq	8*1($n_ptr)
	add	%rax, @acc[1]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	add	@acc[6], @acc[1]
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
	mov	@acc[0], %rax
	adc	\$0, %rdx
___
$code.=<<___	if ($i<5);
	 mov	@acc[1], @acc[7]
	 imulq	$n0, @acc[1]
___
$code.=<<___;
	add	$hi, @acc[3]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*4($n_ptr)
	add	%rax, @acc[4]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	add	$hi, @acc[4]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*5($n_ptr)
	add	%rax, @acc[5]
	mov	@acc[1], %rax
	adc	\$0, %rdx
	add	$hi, @acc[5]
	adc	\$0, %rdx
	mov	%rdx, @acc[6]
___
    push(@acc,shift(@acc));
}
$code.=<<___;
	ret
.size	__mulq_by_1_mont_384,.-__mulq_by_1_mont_384

.type	__redq_tail_mont_384,\@abi-omnipotent
.align	32
__redq_tail_mont_384:
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
.size	__redq_tail_mont_384,.-__redq_tail_mont_384

.globl	sgn0_pty_mont_384
.hidden	sgn0_pty_mont_384
.type	sgn0_pty_mont_384,\@function,3,"unwind"
.align	32
sgn0_pty_mont_384:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	sgn0_pty_mont_384\$1
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
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	$a_ptr, $n_ptr
	lea	0($r_ptr), $a_ptr
	mov	$b_org, $n0
	call	__mulq_by_1_mont_384

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
.size	sgn0_pty_mont_384,.-sgn0_pty_mont_384

.globl	sgn0_pty_mont_384x
.hidden	sgn0_pty_mont_384x
.type	sgn0_pty_mont_384x,\@function,3,"unwind"
.align	32
sgn0_pty_mont_384x:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	sgn0_pty_mont_384x\$1
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
	sub	\$8, %rsp
.cfi_adjust_cfa_offset	8
.cfi_end_prologue

	mov	$a_ptr, $n_ptr
	lea	48($r_ptr), $a_ptr	# sgn0(a->im)
	mov	$b_org, $n0
	call	__mulq_by_1_mont_384

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

	call	__mulq_by_1_mont_384

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
.size	sgn0_pty_mont_384x,.-sgn0_pty_mont_384x
___
} }

{ ########################################################## mulq_mont
my ($bi, $hi) = ("%rdi", "%rbp");

$code.=<<___;
.globl	mul_mont_384
.hidden	mul_mont_384
.type	mul_mont_384,\@function,5,"unwind"
.align	32
mul_mont_384:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	mul_mont_384\$1
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
	sub	\$8*3, %rsp
.cfi_adjust_cfa_offset	8*3
.cfi_end_prologue

	mov	8*0($b_org), %rax
	mov	8*0($a_ptr), @acc[6]
	mov	8*1($a_ptr), @acc[7]
	mov	8*2($a_ptr), @acc[4]
	mov	8*3($a_ptr), @acc[5]
	mov	$b_org, $b_ptr		# evacuate from %rdx
	mov	$n0,    8*0(%rsp)
	mov	$r_ptr, 8*1(%rsp)	# to __mulq_mont_384

	call	__mulq_mont_384

	mov	24(%rsp),%r15
.cfi_restore	%r15
	mov	32(%rsp),%r14
.cfi_restore	%r14
	mov	40(%rsp),%r13
.cfi_restore	%r13
	mov	48(%rsp),%r12
.cfi_restore	%r12
	mov	56(%rsp),%rbx
.cfi_restore	%rbx
	mov	64(%rsp),%rbp
.cfi_restore	%rbp
	lea	72(%rsp),%rsp
.cfi_adjust_cfa_offset	-72
.cfi_epilogue
	ret
.cfi_endproc
.size	mul_mont_384,.-mul_mont_384
___
{ my @acc=@acc;				# will be rotated locally

$code.=<<___;
.type	__mulq_mont_384,\@abi-omnipotent
.align	32
__mulq_mont_384:
	mov	%rax, $bi
	mulq	@acc[6]			# a[0]*b[0]
	mov	%rax, @acc[0]
	mov	$bi, %rax
	mov	%rdx, @acc[1]

	mulq	@acc[7]			# a[1]*b[0]
	add	%rax, @acc[1]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[2]

	mulq	@acc[4]			# a[2]*b[0]
	add	%rax, @acc[2]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[3]

	 mov	@acc[0], $hi
	 imulq	8(%rsp), @acc[0]

	mulq	@acc[5]			# a[3]*b[0]
	add	%rax, @acc[3]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[4]

	mulq	8*4($a_ptr)
	add	%rax, @acc[4]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[5]

	mulq	8*5($a_ptr)
	add	%rax, @acc[5]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	xor	@acc[7], @acc[7]
	mov	%rdx, @acc[6]
___
for (my $i=0; $i<6;) {
my $b_next = $i<5 ? 8*($i+1)."($b_ptr)" : @acc[1];
$code.=<<___;
	################################# reduction $i
	mulq	8*0($n_ptr)
	add	%rax, $hi		# guaranteed to be zero
	mov	@acc[0], %rax
	adc	%rdx, $hi

	mulq	8*1($n_ptr)
	add	%rax, @acc[1]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	add	$hi, @acc[1]
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
	add	$hi, @acc[3]
	adc	\$0, %rdx
	add	%rax, @acc[3]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*4($n_ptr)
	add	%rax, @acc[4]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	add	$hi, @acc[4]
	adc	\$0, %rdx
	mov	%rdx, $hi

	mulq	8*5($n_ptr)
	add	%rax, @acc[5]
	mov	$b_next, %rax
	adc	\$0, %rdx
	add	$hi, @acc[5]
	adc	%rdx, @acc[6]
	adc	\$0, @acc[7]
___
    push(@acc,shift(@acc));
$code.=<<___	if ($i++<5);
	################################# Multiply by b[$i]
	mov	%rax, $bi
	mulq	8*0($a_ptr)
	add	%rax, @acc[0]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[7]

	mulq	8*1($a_ptr)
	add	%rax, @acc[1]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[7], @acc[1]
	adc	\$0, %rdx
	mov	%rdx, @acc[7]

	mulq	8*2($a_ptr)
	add	%rax, @acc[2]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[7], @acc[2]
	adc	\$0, %rdx
	mov	%rdx, @acc[7]

	 mov	@acc[0], $hi
	 imulq	8(%rsp), @acc[0]

	mulq	8*3($a_ptr)
	add	%rax, @acc[3]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[7], @acc[3]
	adc	\$0, %rdx
	mov	%rdx, @acc[7]

	mulq	8*4($a_ptr)
	add	%rax, @acc[4]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[7], @acc[4]
	adc	\$0, %rdx
	mov	%rdx, @acc[7]

	mulq	8*5($a_ptr)
	add	@acc[7], @acc[5]
	adc	\$0, %rdx
	xor	@acc[7], @acc[7]
	add	%rax, @acc[5]
	mov	@acc[0], %rax
	adc	%rdx, @acc[6]
	adc	\$0, @acc[7]
___
}
$code.=<<___;
	#################################
	# Branch-less conditional acc[0:6] - modulus

	#mov	@acc[0], %rax
	mov	8*2(%rsp), $r_ptr	# restore $r_ptr
	sub	8*0($n_ptr), @acc[0]
	mov	@acc[1], %rdx
	sbb	8*1($n_ptr), @acc[1]
	mov	@acc[2], $b_ptr
	sbb	8*2($n_ptr), @acc[2]
	mov	@acc[3], $a_ptr
	sbb	8*3($n_ptr), @acc[3]
	mov	@acc[4], $hi
	sbb	8*4($n_ptr), @acc[4]
	mov	@acc[5], @acc[7]
	sbb	8*5($n_ptr), @acc[5]
	sbb	\$0, @acc[6]

	cmovc	%rax,    @acc[0]
	cmovc	%rdx,    @acc[1]
	cmovc	$b_ptr,  @acc[2]
	mov	@acc[0], 8*0($r_ptr)
	cmovc	$a_ptr,  @acc[3]
	mov	@acc[1], 8*1($r_ptr)
	cmovc	$hi,     @acc[4]
	mov	@acc[2], 8*2($r_ptr)
	cmovc	@acc[7], @acc[5]
	mov	@acc[3], 8*3($r_ptr)
	mov	@acc[4], 8*4($r_ptr)
	mov	@acc[5], 8*5($r_ptr)

	ret
.size	__mulq_mont_384,.-__mulq_mont_384
___
} }
$code.=<<___;
.globl	sqr_n_mul_mont_384
.hidden	sqr_n_mul_mont_384
.type	sqr_n_mul_mont_384,\@function,6,"unwind"
.align	32
sqr_n_mul_mont_384:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	sqr_n_mul_mont_384\$1
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
	sub	\$8*17, %rsp
.cfi_adjust_cfa_offset	8*17
.cfi_end_prologue

	mov	$n0,    8*0(%rsp)
	mov	$r_ptr, 8*1(%rsp)	# to __mulq_mont_384
	mov	$n_ptr, 8*2(%rsp)
	lea	8*4(%rsp), $r_ptr
	mov	%r9, 8*3(%rsp)		# 6th, multiplicand argument
	movq	(%r9), %xmm2		# prefetch b[0]

.Loop_sqr_384:
	movd	%edx, %xmm1		# loop counter

	call	__sqrq_384

	lea	0($r_ptr), $a_ptr
	mov	8*0(%rsp), %rcx		# n0 for mul_by_1
	mov	8*2(%rsp), $b_ptr	# n_ptr for mul_by_1
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

	movd	%xmm1, %edx
	lea	0($r_ptr), $a_ptr
	dec	%edx
	jnz	.Loop_sqr_384

	movq	%xmm2, %rax		# b[0]
	mov	$b_ptr, $n_ptr
	mov	8*3(%rsp), $b_ptr	# 6th, multiplicand argument

	#mov	8*0($b_ptr), %rax
	#mov	8*0($a_ptr), @acc[6]
	#mov	8*1($a_ptr), @acc[7]
	#mov	8*2($a_ptr), @acc[4]
	#mov	8*3($a_ptr), @acc[5]
	mov	@acc[0], @acc[4]
	mov	@acc[1], @acc[5]

	call	__mulq_mont_384

	lea	8*17(%rsp), %r8		# size optimization
	mov	8*17(%rsp), %r15
.cfi_restore	%r15
	mov	8*1(%r8), %r14
.cfi_restore	%r14
	mov	8*2(%r8), %r13
.cfi_restore	%r13
	mov	8*3(%r8), %r12
.cfi_restore	%r12
	mov	8*4(%r8), %rbx
.cfi_restore	%rbx
	mov	8*5(%r8), %rbp
.cfi_restore	%rbp
	lea	8*6(%r8), %rsp
.cfi_adjust_cfa_offset	-8*23
.cfi_epilogue
	ret
.cfi_endproc
.size	sqr_n_mul_mont_384,.-sqr_n_mul_mont_384

.globl	sqr_n_mul_mont_383
.hidden	sqr_n_mul_mont_383
.type	sqr_n_mul_mont_383,\@function,6,"unwind"
.align	32
sqr_n_mul_mont_383:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	sqr_n_mul_mont_383\$1
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
	sub	\$8*17, %rsp
.cfi_adjust_cfa_offset	8*17
.cfi_end_prologue

	mov	$n0, 8*0(%rsp)
	mov	$r_ptr, 8*1(%rsp)	# to __mulq_mont_384
	mov	$n_ptr, 8*2(%rsp)
	lea	8*4(%rsp), $r_ptr
	mov	%r9, 8*3(%rsp)		# 6th, multiplicand argument
	movq	(%r9), %xmm2		# prefetch b[0]

.Loop_sqr_383:
	movd	%edx, %xmm1		# loop counter

	call	__sqrq_384

	lea	0($r_ptr), $a_ptr
	mov	8*0(%rsp), %rcx		# n0 for mul_by_1
	mov	8*2(%rsp), $b_ptr	# n_ptr for mul_by_1
	call	__mulq_by_1_mont_384

	movd	%xmm1, %edx		# loop counter
        add     8*6($a_ptr), @acc[6]	# just accumulate upper half
        adc     8*7($a_ptr), @acc[7]
        adc     8*8($a_ptr), @acc[0]
        adc     8*9($a_ptr), @acc[1]
        adc     8*10($a_ptr), @acc[2]
        adc     8*11($a_ptr), @acc[3]
	lea	0($r_ptr), $a_ptr

	mov	@acc[6], 8*0($r_ptr)	# omitting full reduction gives ~5%
	mov	@acc[7], 8*1($r_ptr)	# in addition-chains
	mov	@acc[0], 8*2($r_ptr)
	mov	@acc[1], 8*3($r_ptr)
	mov	@acc[2], 8*4($r_ptr)
	mov	@acc[3], 8*5($r_ptr)

	dec	%edx
	jnz	.Loop_sqr_383

	movq	%xmm2, %rax		# b[0]
	mov	$b_ptr, $n_ptr
	mov	8*3(%rsp), $b_ptr	# 6th, multiplicand argument

	#movq	8*0($b_ptr), %rax
	#mov	8*0($a_ptr), @acc[6]
	#mov	8*1($a_ptr), @acc[7]
	#mov	8*2($a_ptr), @acc[4]
	#mov	8*3($a_ptr), @acc[5]
	mov	@acc[0], @acc[4]
	mov	@acc[1], @acc[5]

	call	__mulq_mont_384		# formally one can omit full reduction
					# even after multiplication...
	lea	8*17(%rsp), %r8		# size optimization
	mov	8*17(%rsp), %r15
.cfi_restore	%r15
	mov	8*1(%r8), %r14
.cfi_restore	%r14
	mov	8*2(%r8), %r13
.cfi_restore	%r13
	mov	8*3(%r8), %r12
.cfi_restore	%r12
	mov	8*4(%r8), %rbx
.cfi_restore	%rbx
	mov	8*5(%r8), %rbp
.cfi_restore	%rbp
	lea	8*6(%r8), %rsp
.cfi_adjust_cfa_offset	-8*23
.cfi_epilogue
	ret
.cfi_endproc
.size	sqr_n_mul_mont_383,.-sqr_n_mul_mont_383
___
{ my @acc=@acc;				# will be rotated locally
  my $bi = "%rbp";

$code.=<<___;
.type	__mulq_mont_383_nonred,\@abi-omnipotent
.align	32
__mulq_mont_383_nonred:
	mov	%rax, $bi
	mulq	@acc[6]			# a[0]*b[0]
	mov	%rax, @acc[0]
	mov	$bi, %rax
	mov	%rdx, @acc[1]

	mulq	@acc[7]			# a[1]*b[0]
	add	%rax, @acc[1]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[2]

	mulq	@acc[4]			# a[2]*b[0]
	add	%rax, @acc[2]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[3]

	 mov	@acc[0], @acc[7]
	 imulq	8(%rsp), @acc[0]

	mulq	@acc[5]			# a[3]*b[0]
	add	%rax, @acc[3]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[4]

	mulq	8*4($a_ptr)
	add	%rax, @acc[4]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[5]

	mulq	8*5($a_ptr)
	add	%rax, @acc[5]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[6]
___
for (my $i=0; $i<6;) {
my $b_next = $i<5 ? 8*($i+1)."($b_ptr)" : @acc[1];
$code.=<<___;
	################################# reduction $i
	mulq	8*0($n_ptr)
	add	%rax, @acc[7]		# guaranteed to be zero
	mov	@acc[0], %rax
	adc	%rdx, @acc[7]

	mulq	8*1($n_ptr)
	add	%rax, @acc[1]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	add	@acc[7], @acc[1]
	adc	\$0, %rdx
	mov	%rdx, @acc[7]

	mulq	8*2($n_ptr)
	add	%rax, @acc[2]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	add	@acc[7], @acc[2]
	adc	\$0, %rdx
	mov	%rdx, @acc[7]

	mulq	8*3($n_ptr)
	add	@acc[7], @acc[3]
	adc	\$0, %rdx
	add	%rax, @acc[3]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[7]

	mulq	8*4($n_ptr)
	add	%rax, @acc[4]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	add	@acc[7], @acc[4]
	adc	\$0, %rdx
	mov	%rdx, @acc[7]

	mulq	8*5($n_ptr)
	add	%rax, @acc[5]
	mov	$b_next, %rax
	adc	\$0, %rdx
	add	@acc[7], @acc[5]
	adc	%rdx, @acc[6]
___
    push(@acc,shift(@acc));
$code.=<<___	if ($i++<5);
	################################# Multiply by b[$i]
	mov	%rax, $bi
	mulq	8*0($a_ptr)
	add	%rax, @acc[0]
	mov	$bi, %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[6]

	mulq	8*1($a_ptr)
	add	%rax, @acc[1]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[6], @acc[1]
	adc	\$0, %rdx
	mov	%rdx, @acc[6]

	mulq	8*2($a_ptr)
	add	%rax, @acc[2]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[6], @acc[2]
	adc	\$0, %rdx
	mov	%rdx, @acc[6]

	 mov	@acc[0], @acc[7]
	 imulq	8(%rsp), @acc[0]

	mulq	8*3($a_ptr)
	add	%rax, @acc[3]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[6], @acc[3]
	adc	\$0, %rdx
	mov	%rdx, @acc[6]

	mulq	8*4($a_ptr)
	add	%rax, @acc[4]
	mov	$bi, %rax
	adc	\$0, %rdx
	add	@acc[6], @acc[4]
	adc	\$0, %rdx
	mov	%rdx, @acc[6]

	mulq	8*5($a_ptr)
	add	@acc[6], @acc[5]
	adc	\$0, %rdx
	add	%rax, @acc[5]
	mov	@acc[0], %rax
	adc	\$0, %rdx
	mov	%rdx, @acc[6]
___
}
$code.=<<___;
	ret
.size	__mulq_mont_383_nonred,.-__mulq_mont_383_nonred
___
}
{ my $frame = 4*8 +	# place for argument off-load +
	      2*384/8 +	# place for 2 384-bit temporary vectors
	      8;	# align
my @acc = (@acc,"%rax","%rdx","%rbx","%rbp");

# omitting 3 reductions gives 8-11% better performance in add-chains
$code.=<<___;
.globl	sqr_mont_382x
.hidden	sqr_mont_382x
.type	sqr_mont_382x,\@function,4,"unwind"
.align	32
sqr_mont_382x:
.cfi_startproc
#ifdef __BLST_PORTABLE__
	testl	\$1, __blst_platform_cap(%rip)
	jnz	sqr_mont_382x\$1
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

	mov	$n_ptr, 8*0(%rsp)	# n0
	mov	$b_org, $n_ptr		# n_ptr
	mov	$a_ptr, 8*2(%rsp)
	mov	$r_ptr, 8*3(%rsp)

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
	#mov	8*2(%rsp), $a_ptr	# a->re
	lea	48($a_ptr), $b_ptr	# a->im

	mov	48($a_ptr), %rax	# a->im
	mov	8*0($a_ptr), @acc[6]	# a->re
	mov	8*1($a_ptr), @acc[7]
	mov	8*2($a_ptr), @acc[4]
	mov	8*3($a_ptr), @acc[5]

	mov	8*3(%rsp), $r_ptr
	call	__mulq_mont_383_nonred
___
{
my @acc = map("%r$_",14,15,8..11,	# output from __mulq_mont_384
                     12,13,"ax","bx","bp","si");
$code.=<<___;
	add	@acc[0], @acc[0]	# add with itself
	adc	@acc[1], @acc[1]
	adc	@acc[2], @acc[2]
	adc	@acc[3], @acc[3]
	adc	@acc[4], @acc[4]
	adc	@acc[5], @acc[5]

	mov	@acc[0],  8*6($r_ptr)	# ret->im
	mov	@acc[1],  8*7($r_ptr)
	mov	@acc[2],  8*8($r_ptr)
	mov	@acc[3],  8*9($r_ptr)
	mov	@acc[4],  8*10($r_ptr)
	mov	@acc[5],  8*11($r_ptr)
___
}
$code.=<<___;
	################################# mul_mont_384(ret->re, t0, t1, mod, n0);
	lea	32(%rsp), $a_ptr	# t0
	lea	32+8*6(%rsp), $b_ptr	# t1

	mov	32+8*6(%rsp), %rax	# t1[0]
	mov	32+8*0(%rsp), @acc[6]	# t0[0..3]
	mov	32+8*1(%rsp), @acc[7]
	mov	32+8*2(%rsp), @acc[4]
	mov	32+8*3(%rsp), @acc[5]

	call	__mulq_mont_383_nonred
___
{
my @acc = map("%r$_",14,15,8..11,	# output from __mulq_mont_384
                     12,13,"ax","bx","bp","si");
$code.=<<___;
	mov	32+8*12(%rsp), @acc[11]	# account for sign from a->re - a->im
	mov	32+8*0(%rsp), @acc[6]
	mov	32+8*1(%rsp), @acc[7]
	and	@acc[11], @acc[6]
	mov	32+8*2(%rsp), @acc[8]
	and	@acc[11], @acc[7]
	mov	32+8*3(%rsp), @acc[9]
	and	@acc[11], @acc[8]
	mov	32+8*4(%rsp), @acc[10]
	and	@acc[11], @acc[9]
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

	mov	@acc[0],  8*0($r_ptr)	# ret->re
	mov	@acc[1],  8*1($r_ptr)
	mov	@acc[2],  8*2($r_ptr)
	mov	@acc[3],  8*3($r_ptr)
	mov	@acc[4],  8*4($r_ptr)
	mov	@acc[5],  8*5($r_ptr)
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
.size	sqr_mont_382x,.-sqr_mont_382x
___
}

print $code;
close STDOUT;
