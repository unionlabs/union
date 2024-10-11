.text	

.globl	add_mod_384
.hidden	add_mod_384
.type	add_mod_384,@function
.align	32
add_mod_384:
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


	call	__add_mod_384

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
.size	add_mod_384,.-add_mod_384

.type	__add_mod_384,@function
.align	32
__add_mod_384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

__add_mod_384_a_is_loaded:
	addq	0(%rdx),%r8
	adcq	8(%rdx),%r9
	adcq	16(%rdx),%r10
	movq	%r8,%r14
	adcq	24(%rdx),%r11
	movq	%r9,%r15
	adcq	32(%rdx),%r12
	movq	%r10,%rax
	adcq	40(%rdx),%r13
	movq	%r11,%rbx
	sbbq	%rdx,%rdx

	subq	0(%rcx),%r8
	sbbq	8(%rcx),%r9
	movq	%r12,%rbp
	sbbq	16(%rcx),%r10
	sbbq	24(%rcx),%r11
	sbbq	32(%rcx),%r12
	movq	%r13,%rsi
	sbbq	40(%rcx),%r13
	sbbq	$0,%rdx

	cmovcq	%r14,%r8
	cmovcq	%r15,%r9
	cmovcq	%rax,%r10
	movq	%r8,0(%rdi)
	cmovcq	%rbx,%r11
	movq	%r9,8(%rdi)
	cmovcq	%rbp,%r12
	movq	%r10,16(%rdi)
	cmovcq	%rsi,%r13
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
.size	__add_mod_384,.-__add_mod_384

.globl	add_mod_384x
.hidden	add_mod_384x
.type	add_mod_384x,@function
.align	32
add_mod_384x:
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
	subq	$24,%rsp
.cfi_adjust_cfa_offset	24


	movq	%rsi,0(%rsp)
	movq	%rdx,8(%rsp)
	leaq	48(%rsi),%rsi
	leaq	48(%rdx),%rdx
	leaq	48(%rdi),%rdi
	call	__add_mod_384

	movq	0(%rsp),%rsi
	movq	8(%rsp),%rdx
	leaq	-48(%rdi),%rdi
	call	__add_mod_384

	movq	24+0(%rsp),%r15
.cfi_restore	%r15
	movq	24+8(%rsp),%r14
.cfi_restore	%r14
	movq	24+16(%rsp),%r13
.cfi_restore	%r13
	movq	24+24(%rsp),%r12
.cfi_restore	%r12
	movq	24+32(%rsp),%rbx
.cfi_restore	%rbx
	movq	24+40(%rsp),%rbp
.cfi_restore	%rbp
	leaq	24+48(%rsp),%rsp
.cfi_adjust_cfa_offset	-24-8*6

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	
.size	add_mod_384x,.-add_mod_384x


.globl	rshift_mod_384
.hidden	rshift_mod_384
.type	rshift_mod_384,@function
.align	32
rshift_mod_384:
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
	pushq	%rdi
.cfi_adjust_cfa_offset	8


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

.Loop_rshift_mod_384:
	call	__rshift_mod_384
	decl	%edx
	jnz	.Loop_rshift_mod_384

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%r13,40(%rdi)

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
.size	rshift_mod_384,.-rshift_mod_384

.type	__rshift_mod_384,@function
.align	32
__rshift_mod_384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	$1,%rsi
	movq	0(%rcx),%r14
	andq	%r8,%rsi
	movq	8(%rcx),%r15
	negq	%rsi
	movq	16(%rcx),%rax
	andq	%rsi,%r14
	movq	24(%rcx),%rbx
	andq	%rsi,%r15
	movq	32(%rcx),%rbp
	andq	%rsi,%rax
	andq	%rsi,%rbx
	andq	%rsi,%rbp
	andq	40(%rcx),%rsi

	addq	%r8,%r14
	adcq	%r9,%r15
	adcq	%r10,%rax
	adcq	%r11,%rbx
	adcq	%r12,%rbp
	adcq	%r13,%rsi
	sbbq	%r13,%r13

	shrq	$1,%r14
	movq	%r15,%r8
	shrq	$1,%r15
	movq	%rax,%r9
	shrq	$1,%rax
	movq	%rbx,%r10
	shrq	$1,%rbx
	movq	%rbp,%r11
	shrq	$1,%rbp
	movq	%rsi,%r12
	shrq	$1,%rsi
	shlq	$63,%r8
	shlq	$63,%r9
	orq	%r14,%r8
	shlq	$63,%r10
	orq	%r15,%r9
	shlq	$63,%r11
	orq	%rax,%r10
	shlq	$63,%r12
	orq	%rbx,%r11
	shlq	$63,%r13
	orq	%rbp,%r12
	orq	%rsi,%r13

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%r14
	lfence
	jmpq	*%r14
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	__rshift_mod_384,.-__rshift_mod_384

.globl	div_by_2_mod_384
.hidden	div_by_2_mod_384
.type	div_by_2_mod_384,@function
.align	32
div_by_2_mod_384:
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
	pushq	%rdi
.cfi_adjust_cfa_offset	8


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	%rdx,%rcx
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	call	__rshift_mod_384

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%r13,40(%rdi)

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
.size	div_by_2_mod_384,.-div_by_2_mod_384


.globl	lshift_mod_384
.hidden	lshift_mod_384
.type	lshift_mod_384,@function
.align	32
lshift_mod_384:
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
	pushq	%rdi
.cfi_adjust_cfa_offset	8


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

.Loop_lshift_mod_384:
	addq	%r8,%r8
	adcq	%r9,%r9
	adcq	%r10,%r10
	movq	%r8,%r14
	adcq	%r11,%r11
	movq	%r9,%r15
	adcq	%r12,%r12
	movq	%r10,%rax
	adcq	%r13,%r13
	movq	%r11,%rbx
	sbbq	%rdi,%rdi

	subq	0(%rcx),%r8
	sbbq	8(%rcx),%r9
	movq	%r12,%rbp
	sbbq	16(%rcx),%r10
	sbbq	24(%rcx),%r11
	sbbq	32(%rcx),%r12
	movq	%r13,%rsi
	sbbq	40(%rcx),%r13
	sbbq	$0,%rdi

	movq	(%rsp),%rdi
	cmovcq	%r14,%r8
	cmovcq	%r15,%r9
	cmovcq	%rax,%r10
	cmovcq	%rbx,%r11
	cmovcq	%rbp,%r12
	cmovcq	%rsi,%r13

	decl	%edx
	jnz	.Loop_lshift_mod_384

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%r13,40(%rdi)

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
.size	lshift_mod_384,.-lshift_mod_384

.type	__lshift_mod_384,@function
.align	32
__lshift_mod_384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	addq	%r8,%r8
	adcq	%r9,%r9
	adcq	%r10,%r10
	movq	%r8,%r14
	adcq	%r11,%r11
	movq	%r9,%r15
	adcq	%r12,%r12
	movq	%r10,%rax
	adcq	%r13,%r13
	movq	%r11,%rbx
	sbbq	%rdx,%rdx

	subq	0(%rcx),%r8
	sbbq	8(%rcx),%r9
	movq	%r12,%rbp
	sbbq	16(%rcx),%r10
	sbbq	24(%rcx),%r11
	sbbq	32(%rcx),%r12
	movq	%r13,%rsi
	sbbq	40(%rcx),%r13
	sbbq	$0,%rdx

	cmovcq	%r14,%r8
	cmovcq	%r15,%r9
	cmovcq	%rax,%r10
	cmovcq	%rbx,%r11
	cmovcq	%rbp,%r12
	cmovcq	%rsi,%r13

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	__lshift_mod_384,.-__lshift_mod_384


.globl	mul_by_3_mod_384
.hidden	mul_by_3_mod_384
.type	mul_by_3_mod_384,@function
.align	32
mul_by_3_mod_384:
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
	pushq	%rsi
.cfi_adjust_cfa_offset	8


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13
	movq	%rdx,%rcx

	call	__lshift_mod_384

	movq	(%rsp),%rdx
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	call	__add_mod_384_a_is_loaded

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
.size	mul_by_3_mod_384,.-mul_by_3_mod_384

.globl	mul_by_8_mod_384
.hidden	mul_by_8_mod_384
.type	mul_by_8_mod_384,@function
.align	32
mul_by_8_mod_384:
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


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13
	movq	%rdx,%rcx

	call	__lshift_mod_384
	call	__lshift_mod_384
	call	__lshift_mod_384

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%r13,40(%rdi)

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
.size	mul_by_8_mod_384,.-mul_by_8_mod_384


.globl	mul_by_3_mod_384x
.hidden	mul_by_3_mod_384x
.type	mul_by_3_mod_384x,@function
.align	32
mul_by_3_mod_384x:
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
	pushq	%rsi
.cfi_adjust_cfa_offset	8


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13
	movq	%rdx,%rcx

	call	__lshift_mod_384

	movq	(%rsp),%rdx
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	call	__add_mod_384_a_is_loaded

	movq	(%rsp),%rsi
	leaq	48(%rdi),%rdi

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	48(%rsi),%r8
	movq	56(%rsi),%r9
	movq	64(%rsi),%r10
	movq	72(%rsi),%r11
	movq	80(%rsi),%r12
	movq	88(%rsi),%r13

	call	__lshift_mod_384

	movq	$48,%rdx
	addq	(%rsp),%rdx
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	call	__add_mod_384_a_is_loaded

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
.size	mul_by_3_mod_384x,.-mul_by_3_mod_384x

.globl	mul_by_8_mod_384x
.hidden	mul_by_8_mod_384x
.type	mul_by_8_mod_384x,@function
.align	32
mul_by_8_mod_384x:
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
	pushq	%rsi
.cfi_adjust_cfa_offset	8


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13
	movq	%rdx,%rcx

	call	__lshift_mod_384
	call	__lshift_mod_384
	call	__lshift_mod_384

	movq	(%rsp),%rsi
	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)
	movq	%r12,32(%rdi)
	movq	%r13,40(%rdi)

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	48+0(%rsi),%r8
	movq	48+8(%rsi),%r9
	movq	48+16(%rsi),%r10
	movq	48+24(%rsi),%r11
	movq	48+32(%rsi),%r12
	movq	48+40(%rsi),%r13

	call	__lshift_mod_384
	call	__lshift_mod_384
	call	__lshift_mod_384

	movq	%r8,48+0(%rdi)
	movq	%r9,48+8(%rdi)
	movq	%r10,48+16(%rdi)
	movq	%r11,48+24(%rdi)
	movq	%r12,48+32(%rdi)
	movq	%r13,48+40(%rdi)

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
.size	mul_by_8_mod_384x,.-mul_by_8_mod_384x


.globl	cneg_mod_384
.hidden	cneg_mod_384
.type	cneg_mod_384,@function
.align	32
cneg_mod_384:
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
	pushq	%rdx
.cfi_adjust_cfa_offset	8


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%rdx
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	%rdx,%r8
	movq	24(%rsi),%r11
	orq	%r9,%rdx
	movq	32(%rsi),%r12
	orq	%r10,%rdx
	movq	40(%rsi),%r13
	orq	%r11,%rdx
	movq	$-1,%rsi
	orq	%r12,%rdx
	orq	%r13,%rdx

	movq	0(%rcx),%r14
	cmovnzq	%rsi,%rdx
	movq	8(%rcx),%r15
	movq	16(%rcx),%rax
	andq	%rdx,%r14
	movq	24(%rcx),%rbx
	andq	%rdx,%r15
	movq	32(%rcx),%rbp
	andq	%rdx,%rax
	movq	40(%rcx),%rsi
	andq	%rdx,%rbx
	movq	0(%rsp),%rcx
	andq	%rdx,%rbp
	andq	%rdx,%rsi

	subq	%r8,%r14
	sbbq	%r9,%r15
	sbbq	%r10,%rax
	sbbq	%r11,%rbx
	sbbq	%r12,%rbp
	sbbq	%r13,%rsi

	orq	%rcx,%rcx

	cmovzq	%r8,%r14
	cmovzq	%r9,%r15
	cmovzq	%r10,%rax
	movq	%r14,0(%rdi)
	cmovzq	%r11,%rbx
	movq	%r15,8(%rdi)
	cmovzq	%r12,%rbp
	movq	%rax,16(%rdi)
	cmovzq	%r13,%rsi
	movq	%rbx,24(%rdi)
	movq	%rbp,32(%rdi)
	movq	%rsi,40(%rdi)

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
.size	cneg_mod_384,.-cneg_mod_384


.globl	sub_mod_384
.hidden	sub_mod_384
.type	sub_mod_384,@function
.align	32
sub_mod_384:
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


	call	__sub_mod_384

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
.size	sub_mod_384,.-sub_mod_384

.type	__sub_mod_384,@function
.align	32
__sub_mod_384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	subq	0(%rdx),%r8
	movq	0(%rcx),%r14
	sbbq	8(%rdx),%r9
	movq	8(%rcx),%r15
	sbbq	16(%rdx),%r10
	movq	16(%rcx),%rax
	sbbq	24(%rdx),%r11
	movq	24(%rcx),%rbx
	sbbq	32(%rdx),%r12
	movq	32(%rcx),%rbp
	sbbq	40(%rdx),%r13
	movq	40(%rcx),%rsi
	sbbq	%rdx,%rdx

	andq	%rdx,%r14
	andq	%rdx,%r15
	andq	%rdx,%rax
	andq	%rdx,%rbx
	andq	%rdx,%rbp
	andq	%rdx,%rsi

	addq	%r14,%r8
	adcq	%r15,%r9
	movq	%r8,0(%rdi)
	adcq	%rax,%r10
	movq	%r9,8(%rdi)
	adcq	%rbx,%r11
	movq	%r10,16(%rdi)
	adcq	%rbp,%r12
	movq	%r11,24(%rdi)
	adcq	%rsi,%r13
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
.size	__sub_mod_384,.-__sub_mod_384

.globl	sub_mod_384x
.hidden	sub_mod_384x
.type	sub_mod_384x,@function
.align	32
sub_mod_384x:
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
	subq	$24,%rsp
.cfi_adjust_cfa_offset	24


	movq	%rsi,0(%rsp)
	movq	%rdx,8(%rsp)
	leaq	48(%rsi),%rsi
	leaq	48(%rdx),%rdx
	leaq	48(%rdi),%rdi
	call	__sub_mod_384

	movq	0(%rsp),%rsi
	movq	8(%rsp),%rdx
	leaq	-48(%rdi),%rdi
	call	__sub_mod_384

	movq	24+0(%rsp),%r15
.cfi_restore	%r15
	movq	24+8(%rsp),%r14
.cfi_restore	%r14
	movq	24+16(%rsp),%r13
.cfi_restore	%r13
	movq	24+24(%rsp),%r12
.cfi_restore	%r12
	movq	24+32(%rsp),%rbx
.cfi_restore	%rbx
	movq	24+40(%rsp),%rbp
.cfi_restore	%rbp
	leaq	24+48(%rsp),%rsp
.cfi_adjust_cfa_offset	-24-8*6

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	
.size	sub_mod_384x,.-sub_mod_384x
.globl	mul_by_1_plus_i_mod_384x
.hidden	mul_by_1_plus_i_mod_384x
.type	mul_by_1_plus_i_mod_384x,@function
.align	32
mul_by_1_plus_i_mod_384x:
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
	subq	$56,%rsp
.cfi_adjust_cfa_offset	56


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	movq	%r8,%r14
	addq	48(%rsi),%r8
	movq	%r9,%r15
	adcq	56(%rsi),%r9
	movq	%r10,%rax
	adcq	64(%rsi),%r10
	movq	%r11,%rbx
	adcq	72(%rsi),%r11
	movq	%r12,%rcx
	adcq	80(%rsi),%r12
	movq	%r13,%rbp
	adcq	88(%rsi),%r13
	movq	%rdi,48(%rsp)
	sbbq	%rdi,%rdi

	subq	48(%rsi),%r14
	sbbq	56(%rsi),%r15
	sbbq	64(%rsi),%rax
	sbbq	72(%rsi),%rbx
	sbbq	80(%rsi),%rcx
	sbbq	88(%rsi),%rbp
	sbbq	%rsi,%rsi

	movq	%r8,0(%rsp)
	movq	0(%rdx),%r8
	movq	%r9,8(%rsp)
	movq	8(%rdx),%r9
	movq	%r10,16(%rsp)
	movq	16(%rdx),%r10
	movq	%r11,24(%rsp)
	movq	24(%rdx),%r11
	movq	%r12,32(%rsp)
	andq	%rsi,%r8
	movq	32(%rdx),%r12
	movq	%r13,40(%rsp)
	andq	%rsi,%r9
	movq	40(%rdx),%r13
	andq	%rsi,%r10
	andq	%rsi,%r11
	andq	%rsi,%r12
	andq	%rsi,%r13
	movq	48(%rsp),%rsi

	addq	%r8,%r14
	movq	0(%rsp),%r8
	adcq	%r9,%r15
	movq	8(%rsp),%r9
	adcq	%r10,%rax
	movq	16(%rsp),%r10
	adcq	%r11,%rbx
	movq	24(%rsp),%r11
	adcq	%r12,%rcx
	movq	32(%rsp),%r12
	adcq	%r13,%rbp
	movq	40(%rsp),%r13

	movq	%r14,0(%rsi)
	movq	%r8,%r14
	movq	%r15,8(%rsi)
	movq	%rax,16(%rsi)
	movq	%r9,%r15
	movq	%rbx,24(%rsi)
	movq	%rcx,32(%rsi)
	movq	%r10,%rax
	movq	%rbp,40(%rsi)

	subq	0(%rdx),%r8
	movq	%r11,%rbx
	sbbq	8(%rdx),%r9
	sbbq	16(%rdx),%r10
	movq	%r12,%rcx
	sbbq	24(%rdx),%r11
	sbbq	32(%rdx),%r12
	movq	%r13,%rbp
	sbbq	40(%rdx),%r13
	sbbq	$0,%rdi

	cmovcq	%r14,%r8
	cmovcq	%r15,%r9
	cmovcq	%rax,%r10
	movq	%r8,48(%rsi)
	cmovcq	%rbx,%r11
	movq	%r9,56(%rsi)
	cmovcq	%rcx,%r12
	movq	%r10,64(%rsi)
	cmovcq	%rbp,%r13
	movq	%r11,72(%rsi)
	movq	%r12,80(%rsi)
	movq	%r13,88(%rsi)

	movq	56+0(%rsp),%r15
.cfi_restore	%r15
	movq	56+8(%rsp),%r14
.cfi_restore	%r14
	movq	56+16(%rsp),%r13
.cfi_restore	%r13
	movq	56+24(%rsp),%r12
.cfi_restore	%r12
	movq	56+32(%rsp),%rbx
.cfi_restore	%rbx
	movq	56+40(%rsp),%rbp
.cfi_restore	%rbp
	leaq	56+48(%rsp),%rsp
.cfi_adjust_cfa_offset	-56-8*6

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	
.size	mul_by_1_plus_i_mod_384x,.-mul_by_1_plus_i_mod_384x
.globl	sgn0_pty_mod_384
.hidden	sgn0_pty_mod_384
.type	sgn0_pty_mod_384,@function
.align	32
sgn0_pty_mod_384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa



#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rdi),%r8
	movq	8(%rdi),%r9
	movq	16(%rdi),%r10
	movq	24(%rdi),%r11
	movq	32(%rdi),%rcx
	movq	40(%rdi),%rdx

	xorq	%rax,%rax
	movq	%r8,%rdi
	addq	%r8,%r8
	adcq	%r9,%r9
	adcq	%r10,%r10
	adcq	%r11,%r11
	adcq	%rcx,%rcx
	adcq	%rdx,%rdx
	adcq	$0,%rax

	subq	0(%rsi),%r8
	sbbq	8(%rsi),%r9
	sbbq	16(%rsi),%r10
	sbbq	24(%rsi),%r11
	sbbq	32(%rsi),%rcx
	sbbq	40(%rsi),%rdx
	sbbq	$0,%rax

	notq	%rax
	andq	$1,%rdi
	andq	$2,%rax
	orq	%rdi,%rax


	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	
.size	sgn0_pty_mod_384,.-sgn0_pty_mod_384

.globl	sgn0_pty_mod_384x
.hidden	sgn0_pty_mod_384x
.type	sgn0_pty_mod_384x,@function
.align	32
sgn0_pty_mod_384x:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa


	pushq	%rbp
.cfi_adjust_cfa_offset	8
.cfi_offset	%rbp,-16
	pushq	%rbx
.cfi_adjust_cfa_offset	8
.cfi_offset	%rbx,-24
	subq	$8,%rsp
.cfi_adjust_cfa_offset	8


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	48(%rdi),%r8
	movq	56(%rdi),%r9
	movq	64(%rdi),%r10
	movq	72(%rdi),%r11
	movq	80(%rdi),%rcx
	movq	88(%rdi),%rdx

	movq	%r8,%rbx
	orq	%r9,%r8
	orq	%r10,%r8
	orq	%r11,%r8
	orq	%rcx,%r8
	orq	%rdx,%r8

	leaq	0(%rdi),%rax
	xorq	%rdi,%rdi
	movq	%rbx,%rbp
	addq	%rbx,%rbx
	adcq	%r9,%r9
	adcq	%r10,%r10
	adcq	%r11,%r11
	adcq	%rcx,%rcx
	adcq	%rdx,%rdx
	adcq	$0,%rdi

	subq	0(%rsi),%rbx
	sbbq	8(%rsi),%r9
	sbbq	16(%rsi),%r10
	sbbq	24(%rsi),%r11
	sbbq	32(%rsi),%rcx
	sbbq	40(%rsi),%rdx
	sbbq	$0,%rdi

	movq	%r8,0(%rsp)
	notq	%rdi
	andq	$1,%rbp
	andq	$2,%rdi
	orq	%rbp,%rdi

	movq	0(%rax),%r8
	movq	8(%rax),%r9
	movq	16(%rax),%r10
	movq	24(%rax),%r11
	movq	32(%rax),%rcx
	movq	40(%rax),%rdx

	movq	%r8,%rbx
	orq	%r9,%r8
	orq	%r10,%r8
	orq	%r11,%r8
	orq	%rcx,%r8
	orq	%rdx,%r8

	xorq	%rax,%rax
	movq	%rbx,%rbp
	addq	%rbx,%rbx
	adcq	%r9,%r9
	adcq	%r10,%r10
	adcq	%r11,%r11
	adcq	%rcx,%rcx
	adcq	%rdx,%rdx
	adcq	$0,%rax

	subq	0(%rsi),%rbx
	sbbq	8(%rsi),%r9
	sbbq	16(%rsi),%r10
	sbbq	24(%rsi),%r11
	sbbq	32(%rsi),%rcx
	sbbq	40(%rsi),%rdx
	sbbq	$0,%rax

	movq	0(%rsp),%rbx

	notq	%rax

	testq	%r8,%r8
	cmovzq	%rdi,%rbp

	testq	%rbx,%rbx
	cmovnzq	%rdi,%rax

	andq	$1,%rbp
	andq	$2,%rax
	orq	%rbp,%rax

	movq	8(%rsp),%rbx
.cfi_restore	%rbx
	movq	16(%rsp),%rbp
.cfi_restore	%rbp
	leaq	24(%rsp),%rsp
.cfi_adjust_cfa_offset	-24

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc	
.size	sgn0_pty_mod_384x,.-sgn0_pty_mod_384x
.globl	vec_select_32
.hidden	vec_select_32
.type	vec_select_32,@function
.align	32
vec_select_32:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movd	%ecx,%xmm5
	pxor	%xmm4,%xmm4
	pshufd	$0,%xmm5,%xmm5
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movdqu	(%rsi),%xmm0
	leaq	16(%rsi),%rsi
	pcmpeqd	%xmm4,%xmm5
	movdqu	(%rdx),%xmm1
	leaq	16(%rdx),%rdx
	pcmpeqd	%xmm5,%xmm4
	leaq	16(%rdi),%rdi
	pand	%xmm4,%xmm0
	movdqu	0+16-16(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	0+16-16(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,0-16(%rdi)
	pand	%xmm4,%xmm2
	pand	%xmm5,%xmm3
	por	%xmm3,%xmm2
	movdqu	%xmm2,16-16(%rdi)
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	vec_select_32,.-vec_select_32
.globl	vec_select_48
.hidden	vec_select_48
.type	vec_select_48,@function
.align	32
vec_select_48:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movd	%ecx,%xmm5
	pxor	%xmm4,%xmm4
	pshufd	$0,%xmm5,%xmm5
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movdqu	(%rsi),%xmm0
	leaq	24(%rsi),%rsi
	pcmpeqd	%xmm4,%xmm5
	movdqu	(%rdx),%xmm1
	leaq	24(%rdx),%rdx
	pcmpeqd	%xmm5,%xmm4
	leaq	24(%rdi),%rdi
	pand	%xmm4,%xmm0
	movdqu	0+16-24(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	0+16-24(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,0-24(%rdi)
	pand	%xmm4,%xmm2
	movdqu	16+16-24(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	16+16-24(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,16-24(%rdi)
	pand	%xmm4,%xmm0
	pand	%xmm5,%xmm1
	por	%xmm1,%xmm0
	movdqu	%xmm0,32-24(%rdi)
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	vec_select_48,.-vec_select_48
.globl	vec_select_96
.hidden	vec_select_96
.type	vec_select_96,@function
.align	32
vec_select_96:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movd	%ecx,%xmm5
	pxor	%xmm4,%xmm4
	pshufd	$0,%xmm5,%xmm5
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movdqu	(%rsi),%xmm0
	leaq	48(%rsi),%rsi
	pcmpeqd	%xmm4,%xmm5
	movdqu	(%rdx),%xmm1
	leaq	48(%rdx),%rdx
	pcmpeqd	%xmm5,%xmm4
	leaq	48(%rdi),%rdi
	pand	%xmm4,%xmm0
	movdqu	0+16-48(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	0+16-48(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,0-48(%rdi)
	pand	%xmm4,%xmm2
	movdqu	16+16-48(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	16+16-48(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,16-48(%rdi)
	pand	%xmm4,%xmm0
	movdqu	32+16-48(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	32+16-48(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,32-48(%rdi)
	pand	%xmm4,%xmm2
	movdqu	48+16-48(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	48+16-48(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,48-48(%rdi)
	pand	%xmm4,%xmm0
	movdqu	64+16-48(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	64+16-48(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,64-48(%rdi)
	pand	%xmm4,%xmm2
	pand	%xmm5,%xmm3
	por	%xmm3,%xmm2
	movdqu	%xmm2,80-48(%rdi)
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	vec_select_96,.-vec_select_96
.globl	vec_select_192
.hidden	vec_select_192
.type	vec_select_192,@function
.align	32
vec_select_192:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movd	%ecx,%xmm5
	pxor	%xmm4,%xmm4
	pshufd	$0,%xmm5,%xmm5
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movdqu	(%rsi),%xmm0
	leaq	96(%rsi),%rsi
	pcmpeqd	%xmm4,%xmm5
	movdqu	(%rdx),%xmm1
	leaq	96(%rdx),%rdx
	pcmpeqd	%xmm5,%xmm4
	leaq	96(%rdi),%rdi
	pand	%xmm4,%xmm0
	movdqu	0+16-96(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	0+16-96(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,0-96(%rdi)
	pand	%xmm4,%xmm2
	movdqu	16+16-96(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	16+16-96(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,16-96(%rdi)
	pand	%xmm4,%xmm0
	movdqu	32+16-96(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	32+16-96(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,32-96(%rdi)
	pand	%xmm4,%xmm2
	movdqu	48+16-96(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	48+16-96(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,48-96(%rdi)
	pand	%xmm4,%xmm0
	movdqu	64+16-96(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	64+16-96(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,64-96(%rdi)
	pand	%xmm4,%xmm2
	movdqu	80+16-96(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	80+16-96(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,80-96(%rdi)
	pand	%xmm4,%xmm0
	movdqu	96+16-96(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	96+16-96(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,96-96(%rdi)
	pand	%xmm4,%xmm2
	movdqu	112+16-96(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	112+16-96(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,112-96(%rdi)
	pand	%xmm4,%xmm0
	movdqu	128+16-96(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	128+16-96(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,128-96(%rdi)
	pand	%xmm4,%xmm2
	movdqu	144+16-96(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	144+16-96(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,144-96(%rdi)
	pand	%xmm4,%xmm0
	movdqu	160+16-96(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	160+16-96(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,160-96(%rdi)
	pand	%xmm4,%xmm2
	pand	%xmm5,%xmm3
	por	%xmm3,%xmm2
	movdqu	%xmm2,176-96(%rdi)
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	vec_select_192,.-vec_select_192
.globl	vec_select_144
.hidden	vec_select_144
.type	vec_select_144,@function
.align	32
vec_select_144:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movd	%ecx,%xmm5
	pxor	%xmm4,%xmm4
	pshufd	$0,%xmm5,%xmm5
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movdqu	(%rsi),%xmm0
	leaq	72(%rsi),%rsi
	pcmpeqd	%xmm4,%xmm5
	movdqu	(%rdx),%xmm1
	leaq	72(%rdx),%rdx
	pcmpeqd	%xmm5,%xmm4
	leaq	72(%rdi),%rdi
	pand	%xmm4,%xmm0
	movdqu	0+16-72(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	0+16-72(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,0-72(%rdi)
	pand	%xmm4,%xmm2
	movdqu	16+16-72(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	16+16-72(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,16-72(%rdi)
	pand	%xmm4,%xmm0
	movdqu	32+16-72(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	32+16-72(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,32-72(%rdi)
	pand	%xmm4,%xmm2
	movdqu	48+16-72(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	48+16-72(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,48-72(%rdi)
	pand	%xmm4,%xmm0
	movdqu	64+16-72(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	64+16-72(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,64-72(%rdi)
	pand	%xmm4,%xmm2
	movdqu	80+16-72(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	80+16-72(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,80-72(%rdi)
	pand	%xmm4,%xmm0
	movdqu	96+16-72(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	96+16-72(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,96-72(%rdi)
	pand	%xmm4,%xmm2
	movdqu	112+16-72(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	112+16-72(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,112-72(%rdi)
	pand	%xmm4,%xmm0
	pand	%xmm5,%xmm1
	por	%xmm1,%xmm0
	movdqu	%xmm0,128-72(%rdi)
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	vec_select_144,.-vec_select_144
.globl	vec_select_288
.hidden	vec_select_288
.type	vec_select_288,@function
.align	32
vec_select_288:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movd	%ecx,%xmm5
	pxor	%xmm4,%xmm4
	pshufd	$0,%xmm5,%xmm5
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movdqu	(%rsi),%xmm0
	leaq	144(%rsi),%rsi
	pcmpeqd	%xmm4,%xmm5
	movdqu	(%rdx),%xmm1
	leaq	144(%rdx),%rdx
	pcmpeqd	%xmm5,%xmm4
	leaq	144(%rdi),%rdi
	pand	%xmm4,%xmm0
	movdqu	0+16-144(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	0+16-144(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,0-144(%rdi)
	pand	%xmm4,%xmm2
	movdqu	16+16-144(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	16+16-144(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,16-144(%rdi)
	pand	%xmm4,%xmm0
	movdqu	32+16-144(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	32+16-144(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,32-144(%rdi)
	pand	%xmm4,%xmm2
	movdqu	48+16-144(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	48+16-144(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,48-144(%rdi)
	pand	%xmm4,%xmm0
	movdqu	64+16-144(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	64+16-144(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,64-144(%rdi)
	pand	%xmm4,%xmm2
	movdqu	80+16-144(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	80+16-144(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,80-144(%rdi)
	pand	%xmm4,%xmm0
	movdqu	96+16-144(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	96+16-144(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,96-144(%rdi)
	pand	%xmm4,%xmm2
	movdqu	112+16-144(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	112+16-144(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,112-144(%rdi)
	pand	%xmm4,%xmm0
	movdqu	128+16-144(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	128+16-144(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,128-144(%rdi)
	pand	%xmm4,%xmm2
	movdqu	144+16-144(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	144+16-144(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,144-144(%rdi)
	pand	%xmm4,%xmm0
	movdqu	160+16-144(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	160+16-144(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,160-144(%rdi)
	pand	%xmm4,%xmm2
	movdqu	176+16-144(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	176+16-144(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,176-144(%rdi)
	pand	%xmm4,%xmm0
	movdqu	192+16-144(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	192+16-144(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,192-144(%rdi)
	pand	%xmm4,%xmm2
	movdqu	208+16-144(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	208+16-144(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,208-144(%rdi)
	pand	%xmm4,%xmm0
	movdqu	224+16-144(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	224+16-144(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,224-144(%rdi)
	pand	%xmm4,%xmm2
	movdqu	240+16-144(%rsi),%xmm0
	pand	%xmm5,%xmm3
	movdqu	240+16-144(%rdx),%xmm1
	por	%xmm3,%xmm2
	movdqu	%xmm2,240-144(%rdi)
	pand	%xmm4,%xmm0
	movdqu	256+16-144(%rsi),%xmm2
	pand	%xmm5,%xmm1
	movdqu	256+16-144(%rdx),%xmm3
	por	%xmm1,%xmm0
	movdqu	%xmm0,256-144(%rdi)
	pand	%xmm4,%xmm2
	pand	%xmm5,%xmm3
	por	%xmm3,%xmm2
	movdqu	%xmm2,272-144(%rdi)
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	vec_select_288,.-vec_select_288
.globl	vec_prefetch
.hidden	vec_prefetch
.type	vec_prefetch,@function
.align	32
vec_prefetch:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	leaq	-1(%rdi,%rsi,1),%rsi
	movq	$64,%rax
	xorq	%r8,%r8
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	prefetchnta	(%rdi)
	leaq	(%rdi,%rax,1),%rdi
	cmpq	%rsi,%rdi
	cmovaq	%rsi,%rdi
	cmovaq	%r8,%rax
	prefetchnta	(%rdi)
	leaq	(%rdi,%rax,1),%rdi
	cmpq	%rsi,%rdi
	cmovaq	%rsi,%rdi
	cmovaq	%r8,%rax
	prefetchnta	(%rdi)
	leaq	(%rdi,%rax,1),%rdi
	cmpq	%rsi,%rdi
	cmovaq	%rsi,%rdi
	cmovaq	%r8,%rax
	prefetchnta	(%rdi)
	leaq	(%rdi,%rax,1),%rdi
	cmpq	%rsi,%rdi
	cmovaq	%rsi,%rdi
	cmovaq	%r8,%rax
	prefetchnta	(%rdi)
	leaq	(%rdi,%rax,1),%rdi
	cmpq	%rsi,%rdi
	cmovaq	%rsi,%rdi
	cmovaq	%r8,%rax
	prefetchnta	(%rdi)
	leaq	(%rdi,%rax,1),%rdi
	cmpq	%rsi,%rdi
	cmovaq	%rsi,%rdi
	prefetchnta	(%rdi)
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	vec_prefetch,.-vec_prefetch
.globl	vec_is_zero_16x
.hidden	vec_is_zero_16x
.type	vec_is_zero_16x,@function
.align	32
vec_is_zero_16x:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	shrl	$4,%esi
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movdqu	(%rdi),%xmm0
	leaq	16(%rdi),%rdi

.Loop_is_zero:
	decl	%esi
	jz	.Loop_is_zero_done
	movdqu	(%rdi),%xmm1
	leaq	16(%rdi),%rdi
	por	%xmm1,%xmm0
	jmp	.Loop_is_zero

.Loop_is_zero_done:
	pshufd	$0x4e,%xmm0,%xmm1
	por	%xmm1,%xmm0
.byte	102,72,15,126,192
	incl	%esi
	testq	%rax,%rax
	cmovnzl	%esi,%eax
	xorl	$1,%eax
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	vec_is_zero_16x,.-vec_is_zero_16x
.globl	vec_is_equal_16x
.hidden	vec_is_equal_16x
.type	vec_is_equal_16x,@function
.align	32
vec_is_equal_16x:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	shrl	$4,%edx
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movdqu	(%rdi),%xmm0
	movdqu	(%rsi),%xmm1
	subq	%rdi,%rsi
	leaq	16(%rdi),%rdi
	pxor	%xmm1,%xmm0

.Loop_is_equal:
	decl	%edx
	jz	.Loop_is_equal_done
	movdqu	(%rdi),%xmm1
	movdqu	(%rdi,%rsi,1),%xmm2
	leaq	16(%rdi),%rdi
	pxor	%xmm2,%xmm1
	por	%xmm1,%xmm0
	jmp	.Loop_is_equal

.Loop_is_equal_done:
	pshufd	$0x4e,%xmm0,%xmm1
	por	%xmm1,%xmm0
.byte	102,72,15,126,192
	incl	%edx
	testq	%rax,%rax
	cmovnzl	%edx,%eax
	xorl	$1,%eax
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif
.cfi_endproc
.size	vec_is_equal_16x,.-vec_is_equal_16x

.section	.note.GNU-stack,"",@progbits
#ifndef	__SGX_LVI_HARDENING__
.section	.note.gnu.property,"a",@note
	.long	4,2f-1f,5
	.byte	0x47,0x4E,0x55,0
1:	.long	0xc0000002,4,3
.align	8
2:
#endif
