.text	

.globl	div_3_limbs

.def	div_3_limbs;	.scl 2;	.type 32;	.endef
.p2align	5
div_3_limbs:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%rax
.LSEH_begin_div_3_limbs:
	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx

	movq	(%rdi),%r8
	movq	8(%rdi),%r9
	xorq	%rax,%rax
	movl	$64,%ecx

.Loop:
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
	jnz	.Loop

	leaq	1(%rax,%rax,1),%rcx
	sarq	$63,%rax

	subq	%rsi,%r8
	sbbq	%rdx,%r9
	sbbq	$0,%rcx

	orq	%rcx,%rax

	movq	8(%rsp),%rdi
	movq	16(%rsp),%rsi
	.byte	0xf3,0xc3
.LSEH_end_div_3_limbs:
.globl	quot_rem_128

.def	quot_rem_128;	.scl 2;	.type 32;	.endef
.p2align	5
quot_rem_128:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%rax
.LSEH_begin_quot_rem_128:
	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx

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

	movq	8(%rsp),%rdi
	movq	16(%rsp),%rsi
	.byte	0xf3,0xc3
.LSEH_end_quot_rem_128:





.globl	quot_rem_64

.def	quot_rem_64;	.scl 2;	.type 32;	.endef
.p2align	5
quot_rem_64:
	.byte	0xf3,0x0f,0x1e,0xfa
	movq	%rdi,8(%rsp)
	movq	%rsi,16(%rsp)
	movq	%rsp,%rax
.LSEH_begin_quot_rem_64:
	movq	%rcx,%rdi
	movq	%rdx,%rsi
	movq	%r8,%rdx

	movq	%rdx,%rax
	imulq	0(%rsi),%rdx

	movq	0(%rdi),%r10

	subq	%rdx,%r10

	movq	%r10,0(%rdi)
	movq	%rax,8(%rdi)

	movq	8(%rsp),%rdi
	movq	16(%rsp),%rsi
	.byte	0xf3,0xc3
.LSEH_end_quot_rem_64:
.section	.pdata
.p2align	2
.section	.xdata
.p2align	3
