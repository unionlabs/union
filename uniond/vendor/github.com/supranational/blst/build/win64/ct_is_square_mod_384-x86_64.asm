OPTION	DOTNAME
.text$	SEGMENT ALIGN(256) 'CODE'

PUBLIC	ct_is_square_mod_384

ALIGN	32
ct_is_square_mod_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_ct_is_square_mod_384::
	mov	rdi,rcx
	mov	rsi,rdx



	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,536

$L$SEH_body_ct_is_square_mod_384::


	lea	rax,QWORD PTR[((24+255))+rsp]
	and	rax,-256

	mov	r8,QWORD PTR[rdi]
	mov	r9,QWORD PTR[8+rdi]
	mov	r10,QWORD PTR[16+rdi]
	mov	r11,QWORD PTR[24+rdi]
	mov	r12,QWORD PTR[32+rdi]
	mov	r13,QWORD PTR[40+rdi]

	mov	r14,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	rbx,QWORD PTR[16+rsi]
	mov	rcx,QWORD PTR[24+rsi]
	mov	rdx,QWORD PTR[32+rsi]
	mov	rdi,QWORD PTR[40+rsi]
	mov	rsi,rax

	mov	QWORD PTR[rax],r8
	mov	QWORD PTR[8+rax],r9
	mov	QWORD PTR[16+rax],r10
	mov	QWORD PTR[24+rax],r11
	mov	QWORD PTR[32+rax],r12
	mov	QWORD PTR[40+rax],r13

	mov	QWORD PTR[48+rax],r14
	mov	QWORD PTR[56+rax],r15
	mov	QWORD PTR[64+rax],rbx
	mov	QWORD PTR[72+rax],rcx
	mov	QWORD PTR[80+rax],rdx
	mov	QWORD PTR[88+rax],rdi

	xor	rbp,rbp
	mov	ecx,24
	jmp	$L$oop_is_square

ALIGN	32
$L$oop_is_square::
	mov	DWORD PTR[16+rsp],ecx

	call	__ab_approximation_30
	mov	QWORD PTR[rsp],rax
	mov	QWORD PTR[8+rsp],rbx

	mov	rdi,128+8*6
	xor	rdi,rsi
	call	__smulq_384_n_shift_by_30

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rdi,QWORD PTR[((-48))+rdi]
	call	__smulq_384_n_shift_by_30

	mov	ecx,DWORD PTR[16+rsp]
	xor	rsi,128

	and	r14,QWORD PTR[48+rdi]
	shr	r14,1
	add	rbp,r14

	sub	ecx,1
	jnz	$L$oop_is_square




	mov	r9,QWORD PTR[48+rsi]
	call	__inner_loop_48

	mov	rax,1
	and	rax,rbp
	xor	rax,1

	lea	r8,QWORD PTR[536+rsp]
	mov	r15,QWORD PTR[r8]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_ct_is_square_mod_384::
	mov	rdi,QWORD PTR[8+rsp]	;WIN64 epilogue
	mov	rsi,QWORD PTR[16+rsp]

	DB	0F3h,0C3h		;repret

$L$SEH_end_ct_is_square_mod_384::
ct_is_square_mod_384	ENDP


ALIGN	32
__smulq_384_n_shift_by_30	PROC PRIVATE
	DB	243,15,30,250
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

	mov	rbx,rdx
	sar	rdx,63
	xor	rax,rax
	sub	rax,rdx

	xor	rbx,rdx
	add	rbx,rax

	xor	r8,rdx
	xor	r9,rdx
	xor	r10,rdx
	xor	r11,rdx
	xor	r12,rdx
	xor	r13,rdx
	add	rax,r8
	adc	r9,0
	adc	r10,0
	adc	r11,0
	adc	r12,0
	adc	r13,0

	mov	r14,rdx
	and	r14,rbx
	mul	rbx
	mov	r8,rax
	mov	rax,r9
	mov	r9,rdx
	mul	rbx
	add	r9,rax
	mov	rax,r10
	adc	rdx,0
	mov	r10,rdx
	mul	rbx
	add	r10,rax
	mov	rax,r11
	adc	rdx,0
	mov	r11,rdx
	mul	rbx
	add	r11,rax
	mov	rax,r12
	adc	rdx,0
	mov	r12,rdx
	mul	rbx
	add	r12,rax
	mov	rax,r13
	adc	rdx,0
	mov	r13,rdx
	neg	r14
	mul	rbx
	add	r13,rax
	adc	r14,rdx
	lea	rsi,QWORD PTR[48+rsi]
	mov	rdx,rcx

	mov	QWORD PTR[rdi],r8
	mov	QWORD PTR[8+rdi],r9
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11
	mov	QWORD PTR[32+rdi],r12
	mov	QWORD PTR[40+rdi],r13
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

	mov	rbx,rdx
	sar	rdx,63
	xor	rax,rax
	sub	rax,rdx

	xor	rbx,rdx
	add	rbx,rax

	xor	r8,rdx
	xor	r9,rdx
	xor	r10,rdx
	xor	r11,rdx
	xor	r12,rdx
	xor	r13,rdx
	add	rax,r8
	adc	r9,0
	adc	r10,0
	adc	r11,0
	adc	r12,0
	adc	r13,0

	mov	r15,rdx
	and	r15,rbx
	mul	rbx
	mov	r8,rax
	mov	rax,r9
	mov	r9,rdx
	mul	rbx
	add	r9,rax
	mov	rax,r10
	adc	rdx,0
	mov	r10,rdx
	mul	rbx
	add	r10,rax
	mov	rax,r11
	adc	rdx,0
	mov	r11,rdx
	mul	rbx
	add	r11,rax
	mov	rax,r12
	adc	rdx,0
	mov	r12,rdx
	mul	rbx
	add	r12,rax
	mov	rax,r13
	adc	rdx,0
	mov	r13,rdx
	neg	r15
	mul	rbx
	add	r13,rax
	adc	r15,rdx
	lea	rsi,QWORD PTR[((-48))+rsi]

	add	r8,QWORD PTR[rdi]
	adc	r9,QWORD PTR[8+rdi]
	adc	r10,QWORD PTR[16+rdi]
	adc	r11,QWORD PTR[24+rdi]
	adc	r12,QWORD PTR[32+rdi]
	adc	r13,QWORD PTR[40+rdi]
	adc	r14,r15

	shrd	r8,r9,30
	shrd	r9,r10,30
	shrd	r10,r11,30
	shrd	r11,r12,30
	shrd	r12,r13,30
	shrd	r13,r14,30

	sar	r14,63
	xor	rbx,rbx
	sub	rbx,r14

	xor	r8,r14
	xor	r9,r14
	xor	r10,r14
	xor	r11,r14
	xor	r12,r14
	xor	r13,r14
	add	r8,rbx
	adc	r9,0
	adc	r10,0
	adc	r11,0
	adc	r12,0
	adc	r13,0

	mov	QWORD PTR[rdi],r8
	mov	QWORD PTR[8+rdi],r9
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11
	mov	QWORD PTR[32+rdi],r12
	mov	QWORD PTR[40+rdi],r13

	DB	0F3h,0C3h		;repret
__smulq_384_n_shift_by_30	ENDP

ALIGN	32
__ab_approximation_30	PROC PRIVATE
	DB	243,15,30,250
	mov	rbx,QWORD PTR[88+rsi]
	mov	r15,QWORD PTR[80+rsi]
	mov	r14,QWORD PTR[72+rsi]

	mov	rax,r13
	or	rax,rbx
	cmovz	r13,r12
	cmovz	rbx,r15
	cmovz	r12,r11
	mov	r11,QWORD PTR[64+rsi]
	cmovz	r15,r14

	mov	rax,r13
	or	rax,rbx
	cmovz	r13,r12
	cmovz	rbx,r15
	cmovz	r12,r10
	mov	r10,QWORD PTR[56+rsi]
	cmovz	r15,r11

	mov	rax,r13
	or	rax,rbx
	cmovz	r13,r12
	cmovz	rbx,r15
	cmovz	r12,r9
	mov	r9,QWORD PTR[48+rsi]
	cmovz	r15,r10

	mov	rax,r13
	or	rax,rbx
	cmovz	r13,r12
	cmovz	rbx,r15
	cmovz	r12,r8
	cmovz	r15,r9

	mov	rax,r13
	or	rax,rbx
	bsr	rcx,rax
	lea	rcx,QWORD PTR[1+rcx]
	cmovz	r13,r8
	cmovz	rbx,r9
	cmovz	rcx,rax
	neg	rcx


	shld	r13,r12,cl
	shld	rbx,r15,cl

	mov	rax,0FFFFFFFF00000000h
	mov	r8d,r8d
	mov	r9d,r9d
	and	r13,rax
	and	rbx,rax
	or	r8,r13
	or	r9,rbx

	jmp	__inner_loop_30

	DB	0F3h,0C3h		;repret
__ab_approximation_30	ENDP

ALIGN	32
__inner_loop_30	PROC PRIVATE
	DB	243,15,30,250
	mov	rbx,07FFFFFFF80000000h
	mov	rcx,0800000007FFFFFFFh
	lea	r15,QWORD PTR[((-1))+rbx]
	mov	edi,30

$L$oop_30::
	mov	rax,r8
	and	rax,r9
	shr	rax,1

	cmp	r8,r9
	mov	r10,r8
	mov	r11,r9
	lea	rax,QWORD PTR[rbp*1+rax]
	mov	r12,rbx
	mov	r13,rcx
	mov	r14,rbp
	cmovb	r8,r9
	cmovb	r9,r10
	cmovb	rbx,rcx
	cmovb	rcx,r12
	cmovb	rbp,rax

	sub	r8,r9
	sub	rbx,rcx
	add	rbx,r15

	test	r10,1
	cmovz	r8,r10
	cmovz	r9,r11
	cmovz	rbx,r12
	cmovz	rcx,r13
	cmovz	rbp,r14

	lea	rax,QWORD PTR[2+r9]
	shr	r8,1
	shr	rax,2
	add	rcx,rcx
	lea	rbp,QWORD PTR[rbp*1+rax]
	sub	rcx,r15

	sub	edi,1
	jnz	$L$oop_30

	shr	r15,32
	mov	eax,ebx
	shr	rbx,32
	mov	edx,ecx
	shr	rcx,32
	sub	rax,r15
	sub	rbx,r15
	sub	rdx,r15
	sub	rcx,r15

	DB	0F3h,0C3h		;repret
__inner_loop_30	ENDP


ALIGN	32
__inner_loop_48	PROC PRIVATE
	DB	243,15,30,250
	mov	edi,48

$L$oop_48::
	mov	rax,r8
	and	rax,r9
	shr	rax,1

	cmp	r8,r9
	mov	r10,r8
	mov	r11,r9
	lea	rax,QWORD PTR[rbp*1+rax]
	mov	r12,rbp
	cmovb	r8,r9
	cmovb	r9,r10
	cmovb	rbp,rax

	sub	r8,r9

	test	r10,1
	cmovz	r8,r10
	cmovz	r9,r11
	cmovz	rbp,r12

	lea	rax,QWORD PTR[2+r9]
	shr	r8,1
	shr	rax,2
	add	rbp,rax

	sub	edi,1
	jnz	$L$oop_48

	DB	0F3h,0C3h		;repret
__inner_loop_48	ENDP
.text$	ENDS
.pdata	SEGMENT READONLY ALIGN(4)
ALIGN	4
	DD	imagerel $L$SEH_begin_ct_is_square_mod_384
	DD	imagerel $L$SEH_body_ct_is_square_mod_384
	DD	imagerel $L$SEH_info_ct_is_square_mod_384_prologue

	DD	imagerel $L$SEH_body_ct_is_square_mod_384
	DD	imagerel $L$SEH_epilogue_ct_is_square_mod_384
	DD	imagerel $L$SEH_info_ct_is_square_mod_384_body

	DD	imagerel $L$SEH_epilogue_ct_is_square_mod_384
	DD	imagerel $L$SEH_end_ct_is_square_mod_384
	DD	imagerel $L$SEH_info_ct_is_square_mod_384_epilogue

.pdata	ENDS
.xdata	SEGMENT READONLY ALIGN(8)
ALIGN	8
$L$SEH_info_ct_is_square_mod_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,003h
DB	0,0
$L$SEH_info_ct_is_square_mod_384_body::
DB	1,0,18,0
DB	000h,0f4h,043h,000h
DB	000h,0e4h,044h,000h
DB	000h,0d4h,045h,000h
DB	000h,0c4h,046h,000h
DB	000h,034h,047h,000h
DB	000h,054h,048h,000h
DB	000h,074h,04ah,000h
DB	000h,064h,04bh,000h
DB	000h,001h,049h,000h
$L$SEH_info_ct_is_square_mod_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h


.xdata	ENDS
END
