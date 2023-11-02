.text	

.globl	ct_is_square_mod_384
.def	ct_is_square_mod_384;	.scl 2;	.type 32;	.endef
.p2align	5
ct_is_square_mod_384:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%r11
.LSEH_begin_ct_is_square_mod_384:
	movq	%rcx,%rdi
	movq	%rdx,%rsi


	pushq	%rbp

	pushq	%rbx

	pushq	%r12

	pushq	%r13

	pushq	%r14

	pushq	%r15

	subq	$536,%rsp

.LSEH_body_ct_is_square_mod_384:


	leaq	24+255(%rsp),%rax
	andq	$-256,%rax

	movq	0(%rdi),%r8
	movq	8(%rdi),%r9
	movq	16(%rdi),%r10
	movq	24(%rdi),%r11
	movq	32(%rdi),%r12
	movq	40(%rdi),%r13

	movq	0(%rsi),%r14
	movq	8(%rsi),%r15
	movq	16(%rsi),%rbx
	movq	24(%rsi),%rcx
	movq	32(%rsi),%rdx
	movq	40(%rsi),%rdi
	movq	%rax,%rsi

	movq	%r8,0(%rax)
	movq	%r9,8(%rax)
	movq	%r10,16(%rax)
	movq	%r11,24(%rax)
	movq	%r12,32(%rax)
	movq	%r13,40(%rax)

	movq	%r14,48(%rax)
	movq	%r15,56(%rax)
	movq	%rbx,64(%rax)
	movq	%rcx,72(%rax)
	movq	%rdx,80(%rax)
	movq	%rdi,88(%rax)

	xorq	%rbp,%rbp
	movl	$24,%ecx
	jmp	.Loop_is_square

.p2align	5
.Loop_is_square:
	movl	%ecx,16(%rsp)

	call	__ab_approximation_30
	movq	%rax,0(%rsp)
	movq	%rbx,8(%rsp)

	movq	$128+48,%rdi
	xorq	%rsi,%rdi
	call	__smulq_384_n_shift_by_30

	movq	0(%rsp),%rdx
	movq	8(%rsp),%rcx
	leaq	-48(%rdi),%rdi
	call	__smulq_384_n_shift_by_30

	movl	16(%rsp),%ecx
	xorq	$128,%rsi

	andq	48(%rdi),%r14
	shrq	$1,%r14
	addq	%r14,%rbp

	subl	$1,%ecx
	jnz	.Loop_is_square




	movq	48(%rsi),%r9
	call	__inner_loop_48

	movq	$1,%rax
	andq	%rbp,%rax
	xorq	$1,%rax

	leaq	536(%rsp),%r8
	movq	0(%r8),%r15

	movq	8(%r8),%r14

	movq	16(%r8),%r13

	movq	24(%r8),%r12

	movq	32(%r8),%rbx

	movq	40(%r8),%rbp

	leaq	48(%r8),%rsp

.LSEH_epilogue_ct_is_square_mod_384:
	mov	8(%rsp),%rdi
	mov	16(%rsp),%rsi

	.byte	0xf3,0xc3

.LSEH_end_ct_is_square_mod_384:

.def	__smulq_384_n_shift_by_30;	.scl 3;	.type 32;	.endef
.p2align	5
__smulq_384_n_shift_by_30:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

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
	xorq	%rdx,%r12
	xorq	%rdx,%r13
	addq	%r8,%rax
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%r13

	movq	%rdx,%r14
	andq	%rbx,%r14
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
	mulq	%rbx
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	mulq	%rbx
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13
	negq	%r14
	mulq	%rbx
	addq	%rax,%r13
	adcq	%rdx,%r14
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
	xorq	%rdx,%r12
	xorq	%rdx,%r13
	addq	%r8,%rax
	adcq	$0,%r9
	adcq	$0,%r10
	adcq	$0,%r11
	adcq	$0,%r12
	adcq	$0,%r13

	movq	%rdx,%r15
	andq	%rbx,%r15
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
	mulq	%rbx
	addq	%rax,%r11
	movq	%r12,%rax
	adcq	$0,%rdx
	movq	%rdx,%r12
	mulq	%rbx
	addq	%rax,%r12
	movq	%r13,%rax
	adcq	$0,%rdx
	movq	%rdx,%r13
	negq	%r15
	mulq	%rbx
	addq	%rax,%r13
	adcq	%rdx,%r15
	leaq	-48(%rsi),%rsi

	addq	0(%rdi),%r8
	adcq	8(%rdi),%r9
	adcq	16(%rdi),%r10
	adcq	24(%rdi),%r11
	adcq	32(%rdi),%r12
	adcq	40(%rdi),%r13
	adcq	%r15,%r14

	shrdq	$30,%r9,%r8
	shrdq	$30,%r10,%r9
	shrdq	$30,%r11,%r10
	shrdq	$30,%r12,%r11
	shrdq	$30,%r13,%r12
	shrdq	$30,%r14,%r13

	sarq	$63,%r14
	xorq	%rbx,%rbx
	subq	%r14,%rbx

	xorq	%r14,%r8
	xorq	%r14,%r9
	xorq	%r14,%r10
	xorq	%r14,%r11
	xorq	%r14,%r12
	xorq	%r14,%r13
	addq	%rbx,%r8
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

	.byte	0xf3,0xc3

.def	__ab_approximation_30;	.scl 3;	.type 32;	.endef
.p2align	5
__ab_approximation_30:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	88(%rsi),%rbx
	movq	80(%rsi),%r15
	movq	72(%rsi),%r14

	movq	%r13,%rax
	orq	%rbx,%rax
	cmovzq	%r12,%r13
	cmovzq	%r15,%rbx
	cmovzq	%r11,%r12
	movq	64(%rsi),%r11
	cmovzq	%r14,%r15

	movq	%r13,%rax
	orq	%rbx,%rax
	cmovzq	%r12,%r13
	cmovzq	%r15,%rbx
	cmovzq	%r10,%r12
	movq	56(%rsi),%r10
	cmovzq	%r11,%r15

	movq	%r13,%rax
	orq	%rbx,%rax
	cmovzq	%r12,%r13
	cmovzq	%r15,%rbx
	cmovzq	%r9,%r12
	movq	48(%rsi),%r9
	cmovzq	%r10,%r15

	movq	%r13,%rax
	orq	%rbx,%rax
	cmovzq	%r12,%r13
	cmovzq	%r15,%rbx
	cmovzq	%r8,%r12
	cmovzq	%r9,%r15

	movq	%r13,%rax
	orq	%rbx,%rax
	bsrq	%rax,%rcx
	leaq	1(%rcx),%rcx
	cmovzq	%r8,%r13
	cmovzq	%r9,%rbx
	cmovzq	%rax,%rcx
	negq	%rcx


	shldq	%cl,%r12,%r13
	shldq	%cl,%r15,%rbx

	movq	$0xFFFFFFFF00000000,%rax
	movl	%r8d,%r8d
	movl	%r9d,%r9d
	andq	%rax,%r13
	andq	%rax,%rbx
	orq	%r13,%r8
	orq	%rbx,%r9

	jmp	__inner_loop_30

	.byte	0xf3,0xc3

.def	__inner_loop_30;	.scl 3;	.type 32;	.endef
.p2align	5
__inner_loop_30:
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	$0x7FFFFFFF80000000,%rbx
	movq	$0x800000007FFFFFFF,%rcx
	leaq	-1(%rbx),%r15
	movl	$30,%edi

.Loop_30:
	movq	%r8,%rax
	andq	%r9,%rax
	shrq	$1,%rax

	cmpq	%r9,%r8
	movq	%r8,%r10
	movq	%r9,%r11
	leaq	(%rax,%rbp,1),%rax
	movq	%rbx,%r12
	movq	%rcx,%r13
	movq	%rbp,%r14
	cmovbq	%r9,%r8
	cmovbq	%r10,%r9
	cmovbq	%rcx,%rbx
	cmovbq	%r12,%rcx
	cmovbq	%rax,%rbp

	subq	%r9,%r8
	subq	%rcx,%rbx
	addq	%r15,%rbx

	testq	$1,%r10
	cmovzq	%r10,%r8
	cmovzq	%r11,%r9
	cmovzq	%r12,%rbx
	cmovzq	%r13,%rcx
	cmovzq	%r14,%rbp

	leaq	2(%r9),%rax
	shrq	$1,%r8
	shrq	$2,%rax
	addq	%rcx,%rcx
	leaq	(%rax,%rbp,1),%rbp
	subq	%r15,%rcx

	subl	$1,%edi
	jnz	.Loop_30

	shrq	$32,%r15
	movl	%ebx,%eax
	shrq	$32,%rbx
	movl	%ecx,%edx
	shrq	$32,%rcx
	subq	%r15,%rax
	subq	%r15,%rbx
	subq	%r15,%rdx
	subq	%r15,%rcx

	.byte	0xf3,0xc3


.def	__inner_loop_48;	.scl 3;	.type 32;	.endef
.p2align	5
__inner_loop_48:
	.byte	0xf3,0x0f,0x1e,0xfa

	movl	$48,%edi

.Loop_48:
	movq	%r8,%rax
	andq	%r9,%rax
	shrq	$1,%rax

	cmpq	%r9,%r8
	movq	%r8,%r10
	movq	%r9,%r11
	leaq	(%rax,%rbp,1),%rax
	movq	%rbp,%r12
	cmovbq	%r9,%r8
	cmovbq	%r10,%r9
	cmovbq	%rax,%rbp

	subq	%r9,%r8

	testq	$1,%r10
	cmovzq	%r10,%r8
	cmovzq	%r11,%r9
	cmovzq	%r12,%rbp

	leaq	2(%r9),%rax
	shrq	$1,%r8
	shrq	$2,%rax
	addq	%rax,%rbp

	subl	$1,%edi
	jnz	.Loop_48

	.byte	0xf3,0xc3

.section	.pdata
.p2align	2
.rva	.LSEH_begin_ct_is_square_mod_384
.rva	.LSEH_body_ct_is_square_mod_384
.rva	.LSEH_info_ct_is_square_mod_384_prologue

.rva	.LSEH_body_ct_is_square_mod_384
.rva	.LSEH_epilogue_ct_is_square_mod_384
.rva	.LSEH_info_ct_is_square_mod_384_body

.rva	.LSEH_epilogue_ct_is_square_mod_384
.rva	.LSEH_end_ct_is_square_mod_384
.rva	.LSEH_info_ct_is_square_mod_384_epilogue

.section	.xdata
.p2align	3
.LSEH_info_ct_is_square_mod_384_prologue:
.byte	1,0,5,0x0b
.byte	0,0x74,1,0
.byte	0,0x64,2,0
.byte	0,0x03
.byte	0,0
.LSEH_info_ct_is_square_mod_384_body:
.byte	1,0,18,0
.byte	0x00,0xf4,0x43,0x00
.byte	0x00,0xe4,0x44,0x00
.byte	0x00,0xd4,0x45,0x00
.byte	0x00,0xc4,0x46,0x00
.byte	0x00,0x34,0x47,0x00
.byte	0x00,0x54,0x48,0x00
.byte	0x00,0x74,0x4a,0x00
.byte	0x00,0x64,0x4b,0x00
.byte	0x00,0x01,0x49,0x00
.LSEH_info_ct_is_square_mod_384_epilogue:
.byte	1,0,4,0
.byte	0x00,0x74,0x01,0x00
.byte	0x00,0x64,0x02,0x00
.byte	0x00,0x00,0x00,0x00

