.text	

.globl	ct_is_square_mod_384
.hidden	ct_is_square_mod_384
.type	ct_is_square_mod_384,@function
.align	32
ct_is_square_mod_384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa


	pushq	%rbp
.cfi_adjust_cfa_offset	8
.cfi_offset	%rbp,-16
	pushq	%rbx
.cfi_adjust_cfa_offset	8
.cfi_offset	%rbx,-24
	pushq	%r12
.cfi_adjust_cfa_offset	8
.cfi_offset	%r12,-32
	pushq	%r13
.cfi_adjust_cfa_offset	8
.cfi_offset	%r13,-40
	pushq	%r14
.cfi_adjust_cfa_offset	8
.cfi_offset	%r14,-48
	pushq	%r15
.cfi_adjust_cfa_offset	8
.cfi_offset	%r15,-56
	subq	$536,%rsp
.cfi_adjust_cfa_offset	536


	leaq	24+255(%rsp),%rax
	andq	$-256,%rax

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rdi),%r8
	movq	8(%rdi),%r9
	movq	16(%rdi),%r10
	movq	24(%rdi),%r11
	movq	32(%rdi),%r12
	movq	40(%rdi),%r13

	movq	0(%rsi),%r14
	movq	8(%rsi),%r15
	movq	16(%rsi),%rbx
	movq	24(%rsi),%rcx
	movq	32(%rsi),%rdx
	movq	40(%rsi),%rdi
	movq	%rax,%rsi

	movq	%r8,0(%rax)
	movq	%r9,8(%rax)
	movq	%r10,16(%rax)
	movq	%r11,24(%rax)
	movq	%r12,32(%rax)
	movq	%r13,40(%rax)

	movq	%r14,48(%rax)
	movq	%r15,56(%rax)
	movq	%rbx,64(%rax)
	movq	%rcx,72(%rax)
	movq	%rdx,80(%rax)
	movq	%rdi,88(%rax)

	xorq	%rbp,%rbp
	movl	$24,%ecx
	jmp	.Loop_is_square

.align	32
.Loop_is_square:
	movl	%ecx,16(%rsp)

	call	__ab_approximation_30
	movq	%rax,0(%rsp)
	movq	%rbx,8(%rsp)

	movq	$128+48,%rdi
	xorq	%rsi,%rdi
	call	__smulq_384_n_shift_by_30

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	-48(%rdi),%rdi
	call	__smulq_384_n_shift_by_30

	movl	16(%rsp),%ecx
	xorq	$128,%rsi

	andq	48(%rdi),%r14
	shrq	$1,%r14
	addq	%r14,%rbp

	subl	$1,%ecx
	jnz	.Loop_is_square




	movq	48(%rsi),%r9
	call	__inner_loop_48

	movq	$1,%rax
	andq	%rbp,%rax
	xorq	$1,%rax

	leaq	536(%rsp),%r8
	movq	0(%r8),%r15
.cfi_restore	%r15
	movq	8(%r8),%r14
.cfi_restore	%r14
	movq	16(%r8),%r13
.cfi_restore	%r13
	movq	24(%r8),%r12
.cfi_restore	%r12
	movq	32(%r8),%rbx
.cfi_restore	%rbx
	movq	40(%r8),%rbp
.cfi_restore	%rbp
	leaq	48(%r8),%rsp
.cfi_adjust_cfa_offset	-536-8*6

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	
.size	ct_is_square_mod_384,.-ct_is_square_mod_384

.type	__smulq_384_n_shift_by_30,@function
.align	32
__smulq_384_n_shift_by_30:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	movq	%rdx,%rbx
	sarq	$63,%rdx
	xorq	%rax,%rax
	subq	%rdx,%rax

	xorq	%rdx,%rbx
	addq	%rax,%rbx

	xorq	%rdx,%r8
	xorq	%rdx,%r9
	xorq	%rdx,%r10
	xorq	%rdx,%r11
	xorq	%rdx,%r12
	xorq	%rdx,%r13
	addq	%r8,%rax
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%r13

	movq	%rdx,%r14
	andq	%rbx,%r14
	mulq	%rbx
	movq	%rax,%r8
	movq	%r9,%rax
	movq	%rdx,%r9
	mulq	%rbx
	addq	%rax,%r9
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10
	mulq	%rbx
	addq	%rax,%r10
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11
	mulq	%rbx
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	mulq	%rbx
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13
	negq	%r14
	mulq	%rbx
	addq	%rax,%r13
	adcq	%rdx,%r14
	leaq	48(%rsi),%rsi
	movq	%rcx,%rdx

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%r13,40(%rdi)
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	movq	%rdx,%rbx
	sarq	$63,%rdx
	xorq	%rax,%rax
	subq	%rdx,%rax

	xorq	%rdx,%rbx
	addq	%rax,%rbx

	xorq	%rdx,%r8
	xorq	%rdx,%r9
	xorq	%rdx,%r10
	xorq	%rdx,%r11
	xorq	%rdx,%r12
	xorq	%rdx,%r13
	addq	%r8,%rax
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%r13

	movq	%rdx,%r15
	andq	%rbx,%r15
	mulq	%rbx
	movq	%rax,%r8
	movq	%r9,%rax
	movq	%rdx,%r9
	mulq	%rbx
	addq	%rax,%r9
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10
	mulq	%rbx
	addq	%rax,%r10
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11
	mulq	%rbx
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	mulq	%rbx
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13
	negq	%r15
	mulq	%rbx
	addq	%rax,%r13
	adcq	%rdx,%r15
	leaq	-48(%rsi),%rsi

	addq	0(%rdi),%r8
	adcq	8(%rdi),%r9
	adcq	16(%rdi),%r10
	adcq	24(%rdi),%r11
	adcq	32(%rdi),%r12
	adcq	40(%rdi),%r13
	adcq	%r15,%r14

	shrdq	$30,%r9,%r8
	shrdq	$30,%r10,%r9
	shrdq	$30,%r11,%r10
	shrdq	$30,%r12,%r11
	shrdq	$30,%r13,%r12
	shrdq	$30,%r14,%r13

	sarq	$63,%r14
	xorq	%rbx,%rbx
	subq	%r14,%rbx

	xorq	%r14,%r8
	xorq	%r14,%r9
	xorq	%r14,%r10
	xorq	%r14,%r11
	xorq	%r14,%r12
	xorq	%r14,%r13
	addq	%rbx,%r8
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%r13

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%r13,40(%rdi)

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	__smulq_384_n_shift_by_30,.-__smulq_384_n_shift_by_30
.type	__ab_approximation_30,@function
.align	32
__ab_approximation_30:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	88(%rsi),%rbx
	movq	80(%rsi),%r15
	movq	72(%rsi),%r14

	movq	%r13,%rax
	orq	%rbx,%rax
	cmovzq	%r12,%r13
	cmovzq	%r15,%rbx
	cmovzq	%r11,%r12
	movq	64(%rsi),%r11
	cmovzq	%r14,%r15

	movq	%r13,%rax
	orq	%rbx,%rax
	cmovzq	%r12,%r13
	cmovzq	%r15,%rbx
	cmovzq	%r10,%r12
	movq	56(%rsi),%r10
	cmovzq	%r11,%r15

	movq	%r13,%rax
	orq	%rbx,%rax
	cmovzq	%r12,%r13
	cmovzq	%r15,%rbx
	cmovzq	%r9,%r12
	movq	48(%rsi),%r9
	cmovzq	%r10,%r15

	movq	%r13,%rax
	orq	%rbx,%rax
	cmovzq	%r12,%r13
	cmovzq	%r15,%rbx
	cmovzq	%r8,%r12
	cmovzq	%r9,%r15

	movq	%r13,%rax
	orq	%rbx,%rax
	bsrq	%rax,%rcx
	leaq	1(%rcx),%rcx
	cmovzq	%r8,%r13
	cmovzq	%r9,%rbx
	cmovzq	%rax,%rcx
	negq	%rcx


	shldq	%cl,%r12,%r13
	shldq	%cl,%r15,%rbx

	movq	$0xFFFFFFFF00000000,%rax
	movl	%r8d,%r8d
	movl	%r9d,%r9d
	andq	%rax,%r13
	andq	%rax,%rbx
	orq	%r13,%r8
	orq	%rbx,%r9

	jmp	__inner_loop_30

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	__ab_approximation_30,.-__ab_approximation_30
.type	__inner_loop_30,@function
.align	32
__inner_loop_30:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	$0x7FFFFFFF80000000,%rbx
	movq	$0x800000007FFFFFFF,%rcx
	leaq	-1(%rbx),%r15
	movl	$30,%edi

.Loop_30:
	movq	%r8,%rax
	andq	%r9,%rax
	shrq	$1,%rax

	cmpq	%r9,%r8
	movq	%r8,%r10
	movq	%r9,%r11
	leaq	(%rax,%rbp,1),%rax
	movq	%rbx,%r12
	movq	%rcx,%r13
	movq	%rbp,%r14
	cmovbq	%r9,%r8
	cmovbq	%r10,%r9
	cmovbq	%rcx,%rbx
	cmovbq	%r12,%rcx
	cmovbq	%rax,%rbp

	subq	%r9,%r8
	subq	%rcx,%rbx
	addq	%r15,%rbx

	testq	$1,%r10
	cmovzq	%r10,%r8
	cmovzq	%r11,%r9
	cmovzq	%r12,%rbx
	cmovzq	%r13,%rcx
	cmovzq	%r14,%rbp

	leaq	2(%r9),%rax
	shrq	$1,%r8
	shrq	$2,%rax
	addq	%rcx,%rcx
	leaq	(%rax,%rbp,1),%rbp
	subq	%r15,%rcx

	subl	$1,%edi
	jnz	.Loop_30

	shrq	$32,%r15
	movl	%ebx,%eax
	shrq	$32,%rbx
	movl	%ecx,%edx
	shrq	$32,%rcx
	subq	%r15,%rax
	subq	%r15,%rbx
	subq	%r15,%rdx
	subq	%r15,%rcx

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%r8
	lfence
	jmpq	*%r8
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	__inner_loop_30,.-__inner_loop_30

.type	__inner_loop_48,@function
.align	32
__inner_loop_48:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movl	$48,%edi

.Loop_48:
	movq	%r8,%rax
	andq	%r9,%rax
	shrq	$1,%rax

	cmpq	%r9,%r8
	movq	%r8,%r10
	movq	%r9,%r11
	leaq	(%rax,%rbp,1),%rax
	movq	%rbp,%r12
	cmovbq	%r9,%r8
	cmovbq	%r10,%r9
	cmovbq	%rax,%rbp

	subq	%r9,%r8

	testq	$1,%r10
	cmovzq	%r10,%r8
	cmovzq	%r11,%r9
	cmovzq	%r12,%rbp

	leaq	2(%r9),%rax
	shrq	$1,%r8
	shrq	$2,%rax
	addq	%rax,%rbp

	subl	$1,%edi
	jnz	.Loop_48

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	__inner_loop_48,.-__inner_loop_48

.section	.note.GNU-stack,"",@progbits
#ifndef	__SGX_LVI_HARDENING__
.section	.note.gnu.property,"a",@note
	.long	4,2f-1f,5
	.byte	0x47,0x4E,0x55,0
1:	.long	0xc0000002,4,3
.align	8
2:
#endif
