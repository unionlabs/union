.text	

.globl	ct_inverse_mod_383
.def	ct_inverse_mod_383;	.scl 2;	.type 32;	.endef
.p2align	5
ct_inverse_mod_383:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_ct_inverse_mod_383:
	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx


	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$1112,%rsp

.LSEH_body_ct_inverse_mod_383:


	leaq	88+511(%rsp),%rax
	andq	$-512,%rax
	movq	%rdi,32(%rsp)
	movq	%rcx,40(%rsp)

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	movq	0(%rdx),%r14
	movq	8(%rdx),%r15
	movq	16(%rdx),%rbx
	movq	24(%rdx),%rbp
	movq	32(%rdx),%rsi
	movq	40(%rdx),%rdi

	movq	%r8,0(%rax)
	movq	%r9,8(%rax)
	movq	%r10,16(%rax)
	movq	%r11,24(%rax)
	movq	%r12,32(%rax)
	movq	%r13,40(%rax)

	movq	%r14,48(%rax)
	movq	%r15,56(%rax)
	movq	%rbx,64(%rax)
	movq	%rbp,72(%rax)
	movq	%rsi,80(%rax)
	movq	%rax,%rsi
	movq	%rdi,88(%rax)


	movl	$62,%edi
	call	__ab_approximation_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_383_n_shift_by_62


	movq	%rdx,96(%rdi)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383_n_shift_by_62


	movq	%rdx,96(%rdi)


	xorq	$256,%rsi
	movl	$62,%edi
	call	__ab_approximation_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383_n_shift_by_62



	movq	96(%rsi),%rax
	movq	144(%rsi),%r11
	movq	%rdx,%rbx
	movq	%rax,%r10
	imulq	56(%rsp)
	movq	%rax,%r8
	movq	%r11,%rax
	movq	%rdx,%r9
	imulq	64(%rsp)
	addq	%rax,%r8
	adcq	%rdx,%r9
	movq	%r8,48(%rdi)
	movq	%r9,56(%rdi)
	sarq	$63,%r9
	movq	%r9,64(%rdi)
	movq	%r9,72(%rdi)
	movq	%r9,80(%rdi)
	movq	%r9,88(%rdi)
	leaq	96(%rsi),%rsi

	movq	%r10,%rax
	imulq	%rbx
	movq	%rax,%r8
	movq	%r11,%rax
	movq	%rdx,%r9
	imulq	%rcx
	addq	%rax,%r8
	adcq	%rdx,%r9
	movq	%r8,96(%rdi)
	movq	%r9,104(%rdi)
	sarq	$63,%r9
	movq	%r9,112(%rdi)
	movq	%r9,120(%rdi)
	movq	%r9,128(%rdi)
	movq	%r9,136(%rdi)
	xorq	$256+96,%rsi
	movl	$62,%edi
	call	__ab_approximation_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulq_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383x63
	xorq	$256+96,%rsi
	movl	$62,%edi
	call	__ab_approximation_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulq_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383x63
	xorq	$256+96,%rsi
	movl	$62,%edi
	call	__ab_approximation_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulq_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383x63
	xorq	$256+96,%rsi
	movl	$62,%edi
	call	__ab_approximation_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulq_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383x63
	sarq	$63,%r13
	movq	%r13,48(%rdi)
	movq	%r13,56(%rdi)
	movq	%r13,64(%rdi)
	movq	%r13,72(%rdi)
	movq	%r13,80(%rdi)
	movq	%r13,88(%rdi)
	xorq	$256+96,%rsi
	movl	$62,%edi
	call	__ab_approximation_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulq_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_767x63
	xorq	$256+96,%rsi
	movl	$62,%edi
	call	__ab_approximation_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulq_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_767x63
	xorq	$256+96,%rsi
	movl	$62,%edi
	call	__ab_approximation_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulq_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_767x63
	xorq	$256+96,%rsi
	movl	$62,%edi
	call	__ab_approximation_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulq_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_767x63
	xorq	$256+96,%rsi
	movl	$62,%edi
	call	__ab_approximation_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_383_n_shift_by_62
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulq_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_767x63

	xorq	$256+96,%rsi
	movl	$62,%edi

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	48(%rsi),%r10
	movq	56(%rsi),%r11
	call	__inner_loop_62


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	movq	%r8,0(%rdi)
	movq	%r10,48(%rdi)



	leaq	96(%rsi),%rsi
	leaq	96(%rdi),%rdi
	call	__smulq_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulq_767x63


	xorq	$256+96,%rsi
	movl	$22,%edi

	movq	0(%rsi),%r8
	xorq	%r9,%r9
	movq	48(%rsi),%r10
	xorq	%r11,%r11
	call	__inner_loop_62







	leaq	96(%rsi),%rsi





	movq	%r12,%rdx
	movq	%r13,%rcx
	movq	32(%rsp),%rdi
	call	__smulq_767x63

	movq	40(%rsp),%rsi
	movq	%rax,%rdx
	sarq	$63,%rax

	movq	%rax,%r8
	movq	%rax,%r9
	movq	%rax,%r10
	andq	0(%rsi),%r8
	andq	8(%rsi),%r9
	movq	%rax,%r11
	andq	16(%rsi),%r10
	andq	24(%rsi),%r11
	movq	%rax,%r12
	andq	32(%rsi),%r12
	andq	40(%rsi),%rax

	addq	%r8,%r14
	adcq	%r9,%r15
	adcq	%r10,%rbx
	adcq	%r11,%rbp
	adcq	%r12,%rcx
	adcq	%rax,%rdx

	movq	%r14,48(%rdi)
	movq	%r15,56(%rdi)
	movq	%rbx,64(%rdi)
	movq	%rbp,72(%rdi)
	movq	%rcx,80(%rdi)
	movq	%rdx,88(%rdi)

	leaq	1112(%rsp),%r8
	movq	0(%r8),%r15

	movq	8(%r8),%r14

	movq	16(%r8),%r13

	movq	24(%r8),%r12

	movq	32(%r8),%rbx

	movq	40(%r8),%rbp

	leaq	48(%r8),%rsp

.LSEH_epilogue_ct_inverse_mod_383:
	mov	8(%rsp),%rdi
	mov	16(%rsp),%rsi

	.byte	0xf3,0xc3

.LSEH_end_ct_inverse_mod_383:
.def	__smulq_767x63;	.scl 3;	.type 32;	.endef
.p2align	5
__smulq_767x63:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	movq	%rdx,%rbp
	sarq	$63,%rdx
	xorq	%rax,%rax
	subq	%rdx,%rax

	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	leaq	48(%rsi),%rsi

	xorq	%rdx,%rbp
	addq	%rax,%rbp

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

	mulq	%rbp
	movq	%rax,0(%rdi)
	movq	%r9,%rax
	movq	%rdx,%r9
	mulq	%rbp
	addq	%rax,%r9
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10
	movq	%r9,8(%rdi)
	mulq	%rbp
	addq	%rax,%r10
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11
	movq	%r10,16(%rdi)
	mulq	%rbp
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	movq	%r11,24(%rdi)
	mulq	%rbp
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13
	movq	%r12,32(%rdi)
	imulq	%rbp
	addq	%rax,%r13
	adcq	$0,%rdx

	movq	%r13,40(%rdi)
	movq	%rdx,48(%rdi)
	sarq	$63,%rdx
	movq	%rdx,56(%rdi)
	movq	%rcx,%rdx

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13
	movq	48(%rsi),%r14
	movq	56(%rsi),%r15
	movq	64(%rsi),%rbx
	movq	72(%rsi),%rbp
	movq	80(%rsi),%rcx
	movq	88(%rsi),%rdi

	movq	%rdx,%rsi
	sarq	$63,%rdx
	xorq	%rax,%rax
	subq	%rdx,%rax

	xorq	%rdx,%rsi
	addq	%rax,%rsi

	xorq	%rdx,%r8
	xorq	%rdx,%r9
	xorq	%rdx,%r10
	xorq	%rdx,%r11
	xorq	%rdx,%r12
	xorq	%rdx,%r13
	xorq	%rdx,%r14
	xorq	%rdx,%r15
	xorq	%rdx,%rbx
	xorq	%rdx,%rbp
	xorq	%rdx,%rcx
	xorq	%rdx,%rdi
	addq	%r8,%rax
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%r13
	adcq	$0,%r14
	adcq	$0,%r15
	adcq	$0,%rbx
	adcq	$0,%rbp
	adcq	$0,%rcx
	adcq	$0,%rdi

	mulq	%rsi
	movq	%rax,%r8
	movq	%r9,%rax
	movq	%rdx,%r9
	mulq	%rsi
	addq	%rax,%r9
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10
	mulq	%rsi
	addq	%rax,%r10
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11
	mulq	%rsi
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	mulq	%rsi
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13
	mulq	%rsi
	addq	%rax,%r13
	movq	%r14,%rax
	adcq	$0,%rdx
	movq	%rdx,%r14
	mulq	%rsi
	addq	%rax,%r14
	movq	%r15,%rax
	adcq	$0,%rdx
	movq	%rdx,%r15
	mulq	%rsi
	addq	%rax,%r15
	movq	%rbx,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbx
	mulq	%rsi
	addq	%rax,%rbx
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbp
	mulq	%rsi
	addq	%rax,%rbp
	movq	%rcx,%rax
	adcq	$0,%rdx
	movq	%rdx,%rcx
	mulq	%rsi
	addq	%rax,%rcx
	movq	%rdi,%rax
	adcq	$0,%rdx
	movq	%rdx,%rdi
	movq	8(%rsp),%rdx
	imulq	%rsi,%rax
	movq	16(%rsp),%rsi
	addq	%rdi,%rax

	addq	0(%rdx),%r8
	adcq	8(%rdx),%r9
	adcq	16(%rdx),%r10
	adcq	24(%rdx),%r11
	adcq	32(%rdx),%r12
	adcq	40(%rdx),%r13
	adcq	48(%rdx),%r14
	movq	56(%rdx),%rdi
	adcq	%rdi,%r15
	adcq	%rdi,%rbx
	adcq	%rdi,%rbp
	adcq	%rdi,%rcx
	adcq	%rdi,%rax

	movq	%rdx,%rdi

	movq	%r8,0(%rdx)
	movq	%r9,8(%rdx)
	movq	%r10,16(%rdx)
	movq	%r11,24(%rdx)
	movq	%r12,32(%rdx)
	movq	%r13,40(%rdx)
	movq	%r14,48(%rdx)
	movq	%r15,56(%rdx)
	movq	%rbx,64(%rdx)
	movq	%rbp,72(%rdx)
	movq	%rcx,80(%rdx)
	movq	%rax,88(%rdx)

	.byte	0xf3,0xc3

.def	__smulq_383x63;	.scl 3;	.type 32;	.endef
.p2align	5
__smulq_383x63:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	movq	%rdx,%rbp
	sarq	$63,%rdx
	xorq	%rax,%rax
	subq	%rdx,%rax

	xorq	%rdx,%rbp
	addq	%rax,%rbp

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

	mulq	%rbp
	movq	%rax,%r8
	movq	%r9,%rax
	movq	%rdx,%r9
	mulq	%rbp
	addq	%rax,%r9
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10
	mulq	%rbp
	addq	%rax,%r10
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11
	mulq	%rbp
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	mulq	%rbp
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13
	imulq	%rbp,%rax
	addq	%rax,%r13

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

	movq	%rdx,%rbp
	sarq	$63,%rdx
	xorq	%rax,%rax
	subq	%rdx,%rax

	xorq	%rdx,%rbp
	addq	%rax,%rbp

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

	mulq	%rbp
	movq	%rax,%r8
	movq	%r9,%rax
	movq	%rdx,%r9
	mulq	%rbp
	addq	%rax,%r9
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10
	mulq	%rbp
	addq	%rax,%r10
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11
	mulq	%rbp
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	mulq	%rbp
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13
	imulq	%rbp,%rax
	addq	%rax,%r13

	leaq	-48(%rsi),%rsi

	addq	0(%rdi),%r8
	adcq	8(%rdi),%r9
	adcq	16(%rdi),%r10
	adcq	24(%rdi),%r11
	adcq	32(%rdi),%r12
	adcq	40(%rdi),%r13

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%r13,40(%rdi)

	.byte	0xf3,0xc3

.def	__smulq_383_n_shift_by_62;	.scl 3;	.type 32;	.endef
.p2align	5
__smulq_383_n_shift_by_62:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	%rdx,%rbx
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	movq	%rdx,%rbp
	sarq	$63,%rdx
	xorq	%rax,%rax
	subq	%rdx,%rax

	xorq	%rdx,%rbp
	addq	%rax,%rbp

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

	mulq	%rbp
	movq	%rax,%r8
	movq	%r9,%rax
	movq	%rdx,%r9
	mulq	%rbp
	addq	%rax,%r9
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10
	mulq	%rbp
	addq	%rax,%r10
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11
	mulq	%rbp
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	mulq	%rbp
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13
	imulq	%rbp
	addq	%rax,%r13
	adcq	$0,%rdx

	leaq	48(%rsi),%rsi
	movq	%rdx,%r14
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

	movq	%rdx,%rbp
	sarq	$63,%rdx
	xorq	%rax,%rax
	subq	%rdx,%rax

	xorq	%rdx,%rbp
	addq	%rax,%rbp

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

	mulq	%rbp
	movq	%rax,%r8
	movq	%r9,%rax
	movq	%rdx,%r9
	mulq	%rbp
	addq	%rax,%r9
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10
	mulq	%rbp
	addq	%rax,%r10
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11
	mulq	%rbp
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	mulq	%rbp
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13
	imulq	%rbp
	addq	%rax,%r13
	adcq	$0,%rdx

	leaq	-48(%rsi),%rsi

	addq	0(%rdi),%r8
	adcq	8(%rdi),%r9
	adcq	16(%rdi),%r10
	adcq	24(%rdi),%r11
	adcq	32(%rdi),%r12
	adcq	40(%rdi),%r13
	adcq	%rdx,%r14
	movq	%rbx,%rdx

	shrdq	$62,%r9,%r8
	shrdq	$62,%r10,%r9
	shrdq	$62,%r11,%r10
	shrdq	$62,%r12,%r11
	shrdq	$62,%r13,%r12
	shrdq	$62,%r14,%r13

	sarq	$63,%r14
	xorq	%rbp,%rbp
	subq	%r14,%rbp

	xorq	%r14,%r8
	xorq	%r14,%r9
	xorq	%r14,%r10
	xorq	%r14,%r11
	xorq	%r14,%r12
	xorq	%r14,%r13
	addq	%rbp,%r8
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

	xorq	%r14,%rdx
	xorq	%r14,%rcx
	addq	%rbp,%rdx
	addq	%rbp,%rcx

	.byte	0xf3,0xc3

.def	__ab_approximation_62;	.scl 3;	.type 32;	.endef
.p2align	5
__ab_approximation_62:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	40(%rsi),%r9
	movq	88(%rsi),%r11
	movq	32(%rsi),%rbx
	movq	80(%rsi),%rbp
	movq	24(%rsi),%r8
	movq	72(%rsi),%r10

	movq	%r9,%rax
	orq	%r11,%rax
	cmovzq	%rbx,%r9
	cmovzq	%rbp,%r11
	cmovzq	%r8,%rbx
	cmovzq	%r10,%rbp
	movq	16(%rsi),%r8
	movq	64(%rsi),%r10

	movq	%r9,%rax
	orq	%r11,%rax
	cmovzq	%rbx,%r9
	cmovzq	%rbp,%r11
	cmovzq	%r8,%rbx
	cmovzq	%r10,%rbp
	movq	8(%rsi),%r8
	movq	56(%rsi),%r10

	movq	%r9,%rax
	orq	%r11,%rax
	cmovzq	%rbx,%r9
	cmovzq	%rbp,%r11
	cmovzq	%r8,%rbx
	cmovzq	%r10,%rbp
	movq	0(%rsi),%r8
	movq	48(%rsi),%r10

	movq	%r9,%rax
	orq	%r11,%rax
	bsrq	%rax,%rcx
	leaq	1(%rcx),%rcx
	cmovzq	%rbx,%r9
	cmovzq	%rbp,%r11
	cmovzq	%rax,%rcx
	negq	%rcx


	shldq	%cl,%rbx,%r9
	shldq	%cl,%rbp,%r11

	jmp	__inner_loop_62

	.byte	0xf3,0xc3

.def	__inner_loop_62;	.scl 3;	.type 32;	.endef
.p2align	3
.long	0
__inner_loop_62:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	$1,%rdx
	xorq	%rcx,%rcx
	xorq	%r12,%r12
	movq	$1,%r13
	movq	%rsi,8(%rsp)

.Loop_62:
	xorq	%rax,%rax
	xorq	%rbx,%rbx
	testq	$1,%r8
	movq	%r10,%rbp
	movq	%r11,%r14
	cmovnzq	%r10,%rax
	cmovnzq	%r11,%rbx
	subq	%r8,%rbp
	sbbq	%r9,%r14
	movq	%r8,%r15
	movq	%r9,%rsi
	subq	%rax,%r8
	sbbq	%rbx,%r9
	cmovcq	%rbp,%r8
	cmovcq	%r14,%r9
	cmovcq	%r15,%r10
	cmovcq	%rsi,%r11
	movq	%rdx,%rax
	cmovcq	%r12,%rdx
	cmovcq	%rax,%r12
	movq	%rcx,%rbx
	cmovcq	%r13,%rcx
	cmovcq	%rbx,%r13
	xorq	%rax,%rax
	xorq	%rbx,%rbx
	shrdq	$1,%r9,%r8
	shrq	$1,%r9
	testq	$1,%r15
	cmovnzq	%r12,%rax
	cmovnzq	%r13,%rbx
	addq	%r12,%r12
	addq	%r13,%r13
	subq	%rax,%rdx
	subq	%rbx,%rcx
	subl	$1,%edi
	jnz	.Loop_62

	movq	8(%rsp),%rsi
	.byte	0xf3,0xc3

.section	.pdata
.p2align	2
.rva	.LSEH_begin_ct_inverse_mod_383
.rva	.LSEH_body_ct_inverse_mod_383
.rva	.LSEH_info_ct_inverse_mod_383_prologue

.rva	.LSEH_body_ct_inverse_mod_383
.rva	.LSEH_epilogue_ct_inverse_mod_383
.rva	.LSEH_info_ct_inverse_mod_383_body

.rva	.LSEH_epilogue_ct_inverse_mod_383
.rva	.LSEH_end_ct_inverse_mod_383
.rva	.LSEH_info_ct_inverse_mod_383_epilogue

.section	.xdata
.p2align	3
.LSEH_info_ct_inverse_mod_383_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0x03
.byte	0,0
.LSEH_info_ct_inverse_mod_383_body:
.byte	1,0,18,0
.byte	0x00,0xf4,0x8b,0x00
.byte	0x00,0xe4,0x8c,0x00
.byte	0x00,0xd4,0x8d,0x00
.byte	0x00,0xc4,0x8e,0x00
.byte	0x00,0x34,0x8f,0x00
.byte	0x00,0x54,0x90,0x00
.byte	0x00,0x74,0x92,0x00
.byte	0x00,0x64,0x93,0x00
.byte	0x00,0x01,0x91,0x00
.LSEH_info_ct_inverse_mod_383_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

