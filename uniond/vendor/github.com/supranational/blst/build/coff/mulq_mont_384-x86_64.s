.comm	__blst_platform_cap,4
.text	







.def	__subq_mod_384x384;	.scl 3;	.type 32;	.endef
.p2align	5
__subq_mod_384x384:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13
	movq	48(%rsi),%r14

	subq	0(%rdx),%r8
	movq	56(%rsi),%r15
	sbbq	8(%rdx),%r9
	movq	64(%rsi),%rax
	sbbq	16(%rdx),%r10
	movq	72(%rsi),%rbx
	sbbq	24(%rdx),%r11
	movq	80(%rsi),%rbp
	sbbq	32(%rdx),%r12
	movq	88(%rsi),%rsi
	sbbq	40(%rdx),%r13
	movq	%r8,0(%rdi)
	sbbq	48(%rdx),%r14
	movq	0(%rcx),%r8
	movq	%r9,8(%rdi)
	sbbq	56(%rdx),%r15
	movq	8(%rcx),%r9
	movq	%r10,16(%rdi)
	sbbq	64(%rdx),%rax
	movq	16(%rcx),%r10
	movq	%r11,24(%rdi)
	sbbq	72(%rdx),%rbx
	movq	24(%rcx),%r11
	movq	%r12,32(%rdi)
	sbbq	80(%rdx),%rbp
	movq	32(%rcx),%r12
	movq	%r13,40(%rdi)
	sbbq	88(%rdx),%rsi
	movq	40(%rcx),%r13
	sbbq	%rdx,%rdx

	andq	%rdx,%r8
	andq	%rdx,%r9
	andq	%rdx,%r10
	andq	%rdx,%r11
	andq	%rdx,%r12
	andq	%rdx,%r13

	addq	%r8,%r14
	adcq	%r9,%r15
	movq	%r14,48(%rdi)
	adcq	%r10,%rax
	movq	%r15,56(%rdi)
	adcq	%r11,%rbx
	movq	%rax,64(%rdi)
	adcq	%r12,%rbp
	movq	%rbx,72(%rdi)
	adcq	%r13,%rsi
	movq	%rbp,80(%rdi)
	movq	%rsi,88(%rdi)

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif


.def	__addq_mod_384;	.scl 3;	.type 32;	.endef
.p2align	5
__addq_mod_384:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

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


.def	__subq_mod_384;	.scl 3;	.type 32;	.endef
.p2align	5
__subq_mod_384:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

__subq_mod_384_a_is_loaded:
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

.globl	mul_mont_384x

.def	mul_mont_384x;	.scl 2;	.type 32;	.endef
.p2align	5
mul_mont_384x:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_mul_mont_384x:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	movq	40(%rsp),%r8
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	mul_mont_384x$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$328,%rsp

.LSEH_body_mul_mont_384x:


	movq	%rdx,%rbx
	movq	%rdi,32(%rsp)
	movq	%rsi,24(%rsp)
	movq	%rdx,16(%rsp)
	movq	%rcx,8(%rsp)
	movq	%r8,0(%rsp)




	leaq	40(%rsp),%rdi
	call	__mulq_384


	leaq	48(%rbx),%rbx
	leaq	48(%rsi),%rsi
	leaq	40+96(%rsp),%rdi
	call	__mulq_384


	movq	8(%rsp),%rcx
	leaq	-48(%rsi),%rdx
	leaq	40+192+48(%rsp),%rdi
	call	__addq_mod_384

	movq	16(%rsp),%rsi
	leaq	48(%rsi),%rdx
	leaq	-48(%rdi),%rdi
	call	__addq_mod_384

	leaq	(%rdi),%rbx
	leaq	48(%rdi),%rsi
	call	__mulq_384


	leaq	(%rdi),%rsi
	leaq	40(%rsp),%rdx
	movq	8(%rsp),%rcx
	call	__subq_mod_384x384

	leaq	(%rdi),%rsi
	leaq	-96(%rdi),%rdx
	call	__subq_mod_384x384


	leaq	40(%rsp),%rsi
	leaq	40+96(%rsp),%rdx
	leaq	40(%rsp),%rdi
	call	__subq_mod_384x384

	movq	%rcx,%rbx


	leaq	40(%rsp),%rsi
	movq	0(%rsp),%rcx
	movq	32(%rsp),%rdi
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384


	leaq	40+192(%rsp),%rsi
	movq	0(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

	leaq	328(%rsp),%r8
	movq	0(%r8),%r15

	movq	8(%r8),%r14

	movq	16(%r8),%r13

	movq	24(%r8),%r12

	movq	32(%r8),%rbx

	movq	40(%r8),%rbp

	leaq	48(%r8),%rsp

.LSEH_epilogue_mul_mont_384x:
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

.LSEH_end_mul_mont_384x:
.globl	sqr_mont_384x

.def	sqr_mont_384x;	.scl 2;	.type 32;	.endef
.p2align	5
sqr_mont_384x:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sqr_mont_384x:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	sqr_mont_384x$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$136,%rsp

.LSEH_body_sqr_mont_384x:


	movq	%rcx,0(%rsp)
	movq	%rdx,%rcx
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)


	leaq	48(%rsi),%rdx
	leaq	32(%rsp),%rdi
	call	__addq_mod_384


	movq	16(%rsp),%rsi
	leaq	48(%rsi),%rdx
	leaq	32+48(%rsp),%rdi
	call	__subq_mod_384


	movq	16(%rsp),%rsi
	leaq	48(%rsi),%rbx

	movq	48(%rsi),%rax
	movq	0(%rsi),%r14
	movq	8(%rsi),%r15
	movq	16(%rsi),%r12
	movq	24(%rsi),%r13

	call	__mulq_mont_384
	addq	%r14,%r14
	adcq	%r15,%r15
	adcq	%r8,%r8
	movq	%r14,%r12
	adcq	%r9,%r9
	movq	%r15,%r13
	adcq	%r10,%r10
	movq	%r8,%rax
	adcq	%r11,%r11
	movq	%r9,%rbx
	sbbq	%rdx,%rdx

	subq	0(%rcx),%r14
	sbbq	8(%rcx),%r15
	movq	%r10,%rbp
	sbbq	16(%rcx),%r8
	sbbq	24(%rcx),%r9
	sbbq	32(%rcx),%r10
	movq	%r11,%rsi
	sbbq	40(%rcx),%r11
	sbbq	$0,%rdx

	cmovcq	%r12,%r14
	cmovcq	%r13,%r15
	cmovcq	%rax,%r8
	movq	%r14,48(%rdi)
	cmovcq	%rbx,%r9
	movq	%r15,56(%rdi)
	cmovcq	%rbp,%r10
	movq	%r8,64(%rdi)
	cmovcq	%rsi,%r11
	movq	%r9,72(%rdi)
	movq	%r10,80(%rdi)
	movq	%r11,88(%rdi)

	leaq	32(%rsp),%rsi
	leaq	32+48(%rsp),%rbx

	movq	32+48(%rsp),%rax
	movq	32+0(%rsp),%r14
	movq	32+8(%rsp),%r15
	movq	32+16(%rsp),%r12
	movq	32+24(%rsp),%r13

	call	__mulq_mont_384

	leaq	136(%rsp),%r8
	movq	0(%r8),%r15

	movq	8(%r8),%r14

	movq	16(%r8),%r13

	movq	24(%r8),%r12

	movq	32(%r8),%rbx

	movq	40(%r8),%rbp

	leaq	48(%r8),%rsp

.LSEH_epilogue_sqr_mont_384x:
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

.LSEH_end_sqr_mont_384x:

.globl	mul_382x

.def	mul_382x;	.scl 2;	.type 32;	.endef
.p2align	5
mul_382x:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_mul_382x:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	mul_382x$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$136,%rsp

.LSEH_body_mul_382x:


	leaq	96(%rdi),%rdi
	movq	%rsi,0(%rsp)
	movq	%rdx,8(%rsp)
	movq	%rdi,16(%rsp)
	movq	%rcx,24(%rsp)


	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	addq	48(%rsi),%r8
	adcq	56(%rsi),%r9
	adcq	64(%rsi),%r10
	adcq	72(%rsi),%r11
	adcq	80(%rsi),%r12
	adcq	88(%rsi),%r13

	movq	%r8,32+0(%rsp)
	movq	%r9,32+8(%rsp)
	movq	%r10,32+16(%rsp)
	movq	%r11,32+24(%rsp)
	movq	%r12,32+32(%rsp)
	movq	%r13,32+40(%rsp)


	movq	0(%rdx),%r8
	movq	8(%rdx),%r9
	movq	16(%rdx),%r10
	movq	24(%rdx),%r11
	movq	32(%rdx),%r12
	movq	40(%rdx),%r13

	addq	48(%rdx),%r8
	adcq	56(%rdx),%r9
	adcq	64(%rdx),%r10
	adcq	72(%rdx),%r11
	adcq	80(%rdx),%r12
	adcq	88(%rdx),%r13

	movq	%r8,32+48(%rsp)
	movq	%r9,32+56(%rsp)
	movq	%r10,32+64(%rsp)
	movq	%r11,32+72(%rsp)
	movq	%r12,32+80(%rsp)
	movq	%r13,32+88(%rsp)


	leaq	32+0(%rsp),%rsi
	leaq	32+48(%rsp),%rbx
	call	__mulq_384


	movq	0(%rsp),%rsi
	movq	8(%rsp),%rbx
	leaq	-96(%rdi),%rdi
	call	__mulq_384


	leaq	48(%rsi),%rsi
	leaq	48(%rbx),%rbx
	leaq	32(%rsp),%rdi
	call	__mulq_384


	movq	16(%rsp),%rsi
	leaq	32(%rsp),%rdx
	movq	24(%rsp),%rcx
	movq	%rsi,%rdi
	call	__subq_mod_384x384


	leaq	0(%rdi),%rsi
	leaq	-96(%rdi),%rdx
	call	__subq_mod_384x384


	leaq	-96(%rdi),%rsi
	leaq	32(%rsp),%rdx
	leaq	-96(%rdi),%rdi
	call	__subq_mod_384x384

	leaq	136(%rsp),%r8
	movq	0(%r8),%r15

	movq	8(%r8),%r14

	movq	16(%r8),%r13

	movq	24(%r8),%r12

	movq	32(%r8),%rbx

	movq	40(%r8),%rbp

	leaq	48(%r8),%rsp

.LSEH_epilogue_mul_382x:
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

.LSEH_end_mul_382x:
.globl	sqr_382x

.def	sqr_382x;	.scl 2;	.type 32;	.endef
.p2align	5
sqr_382x:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sqr_382x:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	sqr_382x$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	pushq	%rsi

.LSEH_body_sqr_382x:


	movq	%rdx,%rcx


	movq	0(%rsi),%r14
	movq	8(%rsi),%r15
	movq	16(%rsi),%rax
	movq	24(%rsi),%rbx
	movq	32(%rsi),%rbp
	movq	40(%rsi),%rdx

	movq	%r14,%r8
	addq	48(%rsi),%r14
	movq	%r15,%r9
	adcq	56(%rsi),%r15
	movq	%rax,%r10
	adcq	64(%rsi),%rax
	movq	%rbx,%r11
	adcq	72(%rsi),%rbx
	movq	%rbp,%r12
	adcq	80(%rsi),%rbp
	movq	%rdx,%r13
	adcq	88(%rsi),%rdx

	movq	%r14,0(%rdi)
	movq	%r15,8(%rdi)
	movq	%rax,16(%rdi)
	movq	%rbx,24(%rdi)
	movq	%rbp,32(%rdi)
	movq	%rdx,40(%rdi)


	leaq	48(%rsi),%rdx
	leaq	48(%rdi),%rdi
	call	__subq_mod_384_a_is_loaded


	leaq	(%rdi),%rsi
	leaq	-48(%rdi),%rbx
	leaq	-48(%rdi),%rdi
	call	__mulq_384


	movq	(%rsp),%rsi
	leaq	48(%rsi),%rbx
	leaq	96(%rdi),%rdi
	call	__mulq_384

	movq	0(%rdi),%r8
	movq	8(%rdi),%r9
	movq	16(%rdi),%r10
	movq	24(%rdi),%r11
	movq	32(%rdi),%r12
	movq	40(%rdi),%r13
	movq	48(%rdi),%r14
	movq	56(%rdi),%r15
	movq	64(%rdi),%rax
	movq	72(%rdi),%rbx
	movq	80(%rdi),%rbp
	addq	%r8,%r8
	movq	88(%rdi),%rdx
	adcq	%r9,%r9
	movq	%r8,0(%rdi)
	adcq	%r10,%r10
	movq	%r9,8(%rdi)
	adcq	%r11,%r11
	movq	%r10,16(%rdi)
	adcq	%r12,%r12
	movq	%r11,24(%rdi)
	adcq	%r13,%r13
	movq	%r12,32(%rdi)
	adcq	%r14,%r14
	movq	%r13,40(%rdi)
	adcq	%r15,%r15
	movq	%r14,48(%rdi)
	adcq	%rax,%rax
	movq	%r15,56(%rdi)
	adcq	%rbx,%rbx
	movq	%rax,64(%rdi)
	adcq	%rbp,%rbp
	movq	%rbx,72(%rdi)
	adcq	%rdx,%rdx
	movq	%rbp,80(%rdi)
	movq	%rdx,88(%rdi)

	movq	8(%rsp),%r15

	movq	16(%rsp),%r14

	movq	24(%rsp),%r13

	movq	32(%rsp),%r12

	movq	40(%rsp),%rbx

	movq	48(%rsp),%rbp

	leaq	56(%rsp),%rsp

.LSEH_epilogue_sqr_382x:
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

.LSEH_end_sqr_382x:
.globl	mul_384

.def	mul_384;	.scl 2;	.type 32;	.endef
.p2align	5
mul_384:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_mul_384:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	mul_384$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

.LSEH_body_mul_384:


	movq	%rdx,%rbx
	call	__mulq_384

	movq	0(%rsp),%r12

	movq	8(%rsp),%rbx

	movq	16(%rsp),%rbp

	leaq	24(%rsp),%rsp

.LSEH_epilogue_mul_384:
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

.LSEH_end_mul_384:

.def	__mulq_384;	.scl 3;	.type 32;	.endef
.p2align	5
__mulq_384:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rbx),%rax

	movq	%rax,%rbp
	mulq	0(%rsi)
	movq	%rax,0(%rdi)
	movq	%rbp,%rax
	movq	%rdx,%rcx

	mulq	8(%rsi)
	addq	%rax,%rcx
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	16(%rsi)
	addq	%rax,%r8
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	24(%rsi)
	addq	%rax,%r9
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	32(%rsi)
	addq	%rax,%r10
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	40(%rsi)
	addq	%rax,%r11
	movq	8(%rbx),%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	movq	%rax,%rbp
	mulq	0(%rsi)
	addq	%rax,%rcx
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rcx,8(%rdi)
	movq	%rdx,%rcx

	mulq	8(%rsi)
	addq	%rax,%r8
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r8,%rcx
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	16(%rsi)
	addq	%rax,%r9
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r9,%r8
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	24(%rsi)
	addq	%rax,%r10
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r10,%r9
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	32(%rsi)
	addq	%rax,%r11
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r11,%r10
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	40(%rsi)
	addq	%rax,%r12
	movq	16(%rbx),%rax
	adcq	$0,%rdx
	addq	%r12,%r11
	adcq	$0,%rdx
	movq	%rdx,%r12
	movq	%rax,%rbp
	mulq	0(%rsi)
	addq	%rax,%rcx
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rcx,16(%rdi)
	movq	%rdx,%rcx

	mulq	8(%rsi)
	addq	%rax,%r8
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r8,%rcx
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	16(%rsi)
	addq	%rax,%r9
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r9,%r8
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	24(%rsi)
	addq	%rax,%r10
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r10,%r9
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	32(%rsi)
	addq	%rax,%r11
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r11,%r10
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	40(%rsi)
	addq	%rax,%r12
	movq	24(%rbx),%rax
	adcq	$0,%rdx
	addq	%r12,%r11
	adcq	$0,%rdx
	movq	%rdx,%r12
	movq	%rax,%rbp
	mulq	0(%rsi)
	addq	%rax,%rcx
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rcx,24(%rdi)
	movq	%rdx,%rcx

	mulq	8(%rsi)
	addq	%rax,%r8
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r8,%rcx
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	16(%rsi)
	addq	%rax,%r9
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r9,%r8
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	24(%rsi)
	addq	%rax,%r10
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r10,%r9
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	32(%rsi)
	addq	%rax,%r11
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r11,%r10
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	40(%rsi)
	addq	%rax,%r12
	movq	32(%rbx),%rax
	adcq	$0,%rdx
	addq	%r12,%r11
	adcq	$0,%rdx
	movq	%rdx,%r12
	movq	%rax,%rbp
	mulq	0(%rsi)
	addq	%rax,%rcx
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rcx,32(%rdi)
	movq	%rdx,%rcx

	mulq	8(%rsi)
	addq	%rax,%r8
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r8,%rcx
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	16(%rsi)
	addq	%rax,%r9
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r9,%r8
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	24(%rsi)
	addq	%rax,%r10
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r10,%r9
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	32(%rsi)
	addq	%rax,%r11
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r11,%r10
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	40(%rsi)
	addq	%rax,%r12
	movq	40(%rbx),%rax
	adcq	$0,%rdx
	addq	%r12,%r11
	adcq	$0,%rdx
	movq	%rdx,%r12
	movq	%rax,%rbp
	mulq	0(%rsi)
	addq	%rax,%rcx
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rcx,40(%rdi)
	movq	%rdx,%rcx

	mulq	8(%rsi)
	addq	%rax,%r8
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r8,%rcx
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	16(%rsi)
	addq	%rax,%r9
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r9,%r8
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	24(%rsi)
	addq	%rax,%r10
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r10,%r9
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	32(%rsi)
	addq	%rax,%r11
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r11,%r10
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	40(%rsi)
	addq	%rax,%r12
	movq	%rax,%rax
	adcq	$0,%rdx
	addq	%r12,%r11
	adcq	$0,%rdx
	movq	%rdx,%r12
	movq	%rcx,48(%rdi)
	movq	%r8,56(%rdi)
	movq	%r9,64(%rdi)
	movq	%r10,72(%rdi)
	movq	%r11,80(%rdi)
	movq	%r12,88(%rdi)

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif

.globl	sqr_384

.def	sqr_384;	.scl 2;	.type 32;	.endef
.p2align	5
sqr_384:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sqr_384:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	sqr_384$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$8,%rsp

.LSEH_body_sqr_384:


	call	__sqrq_384

	movq	8(%rsp),%r15

	movq	16(%rsp),%r14

	movq	24(%rsp),%r13

	movq	32(%rsp),%r12

	movq	40(%rsp),%rbx

	movq	48(%rsp),%rbp

	leaq	56(%rsp),%rsp

.LSEH_epilogue_sqr_384:
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

.LSEH_end_sqr_384:

.def	__sqrq_384;	.scl 3;	.type 32;	.endef
.p2align	5
__sqrq_384:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%rax
	movq	8(%rsi),%r15
	movq	16(%rsi),%rcx
	movq	24(%rsi),%rbx


	movq	%rax,%r14
	mulq	%r15
	movq	%rax,%r9
	movq	%r14,%rax
	movq	32(%rsi),%rbp
	movq	%rdx,%r10

	mulq	%rcx
	addq	%rax,%r10
	movq	%r14,%rax
	adcq	$0,%rdx
	movq	40(%rsi),%rsi
	movq	%rdx,%r11

	mulq	%rbx
	addq	%rax,%r11
	movq	%r14,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	%rbp
	addq	%rax,%r12
	movq	%r14,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13

	mulq	%rsi
	addq	%rax,%r13
	movq	%r14,%rax
	adcq	$0,%rdx
	movq	%rdx,%r14

	mulq	%rax
	xorq	%r8,%r8
	movq	%rax,0(%rdi)
	movq	%r15,%rax
	addq	%r9,%r9
	adcq	$0,%r8
	addq	%rdx,%r9
	adcq	$0,%r8
	movq	%r9,8(%rdi)

	mulq	%rcx
	addq	%rax,%r11
	movq	%r15,%rax
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	%rbx
	addq	%rax,%r12
	movq	%r15,%rax
	adcq	$0,%rdx
	addq	%r9,%r12
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	%rbp
	addq	%rax,%r13
	movq	%r15,%rax
	adcq	$0,%rdx
	addq	%r9,%r13
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	%rsi
	addq	%rax,%r14
	movq	%r15,%rax
	adcq	$0,%rdx
	addq	%r9,%r14
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	%rax
	xorq	%r9,%r9
	addq	%rax,%r8
	movq	%rcx,%rax
	addq	%r10,%r10
	adcq	%r11,%r11
	adcq	$0,%r9
	addq	%r8,%r10
	adcq	%rdx,%r11
	adcq	$0,%r9
	movq	%r10,16(%rdi)

	mulq	%rbx
	addq	%rax,%r13
	movq	%rcx,%rax
	adcq	$0,%rdx
	movq	%r11,24(%rdi)
	movq	%rdx,%r8

	mulq	%rbp
	addq	%rax,%r14
	movq	%rcx,%rax
	adcq	$0,%rdx
	addq	%r8,%r14
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	%rsi
	addq	%rax,%r15
	movq	%rcx,%rax
	adcq	$0,%rdx
	addq	%r8,%r15
	adcq	$0,%rdx
	movq	%rdx,%rcx

	mulq	%rax
	xorq	%r11,%r11
	addq	%rax,%r9
	movq	%rbx,%rax
	addq	%r12,%r12
	adcq	%r13,%r13
	adcq	$0,%r11
	addq	%r9,%r12
	adcq	%rdx,%r13
	adcq	$0,%r11
	movq	%r12,32(%rdi)


	mulq	%rbp
	addq	%rax,%r15
	movq	%rbx,%rax
	adcq	$0,%rdx
	movq	%r13,40(%rdi)
	movq	%rdx,%r8

	mulq	%rsi
	addq	%rax,%rcx
	movq	%rbx,%rax
	adcq	$0,%rdx
	addq	%r8,%rcx
	adcq	$0,%rdx
	movq	%rdx,%rbx

	mulq	%rax
	xorq	%r12,%r12
	addq	%rax,%r11
	movq	%rbp,%rax
	addq	%r14,%r14
	adcq	%r15,%r15
	adcq	$0,%r12
	addq	%r11,%r14
	adcq	%rdx,%r15
	movq	%r14,48(%rdi)
	adcq	$0,%r12
	movq	%r15,56(%rdi)


	mulq	%rsi
	addq	%rax,%rbx
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	%rax
	xorq	%r13,%r13
	addq	%rax,%r12
	movq	%rsi,%rax
	addq	%rcx,%rcx
	adcq	%rbx,%rbx
	adcq	$0,%r13
	addq	%r12,%rcx
	adcq	%rdx,%rbx
	movq	%rcx,64(%rdi)
	adcq	$0,%r13
	movq	%rbx,72(%rdi)


	mulq	%rax
	addq	%r13,%rax
	addq	%rbp,%rbp
	adcq	$0,%rdx
	addq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rax,80(%rdi)
	movq	%rdx,88(%rdi)

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif


.globl	sqr_mont_384

.def	sqr_mont_384;	.scl 2;	.type 32;	.endef
.p2align	5
sqr_mont_384:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sqr_mont_384:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	sqr_mont_384$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$120,%rsp

.LSEH_body_sqr_mont_384:


	movq	%rcx,96(%rsp)
	movq	%rdx,104(%rsp)
	movq	%rdi,112(%rsp)

	movq	%rsp,%rdi
	call	__sqrq_384

	leaq	0(%rsp),%rsi
	movq	96(%rsp),%rcx
	movq	104(%rsp),%rbx
	movq	112(%rsp),%rdi
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

	leaq	120(%rsp),%r8
	movq	120(%rsp),%r15

	movq	8(%r8),%r14

	movq	16(%r8),%r13

	movq	24(%r8),%r12

	movq	32(%r8),%rbx

	movq	40(%r8),%rbp

	leaq	48(%r8),%rsp

.LSEH_epilogue_sqr_mont_384:
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

.LSEH_end_sqr_mont_384:



.globl	redc_mont_384

.def	redc_mont_384;	.scl 2;	.type 32;	.endef
.p2align	5
redc_mont_384:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_redc_mont_384:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	redc_mont_384$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$8,%rsp

.LSEH_body_redc_mont_384:


	movq	%rdx,%rbx
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

	movq	8(%rsp),%r15

	movq	16(%rsp),%r14

	movq	24(%rsp),%r13

	movq	32(%rsp),%r12

	movq	40(%rsp),%rbx

	movq	48(%rsp),%rbp

	leaq	56(%rsp),%rsp

.LSEH_epilogue_redc_mont_384:
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

.LSEH_end_redc_mont_384:




.globl	from_mont_384

.def	from_mont_384;	.scl 2;	.type 32;	.endef
.p2align	5
from_mont_384:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_from_mont_384:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	from_mont_384$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$8,%rsp

.LSEH_body_from_mont_384:


	movq	%rdx,%rbx
	call	__mulq_by_1_mont_384





	movq	%r15,%rcx
	movq	%r8,%rdx
	movq	%r9,%rbp

	subq	0(%rbx),%r14
	sbbq	8(%rbx),%r15
	movq	%r10,%r13
	sbbq	16(%rbx),%r8
	sbbq	24(%rbx),%r9
	sbbq	32(%rbx),%r10
	movq	%r11,%rsi
	sbbq	40(%rbx),%r11

	cmovcq	%rax,%r14
	cmovcq	%rcx,%r15
	cmovcq	%rdx,%r8
	movq	%r14,0(%rdi)
	cmovcq	%rbp,%r9
	movq	%r15,8(%rdi)
	cmovcq	%r13,%r10
	movq	%r8,16(%rdi)
	cmovcq	%rsi,%r11
	movq	%r9,24(%rdi)
	movq	%r10,32(%rdi)
	movq	%r11,40(%rdi)

	movq	8(%rsp),%r15

	movq	16(%rsp),%r14

	movq	24(%rsp),%r13

	movq	32(%rsp),%r12

	movq	40(%rsp),%rbx

	movq	48(%rsp),%rbp

	leaq	56(%rsp),%rsp

.LSEH_epilogue_from_mont_384:
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

.LSEH_end_from_mont_384:
.def	__mulq_by_1_mont_384;	.scl 3;	.type 32;	.endef
.p2align	5
__mulq_by_1_mont_384:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%rax
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

	movq	%rax,%r14
	imulq	%rcx,%rax
	movq	%rax,%r8

	mulq	0(%rbx)
	addq	%rax,%r14
	movq	%r8,%rax
	adcq	%rdx,%r14

	mulq	8(%rbx)
	addq	%rax,%r9
	movq	%r8,%rax
	adcq	$0,%rdx
	addq	%r14,%r9
	adcq	$0,%rdx
	movq	%rdx,%r14

	mulq	16(%rbx)
	addq	%rax,%r10
	movq	%r8,%rax
	adcq	$0,%rdx
	addq	%r14,%r10
	adcq	$0,%rdx
	movq	%rdx,%r14

	mulq	24(%rbx)
	addq	%rax,%r11
	movq	%r8,%rax
	adcq	$0,%rdx
	movq	%r9,%r15
	imulq	%rcx,%r9
	addq	%r14,%r11
	adcq	$0,%rdx
	movq	%rdx,%r14

	mulq	32(%rbx)
	addq	%rax,%r12
	movq	%r8,%rax
	adcq	$0,%rdx
	addq	%r14,%r12
	adcq	$0,%rdx
	movq	%rdx,%r14

	mulq	40(%rbx)
	addq	%rax,%r13
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%r14,%r13
	adcq	$0,%rdx
	movq	%rdx,%r14

	mulq	0(%rbx)
	addq	%rax,%r15
	movq	%r9,%rax
	adcq	%rdx,%r15

	mulq	8(%rbx)
	addq	%rax,%r10
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%r15,%r10
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	16(%rbx)
	addq	%rax,%r11
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%r15,%r11
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	24(%rbx)
	addq	%rax,%r12
	movq	%r9,%rax
	adcq	$0,%rdx
	movq	%r10,%r8
	imulq	%rcx,%r10
	addq	%r15,%r12
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	32(%rbx)
	addq	%rax,%r13
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%r15,%r13
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	40(%rbx)
	addq	%rax,%r14
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%r15,%r14
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	0(%rbx)
	addq	%rax,%r8
	movq	%r10,%rax
	adcq	%rdx,%r8

	mulq	8(%rbx)
	addq	%rax,%r11
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%r8,%r11
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	16(%rbx)
	addq	%rax,%r12
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%r8,%r12
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	24(%rbx)
	addq	%rax,%r13
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%r11,%r9
	imulq	%rcx,%r11
	addq	%r8,%r13
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	32(%rbx)
	addq	%rax,%r14
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%r8,%r14
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	40(%rbx)
	addq	%rax,%r15
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%r8,%r15
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	0(%rbx)
	addq	%rax,%r9
	movq	%r11,%rax
	adcq	%rdx,%r9

	mulq	8(%rbx)
	addq	%rax,%r12
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%r9,%r12
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	16(%rbx)
	addq	%rax,%r13
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%r9,%r13
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	24(%rbx)
	addq	%rax,%r14
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%r12,%r10
	imulq	%rcx,%r12
	addq	%r9,%r14
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	32(%rbx)
	addq	%rax,%r15
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%r9,%r15
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	40(%rbx)
	addq	%rax,%r8
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%r9,%r8
	adcq	$0,%rdx
	movq	%rdx,%r9

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
	addq	%rax,%r14
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%r10,%r14
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	24(%rbx)
	addq	%rax,%r15
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%r13,%r11
	imulq	%rcx,%r13
	addq	%r10,%r15
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	32(%rbx)
	addq	%rax,%r8
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%r10,%r8
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	40(%rbx)
	addq	%rax,%r9
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%r10,%r9
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
	addq	%rax,%r8
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%r11,%r8
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	32(%rbx)
	addq	%rax,%r9
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%r11,%r9
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	40(%rbx)
	addq	%rax,%r10
	movq	%r14,%rax
	adcq	$0,%rdx
	addq	%r11,%r10
	adcq	$0,%rdx
	movq	%rdx,%r11
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif


.def	__redq_tail_mont_384;	.scl 3;	.type 32;	.endef
.p2align	5
__redq_tail_mont_384:
	.byte	0xf3,0x0f,0x1e,0xfa

	addq	48(%rsi),%r14
	movq	%r14,%rax
	adcq	56(%rsi),%r15
	adcq	64(%rsi),%r8
	adcq	72(%rsi),%r9
	movq	%r15,%rcx
	adcq	80(%rsi),%r10
	adcq	88(%rsi),%r11
	sbbq	%r12,%r12




	movq	%r8,%rdx
	movq	%r9,%rbp

	subq	0(%rbx),%r14
	sbbq	8(%rbx),%r15
	movq	%r10,%r13
	sbbq	16(%rbx),%r8
	sbbq	24(%rbx),%r9
	sbbq	32(%rbx),%r10
	movq	%r11,%rsi
	sbbq	40(%rbx),%r11
	sbbq	$0,%r12

	cmovcq	%rax,%r14
	cmovcq	%rcx,%r15
	cmovcq	%rdx,%r8
	movq	%r14,0(%rdi)
	cmovcq	%rbp,%r9
	movq	%r15,8(%rdi)
	cmovcq	%r13,%r10
	movq	%r8,16(%rdi)
	cmovcq	%rsi,%r11
	movq	%r9,24(%rdi)
	movq	%r10,32(%rdi)
	movq	%r11,40(%rdi)

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif


.globl	sgn0_pty_mont_384

.def	sgn0_pty_mont_384;	.scl 2;	.type 32;	.endef
.p2align	5
sgn0_pty_mont_384:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sgn0_pty_mont_384:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	sgn0_pty_mont_384$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$8,%rsp

.LSEH_body_sgn0_pty_mont_384:


	movq	%rsi,%rbx
	leaq	0(%rdi),%rsi
	movq	%rdx,%rcx
	call	__mulq_by_1_mont_384

	xorq	%rax,%rax
	movq	%r14,%r13
	addq	%r14,%r14
	adcq	%r15,%r15
	adcq	%r8,%r8
	adcq	%r9,%r9
	adcq	%r10,%r10
	adcq	%r11,%r11
	adcq	$0,%rax

	subq	0(%rbx),%r14
	sbbq	8(%rbx),%r15
	sbbq	16(%rbx),%r8
	sbbq	24(%rbx),%r9
	sbbq	32(%rbx),%r10
	sbbq	40(%rbx),%r11
	sbbq	$0,%rax

	notq	%rax
	andq	$1,%r13
	andq	$2,%rax
	orq	%r13,%rax

	movq	8(%rsp),%r15

	movq	16(%rsp),%r14

	movq	24(%rsp),%r13

	movq	32(%rsp),%r12

	movq	40(%rsp),%rbx

	movq	48(%rsp),%rbp

	leaq	56(%rsp),%rsp

.LSEH_epilogue_sgn0_pty_mont_384:
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

.LSEH_end_sgn0_pty_mont_384:

.globl	sgn0_pty_mont_384x

.def	sgn0_pty_mont_384x;	.scl 2;	.type 32;	.endef
.p2align	5
sgn0_pty_mont_384x:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sgn0_pty_mont_384x:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	sgn0_pty_mont_384x$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$8,%rsp

.LSEH_body_sgn0_pty_mont_384x:


	movq	%rsi,%rbx
	leaq	48(%rdi),%rsi
	movq	%rdx,%rcx
	call	__mulq_by_1_mont_384

	movq	%r14,%r12
	orq	%r15,%r14
	orq	%r8,%r14
	orq	%r9,%r14
	orq	%r10,%r14
	orq	%r11,%r14

	leaq	0(%rdi),%rsi
	xorq	%rdi,%rdi
	movq	%r12,%r13
	addq	%r12,%r12
	adcq	%r15,%r15
	adcq	%r8,%r8
	adcq	%r9,%r9
	adcq	%r10,%r10
	adcq	%r11,%r11
	adcq	$0,%rdi

	subq	0(%rbx),%r12
	sbbq	8(%rbx),%r15
	sbbq	16(%rbx),%r8
	sbbq	24(%rbx),%r9
	sbbq	32(%rbx),%r10
	sbbq	40(%rbx),%r11
	sbbq	$0,%rdi

	movq	%r14,0(%rsp)
	notq	%rdi
	andq	$1,%r13
	andq	$2,%rdi
	orq	%r13,%rdi

	call	__mulq_by_1_mont_384

	movq	%r14,%r12
	orq	%r15,%r14
	orq	%r8,%r14
	orq	%r9,%r14
	orq	%r10,%r14
	orq	%r11,%r14

	xorq	%rax,%rax
	movq	%r12,%r13
	addq	%r12,%r12
	adcq	%r15,%r15
	adcq	%r8,%r8
	adcq	%r9,%r9
	adcq	%r10,%r10
	adcq	%r11,%r11
	adcq	$0,%rax

	subq	0(%rbx),%r12
	sbbq	8(%rbx),%r15
	sbbq	16(%rbx),%r8
	sbbq	24(%rbx),%r9
	sbbq	32(%rbx),%r10
	sbbq	40(%rbx),%r11
	sbbq	$0,%rax

	movq	0(%rsp),%r12

	notq	%rax

	testq	%r14,%r14
	cmovzq	%rdi,%r13

	testq	%r12,%r12
	cmovnzq	%rdi,%rax

	andq	$1,%r13
	andq	$2,%rax
	orq	%r13,%rax

	movq	8(%rsp),%r15

	movq	16(%rsp),%r14

	movq	24(%rsp),%r13

	movq	32(%rsp),%r12

	movq	40(%rsp),%rbx

	movq	48(%rsp),%rbp

	leaq	56(%rsp),%rsp

.LSEH_epilogue_sgn0_pty_mont_384x:
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

.LSEH_end_sgn0_pty_mont_384x:
.globl	mul_mont_384

.def	mul_mont_384;	.scl 2;	.type 32;	.endef
.p2align	5
mul_mont_384:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_mul_mont_384:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	movq	40(%rsp),%r8
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	mul_mont_384$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$24,%rsp

.LSEH_body_mul_mont_384:


	movq	0(%rdx),%rax
	movq	0(%rsi),%r14
	movq	8(%rsi),%r15
	movq	16(%rsi),%r12
	movq	24(%rsi),%r13
	movq	%rdx,%rbx
	movq	%r8,0(%rsp)
	movq	%rdi,8(%rsp)

	call	__mulq_mont_384

	movq	24(%rsp),%r15

	movq	32(%rsp),%r14

	movq	40(%rsp),%r13

	movq	48(%rsp),%r12

	movq	56(%rsp),%rbx

	movq	64(%rsp),%rbp

	leaq	72(%rsp),%rsp

.LSEH_epilogue_mul_mont_384:
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

.LSEH_end_mul_mont_384:
.def	__mulq_mont_384;	.scl 3;	.type 32;	.endef
.p2align	5
__mulq_mont_384:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	%rax,%rdi
	mulq	%r14
	movq	%rax,%r8
	movq	%rdi,%rax
	movq	%rdx,%r9

	mulq	%r15
	addq	%rax,%r9
	movq	%rdi,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	%r12
	addq	%rax,%r10
	movq	%rdi,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11

	movq	%r8,%rbp
	imulq	8(%rsp),%r8

	mulq	%r13
	addq	%rax,%r11
	movq	%rdi,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	32(%rsi)
	addq	%rax,%r12
	movq	%rdi,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13

	mulq	40(%rsi)
	addq	%rax,%r13
	movq	%r8,%rax
	adcq	$0,%rdx
	xorq	%r15,%r15
	movq	%rdx,%r14

	mulq	0(%rcx)
	addq	%rax,%rbp
	movq	%r8,%rax
	adcq	%rdx,%rbp

	mulq	8(%rcx)
	addq	%rax,%r9
	movq	%r8,%rax
	adcq	$0,%rdx
	addq	%rbp,%r9
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	16(%rcx)
	addq	%rax,%r10
	movq	%r8,%rax
	adcq	$0,%rdx
	addq	%rbp,%r10
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	24(%rcx)
	addq	%rbp,%r11
	adcq	$0,%rdx
	addq	%rax,%r11
	movq	%r8,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	32(%rcx)
	addq	%rax,%r12
	movq	%r8,%rax
	adcq	$0,%rdx
	addq	%rbp,%r12
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	40(%rcx)
	addq	%rax,%r13
	movq	8(%rbx),%rax
	adcq	$0,%rdx
	addq	%rbp,%r13
	adcq	%rdx,%r14
	adcq	$0,%r15

	movq	%rax,%rdi
	mulq	0(%rsi)
	addq	%rax,%r9
	movq	%rdi,%rax
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	8(%rsi)
	addq	%rax,%r10
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r8,%r10
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	16(%rsi)
	addq	%rax,%r11
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r8,%r11
	adcq	$0,%rdx
	movq	%rdx,%r8

	movq	%r9,%rbp
	imulq	8(%rsp),%r9

	mulq	24(%rsi)
	addq	%rax,%r12
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r8,%r12
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	32(%rsi)
	addq	%rax,%r13
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r8,%r13
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	40(%rsi)
	addq	%r8,%r14
	adcq	$0,%rdx
	xorq	%r8,%r8
	addq	%rax,%r14
	movq	%r9,%rax
	adcq	%rdx,%r15
	adcq	$0,%r8

	mulq	0(%rcx)
	addq	%rax,%rbp
	movq	%r9,%rax
	adcq	%rdx,%rbp

	mulq	8(%rcx)
	addq	%rax,%r10
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%rbp,%r10
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
	addq	%rbp,%r12
	adcq	$0,%rdx
	addq	%rax,%r12
	movq	%r9,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	32(%rcx)
	addq	%rax,%r13
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%rbp,%r13
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	40(%rcx)
	addq	%rax,%r14
	movq	16(%rbx),%rax
	adcq	$0,%rdx
	addq	%rbp,%r14
	adcq	%rdx,%r15
	adcq	$0,%r8

	movq	%rax,%rdi
	mulq	0(%rsi)
	addq	%rax,%r10
	movq	%rdi,%rax
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	8(%rsi)
	addq	%rax,%r11
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r9,%r11
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	16(%rsi)
	addq	%rax,%r12
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r9,%r12
	adcq	$0,%rdx
	movq	%rdx,%r9

	movq	%r10,%rbp
	imulq	8(%rsp),%r10

	mulq	24(%rsi)
	addq	%rax,%r13
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r9,%r13
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	32(%rsi)
	addq	%rax,%r14
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r9,%r14
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	40(%rsi)
	addq	%r9,%r15
	adcq	$0,%rdx
	xorq	%r9,%r9
	addq	%rax,%r15
	movq	%r10,%rax
	adcq	%rdx,%r8
	adcq	$0,%r9

	mulq	0(%rcx)
	addq	%rax,%rbp
	movq	%r10,%rax
	adcq	%rdx,%rbp

	mulq	8(%rcx)
	addq	%rax,%r11
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%rbp,%r11
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
	addq	%rbp,%r13
	adcq	$0,%rdx
	addq	%rax,%r13
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	32(%rcx)
	addq	%rax,%r14
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%rbp,%r14
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	40(%rcx)
	addq	%rax,%r15
	movq	24(%rbx),%rax
	adcq	$0,%rdx
	addq	%rbp,%r15
	adcq	%rdx,%r8
	adcq	$0,%r9

	movq	%rax,%rdi
	mulq	0(%rsi)
	addq	%rax,%r11
	movq	%rdi,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	8(%rsi)
	addq	%rax,%r12
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r10,%r12
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	16(%rsi)
	addq	%rax,%r13
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r10,%r13
	adcq	$0,%rdx
	movq	%rdx,%r10

	movq	%r11,%rbp
	imulq	8(%rsp),%r11

	mulq	24(%rsi)
	addq	%rax,%r14
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r10,%r14
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	32(%rsi)
	addq	%rax,%r15
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r10,%r15
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	40(%rsi)
	addq	%r10,%r8
	adcq	$0,%rdx
	xorq	%r10,%r10
	addq	%rax,%r8
	movq	%r11,%rax
	adcq	%rdx,%r9
	adcq	$0,%r10

	mulq	0(%rcx)
	addq	%rax,%rbp
	movq	%r11,%rax
	adcq	%rdx,%rbp

	mulq	8(%rcx)
	addq	%rax,%r12
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%rbp,%r12
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
	addq	%rbp,%r14
	adcq	$0,%rdx
	addq	%rax,%r14
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	32(%rcx)
	addq	%rax,%r15
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%rbp,%r15
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	40(%rcx)
	addq	%rax,%r8
	movq	32(%rbx),%rax
	adcq	$0,%rdx
	addq	%rbp,%r8
	adcq	%rdx,%r9
	adcq	$0,%r10

	movq	%rax,%rdi
	mulq	0(%rsi)
	addq	%rax,%r12
	movq	%rdi,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	8(%rsi)
	addq	%rax,%r13
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r11,%r13
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	16(%rsi)
	addq	%rax,%r14
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r11,%r14
	adcq	$0,%rdx
	movq	%rdx,%r11

	movq	%r12,%rbp
	imulq	8(%rsp),%r12

	mulq	24(%rsi)
	addq	%rax,%r15
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r11,%r15
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	32(%rsi)
	addq	%rax,%r8
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r11,%r8
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	40(%rsi)
	addq	%r11,%r9
	adcq	$0,%rdx
	xorq	%r11,%r11
	addq	%rax,%r9
	movq	%r12,%rax
	adcq	%rdx,%r10
	adcq	$0,%r11

	mulq	0(%rcx)
	addq	%rax,%rbp
	movq	%r12,%rax
	adcq	%rdx,%rbp

	mulq	8(%rcx)
	addq	%rax,%r13
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%rbp,%r13
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	16(%rcx)
	addq	%rax,%r14
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%rbp,%r14
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	24(%rcx)
	addq	%rbp,%r15
	adcq	$0,%rdx
	addq	%rax,%r15
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	32(%rcx)
	addq	%rax,%r8
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%rbp,%r8
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	40(%rcx)
	addq	%rax,%r9
	movq	40(%rbx),%rax
	adcq	$0,%rdx
	addq	%rbp,%r9
	adcq	%rdx,%r10
	adcq	$0,%r11

	movq	%rax,%rdi
	mulq	0(%rsi)
	addq	%rax,%r13
	movq	%rdi,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	8(%rsi)
	addq	%rax,%r14
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r12,%r14
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	16(%rsi)
	addq	%rax,%r15
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r12,%r15
	adcq	$0,%rdx
	movq	%rdx,%r12

	movq	%r13,%rbp
	imulq	8(%rsp),%r13

	mulq	24(%rsi)
	addq	%rax,%r8
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r12,%r8
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	32(%rsi)
	addq	%rax,%r9
	movq	%rdi,%rax
	adcq	$0,%rdx
	addq	%r12,%r9
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	40(%rsi)
	addq	%r12,%r10
	adcq	$0,%rdx
	xorq	%r12,%r12
	addq	%rax,%r10
	movq	%r13,%rax
	adcq	%rdx,%r11
	adcq	$0,%r12

	mulq	0(%rcx)
	addq	%rax,%rbp
	movq	%r13,%rax
	adcq	%rdx,%rbp

	mulq	8(%rcx)
	addq	%rax,%r14
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%rbp,%r14
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	16(%rcx)
	addq	%rax,%r15
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%rbp,%r15
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	24(%rcx)
	addq	%rbp,%r8
	adcq	$0,%rdx
	addq	%rax,%r8
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	32(%rcx)
	addq	%rax,%r9
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%rbp,%r9
	adcq	$0,%rdx
	movq	%rdx,%rbp

	mulq	40(%rcx)
	addq	%rax,%r10
	movq	%r14,%rax
	adcq	$0,%rdx
	addq	%rbp,%r10
	adcq	%rdx,%r11
	adcq	$0,%r12




	movq	16(%rsp),%rdi
	subq	0(%rcx),%r14
	movq	%r15,%rdx
	sbbq	8(%rcx),%r15
	movq	%r8,%rbx
	sbbq	16(%rcx),%r8
	movq	%r9,%rsi
	sbbq	24(%rcx),%r9
	movq	%r10,%rbp
	sbbq	32(%rcx),%r10
	movq	%r11,%r13
	sbbq	40(%rcx),%r11
	sbbq	$0,%r12

	cmovcq	%rax,%r14
	cmovcq	%rdx,%r15
	cmovcq	%rbx,%r8
	movq	%r14,0(%rdi)
	cmovcq	%rsi,%r9
	movq	%r15,8(%rdi)
	cmovcq	%rbp,%r10
	movq	%r8,16(%rdi)
	cmovcq	%r13,%r11
	movq	%r9,24(%rdi)
	movq	%r10,32(%rdi)
	movq	%r11,40(%rdi)

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif

.globl	sqr_n_mul_mont_384

.def	sqr_n_mul_mont_384;	.scl 2;	.type 32;	.endef
.p2align	5
sqr_n_mul_mont_384:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sqr_n_mul_mont_384:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	movq	40(%rsp),%r8
	movq	48(%rsp),%r9
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	sqr_n_mul_mont_384$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$136,%rsp

.LSEH_body_sqr_n_mul_mont_384:


	movq	%r8,0(%rsp)
	movq	%rdi,8(%rsp)
	movq	%rcx,16(%rsp)
	leaq	32(%rsp),%rdi
	movq	%r9,24(%rsp)
	movq	(%r9),%xmm2

.Loop_sqr_384:
	movd	%edx,%xmm1

	call	__sqrq_384

	leaq	0(%rdi),%rsi
	movq	0(%rsp),%rcx
	movq	16(%rsp),%rbx
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

	movd	%xmm1,%edx
	leaq	0(%rdi),%rsi
	decl	%edx
	jnz	.Loop_sqr_384

.byte	102,72,15,126,208
	movq	%rbx,%rcx
	movq	24(%rsp),%rbx






	movq	%r8,%r12
	movq	%r9,%r13

	call	__mulq_mont_384

	leaq	136(%rsp),%r8
	movq	136(%rsp),%r15

	movq	8(%r8),%r14

	movq	16(%r8),%r13

	movq	24(%r8),%r12

	movq	32(%r8),%rbx

	movq	40(%r8),%rbp

	leaq	48(%r8),%rsp

.LSEH_epilogue_sqr_n_mul_mont_384:
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

.LSEH_end_sqr_n_mul_mont_384:

.globl	sqr_n_mul_mont_383

.def	sqr_n_mul_mont_383;	.scl 2;	.type 32;	.endef
.p2align	5
sqr_n_mul_mont_383:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sqr_n_mul_mont_383:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	movq	40(%rsp),%r8
	movq	48(%rsp),%r9
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	sqr_n_mul_mont_383$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$136,%rsp

.LSEH_body_sqr_n_mul_mont_383:


	movq	%r8,0(%rsp)
	movq	%rdi,8(%rsp)
	movq	%rcx,16(%rsp)
	leaq	32(%rsp),%rdi
	movq	%r9,24(%rsp)
	movq	(%r9),%xmm2

.Loop_sqr_383:
	movd	%edx,%xmm1

	call	__sqrq_384

	leaq	0(%rdi),%rsi
	movq	0(%rsp),%rcx
	movq	16(%rsp),%rbx
	call	__mulq_by_1_mont_384

	movd	%xmm1,%edx
	addq	48(%rsi),%r14
	adcq	56(%rsi),%r15
	adcq	64(%rsi),%r8
	adcq	72(%rsi),%r9
	adcq	80(%rsi),%r10
	adcq	88(%rsi),%r11
	leaq	0(%rdi),%rsi

	movq	%r14,0(%rdi)
	movq	%r15,8(%rdi)
	movq	%r8,16(%rdi)
	movq	%r9,24(%rdi)
	movq	%r10,32(%rdi)
	movq	%r11,40(%rdi)

	decl	%edx
	jnz	.Loop_sqr_383

.byte	102,72,15,126,208
	movq	%rbx,%rcx
	movq	24(%rsp),%rbx






	movq	%r8,%r12
	movq	%r9,%r13

	call	__mulq_mont_384

	leaq	136(%rsp),%r8
	movq	136(%rsp),%r15

	movq	8(%r8),%r14

	movq	16(%r8),%r13

	movq	24(%r8),%r12

	movq	32(%r8),%rbx

	movq	40(%r8),%rbp

	leaq	48(%r8),%rsp

.LSEH_epilogue_sqr_n_mul_mont_383:
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

.LSEH_end_sqr_n_mul_mont_383:
.def	__mulq_mont_383_nonred;	.scl 3;	.type 32;	.endef
.p2align	5
__mulq_mont_383_nonred:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	%rax,%rbp
	mulq	%r14
	movq	%rax,%r8
	movq	%rbp,%rax
	movq	%rdx,%r9

	mulq	%r15
	addq	%rax,%r9
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	%r12
	addq	%rax,%r10
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11

	movq	%r8,%r15
	imulq	8(%rsp),%r8

	mulq	%r13
	addq	%rax,%r11
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	32(%rsi)
	addq	%rax,%r12
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13

	mulq	40(%rsi)
	addq	%rax,%r13
	movq	%r8,%rax
	adcq	$0,%rdx
	movq	%rdx,%r14

	mulq	0(%rcx)
	addq	%rax,%r15
	movq	%r8,%rax
	adcq	%rdx,%r15

	mulq	8(%rcx)
	addq	%rax,%r9
	movq	%r8,%rax
	adcq	$0,%rdx
	addq	%r15,%r9
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	16(%rcx)
	addq	%rax,%r10
	movq	%r8,%rax
	adcq	$0,%rdx
	addq	%r15,%r10
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	24(%rcx)
	addq	%r15,%r11
	adcq	$0,%rdx
	addq	%rax,%r11
	movq	%r8,%rax
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	32(%rcx)
	addq	%rax,%r12
	movq	%r8,%rax
	adcq	$0,%rdx
	addq	%r15,%r12
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	40(%rcx)
	addq	%rax,%r13
	movq	8(%rbx),%rax
	adcq	$0,%rdx
	addq	%r15,%r13
	adcq	%rdx,%r14

	movq	%rax,%rbp
	mulq	0(%rsi)
	addq	%rax,%r9
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	8(%rsi)
	addq	%rax,%r10
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r15,%r10
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	16(%rsi)
	addq	%rax,%r11
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r15,%r11
	adcq	$0,%rdx
	movq	%rdx,%r15

	movq	%r9,%r8
	imulq	8(%rsp),%r9

	mulq	24(%rsi)
	addq	%rax,%r12
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r15,%r12
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	32(%rsi)
	addq	%rax,%r13
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r15,%r13
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	40(%rsi)
	addq	%r15,%r14
	adcq	$0,%rdx
	addq	%rax,%r14
	movq	%r9,%rax
	adcq	$0,%rdx
	movq	%rdx,%r15

	mulq	0(%rcx)
	addq	%rax,%r8
	movq	%r9,%rax
	adcq	%rdx,%r8

	mulq	8(%rcx)
	addq	%rax,%r10
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%r8,%r10
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	16(%rcx)
	addq	%rax,%r11
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%r8,%r11
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	24(%rcx)
	addq	%r8,%r12
	adcq	$0,%rdx
	addq	%rax,%r12
	movq	%r9,%rax
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	32(%rcx)
	addq	%rax,%r13
	movq	%r9,%rax
	adcq	$0,%rdx
	addq	%r8,%r13
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	40(%rcx)
	addq	%rax,%r14
	movq	16(%rbx),%rax
	adcq	$0,%rdx
	addq	%r8,%r14
	adcq	%rdx,%r15

	movq	%rax,%rbp
	mulq	0(%rsi)
	addq	%rax,%r10
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	8(%rsi)
	addq	%rax,%r11
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r8,%r11
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	16(%rsi)
	addq	%rax,%r12
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r8,%r12
	adcq	$0,%rdx
	movq	%rdx,%r8

	movq	%r10,%r9
	imulq	8(%rsp),%r10

	mulq	24(%rsi)
	addq	%rax,%r13
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r8,%r13
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	32(%rsi)
	addq	%rax,%r14
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r8,%r14
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	40(%rsi)
	addq	%r8,%r15
	adcq	$0,%rdx
	addq	%rax,%r15
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%r8

	mulq	0(%rcx)
	addq	%rax,%r9
	movq	%r10,%rax
	adcq	%rdx,%r9

	mulq	8(%rcx)
	addq	%rax,%r11
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%r9,%r11
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	16(%rcx)
	addq	%rax,%r12
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%r9,%r12
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	24(%rcx)
	addq	%r9,%r13
	adcq	$0,%rdx
	addq	%rax,%r13
	movq	%r10,%rax
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	32(%rcx)
	addq	%rax,%r14
	movq	%r10,%rax
	adcq	$0,%rdx
	addq	%r9,%r14
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	40(%rcx)
	addq	%rax,%r15
	movq	24(%rbx),%rax
	adcq	$0,%rdx
	addq	%r9,%r15
	adcq	%rdx,%r8

	movq	%rax,%rbp
	mulq	0(%rsi)
	addq	%rax,%r11
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	8(%rsi)
	addq	%rax,%r12
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r9,%r12
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	16(%rsi)
	addq	%rax,%r13
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r9,%r13
	adcq	$0,%rdx
	movq	%rdx,%r9

	movq	%r11,%r10
	imulq	8(%rsp),%r11

	mulq	24(%rsi)
	addq	%rax,%r14
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r9,%r14
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	32(%rsi)
	addq	%rax,%r15
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r9,%r15
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	40(%rsi)
	addq	%r9,%r8
	adcq	$0,%rdx
	addq	%rax,%r8
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%r9

	mulq	0(%rcx)
	addq	%rax,%r10
	movq	%r11,%rax
	adcq	%rdx,%r10

	mulq	8(%rcx)
	addq	%rax,%r12
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%r10,%r12
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	16(%rcx)
	addq	%rax,%r13
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%r10,%r13
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	24(%rcx)
	addq	%r10,%r14
	adcq	$0,%rdx
	addq	%rax,%r14
	movq	%r11,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	32(%rcx)
	addq	%rax,%r15
	movq	%r11,%rax
	adcq	$0,%rdx
	addq	%r10,%r15
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	40(%rcx)
	addq	%rax,%r8
	movq	32(%rbx),%rax
	adcq	$0,%rdx
	addq	%r10,%r8
	adcq	%rdx,%r9

	movq	%rax,%rbp
	mulq	0(%rsi)
	addq	%rax,%r12
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	8(%rsi)
	addq	%rax,%r13
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r10,%r13
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	16(%rsi)
	addq	%rax,%r14
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r10,%r14
	adcq	$0,%rdx
	movq	%rdx,%r10

	movq	%r12,%r11
	imulq	8(%rsp),%r12

	mulq	24(%rsi)
	addq	%rax,%r15
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r10,%r15
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	32(%rsi)
	addq	%rax,%r8
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r10,%r8
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	40(%rsi)
	addq	%r10,%r9
	adcq	$0,%rdx
	addq	%rax,%r9
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r10

	mulq	0(%rcx)
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	%rdx,%r11

	mulq	8(%rcx)
	addq	%rax,%r13
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%r11,%r13
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	16(%rcx)
	addq	%rax,%r14
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%r11,%r14
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	24(%rcx)
	addq	%r11,%r15
	adcq	$0,%rdx
	addq	%rax,%r15
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	32(%rcx)
	addq	%rax,%r8
	movq	%r12,%rax
	adcq	$0,%rdx
	addq	%r11,%r8
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	40(%rcx)
	addq	%rax,%r9
	movq	40(%rbx),%rax
	adcq	$0,%rdx
	addq	%r11,%r9
	adcq	%rdx,%r10

	movq	%rax,%rbp
	mulq	0(%rsi)
	addq	%rax,%r13
	movq	%rbp,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	8(%rsi)
	addq	%rax,%r14
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r11,%r14
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	16(%rsi)
	addq	%rax,%r15
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r11,%r15
	adcq	$0,%rdx
	movq	%rdx,%r11

	movq	%r13,%r12
	imulq	8(%rsp),%r13

	mulq	24(%rsi)
	addq	%rax,%r8
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r11,%r8
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	32(%rsi)
	addq	%rax,%r9
	movq	%rbp,%rax
	adcq	$0,%rdx
	addq	%r11,%r9
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	40(%rsi)
	addq	%r11,%r10
	adcq	$0,%rdx
	addq	%rax,%r10
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r11

	mulq	0(%rcx)
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	%rdx,%r12

	mulq	8(%rcx)
	addq	%rax,%r14
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%r12,%r14
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	16(%rcx)
	addq	%rax,%r15
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%r12,%r15
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	24(%rcx)
	addq	%r12,%r8
	adcq	$0,%rdx
	addq	%rax,%r8
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	32(%rcx)
	addq	%rax,%r9
	movq	%r13,%rax
	adcq	$0,%rdx
	addq	%r12,%r9
	adcq	$0,%rdx
	movq	%rdx,%r12

	mulq	40(%rcx)
	addq	%rax,%r10
	movq	%r14,%rax
	adcq	$0,%rdx
	addq	%r12,%r10
	adcq	%rdx,%r11
	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rdx
	lfence
	jmpq	*%rdx
	ud2
#else
	.byte	0xf3,0xc3
#endif

.globl	sqr_mont_382x

.def	sqr_mont_382x;	.scl 2;	.type 32;	.endef
.p2align	5
sqr_mont_382x:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sqr_mont_382x:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
#ifdef __BLST_PORTABLE__
	testl	$1,__blst_platform_cap(%rip)
	jnz	sqr_mont_382x$1
#endif
	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$136,%rsp

.LSEH_body_sqr_mont_382x:


	movq	%rcx,0(%rsp)
	movq	%rdx,%rcx
	movq	%rsi,16(%rsp)
	movq	%rdi,24(%rsp)


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
	movq	%r11,%rdx
	adcq	72(%rsi),%r11
	movq	%r12,%rbx
	adcq	80(%rsi),%r12
	movq	%r13,%rbp
	adcq	88(%rsi),%r13

	subq	48(%rsi),%r14
	sbbq	56(%rsi),%r15
	sbbq	64(%rsi),%rax
	sbbq	72(%rsi),%rdx
	sbbq	80(%rsi),%rbx
	sbbq	88(%rsi),%rbp
	sbbq	%rdi,%rdi

	movq	%r8,32+0(%rsp)
	movq	%r9,32+8(%rsp)
	movq	%r10,32+16(%rsp)
	movq	%r11,32+24(%rsp)
	movq	%r12,32+32(%rsp)
	movq	%r13,32+40(%rsp)

	movq	%r14,32+48(%rsp)
	movq	%r15,32+56(%rsp)
	movq	%rax,32+64(%rsp)
	movq	%rdx,32+72(%rsp)
	movq	%rbx,32+80(%rsp)
	movq	%rbp,32+88(%rsp)
	movq	%rdi,32+96(%rsp)



	leaq	48(%rsi),%rbx

	movq	48(%rsi),%rax
	movq	0(%rsi),%r14
	movq	8(%rsi),%r15
	movq	16(%rsi),%r12
	movq	24(%rsi),%r13

	movq	24(%rsp),%rdi
	call	__mulq_mont_383_nonred
	addq	%r14,%r14
	adcq	%r15,%r15
	adcq	%r8,%r8
	adcq	%r9,%r9
	adcq	%r10,%r10
	adcq	%r11,%r11

	movq	%r14,48(%rdi)
	movq	%r15,56(%rdi)
	movq	%r8,64(%rdi)
	movq	%r9,72(%rdi)
	movq	%r10,80(%rdi)
	movq	%r11,88(%rdi)

	leaq	32(%rsp),%rsi
	leaq	32+48(%rsp),%rbx

	movq	32+48(%rsp),%rax
	movq	32+0(%rsp),%r14
	movq	32+8(%rsp),%r15
	movq	32+16(%rsp),%r12
	movq	32+24(%rsp),%r13

	call	__mulq_mont_383_nonred
	movq	32+96(%rsp),%rsi
	movq	32+0(%rsp),%r12
	movq	32+8(%rsp),%r13
	andq	%rsi,%r12
	movq	32+16(%rsp),%rax
	andq	%rsi,%r13
	movq	32+24(%rsp),%rbx
	andq	%rsi,%rax
	movq	32+32(%rsp),%rbp
	andq	%rsi,%rbx
	andq	%rsi,%rbp
	andq	32+40(%rsp),%rsi

	subq	%r12,%r14
	movq	0(%rcx),%r12
	sbbq	%r13,%r15
	movq	8(%rcx),%r13
	sbbq	%rax,%r8
	movq	16(%rcx),%rax
	sbbq	%rbx,%r9
	movq	24(%rcx),%rbx
	sbbq	%rbp,%r10
	movq	32(%rcx),%rbp
	sbbq	%rsi,%r11
	sbbq	%rsi,%rsi

	andq	%rsi,%r12
	andq	%rsi,%r13
	andq	%rsi,%rax
	andq	%rsi,%rbx
	andq	%rsi,%rbp
	andq	40(%rcx),%rsi

	addq	%r12,%r14
	adcq	%r13,%r15
	adcq	%rax,%r8
	adcq	%rbx,%r9
	adcq	%rbp,%r10
	adcq	%rsi,%r11

	movq	%r14,0(%rdi)
	movq	%r15,8(%rdi)
	movq	%r8,16(%rdi)
	movq	%r9,24(%rdi)
	movq	%r10,32(%rdi)
	movq	%r11,40(%rdi)
	leaq	136(%rsp),%r8
	movq	0(%r8),%r15

	movq	8(%r8),%r14

	movq	16(%r8),%r13

	movq	24(%r8),%r12

	movq	32(%r8),%rbx

	movq	40(%r8),%rbp

	leaq	48(%r8),%rsp

.LSEH_epilogue_sqr_mont_382x:
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

.LSEH_end_sqr_mont_382x:
.section	.pdata
.p2align	2
.rva	.LSEH_begin_mul_mont_384x
.rva	.LSEH_body_mul_mont_384x
.rva	.LSEH_info_mul_mont_384x_prologue

.rva	.LSEH_body_mul_mont_384x
.rva	.LSEH_epilogue_mul_mont_384x
.rva	.LSEH_info_mul_mont_384x_body

.rva	.LSEH_epilogue_mul_mont_384x
.rva	.LSEH_end_mul_mont_384x
.rva	.LSEH_info_mul_mont_384x_epilogue

.rva	.LSEH_begin_sqr_mont_384x
.rva	.LSEH_body_sqr_mont_384x
.rva	.LSEH_info_sqr_mont_384x_prologue

.rva	.LSEH_body_sqr_mont_384x
.rva	.LSEH_epilogue_sqr_mont_384x
.rva	.LSEH_info_sqr_mont_384x_body

.rva	.LSEH_epilogue_sqr_mont_384x
.rva	.LSEH_end_sqr_mont_384x
.rva	.LSEH_info_sqr_mont_384x_epilogue

.rva	.LSEH_begin_mul_382x
.rva	.LSEH_body_mul_382x
.rva	.LSEH_info_mul_382x_prologue

.rva	.LSEH_body_mul_382x
.rva	.LSEH_epilogue_mul_382x
.rva	.LSEH_info_mul_382x_body

.rva	.LSEH_epilogue_mul_382x
.rva	.LSEH_end_mul_382x
.rva	.LSEH_info_mul_382x_epilogue

.rva	.LSEH_begin_sqr_382x
.rva	.LSEH_body_sqr_382x
.rva	.LSEH_info_sqr_382x_prologue

.rva	.LSEH_body_sqr_382x
.rva	.LSEH_epilogue_sqr_382x
.rva	.LSEH_info_sqr_382x_body

.rva	.LSEH_epilogue_sqr_382x
.rva	.LSEH_end_sqr_382x
.rva	.LSEH_info_sqr_382x_epilogue

.rva	.LSEH_begin_mul_384
.rva	.LSEH_body_mul_384
.rva	.LSEH_info_mul_384_prologue

.rva	.LSEH_body_mul_384
.rva	.LSEH_epilogue_mul_384
.rva	.LSEH_info_mul_384_body

.rva	.LSEH_epilogue_mul_384
.rva	.LSEH_end_mul_384
.rva	.LSEH_info_mul_384_epilogue

.rva	.LSEH_begin_sqr_384
.rva	.LSEH_body_sqr_384
.rva	.LSEH_info_sqr_384_prologue

.rva	.LSEH_body_sqr_384
.rva	.LSEH_epilogue_sqr_384
.rva	.LSEH_info_sqr_384_body

.rva	.LSEH_epilogue_sqr_384
.rva	.LSEH_end_sqr_384
.rva	.LSEH_info_sqr_384_epilogue

.rva	.LSEH_begin_sqr_mont_384
.rva	.LSEH_body_sqr_mont_384
.rva	.LSEH_info_sqr_mont_384_prologue

.rva	.LSEH_body_sqr_mont_384
.rva	.LSEH_epilogue_sqr_mont_384
.rva	.LSEH_info_sqr_mont_384_body

.rva	.LSEH_epilogue_sqr_mont_384
.rva	.LSEH_end_sqr_mont_384
.rva	.LSEH_info_sqr_mont_384_epilogue

.rva	.LSEH_begin_redc_mont_384
.rva	.LSEH_body_redc_mont_384
.rva	.LSEH_info_redc_mont_384_prologue

.rva	.LSEH_body_redc_mont_384
.rva	.LSEH_epilogue_redc_mont_384
.rva	.LSEH_info_redc_mont_384_body

.rva	.LSEH_epilogue_redc_mont_384
.rva	.LSEH_end_redc_mont_384
.rva	.LSEH_info_redc_mont_384_epilogue

.rva	.LSEH_begin_from_mont_384
.rva	.LSEH_body_from_mont_384
.rva	.LSEH_info_from_mont_384_prologue

.rva	.LSEH_body_from_mont_384
.rva	.LSEH_epilogue_from_mont_384
.rva	.LSEH_info_from_mont_384_body

.rva	.LSEH_epilogue_from_mont_384
.rva	.LSEH_end_from_mont_384
.rva	.LSEH_info_from_mont_384_epilogue

.rva	.LSEH_begin_sgn0_pty_mont_384
.rva	.LSEH_body_sgn0_pty_mont_384
.rva	.LSEH_info_sgn0_pty_mont_384_prologue

.rva	.LSEH_body_sgn0_pty_mont_384
.rva	.LSEH_epilogue_sgn0_pty_mont_384
.rva	.LSEH_info_sgn0_pty_mont_384_body

.rva	.LSEH_epilogue_sgn0_pty_mont_384
.rva	.LSEH_end_sgn0_pty_mont_384
.rva	.LSEH_info_sgn0_pty_mont_384_epilogue

.rva	.LSEH_begin_sgn0_pty_mont_384x
.rva	.LSEH_body_sgn0_pty_mont_384x
.rva	.LSEH_info_sgn0_pty_mont_384x_prologue

.rva	.LSEH_body_sgn0_pty_mont_384x
.rva	.LSEH_epilogue_sgn0_pty_mont_384x
.rva	.LSEH_info_sgn0_pty_mont_384x_body

.rva	.LSEH_epilogue_sgn0_pty_mont_384x
.rva	.LSEH_end_sgn0_pty_mont_384x
.rva	.LSEH_info_sgn0_pty_mont_384x_epilogue

.rva	.LSEH_begin_mul_mont_384
.rva	.LSEH_body_mul_mont_384
.rva	.LSEH_info_mul_mont_384_prologue

.rva	.LSEH_body_mul_mont_384
.rva	.LSEH_epilogue_mul_mont_384
.rva	.LSEH_info_mul_mont_384_body

.rva	.LSEH_epilogue_mul_mont_384
.rva	.LSEH_end_mul_mont_384
.rva	.LSEH_info_mul_mont_384_epilogue

.rva	.LSEH_begin_sqr_n_mul_mont_384
.rva	.LSEH_body_sqr_n_mul_mont_384
.rva	.LSEH_info_sqr_n_mul_mont_384_prologue

.rva	.LSEH_body_sqr_n_mul_mont_384
.rva	.LSEH_epilogue_sqr_n_mul_mont_384
.rva	.LSEH_info_sqr_n_mul_mont_384_body

.rva	.LSEH_epilogue_sqr_n_mul_mont_384
.rva	.LSEH_end_sqr_n_mul_mont_384
.rva	.LSEH_info_sqr_n_mul_mont_384_epilogue

.rva	.LSEH_begin_sqr_n_mul_mont_383
.rva	.LSEH_body_sqr_n_mul_mont_383
.rva	.LSEH_info_sqr_n_mul_mont_383_prologue

.rva	.LSEH_body_sqr_n_mul_mont_383
.rva	.LSEH_epilogue_sqr_n_mul_mont_383
.rva	.LSEH_info_sqr_n_mul_mont_383_body

.rva	.LSEH_epilogue_sqr_n_mul_mont_383
.rva	.LSEH_end_sqr_n_mul_mont_383
.rva	.LSEH_info_sqr_n_mul_mont_383_epilogue

.rva	.LSEH_begin_sqr_mont_382x
.rva	.LSEH_body_sqr_mont_382x
.rva	.LSEH_info_sqr_mont_382x_prologue

.rva	.LSEH_body_sqr_mont_382x
.rva	.LSEH_epilogue_sqr_mont_382x
.rva	.LSEH_info_sqr_mont_382x_body

.rva	.LSEH_epilogue_sqr_mont_382x
.rva	.LSEH_end_sqr_mont_382x
.rva	.LSEH_info_sqr_mont_382x_epilogue

.section	.xdata
.p2align	3
.LSEH_info_mul_mont_384x_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_mul_mont_384x_body:
.byte	1,0,18,0
.byte	0x00,0xf4,0x29,0x00
.byte	0x00,0xe4,0x2a,0x00
.byte	0x00,0xd4,0x2b,0x00
.byte	0x00,0xc4,0x2c,0x00
.byte	0x00,0x34,0x2d,0x00
.byte	0x00,0x54,0x2e,0x00
.byte	0x00,0x74,0x30,0x00
.byte	0x00,0x64,0x31,0x00
.byte	0x00,0x01,0x2f,0x00
.byte	0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_mul_mont_384x_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sqr_mont_384x_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sqr_mont_384x_body:
.byte	1,0,18,0
.byte	0x00,0xf4,0x11,0x00
.byte	0x00,0xe4,0x12,0x00
.byte	0x00,0xd4,0x13,0x00
.byte	0x00,0xc4,0x14,0x00
.byte	0x00,0x34,0x15,0x00
.byte	0x00,0x54,0x16,0x00
.byte	0x00,0x74,0x18,0x00
.byte	0x00,0x64,0x19,0x00
.byte	0x00,0x01,0x17,0x00
.byte	0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_sqr_mont_384x_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_mul_382x_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_mul_382x_body:
.byte	1,0,18,0
.byte	0x00,0xf4,0x11,0x00
.byte	0x00,0xe4,0x12,0x00
.byte	0x00,0xd4,0x13,0x00
.byte	0x00,0xc4,0x14,0x00
.byte	0x00,0x34,0x15,0x00
.byte	0x00,0x54,0x16,0x00
.byte	0x00,0x74,0x18,0x00
.byte	0x00,0x64,0x19,0x00
.byte	0x00,0x01,0x17,0x00
.byte	0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_mul_382x_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sqr_382x_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sqr_382x_body:
.byte	1,0,17,0
.byte	0x00,0xf4,0x01,0x00
.byte	0x00,0xe4,0x02,0x00
.byte	0x00,0xd4,0x03,0x00
.byte	0x00,0xc4,0x04,0x00
.byte	0x00,0x34,0x05,0x00
.byte	0x00,0x54,0x06,0x00
.byte	0x00,0x74,0x08,0x00
.byte	0x00,0x64,0x09,0x00
.byte	0x00,0x62
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_sqr_382x_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_mul_384_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_mul_384_body:
.byte	1,0,11,0
.byte	0x00,0xc4,0x00,0x00
.byte	0x00,0x34,0x01,0x00
.byte	0x00,0x54,0x02,0x00
.byte	0x00,0x74,0x04,0x00
.byte	0x00,0x64,0x05,0x00
.byte	0x00,0x22
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.LSEH_info_mul_384_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sqr_384_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sqr_384_body:
.byte	1,0,17,0
.byte	0x00,0xf4,0x01,0x00
.byte	0x00,0xe4,0x02,0x00
.byte	0x00,0xd4,0x03,0x00
.byte	0x00,0xc4,0x04,0x00
.byte	0x00,0x34,0x05,0x00
.byte	0x00,0x54,0x06,0x00
.byte	0x00,0x74,0x08,0x00
.byte	0x00,0x64,0x09,0x00
.byte	0x00,0x62
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_sqr_384_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sqr_mont_384_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sqr_mont_384_body:
.byte	1,0,18,0
.byte	0x00,0xf4,0x0f,0x00
.byte	0x00,0xe4,0x10,0x00
.byte	0x00,0xd4,0x11,0x00
.byte	0x00,0xc4,0x12,0x00
.byte	0x00,0x34,0x13,0x00
.byte	0x00,0x54,0x14,0x00
.byte	0x00,0x74,0x16,0x00
.byte	0x00,0x64,0x17,0x00
.byte	0x00,0x01,0x15,0x00
.byte	0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_sqr_mont_384_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_redc_mont_384_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_redc_mont_384_body:
.byte	1,0,17,0
.byte	0x00,0xf4,0x01,0x00
.byte	0x00,0xe4,0x02,0x00
.byte	0x00,0xd4,0x03,0x00
.byte	0x00,0xc4,0x04,0x00
.byte	0x00,0x34,0x05,0x00
.byte	0x00,0x54,0x06,0x00
.byte	0x00,0x74,0x08,0x00
.byte	0x00,0x64,0x09,0x00
.byte	0x00,0x62
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_redc_mont_384_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_from_mont_384_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_from_mont_384_body:
.byte	1,0,17,0
.byte	0x00,0xf4,0x01,0x00
.byte	0x00,0xe4,0x02,0x00
.byte	0x00,0xd4,0x03,0x00
.byte	0x00,0xc4,0x04,0x00
.byte	0x00,0x34,0x05,0x00
.byte	0x00,0x54,0x06,0x00
.byte	0x00,0x74,0x08,0x00
.byte	0x00,0x64,0x09,0x00
.byte	0x00,0x62
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_from_mont_384_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sgn0_pty_mont_384_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sgn0_pty_mont_384_body:
.byte	1,0,17,0
.byte	0x00,0xf4,0x01,0x00
.byte	0x00,0xe4,0x02,0x00
.byte	0x00,0xd4,0x03,0x00
.byte	0x00,0xc4,0x04,0x00
.byte	0x00,0x34,0x05,0x00
.byte	0x00,0x54,0x06,0x00
.byte	0x00,0x74,0x08,0x00
.byte	0x00,0x64,0x09,0x00
.byte	0x00,0x62
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_sgn0_pty_mont_384_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sgn0_pty_mont_384x_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sgn0_pty_mont_384x_body:
.byte	1,0,17,0
.byte	0x00,0xf4,0x01,0x00
.byte	0x00,0xe4,0x02,0x00
.byte	0x00,0xd4,0x03,0x00
.byte	0x00,0xc4,0x04,0x00
.byte	0x00,0x34,0x05,0x00
.byte	0x00,0x54,0x06,0x00
.byte	0x00,0x74,0x08,0x00
.byte	0x00,0x64,0x09,0x00
.byte	0x00,0x62
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_sgn0_pty_mont_384x_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_mul_mont_384_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_mul_mont_384_body:
.byte	1,0,17,0
.byte	0x00,0xf4,0x03,0x00
.byte	0x00,0xe4,0x04,0x00
.byte	0x00,0xd4,0x05,0x00
.byte	0x00,0xc4,0x06,0x00
.byte	0x00,0x34,0x07,0x00
.byte	0x00,0x54,0x08,0x00
.byte	0x00,0x74,0x0a,0x00
.byte	0x00,0x64,0x0b,0x00
.byte	0x00,0x82
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_mul_mont_384_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sqr_n_mul_mont_384_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sqr_n_mul_mont_384_body:
.byte	1,0,18,0
.byte	0x00,0xf4,0x11,0x00
.byte	0x00,0xe4,0x12,0x00
.byte	0x00,0xd4,0x13,0x00
.byte	0x00,0xc4,0x14,0x00
.byte	0x00,0x34,0x15,0x00
.byte	0x00,0x54,0x16,0x00
.byte	0x00,0x74,0x18,0x00
.byte	0x00,0x64,0x19,0x00
.byte	0x00,0x01,0x17,0x00
.byte	0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_sqr_n_mul_mont_384_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sqr_n_mul_mont_383_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sqr_n_mul_mont_383_body:
.byte	1,0,18,0
.byte	0x00,0xf4,0x11,0x00
.byte	0x00,0xe4,0x12,0x00
.byte	0x00,0xd4,0x13,0x00
.byte	0x00,0xc4,0x14,0x00
.byte	0x00,0x34,0x15,0x00
.byte	0x00,0x54,0x16,0x00
.byte	0x00,0x74,0x18,0x00
.byte	0x00,0x64,0x19,0x00
.byte	0x00,0x01,0x17,0x00
.byte	0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_sqr_n_mul_mont_383_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sqr_mont_382x_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sqr_mont_382x_body:
.byte	1,0,18,0
.byte	0x00,0xf4,0x11,0x00
.byte	0x00,0xe4,0x12,0x00
.byte	0x00,0xd4,0x13,0x00
.byte	0x00,0xc4,0x14,0x00
.byte	0x00,0x34,0x15,0x00
.byte	0x00,0x54,0x16,0x00
.byte	0x00,0x74,0x18,0x00
.byte	0x00,0x64,0x19,0x00
.byte	0x00,0x01,0x17,0x00
.byte	0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_sqr_mont_382x_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

