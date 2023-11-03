.text	







.type	__sub_mod_384x384,@function
.align	32
__sub_mod_384x384:
.cfi_startproc
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

	.byte	0xf3,0xc3
.cfi_endproc
.size	__sub_mod_384x384,.-__sub_mod_384x384

.type	__add_mod_384,@function
.align	32
__add_mod_384:
.cfi_startproc
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

	.byte	0xf3,0xc3
.cfi_endproc
.size	__add_mod_384,.-__add_mod_384

.type	__sub_mod_384,@function
.align	32
__sub_mod_384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%r8
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13

__sub_mod_384_a_is_loaded:
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

	.byte	0xf3,0xc3
.cfi_endproc
.size	__sub_mod_384,.-__sub_mod_384
.globl	mulx_mont_384x
.hidden	mulx_mont_384x
.type	mulx_mont_384x,@function
.align	32
mulx_mont_384x:
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
	subq	$328,%rsp
.cfi_adjust_cfa_offset	328


	movq	%rdx,%rbx
	movq	%rdi,32(%rsp)
	movq	%rsi,24(%rsp)
	movq	%rdx,16(%rsp)
	movq	%rcx,8(%rsp)
	movq	%r8,0(%rsp)




	leaq	40(%rsp),%rdi
	call	__mulx_384


	leaq	48(%rbx),%rbx
	leaq	128+48(%rsi),%rsi
	leaq	96(%rdi),%rdi
	call	__mulx_384


	movq	8(%rsp),%rcx
	leaq	(%rbx),%rsi
	leaq	-48(%rbx),%rdx
	leaq	40+192+48(%rsp),%rdi
	call	__add_mod_384

	movq	24(%rsp),%rsi
	leaq	48(%rsi),%rdx
	leaq	-48(%rdi),%rdi
	call	__add_mod_384

	leaq	(%rdi),%rbx
	leaq	48(%rdi),%rsi
	call	__mulx_384


	leaq	(%rdi),%rsi
	leaq	40(%rsp),%rdx
	movq	8(%rsp),%rcx
	call	__sub_mod_384x384

	leaq	(%rdi),%rsi
	leaq	-96(%rdi),%rdx
	call	__sub_mod_384x384


	leaq	40(%rsp),%rsi
	leaq	40+96(%rsp),%rdx
	leaq	40(%rsp),%rdi
	call	__sub_mod_384x384

	leaq	(%rcx),%rbx


	leaq	40(%rsp),%rsi
	movq	0(%rsp),%rcx
	movq	32(%rsp),%rdi
	call	__mulx_by_1_mont_384
	call	__redc_tail_mont_384


	leaq	40+192(%rsp),%rsi
	movq	0(%rsp),%rcx
	leaq	48(%rdi),%rdi
	call	__mulx_by_1_mont_384
	call	__redc_tail_mont_384

	leaq	328(%rsp),%r8
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
.cfi_adjust_cfa_offset	-328-8*6

	.byte	0xf3,0xc3
.cfi_endproc	
.size	mulx_mont_384x,.-mulx_mont_384x
.globl	sqrx_mont_384x
.hidden	sqrx_mont_384x
.type	sqrx_mont_384x,@function
.align	32
sqrx_mont_384x:
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
	subq	$136,%rsp
.cfi_adjust_cfa_offset	136


	movq	%rcx,0(%rsp)
	movq	%rdx,%rcx

	movq	%rdi,16(%rsp)
	movq	%rsi,24(%rsp)


	leaq	48(%rsi),%rdx
	leaq	32(%rsp),%rdi
	call	__add_mod_384


	movq	24(%rsp),%rsi
	leaq	48(%rsi),%rdx
	leaq	32+48(%rsp),%rdi
	call	__sub_mod_384


	movq	24(%rsp),%rsi
	leaq	48(%rsi),%rbx

	movq	48(%rsi),%rdx
	movq	0(%rsi),%r14
	movq	8(%rsi),%r15
	movq	16(%rsi),%rax
	movq	24(%rsi),%r12
	movq	32(%rsi),%rdi
	movq	40(%rsi),%rbp
	leaq	-128(%rsi),%rsi
	leaq	-128(%rcx),%rcx

	mulxq	%r14,%r8,%r9
	call	__mulx_mont_384
	addq	%rdx,%rdx
	adcq	%r15,%r15
	adcq	%rax,%rax
	movq	%rdx,%r8
	adcq	%r12,%r12
	movq	%r15,%r9
	adcq	%rdi,%rdi
	movq	%rax,%r10
	adcq	%rbp,%rbp
	movq	%r12,%r11
	sbbq	%rsi,%rsi

	subq	0(%rcx),%rdx
	sbbq	8(%rcx),%r15
	movq	%rdi,%r13
	sbbq	16(%rcx),%rax
	sbbq	24(%rcx),%r12
	sbbq	32(%rcx),%rdi
	movq	%rbp,%r14
	sbbq	40(%rcx),%rbp
	sbbq	$0,%rsi

	cmovcq	%r8,%rdx
	cmovcq	%r9,%r15
	cmovcq	%r10,%rax
	movq	%rdx,48(%rbx)
	cmovcq	%r11,%r12
	movq	%r15,56(%rbx)
	cmovcq	%r13,%rdi
	movq	%rax,64(%rbx)
	cmovcq	%r14,%rbp
	movq	%r12,72(%rbx)
	movq	%rdi,80(%rbx)
	movq	%rbp,88(%rbx)

	leaq	32(%rsp),%rsi
	leaq	32+48(%rsp),%rbx

	movq	32+48(%rsp),%rdx
	movq	32+0(%rsp),%r14
	movq	32+8(%rsp),%r15
	movq	32+16(%rsp),%rax
	movq	32+24(%rsp),%r12
	movq	32+32(%rsp),%rdi
	movq	32+40(%rsp),%rbp
	leaq	-128(%rsi),%rsi
	leaq	-128(%rcx),%rcx

	mulxq	%r14,%r8,%r9
	call	__mulx_mont_384

	leaq	136(%rsp),%r8
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
.cfi_adjust_cfa_offset	-136-8*6

	.byte	0xf3,0xc3
.cfi_endproc	
.size	sqrx_mont_384x,.-sqrx_mont_384x

.globl	mulx_382x
.hidden	mulx_382x
.type	mulx_382x,@function
.align	32
mulx_382x:
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
	subq	$136,%rsp
.cfi_adjust_cfa_offset	136


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
	call	__mulx_384


	movq	0(%rsp),%rsi
	movq	8(%rsp),%rbx
	leaq	-96(%rdi),%rdi
	call	__mulx_384


	leaq	48+128(%rsi),%rsi
	leaq	48(%rbx),%rbx
	leaq	32(%rsp),%rdi
	call	__mulx_384


	movq	16(%rsp),%rsi
	leaq	32(%rsp),%rdx
	movq	24(%rsp),%rcx
	movq	%rsi,%rdi
	call	__sub_mod_384x384


	leaq	0(%rdi),%rsi
	leaq	-96(%rdi),%rdx
	call	__sub_mod_384x384


	leaq	-96(%rdi),%rsi
	leaq	32(%rsp),%rdx
	leaq	-96(%rdi),%rdi
	call	__sub_mod_384x384

	leaq	136(%rsp),%r8
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
.cfi_adjust_cfa_offset	-136-8*6

	.byte	0xf3,0xc3
.cfi_endproc	
.size	mulx_382x,.-mulx_382x
.globl	sqrx_382x
.hidden	sqrx_382x
.type	sqrx_382x,@function
.align	32
sqrx_382x:
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
	call	__sub_mod_384_a_is_loaded


	leaq	(%rdi),%rsi
	leaq	-48(%rdi),%rbx
	leaq	-48(%rdi),%rdi
	call	__mulx_384


	movq	(%rsp),%rsi
	leaq	48(%rsi),%rbx
	leaq	96(%rdi),%rdi
	call	__mulx_384

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
.cfi_adjust_cfa_offset	-8*7

	.byte	0xf3,0xc3
.cfi_endproc	
.size	sqrx_382x,.-sqrx_382x
.globl	mulx_384
.hidden	mulx_384
.type	mulx_384,@function
.align	32
mulx_384:
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


	movq	%rdx,%rbx
	call	__mulx_384

	movq	0(%rsp),%r15
.cfi_restore	%r15
	movq	8(%rsp),%r14
.cfi_restore	%r14
	movq	16(%rsp),%r13
.cfi_restore	%r13
	movq	24(%rsp),%r12
.cfi_restore	%r12
	movq	32(%rsp),%rbx
.cfi_restore	%rbx
	movq	40(%rsp),%rbp
.cfi_restore	%rbp
	leaq	48(%rsp),%rsp
.cfi_adjust_cfa_offset	-48

	.byte	0xf3,0xc3
.cfi_endproc	
.size	mulx_384,.-mulx_384

.type	__mulx_384,@function
.align	32
__mulx_384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rbx),%rdx
	movq	0(%rsi),%r14
	movq	8(%rsi),%r15
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13
	leaq	-128(%rsi),%rsi

	mulxq	%r14,%r9,%rcx
	xorq	%rbp,%rbp

	mulxq	%r15,%r8,%rax
	adcxq	%rcx,%r8
	movq	%r9,0(%rdi)

	mulxq	%r10,%r9,%rcx
	adcxq	%rax,%r9

	mulxq	%r11,%r10,%rax
	adcxq	%rcx,%r10

	mulxq	%r12,%r11,%rcx
	adcxq	%rax,%r11

	mulxq	%r13,%r12,%r13
	movq	8(%rbx),%rdx
	adcxq	%rcx,%r12
	adcxq	%rbp,%r13
	mulxq	%r14,%rax,%rcx
	adcxq	%r8,%rax
	adoxq	%rcx,%r9
	movq	%rax,8(%rdi)

	mulxq	%r15,%r8,%rcx
	adcxq	%r9,%r8
	adoxq	%rcx,%r10

	mulxq	128+16(%rsi),%r9,%rax
	adcxq	%r10,%r9
	adoxq	%rax,%r11

	mulxq	128+24(%rsi),%r10,%rcx
	adcxq	%r11,%r10
	adoxq	%rcx,%r12

	mulxq	128+32(%rsi),%r11,%rax
	adcxq	%r12,%r11
	adoxq	%r13,%rax

	mulxq	128+40(%rsi),%r12,%r13
	movq	16(%rbx),%rdx
	adcxq	%rax,%r12
	adoxq	%rbp,%r13
	adcxq	%rbp,%r13
	mulxq	%r14,%rax,%rcx
	adcxq	%r8,%rax
	adoxq	%rcx,%r9
	movq	%rax,16(%rdi)

	mulxq	%r15,%r8,%rcx
	adcxq	%r9,%r8
	adoxq	%rcx,%r10

	mulxq	128+16(%rsi),%r9,%rax
	adcxq	%r10,%r9
	adoxq	%rax,%r11

	mulxq	128+24(%rsi),%r10,%rcx
	adcxq	%r11,%r10
	adoxq	%rcx,%r12

	mulxq	128+32(%rsi),%r11,%rax
	adcxq	%r12,%r11
	adoxq	%r13,%rax

	mulxq	128+40(%rsi),%r12,%r13
	movq	24(%rbx),%rdx
	adcxq	%rax,%r12
	adoxq	%rbp,%r13
	adcxq	%rbp,%r13
	mulxq	%r14,%rax,%rcx
	adcxq	%r8,%rax
	adoxq	%rcx,%r9
	movq	%rax,24(%rdi)

	mulxq	%r15,%r8,%rcx
	adcxq	%r9,%r8
	adoxq	%rcx,%r10

	mulxq	128+16(%rsi),%r9,%rax
	adcxq	%r10,%r9
	adoxq	%rax,%r11

	mulxq	128+24(%rsi),%r10,%rcx
	adcxq	%r11,%r10
	adoxq	%rcx,%r12

	mulxq	128+32(%rsi),%r11,%rax
	adcxq	%r12,%r11
	adoxq	%r13,%rax

	mulxq	128+40(%rsi),%r12,%r13
	movq	32(%rbx),%rdx
	adcxq	%rax,%r12
	adoxq	%rbp,%r13
	adcxq	%rbp,%r13
	mulxq	%r14,%rax,%rcx
	adcxq	%r8,%rax
	adoxq	%rcx,%r9
	movq	%rax,32(%rdi)

	mulxq	%r15,%r8,%rcx
	adcxq	%r9,%r8
	adoxq	%rcx,%r10

	mulxq	128+16(%rsi),%r9,%rax
	adcxq	%r10,%r9
	adoxq	%rax,%r11

	mulxq	128+24(%rsi),%r10,%rcx
	adcxq	%r11,%r10
	adoxq	%rcx,%r12

	mulxq	128+32(%rsi),%r11,%rax
	adcxq	%r12,%r11
	adoxq	%r13,%rax

	mulxq	128+40(%rsi),%r12,%r13
	movq	40(%rbx),%rdx
	adcxq	%rax,%r12
	adoxq	%rbp,%r13
	adcxq	%rbp,%r13
	mulxq	%r14,%rax,%rcx
	adcxq	%r8,%rax
	adoxq	%rcx,%r9
	movq	%rax,40(%rdi)

	mulxq	%r15,%r8,%rcx
	adcxq	%r9,%r8
	adoxq	%rcx,%r10

	mulxq	128+16(%rsi),%r9,%rax
	adcxq	%r10,%r9
	adoxq	%rax,%r11

	mulxq	128+24(%rsi),%r10,%rcx
	adcxq	%r11,%r10
	adoxq	%rcx,%r12

	mulxq	128+32(%rsi),%r11,%rax
	adcxq	%r12,%r11
	adoxq	%r13,%rax

	mulxq	128+40(%rsi),%r12,%r13
	movq	%rax,%rdx
	adcxq	%rax,%r12
	adoxq	%rbp,%r13
	adcxq	%rbp,%r13
	movq	%r8,48(%rdi)
	movq	%r9,56(%rdi)
	movq	%r10,64(%rdi)
	movq	%r11,72(%rdi)
	movq	%r12,80(%rdi)
	movq	%r13,88(%rdi)

	.byte	0xf3,0xc3
.cfi_endproc
.size	__mulx_384,.-__mulx_384
.globl	sqrx_384
.hidden	sqrx_384
.type	sqrx_384,@function
.align	32
sqrx_384:
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


	call	__sqrx_384

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
.size	sqrx_384,.-sqrx_384
.type	__sqrx_384,@function
.align	32
__sqrx_384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%rdx
	movq	8(%rsi),%r14
	movq	16(%rsi),%r15
	movq	24(%rsi),%rcx
	movq	32(%rsi),%rbx


	mulxq	%r14,%r8,%rdi
	movq	40(%rsi),%rbp
	mulxq	%r15,%r9,%rax
	addq	%rdi,%r9
	mulxq	%rcx,%r10,%rdi
	adcq	%rax,%r10
	mulxq	%rbx,%r11,%rax
	adcq	%rdi,%r11
	mulxq	%rbp,%r12,%r13
	movq	%r14,%rdx
	adcq	%rax,%r12
	adcq	$0,%r13


	xorq	%r14,%r14
	mulxq	%r15,%rdi,%rax
	adcxq	%rdi,%r10
	adoxq	%rax,%r11

	mulxq	%rcx,%rdi,%rax
	adcxq	%rdi,%r11
	adoxq	%rax,%r12

	mulxq	%rbx,%rdi,%rax
	adcxq	%rdi,%r12
	adoxq	%rax,%r13

	mulxq	%rbp,%rdi,%rax
	movq	%r15,%rdx
	adcxq	%rdi,%r13
	adoxq	%r14,%rax
	adcxq	%rax,%r14


	xorq	%r15,%r15
	mulxq	%rcx,%rdi,%rax
	adcxq	%rdi,%r12
	adoxq	%rax,%r13

	mulxq	%rbx,%rdi,%rax
	adcxq	%rdi,%r13
	adoxq	%rax,%r14

	mulxq	%rbp,%rdi,%rax
	movq	%rcx,%rdx
	adcxq	%rdi,%r14
	adoxq	%r15,%rax
	adcxq	%rax,%r15


	xorq	%rcx,%rcx
	mulxq	%rbx,%rdi,%rax
	adcxq	%rdi,%r14
	adoxq	%rax,%r15

	mulxq	%rbp,%rdi,%rax
	movq	%rbx,%rdx
	adcxq	%rdi,%r15
	adoxq	%rcx,%rax
	adcxq	%rax,%rcx


	mulxq	%rbp,%rdi,%rbx
	movq	0(%rsi),%rdx
	addq	%rdi,%rcx
	movq	8(%rsp),%rdi
	adcq	$0,%rbx


	xorq	%rbp,%rbp
	adcxq	%r8,%r8
	adcxq	%r9,%r9
	adcxq	%r10,%r10
	adcxq	%r11,%r11
	adcxq	%r12,%r12


	mulxq	%rdx,%rdx,%rax
	movq	%rdx,0(%rdi)
	movq	8(%rsi),%rdx
	adoxq	%rax,%r8
	movq	%r8,8(%rdi)

	mulxq	%rdx,%r8,%rax
	movq	16(%rsi),%rdx
	adoxq	%r8,%r9
	adoxq	%rax,%r10
	movq	%r9,16(%rdi)
	movq	%r10,24(%rdi)

	mulxq	%rdx,%r8,%r9
	movq	24(%rsi),%rdx
	adoxq	%r8,%r11
	adoxq	%r9,%r12
	adcxq	%r13,%r13
	adcxq	%r14,%r14
	movq	%r11,32(%rdi)
	movq	%r12,40(%rdi)

	mulxq	%rdx,%r8,%r9
	movq	32(%rsi),%rdx
	adoxq	%r8,%r13
	adoxq	%r9,%r14
	adcxq	%r15,%r15
	adcxq	%rcx,%rcx
	movq	%r13,48(%rdi)
	movq	%r14,56(%rdi)

	mulxq	%rdx,%r8,%r9
	movq	40(%rsi),%rdx
	adoxq	%r8,%r15
	adoxq	%r9,%rcx
	adcxq	%rbx,%rbx
	adcxq	%rbp,%rbp
	movq	%r15,64(%rdi)
	movq	%rcx,72(%rdi)

	mulxq	%rdx,%r8,%r9
	adoxq	%r8,%rbx
	adoxq	%r9,%rbp

	movq	%rbx,80(%rdi)
	movq	%rbp,88(%rdi)

	.byte	0xf3,0xc3
.cfi_endproc
.size	__sqrx_384,.-__sqrx_384



.globl	redcx_mont_384
.hidden	redcx_mont_384
.type	redcx_mont_384,@function
.align	32
redcx_mont_384:
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
	call	__mulx_by_1_mont_384
	call	__redc_tail_mont_384

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
.size	redcx_mont_384,.-redcx_mont_384




.globl	fromx_mont_384
.hidden	fromx_mont_384
.type	fromx_mont_384,@function
.align	32
fromx_mont_384:
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
	call	__mulx_by_1_mont_384




	movq	%r14,%rax
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
.size	fromx_mont_384,.-fromx_mont_384
.type	__mulx_by_1_mont_384,@function
.align	32
__mulx_by_1_mont_384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

	movq	0(%rsi),%r8
	movq	%rcx,%rdx
	movq	8(%rsi),%r9
	movq	16(%rsi),%r10
	movq	24(%rsi),%r11
	movq	32(%rsi),%r12
	movq	40(%rsi),%r13
	imulq	%r8,%rdx


	xorq	%r14,%r14
	mulxq	0(%rbx),%rax,%rbp
	adcxq	%rax,%r8
	adoxq	%rbp,%r9

	mulxq	8(%rbx),%rax,%rbp
	adcxq	%rax,%r9
	adoxq	%rbp,%r10

	mulxq	16(%rbx),%rax,%rbp
	adcxq	%rax,%r10
	adoxq	%rbp,%r11

	mulxq	24(%rbx),%rax,%rbp
	adcxq	%rax,%r11
	adoxq	%rbp,%r12

	mulxq	32(%rbx),%rax,%rbp
	adcxq	%rax,%r12
	adoxq	%rbp,%r13

	mulxq	40(%rbx),%rax,%rbp
	movq	%rcx,%rdx
	adcxq	%rax,%r13
	adoxq	%r14,%rbp
	adcxq	%rbp,%r14
	imulq	%r9,%rdx


	xorq	%r15,%r15
	mulxq	0(%rbx),%rax,%rbp
	adcxq	%rax,%r9
	adoxq	%rbp,%r10

	mulxq	8(%rbx),%rax,%rbp
	adcxq	%rax,%r10
	adoxq	%rbp,%r11

	mulxq	16(%rbx),%rax,%rbp
	adcxq	%rax,%r11
	adoxq	%rbp,%r12

	mulxq	24(%rbx),%rax,%rbp
	adcxq	%rax,%r12
	adoxq	%rbp,%r13

	mulxq	32(%rbx),%rax,%rbp
	adcxq	%rax,%r13
	adoxq	%rbp,%r14

	mulxq	40(%rbx),%rax,%rbp
	movq	%rcx,%rdx
	adcxq	%rax,%r14
	adoxq	%r15,%rbp
	adcxq	%rbp,%r15
	imulq	%r10,%rdx


	xorq	%r8,%r8
	mulxq	0(%rbx),%rax,%rbp
	adcxq	%rax,%r10
	adoxq	%rbp,%r11

	mulxq	8(%rbx),%rax,%rbp
	adcxq	%rax,%r11
	adoxq	%rbp,%r12

	mulxq	16(%rbx),%rax,%rbp
	adcxq	%rax,%r12
	adoxq	%rbp,%r13

	mulxq	24(%rbx),%rax,%rbp
	adcxq	%rax,%r13
	adoxq	%rbp,%r14

	mulxq	32(%rbx),%rax,%rbp
	adcxq	%rax,%r14
	adoxq	%rbp,%r15

	mulxq	40(%rbx),%rax,%rbp
	movq	%rcx,%rdx
	adcxq	%rax,%r15
	adoxq	%r8,%rbp
	adcxq	%rbp,%r8
	imulq	%r11,%rdx


	xorq	%r9,%r9
	mulxq	0(%rbx),%rax,%rbp
	adcxq	%rax,%r11
	adoxq	%rbp,%r12

	mulxq	8(%rbx),%rax,%rbp
	adcxq	%rax,%r12
	adoxq	%rbp,%r13

	mulxq	16(%rbx),%rax,%rbp
	adcxq	%rax,%r13
	adoxq	%rbp,%r14

	mulxq	24(%rbx),%rax,%rbp
	adcxq	%rax,%r14
	adoxq	%rbp,%r15

	mulxq	32(%rbx),%rax,%rbp
	adcxq	%rax,%r15
	adoxq	%rbp,%r8

	mulxq	40(%rbx),%rax,%rbp
	movq	%rcx,%rdx
	adcxq	%rax,%r8
	adoxq	%r9,%rbp
	adcxq	%rbp,%r9
	imulq	%r12,%rdx


	xorq	%r10,%r10
	mulxq	0(%rbx),%rax,%rbp
	adcxq	%rax,%r12
	adoxq	%rbp,%r13

	mulxq	8(%rbx),%rax,%rbp
	adcxq	%rax,%r13
	adoxq	%rbp,%r14

	mulxq	16(%rbx),%rax,%rbp
	adcxq	%rax,%r14
	adoxq	%rbp,%r15

	mulxq	24(%rbx),%rax,%rbp
	adcxq	%rax,%r15
	adoxq	%rbp,%r8

	mulxq	32(%rbx),%rax,%rbp
	adcxq	%rax,%r8
	adoxq	%rbp,%r9

	mulxq	40(%rbx),%rax,%rbp
	movq	%rcx,%rdx
	adcxq	%rax,%r9
	adoxq	%r10,%rbp
	adcxq	%rbp,%r10
	imulq	%r13,%rdx


	xorq	%r11,%r11
	mulxq	0(%rbx),%rax,%rbp
	adcxq	%rax,%r13
	adoxq	%rbp,%r14

	mulxq	8(%rbx),%rax,%rbp
	adcxq	%rax,%r14
	adoxq	%rbp,%r15

	mulxq	16(%rbx),%rax,%rbp
	adcxq	%rax,%r15
	adoxq	%rbp,%r8

	mulxq	24(%rbx),%rax,%rbp
	adcxq	%rax,%r8
	adoxq	%rbp,%r9

	mulxq	32(%rbx),%rax,%rbp
	adcxq	%rax,%r9
	adoxq	%rbp,%r10

	mulxq	40(%rbx),%rax,%rbp
	movq	%rcx,%rdx
	adcxq	%rax,%r10
	adoxq	%r11,%rbp
	adcxq	%rbp,%r11
	.byte	0xf3,0xc3
.cfi_endproc
.size	__mulx_by_1_mont_384,.-__mulx_by_1_mont_384

.type	__redc_tail_mont_384,@function
.align	32
__redc_tail_mont_384:
.cfi_startproc
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

	.byte	0xf3,0xc3
.cfi_endproc
.size	__redc_tail_mont_384,.-__redc_tail_mont_384

.globl	sgn0x_pty_mont_384
.hidden	sgn0x_pty_mont_384
.type	sgn0x_pty_mont_384,@function
.align	32
sgn0x_pty_mont_384:
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
	leaq	0(%rdi),%rsi
	movq	%rdx,%rcx
	call	__mulx_by_1_mont_384

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
.size	sgn0x_pty_mont_384,.-sgn0x_pty_mont_384

.globl	sgn0x_pty_mont_384x
.hidden	sgn0x_pty_mont_384x
.type	sgn0x_pty_mont_384x,@function
.align	32
sgn0x_pty_mont_384x:
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
	leaq	48(%rdi),%rsi
	movq	%rdx,%rcx
	call	__mulx_by_1_mont_384

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

	call	__mulx_by_1_mont_384

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
.size	sgn0x_pty_mont_384x,.-sgn0x_pty_mont_384x
.globl	mulx_mont_384
.hidden	mulx_mont_384
.type	mulx_mont_384,@function
.align	32
mulx_mont_384:
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
	leaq	-24(%rsp),%rsp
.cfi_adjust_cfa_offset	8*3


	movq	%rdx,%rbx
	movq	0(%rdx),%rdx
	movq	0(%rsi),%r14
	movq	8(%rsi),%r15
	movq	16(%rsi),%rax
	movq	24(%rsi),%r12
	movq	%rdi,16(%rsp)
	movq	32(%rsi),%rdi
	movq	40(%rsi),%rbp
	leaq	-128(%rsi),%rsi
	leaq	-128(%rcx),%rcx
	movq	%r8,(%rsp)

	mulxq	%r14,%r8,%r9
	call	__mulx_mont_384

	movq	24(%rsp),%r15
.cfi_restore	%r15
	movq	32(%rsp),%r14
.cfi_restore	%r14
	movq	40(%rsp),%r13
.cfi_restore	%r13
	movq	48(%rsp),%r12
.cfi_restore	%r12
	movq	56(%rsp),%rbx
.cfi_restore	%rbx
	movq	64(%rsp),%rbp
.cfi_restore	%rbp
	leaq	72(%rsp),%rsp
.cfi_adjust_cfa_offset	-8*9

	.byte	0xf3,0xc3
.cfi_endproc	
.size	mulx_mont_384,.-mulx_mont_384
.type	__mulx_mont_384,@function
.align	32
__mulx_mont_384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa


	mulxq	%r15,%r14,%r10
	mulxq	%rax,%r15,%r11
	addq	%r14,%r9
	mulxq	%r12,%rax,%r12
	adcq	%r15,%r10
	mulxq	%rdi,%rdi,%r13
	adcq	%rax,%r11
	mulxq	%rbp,%rbp,%r14
	movq	8(%rbx),%rdx
	adcq	%rdi,%r12
	adcq	%rbp,%r13
	adcq	$0,%r14
	xorq	%r15,%r15

	movq	%r8,16(%rsp)
	imulq	8(%rsp),%r8


	xorq	%rax,%rax
	mulxq	0+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r9
	adcxq	%rbp,%r10

	mulxq	8+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r10
	adcxq	%rbp,%r11

	mulxq	16+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r11
	adcxq	%rbp,%r12

	mulxq	24+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r12
	adcxq	%rbp,%r13

	mulxq	32+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r13
	adcxq	%rbp,%r14

	mulxq	40+128(%rsi),%rdi,%rbp
	movq	%r8,%rdx
	adoxq	%rdi,%r14
	adcxq	%rbp,%r15
	adoxq	%rax,%r15
	adoxq	%rax,%rax


	xorq	%r8,%r8
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	16(%rsp),%rdi
	adoxq	%rbp,%r9

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r9
	adoxq	%rbp,%r10

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r10
	adoxq	%rbp,%r11

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r11
	adoxq	%rbp,%r12

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r12
	adoxq	%rbp,%r13

	mulxq	40+128(%rcx),%rdi,%rbp
	movq	16(%rbx),%rdx
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14
	adcxq	%r8,%r14
	adoxq	%r8,%r15
	adcxq	%r8,%r15
	adoxq	%r8,%rax
	adcxq	%r8,%rax
	movq	%r9,16(%rsp)
	imulq	8(%rsp),%r9


	xorq	%r8,%r8
	mulxq	0+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r10
	adcxq	%rbp,%r11

	mulxq	8+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r11
	adcxq	%rbp,%r12

	mulxq	16+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r12
	adcxq	%rbp,%r13

	mulxq	24+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r13
	adcxq	%rbp,%r14

	mulxq	32+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r14
	adcxq	%rbp,%r15

	mulxq	40+128(%rsi),%rdi,%rbp
	movq	%r9,%rdx
	adoxq	%rdi,%r15
	adcxq	%rbp,%rax
	adoxq	%r8,%rax
	adoxq	%r8,%r8


	xorq	%r9,%r9
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	16(%rsp),%rdi
	adoxq	%rbp,%r10

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r10
	adoxq	%rbp,%r11

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r11
	adoxq	%rbp,%r12

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r12
	adoxq	%rbp,%r13

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14

	mulxq	40+128(%rcx),%rdi,%rbp
	movq	24(%rbx),%rdx
	adcxq	%rdi,%r14
	adoxq	%rbp,%r15
	adcxq	%r9,%r15
	adoxq	%r9,%rax
	adcxq	%r9,%rax
	adoxq	%r9,%r8
	adcxq	%r9,%r8
	movq	%r10,16(%rsp)
	imulq	8(%rsp),%r10


	xorq	%r9,%r9
	mulxq	0+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r11
	adcxq	%rbp,%r12

	mulxq	8+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r12
	adcxq	%rbp,%r13

	mulxq	16+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r13
	adcxq	%rbp,%r14

	mulxq	24+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r14
	adcxq	%rbp,%r15

	mulxq	32+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r15
	adcxq	%rbp,%rax

	mulxq	40+128(%rsi),%rdi,%rbp
	movq	%r10,%rdx
	adoxq	%rdi,%rax
	adcxq	%rbp,%r8
	adoxq	%r9,%r8
	adoxq	%r9,%r9


	xorq	%r10,%r10
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	16(%rsp),%rdi
	adoxq	%rbp,%r11

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r11
	adoxq	%rbp,%r12

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r12
	adoxq	%rbp,%r13

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r14
	adoxq	%rbp,%r15

	mulxq	40+128(%rcx),%rdi,%rbp
	movq	32(%rbx),%rdx
	adcxq	%rdi,%r15
	adoxq	%rbp,%rax
	adcxq	%r10,%rax
	adoxq	%r10,%r8
	adcxq	%r10,%r8
	adoxq	%r10,%r9
	adcxq	%r10,%r9
	movq	%r11,16(%rsp)
	imulq	8(%rsp),%r11


	xorq	%r10,%r10
	mulxq	0+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r12
	adcxq	%rbp,%r13

	mulxq	8+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r13
	adcxq	%rbp,%r14

	mulxq	16+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r14
	adcxq	%rbp,%r15

	mulxq	24+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r15
	adcxq	%rbp,%rax

	mulxq	32+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%rax
	adcxq	%rbp,%r8

	mulxq	40+128(%rsi),%rdi,%rbp
	movq	%r11,%rdx
	adoxq	%rdi,%r8
	adcxq	%rbp,%r9
	adoxq	%r10,%r9
	adoxq	%r10,%r10


	xorq	%r11,%r11
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	16(%rsp),%rdi
	adoxq	%rbp,%r12

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r12
	adoxq	%rbp,%r13

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r14
	adoxq	%rbp,%r15

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r15
	adoxq	%rbp,%rax

	mulxq	40+128(%rcx),%rdi,%rbp
	movq	40(%rbx),%rdx
	adcxq	%rdi,%rax
	adoxq	%rbp,%r8
	adcxq	%r11,%r8
	adoxq	%r11,%r9
	adcxq	%r11,%r9
	adoxq	%r11,%r10
	adcxq	%r11,%r10
	movq	%r12,16(%rsp)
	imulq	8(%rsp),%r12


	xorq	%r11,%r11
	mulxq	0+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r13
	adcxq	%rbp,%r14

	mulxq	8+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r14
	adcxq	%rbp,%r15

	mulxq	16+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r15
	adcxq	%rbp,%rax

	mulxq	24+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%rax
	adcxq	%rbp,%r8

	mulxq	32+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r8
	adcxq	%rbp,%r9

	mulxq	40+128(%rsi),%rdi,%rbp
	movq	%r12,%rdx
	adoxq	%rdi,%r9
	adcxq	%rbp,%r10
	adoxq	%r11,%r10
	adoxq	%r11,%r11


	xorq	%r12,%r12
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	16(%rsp),%rdi
	adoxq	%rbp,%r13

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r14
	adoxq	%rbp,%r15

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r15
	adoxq	%rbp,%rax

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%rax
	adoxq	%rbp,%r8

	mulxq	40+128(%rcx),%rdi,%rbp
	movq	%r13,%rdx
	adcxq	%rdi,%r8
	adoxq	%rbp,%r9
	adcxq	%r12,%r9
	adoxq	%r12,%r10
	adcxq	%r12,%r10
	adoxq	%r12,%r11
	adcxq	%r12,%r11
	imulq	8(%rsp),%rdx
	movq	24(%rsp),%rbx


	xorq	%r12,%r12
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r14
	adoxq	%rbp,%r15

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r15
	adoxq	%rbp,%rax

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%rax
	adoxq	%rbp,%r8
	movq	%r15,%r13

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r8
	adoxq	%rbp,%r9
	movq	%rax,%rsi

	mulxq	40+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r9
	adoxq	%rbp,%r10
	movq	%r14,%rdx
	adcxq	%r12,%r10
	adoxq	%r12,%r11
	leaq	128(%rcx),%rcx
	movq	%r8,%r12
	adcq	$0,%r11




	subq	0(%rcx),%r14
	sbbq	8(%rcx),%r15
	movq	%r9,%rdi
	sbbq	16(%rcx),%rax
	sbbq	24(%rcx),%r8
	sbbq	32(%rcx),%r9
	movq	%r10,%rbp
	sbbq	40(%rcx),%r10
	sbbq	$0,%r11

	cmovncq	%r14,%rdx
	cmovcq	%r13,%r15
	cmovcq	%rsi,%rax
	cmovncq	%r8,%r12
	movq	%rdx,0(%rbx)
	cmovncq	%r9,%rdi
	movq	%r15,8(%rbx)
	cmovncq	%r10,%rbp
	movq	%rax,16(%rbx)
	movq	%r12,24(%rbx)
	movq	%rdi,32(%rbx)
	movq	%rbp,40(%rbx)

	.byte	0xf3,0xc3
.cfi_endproc	
.size	__mulx_mont_384,.-__mulx_mont_384
.globl	sqrx_mont_384
.hidden	sqrx_mont_384
.type	sqrx_mont_384,@function
.align	32
sqrx_mont_384:
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
	leaq	-24(%rsp),%rsp
.cfi_adjust_cfa_offset	8*3


	movq	%rcx,%r8
	leaq	-128(%rdx),%rcx
	movq	0(%rsi),%rdx
	movq	8(%rsi),%r15
	movq	16(%rsi),%rax
	movq	24(%rsi),%r12
	movq	%rdi,16(%rsp)
	movq	32(%rsi),%rdi
	movq	40(%rsi),%rbp

	leaq	(%rsi),%rbx
	movq	%r8,(%rsp)
	leaq	-128(%rsi),%rsi

	mulxq	%rdx,%r8,%r9
	call	__mulx_mont_384

	movq	24(%rsp),%r15
.cfi_restore	%r15
	movq	32(%rsp),%r14
.cfi_restore	%r14
	movq	40(%rsp),%r13
.cfi_restore	%r13
	movq	48(%rsp),%r12
.cfi_restore	%r12
	movq	56(%rsp),%rbx
.cfi_restore	%rbx
	movq	64(%rsp),%rbp
.cfi_restore	%rbp
	leaq	72(%rsp),%rsp
.cfi_adjust_cfa_offset	-8*9

	.byte	0xf3,0xc3
.cfi_endproc	
.size	sqrx_mont_384,.-sqrx_mont_384

.globl	sqrx_n_mul_mont_384
.hidden	sqrx_n_mul_mont_384
.type	sqrx_n_mul_mont_384,@function
.align	32
sqrx_n_mul_mont_384:
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
	leaq	-40(%rsp),%rsp
.cfi_adjust_cfa_offset	8*5


	movq	%rdx,%r10
	movq	0(%rsi),%rdx
	movq	8(%rsi),%r15
	movq	16(%rsi),%rax
	movq	%rsi,%rbx
	movq	24(%rsi),%r12
	movq	%rdi,16(%rsp)
	movq	32(%rsi),%rdi
	movq	40(%rsi),%rbp

	movq	%r8,(%rsp)
	movq	%r9,24(%rsp)
	movq	0(%r9),%xmm2

.Loop_sqrx_384:
	movd	%r10d,%xmm1
	leaq	-128(%rbx),%rsi
	leaq	-128(%rcx),%rcx

	mulxq	%rdx,%r8,%r9
	call	__mulx_mont_384

	movd	%xmm1,%r10d
	decl	%r10d
	jnz	.Loop_sqrx_384

	movq	%rdx,%r14
.byte	102,72,15,126,210
	leaq	-128(%rbx),%rsi
	movq	24(%rsp),%rbx
	leaq	-128(%rcx),%rcx

	mulxq	%r14,%r8,%r9
	call	__mulx_mont_384

	movq	40(%rsp),%r15
.cfi_restore	%r15
	movq	48(%rsp),%r14
.cfi_restore	%r14
	movq	56(%rsp),%r13
.cfi_restore	%r13
	movq	64(%rsp),%r12
.cfi_restore	%r12
	movq	72(%rsp),%rbx
.cfi_restore	%rbx
	movq	80(%rsp),%rbp
.cfi_restore	%rbp
	leaq	88(%rsp),%rsp
.cfi_adjust_cfa_offset	-8*11

	.byte	0xf3,0xc3
.cfi_endproc	
.size	sqrx_n_mul_mont_384,.-sqrx_n_mul_mont_384

.globl	sqrx_n_mul_mont_383
.hidden	sqrx_n_mul_mont_383
.type	sqrx_n_mul_mont_383,@function
.align	32
sqrx_n_mul_mont_383:
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
	leaq	-40(%rsp),%rsp
.cfi_adjust_cfa_offset	8*5


	movq	%rdx,%r10
	movq	0(%rsi),%rdx
	movq	8(%rsi),%r15
	movq	16(%rsi),%rax
	movq	%rsi,%rbx
	movq	24(%rsi),%r12
	movq	%rdi,16(%rsp)
	movq	32(%rsi),%rdi
	movq	40(%rsi),%rbp

	movq	%r8,(%rsp)
	movq	%r9,24(%rsp)
	movq	0(%r9),%xmm2
	leaq	-128(%rcx),%rcx

.Loop_sqrx_383:
	movd	%r10d,%xmm1
	leaq	-128(%rbx),%rsi

	mulxq	%rdx,%r8,%r9
	call	__mulx_mont_383_nonred

	movd	%xmm1,%r10d
	decl	%r10d
	jnz	.Loop_sqrx_383

	movq	%rdx,%r14
.byte	102,72,15,126,210
	leaq	-128(%rbx),%rsi
	movq	24(%rsp),%rbx

	mulxq	%r14,%r8,%r9
	call	__mulx_mont_384

	movq	40(%rsp),%r15
.cfi_restore	%r15
	movq	48(%rsp),%r14
.cfi_restore	%r14
	movq	56(%rsp),%r13
.cfi_restore	%r13
	movq	64(%rsp),%r12
.cfi_restore	%r12
	movq	72(%rsp),%rbx
.cfi_restore	%rbx
	movq	80(%rsp),%rbp
.cfi_restore	%rbp
	leaq	88(%rsp),%rsp
.cfi_adjust_cfa_offset	-8*11

	.byte	0xf3,0xc3
.cfi_endproc	
.size	sqrx_n_mul_mont_383,.-sqrx_n_mul_mont_383
.type	__mulx_mont_383_nonred,@function
.align	32
__mulx_mont_383_nonred:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa


	mulxq	%r15,%r14,%r10
	mulxq	%rax,%r15,%r11
	addq	%r14,%r9
	mulxq	%r12,%rax,%r12
	adcq	%r15,%r10
	mulxq	%rdi,%rdi,%r13
	adcq	%rax,%r11
	mulxq	%rbp,%rbp,%r14
	movq	8(%rbx),%rdx
	adcq	%rdi,%r12
	adcq	%rbp,%r13
	adcq	$0,%r14
	movq	%r8,%rax
	imulq	8(%rsp),%r8


	xorq	%r15,%r15
	mulxq	0+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r9
	adcxq	%rbp,%r10

	mulxq	8+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r10
	adcxq	%rbp,%r11

	mulxq	16+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r11
	adcxq	%rbp,%r12

	mulxq	24+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r12
	adcxq	%rbp,%r13

	mulxq	32+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r13
	adcxq	%rbp,%r14

	mulxq	40+128(%rsi),%rdi,%rbp
	movq	%r8,%rdx
	adoxq	%rdi,%r14
	adcxq	%r15,%rbp
	adoxq	%rbp,%r15


	xorq	%r8,%r8
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%rax
	adoxq	%rbp,%r9

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r9
	adoxq	%rbp,%r10

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r10
	adoxq	%rbp,%r11

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r11
	adoxq	%rbp,%r12

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r12
	adoxq	%rbp,%r13

	mulxq	40+128(%rcx),%rdi,%rbp
	movq	16(%rbx),%rdx
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14
	adcxq	%rax,%r14
	adoxq	%rax,%r15
	adcxq	%rax,%r15
	movq	%r9,%r8
	imulq	8(%rsp),%r9


	xorq	%rax,%rax
	mulxq	0+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r10
	adcxq	%rbp,%r11

	mulxq	8+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r11
	adcxq	%rbp,%r12

	mulxq	16+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r12
	adcxq	%rbp,%r13

	mulxq	24+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r13
	adcxq	%rbp,%r14

	mulxq	32+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r14
	adcxq	%rbp,%r15

	mulxq	40+128(%rsi),%rdi,%rbp
	movq	%r9,%rdx
	adoxq	%rdi,%r15
	adcxq	%rax,%rbp
	adoxq	%rbp,%rax


	xorq	%r9,%r9
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r8
	adoxq	%rbp,%r10

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r10
	adoxq	%rbp,%r11

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r11
	adoxq	%rbp,%r12

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r12
	adoxq	%rbp,%r13

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14

	mulxq	40+128(%rcx),%rdi,%rbp
	movq	24(%rbx),%rdx
	adcxq	%rdi,%r14
	adoxq	%rbp,%r15
	adcxq	%r8,%r15
	adoxq	%r8,%rax
	adcxq	%r8,%rax
	movq	%r10,%r9
	imulq	8(%rsp),%r10


	xorq	%r8,%r8
	mulxq	0+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r11
	adcxq	%rbp,%r12

	mulxq	8+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r12
	adcxq	%rbp,%r13

	mulxq	16+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r13
	adcxq	%rbp,%r14

	mulxq	24+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r14
	adcxq	%rbp,%r15

	mulxq	32+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r15
	adcxq	%rbp,%rax

	mulxq	40+128(%rsi),%rdi,%rbp
	movq	%r10,%rdx
	adoxq	%rdi,%rax
	adcxq	%r8,%rbp
	adoxq	%rbp,%r8


	xorq	%r10,%r10
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r9
	adoxq	%rbp,%r11

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r11
	adoxq	%rbp,%r12

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r12
	adoxq	%rbp,%r13

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r14
	adoxq	%rbp,%r15

	mulxq	40+128(%rcx),%rdi,%rbp
	movq	32(%rbx),%rdx
	adcxq	%rdi,%r15
	adoxq	%rbp,%rax
	adcxq	%r9,%rax
	adoxq	%r9,%r8
	adcxq	%r9,%r8
	movq	%r11,%r10
	imulq	8(%rsp),%r11


	xorq	%r9,%r9
	mulxq	0+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r12
	adcxq	%rbp,%r13

	mulxq	8+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r13
	adcxq	%rbp,%r14

	mulxq	16+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r14
	adcxq	%rbp,%r15

	mulxq	24+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r15
	adcxq	%rbp,%rax

	mulxq	32+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%rax
	adcxq	%rbp,%r8

	mulxq	40+128(%rsi),%rdi,%rbp
	movq	%r11,%rdx
	adoxq	%rdi,%r8
	adcxq	%r9,%rbp
	adoxq	%rbp,%r9


	xorq	%r11,%r11
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r10
	adoxq	%rbp,%r12

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r12
	adoxq	%rbp,%r13

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r14
	adoxq	%rbp,%r15

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r15
	adoxq	%rbp,%rax

	mulxq	40+128(%rcx),%rdi,%rbp
	movq	40(%rbx),%rdx
	adcxq	%rdi,%rax
	adoxq	%rbp,%r8
	adcxq	%r10,%r8
	adoxq	%r10,%r9
	adcxq	%r10,%r9
	movq	%r12,%r11
	imulq	8(%rsp),%r12


	xorq	%r10,%r10
	mulxq	0+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r13
	adcxq	%rbp,%r14

	mulxq	8+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r14
	adcxq	%rbp,%r15

	mulxq	16+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r15
	adcxq	%rbp,%rax

	mulxq	24+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%rax
	adcxq	%rbp,%r8

	mulxq	32+128(%rsi),%rdi,%rbp
	adoxq	%rdi,%r8
	adcxq	%rbp,%r9

	mulxq	40+128(%rsi),%rdi,%rbp
	movq	%r12,%rdx
	adoxq	%rdi,%r9
	adcxq	%r10,%rbp
	adoxq	%rbp,%r10


	xorq	%r12,%r12
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r11
	adoxq	%rbp,%r13

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r14
	adoxq	%rbp,%r15

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r15
	adoxq	%rbp,%rax

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%rax
	adoxq	%rbp,%r8

	mulxq	40+128(%rcx),%rdi,%rbp
	movq	%r13,%rdx
	adcxq	%rdi,%r8
	adoxq	%rbp,%r9
	adcxq	%r11,%r9
	adoxq	%r11,%r10
	adcxq	%r11,%r10
	imulq	8(%rsp),%rdx
	movq	24(%rsp),%rbx


	xorq	%r12,%r12
	mulxq	0+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r13
	adoxq	%rbp,%r14

	mulxq	8+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r14
	adoxq	%rbp,%r15

	mulxq	16+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r15
	adoxq	%rbp,%rax

	mulxq	24+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%rax
	adoxq	%rbp,%r8

	mulxq	32+128(%rcx),%rdi,%rbp
	adcxq	%rdi,%r8
	adoxq	%rbp,%r9

	mulxq	40+128(%rcx),%rdi,%rbp
	movq	%r14,%rdx
	adcxq	%rdi,%r9
	adoxq	%rbp,%r10
	adcq	$0,%r10
	movq	%r8,%r12

	movq	%r14,0(%rbx)
	movq	%r15,8(%rbx)
	movq	%rax,16(%rbx)
	movq	%r9,%rdi
	movq	%r8,24(%rbx)
	movq	%r9,32(%rbx)
	movq	%r10,40(%rbx)
	movq	%r10,%rbp

	.byte	0xf3,0xc3
.cfi_endproc	
.size	__mulx_mont_383_nonred,.-__mulx_mont_383_nonred
.globl	sqrx_mont_382x
.hidden	sqrx_mont_382x
.type	sqrx_mont_382x,@function
.align	32
sqrx_mont_382x:
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
	subq	$136,%rsp
.cfi_adjust_cfa_offset	136


	movq	%rcx,0(%rsp)
	movq	%rdx,%rcx
	movq	%rdi,16(%rsp)
	movq	%rsi,24(%rsp)


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

	movq	48(%rsi),%rdx
	movq	0(%rsi),%r14
	movq	8(%rsi),%r15
	movq	16(%rsi),%rax
	movq	24(%rsi),%r12
	movq	32(%rsi),%rdi
	movq	40(%rsi),%rbp
	leaq	-128(%rsi),%rsi
	leaq	-128(%rcx),%rcx

	mulxq	%r14,%r8,%r9
	call	__mulx_mont_383_nonred
	addq	%rdx,%rdx
	adcq	%r15,%r15
	adcq	%rax,%rax
	adcq	%r12,%r12
	adcq	%rdi,%rdi
	adcq	%rbp,%rbp

	movq	%rdx,48(%rbx)
	movq	%r15,56(%rbx)
	movq	%rax,64(%rbx)
	movq	%r12,72(%rbx)
	movq	%rdi,80(%rbx)
	movq	%rbp,88(%rbx)

	leaq	32-128(%rsp),%rsi
	leaq	32+48(%rsp),%rbx

	movq	32+48(%rsp),%rdx
	movq	32+0(%rsp),%r14
	movq	32+8(%rsp),%r15
	movq	32+16(%rsp),%rax
	movq	32+24(%rsp),%r12
	movq	32+32(%rsp),%rdi
	movq	32+40(%rsp),%rbp



	mulxq	%r14,%r8,%r9
	call	__mulx_mont_383_nonred
	movq	32+96(%rsp),%r14
	leaq	128(%rcx),%rcx
	movq	32+0(%rsp),%r8
	andq	%r14,%r8
	movq	32+8(%rsp),%r9
	andq	%r14,%r9
	movq	32+16(%rsp),%r10
	andq	%r14,%r10
	movq	32+24(%rsp),%r11
	andq	%r14,%r11
	movq	32+32(%rsp),%r13
	andq	%r14,%r13
	andq	32+40(%rsp),%r14

	subq	%r8,%rdx
	movq	0(%rcx),%r8
	sbbq	%r9,%r15
	movq	8(%rcx),%r9
	sbbq	%r10,%rax
	movq	16(%rcx),%r10
	sbbq	%r11,%r12
	movq	24(%rcx),%r11
	sbbq	%r13,%rdi
	movq	32(%rcx),%r13
	sbbq	%r14,%rbp
	sbbq	%r14,%r14

	andq	%r14,%r8
	andq	%r14,%r9
	andq	%r14,%r10
	andq	%r14,%r11
	andq	%r14,%r13
	andq	40(%rcx),%r14

	addq	%r8,%rdx
	adcq	%r9,%r15
	adcq	%r10,%rax
	adcq	%r11,%r12
	adcq	%r13,%rdi
	adcq	%r14,%rbp

	movq	%rdx,0(%rbx)
	movq	%r15,8(%rbx)
	movq	%rax,16(%rbx)
	movq	%r12,24(%rbx)
	movq	%rdi,32(%rbx)
	movq	%rbp,40(%rbx)
	leaq	136(%rsp),%r8
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
.cfi_adjust_cfa_offset	-136-8*6

	.byte	0xf3,0xc3
.cfi_endproc	
.size	sqrx_mont_382x,.-sqrx_mont_382x

.section	.note.GNU-stack,"",@progbits
.section	.note.gnu.property,"a",@note
	.long	4,2f-1f,5
	.byte	0x47,0x4E,0x55,0
1:	.long	0xc0000002,4,3
.align	8
2:
