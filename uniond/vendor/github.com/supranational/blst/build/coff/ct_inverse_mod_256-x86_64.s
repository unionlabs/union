.text	

.globl	ct_inverse_mod_256

.def	ct_inverse_mod_256;	.scl 2;	.type 32;	.endef
.p2align	5
ct_inverse_mod_256:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_ct_inverse_mod_256:


	pushq	%rbp

	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$1072,%rsp

.LSEH_body_ct_inverse_mod_256:


	leaq	48+511(%rsp),%rax
	andq	$-512,%rax
	movq	%rdi,32(%rsp)
	movq	%rcx,40(%rsp)

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11

	movq	0(%rdx),%r12
	movq	8(%rdx),%r13
	movq	16(%rdx),%r14
	movq	24(%rdx),%r15

	movq	%r8,0(%rax)
	movq	%r9,8(%rax)
	movq	%r10,16(%rax)
	movq	%r11,24(%rax)

	movq	%r12,32(%rax)
	movq	%r13,40(%rax)
	movq	%r14,48(%rax)
	movq	%r15,56(%rax)
	movq	%rax,%rsi


	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31


	movq	%rdx,64(%rdi)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31


	movq	%rdx,72(%rdi)


	xorq	$256,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31



	movq	64(%rsi),%r8
	movq	104(%rsi),%r12
	movq	%r8,%r9
	imulq	0(%rsp),%r8
	movq	%r12,%r13
	imulq	8(%rsp),%r12
	addq	%r12,%r8
	movq	%r8,32(%rdi)
	sarq	$63,%r8
	movq	%r8,40(%rdi)
	movq	%r8,48(%rdi)
	movq	%r8,56(%rdi)
	movq	%r8,64(%rdi)
	leaq	64(%rsi),%rsi

	imulq	%rdx,%r9
	imulq	%rcx,%r13
	addq	%r13,%r9
	movq	%r9,72(%rdi)
	sarq	$63,%r9
	movq	%r9,80(%rdi)
	movq	%r9,88(%rdi)
	movq	%r9,96(%rdi)
	movq	%r9,104(%rdi)
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_256x63
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_256x63
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_256x63
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_256x63
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_256x63
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_256x63
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_256x63
	sarq	$63,%rbp
	movq	%rbp,40(%rdi)
	movq	%rbp,48(%rdi)
	movq	%rbp,56(%rdi)
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_512x63
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_512x63
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_512x63
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_512x63
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_512x63
	xorq	$256+64,%rsi
	movl	$31,%edx
	call	__ab_approximation_31_256


	movq	%r12,16(%rsp)
	movq	%r13,24(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,0(%rsp)
	movq	%rcx,8(%rsp)

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	32(%rdi),%rdi
	call	__smulq_256_n_shift_by_31
	movq	%rdx,16(%rsp)
	movq	%rcx,24(%rsp)

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	64(%rsi),%rsi
	leaq	32(%rdi),%rdi
	call	__smulq_256x63

	movq	16(%rsp),%rdx
	movq	24(%rsp),%rcx
	leaq	40(%rdi),%rdi
	call	__smulq_512x63

	xorq	$256+64,%rsi
	movl	$47,%edx

	movq	0(%rsi),%r8

	movq	32(%rsi),%r10

	call	__inner_loop_62_256







	leaq	64(%rsi),%rsi





	movq	%r12,%rdx
	movq	%r13,%rcx
	movq	32(%rsp),%rdi
	call	__smulq_512x63
	adcq	%rbp,%rdx

	movq	40(%rsp),%rsi
	movq	%rdx,%rax
	sarq	$63,%rdx

	movq	%rdx,%r8
	movq	%rdx,%r9
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	andq	0(%rsi),%r8
	movq	%rdx,%r10
	andq	8(%rsi),%r9
	andq	16(%rsi),%r10
	andq	24(%rsi),%rdx

	addq	%r8,%r12
	adcq	%r9,%r13
	adcq	%r10,%r14
	adcq	%rdx,%r15
	adcq	$0,%rax

	movq	%rax,%rdx
	negq	%rax
	orq	%rax,%rdx
	sarq	$63,%rax

	movq	%rdx,%r8
	movq	%rdx,%r9
	andq	0(%rsi),%r8
	movq	%rdx,%r10
	andq	8(%rsi),%r9
	andq	16(%rsi),%r10
	andq	24(%rsi),%rdx

	xorq	%rax,%r8
	xorq	%rcx,%rcx
	xorq	%rax,%r9
	subq	%rax,%rcx
	xorq	%rax,%r10
	xorq	%rax,%rdx
	addq	%rcx,%r8
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%rdx

	addq	%r8,%r12
	adcq	%r9,%r13
	adcq	%r10,%r14
	adcq	%rdx,%r15

	movq	%r12,32(%rdi)
	movq	%r13,40(%rdi)
	movq	%r14,48(%rdi)
	movq	%r15,56(%rdi)

	leaq	1072(%rsp),%r8
	movq	0(%r8),%r15

	movq	8(%r8),%r14

	movq	16(%r8),%r13

	movq	24(%r8),%r12

	movq	32(%r8),%rbx

	movq	40(%r8),%rbp

	leaq	48(%r8),%rsp

.LSEH_epilogue_ct_inverse_mod_256:
	mov	8(%rsp),%rdi
	mov	16(%rsp),%rsi

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif

.LSEH_end_ct_inverse_mod_256:
.def	__smulq_512x63;	.scl 3;	.type 32;	.endef
.p2align	5
__smulq_512x63:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%rbp

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
	xorq	%rdx,%rbp
	addq	%r8,%rax
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%rbp

	mulq	%rbx
	movq	%rax,0(%rdi)
	movq	%r9,%rax
	movq	%rdx,%r9
	mulq	%rbx
	addq	%rax,%r9
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%r9,8(%rdi)
	movq	%rdx,%r10
	mulq	%rbx
	addq	%rax,%r10
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%r10,16(%rdi)
	movq	%rdx,%r11
	andq	%rbx,%rbp
	negq	%rbp
	mulq	%rbx
	addq	%rax,%r11
	adcq	%rdx,%rbp
	movq	%r11,24(%rdi)

	movq	40(%rsi),%r8
	movq	48(%rsi),%r9
	movq	56(%rsi),%r10
	movq	64(%rsi),%r11
	movq	72(%rsi),%r12
	movq	80(%rsi),%r13
	movq	88(%rsi),%r14
	movq	96(%rsi),%r15

	movq	%rcx,%rdx
	sarq	$63,%rdx
	xorq	%rax,%rax
	subq	%rdx,%rax

	xorq	%rdx,%rcx
	addq	%rax,%rcx

	xorq	%rdx,%r8
	xorq	%rdx,%r9
	xorq	%rdx,%r10
	xorq	%rdx,%r11
	xorq	%rdx,%r12
	xorq	%rdx,%r13
	xorq	%rdx,%r14
	xorq	%rdx,%r15
	addq	%r8,%rax
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%r13
	adcq	$0,%r14
	adcq	$0,%r15

	mulq	%rcx
	movq	%rax,%r8
	movq	%r9,%rax
	movq	%rdx,%r9
	mulq	%rcx
	addq	%rax,%r9
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10
	mulq	%rcx
	addq	%rax,%r10
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11
	mulq	%rcx
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	mulq	%rcx
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13
	mulq	%rcx
	addq	%rax,%r13
	movq	%r14,%rax
	adcq	$0,%rdx
	movq	%rdx,%r14
	mulq	%rcx
	addq	%rax,%r14
	movq	%r15,%rax
	adcq	$0,%rdx
	movq	%rdx,%r15
	imulq	%rcx
	addq	%rax,%r15
	adcq	$0,%rdx

	movq	%rbp,%rbx
	sarq	$63,%rbp

	addq	0(%rdi),%r8
	adcq	8(%rdi),%r9
	adcq	16(%rdi),%r10
	adcq	24(%rdi),%r11
	adcq	%rbx,%r12
	adcq	%rbp,%r13
	adcq	%rbp,%r14
	adcq	%rbp,%r15

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%r13,40(%rdi)
	movq	%r14,48(%rdi)
	movq	%r15,56(%rdi)

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%r8
	lfence
	jmpq	*%r8
	ud2
#else
	.byte	0xf3,0xc3
#endif


.def	__smulq_256x63;	.scl 3;	.type 32;	.endef
.p2align	5
__smulq_256x63:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0+0(%rsi),%r8
	movq	0+8(%rsi),%r9
	movq	0+16(%rsi),%r10
	movq	0+24(%rsi),%r11
	movq	0+32(%rsi),%rbp

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
	xorq	%rdx,%rbp
	addq	%r8,%rax
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%rbp

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
	andq	%rbx,%rbp
	negq	%rbp
	mulq	%rbx
	addq	%rax,%r11
	adcq	%rdx,%rbp
	movq	%rcx,%rdx
	movq	40+0(%rsi),%r12
	movq	40+8(%rsi),%r13
	movq	40+16(%rsi),%r14
	movq	40+24(%rsi),%r15
	movq	40+32(%rsi),%rcx

	movq	%rdx,%rbx
	sarq	$63,%rdx
	xorq	%rax,%rax
	subq	%rdx,%rax

	xorq	%rdx,%rbx
	addq	%rax,%rbx

	xorq	%rdx,%r12
	xorq	%rdx,%r13
	xorq	%rdx,%r14
	xorq	%rdx,%r15
	xorq	%rdx,%rcx
	addq	%r12,%rax
	adcq	$0,%r13
	adcq	$0,%r14
	adcq	$0,%r15
	adcq	$0,%rcx

	mulq	%rbx
	movq	%rax,%r12
	movq	%r13,%rax
	movq	%rdx,%r13
	mulq	%rbx
	addq	%rax,%r13
	movq	%r14,%rax
	adcq	$0,%rdx
	movq	%rdx,%r14
	mulq	%rbx
	addq	%rax,%r14
	movq	%r15,%rax
	adcq	$0,%rdx
	movq	%rdx,%r15
	andq	%rbx,%rcx
	negq	%rcx
	mulq	%rbx
	addq	%rax,%r15
	adcq	%rdx,%rcx
	addq	%r12,%r8
	adcq	%r13,%r9
	adcq	%r14,%r10
	adcq	%r15,%r11
	adcq	%rcx,%rbp

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%rbp,32(%rdi)

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif

.def	__smulq_256_n_shift_by_31;	.scl 3;	.type 32;	.endef
.p2align	5
__smulq_256_n_shift_by_31:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	%rdx,0(%rdi)
	movq	%rcx,8(%rdi)
	movq	%rdx,%rbp
	movq	0+0(%rsi),%r8
	movq	0+8(%rsi),%r9
	movq	0+16(%rsi),%r10
	movq	0+24(%rsi),%r11

	movq	%rbp,%rbx
	sarq	$63,%rbp
	xorq	%rax,%rax
	subq	%rbp,%rax

	xorq	%rbp,%rbx
	addq	%rax,%rbx

	xorq	%rbp,%r8
	xorq	%rbp,%r9
	xorq	%rbp,%r10
	xorq	%rbp,%r11
	addq	%r8,%rax
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11

	mulq	%rbx
	movq	%rax,%r8
	movq	%r9,%rax
	andq	%rbx,%rbp
	negq	%rbp
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
	adcq	%rdx,%rbp
	movq	32+0(%rsi),%r12
	movq	32+8(%rsi),%r13
	movq	32+16(%rsi),%r14
	movq	32+24(%rsi),%r15

	movq	%rcx,%rbx
	sarq	$63,%rcx
	xorq	%rax,%rax
	subq	%rcx,%rax

	xorq	%rcx,%rbx
	addq	%rax,%rbx

	xorq	%rcx,%r12
	xorq	%rcx,%r13
	xorq	%rcx,%r14
	xorq	%rcx,%r15
	addq	%r12,%rax
	adcq	$0,%r13
	adcq	$0,%r14
	adcq	$0,%r15

	mulq	%rbx
	movq	%rax,%r12
	movq	%r13,%rax
	andq	%rbx,%rcx
	negq	%rcx
	movq	%rdx,%r13
	mulq	%rbx
	addq	%rax,%r13
	movq	%r14,%rax
	adcq	$0,%rdx
	movq	%rdx,%r14
	mulq	%rbx
	addq	%rax,%r14
	movq	%r15,%rax
	adcq	$0,%rdx
	movq	%rdx,%r15
	mulq	%rbx
	addq	%rax,%r15
	adcq	%rdx,%rcx
	addq	%r12,%r8
	adcq	%r13,%r9
	adcq	%r14,%r10
	adcq	%r15,%r11
	adcq	%rcx,%rbp

	movq	0(%rdi),%rdx
	movq	8(%rdi),%rcx

	shrdq	$31,%r9,%r8
	shrdq	$31,%r10,%r9
	shrdq	$31,%r11,%r10
	shrdq	$31,%rbp,%r11

	sarq	$63,%rbp
	xorq	%rax,%rax
	subq	%rbp,%rax

	xorq	%rbp,%r8
	xorq	%rbp,%r9
	xorq	%rbp,%r10
	xorq	%rbp,%r11
	addq	%rax,%r8
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)

	xorq	%rbp,%rdx
	xorq	%rbp,%rcx
	addq	%rax,%rdx
	addq	%rax,%rcx

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%r8
	lfence
	jmpq	*%r8
	ud2
#else
	.byte	0xf3,0xc3
#endif

.def	__ab_approximation_31_256;	.scl 3;	.type 32;	.endef
.p2align	5
__ab_approximation_31_256:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	24(%rsi),%r9
	movq	56(%rsi),%r11
	movq	16(%rsi),%rbx
	movq	48(%rsi),%rbp
	movq	8(%rsi),%r8
	movq	40(%rsi),%r10

	movq	%r9,%rax
	orq	%r11,%rax
	cmovzq	%rbx,%r9
	cmovzq	%rbp,%r11
	cmovzq	%r8,%rbx
	movq	0(%rsi),%r8
	cmovzq	%r10,%rbp
	movq	32(%rsi),%r10

	movq	%r9,%rax
	orq	%r11,%rax
	cmovzq	%rbx,%r9
	cmovzq	%rbp,%r11
	cmovzq	%r8,%rbx
	cmovzq	%r10,%rbp

	movq	%r9,%rax
	orq	%r11,%rax
	bsrq	%rax,%rcx
	leaq	1(%rcx),%rcx
	cmovzq	%r8,%r9
	cmovzq	%r10,%r11
	cmovzq	%rax,%rcx
	negq	%rcx


	shldq	%cl,%rbx,%r9
	shldq	%cl,%rbp,%r11

	movl	$0x7FFFFFFF,%eax
	andq	%rax,%r8
	andq	%rax,%r10
	notq	%rax
	andq	%rax,%r9
	andq	%rax,%r11
	orq	%r9,%r8
	orq	%r11,%r10

	jmp	__inner_loop_31_256

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif

.def	__inner_loop_31_256;	.scl 3;	.type 32;	.endef
.p2align	5
__inner_loop_31_256:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	$0x7FFFFFFF80000000,%rcx
	movq	$0x800000007FFFFFFF,%r13
	movq	$0x7FFFFFFF7FFFFFFF,%r15

.Loop_31_256:
	cmpq	%r10,%r8
	movq	%r8,%rax
	movq	%r10,%rbx
	movq	%rcx,%rbp
	movq	%r13,%r14
	cmovbq	%r10,%r8
	cmovbq	%rax,%r10
	cmovbq	%r13,%rcx
	cmovbq	%rbp,%r13

	subq	%r10,%r8
	subq	%r13,%rcx
	addq	%r15,%rcx

	testq	$1,%rax
	cmovzq	%rax,%r8
	cmovzq	%rbx,%r10
	cmovzq	%rbp,%rcx
	cmovzq	%r14,%r13

	shrq	$1,%r8
	addq	%r13,%r13
	subq	%r15,%r13
	subl	$1,%edx
	jnz	.Loop_31_256

	shrq	$32,%r15
	movl	%ecx,%edx
	movl	%r13d,%r12d
	shrq	$32,%rcx
	shrq	$32,%r13
	subq	%r15,%rdx
	subq	%r15,%rcx
	subq	%r15,%r12
	subq	%r15,%r13

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%r8
	lfence
	jmpq	*%r8
	ud2
#else
	.byte	0xf3,0xc3
#endif


.def	__inner_loop_62_256;	.scl 3;	.type 32;	.endef
.p2align	5
__inner_loop_62_256:
	.byte	0xf3,0x0f,0x1e,0xfa

	movl	%edx,%r15d
	movq	$1,%rdx
	xorq	%rcx,%rcx
	xorq	%r12,%r12
	movq	%rdx,%r13
	movq	%rdx,%r14

.Loop_62_256:
	xorq	%rax,%rax
	testq	%r14,%r8
	movq	%r10,%rbx
	cmovnzq	%r10,%rax
	subq	%r8,%rbx
	movq	%r8,%rbp
	subq	%rax,%r8
	cmovcq	%rbx,%r8
	cmovcq	%rbp,%r10
	movq	%rdx,%rax
	cmovcq	%r12,%rdx
	cmovcq	%rax,%r12
	movq	%rcx,%rbx
	cmovcq	%r13,%rcx
	cmovcq	%rbx,%r13
	xorq	%rax,%rax
	xorq	%rbx,%rbx
	shrq	$1,%r8
	testq	%r14,%rbp
	cmovnzq	%r12,%rax
	cmovnzq	%r13,%rbx
	addq	%r12,%r12
	addq	%r13,%r13
	subq	%rax,%rdx
	subq	%rbx,%rcx
	subl	$1,%r15d
	jnz	.Loop_62_256

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%r8
	lfence
	jmpq	*%r8
	ud2
#else
	.byte	0xf3,0xc3
#endif

.section	.pdata
.p2align	2
.rva	.LSEH_begin_ct_inverse_mod_256
.rva	.LSEH_body_ct_inverse_mod_256
.rva	.LSEH_info_ct_inverse_mod_256_prologue

.rva	.LSEH_body_ct_inverse_mod_256
.rva	.LSEH_epilogue_ct_inverse_mod_256
.rva	.LSEH_info_ct_inverse_mod_256_body

.rva	.LSEH_epilogue_ct_inverse_mod_256
.rva	.LSEH_end_ct_inverse_mod_256
.rva	.LSEH_info_ct_inverse_mod_256_epilogue

.section	.xdata
.p2align	3
.LSEH_info_ct_inverse_mod_256_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_ct_inverse_mod_256_body:
.byte	1,0,18,0
.byte	0x00,0xf4,0x86,0x00
.byte	0x00,0xe4,0x87,0x00
.byte	0x00,0xd4,0x88,0x00
.byte	0x00,0xc4,0x89,0x00
.byte	0x00,0x34,0x8a,0x00
.byte	0x00,0x54,0x8b,0x00
.byte	0x00,0x74,0x8d,0x00
.byte	0x00,0x64,0x8e,0x00
.byte	0x00,0x01,0x8c,0x00
.byte	0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_ct_inverse_mod_256_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

