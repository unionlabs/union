.text	

.globl	add_mod_384x384

.def	add_mod_384x384;	.scl 2;	.type 32;	.endef
.p2align	5
add_mod_384x384:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_add_mod_384x384:


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

	subq	$8,%rsp

.LSEH_body_add_mod_384x384:


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13
	movq	48(%rsi),%r14

	addq	0(%rdx),%r8
	movq	56(%rsi),%r15
	adcq	8(%rdx),%r9
	movq	64(%rsi),%rax
	adcq	16(%rdx),%r10
	movq	72(%rsi),%rbx
	adcq	24(%rdx),%r11
	movq	80(%rsi),%rbp
	adcq	32(%rdx),%r12
	movq	88(%rsi),%rsi
	adcq	40(%rdx),%r13
	movq	%r8,0(%rdi)
	adcq	48(%rdx),%r14
	movq	%r9,8(%rdi)
	adcq	56(%rdx),%r15
	movq	%r10,16(%rdi)
	adcq	64(%rdx),%rax
	movq	%r12,32(%rdi)
	movq	%r14,%r8
	adcq	72(%rdx),%rbx
	movq	%r11,24(%rdi)
	movq	%r15,%r9
	adcq	80(%rdx),%rbp
	movq	%r13,40(%rdi)
	movq	%rax,%r10
	adcq	88(%rdx),%rsi
	movq	%rbx,%r11
	sbbq	%rdx,%rdx

	subq	0(%rcx),%r14
	sbbq	8(%rcx),%r15
	movq	%rbp,%r12
	sbbq	16(%rcx),%rax
	sbbq	24(%rcx),%rbx
	sbbq	32(%rcx),%rbp
	movq	%rsi,%r13
	sbbq	40(%rcx),%rsi
	sbbq	$0,%rdx

	cmovcq	%r8,%r14
	cmovcq	%r9,%r15
	cmovcq	%r10,%rax
	movq	%r14,48(%rdi)
	cmovcq	%r11,%rbx
	movq	%r15,56(%rdi)
	cmovcq	%r12,%rbp
	movq	%rax,64(%rdi)
	cmovcq	%r13,%rsi
	movq	%rbx,72(%rdi)
	movq	%rbp,80(%rdi)
	movq	%rsi,88(%rdi)

	movq	8(%rsp),%r15

	movq	16(%rsp),%r14

	movq	24(%rsp),%r13

	movq	32(%rsp),%r12

	movq	40(%rsp),%rbx

	movq	48(%rsp),%rbp

	leaq	56(%rsp),%rsp

.LSEH_epilogue_add_mod_384x384:
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

.LSEH_end_add_mod_384x384:

.globl	sub_mod_384x384

.def	sub_mod_384x384;	.scl 2;	.type 32;	.endef
.p2align	5
sub_mod_384x384:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_sub_mod_384x384:


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

	subq	$8,%rsp

.LSEH_body_sub_mod_384x384:


#ifdef	__SGX_LVI_HARDENING__
	lfence
#endif
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

	movq	8(%rsp),%r15

	movq	16(%rsp),%r14

	movq	24(%rsp),%r13

	movq	32(%rsp),%r12

	movq	40(%rsp),%rbx

	movq	48(%rsp),%rbp

	leaq	56(%rsp),%rsp

.LSEH_epilogue_sub_mod_384x384:
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

.LSEH_end_sub_mod_384x384:
.section	.pdata
.p2align	2
.rva	.LSEH_begin_add_mod_384x384
.rva	.LSEH_body_add_mod_384x384
.rva	.LSEH_info_add_mod_384x384_prologue

.rva	.LSEH_body_add_mod_384x384
.rva	.LSEH_epilogue_add_mod_384x384
.rva	.LSEH_info_add_mod_384x384_body

.rva	.LSEH_epilogue_add_mod_384x384
.rva	.LSEH_end_add_mod_384x384
.rva	.LSEH_info_add_mod_384x384_epilogue

.rva	.LSEH_begin_sub_mod_384x384
.rva	.LSEH_body_sub_mod_384x384
.rva	.LSEH_info_sub_mod_384x384_prologue

.rva	.LSEH_body_sub_mod_384x384
.rva	.LSEH_epilogue_sub_mod_384x384
.rva	.LSEH_info_sub_mod_384x384_body

.rva	.LSEH_epilogue_sub_mod_384x384
.rva	.LSEH_end_sub_mod_384x384
.rva	.LSEH_info_sub_mod_384x384_epilogue

.section	.xdata
.p2align	3
.LSEH_info_add_mod_384x384_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_add_mod_384x384_body:
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
.LSEH_info_add_mod_384x384_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

.LSEH_info_sub_mod_384x384_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0xb3
.byte	0,0
.long	0,0
.LSEH_info_sub_mod_384x384_body:
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
.LSEH_info_sub_mod_384x384_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

