// This file is generated from a similarly-named Perl script in the BoringSSL
// source tree. Do not edit by hand.

#if !defined(__has_feature)
#define __has_feature(x) 0
#endif
#if __has_feature(memory_sanitizer) && !defined(OPENSSL_NO_ASM)
#define OPENSSL_NO_ASM
#endif

#if !defined(OPENSSL_NO_ASM)
#if defined(__arm__)
#include <GFp/arm_arch.h>

.text
#if defined(__thumb2__)
.syntax	unified
.thumb
#else
.code	32
#endif

.byte	69,67,80,95,78,73,83,84,90,50,53,54,32,102,111,114,32,65,82,77,118,52,44,32,67,82,89,80,84,79,71,65,77,83,32,98,121,32,60,97,112,112,114,111,64,111,112,101,110,115,115,108,46,111,114,103,62,0
.align	2
.align	6
.type	__ecp_nistz256_mul_by_2,%function
.align	4
__ecp_nistz256_mul_by_2:
	ldr	r4,[r1,#0]
	ldr	r5,[r1,#4]
	ldr	r6,[r1,#8]
	adds	r4,r4,r4		@ a[0:7]+=a[0:7], i.e. add with itself
	ldr	r7,[r1,#12]
	adcs	r5,r5,r5
	ldr	r8,[r1,#16]
	adcs	r6,r6,r6
	ldr	r9,[r1,#20]
	adcs	r7,r7,r7
	ldr	r10,[r1,#24]
	adcs	r8,r8,r8
	ldr	r11,[r1,#28]
	adcs	r9,r9,r9
	adcs	r10,r10,r10
	mov	r3,#0
	adcs	r11,r11,r11
	adc	r3,r3,#0

	b	.Lreduce_by_sub
.size	__ecp_nistz256_mul_by_2,.-__ecp_nistz256_mul_by_2

@ void	GFp_nistz256_add(BN_ULONG r0[8],const BN_ULONG r1[8],
@					const BN_ULONG r2[8]);
.globl	GFp_nistz256_add
.hidden	GFp_nistz256_add
.type	GFp_nistz256_add,%function
.align	4
GFp_nistz256_add:
	stmdb	sp!,{r4,r5,r6,r7,r8,r9,r10,r11,r12,lr}
	bl	__ecp_nistz256_add
#if __ARM_ARCH__>=5 || !defined(__thumb__)
	ldmia	sp!,{r4,r5,r6,r7,r8,r9,r10,r11,r12,pc}
#else
	ldmia	sp!,{r4,r5,r6,r7,r8,r9,r10,r11,r12,lr}
	bx	lr			@ interoperable with Thumb ISA:-)
#endif
.size	GFp_nistz256_add,.-GFp_nistz256_add

.type	__ecp_nistz256_add,%function
.align	4
__ecp_nistz256_add:
	str	lr,[sp,#-4]!		@ push lr

	ldr	r4,[r1,#0]
	ldr	r5,[r1,#4]
	ldr	r6,[r1,#8]
	ldr	r7,[r1,#12]
	ldr	r8,[r1,#16]
	ldr	r3,[r2,#0]
	ldr	r9,[r1,#20]
	ldr	r12,[r2,#4]
	ldr	r10,[r1,#24]
	ldr	r14,[r2,#8]
	ldr	r11,[r1,#28]
	ldr	r1,[r2,#12]
	adds	r4,r4,r3
	ldr	r3,[r2,#16]
	adcs	r5,r5,r12
	ldr	r12,[r2,#20]
	adcs	r6,r6,r14
	ldr	r14,[r2,#24]
	adcs	r7,r7,r1
	ldr	r1,[r2,#28]
	adcs	r8,r8,r3
	adcs	r9,r9,r12
	adcs	r10,r10,r14
	mov	r3,#0
	adcs	r11,r11,r1
	adc	r3,r3,#0
	ldr	lr,[sp],#4		@ pop lr

.Lreduce_by_sub:

	@ if a+b >= modulus, subtract modulus.
	@
	@ But since comparison implies subtraction, we subtract
	@ modulus and then add it back if subtraction borrowed.

	subs	r4,r4,#-1
	sbcs	r5,r5,#-1
	sbcs	r6,r6,#-1
	sbcs	r7,r7,#0
	sbcs	r8,r8,#0
	sbcs	r9,r9,#0
	sbcs	r10,r10,#1
	sbcs	r11,r11,#-1
	sbc	r3,r3,#0

	@ Note that because mod has special form, i.e. consists of
	@ 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	@ using value of borrow as a whole or extracting single bit.
	@ Follow r3 register...

	adds	r4,r4,r3		@ add synthesized modulus
	adcs	r5,r5,r3
	str	r4,[r0,#0]
	adcs	r6,r6,r3
	str	r5,[r0,#4]
	adcs	r7,r7,#0
	str	r6,[r0,#8]
	adcs	r8,r8,#0
	str	r7,[r0,#12]
	adcs	r9,r9,#0
	str	r8,[r0,#16]
	adcs	r10,r10,r3,lsr#31
	str	r9,[r0,#20]
	adcs	r11,r11,r3
	str	r10,[r0,#24]
	str	r11,[r0,#28]

	mov	pc,lr
.size	__ecp_nistz256_add,.-__ecp_nistz256_add

.type	__ecp_nistz256_mul_by_3,%function
.align	4
__ecp_nistz256_mul_by_3:
	str	lr,[sp,#-4]!		@ push lr

	@ As multiplication by 3 is performed as 2*n+n, below are inline
	@ copies of __ecp_nistz256_mul_by_2 and __ecp_nistz256_add, see
	@ corresponding subroutines for details.

	ldr	r4,[r1,#0]
	ldr	r5,[r1,#4]
	ldr	r6,[r1,#8]
	adds	r4,r4,r4		@ a[0:7]+=a[0:7]
	ldr	r7,[r1,#12]
	adcs	r5,r5,r5
	ldr	r8,[r1,#16]
	adcs	r6,r6,r6
	ldr	r9,[r1,#20]
	adcs	r7,r7,r7
	ldr	r10,[r1,#24]
	adcs	r8,r8,r8
	ldr	r11,[r1,#28]
	adcs	r9,r9,r9
	adcs	r10,r10,r10
	mov	r3,#0
	adcs	r11,r11,r11
	adc	r3,r3,#0

	subs	r4,r4,#-1		@ .Lreduce_by_sub but without stores
	sbcs	r5,r5,#-1
	sbcs	r6,r6,#-1
	sbcs	r7,r7,#0
	sbcs	r8,r8,#0
	sbcs	r9,r9,#0
	sbcs	r10,r10,#1
	sbcs	r11,r11,#-1
	sbc	r3,r3,#0

	adds	r4,r4,r3		@ add synthesized modulus
	adcs	r5,r5,r3
	adcs	r6,r6,r3
	adcs	r7,r7,#0
	adcs	r8,r8,#0
	ldr	r2,[r1,#0]
	adcs	r9,r9,#0
	ldr	r12,[r1,#4]
	adcs	r10,r10,r3,lsr#31
	ldr	r14,[r1,#8]
	adc	r11,r11,r3

	ldr	r3,[r1,#12]
	adds	r4,r4,r2		@ 2*a[0:7]+=a[0:7]
	ldr	r2,[r1,#16]
	adcs	r5,r5,r12
	ldr	r12,[r1,#20]
	adcs	r6,r6,r14
	ldr	r14,[r1,#24]
	adcs	r7,r7,r3
	ldr	r1,[r1,#28]
	adcs	r8,r8,r2
	adcs	r9,r9,r12
	adcs	r10,r10,r14
	mov	r3,#0
	adcs	r11,r11,r1
	adc	r3,r3,#0
	ldr	lr,[sp],#4		@ pop lr

	b	.Lreduce_by_sub
.size	__ecp_nistz256_mul_by_3,.-__ecp_nistz256_mul_by_3

.type	__ecp_nistz256_div_by_2,%function
.align	4
__ecp_nistz256_div_by_2:
	@ ret = (a is odd ? a+mod : a) >> 1

	ldr	r4,[r1,#0]
	ldr	r5,[r1,#4]
	ldr	r6,[r1,#8]
	mov	r3,r4,lsl#31		@ place least significant bit to most
					@ significant position, now arithmetic
					@ right shift by 31 will produce -1 or
					@ 0, while logical right shift 1 or 0,
					@ this is how modulus is conditionally
					@ synthesized in this case...
	ldr	r7,[r1,#12]
	adds	r4,r4,r3,asr#31
	ldr	r8,[r1,#16]
	adcs	r5,r5,r3,asr#31
	ldr	r9,[r1,#20]
	adcs	r6,r6,r3,asr#31
	ldr	r10,[r1,#24]
	adcs	r7,r7,#0
	ldr	r11,[r1,#28]
	adcs	r8,r8,#0
	mov	r4,r4,lsr#1		@ a[0:7]>>=1, we can start early
					@ because it doesn't affect flags
	adcs	r9,r9,#0
	orr	r4,r4,r5,lsl#31
	adcs	r10,r10,r3,lsr#31
	mov	r2,#0
	adcs	r11,r11,r3,asr#31
	mov	r5,r5,lsr#1
	adc	r2,r2,#0	@ top-most carry bit from addition

	orr	r5,r5,r6,lsl#31
	mov	r6,r6,lsr#1
	str	r4,[r0,#0]
	orr	r6,r6,r7,lsl#31
	mov	r7,r7,lsr#1
	str	r5,[r0,#4]
	orr	r7,r7,r8,lsl#31
	mov	r8,r8,lsr#1
	str	r6,[r0,#8]
	orr	r8,r8,r9,lsl#31
	mov	r9,r9,lsr#1
	str	r7,[r0,#12]
	orr	r9,r9,r10,lsl#31
	mov	r10,r10,lsr#1
	str	r8,[r0,#16]
	orr	r10,r10,r11,lsl#31
	mov	r11,r11,lsr#1
	str	r9,[r0,#20]
	orr	r11,r11,r2,lsl#31	@ don't forget the top-most carry bit
	str	r10,[r0,#24]
	str	r11,[r0,#28]

	mov	pc,lr
.size	__ecp_nistz256_div_by_2,.-__ecp_nistz256_div_by_2

.type	__ecp_nistz256_sub,%function
.align	4
__ecp_nistz256_sub:
	str	lr,[sp,#-4]!		@ push lr

	ldr	r4,[r1,#0]
	ldr	r5,[r1,#4]
	ldr	r6,[r1,#8]
	ldr	r7,[r1,#12]
	ldr	r8,[r1,#16]
	ldr	r3,[r2,#0]
	ldr	r9,[r1,#20]
	ldr	r12,[r2,#4]
	ldr	r10,[r1,#24]
	ldr	r14,[r2,#8]
	ldr	r11,[r1,#28]
	ldr	r1,[r2,#12]
	subs	r4,r4,r3
	ldr	r3,[r2,#16]
	sbcs	r5,r5,r12
	ldr	r12,[r2,#20]
	sbcs	r6,r6,r14
	ldr	r14,[r2,#24]
	sbcs	r7,r7,r1
	ldr	r1,[r2,#28]
	sbcs	r8,r8,r3
	sbcs	r9,r9,r12
	sbcs	r10,r10,r14
	sbcs	r11,r11,r1
	sbc	r3,r3,r3		@ broadcast borrow bit
	ldr	lr,[sp],#4		@ pop lr

.Lreduce_by_add:

	@ if a-b borrows, add modulus.
	@
	@ Note that because mod has special form, i.e. consists of
	@ 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	@ broadcasting borrow bit to a register, r3, and using it as
	@ a whole or extracting single bit.

	adds	r4,r4,r3		@ add synthesized modulus
	adcs	r5,r5,r3
	str	r4,[r0,#0]
	adcs	r6,r6,r3
	str	r5,[r0,#4]
	adcs	r7,r7,#0
	str	r6,[r0,#8]
	adcs	r8,r8,#0
	str	r7,[r0,#12]
	adcs	r9,r9,#0
	str	r8,[r0,#16]
	adcs	r10,r10,r3,lsr#31
	str	r9,[r0,#20]
	adcs	r11,r11,r3
	str	r10,[r0,#24]
	str	r11,[r0,#28]

	mov	pc,lr
.size	__ecp_nistz256_sub,.-__ecp_nistz256_sub

@ void	GFp_nistz256_neg(BN_ULONG r0[8],const BN_ULONG r1[8]);
.globl	GFp_nistz256_neg
.hidden	GFp_nistz256_neg
.type	GFp_nistz256_neg,%function
.align	4
GFp_nistz256_neg:
	stmdb	sp!,{r4,r5,r6,r7,r8,r9,r10,r11,r12,lr}
	bl	__ecp_nistz256_neg
#if __ARM_ARCH__>=5 || !defined(__thumb__)
	ldmia	sp!,{r4,r5,r6,r7,r8,r9,r10,r11,r12,pc}
#else
	ldmia	sp!,{r4,r5,r6,r7,r8,r9,r10,r11,r12,lr}
	bx	lr			@ interoperable with Thumb ISA:-)
#endif
.size	GFp_nistz256_neg,.-GFp_nistz256_neg

.type	__ecp_nistz256_neg,%function
.align	4
__ecp_nistz256_neg:
	ldr	r4,[r1,#0]
	eor	r3,r3,r3
	ldr	r5,[r1,#4]
	ldr	r6,[r1,#8]
	subs	r4,r3,r4
	ldr	r7,[r1,#12]
	sbcs	r5,r3,r5
	ldr	r8,[r1,#16]
	sbcs	r6,r3,r6
	ldr	r9,[r1,#20]
	sbcs	r7,r3,r7
	ldr	r10,[r1,#24]
	sbcs	r8,r3,r8
	ldr	r11,[r1,#28]
	sbcs	r9,r3,r9
	sbcs	r10,r3,r10
	sbcs	r11,r3,r11
	sbc	r3,r3,r3

	b	.Lreduce_by_add
.size	__ecp_nistz256_neg,.-__ecp_nistz256_neg
@ void	GFp_nistz256_mul_mont(BN_ULONG r0[8],const BN_ULONG r1[8],
@					     const BN_ULONG r2[8]);
.globl	GFp_nistz256_mul_mont
.hidden	GFp_nistz256_mul_mont
.type	GFp_nistz256_mul_mont,%function
.align	4
GFp_nistz256_mul_mont:
	stmdb	sp!,{r4,r5,r6,r7,r8,r9,r10,r11,r12,lr}
	bl	__ecp_nistz256_mul_mont
#if __ARM_ARCH__>=5 || !defined(__thumb__)
	ldmia	sp!,{r4,r5,r6,r7,r8,r9,r10,r11,r12,pc}
#else
	ldmia	sp!,{r4,r5,r6,r7,r8,r9,r10,r11,r12,lr}
	bx	lr			@ interoperable with Thumb ISA:-)
#endif
.size	GFp_nistz256_mul_mont,.-GFp_nistz256_mul_mont

.type	__ecp_nistz256_mul_mont,%function
.align	4
__ecp_nistz256_mul_mont:
	stmdb	sp!,{r0,r1,r2,lr}			@ make a copy of arguments too

	ldr	r2,[r2,#0]			@ b[0]
	ldmia	r1,{r4,r5,r6,r7,r8,r9,r10,r11}

	umull	r3,r14,r4,r2		@ r[0]=a[0]*b[0]
	stmdb	sp!,{r4,r5,r6,r7,r8,r9,r10,r11}		@ copy a[0-7] to stack, so
						@ that it can be addressed
						@ without spending register
						@ on address
	umull	r4,r0,r5,r2		@ r[1]=a[1]*b[0]
	umull	r5,r1,r6,r2
	adds	r4,r4,r14		@ accumulate high part of mult
	umull	r6,r12,r7,r2
	adcs	r5,r5,r0
	umull	r7,r14,r8,r2
	adcs	r6,r6,r1
	umull	r8,r0,r9,r2
	adcs	r7,r7,r12
	umull	r9,r1,r10,r2
	adcs	r8,r8,r14
	umull	r10,r12,r11,r2
	adcs	r9,r9,r0
	adcs	r10,r10,r1
	eor	r14,r14,r14			@ first overflow bit is zero
	adc	r11,r12,#0
	@ multiplication-less reduction 1
	adds	r6,r6,r3		@ r[3]+=r[0]
	ldr	r2,[sp,#40]			@ restore b_ptr
	adcs	r7,r7,#0		@ r[4]+=0
	adcs	r8,r8,#0		@ r[5]+=0
	adcs	r9,r9,r3		@ r[6]+=r[0]
	ldr	r1,[sp,#0]			@ load a[0]
	adcs	r10,r10,#0		@ r[7]+=0
	ldr	r2,[r2,#4*1]			@ load b[i]
	adcs	r11,r11,r3		@ r[8]+=r[0]
	eor	r0,r0,r0
	adc	r14,r14,#0			@ overflow bit
	subs	r10,r10,r3		@ r[7]-=r[0]
	ldr	r12,[sp,#4]			@ a[1]
	sbcs	r11,r11,#0		@ r[8]-=0
	umlal	r4,r0,r1,r2		@ "r[0]"+=a[0]*b[i]
	eor	r1,r1,r1
	sbc	r3,r14,#0			@ overflow bit, keep in mind
						@ that netto result is
						@ addition of a value which
						@ makes underflow impossible

	ldr	r14,[sp,#8]			@ a[2]
	umlal	r5,r1,r12,r2		@ "r[1]"+=a[1]*b[i]
	str	r3,[sp,#36]		@ temporarily offload overflow
	eor	r12,r12,r12
	ldr	r3,[sp,#12]			@ a[3], r3 is alias r3
	umlal	r6,r12,r14,r2		@ "r[2]"+=a[2]*b[i]
	eor	r14,r14,r14
	adds	r5,r5,r0		@ accumulate high part of mult
	ldr	r0,[sp,#16]			@ a[4]
	umlal	r7,r14,r3,r2		@ "r[3]"+=a[3]*b[i]
	eor	r3,r3,r3
	adcs	r6,r6,r1
	ldr	r1,[sp,#20]			@ a[5]
	umlal	r8,r3,r0,r2		@ "r[4]"+=a[4]*b[i]
	eor	r0,r0,r0
	adcs	r7,r7,r12
	ldr	r12,[sp,#24]			@ a[6]
	umlal	r9,r0,r1,r2		@ "r[5]"+=a[5]*b[i]
	eor	r1,r1,r1
	adcs	r8,r8,r14
	ldr	r14,[sp,#28]			@ a[7]
	umlal	r10,r1,r12,r2		@ "r[6]"+=a[6]*b[i]
	eor	r12,r12,r12
	adcs	r9,r9,r3
	ldr	r3,[sp,#36]		@ restore overflow bit
	umlal	r11,r12,r14,r2		@ "r[7]"+=a[7]*b[i]
	eor	r14,r14,r14
	adcs	r10,r10,r0
	adcs	r11,r11,r1
	adcs	r3,r3,r12
	adc	r14,r14,#0			@ new overflow bit
	@ multiplication-less reduction 2
	adds	r7,r7,r4		@ r[3]+=r[0]
	ldr	r2,[sp,#40]			@ restore b_ptr
	adcs	r8,r8,#0		@ r[4]+=0
	adcs	r9,r9,#0		@ r[5]+=0
	adcs	r10,r10,r4		@ r[6]+=r[0]
	ldr	r1,[sp,#0]			@ load a[0]
	adcs	r11,r11,#0		@ r[7]+=0
	ldr	r2,[r2,#4*2]			@ load b[i]
	adcs	r3,r3,r4		@ r[8]+=r[0]
	eor	r0,r0,r0
	adc	r14,r14,#0			@ overflow bit
	subs	r11,r11,r4		@ r[7]-=r[0]
	ldr	r12,[sp,#4]			@ a[1]
	sbcs	r3,r3,#0		@ r[8]-=0
	umlal	r5,r0,r1,r2		@ "r[0]"+=a[0]*b[i]
	eor	r1,r1,r1
	sbc	r4,r14,#0			@ overflow bit, keep in mind
						@ that netto result is
						@ addition of a value which
						@ makes underflow impossible

	ldr	r14,[sp,#8]			@ a[2]
	umlal	r6,r1,r12,r2		@ "r[1]"+=a[1]*b[i]
	str	r4,[sp,#36]		@ temporarily offload overflow
	eor	r12,r12,r12
	ldr	r4,[sp,#12]			@ a[3], r4 is alias r4
	umlal	r7,r12,r14,r2		@ "r[2]"+=a[2]*b[i]
	eor	r14,r14,r14
	adds	r6,r6,r0		@ accumulate high part of mult
	ldr	r0,[sp,#16]			@ a[4]
	umlal	r8,r14,r4,r2		@ "r[3]"+=a[3]*b[i]
	eor	r4,r4,r4
	adcs	r7,r7,r1
	ldr	r1,[sp,#20]			@ a[5]
	umlal	r9,r4,r0,r2		@ "r[4]"+=a[4]*b[i]
	eor	r0,r0,r0
	adcs	r8,r8,r12
	ldr	r12,[sp,#24]			@ a[6]
	umlal	r10,r0,r1,r2		@ "r[5]"+=a[5]*b[i]
	eor	r1,r1,r1
	adcs	r9,r9,r14
	ldr	r14,[sp,#28]			@ a[7]
	umlal	r11,r1,r12,r2		@ "r[6]"+=a[6]*b[i]
	eor	r12,r12,r12
	adcs	r10,r10,r4
	ldr	r4,[sp,#36]		@ restore overflow bit
	umlal	r3,r12,r14,r2		@ "r[7]"+=a[7]*b[i]
	eor	r14,r14,r14
	adcs	r11,r11,r0
	adcs	r3,r3,r1
	adcs	r4,r4,r12
	adc	r14,r14,#0			@ new overflow bit
	@ multiplication-less reduction 3
	adds	r8,r8,r5		@ r[3]+=r[0]
	ldr	r2,[sp,#40]			@ restore b_ptr
	adcs	r9,r9,#0		@ r[4]+=0
	adcs	r10,r10,#0		@ r[5]+=0
	adcs	r11,r11,r5		@ r[6]+=r[0]
	ldr	r1,[sp,#0]			@ load a[0]
	adcs	r3,r3,#0		@ r[7]+=0
	ldr	r2,[r2,#4*3]			@ load b[i]
	adcs	r4,r4,r5		@ r[8]+=r[0]
	eor	r0,r0,r0
	adc	r14,r14,#0			@ overflow bit
	subs	r3,r3,r5		@ r[7]-=r[0]
	ldr	r12,[sp,#4]			@ a[1]
	sbcs	r4,r4,#0		@ r[8]-=0
	umlal	r6,r0,r1,r2		@ "r[0]"+=a[0]*b[i]
	eor	r1,r1,r1
	sbc	r5,r14,#0			@ overflow bit, keep in mind
						@ that netto result is
						@ addition of a value which
						@ makes underflow impossible

	ldr	r14,[sp,#8]			@ a[2]
	umlal	r7,r1,r12,r2		@ "r[1]"+=a[1]*b[i]
	str	r5,[sp,#36]		@ temporarily offload overflow
	eor	r12,r12,r12
	ldr	r5,[sp,#12]			@ a[3], r5 is alias r5
	umlal	r8,r12,r14,r2		@ "r[2]"+=a[2]*b[i]
	eor	r14,r14,r14
	adds	r7,r7,r0		@ accumulate high part of mult
	ldr	r0,[sp,#16]			@ a[4]
	umlal	r9,r14,r5,r2		@ "r[3]"+=a[3]*b[i]
	eor	r5,r5,r5
	adcs	r8,r8,r1
	ldr	r1,[sp,#20]			@ a[5]
	umlal	r10,r5,r0,r2		@ "r[4]"+=a[4]*b[i]
	eor	r0,r0,r0
	adcs	r9,r9,r12
	ldr	r12,[sp,#24]			@ a[6]
	umlal	r11,r0,r1,r2		@ "r[5]"+=a[5]*b[i]
	eor	r1,r1,r1
	adcs	r10,r10,r14
	ldr	r14,[sp,#28]			@ a[7]
	umlal	r3,r1,r12,r2		@ "r[6]"+=a[6]*b[i]
	eor	r12,r12,r12
	adcs	r11,r11,r5
	ldr	r5,[sp,#36]		@ restore overflow bit
	umlal	r4,r12,r14,r2		@ "r[7]"+=a[7]*b[i]
	eor	r14,r14,r14
	adcs	r3,r3,r0
	adcs	r4,r4,r1
	adcs	r5,r5,r12
	adc	r14,r14,#0			@ new overflow bit
	@ multiplication-less reduction 4
	adds	r9,r9,r6		@ r[3]+=r[0]
	ldr	r2,[sp,#40]			@ restore b_ptr
	adcs	r10,r10,#0		@ r[4]+=0
	adcs	r11,r11,#0		@ r[5]+=0
	adcs	r3,r3,r6		@ r[6]+=r[0]
	ldr	r1,[sp,#0]			@ load a[0]
	adcs	r4,r4,#0		@ r[7]+=0
	ldr	r2,[r2,#4*4]			@ load b[i]
	adcs	r5,r5,r6		@ r[8]+=r[0]
	eor	r0,r0,r0
	adc	r14,r14,#0			@ overflow bit
	subs	r4,r4,r6		@ r[7]-=r[0]
	ldr	r12,[sp,#4]			@ a[1]
	sbcs	r5,r5,#0		@ r[8]-=0
	umlal	r7,r0,r1,r2		@ "r[0]"+=a[0]*b[i]
	eor	r1,r1,r1
	sbc	r6,r14,#0			@ overflow bit, keep in mind
						@ that netto result is
						@ addition of a value which
						@ makes underflow impossible

	ldr	r14,[sp,#8]			@ a[2]
	umlal	r8,r1,r12,r2		@ "r[1]"+=a[1]*b[i]
	str	r6,[sp,#36]		@ temporarily offload overflow
	eor	r12,r12,r12
	ldr	r6,[sp,#12]			@ a[3], r6 is alias r6
	umlal	r9,r12,r14,r2		@ "r[2]"+=a[2]*b[i]
	eor	r14,r14,r14
	adds	r8,r8,r0		@ accumulate high part of mult
	ldr	r0,[sp,#16]			@ a[4]
	umlal	r10,r14,r6,r2		@ "r[3]"+=a[3]*b[i]
	eor	r6,r6,r6
	adcs	r9,r9,r1
	ldr	r1,[sp,#20]			@ a[5]
	umlal	r11,r6,r0,r2		@ "r[4]"+=a[4]*b[i]
	eor	r0,r0,r0
	adcs	r10,r10,r12
	ldr	r12,[sp,#24]			@ a[6]
	umlal	r3,r0,r1,r2		@ "r[5]"+=a[5]*b[i]
	eor	r1,r1,r1
	adcs	r11,r11,r14
	ldr	r14,[sp,#28]			@ a[7]
	umlal	r4,r1,r12,r2		@ "r[6]"+=a[6]*b[i]
	eor	r12,r12,r12
	adcs	r3,r3,r6
	ldr	r6,[sp,#36]		@ restore overflow bit
	umlal	r5,r12,r14,r2		@ "r[7]"+=a[7]*b[i]
	eor	r14,r14,r14
	adcs	r4,r4,r0
	adcs	r5,r5,r1
	adcs	r6,r6,r12
	adc	r14,r14,#0			@ new overflow bit
	@ multiplication-less reduction 5
	adds	r10,r10,r7		@ r[3]+=r[0]
	ldr	r2,[sp,#40]			@ restore b_ptr
	adcs	r11,r11,#0		@ r[4]+=0
	adcs	r3,r3,#0		@ r[5]+=0
	adcs	r4,r4,r7		@ r[6]+=r[0]
	ldr	r1,[sp,#0]			@ load a[0]
	adcs	r5,r5,#0		@ r[7]+=0
	ldr	r2,[r2,#4*5]			@ load b[i]
	adcs	r6,r6,r7		@ r[8]+=r[0]
	eor	r0,r0,r0
	adc	r14,r14,#0			@ overflow bit
	subs	r5,r5,r7		@ r[7]-=r[0]
	ldr	r12,[sp,#4]			@ a[1]
	sbcs	r6,r6,#0		@ r[8]-=0
	umlal	r8,r0,r1,r2		@ "r[0]"+=a[0]*b[i]
	eor	r1,r1,r1
	sbc	r7,r14,#0			@ overflow bit, keep in mind
						@ that netto result is
						@ addition of a value which
						@ makes underflow impossible

	ldr	r14,[sp,#8]			@ a[2]
	umlal	r9,r1,r12,r2		@ "r[1]"+=a[1]*b[i]
	str	r7,[sp,#36]		@ temporarily offload overflow
	eor	r12,r12,r12
	ldr	r7,[sp,#12]			@ a[3], r7 is alias r7
	umlal	r10,r12,r14,r2		@ "r[2]"+=a[2]*b[i]
	eor	r14,r14,r14
	adds	r9,r9,r0		@ accumulate high part of mult
	ldr	r0,[sp,#16]			@ a[4]
	umlal	r11,r14,r7,r2		@ "r[3]"+=a[3]*b[i]
	eor	r7,r7,r7
	adcs	r10,r10,r1
	ldr	r1,[sp,#20]			@ a[5]
	umlal	r3,r7,r0,r2		@ "r[4]"+=a[4]*b[i]
	eor	r0,r0,r0
	adcs	r11,r11,r12
	ldr	r12,[sp,#24]			@ a[6]
	umlal	r4,r0,r1,r2		@ "r[5]"+=a[5]*b[i]
	eor	r1,r1,r1
	adcs	r3,r3,r14
	ldr	r14,[sp,#28]			@ a[7]
	umlal	r5,r1,r12,r2		@ "r[6]"+=a[6]*b[i]
	eor	r12,r12,r12
	adcs	r4,r4,r7
	ldr	r7,[sp,#36]		@ restore overflow bit
	umlal	r6,r12,r14,r2		@ "r[7]"+=a[7]*b[i]
	eor	r14,r14,r14
	adcs	r5,r5,r0
	adcs	r6,r6,r1
	adcs	r7,r7,r12
	adc	r14,r14,#0			@ new overflow bit
	@ multiplication-less reduction 6
	adds	r11,r11,r8		@ r[3]+=r[0]
	ldr	r2,[sp,#40]			@ restore b_ptr
	adcs	r3,r3,#0		@ r[4]+=0
	adcs	r4,r4,#0		@ r[5]+=0
	adcs	r5,r5,r8		@ r[6]+=r[0]
	ldr	r1,[sp,#0]			@ load a[0]
	adcs	r6,r6,#0		@ r[7]+=0
	ldr	r2,[r2,#4*6]			@ load b[i]
	adcs	r7,r7,r8		@ r[8]+=r[0]
	eor	r0,r0,r0
	adc	r14,r14,#0			@ overflow bit
	subs	r6,r6,r8		@ r[7]-=r[0]
	ldr	r12,[sp,#4]			@ a[1]
	sbcs	r7,r7,#0		@ r[8]-=0
	umlal	r9,r0,r1,r2		@ "r[0]"+=a[0]*b[i]
	eor	r1,r1,r1
	sbc	r8,r14,#0			@ overflow bit, keep in mind
						@ that netto result is
						@ addition of a value which
						@ makes underflow impossible

	ldr	r14,[sp,#8]			@ a[2]
	umlal	r10,r1,r12,r2		@ "r[1]"+=a[1]*b[i]
	str	r8,[sp,#36]		@ temporarily offload overflow
	eor	r12,r12,r12
	ldr	r8,[sp,#12]			@ a[3], r8 is alias r8
	umlal	r11,r12,r14,r2		@ "r[2]"+=a[2]*b[i]
	eor	r14,r14,r14
	adds	r10,r10,r0		@ accumulate high part of mult
	ldr	r0,[sp,#16]			@ a[4]
	umlal	r3,r14,r8,r2		@ "r[3]"+=a[3]*b[i]
	eor	r8,r8,r8
	adcs	r11,r11,r1
	ldr	r1,[sp,#20]			@ a[5]
	umlal	r4,r8,r0,r2		@ "r[4]"+=a[4]*b[i]
	eor	r0,r0,r0
	adcs	r3,r3,r12
	ldr	r12,[sp,#24]			@ a[6]
	umlal	r5,r0,r1,r2		@ "r[5]"+=a[5]*b[i]
	eor	r1,r1,r1
	adcs	r4,r4,r14
	ldr	r14,[sp,#28]			@ a[7]
	umlal	r6,r1,r12,r2		@ "r[6]"+=a[6]*b[i]
	eor	r12,r12,r12
	adcs	r5,r5,r8
	ldr	r8,[sp,#36]		@ restore overflow bit
	umlal	r7,r12,r14,r2		@ "r[7]"+=a[7]*b[i]
	eor	r14,r14,r14
	adcs	r6,r6,r0
	adcs	r7,r7,r1
	adcs	r8,r8,r12
	adc	r14,r14,#0			@ new overflow bit
	@ multiplication-less reduction 7
	adds	r3,r3,r9		@ r[3]+=r[0]
	ldr	r2,[sp,#40]			@ restore b_ptr
	adcs	r4,r4,#0		@ r[4]+=0
	adcs	r5,r5,#0		@ r[5]+=0
	adcs	r6,r6,r9		@ r[6]+=r[0]
	ldr	r1,[sp,#0]			@ load a[0]
	adcs	r7,r7,#0		@ r[7]+=0
	ldr	r2,[r2,#4*7]			@ load b[i]
	adcs	r8,r8,r9		@ r[8]+=r[0]
	eor	r0,r0,r0
	adc	r14,r14,#0			@ overflow bit
	subs	r7,r7,r9		@ r[7]-=r[0]
	ldr	r12,[sp,#4]			@ a[1]
	sbcs	r8,r8,#0		@ r[8]-=0
	umlal	r10,r0,r1,r2		@ "r[0]"+=a[0]*b[i]
	eor	r1,r1,r1
	sbc	r9,r14,#0			@ overflow bit, keep in mind
						@ that netto result is
						@ addition of a value which
						@ makes underflow impossible

	ldr	r14,[sp,#8]			@ a[2]
	umlal	r11,r1,r12,r2		@ "r[1]"+=a[1]*b[i]
	str	r9,[sp,#36]		@ temporarily offload overflow
	eor	r12,r12,r12
	ldr	r9,[sp,#12]			@ a[3], r9 is alias r9
	umlal	r3,r12,r14,r2		@ "r[2]"+=a[2]*b[i]
	eor	r14,r14,r14
	adds	r11,r11,r0		@ accumulate high part of mult
	ldr	r0,[sp,#16]			@ a[4]
	umlal	r4,r14,r9,r2		@ "r[3]"+=a[3]*b[i]
	eor	r9,r9,r9
	adcs	r3,r3,r1
	ldr	r1,[sp,#20]			@ a[5]
	umlal	r5,r9,r0,r2		@ "r[4]"+=a[4]*b[i]
	eor	r0,r0,r0
	adcs	r4,r4,r12
	ldr	r12,[sp,#24]			@ a[6]
	umlal	r6,r0,r1,r2		@ "r[5]"+=a[5]*b[i]
	eor	r1,r1,r1
	adcs	r5,r5,r14
	ldr	r14,[sp,#28]			@ a[7]
	umlal	r7,r1,r12,r2		@ "r[6]"+=a[6]*b[i]
	eor	r12,r12,r12
	adcs	r6,r6,r9
	ldr	r9,[sp,#36]		@ restore overflow bit
	umlal	r8,r12,r14,r2		@ "r[7]"+=a[7]*b[i]
	eor	r14,r14,r14
	adcs	r7,r7,r0
	adcs	r8,r8,r1
	adcs	r9,r9,r12
	adc	r14,r14,#0			@ new overflow bit
	@ last multiplication-less reduction
	adds	r4,r4,r10
	ldr	r0,[sp,#32]			@ restore r_ptr
	adcs	r5,r5,#0
	adcs	r6,r6,#0
	adcs	r7,r7,r10
	adcs	r8,r8,#0
	adcs	r9,r9,r10
	adc	r14,r14,#0
	subs	r8,r8,r10
	sbcs	r9,r9,#0
	sbc	r10,r14,#0			@ overflow bit

	@ Final step is "if result > mod, subtract mod", but we do it
	@ "other way around", namely subtract modulus from result
	@ and if it borrowed, add modulus back.

	adds	r11,r11,#1		@ subs	r11,r11,#-1
	adcs	r3,r3,#0		@ sbcs	r3,r3,#-1
	adcs	r4,r4,#0		@ sbcs	r4,r4,#-1
	sbcs	r5,r5,#0
	sbcs	r6,r6,#0
	sbcs	r7,r7,#0
	sbcs	r8,r8,#1
	adcs	r9,r9,#0		@ sbcs	r9,r9,#-1
	ldr	lr,[sp,#44]			@ restore lr
	sbc	r10,r10,#0		@ broadcast borrow bit
	add	sp,sp,#48

	@ Note that because mod has special form, i.e. consists of
	@ 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	@ broadcasting borrow bit to a register, r10, and using it as
	@ a whole or extracting single bit.

	adds	r11,r11,r10		@ add modulus or zero
	adcs	r3,r3,r10
	str	r11,[r0,#0]
	adcs	r4,r4,r10
	str	r3,[r0,#4]
	adcs	r5,r5,#0
	str	r4,[r0,#8]
	adcs	r6,r6,#0
	str	r5,[r0,#12]
	adcs	r7,r7,#0
	str	r6,[r0,#16]
	adcs	r8,r8,r10,lsr#31
	str	r7,[r0,#20]
	adc	r9,r9,r10
	str	r8,[r0,#24]
	str	r9,[r0,#28]

	mov	pc,lr
.size	__ecp_nistz256_mul_mont,.-__ecp_nistz256_mul_mont
.type	__ecp_nistz256_sub_from,%function
.align	5
__ecp_nistz256_sub_from:
	str	lr,[sp,#-4]!		@ push lr

	ldr	r10,[r2,#0]
	ldr	r12,[r2,#4]
	ldr	r14,[r2,#8]
	ldr	r1,[r2,#12]
	subs	r11,r11,r10
	ldr	r10,[r2,#16]
	sbcs	r3,r3,r12
	ldr	r12,[r2,#20]
	sbcs	r4,r4,r14
	ldr	r14,[r2,#24]
	sbcs	r5,r5,r1
	ldr	r1,[r2,#28]
	sbcs	r6,r6,r10
	sbcs	r7,r7,r12
	sbcs	r8,r8,r14
	sbcs	r9,r9,r1
	sbc	r2,r2,r2		@ broadcast borrow bit
	ldr	lr,[sp],#4		@ pop lr

	adds	r11,r11,r2		@ add synthesized modulus
	adcs	r3,r3,r2
	str	r11,[r0,#0]
	adcs	r4,r4,r2
	str	r3,[r0,#4]
	adcs	r5,r5,#0
	str	r4,[r0,#8]
	adcs	r6,r6,#0
	str	r5,[r0,#12]
	adcs	r7,r7,#0
	str	r6,[r0,#16]
	adcs	r8,r8,r2,lsr#31
	str	r7,[r0,#20]
	adcs	r9,r9,r2
	str	r8,[r0,#24]
	str	r9,[r0,#28]

	mov	pc,lr
.size	__ecp_nistz256_sub_from,.-__ecp_nistz256_sub_from

.type	__ecp_nistz256_sub_morf,%function
.align	5
__ecp_nistz256_sub_morf:
	str	lr,[sp,#-4]!		@ push lr

	ldr	r10,[r2,#0]
	ldr	r12,[r2,#4]
	ldr	r14,[r2,#8]
	ldr	r1,[r2,#12]
	subs	r11,r10,r11
	ldr	r10,[r2,#16]
	sbcs	r3,r12,r3
	ldr	r12,[r2,#20]
	sbcs	r4,r14,r4
	ldr	r14,[r2,#24]
	sbcs	r5,r1,r5
	ldr	r1,[r2,#28]
	sbcs	r6,r10,r6
	sbcs	r7,r12,r7
	sbcs	r8,r14,r8
	sbcs	r9,r1,r9
	sbc	r2,r2,r2		@ broadcast borrow bit
	ldr	lr,[sp],#4		@ pop lr

	adds	r11,r11,r2		@ add synthesized modulus
	adcs	r3,r3,r2
	str	r11,[r0,#0]
	adcs	r4,r4,r2
	str	r3,[r0,#4]
	adcs	r5,r5,#0
	str	r4,[r0,#8]
	adcs	r6,r6,#0
	str	r5,[r0,#12]
	adcs	r7,r7,#0
	str	r6,[r0,#16]
	adcs	r8,r8,r2,lsr#31
	str	r7,[r0,#20]
	adcs	r9,r9,r2
	str	r8,[r0,#24]
	str	r9,[r0,#28]

	mov	pc,lr
.size	__ecp_nistz256_sub_morf,.-__ecp_nistz256_sub_morf

.type	__ecp_nistz256_add_self,%function
.align	4
__ecp_nistz256_add_self:
	adds	r11,r11,r11		@ a[0:7]+=a[0:7]
	adcs	r3,r3,r3
	adcs	r4,r4,r4
	adcs	r5,r5,r5
	adcs	r6,r6,r6
	adcs	r7,r7,r7
	adcs	r8,r8,r8
	mov	r2,#0
	adcs	r9,r9,r9
	adc	r2,r2,#0

	@ if a+b >= modulus, subtract modulus.
	@
	@ But since comparison implies subtraction, we subtract
	@ modulus and then add it back if subtraction borrowed.

	subs	r11,r11,#-1
	sbcs	r3,r3,#-1
	sbcs	r4,r4,#-1
	sbcs	r5,r5,#0
	sbcs	r6,r6,#0
	sbcs	r7,r7,#0
	sbcs	r8,r8,#1
	sbcs	r9,r9,#-1
	sbc	r2,r2,#0

	@ Note that because mod has special form, i.e. consists of
	@ 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	@ using value of borrow as a whole or extracting single bit.
	@ Follow r2 register...

	adds	r11,r11,r2		@ add synthesized modulus
	adcs	r3,r3,r2
	str	r11,[r0,#0]
	adcs	r4,r4,r2
	str	r3,[r0,#4]
	adcs	r5,r5,#0
	str	r4,[r0,#8]
	adcs	r6,r6,#0
	str	r5,[r0,#12]
	adcs	r7,r7,#0
	str	r6,[r0,#16]
	adcs	r8,r8,r2,lsr#31
	str	r7,[r0,#20]
	adcs	r9,r9,r2
	str	r8,[r0,#24]
	str	r9,[r0,#28]

	mov	pc,lr
.size	__ecp_nistz256_add_self,.-__ecp_nistz256_add_self

.globl	GFp_nistz256_point_double
.hidden	GFp_nistz256_point_double
.type	GFp_nistz256_point_double,%function
.align	5
GFp_nistz256_point_double:
	stmdb	sp!,{r0,r1,r2,r3,r4,r5,r6,r7,r8,r9,r10,r11,r12,lr}		@ push from r0, unusual, but intentional
	sub	sp,sp,#32*5

.Lpoint_double_shortcut:
	add	r3,sp,#96
	ldmia	r1!,{r4,r5,r6,r7,r8,r9,r10,r11}	@ copy in_x
	stmia	r3,{r4,r5,r6,r7,r8,r9,r10,r11}

	add	r0,sp,#0
	bl	__ecp_nistz256_mul_by_2	@ p256_mul_by_2(S, in_y);

	add	r2,r1,#32
	add	r1,r1,#32
	add	r0,sp,#64
	bl	__ecp_nistz256_mul_mont	@ p256_sqr_mont(Zsqr, in_z);

	add	r1,sp,#0
	add	r2,sp,#0
	add	r0,sp,#0
	bl	__ecp_nistz256_mul_mont	@ p256_sqr_mont(S, S);

	ldr	r2,[sp,#32*5+4]
	add	r1,r2,#32
	add	r2,r2,#64
	add	r0,sp,#128
	bl	__ecp_nistz256_mul_mont	@ p256_mul_mont(tmp0, in_z, in_y);

	ldr	r0,[sp,#32*5]
	add	r0,r0,#64
	bl	__ecp_nistz256_add_self	@ p256_mul_by_2(res_z, tmp0);

	add	r1,sp,#96
	add	r2,sp,#64
	add	r0,sp,#32
	bl	__ecp_nistz256_add	@ p256_add(M, in_x, Zsqr);

	add	r1,sp,#96
	add	r2,sp,#64
	add	r0,sp,#64
	bl	__ecp_nistz256_sub	@ p256_sub(Zsqr, in_x, Zsqr);

	add	r1,sp,#0
	add	r2,sp,#0
	add	r0,sp,#128
	bl	__ecp_nistz256_mul_mont	@ p256_sqr_mont(tmp0, S);

	add	r1,sp,#64
	add	r2,sp,#32
	add	r0,sp,#32
	bl	__ecp_nistz256_mul_mont	@ p256_mul_mont(M, M, Zsqr);

	ldr	r0,[sp,#32*5]
	add	r1,sp,#128
	add	r0,r0,#32
	bl	__ecp_nistz256_div_by_2	@ p256_div_by_2(res_y, tmp0);

	add	r1,sp,#32
	add	r0,sp,#32
	bl	__ecp_nistz256_mul_by_3	@ p256_mul_by_3(M, M);

	add	r1,sp,#96
	add	r2,sp,#0
	add	r0,sp,#0
	bl	__ecp_nistz256_mul_mont	@ p256_mul_mont(S, S, in_x);

	add	r0,sp,#128
	bl	__ecp_nistz256_add_self	@ p256_mul_by_2(tmp0, S);

	ldr	r0,[sp,#32*5]
	add	r1,sp,#32
	add	r2,sp,#32
	bl	__ecp_nistz256_mul_mont	@ p256_sqr_mont(res_x, M);

	add	r2,sp,#128
	bl	__ecp_nistz256_sub_from	@ p256_sub(res_x, res_x, tmp0);

	add	r2,sp,#0
	add	r0,sp,#0
	bl	__ecp_nistz256_sub_morf	@ p256_sub(S, S, res_x);

	add	r1,sp,#32
	add	r2,sp,#0
	bl	__ecp_nistz256_mul_mont	@ p256_mul_mont(S, S, M);

	ldr	r0,[sp,#32*5]
	add	r2,r0,#32
	add	r0,r0,#32
	bl	__ecp_nistz256_sub_from	@ p256_sub(res_y, S, res_y);

	add	sp,sp,#32*5+16		@ +16 means "skip even over saved r0-r3"
#if __ARM_ARCH__>=5 || !defined(__thumb__)
	ldmia	sp!,{r4,r5,r6,r7,r8,r9,r10,r11,r12,pc}
#else
	ldmia	sp!,{r4,r5,r6,r7,r8,r9,r10,r11,r12,lr}
	bx	lr			@ interoperable with Thumb ISA:-)
#endif
.size	GFp_nistz256_point_double,.-GFp_nistz256_point_double
#endif
#endif  // !OPENSSL_NO_ASM
.section	.note.GNU-stack,"",%progbits
