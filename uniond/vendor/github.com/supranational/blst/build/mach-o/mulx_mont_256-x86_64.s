.text	

.globl	_mulx_mont_sparse_256
.private_extern	_mulx_mont_sparse_256

.p2align	5
_mulx_mont_sparse_256:
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
	subq	$8,%rsp
.cfi_adjust_cfa_offset	8


	movq	%rdx,%rbx
	movq	0(%rdx),%rdx
	movq	0(%rsi),%r14
	movq	8(%rsi),%r15
	movq	16(%rsi),%rbp
	movq	24(%rsi),%r9
	leaq	-128(%rsi),%rsi
	leaq	-128(%rcx),%rcx

	mulxq	%r14,%rax,%r11
	call	__mulx_mont_sparse_256

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

	.byte	0xf3,0xc3
.cfi_endproc	


.globl	_sqrx_mont_sparse_256
.private_extern	_sqrx_mont_sparse_256

.p2align	5
_sqrx_mont_sparse_256:
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
	subq	$8,%rsp
.cfi_adjust_cfa_offset	8


	movq	%rsi,%rbx
	movq	%rcx,%r8
	movq	%rdx,%rcx
	movq	0(%rsi),%rdx
	movq	8(%rsi),%r15
	movq	16(%rsi),%rbp
	movq	24(%rsi),%r9
	leaq	-128(%rbx),%rsi
	leaq	-128(%rcx),%rcx

	mulxq	%rdx,%rax,%r11
	call	__mulx_mont_sparse_256

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

	.byte	0xf3,0xc3
.cfi_endproc	


.p2align	5
__mulx_mont_sparse_256:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	mulxq	%r15,%r15,%r12
	mulxq	%rbp,%rbp,%r13
	addq	%r15,%r11
	mulxq	%r9,%r9,%r14
	movq	8(%rbx),%rdx
	adcq	%rbp,%r12
	adcq	%r9,%r13
	adcq	$0,%r14

	movq	%rax,%r10
	imulq	%r8,%rax


	xorq	%r15,%r15
	mulxq	0+128(%rsi),%rbp,%r9
	adoxq	%rbp,%r11
	adcxq	%r9,%r12

	mulxq	8+128(%rsi),%rbp,%r9
	adoxq	%rbp,%r12
	adcxq	%r9,%r13

	mulxq	16+128(%rsi),%rbp,%r9
	adoxq	%rbp,%r13
	adcxq	%r9,%r14

	mulxq	24+128(%rsi),%rbp,%r9
	movq	%rax,%rdx
	adoxq	%rbp,%r14
	adcxq	%r15,%r9
	adoxq	%r9,%r15


	mulxq	0+128(%rcx),%rbp,%rax
	adcxq	%rbp,%r10
	adoxq	%r11,%rax

	mulxq	8+128(%rcx),%rbp,%r9
	adcxq	%rbp,%rax
	adoxq	%r9,%r12

	mulxq	16+128(%rcx),%rbp,%r9
	adcxq	%rbp,%r12
	adoxq	%r9,%r13

	mulxq	24+128(%rcx),%rbp,%r9
	movq	16(%rbx),%rdx
	adcxq	%rbp,%r13
	adoxq	%r9,%r14
	adcxq	%r10,%r14
	adoxq	%r10,%r15
	adcxq	%r10,%r15
	adoxq	%r10,%r10
	adcq	$0,%r10
	movq	%rax,%r11
	imulq	%r8,%rax


	xorq	%rbp,%rbp
	mulxq	0+128(%rsi),%rbp,%r9
	adoxq	%rbp,%r12
	adcxq	%r9,%r13

	mulxq	8+128(%rsi),%rbp,%r9
	adoxq	%rbp,%r13
	adcxq	%r9,%r14

	mulxq	16+128(%rsi),%rbp,%r9
	adoxq	%rbp,%r14
	adcxq	%r9,%r15

	mulxq	24+128(%rsi),%rbp,%r9
	movq	%rax,%rdx
	adoxq	%rbp,%r15
	adcxq	%r10,%r9
	adoxq	%r9,%r10


	mulxq	0+128(%rcx),%rbp,%rax
	adcxq	%rbp,%r11
	adoxq	%r12,%rax

	mulxq	8+128(%rcx),%rbp,%r9
	adcxq	%rbp,%rax
	adoxq	%r9,%r13

	mulxq	16+128(%rcx),%rbp,%r9
	adcxq	%rbp,%r13
	adoxq	%r9,%r14

	mulxq	24+128(%rcx),%rbp,%r9
	movq	24(%rbx),%rdx
	adcxq	%rbp,%r14
	adoxq	%r9,%r15
	adcxq	%r11,%r15
	adoxq	%r11,%r10
	adcxq	%r11,%r10
	adoxq	%r11,%r11
	adcq	$0,%r11
	movq	%rax,%r12
	imulq	%r8,%rax


	xorq	%rbp,%rbp
	mulxq	0+128(%rsi),%rbp,%r9
	adoxq	%rbp,%r13
	adcxq	%r9,%r14

	mulxq	8+128(%rsi),%rbp,%r9
	adoxq	%rbp,%r14
	adcxq	%r9,%r15

	mulxq	16+128(%rsi),%rbp,%r9
	adoxq	%rbp,%r15
	adcxq	%r9,%r10

	mulxq	24+128(%rsi),%rbp,%r9
	movq	%rax,%rdx
	adoxq	%rbp,%r10
	adcxq	%r11,%r9
	adoxq	%r9,%r11


	mulxq	0+128(%rcx),%rbp,%rax
	adcxq	%rbp,%r12
	adoxq	%r13,%rax

	mulxq	8+128(%rcx),%rbp,%r9
	adcxq	%rbp,%rax
	adoxq	%r9,%r14

	mulxq	16+128(%rcx),%rbp,%r9
	adcxq	%rbp,%r14
	adoxq	%r9,%r15

	mulxq	24+128(%rcx),%rbp,%r9
	movq	%rax,%rdx
	adcxq	%rbp,%r15
	adoxq	%r9,%r10
	adcxq	%r12,%r10
	adoxq	%r12,%r11
	adcxq	%r12,%r11
	adoxq	%r12,%r12
	adcq	$0,%r12
	imulq	%r8,%rdx


	xorq	%rbp,%rbp
	mulxq	0+128(%rcx),%r13,%r9
	adcxq	%rax,%r13
	adoxq	%r9,%r14

	mulxq	8+128(%rcx),%rbp,%r9
	adcxq	%rbp,%r14
	adoxq	%r9,%r15

	mulxq	16+128(%rcx),%rbp,%r9
	adcxq	%rbp,%r15
	adoxq	%r9,%r10

	mulxq	24+128(%rcx),%rbp,%r9
	movq	%r14,%rdx
	leaq	128(%rcx),%rcx
	adcxq	%rbp,%r10
	adoxq	%r9,%r11
	movq	%r15,%rax
	adcxq	%r13,%r11
	adoxq	%r13,%r12
	adcq	$0,%r12




	movq	%r10,%rbp
	subq	0(%rcx),%r14
	sbbq	8(%rcx),%r15
	sbbq	16(%rcx),%r10
	movq	%r11,%r9
	sbbq	24(%rcx),%r11
	sbbq	$0,%r12

	cmovcq	%rdx,%r14
	cmovcq	%rax,%r15
	cmovcq	%rbp,%r10
	movq	%r14,0(%rdi)
	cmovcq	%r9,%r11
	movq	%r15,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)

	.byte	0xf3,0xc3
.cfi_endproc

.globl	_fromx_mont_256
.private_extern	_fromx_mont_256

.p2align	5
_fromx_mont_256:
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
	subq	$8,%rsp
.cfi_adjust_cfa_offset	8


	movq	%rdx,%rbx
	call	__mulx_by_1_mont_256





	movq	%r15,%rdx
	movq	%r10,%r12
	movq	%r11,%r13

	subq	0(%rbx),%r14
	sbbq	8(%rbx),%r15
	sbbq	16(%rbx),%r10
	sbbq	24(%rbx),%r11

	cmovncq	%r14,%rax
	cmovncq	%r15,%rdx
	cmovncq	%r10,%r12
	movq	%rax,0(%rdi)
	cmovncq	%r11,%r13
	movq	%rdx,8(%rdi)
	movq	%r12,16(%rdi)
	movq	%r13,24(%rdi)

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

	.byte	0xf3,0xc3
.cfi_endproc	


.globl	_redcx_mont_256
.private_extern	_redcx_mont_256

.p2align	5
_redcx_mont_256:
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
	subq	$8,%rsp
.cfi_adjust_cfa_offset	8


	movq	%rdx,%rbx
	call	__mulx_by_1_mont_256

	addq	32(%rsi),%r14
	adcq	40(%rsi),%r15
	movq	%r14,%rax
	adcq	48(%rsi),%r10
	movq	%r15,%rdx
	adcq	56(%rsi),%r11
	sbbq	%rsi,%rsi




	movq	%r10,%r12
	subq	0(%rbx),%r14
	sbbq	8(%rbx),%r15
	sbbq	16(%rbx),%r10
	movq	%r11,%r13
	sbbq	24(%rbx),%r11
	sbbq	$0,%rsi

	cmovncq	%r14,%rax
	cmovncq	%r15,%rdx
	cmovncq	%r10,%r12
	movq	%rax,0(%rdi)
	cmovncq	%r11,%r13
	movq	%rdx,8(%rdi)
	movq	%r12,16(%rdi)
	movq	%r13,24(%rdi)

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

	.byte	0xf3,0xc3
.cfi_endproc	


.p2align	5
__mulx_by_1_mont_256:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%rax
	movq	8(%rsi),%r11
	movq	16(%rsi),%r12
	movq	24(%rsi),%r13

	movq	%rax,%r14
	imulq	%rcx,%rax
	movq	%rax,%r10

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
	movq	%r12,%r10
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
	addq	%rax,%r10
	movq	%r12,%rax
	adcq	%rdx,%r10

	mulq	8(%rbx)
	addq	%rax,%r13
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%r10,%r13
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	16(%rbx)
	movq	%r13,%r11
	imulq	%rcx,%r13
	addq	%rax,%r14
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%r10,%r14
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	24(%rbx)
	addq	%rax,%r15
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%r10,%r15
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	0(%rbx)
	addq	%rax,%r11
	movq	%r13,%rax
	adcq	%rdx,%r11

	mulq	8(%rbx)
	addq	%rax,%r14
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%r11,%r14
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	16(%rbx)
	addq	%rax,%r15
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%r11,%r15
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	24(%rbx)
	addq	%rax,%r10
	movq	%r14,%rax
	adcq	$0,%rdx
	addq	%r11,%r10
	adcq	$0,%rdx
	movq	%rdx,%r11
	.byte	0xf3,0xc3
.cfi_endproc

