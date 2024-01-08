.text	

.globl	ctx_inverse_mod_383
.type	ctx_inverse_mod_383,@function
.align	32
ctx_inverse_mod_383:
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
	subq	$1112,%rsp
.cfi_adjust_cfa_offset	1112


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


	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31


	movq	%rdx,96(%rdi)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31


	movq	%rdx,96(%rdi)


	xorq	$256,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31



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
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383x63
	sarq	$63,%r13
	movq	%r13,48(%rdi)
	movq	%r13,56(%rdi)
	movq	%r13,64(%rdi)
	movq	%r13,72(%rdi)
	movq	%r13,80(%rdi)
	movq	%r13,88(%rdi)
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_767x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_767x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_767x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_767x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_767x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_767x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_383_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_767x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_191_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_191_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_767x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_191_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_191_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_767x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_191_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_191_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_767x63
	xorq	$256+96,%rsi
	movl	$31,%edi
	call	__ab_approximation_31


	movq	%r12,72(%rsp)
	movq	%r13,80(%rsp)

	movq	$256,%rdi
	xorq	%rsi,%rdi
	call	__smulx_191_n_shift_by_31
	movq	%rdx,56(%rsp)
	movq	%rcx,64(%rsp)

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_191_n_shift_by_31
	movq	%rdx,72(%rsp)
	movq	%rcx,80(%rsp)

	movq	56(%rsp),%rdx
	movq	64(%rsp),%rcx
	leaq	96(%rsi),%rsi
	leaq	48(%rdi),%rdi
	call	__smulx_383x63

	movq	72(%rsp),%rdx
	movq	80(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__smulx_767x63

	xorq	$256+96,%rsi
	movl	$53,%edi

	movq	0(%rsi),%r8

	movq	48(%rsi),%r10

	call	__inner_loop_62







	leaq	96(%rsi),%rsi





	movq	%r12,%rdx
	movq	%r13,%rcx
	movq	32(%rsp),%rdi
	call	__smulx_767x63

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
.cfi_adjust_cfa_offset	-1112-8*6

	.byte	0xf3,0xc3
.cfi_endproc	
.size	ctx_inverse_mod_383,.-ctx_inverse_mod_383
.type	__smulx_767x63,@function
.align	32
__smulx_767x63:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	movq	%rdx,%rax
	sarq	$63,%rax
	xorq	%rbp,%rbp
	subq	%rax,%rbp

	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	leaq	48(%rsi),%rsi

	xorq	%rax,%rdx
	addq	%rbp,%rdx

	xorq	%rax,%r8
	xorq	%rax,%r9
	xorq	%rax,%r10
	xorq	%rax,%r11
	xorq	%rax,%r12
	xorq	%r13,%rax
	addq	%rbp,%r8
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%rax

	mulxq	%r8,%r8,%rbp
	mulxq	%r9,%r9,%r13
	addq	%rbp,%r9
	mulxq	%r10,%r10,%rbp
	adcq	%r13,%r10
	mulxq	%r11,%r11,%r13
	adcq	%rbp,%r11
	mulxq	%r12,%r12,%rbp
	adcq	%r13,%r12
	adcq	$0,%rbp
	imulq	%rdx
	addq	%rbp,%rax
	adcq	$0,%rdx

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%rax,40(%rdi)
	movq	%rdx,48(%rdi)
	sarq	$63,%rdx
	movq	%rdx,56(%rdi)
	movq	%rcx,%rdx
	movq	%rcx,%rax

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

	sarq	$63,%rax
	xorq	%rsi,%rsi
	subq	%rax,%rsi

	xorq	%rax,%rdx
	addq	%rsi,%rdx

	xorq	%rax,%r8
	xorq	%rax,%r9
	xorq	%rax,%r10
	xorq	%rax,%r11
	xorq	%rax,%r12
	xorq	%rax,%r13
	xorq	%rax,%r14
	xorq	%rax,%r15
	xorq	%rax,%rbx
	xorq	%rax,%rbp
	xorq	%rax,%rcx
	xorq	%rax,%rdi
	addq	%rsi,%r8
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

	mulxq	%r8,%r8,%rax
	mulxq	%r9,%r9,%rsi
	addq	%rax,%r9
	mulxq	%r10,%r10,%rax
	adcq	%rsi,%r10
	mulxq	%r11,%r11,%rsi
	adcq	%rax,%r11
	mulxq	%r12,%r12,%rax
	adcq	%rsi,%r12
	mulxq	%r13,%r13,%rsi
	adcq	%rax,%r13
	mulxq	%r14,%r14,%rax
	adcq	%rsi,%r14
	mulxq	%r15,%r15,%rsi
	adcq	%rax,%r15
	mulxq	%rbx,%rbx,%rax
	adcq	%rsi,%rbx
	mulxq	%rbp,%rbp,%rsi
	adcq	%rax,%rbp
	mulxq	%rcx,%rcx,%rax
	adcq	%rsi,%rcx
	mulxq	%rdi,%rdi,%rsi
	movq	8(%rsp),%rdx
	movq	16(%rsp),%rsi
	adcq	%rdi,%rax

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
.cfi_endproc
.size	__smulx_767x63,.-__smulx_767x63
.type	__smulx_383x63,@function
.align	32
__smulx_383x63:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0+0(%rsi),%r8
	movq	0+8(%rsi),%r9
	movq	0+16(%rsi),%r10
	movq	0+24(%rsi),%r11
	movq	0+32(%rsi),%r12
	movq	0+40(%rsi),%r13

	movq	%rdx,%rbp
	sarq	$63,%rbp
	xorq	%rax,%rax
	subq	%rbp,%rax

	xorq	%rbp,%rdx
	addq	%rax,%rdx

	xorq	%rbp,%r8
	xorq	%rbp,%r9
	xorq	%rbp,%r10
	xorq	%rbp,%r11
	xorq	%rbp,%r12
	xorq	%rbp,%r13
	addq	%rax,%r8
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%r13

	mulxq	%r8,%r8,%rbp
	mulxq	%r9,%r9,%rax
	addq	%rbp,%r9
	mulxq	%r10,%r10,%rbp
	adcq	%rax,%r10
	mulxq	%r11,%r11,%rax
	adcq	%rbp,%r11
	mulxq	%r12,%r12,%rbp
	adcq	%rax,%r12
	mulxq	%r13,%r13,%rax
	movq	%rcx,%rdx
	adcq	%rbp,%r13

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%r13,40(%rdi)
	movq	48+0(%rsi),%r8
	movq	48+8(%rsi),%r9
	movq	48+16(%rsi),%r10
	movq	48+24(%rsi),%r11
	movq	48+32(%rsi),%r12
	movq	48+40(%rsi),%r13

	movq	%rdx,%rbp
	sarq	$63,%rbp
	xorq	%rax,%rax
	subq	%rbp,%rax

	xorq	%rbp,%rdx
	addq	%rax,%rdx

	xorq	%rbp,%r8
	xorq	%rbp,%r9
	xorq	%rbp,%r10
	xorq	%rbp,%r11
	xorq	%rbp,%r12
	xorq	%rbp,%r13
	addq	%rax,%r8
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%r13

	mulxq	%r8,%r8,%rbp
	mulxq	%r9,%r9,%rax
	addq	%rbp,%r9
	mulxq	%r10,%r10,%rbp
	adcq	%rax,%r10
	mulxq	%r11,%r11,%rax
	adcq	%rbp,%r11
	mulxq	%r12,%r12,%rbp
	adcq	%rax,%r12
	mulxq	%r13,%r13,%rax
	adcq	%rbp,%r13

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
.cfi_endproc
.size	__smulx_383x63,.-__smulx_383x63
.type	__smulx_383_n_shift_by_31,@function
.align	32
__smulx_383_n_shift_by_31:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	%rdx,%rbx
	xorq	%r14,%r14
	movq	0+0(%rsi),%r8
	movq	0+8(%rsi),%r9
	movq	0+16(%rsi),%r10
	movq	0+24(%rsi),%r11
	movq	0+32(%rsi),%r12
	movq	0+40(%rsi),%r13

	movq	%rdx,%rax
	sarq	$63,%rax
	xorq	%rbp,%rbp
	subq	%rax,%rbp

	xorq	%rax,%rdx
	addq	%rbp,%rdx

	xorq	%rax,%r8
	xorq	%rax,%r9
	xorq	%rax,%r10
	xorq	%rax,%r11
	xorq	%rax,%r12
	xorq	%r13,%rax
	addq	%rbp,%r8
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%rax

	mulxq	%r8,%r8,%rbp
	mulxq	%r9,%r9,%r13
	addq	%rbp,%r9
	mulxq	%r10,%r10,%rbp
	adcq	%r13,%r10
	mulxq	%r11,%r11,%r13
	adcq	%rbp,%r11
	mulxq	%r12,%r12,%rbp
	adcq	%r13,%r12
	adcq	$0,%rbp
	imulq	%rdx
	addq	%rbp,%rax
	adcq	%rdx,%r14

	movq	%rcx,%rdx

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%rax,40(%rdi)
	movq	48+0(%rsi),%r8
	movq	48+8(%rsi),%r9
	movq	48+16(%rsi),%r10
	movq	48+24(%rsi),%r11
	movq	48+32(%rsi),%r12
	movq	48+40(%rsi),%r13

	movq	%rdx,%rax
	sarq	$63,%rax
	xorq	%rbp,%rbp
	subq	%rax,%rbp

	xorq	%rax,%rdx
	addq	%rbp,%rdx

	xorq	%rax,%r8
	xorq	%rax,%r9
	xorq	%rax,%r10
	xorq	%rax,%r11
	xorq	%rax,%r12
	xorq	%r13,%rax
	addq	%rbp,%r8
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%rax

	mulxq	%r8,%r8,%rbp
	mulxq	%r9,%r9,%r13
	addq	%rbp,%r9
	mulxq	%r10,%r10,%rbp
	adcq	%r13,%r10
	mulxq	%r11,%r11,%r13
	adcq	%rbp,%r11
	mulxq	%r12,%r12,%rbp
	adcq	%r13,%r12
	adcq	$0,%rbp
	imulq	%rdx
	addq	%rbp,%rax
	adcq	$0,%rdx

	addq	0(%rdi),%r8
	adcq	8(%rdi),%r9
	adcq	16(%rdi),%r10
	adcq	24(%rdi),%r11
	adcq	32(%rdi),%r12
	adcq	40(%rdi),%rax
	adcq	%rdx,%r14
	movq	%rbx,%rdx

	shrdq	$31,%r9,%r8
	shrdq	$31,%r10,%r9
	shrdq	$31,%r11,%r10
	shrdq	$31,%r12,%r11
	shrdq	$31,%rax,%r12
	shrdq	$31,%r14,%rax

	sarq	$63,%r14
	xorq	%rbp,%rbp
	subq	%r14,%rbp

	xorq	%r14,%r8
	xorq	%r14,%r9
	xorq	%r14,%r10
	xorq	%r14,%r11
	xorq	%r14,%r12
	xorq	%r14,%rax
	addq	%rbp,%r8
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%rax

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%rax,40(%rdi)

	xorq	%r14,%rdx
	xorq	%r14,%rcx
	addq	%rbp,%rdx
	addq	%rbp,%rcx

	.byte	0xf3,0xc3
.cfi_endproc
.size	__smulx_383_n_shift_by_31,.-__smulx_383_n_shift_by_31
.type	__smulx_191_n_shift_by_31,@function
.align	32
__smulx_191_n_shift_by_31:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	%rdx,%rbx
	movq	0+0(%rsi),%r8
	movq	0+8(%rsi),%r9
	movq	0+16(%rsi),%r10

	movq	%rdx,%rax
	sarq	$63,%rax
	xorq	%rbp,%rbp
	subq	%rax,%rbp

	xorq	%rax,%rdx
	addq	%rbp,%rdx

	xorq	%rax,%r8
	xorq	%rax,%r9
	xorq	%r10,%rax
	addq	%rbp,%r8
	adcq	$0,%r9
	adcq	$0,%rax

	mulxq	%r8,%r8,%rbp
	mulxq	%r9,%r9,%r10
	addq	%rbp,%r9
	adcq	$0,%r10
	imulq	%rdx
	addq	%rax,%r10
	adcq	$0,%rdx
	movq	%rdx,%r14
	movq	%rcx,%rdx
	movq	48+0(%rsi),%r11
	movq	48+8(%rsi),%r12
	movq	48+16(%rsi),%r13

	movq	%rdx,%rax
	sarq	$63,%rax
	xorq	%rbp,%rbp
	subq	%rax,%rbp

	xorq	%rax,%rdx
	addq	%rbp,%rdx

	xorq	%rax,%r11
	xorq	%rax,%r12
	xorq	%r13,%rax
	addq	%rbp,%r11
	adcq	$0,%r12
	adcq	$0,%rax

	mulxq	%r11,%r11,%rbp
	mulxq	%r12,%r12,%r13
	addq	%rbp,%r12
	adcq	$0,%r13
	imulq	%rdx
	addq	%rax,%r13
	adcq	$0,%rdx
	addq	%r8,%r11
	adcq	%r9,%r12
	adcq	%r10,%r13
	adcq	%rdx,%r14
	movq	%rbx,%rdx

	shrdq	$31,%r12,%r11
	shrdq	$31,%r13,%r12
	shrdq	$31,%r14,%r13

	sarq	$63,%r14
	xorq	%rbp,%rbp
	subq	%r14,%rbp

	xorq	%r14,%r11
	xorq	%r14,%r12
	xorq	%r14,%r13
	addq	%rbp,%r11
	adcq	$0,%r12
	adcq	$0,%r13

	movq	%r11,0(%rdi)
	movq	%r12,8(%rdi)
	movq	%r13,16(%rdi)

	xorq	%r14,%rdx
	xorq	%r14,%rcx
	addq	%rbp,%rdx
	addq	%rbp,%rcx

	.byte	0xf3,0xc3
.cfi_endproc
.size	__smulx_191_n_shift_by_31,.-__smulx_191_n_shift_by_31
.type	__ab_approximation_31,@function
.align	32
__ab_approximation_31:
.cfi_startproc
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
	movq	16(%rsi),%r8
	cmovzq	%r10,%rbp
	movq	64(%rsi),%r10

	movq	%r9,%rax
	orq	%r11,%rax
	cmovzq	%rbx,%r9
	cmovzq	%rbp,%r11
	cmovzq	%r8,%rbx
	movq	8(%rsi),%r8
	cmovzq	%r10,%rbp
	movq	56(%rsi),%r10

	movq	%r9,%rax
	orq	%r11,%rax
	cmovzq	%rbx,%r9
	cmovzq	%rbp,%r11
	cmovzq	%r8,%rbx
	movq	0(%rsi),%r8
	cmovzq	%r10,%rbp
	movq	48(%rsi),%r10

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
	andnq	%r9,%rax,%r9
	andnq	%r11,%rax,%r11
	orq	%r9,%r8
	orq	%r11,%r10

	jmp	__inner_loop_31

	.byte	0xf3,0xc3
.cfi_endproc
.size	__ab_approximation_31,.-__ab_approximation_31
.type	__inner_loop_31,@function
.align	32
__inner_loop_31:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	$0x7FFFFFFF80000000,%rcx
	movq	$0x800000007FFFFFFF,%r13
	movq	$0x7FFFFFFF7FFFFFFF,%r15

.Loop_31:
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
	subl	$1,%edi
	jnz	.Loop_31

	shrq	$32,%r15
	movl	%ecx,%edx
	movl	%r13d,%r12d
	shrq	$32,%rcx
	shrq	$32,%r13
	subq	%r15,%rdx
	subq	%r15,%rcx
	subq	%r15,%r12
	subq	%r15,%r13

	.byte	0xf3,0xc3
.cfi_endproc
.size	__inner_loop_31,.-__inner_loop_31

.type	__inner_loop_62,@function
.align	32
__inner_loop_62:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	$1,%rdx
	xorq	%rcx,%rcx
	xorq	%r12,%r12
	movq	$1,%r13

.Loop_62:
	xorq	%rax,%rax
	testq	$1,%r8
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
	testq	$1,%rbp
	cmovnzq	%r12,%rax
	cmovnzq	%r13,%rbx
	addq	%r12,%r12
	addq	%r13,%r13
	subq	%rax,%rdx
	subq	%rbx,%rcx
	subl	$1,%edi
	jnz	.Loop_62

	.byte	0xf3,0xc3
.cfi_endproc
.size	__inner_loop_62,.-__inner_loop_62

.section	.note.GNU-stack,"",@progbits
.section	.note.gnu.property,"a",@note
	.long	4,2f-1f,5
	.byte	0x47,0x4E,0x55,0
1:	.long	0xc0000002,4,3
.align	8
2:
