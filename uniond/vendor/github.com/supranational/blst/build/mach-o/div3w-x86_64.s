.text	

.globl	_div_3_limbs
.private_extern	_div_3_limbs

.p2align	5
_div_3_limbs:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa



#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	(%rdi),%r8
	movq	8(%rdi),%r9
	xorq	%rax,%rax
	movl	$64,%ecx

L$oop:
	movq	%r8,%r10
	subq	%rsi,%r8
	movq	%r9,%r11
	sbbq	%rdx,%r9
	leaq	1(%rax,%rax,1),%rax
	movq	%rdx,%rdi
	cmovcq	%r10,%r8
	cmovcq	%r11,%r9
	sbbq	$0,%rax
	shlq	$63,%rdi
	shrq	$1,%rsi
	shrq	$1,%rdx
	orq	%rdi,%rsi
	subl	$1,%ecx
	jnz	L$oop

	leaq	1(%rax,%rax,1),%rcx
	sarq	$63,%rax

	subq	%rsi,%r8
	sbbq	%rdx,%r9
	sbbq	$0,%rcx

	orq	%rcx,%rax


	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	

.globl	_quot_rem_128
.private_extern	_quot_rem_128

.p2align	5
_quot_rem_128:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa



#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	%rdx,%rax
	movq	%rdx,%rcx

	mulq	0(%rsi)
	movq	%rax,%r8
	movq	%rcx,%rax
	movq	%rdx,%r9

	mulq	8(%rsi)
	addq	%rax,%r9
	adcq	$0,%rdx

	movq	0(%rdi),%r10
	movq	8(%rdi),%r11
	movq	16(%rdi),%rax

	subq	%r8,%r10
	sbbq	%r9,%r11
	sbbq	%rdx,%rax
	sbbq	%r8,%r8

	addq	%r8,%rcx
	movq	%r8,%r9
	andq	0(%rsi),%r8
	andq	8(%rsi),%r9
	addq	%r8,%r10
	adcq	%r9,%r11

	movq	%r10,0(%rdi)
	movq	%r11,8(%rdi)
	movq	%rcx,16(%rdi)

	movq	%rcx,%rax


	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	






.globl	_quot_rem_64
.private_extern	_quot_rem_64

.p2align	5
_quot_rem_64:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa



#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	%rdx,%rax
	imulq	0(%rsi),%rdx

	movq	0(%rdi),%r10

	subq	%rdx,%r10

	movq	%r10,0(%rdi)
	movq	%rax,8(%rdi)


	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	

