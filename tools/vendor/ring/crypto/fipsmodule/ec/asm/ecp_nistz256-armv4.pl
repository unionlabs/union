#! /usr/bin/env perl
# Copyright 2015-2018 The OpenSSL Project Authors. All Rights Reserved.
#
# Licensed under the OpenSSL license (the "License").  You may not use
# this file except in compliance with the License.  You can obtain a copy
# in the file LICENSE in the source distribution or at
# https://www.openssl.org/source/license.html


# ====================================================================
# Written by Andy Polyakov <appro@openssl.org> for the OpenSSL
# project. The module is, however, dual licensed under OpenSSL and
# CRYPTOGAMS licenses depending on where you obtain it. For further
# details see http://www.openssl.org/~appro/cryptogams/.
# ====================================================================
#
# ECP_NISTZ256 module for ARMv4.
#
# October 2014.
#
# Original ECP_NISTZ256 submission targeting x86_64 is detailed in
# http://eprint.iacr.org/2013/816. In the process of adaptation
# original .c module was made 32-bit savvy in order to make this
# implementation possible.
#
#			with/without -DECP_NISTZ256_ASM
# Cortex-A8		+53-170%
# Cortex-A9		+76-205%
# Cortex-A15		+100-316%
# Snapdragon S4		+66-187%
#
# Ranges denote minimum and maximum improvement coefficients depending
# on benchmark. Lower coefficients are for ECDSA sign, server-side
# operation. Keep in mind that +200% means 3x improvement.

$flavour = shift;
if ($flavour=~/\w[\w\-]*\.\w+$/) { $output=$flavour; undef $flavour; }
else { while (($output=shift) && ($output!~/\w[\w\-]*\.\w+$/)) {} }

if ($flavour && $flavour ne "void") {
    $0 =~ m/(.*[\/\\])[^\/\\]+$/; $dir=$1;
    ( $xlate="${dir}arm-xlate.pl" and -f $xlate ) or
    ( $xlate="${dir}../../../perlasm/arm-xlate.pl" and -f $xlate) or
    die "can't locate arm-xlate.pl";

    open STDOUT,"| \"$^X\" $xlate $flavour $output";
} else {
    open STDOUT,">$output";
}

$code.=<<___;
#include <GFp/arm_arch.h>

.text
#if defined(__thumb2__)
.syntax	unified
.thumb
#else
.code	32
#endif

.asciz	"ECP_NISTZ256 for ARMv4, CRYPTOGAMS by <appro\@openssl.org>"
.align	6
___

########################################################################
# common register layout, note that $t2 is link register, so that if
# internal subroutine uses $t2, then it has to offload lr...

($r_ptr,$a_ptr,$b_ptr,$ff,$a0,$a1,$a2,$a3,$a4,$a5,$a6,$a7,$t1,$t2)=
		map("r$_",(0..12,14));
($t0,$t3)=($ff,$a_ptr);

$code.=<<___;
.type	__ecp_nistz256_mul_by_2,%function
.align	4
__ecp_nistz256_mul_by_2:
	ldr	$a0,[$a_ptr,#0]
	ldr	$a1,[$a_ptr,#4]
	ldr	$a2,[$a_ptr,#8]
	adds	$a0,$a0,$a0		@ a[0:7]+=a[0:7], i.e. add with itself
	ldr	$a3,[$a_ptr,#12]
	adcs	$a1,$a1,$a1
	ldr	$a4,[$a_ptr,#16]
	adcs	$a2,$a2,$a2
	ldr	$a5,[$a_ptr,#20]
	adcs	$a3,$a3,$a3
	ldr	$a6,[$a_ptr,#24]
	adcs	$a4,$a4,$a4
	ldr	$a7,[$a_ptr,#28]
	adcs	$a5,$a5,$a5
	adcs	$a6,$a6,$a6
	mov	$ff,#0
	adcs	$a7,$a7,$a7
	adc	$ff,$ff,#0

	b	.Lreduce_by_sub
.size	__ecp_nistz256_mul_by_2,.-__ecp_nistz256_mul_by_2

@ void	GFp_nistz256_add(BN_ULONG r0[8],const BN_ULONG r1[8],
@					const BN_ULONG r2[8]);
.globl	GFp_nistz256_add
.type	GFp_nistz256_add,%function
.align	4
GFp_nistz256_add:
	stmdb	sp!,{r4-r12,lr}
	bl	__ecp_nistz256_add
#if __ARM_ARCH__>=5 || !defined(__thumb__)
	ldmia	sp!,{r4-r12,pc}
#else
	ldmia	sp!,{r4-r12,lr}
	bx	lr			@ interoperable with Thumb ISA:-)
#endif
.size	GFp_nistz256_add,.-GFp_nistz256_add

.type	__ecp_nistz256_add,%function
.align	4
__ecp_nistz256_add:
	str	lr,[sp,#-4]!		@ push lr

	ldr	$a0,[$a_ptr,#0]
	ldr	$a1,[$a_ptr,#4]
	ldr	$a2,[$a_ptr,#8]
	ldr	$a3,[$a_ptr,#12]
	ldr	$a4,[$a_ptr,#16]
	 ldr	$t0,[$b_ptr,#0]
	ldr	$a5,[$a_ptr,#20]
	 ldr	$t1,[$b_ptr,#4]
	ldr	$a6,[$a_ptr,#24]
	 ldr	$t2,[$b_ptr,#8]
	ldr	$a7,[$a_ptr,#28]
	 ldr	$t3,[$b_ptr,#12]
	adds	$a0,$a0,$t0
	 ldr	$t0,[$b_ptr,#16]
	adcs	$a1,$a1,$t1
	 ldr	$t1,[$b_ptr,#20]
	adcs	$a2,$a2,$t2
	 ldr	$t2,[$b_ptr,#24]
	adcs	$a3,$a3,$t3
	 ldr	$t3,[$b_ptr,#28]
	adcs	$a4,$a4,$t0
	adcs	$a5,$a5,$t1
	adcs	$a6,$a6,$t2
	mov	$ff,#0
	adcs	$a7,$a7,$t3
	adc	$ff,$ff,#0
	ldr	lr,[sp],#4		@ pop lr

.Lreduce_by_sub:

	@ if a+b >= modulus, subtract modulus.
	@
	@ But since comparison implies subtraction, we subtract
	@ modulus and then add it back if subtraction borrowed.

	subs	$a0,$a0,#-1
	sbcs	$a1,$a1,#-1
	sbcs	$a2,$a2,#-1
	sbcs	$a3,$a3,#0
	sbcs	$a4,$a4,#0
	sbcs	$a5,$a5,#0
	sbcs	$a6,$a6,#1
	sbcs	$a7,$a7,#-1
	sbc	$ff,$ff,#0

	@ Note that because mod has special form, i.e. consists of
	@ 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	@ using value of borrow as a whole or extracting single bit.
	@ Follow $ff register...

	adds	$a0,$a0,$ff		@ add synthesized modulus
	adcs	$a1,$a1,$ff
	str	$a0,[$r_ptr,#0]
	adcs	$a2,$a2,$ff
	str	$a1,[$r_ptr,#4]
	adcs	$a3,$a3,#0
	str	$a2,[$r_ptr,#8]
	adcs	$a4,$a4,#0
	str	$a3,[$r_ptr,#12]
	adcs	$a5,$a5,#0
	str	$a4,[$r_ptr,#16]
	adcs	$a6,$a6,$ff,lsr#31
	str	$a5,[$r_ptr,#20]
	adcs	$a7,$a7,$ff
	str	$a6,[$r_ptr,#24]
	str	$a7,[$r_ptr,#28]

	mov	pc,lr
.size	__ecp_nistz256_add,.-__ecp_nistz256_add

.type	__ecp_nistz256_mul_by_3,%function
.align	4
__ecp_nistz256_mul_by_3:
	str	lr,[sp,#-4]!		@ push lr

	@ As multiplication by 3 is performed as 2*n+n, below are inline
	@ copies of __ecp_nistz256_mul_by_2 and __ecp_nistz256_add, see
	@ corresponding subroutines for details.

	ldr	$a0,[$a_ptr,#0]
	ldr	$a1,[$a_ptr,#4]
	ldr	$a2,[$a_ptr,#8]
	adds	$a0,$a0,$a0		@ a[0:7]+=a[0:7]
	ldr	$a3,[$a_ptr,#12]
	adcs	$a1,$a1,$a1
	ldr	$a4,[$a_ptr,#16]
	adcs	$a2,$a2,$a2
	ldr	$a5,[$a_ptr,#20]
	adcs	$a3,$a3,$a3
	ldr	$a6,[$a_ptr,#24]
	adcs	$a4,$a4,$a4
	ldr	$a7,[$a_ptr,#28]
	adcs	$a5,$a5,$a5
	adcs	$a6,$a6,$a6
	mov	$ff,#0
	adcs	$a7,$a7,$a7
	adc	$ff,$ff,#0

	subs	$a0,$a0,#-1		@ .Lreduce_by_sub but without stores
	sbcs	$a1,$a1,#-1
	sbcs	$a2,$a2,#-1
	sbcs	$a3,$a3,#0
	sbcs	$a4,$a4,#0
	sbcs	$a5,$a5,#0
	sbcs	$a6,$a6,#1
	sbcs	$a7,$a7,#-1
	sbc	$ff,$ff,#0

	adds	$a0,$a0,$ff		@ add synthesized modulus
	adcs	$a1,$a1,$ff
	adcs	$a2,$a2,$ff
	adcs	$a3,$a3,#0
	adcs	$a4,$a4,#0
	 ldr	$b_ptr,[$a_ptr,#0]
	adcs	$a5,$a5,#0
	 ldr	$t1,[$a_ptr,#4]
	adcs	$a6,$a6,$ff,lsr#31
	 ldr	$t2,[$a_ptr,#8]
	adc	$a7,$a7,$ff

	ldr	$t0,[$a_ptr,#12]
	adds	$a0,$a0,$b_ptr		@ 2*a[0:7]+=a[0:7]
	ldr	$b_ptr,[$a_ptr,#16]
	adcs	$a1,$a1,$t1
	ldr	$t1,[$a_ptr,#20]
	adcs	$a2,$a2,$t2
	ldr	$t2,[$a_ptr,#24]
	adcs	$a3,$a3,$t0
	ldr	$t3,[$a_ptr,#28]
	adcs	$a4,$a4,$b_ptr
	adcs	$a5,$a5,$t1
	adcs	$a6,$a6,$t2
	mov	$ff,#0
	adcs	$a7,$a7,$t3
	adc	$ff,$ff,#0
	ldr	lr,[sp],#4		@ pop lr

	b	.Lreduce_by_sub
.size	__ecp_nistz256_mul_by_3,.-__ecp_nistz256_mul_by_3

.type	__ecp_nistz256_div_by_2,%function
.align	4
__ecp_nistz256_div_by_2:
	@ ret = (a is odd ? a+mod : a) >> 1

	ldr	$a0,[$a_ptr,#0]
	ldr	$a1,[$a_ptr,#4]
	ldr	$a2,[$a_ptr,#8]
	mov	$ff,$a0,lsl#31		@ place least significant bit to most
					@ significant position, now arithmetic
					@ right shift by 31 will produce -1 or
					@ 0, while logical right shift 1 or 0,
					@ this is how modulus is conditionally
					@ synthesized in this case...
	ldr	$a3,[$a_ptr,#12]
	adds	$a0,$a0,$ff,asr#31
	ldr	$a4,[$a_ptr,#16]
	adcs	$a1,$a1,$ff,asr#31
	ldr	$a5,[$a_ptr,#20]
	adcs	$a2,$a2,$ff,asr#31
	ldr	$a6,[$a_ptr,#24]
	adcs	$a3,$a3,#0
	ldr	$a7,[$a_ptr,#28]
	adcs	$a4,$a4,#0
	 mov	$a0,$a0,lsr#1		@ a[0:7]>>=1, we can start early
					@ because it doesn't affect flags
	adcs	$a5,$a5,#0
	 orr	$a0,$a0,$a1,lsl#31
	adcs	$a6,$a6,$ff,lsr#31
	mov	$b_ptr,#0
	adcs	$a7,$a7,$ff,asr#31
	 mov	$a1,$a1,lsr#1
	adc	$b_ptr,$b_ptr,#0	@ top-most carry bit from addition

	orr	$a1,$a1,$a2,lsl#31
	mov	$a2,$a2,lsr#1
	str	$a0,[$r_ptr,#0]
	orr	$a2,$a2,$a3,lsl#31
	mov	$a3,$a3,lsr#1
	str	$a1,[$r_ptr,#4]
	orr	$a3,$a3,$a4,lsl#31
	mov	$a4,$a4,lsr#1
	str	$a2,[$r_ptr,#8]
	orr	$a4,$a4,$a5,lsl#31
	mov	$a5,$a5,lsr#1
	str	$a3,[$r_ptr,#12]
	orr	$a5,$a5,$a6,lsl#31
	mov	$a6,$a6,lsr#1
	str	$a4,[$r_ptr,#16]
	orr	$a6,$a6,$a7,lsl#31
	mov	$a7,$a7,lsr#1
	str	$a5,[$r_ptr,#20]
	orr	$a7,$a7,$b_ptr,lsl#31	@ don't forget the top-most carry bit
	str	$a6,[$r_ptr,#24]
	str	$a7,[$r_ptr,#28]

	mov	pc,lr
.size	__ecp_nistz256_div_by_2,.-__ecp_nistz256_div_by_2

.type	__ecp_nistz256_sub,%function
.align	4
__ecp_nistz256_sub:
	str	lr,[sp,#-4]!		@ push lr

	ldr	$a0,[$a_ptr,#0]
	ldr	$a1,[$a_ptr,#4]
	ldr	$a2,[$a_ptr,#8]
	ldr	$a3,[$a_ptr,#12]
	ldr	$a4,[$a_ptr,#16]
	 ldr	$t0,[$b_ptr,#0]
	ldr	$a5,[$a_ptr,#20]
	 ldr	$t1,[$b_ptr,#4]
	ldr	$a6,[$a_ptr,#24]
	 ldr	$t2,[$b_ptr,#8]
	ldr	$a7,[$a_ptr,#28]
	 ldr	$t3,[$b_ptr,#12]
	subs	$a0,$a0,$t0
	 ldr	$t0,[$b_ptr,#16]
	sbcs	$a1,$a1,$t1
	 ldr	$t1,[$b_ptr,#20]
	sbcs	$a2,$a2,$t2
	 ldr	$t2,[$b_ptr,#24]
	sbcs	$a3,$a3,$t3
	 ldr	$t3,[$b_ptr,#28]
	sbcs	$a4,$a4,$t0
	sbcs	$a5,$a5,$t1
	sbcs	$a6,$a6,$t2
	sbcs	$a7,$a7,$t3
	sbc	$ff,$ff,$ff		@ broadcast borrow bit
	ldr	lr,[sp],#4		@ pop lr

.Lreduce_by_add:

	@ if a-b borrows, add modulus.
	@
	@ Note that because mod has special form, i.e. consists of
	@ 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	@ broadcasting borrow bit to a register, $ff, and using it as
	@ a whole or extracting single bit.

	adds	$a0,$a0,$ff		@ add synthesized modulus
	adcs	$a1,$a1,$ff
	str	$a0,[$r_ptr,#0]
	adcs	$a2,$a2,$ff
	str	$a1,[$r_ptr,#4]
	adcs	$a3,$a3,#0
	str	$a2,[$r_ptr,#8]
	adcs	$a4,$a4,#0
	str	$a3,[$r_ptr,#12]
	adcs	$a5,$a5,#0
	str	$a4,[$r_ptr,#16]
	adcs	$a6,$a6,$ff,lsr#31
	str	$a5,[$r_ptr,#20]
	adcs	$a7,$a7,$ff
	str	$a6,[$r_ptr,#24]
	str	$a7,[$r_ptr,#28]

	mov	pc,lr
.size	__ecp_nistz256_sub,.-__ecp_nistz256_sub

@ void	GFp_nistz256_neg(BN_ULONG r0[8],const BN_ULONG r1[8]);
.globl	GFp_nistz256_neg
.type	GFp_nistz256_neg,%function
.align	4
GFp_nistz256_neg:
	stmdb	sp!,{r4-r12,lr}
	bl	__ecp_nistz256_neg
#if __ARM_ARCH__>=5 || !defined(__thumb__)
	ldmia	sp!,{r4-r12,pc}
#else
	ldmia	sp!,{r4-r12,lr}
	bx	lr			@ interoperable with Thumb ISA:-)
#endif
.size	GFp_nistz256_neg,.-GFp_nistz256_neg

.type	__ecp_nistz256_neg,%function
.align	4
__ecp_nistz256_neg:
	ldr	$a0,[$a_ptr,#0]
	eor	$ff,$ff,$ff
	ldr	$a1,[$a_ptr,#4]
	ldr	$a2,[$a_ptr,#8]
	subs	$a0,$ff,$a0
	ldr	$a3,[$a_ptr,#12]
	sbcs	$a1,$ff,$a1
	ldr	$a4,[$a_ptr,#16]
	sbcs	$a2,$ff,$a2
	ldr	$a5,[$a_ptr,#20]
	sbcs	$a3,$ff,$a3
	ldr	$a6,[$a_ptr,#24]
	sbcs	$a4,$ff,$a4
	ldr	$a7,[$a_ptr,#28]
	sbcs	$a5,$ff,$a5
	sbcs	$a6,$ff,$a6
	sbcs	$a7,$ff,$a7
	sbc	$ff,$ff,$ff

	b	.Lreduce_by_add
.size	__ecp_nistz256_neg,.-__ecp_nistz256_neg
___
{
my @acc=map("r$_",(3..11));
my ($t0,$t1,$bj,$t2,$t3)=map("r$_",(0,1,2,12,14));

$code.=<<___;
@ void	GFp_nistz256_mul_mont(BN_ULONG r0[8],const BN_ULONG r1[8],
@					     const BN_ULONG r2[8]);
.globl	GFp_nistz256_mul_mont
.type	GFp_nistz256_mul_mont,%function
.align	4
GFp_nistz256_mul_mont:
	stmdb	sp!,{r4-r12,lr}
	bl	__ecp_nistz256_mul_mont
#if __ARM_ARCH__>=5 || !defined(__thumb__)
	ldmia	sp!,{r4-r12,pc}
#else
	ldmia	sp!,{r4-r12,lr}
	bx	lr			@ interoperable with Thumb ISA:-)
#endif
.size	GFp_nistz256_mul_mont,.-GFp_nistz256_mul_mont

.type	__ecp_nistz256_mul_mont,%function
.align	4
__ecp_nistz256_mul_mont:
	stmdb	sp!,{r0-r2,lr}			@ make a copy of arguments too

	ldr	$bj,[$b_ptr,#0]			@ b[0]
	ldmia	$a_ptr,{@acc[1]-@acc[8]}

	umull	@acc[0],$t3,@acc[1],$bj		@ r[0]=a[0]*b[0]
	stmdb	sp!,{$acc[1]-@acc[8]}		@ copy a[0-7] to stack, so
						@ that it can be addressed
						@ without spending register
						@ on address
	umull	@acc[1],$t0,@acc[2],$bj		@ r[1]=a[1]*b[0]
	umull	@acc[2],$t1,@acc[3],$bj
	adds	@acc[1],@acc[1],$t3		@ accumulate high part of mult
	umull	@acc[3],$t2,@acc[4],$bj
	adcs	@acc[2],@acc[2],$t0
	umull	@acc[4],$t3,@acc[5],$bj
	adcs	@acc[3],@acc[3],$t1
	umull	@acc[5],$t0,@acc[6],$bj
	adcs	@acc[4],@acc[4],$t2
	umull	@acc[6],$t1,@acc[7],$bj
	adcs	@acc[5],@acc[5],$t3
	umull	@acc[7],$t2,@acc[8],$bj
	adcs	@acc[6],@acc[6],$t0
	adcs	@acc[7],@acc[7],$t1
	eor	$t3,$t3,$t3			@ first overflow bit is zero
	adc	@acc[8],$t2,#0
___
for(my $i=1;$i<8;$i++) {
my $t4=@acc[0];

	# Reduction iteration is normally performed by accumulating
	# result of multiplication of modulus by "magic" digit [and
	# omitting least significant word, which is guaranteed to
	# be 0], but thanks to special form of modulus and "magic"
	# digit being equal to least significant word, it can be
	# performed with additions and subtractions alone. Indeed:
	#
	#        ffff.0001.0000.0000.0000.ffff.ffff.ffff
	# *                                         abcd
	# + xxxx.xxxx.xxxx.xxxx.xxxx.xxxx.xxxx.xxxx.abcd
	#
	# Now observing that ff..ff*x = (2^n-1)*x = 2^n*x-x, we
	# rewrite above as:
	#
	#   xxxx.xxxx.xxxx.xxxx.xxxx.xxxx.xxxx.xxxx.abcd
	# + abcd.0000.abcd.0000.0000.abcd.0000.0000.0000
	# -      abcd.0000.0000.0000.0000.0000.0000.abcd
	#
	# or marking redundant operations:
	#
	#   xxxx.xxxx.xxxx.xxxx.xxxx.xxxx.xxxx.xxxx.----
	# + abcd.0000.abcd.0000.0000.abcd.----.----.----
	# -      abcd.----.----.----.----.----.----.----

$code.=<<___;
	@ multiplication-less reduction $i
	adds	@acc[3],@acc[3],@acc[0]		@ r[3]+=r[0]
	 ldr	$bj,[sp,#40]			@ restore b_ptr
	adcs	@acc[4],@acc[4],#0		@ r[4]+=0
	adcs	@acc[5],@acc[5],#0		@ r[5]+=0
	adcs	@acc[6],@acc[6],@acc[0]		@ r[6]+=r[0]
	 ldr	$t1,[sp,#0]			@ load a[0]
	adcs	@acc[7],@acc[7],#0		@ r[7]+=0
	 ldr	$bj,[$bj,#4*$i]			@ load b[i]
	adcs	@acc[8],@acc[8],@acc[0]		@ r[8]+=r[0]
	 eor	$t0,$t0,$t0
	adc	$t3,$t3,#0			@ overflow bit
	subs	@acc[7],@acc[7],@acc[0]		@ r[7]-=r[0]
	 ldr	$t2,[sp,#4]			@ a[1]
	sbcs	@acc[8],@acc[8],#0		@ r[8]-=0
	 umlal	@acc[1],$t0,$t1,$bj		@ "r[0]"+=a[0]*b[i]
	 eor	$t1,$t1,$t1
	sbc	@acc[0],$t3,#0			@ overflow bit, keep in mind
						@ that netto result is
						@ addition of a value which
						@ makes underflow impossible

	ldr	$t3,[sp,#8]			@ a[2]
	umlal	@acc[2],$t1,$t2,$bj		@ "r[1]"+=a[1]*b[i]
	 str	@acc[0],[sp,#36]		@ temporarily offload overflow
	eor	$t2,$t2,$t2
	ldr	$t4,[sp,#12]			@ a[3], $t4 is alias @acc[0]
	umlal	@acc[3],$t2,$t3,$bj		@ "r[2]"+=a[2]*b[i]
	eor	$t3,$t3,$t3
	adds	@acc[2],@acc[2],$t0		@ accumulate high part of mult
	ldr	$t0,[sp,#16]			@ a[4]
	umlal	@acc[4],$t3,$t4,$bj		@ "r[3]"+=a[3]*b[i]
	eor	$t4,$t4,$t4
	adcs	@acc[3],@acc[3],$t1
	ldr	$t1,[sp,#20]			@ a[5]
	umlal	@acc[5],$t4,$t0,$bj		@ "r[4]"+=a[4]*b[i]
	eor	$t0,$t0,$t0
	adcs	@acc[4],@acc[4],$t2
	ldr	$t2,[sp,#24]			@ a[6]
	umlal	@acc[6],$t0,$t1,$bj		@ "r[5]"+=a[5]*b[i]
	eor	$t1,$t1,$t1
	adcs	@acc[5],@acc[5],$t3
	ldr	$t3,[sp,#28]			@ a[7]
	umlal	@acc[7],$t1,$t2,$bj		@ "r[6]"+=a[6]*b[i]
	eor	$t2,$t2,$t2
	adcs	@acc[6],@acc[6],$t4
	 ldr	@acc[0],[sp,#36]		@ restore overflow bit
	umlal	@acc[8],$t2,$t3,$bj		@ "r[7]"+=a[7]*b[i]
	eor	$t3,$t3,$t3
	adcs	@acc[7],@acc[7],$t0
	adcs	@acc[8],@acc[8],$t1
	adcs	@acc[0],$acc[0],$t2
	adc	$t3,$t3,#0			@ new overflow bit
___
	push(@acc,shift(@acc));			# rotate registers, so that
						# "r[i]" becomes r[i]
}
$code.=<<___;
	@ last multiplication-less reduction
	adds	@acc[3],@acc[3],@acc[0]
	ldr	$r_ptr,[sp,#32]			@ restore r_ptr
	adcs	@acc[4],@acc[4],#0
	adcs	@acc[5],@acc[5],#0
	adcs	@acc[6],@acc[6],@acc[0]
	adcs	@acc[7],@acc[7],#0
	adcs	@acc[8],@acc[8],@acc[0]
	adc	$t3,$t3,#0
	subs	@acc[7],@acc[7],@acc[0]
	sbcs	@acc[8],@acc[8],#0
	sbc	@acc[0],$t3,#0			@ overflow bit

	@ Final step is "if result > mod, subtract mod", but we do it
	@ "other way around", namely subtract modulus from result
	@ and if it borrowed, add modulus back.

	adds	@acc[1],@acc[1],#1		@ subs	@acc[1],@acc[1],#-1
	adcs	@acc[2],@acc[2],#0		@ sbcs	@acc[2],@acc[2],#-1
	adcs	@acc[3],@acc[3],#0		@ sbcs	@acc[3],@acc[3],#-1
	sbcs	@acc[4],@acc[4],#0
	sbcs	@acc[5],@acc[5],#0
	sbcs	@acc[6],@acc[6],#0
	sbcs	@acc[7],@acc[7],#1
	adcs	@acc[8],@acc[8],#0		@ sbcs	@acc[8],@acc[8],#-1
	ldr	lr,[sp,#44]			@ restore lr
	sbc	@acc[0],@acc[0],#0		@ broadcast borrow bit
	add	sp,sp,#48

	@ Note that because mod has special form, i.e. consists of
	@ 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	@ broadcasting borrow bit to a register, @acc[0], and using it as
	@ a whole or extracting single bit.

	adds	@acc[1],@acc[1],@acc[0]		@ add modulus or zero
	adcs	@acc[2],@acc[2],@acc[0]
	str	@acc[1],[$r_ptr,#0]
	adcs	@acc[3],@acc[3],@acc[0]
	str	@acc[2],[$r_ptr,#4]
	adcs	@acc[4],@acc[4],#0
	str	@acc[3],[$r_ptr,#8]
	adcs	@acc[5],@acc[5],#0
	str	@acc[4],[$r_ptr,#12]
	adcs	@acc[6],@acc[6],#0
	str	@acc[5],[$r_ptr,#16]
	adcs	@acc[7],@acc[7],@acc[0],lsr#31
	str	@acc[6],[$r_ptr,#20]
	adc	@acc[8],@acc[8],@acc[0]
	str	@acc[7],[$r_ptr,#24]
	str	@acc[8],[$r_ptr,#28]

	mov	pc,lr
.size	__ecp_nistz256_mul_mont,.-__ecp_nistz256_mul_mont
___
}

{{{
########################################################################
# Below $aN assignment matches order in which 256-bit result appears in
# register bank at return from __ecp_nistz256_mul_mont, so that we can
# skip over reloading it from memory. This means that below functions
# use custom calling sequence accepting 256-bit input in registers,
# output pointer in r0, $r_ptr, and optional pointer in r2, $b_ptr.
#
# See their "normal" counterparts for insights on calculations.

my ($a0,$a1,$a2,$a3,$a4,$a5,$a6,$a7,
    $t0,$t1,$t2,$t3)=map("r$_",(11,3..10,12,14,1));
my $ff=$b_ptr;

$code.=<<___;
.type	__ecp_nistz256_sub_from,%function
.align	5
__ecp_nistz256_sub_from:
	str	lr,[sp,#-4]!		@ push lr

	 ldr	$t0,[$b_ptr,#0]
	 ldr	$t1,[$b_ptr,#4]
	 ldr	$t2,[$b_ptr,#8]
	 ldr	$t3,[$b_ptr,#12]
	subs	$a0,$a0,$t0
	 ldr	$t0,[$b_ptr,#16]
	sbcs	$a1,$a1,$t1
	 ldr	$t1,[$b_ptr,#20]
	sbcs	$a2,$a2,$t2
	 ldr	$t2,[$b_ptr,#24]
	sbcs	$a3,$a3,$t3
	 ldr	$t3,[$b_ptr,#28]
	sbcs	$a4,$a4,$t0
	sbcs	$a5,$a5,$t1
	sbcs	$a6,$a6,$t2
	sbcs	$a7,$a7,$t3
	sbc	$ff,$ff,$ff		@ broadcast borrow bit
	ldr	lr,[sp],#4		@ pop lr

	adds	$a0,$a0,$ff		@ add synthesized modulus
	adcs	$a1,$a1,$ff
	str	$a0,[$r_ptr,#0]
	adcs	$a2,$a2,$ff
	str	$a1,[$r_ptr,#4]
	adcs	$a3,$a3,#0
	str	$a2,[$r_ptr,#8]
	adcs	$a4,$a4,#0
	str	$a3,[$r_ptr,#12]
	adcs	$a5,$a5,#0
	str	$a4,[$r_ptr,#16]
	adcs	$a6,$a6,$ff,lsr#31
	str	$a5,[$r_ptr,#20]
	adcs	$a7,$a7,$ff
	str	$a6,[$r_ptr,#24]
	str	$a7,[$r_ptr,#28]

	mov	pc,lr
.size	__ecp_nistz256_sub_from,.-__ecp_nistz256_sub_from

.type	__ecp_nistz256_sub_morf,%function
.align	5
__ecp_nistz256_sub_morf:
	str	lr,[sp,#-4]!		@ push lr

	 ldr	$t0,[$b_ptr,#0]
	 ldr	$t1,[$b_ptr,#4]
	 ldr	$t2,[$b_ptr,#8]
	 ldr	$t3,[$b_ptr,#12]
	subs	$a0,$t0,$a0
	 ldr	$t0,[$b_ptr,#16]
	sbcs	$a1,$t1,$a1
	 ldr	$t1,[$b_ptr,#20]
	sbcs	$a2,$t2,$a2
	 ldr	$t2,[$b_ptr,#24]
	sbcs	$a3,$t3,$a3
	 ldr	$t3,[$b_ptr,#28]
	sbcs	$a4,$t0,$a4
	sbcs	$a5,$t1,$a5
	sbcs	$a6,$t2,$a6
	sbcs	$a7,$t3,$a7
	sbc	$ff,$ff,$ff		@ broadcast borrow bit
	ldr	lr,[sp],#4		@ pop lr

	adds	$a0,$a0,$ff		@ add synthesized modulus
	adcs	$a1,$a1,$ff
	str	$a0,[$r_ptr,#0]
	adcs	$a2,$a2,$ff
	str	$a1,[$r_ptr,#4]
	adcs	$a3,$a3,#0
	str	$a2,[$r_ptr,#8]
	adcs	$a4,$a4,#0
	str	$a3,[$r_ptr,#12]
	adcs	$a5,$a5,#0
	str	$a4,[$r_ptr,#16]
	adcs	$a6,$a6,$ff,lsr#31
	str	$a5,[$r_ptr,#20]
	adcs	$a7,$a7,$ff
	str	$a6,[$r_ptr,#24]
	str	$a7,[$r_ptr,#28]

	mov	pc,lr
.size	__ecp_nistz256_sub_morf,.-__ecp_nistz256_sub_morf

.type	__ecp_nistz256_add_self,%function
.align	4
__ecp_nistz256_add_self:
	adds	$a0,$a0,$a0		@ a[0:7]+=a[0:7]
	adcs	$a1,$a1,$a1
	adcs	$a2,$a2,$a2
	adcs	$a3,$a3,$a3
	adcs	$a4,$a4,$a4
	adcs	$a5,$a5,$a5
	adcs	$a6,$a6,$a6
	mov	$ff,#0
	adcs	$a7,$a7,$a7
	adc	$ff,$ff,#0

	@ if a+b >= modulus, subtract modulus.
	@
	@ But since comparison implies subtraction, we subtract
	@ modulus and then add it back if subtraction borrowed.

	subs	$a0,$a0,#-1
	sbcs	$a1,$a1,#-1
	sbcs	$a2,$a2,#-1
	sbcs	$a3,$a3,#0
	sbcs	$a4,$a4,#0
	sbcs	$a5,$a5,#0
	sbcs	$a6,$a6,#1
	sbcs	$a7,$a7,#-1
	sbc	$ff,$ff,#0

	@ Note that because mod has special form, i.e. consists of
	@ 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	@ using value of borrow as a whole or extracting single bit.
	@ Follow $ff register...

	adds	$a0,$a0,$ff		@ add synthesized modulus
	adcs	$a1,$a1,$ff
	str	$a0,[$r_ptr,#0]
	adcs	$a2,$a2,$ff
	str	$a1,[$r_ptr,#4]
	adcs	$a3,$a3,#0
	str	$a2,[$r_ptr,#8]
	adcs	$a4,$a4,#0
	str	$a3,[$r_ptr,#12]
	adcs	$a5,$a5,#0
	str	$a4,[$r_ptr,#16]
	adcs	$a6,$a6,$ff,lsr#31
	str	$a5,[$r_ptr,#20]
	adcs	$a7,$a7,$ff
	str	$a6,[$r_ptr,#24]
	str	$a7,[$r_ptr,#28]

	mov	pc,lr
.size	__ecp_nistz256_add_self,.-__ecp_nistz256_add_self

___

########################################################################
# following subroutines are "literal" implementation of those found in
# ecp_nistz256.c
#
########################################################################
# void ecp_nistz256_point_double(P256_POINT *out,const P256_POINT *inp);
#
{
my ($S,$M,$Zsqr,$in_x,$tmp0)=map(32*$_,(0..4));
# above map() describes stack layout with 5 temporary
# 256-bit vectors on top. Then note that we push
# starting from r0, which means that we have copy of
# input arguments just below these temporary vectors.

$code.=<<___;
.globl	GFp_nistz256_point_double
.type	GFp_nistz256_point_double,%function
.align	5
GFp_nistz256_point_double:
	stmdb	sp!,{r0-r12,lr}		@ push from r0, unusual, but intentional
	sub	sp,sp,#32*5

.Lpoint_double_shortcut:
	add	r3,sp,#$in_x
	ldmia	$a_ptr!,{r4-r11}	@ copy in_x
	stmia	r3,{r4-r11}

	add	$r_ptr,sp,#$S
	bl	__ecp_nistz256_mul_by_2	@ p256_mul_by_2(S, in_y);

	add	$b_ptr,$a_ptr,#32
	add	$a_ptr,$a_ptr,#32
	add	$r_ptr,sp,#$Zsqr
	bl	__ecp_nistz256_mul_mont	@ p256_sqr_mont(Zsqr, in_z);

	add	$a_ptr,sp,#$S
	add	$b_ptr,sp,#$S
	add	$r_ptr,sp,#$S
	bl	__ecp_nistz256_mul_mont	@ p256_sqr_mont(S, S);

	ldr	$b_ptr,[sp,#32*5+4]
	add	$a_ptr,$b_ptr,#32
	add	$b_ptr,$b_ptr,#64
	add	$r_ptr,sp,#$tmp0
	bl	__ecp_nistz256_mul_mont	@ p256_mul_mont(tmp0, in_z, in_y);

	ldr	$r_ptr,[sp,#32*5]
	add	$r_ptr,$r_ptr,#64
	bl	__ecp_nistz256_add_self	@ p256_mul_by_2(res_z, tmp0);

	add	$a_ptr,sp,#$in_x
	add	$b_ptr,sp,#$Zsqr
	add	$r_ptr,sp,#$M
	bl	__ecp_nistz256_add	@ p256_add(M, in_x, Zsqr);

	add	$a_ptr,sp,#$in_x
	add	$b_ptr,sp,#$Zsqr
	add	$r_ptr,sp,#$Zsqr
	bl	__ecp_nistz256_sub	@ p256_sub(Zsqr, in_x, Zsqr);

	add	$a_ptr,sp,#$S
	add	$b_ptr,sp,#$S
	add	$r_ptr,sp,#$tmp0
	bl	__ecp_nistz256_mul_mont	@ p256_sqr_mont(tmp0, S);

	add	$a_ptr,sp,#$Zsqr
	add	$b_ptr,sp,#$M
	add	$r_ptr,sp,#$M
	bl	__ecp_nistz256_mul_mont	@ p256_mul_mont(M, M, Zsqr);

	ldr	$r_ptr,[sp,#32*5]
	add	$a_ptr,sp,#$tmp0
	add	$r_ptr,$r_ptr,#32
	bl	__ecp_nistz256_div_by_2	@ p256_div_by_2(res_y, tmp0);

	add	$a_ptr,sp,#$M
	add	$r_ptr,sp,#$M
	bl	__ecp_nistz256_mul_by_3	@ p256_mul_by_3(M, M);

	add	$a_ptr,sp,#$in_x
	add	$b_ptr,sp,#$S
	add	$r_ptr,sp,#$S
	bl	__ecp_nistz256_mul_mont	@ p256_mul_mont(S, S, in_x);

	add	$r_ptr,sp,#$tmp0
	bl	__ecp_nistz256_add_self	@ p256_mul_by_2(tmp0, S);

	ldr	$r_ptr,[sp,#32*5]
	add	$a_ptr,sp,#$M
	add	$b_ptr,sp,#$M
	bl	__ecp_nistz256_mul_mont	@ p256_sqr_mont(res_x, M);

	add	$b_ptr,sp,#$tmp0
	bl	__ecp_nistz256_sub_from	@ p256_sub(res_x, res_x, tmp0);

	add	$b_ptr,sp,#$S
	add	$r_ptr,sp,#$S
	bl	__ecp_nistz256_sub_morf	@ p256_sub(S, S, res_x);

	add	$a_ptr,sp,#$M
	add	$b_ptr,sp,#$S
	bl	__ecp_nistz256_mul_mont	@ p256_mul_mont(S, S, M);

	ldr	$r_ptr,[sp,#32*5]
	add	$b_ptr,$r_ptr,#32
	add	$r_ptr,$r_ptr,#32
	bl	__ecp_nistz256_sub_from	@ p256_sub(res_y, S, res_y);

	add	sp,sp,#32*5+16		@ +16 means "skip even over saved r0-r3"
#if __ARM_ARCH__>=5 || !defined(__thumb__)
	ldmia	sp!,{r4-r12,pc}
#else
	ldmia	sp!,{r4-r12,lr}
	bx	lr			@ interoperable with Thumb ISA:-)
#endif
.size	GFp_nistz256_point_double,.-GFp_nistz256_point_double
___
}

}}}

foreach (split("\n",$code)) {
	s/\`([^\`]*)\`/eval $1/geo;

	s/\bq([0-9]+)#(lo|hi)/sprintf "d%d",2*$1+($2 eq "hi")/geo;

	print $_,"\n";
}
close STDOUT or die "error closing STDOUT";
