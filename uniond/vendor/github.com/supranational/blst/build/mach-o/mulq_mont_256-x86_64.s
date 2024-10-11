.comm	___blst_platform_cap,4
.text	

.globl	_mul_mont_sparse_256
.private_extern	_mul_mont_sparse_256

.p2align	5
_mul_mont_sparse_256:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa


#ifdef __BLST_PORTABLE__
	testl	$1,___blst_platform_cap(%rip)
	jnz	L$mul_mont_sparse_256$1
#endif
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
	pushq	%rdi
.cfi_adjust_cfa_offset	8


	movq	0(%rdx),%rax
	movq	0(%rsi),%r13
	movq	8(%rsi),%r14
	movq	16(%rsi),%r12
	movq	24(%rsi),%rbp
	movq	%rdx,%rbx

	movq	%rax,%r15
	mulq	%r13
	movq	%rax,%r9
	movq	%r15,%rax
	movq	%rdx,%r10
	call	__mulq_mont_sparse_256

	movq	8(%rsp),%r15
.cfi_restore	%r15
	movq	16(%rsp),%r14
.cfi_restore	%r14
	movq	24(%rsp),%r13
.cfi_restore	%r13
	movq	32(%rsp),%r12
.cfi_restore	%r12
	movq	40(%rsp),%rbx
.cfi_restore	%rbx
	movq	48(%rsp),%rbp
.cfi_restore	%rbp
	leaq	56(%rsp),%rsp
.cfi_adjust_cfa_offset	-56

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	


.globl	_sqr_mont_sparse_256
.private_extern	_sqr_mont_sparse_256

.p2align	5
_sqr_mont_sparse_256:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa


#ifdef __BLST_PORTABLE__
	testl	$1,___blst_platform_cap(%rip)
	jnz	L$sqr_mont_sparse_256$1
#endif
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
	pushq	%rdi
.cfi_adjust_cfa_offset	8


	movq	0(%rsi),%rax
	movq	%rcx,%r8
	movq	8(%rsi),%r14
	movq	%rdx,%rcx
	movq	16(%rsi),%r12
	leaq	(%rsi),%rbx
	movq	24(%rsi),%rbp

	movq	%rax,%r15
	mulq	%rax
	movq	%rax,%r9
	movq	%r15,%rax
	movq	%rdx,%r10
	call	__mulq_mont_sparse_256

	movq	8(%rsp),%r15
.cfi_restore	%r15
	movq	16(%rsp),%r14
.cfi_restore	%r14
	movq	24(%rsp),%r13
.cfi_restore	%r13
	movq	32(%rsp),%r12
.cfi_restore	%r12
	movq	40(%rsp),%rbx
.cfi_restore	%rbx
	movq	48(%rsp),%rbp
.cfi_restore	%rbp
	leaq	56(%rsp),%rsp
.cfi_adjust_cfa_offset	-56

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	


.p2align	5
__mulq_mont_sparse_256:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	mulq	%r14
	addq	%rax,%r10
	movq	%r15,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	%r12
	addq	%rax,%r11
	movq	%r15,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	%rbp
	addq	%rax,%r12
	movq	8(%rbx),%rax
	adcq	$0,%rdx
	xorq	%r14,%r14
	movq	%rdx,%r13

	movq	%r9,%rdi
	imulq	%r8,%r9


	movq	%rax,%r15
	mulq	0(%rsi)
	addq	%rax,%r10
	movq	%r15,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	8(%rsi)
	addq	%rax,%r11
	movq	%r15,%rax
	adcq	$0,%rdx
	addq	%rbp,%r11
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	16(%rsi)
	addq	%rax,%r12
	movq	%r15,%rax
	adcq	$0,%rdx
	addq	%rbp,%r12
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	24(%rsi)
	addq	%rax,%r13
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%rbp,%r13
	adcq	%rdx,%r14
	xorq	%r15,%r15


	mulq	0(%rcx)
	addq	%rax,%rdi
	movq	%r9,%rax
	adcq	%rdx,%rdi

	mulq	8(%rcx)
	addq	%rax,%r10
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%rdi,%r10
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	16(%rcx)
	addq	%rax,%r11
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%rbp,%r11
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	24(%rcx)
	addq	%rax,%r12
	movq	16(%rbx),%rax
	adcq	$0,%rdx
	addq	%rbp,%r12
	adcq	$0,%rdx
	addq	%rdx,%r13
	adcq	$0,%r14
	adcq	$0,%r15
	movq	%r10,%rdi
	imulq	%r8,%r10


	movq	%rax,%r9
	mulq	0(%rsi)
	addq	%rax,%r11
	movq	%r9,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	8(%rsi)
	addq	%rax,%r12
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%rbp,%r12
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	16(%rsi)
	addq	%rax,%r13
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%rbp,%r13
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	24(%rsi)
	addq	%rax,%r14
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%rbp,%r14
	adcq	%rdx,%r15
	xorq	%r9,%r9


	mulq	0(%rcx)
	addq	%rax,%rdi
	movq	%r10,%rax
	adcq	%rdx,%rdi

	mulq	8(%rcx)
	addq	%rax,%r11
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%rdi,%r11
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	16(%rcx)
	addq	%rax,%r12
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%rbp,%r12
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	24(%rcx)
	addq	%rax,%r13
	movq	24(%rbx),%rax
	adcq	$0,%rdx
	addq	%rbp,%r13
	adcq	$0,%rdx
	addq	%rdx,%r14
	adcq	$0,%r15
	adcq	$0,%r9
	movq	%r11,%rdi
	imulq	%r8,%r11


	movq	%rax,%r10
	mulq	0(%rsi)
	addq	%rax,%r12
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	8(%rsi)
	addq	%rax,%r13
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%rbp,%r13
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	16(%rsi)
	addq	%rax,%r14
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%rbp,%r14
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	24(%rsi)
	addq	%rax,%r15
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%rbp,%r15
	adcq	%rdx,%r9
	xorq	%r10,%r10


	mulq	0(%rcx)
	addq	%rax,%rdi
	movq	%r11,%rax
	adcq	%rdx,%rdi

	mulq	8(%rcx)
	addq	%rax,%r12
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%rdi,%r12
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	16(%rcx)
	addq	%rax,%r13
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%rbp,%r13
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	24(%rcx)
	addq	%rax,%r14
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%rbp,%r14
	adcq	$0,%rdx
	addq	%rdx,%r15
	adcq	$0,%r9
	adcq	$0,%r10
	imulq	%r8,%rax
	movq	8(%rsp),%rsi


	movq	%rax,%r11
	mulq	0(%rcx)
	addq	%rax,%r12
	movq	%r11,%rax
	adcq	%rdx,%r12

	mulq	8(%rcx)
	addq	%rax,%r13
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%r12,%r13
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	16(%rcx)
	addq	%rax,%r14
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%rbp,%r14
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	24(%rcx)
	movq	%r14,%rbx
	addq	%rbp,%r15
	adcq	$0,%rdx
	addq	%rax,%r15
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%rdx,%r9
	adcq	$0,%r10




	movq	%r15,%r12
	subq	0(%rcx),%r13
	sbbq	8(%rcx),%r14
	sbbq	16(%rcx),%r15
	movq	%r9,%rbp
	sbbq	24(%rcx),%r9
	sbbq	$0,%r10

	cmovcq	%rax,%r13
	cmovcq	%rbx,%r14
	cmovcq	%r12,%r15
	movq	%r13,0(%rsi)
	cmovcq	%rbp,%r9
	movq	%r14,8(%rsi)
	movq	%r15,16(%rsi)
	movq	%r9,24(%rsi)

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	

.globl	_from_mont_256
.private_extern	_from_mont_256

.p2align	5
_from_mont_256:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa


#ifdef __BLST_PORTABLE__
	testl	$1,___blst_platform_cap(%rip)
	jnz	L$from_mont_256$1
#endif
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
	subq	$8,%rsp
.cfi_adjust_cfa_offset	8


	movq	%rdx,%rbx
	call	__mulq_by_1_mont_256





	movq	%r14,%r10
	movq	%r15,%r11
	movq	%r9,%r12

	subq	0(%rbx),%r13
	sbbq	8(%rbx),%r14
	sbbq	16(%rbx),%r15
	sbbq	24(%rbx),%r9

	cmovncq	%r13,%rax
	cmovncq	%r14,%r10
	cmovncq	%r15,%r11
	movq	%rax,0(%rdi)
	cmovncq	%r9,%r12
	movq	%r10,8(%rdi)
	movq	%r11,16(%rdi)
	movq	%r12,24(%rdi)

	movq	8(%rsp),%r15
.cfi_restore	%r15
	movq	16(%rsp),%r14
.cfi_restore	%r14
	movq	24(%rsp),%r13
.cfi_restore	%r13
	movq	32(%rsp),%r12
.cfi_restore	%r12
	movq	40(%rsp),%rbx
.cfi_restore	%rbx
	movq	48(%rsp),%rbp
.cfi_restore	%rbp
	leaq	56(%rsp),%rsp
.cfi_adjust_cfa_offset	-56

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	


.globl	_redc_mont_256
.private_extern	_redc_mont_256

.p2align	5
_redc_mont_256:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa


#ifdef __BLST_PORTABLE__
	testl	$1,___blst_platform_cap(%rip)
	jnz	L$redc_mont_256$1
#endif
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
	subq	$8,%rsp
.cfi_adjust_cfa_offset	8


	movq	%rdx,%rbx
	call	__mulq_by_1_mont_256

	addq	32(%rsi),%r13
	adcq	40(%rsi),%r14
	movq	%r13,%rax
	adcq	48(%rsi),%r15
	movq	%r14,%r10
	adcq	56(%rsi),%r9
	sbbq	%rsi,%rsi




	movq	%r15,%r11
	subq	0(%rbx),%r13
	sbbq	8(%rbx),%r14
	sbbq	16(%rbx),%r15
	movq	%r9,%r12
	sbbq	24(%rbx),%r9
	sbbq	$0,%rsi

	cmovncq	%r13,%rax
	cmovncq	%r14,%r10
	cmovncq	%r15,%r11
	movq	%rax,0(%rdi)
	cmovncq	%r9,%r12
	movq	%r10,8(%rdi)
	movq	%r11,16(%rdi)
	movq	%r12,24(%rdi)

	movq	8(%rsp),%r15
.cfi_restore	%r15
	movq	16(%rsp),%r14
.cfi_restore	%r14
	movq	24(%rsp),%r13
.cfi_restore	%r13
	movq	32(%rsp),%r12
.cfi_restore	%r12
	movq	40(%rsp),%rbx
.cfi_restore	%rbx
	movq	48(%rsp),%rbp
.cfi_restore	%rbp
	leaq	56(%rsp),%rsp
.cfi_adjust_cfa_offset	-56

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	


.p2align	5
__mulq_by_1_mont_256:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%rax
	movq	8(%rsi),%r10
	movq	16(%rsi),%r11
	movq	24(%rsi),%r12

	movq	%rax,%r13
	imulq	%rcx,%rax
	movq	%rax,%r9

	mulq	0(%rbx)
	addq	%rax,%r13
	movq	%r9,%rax
	adcq	%rdx,%r13

	mulq	8(%rbx)
	addq	%rax,%r10
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%r13,%r10
	adcq	$0,%rdx
	movq	%rdx,%r13

	mulq	16(%rbx)
	movq	%r10,%r14
	imulq	%rcx,%r10
	addq	%rax,%r11
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%r13,%r11
	adcq	$0,%rdx
	movq	%rdx,%r13

	mulq	24(%rbx)
	addq	%rax,%r12
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%r13,%r12
	adcq	$0,%rdx
	movq	%rdx,%r13

	mulq	0(%rbx)
	addq	%rax,%r14
	movq	%r10,%rax
	adcq	%rdx,%r14

	mulq	8(%rbx)
	addq	%rax,%r11
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%r14,%r11
	adcq	$0,%rdx
	movq	%rdx,%r14

	mulq	16(%rbx)
	movq	%r11,%r15
	imulq	%rcx,%r11
	addq	%rax,%r12
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%r14,%r12
	adcq	$0,%rdx
	movq	%rdx,%r14

	mulq	24(%rbx)
	addq	%rax,%r13
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%r14,%r13
	adcq	$0,%rdx
	movq	%rdx,%r14

	mulq	0(%rbx)
	addq	%rax,%r15
	movq	%r11,%rax
	adcq	%rdx,%r15

	mulq	8(%rbx)
	addq	%rax,%r12
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%r15,%r12
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	16(%rbx)
	movq	%r12,%r9
	imulq	%rcx,%r12
	addq	%rax,%r13
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%r15,%r13
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	24(%rbx)
	addq	%rax,%r14
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%r15,%r14
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	0(%rbx)
	addq	%rax,%r9
	movq	%r12,%rax
	adcq	%rdx,%r9

	mulq	8(%rbx)
	addq	%rax,%r13
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%r9,%r13
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	16(%rbx)
	addq	%rax,%r14
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%r9,%r14
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	24(%rbx)
	addq	%rax,%r15
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%r9,%r15
	adcq	$0,%rdx
	movq	%rdx,%r9
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc

