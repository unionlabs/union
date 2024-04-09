.text	

.type	__add_mod_384x384,@function
.align	32
__add_mod_384x384:
.cfi_startproc
	.byte	0xf3,0x0f,0x1e,0xfa

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

	.byte	0xf3,0xc3
.cfi_endproc
.size	__add_mod_384x384,.-__add_mod_384x384

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

.globl	add_mod_384x384
.hidden	add_mod_384x384
.type	add_mod_384x384,@function
.align	32
add_mod_384x384:
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


	call	__add_mod_384x384

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
.size	add_mod_384x384,.-add_mod_384x384

.globl	sub_mod_384x384
.hidden	sub_mod_384x384
.type	sub_mod_384x384,@function
.align	32
sub_mod_384x384:
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


	call	__sub_mod_384x384

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
.size	sub_mod_384x384,.-sub_mod_384x384

.section	.note.GNU-stack,"",@progbits
.section	.note.gnu.property,"a",@note
	.long	4,2f-1f,5
	.byte	0x47,0x4E,0x55,0
1:	.long	0xc0000002,4,3
.align	8
2:
