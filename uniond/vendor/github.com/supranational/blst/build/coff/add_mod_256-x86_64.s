.text	

.globl	add_mod_256

.def	add_mod_256;	.scl 2;	.type 32;	.endef
.p2align	5
add_mod_256:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_add_mod_256:


	pushq	%rbp

	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	pushq	%rbx

	subq	$8,%rsp

.LSEH_body_add_mod_256:


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11

.Loaded_a_add_mod_256:
	addq	0(%rdx),%r8
	adcq	8(%rdx),%r9
	movq	%r8,%rax
	adcq	16(%rdx),%r10
	movq	%r9,%rsi
	adcq	24(%rdx),%r11
	sbbq	%rdx,%rdx

	movq	%r10,%rbx
	subq	0(%rcx),%r8
	sbbq	8(%rcx),%r9
	sbbq	16(%rcx),%r10
	movq	%r11,%rbp
	sbbq	24(%rcx),%r11
	sbbq	$0,%rdx

	cmovcq	%rax,%r8
	cmovcq	%rsi,%r9
	movq	%r8,0(%rdi)
	cmovcq	%rbx,%r10
	movq	%r9,8(%rdi)
	cmovcq	%rbp,%r11
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)

	movq	8(%rsp),%rbx

	movq	16(%rsp),%rbp

	leaq	24(%rsp),%rsp

.LSEH_epilogue_add_mod_256:
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

.LSEH_end_add_mod_256:


.globl	mul_by_3_mod_256

.def	mul_by_3_mod_256;	.scl 2;	.type 32;	.endef
.p2align	5
mul_by_3_mod_256:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_mul_by_3_mod_256:


	pushq	%rbp

	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	pushq	%rbx

	pushq	%r12

.LSEH_body_mul_by_3_mod_256:


	movq	%rdx,%rcx
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	%rsi,%rdx
	movq	24(%rsi),%r11

	call	__lshift_mod_256
	movq	0(%rsp),%r12

	jmp	.Loaded_a_add_mod_256

	movq	8(%rsp),%rbx

	movq	16(%rsp),%rbp

	leaq	24(%rsp),%rsp

.LSEH_epilogue_mul_by_3_mod_256:
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

.LSEH_end_mul_by_3_mod_256:

.def	__lshift_mod_256;	.scl 3;	.type 32;	.endef
.p2align	5
__lshift_mod_256:
	.byte	0xf3,0x0f,0x1e,0xfa

	addq	%r8,%r8
	adcq	%r9,%r9
	movq	%r8,%rax
	adcq	%r10,%r10
	movq	%r9,%rsi
	adcq	%r11,%r11
	sbbq	%r12,%r12

	movq	%r10,%rbx
	subq	0(%rcx),%r8
	sbbq	8(%rcx),%r9
	sbbq	16(%rcx),%r10
	movq	%r11,%rbp
	sbbq	24(%rcx),%r11
	sbbq	$0,%r12

	cmovcq	%rax,%r8
	cmovcq	%rsi,%r9
	cmovcq	%rbx,%r10
	cmovcq	%rbp,%r11

	
#ifdef	__SGX_LVI_HARDENING__
	popq	%rax
	lfence
	jmpq	*%rax
	ud2
#else
	.byte	0xf3,0xc3
#endif



.globl	lshift_mod_256

.def	lshift_mod_256;	.scl 2;	.type 32;	.endef
.p2align	5
lshift_mod_256:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_lshift_mod_256:


	pushq	%rbp

	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	pushq	%rbx

	pushq	%r12

.LSEH_body_lshift_mod_256:


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11

.Loop_lshift_mod_256:
	call	__lshift_mod_256
	decl	%edx
	jnz	.Loop_lshift_mod_256

	movq	%r8,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)

	movq	0(%rsp),%r12

	movq	8(%rsp),%rbx

	movq	16(%rsp),%rbp

	leaq	24(%rsp),%rsp

.LSEH_epilogue_lshift_mod_256:
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

.LSEH_end_lshift_mod_256:


.globl	rshift_mod_256

.def	rshift_mod_256;	.scl 2;	.type 32;	.endef
.p2align	5
rshift_mod_256:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_rshift_mod_256:


	pushq	%rbp

	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	pushq	%rbx

	subq	$8,%rsp

.LSEH_body_rshift_mod_256:


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%rbp
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11

.Loop_rshift_mod_256:
	movq	%rbp,%r8
	andq	$1,%rbp
	movq	0(%rcx),%rax
	negq	%rbp
	movq	8(%rcx),%rsi
	movq	16(%rcx),%rbx

	andq	%rbp,%rax
	andq	%rbp,%rsi
	andq	%rbp,%rbx
	andq	24(%rcx),%rbp

	addq	%rax,%r8
	adcq	%rsi,%r9
	adcq	%rbx,%r10
	adcq	%rbp,%r11
	sbbq	%rax,%rax

	shrq	$1,%r8
	movq	%r9,%rbp
	shrq	$1,%r9
	movq	%r10,%rbx
	shrq	$1,%r10
	movq	%r11,%rsi
	shrq	$1,%r11

	shlq	$63,%rbp
	shlq	$63,%rbx
	orq	%r8,%rbp
	shlq	$63,%rsi
	orq	%rbx,%r9
	shlq	$63,%rax
	orq	%rsi,%r10
	orq	%rax,%r11

	decl	%edx
	jnz	.Loop_rshift_mod_256

	movq	%rbp,0(%rdi)
	movq	%r9,8(%rdi)
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)

	movq	8(%rsp),%rbx

	movq	16(%rsp),%rbp

	leaq	24(%rsp),%rsp

.LSEH_epilogue_rshift_mod_256:
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

.LSEH_end_rshift_mod_256:


.globl	cneg_mod_256

.def	cneg_mod_256;	.scl 2;	.type 32;	.endef
.p2align	5
cneg_mod_256:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_cneg_mod_256:


	pushq	%rbp

	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	pushq	%rbx

	pushq	%r12

.LSEH_body_cneg_mod_256:


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r12
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	%r12,%r8
	movq	24(%rsi),%r11
	orq	%r9,%r12
	orq	%r10,%r12
	orq	%r11,%r12
	movq	$-1,%rbp

	movq	0(%rcx),%rax
	cmovnzq	%rbp,%r12
	movq	8(%rcx),%rsi
	movq	16(%rcx),%rbx
	andq	%r12,%rax
	movq	24(%rcx),%rbp
	andq	%r12,%rsi
	andq	%r12,%rbx
	andq	%r12,%rbp

	subq	%r8,%rax
	sbbq	%r9,%rsi
	sbbq	%r10,%rbx
	sbbq	%r11,%rbp

	orq	%rdx,%rdx

	cmovzq	%r8,%rax
	cmovzq	%r9,%rsi
	movq	%rax,0(%rdi)
	cmovzq	%r10,%rbx
	movq	%rsi,8(%rdi)
	cmovzq	%r11,%rbp
	movq	%rbx,16(%rdi)
	movq	%rbp,24(%rdi)

	movq	0(%rsp),%r12

	movq	8(%rsp),%rbx

	movq	16(%rsp),%rbp

	leaq	24(%rsp),%rsp

.LSEH_epilogue_cneg_mod_256:
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

.LSEH_end_cneg_mod_256:


.globl	sub_mod_256

.def	sub_mod_256;	.scl 2;	.type 32;	.endef
.p2align	5
sub_mod_256:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sub_mod_256:


	pushq	%rbp

	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	pushq	%rbx

	subq	$8,%rsp

.LSEH_body_sub_mod_256:


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11

	subq	0(%rdx),%r8
	movq	0(%rcx),%rax
	sbbq	8(%rdx),%r9
	movq	8(%rcx),%rsi
	sbbq	16(%rdx),%r10
	movq	16(%rcx),%rbx
	sbbq	24(%rdx),%r11
	movq	24(%rcx),%rbp
	sbbq	%rdx,%rdx

	andq	%rdx,%rax
	andq	%rdx,%rsi
	andq	%rdx,%rbx
	andq	%rdx,%rbp

	addq	%rax,%r8
	adcq	%rsi,%r9
	movq	%r8,0(%rdi)
	adcq	%rbx,%r10
	movq	%r9,8(%rdi)
	adcq	%rbp,%r11
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)

	movq	8(%rsp),%rbx

	movq	16(%rsp),%rbp

	leaq	24(%rsp),%rsp

.LSEH_epilogue_sub_mod_256:
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

.LSEH_end_sub_mod_256:


.globl	check_mod_256

.def	check_mod_256;	.scl 2;	.type 32;	.endef
.p2align	5
check_mod_256:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_check_mod_256:


	movq	%rcx,%rdi
	movq	%rdx,%rsi
#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rdi),%rax
	movq	8(%rdi),%r9
	movq	16(%rdi),%r10
	movq	24(%rdi),%r11

	movq	%rax,%r8
	orq	%r9,%rax
	orq	%r10,%rax
	orq	%r11,%rax

	subq	0(%rsi),%r8
	sbbq	8(%rsi),%r9
	sbbq	16(%rsi),%r10
	sbbq	24(%rsi),%r11
	sbbq	%rsi,%rsi

	movq	$1,%rdx
	cmpq	$0,%rax
	cmovneq	%rdx,%rax
	andq	%rsi,%rax
.LSEH_epilogue_check_mod_256:
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

.LSEH_end_check_mod_256:


.globl	add_n_check_mod_256

.def	add_n_check_mod_256;	.scl 2;	.type 32;	.endef
.p2align	5
add_n_check_mod_256:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_add_n_check_mod_256:


	pushq	%rbp

	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	pushq	%rbx

	subq	$8,%rsp

.LSEH_body_add_n_check_mod_256:


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11

	addq	0(%rdx),%r8
	adcq	8(%rdx),%r9
	movq	%r8,%rax
	adcq	16(%rdx),%r10
	movq	%r9,%rsi
	adcq	24(%rdx),%r11
	sbbq	%rdx,%rdx

	movq	%r10,%rbx
	subq	0(%rcx),%r8
	sbbq	8(%rcx),%r9
	sbbq	16(%rcx),%r10
	movq	%r11,%rbp
	sbbq	24(%rcx),%r11
	sbbq	$0,%rdx

	cmovcq	%rax,%r8
	cmovcq	%rsi,%r9
	movq	%r8,0(%rdi)
	cmovcq	%rbx,%r10
	movq	%r9,8(%rdi)
	cmovcq	%rbp,%r11
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)

	orq	%r9,%r8
	orq	%r11,%r10
	orq	%r10,%r8
	movq	$1,%rax
	cmovzq	%r8,%rax

	movq	8(%rsp),%rbx

	movq	16(%rsp),%rbp

	leaq	24(%rsp),%rsp

.LSEH_epilogue_add_n_check_mod_256:
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

.LSEH_end_add_n_check_mod_256:


.globl	sub_n_check_mod_256

.def	sub_n_check_mod_256;	.scl 2;	.type 32;	.endef
.p2align	5
sub_n_check_mod_256:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sub_n_check_mod_256:


	pushq	%rbp

	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx
	movq	%r9,%rcx
	pushq	%rbx

	subq	$8,%rsp

.LSEH_body_sub_n_check_mod_256:


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11

	subq	0(%rdx),%r8
	movq	0(%rcx),%rax
	sbbq	8(%rdx),%r9
	movq	8(%rcx),%rsi
	sbbq	16(%rdx),%r10
	movq	16(%rcx),%rbx
	sbbq	24(%rdx),%r11
	movq	24(%rcx),%rbp
	sbbq	%rdx,%rdx

	andq	%rdx,%rax
	andq	%rdx,%rsi
	andq	%rdx,%rbx
	andq	%rdx,%rbp

	addq	%rax,%r8
	adcq	%rsi,%r9
	movq	%r8,0(%rdi)
	adcq	%rbx,%r10
	movq	%r9,8(%rdi)
	adcq	%rbp,%r11
	movq	%r10,16(%rdi)
	movq	%r11,24(%rdi)

	orq	%r9,%r8
	orq	%r11,%r10
	orq	%r10,%r8
	movq	$1,%rax
	cmovzq	%r8,%rax

	movq	8(%rsp),%rbx

	movq	16(%rsp),%rbp

	leaq	24(%rsp),%rsp

.LSEH_epilogue_sub_n_check_mod_256:
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

.LSEH_end_sub_n_check_mod_256:
.section	.pdata
.p2align	2
.rva	.LSEH_begin_add_mod_256
.rva	.LSEH_body_add_mod_256
.rva	.LSEH_info_add_mod_256_prologue

.rva	.LSEH_body_add_mod_256
.rva	.LSEH_epilogue_add_mod_256
.rva	.LSEH_info_add_mod_256_body

.rva	.LSEH_epilogue_add_mod_256
.rva	.LSEH_end_add_mod_256
.rva	.LSEH_info_add_mod_256_epilogue

.rva	.LSEH_begin_mul_by_3_mod_256
.rva	.LSEH_body_mul_by_3_mod_256
.rva	.LSEH_info_mul_by_3_mod_256_prologue

.rva	.LSEH_body_mul_by_3_mod_256
.rva	.LSEH_epilogue_mul_by_3_mod_256
.rva	.LSEH_info_mul_by_3_mod_256_body

.rva	.LSEH_epilogue_mul_by_3_mod_256
.rva	.LSEH_end_mul_by_3_mod_256
.rva	.LSEH_info_mul_by_3_mod_256_epilogue

.rva	.LSEH_begin_lshift_mod_256
.rva	.LSEH_body_lshift_mod_256
.rva	.LSEH_info_lshift_mod_256_prologue

.rva	.LSEH_body_lshift_mod_256
.rva	.LSEH_epilogue_lshift_mod_256
.rva	.LSEH_info_lshift_mod_256_body

.rva	.LSEH_epilogue_lshift_mod_256
.rva	.LSEH_end_lshift_mod_256
.rva	.LSEH_info_lshift_mod_256_epilogue

.rva	.LSEH_begin_rshift_mod_256
.rva	.LSEH_body_rshift_mod_256
.rva	.LSEH_info_rshift_mod_256_prologue

.rva	.LSEH_body_rshift_mod_256
.rva	.LSEH_epilogue_rshift_mod_256
.rva	.LSEH_info_rshift_mod_256_body

.rva	.LSEH_epilogue_rshift_mod_256
.rva	.LSEH_end_rshift_mod_256
.rva	.LSEH_info_rshift_mod_256_epilogue

.rva	.LSEH_begin_cneg_mod_256
.rva	.LSEH_body_cneg_mod_256
.rva	.LSEH_info_cneg_mod_256_prologue

.rva	.LSEH_body_cneg_mod_256
.rva	.LSEH_epilogue_cneg_mod_256
.rva	.LSEH_info_cneg_mod_256_body

.rva	.LSEH_epilogue_cneg_mod_256
.rva	.LSEH_end_cneg_mod_256
.rva	.LSEH_info_cneg_mod_256_epilogue

.rva	.LSEH_begin_sub_mod_256
.rva	.LSEH_body_sub_mod_256
.rva	.LSEH_info_sub_mod_256_prologue

.rva	.LSEH_body_sub_mod_256
.rva	.LSEH_epilogue_sub_mod_256
.rva	.LSEH_info_sub_mod_256_body

.rva	.LSEH_epilogue_sub_mod_256
.rva	.LSEH_end_sub_mod_256
.rva	.LSEH_info_sub_mod_256_epilogue

.rva	.LSEH_epilogue_check_mod_256
.rva	.LSEH_end_check_mod_256
.rva	.LSEH_info_check_mod_256_epilogue

.rva	.LSEH_begin_add_n_check_mod_256
.rva	.LSEH_body_add_n_check_mod_256
.rva	.LSEH_info_add_n_check_mod_256_prologue

.rva	.LSEH_body_add_n_check_mod_256
.rva	.LSEH_epilogue_add_n_check_mod_256
.rva	.LSEH_info_add_n_check_mod_256_body

.rva	.LSEH_epilogue_add_n_check_mod_256
.rva	.LSEH_end_add_n_check_mod_256
.rva	.LSEH_info_add_n_check_mod_256_epilogue

.rva	.LSEH_begin_sub_n_check_mod_256
.rva	.LSEH_body_sub_n_check_mod_256
.rva	.LSEH_info_sub_n_check_mod_256_prologue

.rva	.LSEH_body_sub_n_check_mod_256
.rva	.LSEH_epilogue_sub_n_check_mod_256
.rva	.LSEH_info_sub_n_check_mod_256_body

.rva	.LSEH_epilogue_sub_n_check_mod_256
.rva	.LSEH_end_sub_n_check_mod_256
.rva	.LSEH_info_sub_n_check_mod_256_epilogue

.section	.xdata
.p2align	3
.LSEH_info_add_mod_256_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_add_mod_256_body:
.byte	1,0,9,0
.byte	0x00,0x34,0x01,0x00
.byte	0x00,0x54,0x02,0x00
.byte	0x00,0x74,0x04,0x00
.byte	0x00,0x64,0x05,0x00
.byte	0x00,0x22
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_add_mod_256_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_mul_by_3_mod_256_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_mul_by_3_mod_256_body:
.byte	1,0,11,0
.byte	0x00,0xc4,0x00,0x00
.byte	0x00,0x34,0x01,0x00
.byte	0x00,0x54,0x02,0x00
.byte	0x00,0x74,0x04,0x00
.byte	0x00,0x64,0x05,0x00
.byte	0x00,0x22
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.LSEH_info_mul_by_3_mod_256_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_lshift_mod_256_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_lshift_mod_256_body:
.byte	1,0,11,0
.byte	0x00,0xc4,0x00,0x00
.byte	0x00,0x34,0x01,0x00
.byte	0x00,0x54,0x02,0x00
.byte	0x00,0x74,0x04,0x00
.byte	0x00,0x64,0x05,0x00
.byte	0x00,0x22
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.LSEH_info_lshift_mod_256_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_rshift_mod_256_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_rshift_mod_256_body:
.byte	1,0,9,0
.byte	0x00,0x34,0x01,0x00
.byte	0x00,0x54,0x02,0x00
.byte	0x00,0x74,0x04,0x00
.byte	0x00,0x64,0x05,0x00
.byte	0x00,0x22
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_rshift_mod_256_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_cneg_mod_256_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_cneg_mod_256_body:
.byte	1,0,11,0
.byte	0x00,0xc4,0x00,0x00
.byte	0x00,0x34,0x01,0x00
.byte	0x00,0x54,0x02,0x00
.byte	0x00,0x74,0x04,0x00
.byte	0x00,0x64,0x05,0x00
.byte	0x00,0x22
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.LSEH_info_cneg_mod_256_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sub_mod_256_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sub_mod_256_body:
.byte	1,0,9,0
.byte	0x00,0x34,0x01,0x00
.byte	0x00,0x54,0x02,0x00
.byte	0x00,0x74,0x04,0x00
.byte	0x00,0x64,0x05,0x00
.byte	0x00,0x22
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_sub_mod_256_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_check_mod_256_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_add_n_check_mod_256_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_add_n_check_mod_256_body:
.byte	1,0,9,0
.byte	0x00,0x34,0x01,0x00
.byte	0x00,0x54,0x02,0x00
.byte	0x00,0x74,0x04,0x00
.byte	0x00,0x64,0x05,0x00
.byte	0x00,0x22
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_add_n_check_mod_256_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sub_n_check_mod_256_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sub_n_check_mod_256_body:
.byte	1,0,9,0
.byte	0x00,0x34,0x01,0x00
.byte	0x00,0x54,0x02,0x00
.byte	0x00,0x74,0x04,0x00
.byte	0x00,0x64,0x05,0x00
.byte	0x00,0x22
.byte	0x00,0x00,0x00,0x00,0x00,0x00
.byte	0x00,0x00,0x00,0x00
.LSEH_info_sub_n_check_mod_256_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

