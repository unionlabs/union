OPTION	DOTNAME
.text$	SEGMENT ALIGN(256) 'CODE'

PUBLIC	ct_inverse_mod_383

ALIGN	32
ct_inverse_mod_383	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_ct_inverse_mod_383::
	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9



	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,1112

$L$SEH_body_ct_inverse_mod_383::


	lea	rax,QWORD PTR[((88+511))+rsp]
	and	rax,-512
	mov	QWORD PTR[32+rsp],rdi
	mov	QWORD PTR[40+rsp],rcx

	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

	mov	r14,QWORD PTR[rdx]
	mov	r15,QWORD PTR[8+rdx]
	mov	rbx,QWORD PTR[16+rdx]
	mov	rbp,QWORD PTR[24+rdx]
	mov	rsi,QWORD PTR[32+rdx]
	mov	rdi,QWORD PTR[40+rdx]

	mov	QWORD PTR[rax],r8
	mov	QWORD PTR[8+rax],r9
	mov	QWORD PTR[16+rax],r10
	mov	QWORD PTR[24+rax],r11
	mov	QWORD PTR[32+rax],r12
	mov	QWORD PTR[40+rax],r13

	mov	QWORD PTR[48+rax],r14
	mov	QWORD PTR[56+rax],r15
	mov	QWORD PTR[64+rax],rbx
	mov	QWORD PTR[72+rax],rbp
	mov	QWORD PTR[80+rax],rsi
	mov	rsi,rax
	mov	QWORD PTR[88+rax],rdi


	mov	edi,62
	call	__ab_approximation_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_383_n_shift_by_62


	mov	QWORD PTR[96+rdi],rdx

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383_n_shift_by_62


	mov	QWORD PTR[96+rdi],rdx


	xor	rsi,256
	mov	edi,62
	call	__ab_approximation_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[56+rsp],rdx
	mov	QWORD PTR[64+rsp],rcx

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383_n_shift_by_62



	mov	rax,QWORD PTR[96+rsi]
	mov	r11,QWORD PTR[144+rsi]
	mov	rbx,rdx
	mov	r10,rax
	imul	QWORD PTR[56+rsp]
	mov	r8,rax
	mov	rax,r11
	mov	r9,rdx
	imul	QWORD PTR[64+rsp]
	add	r8,rax
	adc	r9,rdx
	mov	QWORD PTR[48+rdi],r8
	mov	QWORD PTR[56+rdi],r9
	sar	r9,63
	mov	QWORD PTR[64+rdi],r9
	mov	QWORD PTR[72+rdi],r9
	mov	QWORD PTR[80+rdi],r9
	mov	QWORD PTR[88+rdi],r9
	lea	rsi,QWORD PTR[96+rsi]

	mov	rax,r10
	imul	rbx
	mov	r8,rax
	mov	rax,r11
	mov	r9,rdx
	imul	rcx
	add	r8,rax
	adc	r9,rdx
	mov	QWORD PTR[96+rdi],r8
	mov	QWORD PTR[104+rdi],r9
	sar	r9,63
	mov	QWORD PTR[112+rdi],r9
	mov	QWORD PTR[120+rdi],r9
	mov	QWORD PTR[128+rdi],r9
	mov	QWORD PTR[136+rdi],r9
	xor	rsi,256+8*12
	mov	edi,62
	call	__ab_approximation_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[56+rsp],rdx
	mov	QWORD PTR[64+rsp],rcx

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[72+rsp],rdx
	mov	QWORD PTR[80+rsp],rcx

	mov	rdx,QWORD PTR[56+rsp]
	mov	rcx,QWORD PTR[64+rsp]
	lea	rsi,QWORD PTR[96+rsi]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63
	xor	rsi,256+8*12
	mov	edi,62
	call	__ab_approximation_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[56+rsp],rdx
	mov	QWORD PTR[64+rsp],rcx

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[72+rsp],rdx
	mov	QWORD PTR[80+rsp],rcx

	mov	rdx,QWORD PTR[56+rsp]
	mov	rcx,QWORD PTR[64+rsp]
	lea	rsi,QWORD PTR[96+rsi]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63
	xor	rsi,256+8*12
	mov	edi,62
	call	__ab_approximation_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[56+rsp],rdx
	mov	QWORD PTR[64+rsp],rcx

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[72+rsp],rdx
	mov	QWORD PTR[80+rsp],rcx

	mov	rdx,QWORD PTR[56+rsp]
	mov	rcx,QWORD PTR[64+rsp]
	lea	rsi,QWORD PTR[96+rsi]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63
	xor	rsi,256+8*12
	mov	edi,62
	call	__ab_approximation_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[56+rsp],rdx
	mov	QWORD PTR[64+rsp],rcx

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[72+rsp],rdx
	mov	QWORD PTR[80+rsp],rcx

	mov	rdx,QWORD PTR[56+rsp]
	mov	rcx,QWORD PTR[64+rsp]
	lea	rsi,QWORD PTR[96+rsi]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63
	sar	r13,63
	mov	QWORD PTR[48+rdi],r13
	mov	QWORD PTR[56+rdi],r13
	mov	QWORD PTR[64+rdi],r13
	mov	QWORD PTR[72+rdi],r13
	mov	QWORD PTR[80+rdi],r13
	mov	QWORD PTR[88+rdi],r13
	xor	rsi,256+8*12
	mov	edi,62
	call	__ab_approximation_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[56+rsp],rdx
	mov	QWORD PTR[64+rsp],rcx

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[72+rsp],rdx
	mov	QWORD PTR[80+rsp],rcx

	mov	rdx,QWORD PTR[56+rsp]
	mov	rcx,QWORD PTR[64+rsp]
	lea	rsi,QWORD PTR[96+rsi]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_767x63
	xor	rsi,256+8*12
	mov	edi,62
	call	__ab_approximation_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[56+rsp],rdx
	mov	QWORD PTR[64+rsp],rcx

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[72+rsp],rdx
	mov	QWORD PTR[80+rsp],rcx

	mov	rdx,QWORD PTR[56+rsp]
	mov	rcx,QWORD PTR[64+rsp]
	lea	rsi,QWORD PTR[96+rsi]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_767x63
	xor	rsi,256+8*12
	mov	edi,62
	call	__ab_approximation_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[56+rsp],rdx
	mov	QWORD PTR[64+rsp],rcx

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[72+rsp],rdx
	mov	QWORD PTR[80+rsp],rcx

	mov	rdx,QWORD PTR[56+rsp]
	mov	rcx,QWORD PTR[64+rsp]
	lea	rsi,QWORD PTR[96+rsi]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_767x63
	xor	rsi,256+8*12
	mov	edi,62
	call	__ab_approximation_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[56+rsp],rdx
	mov	QWORD PTR[64+rsp],rcx

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[72+rsp],rdx
	mov	QWORD PTR[80+rsp],rcx

	mov	rdx,QWORD PTR[56+rsp]
	mov	rcx,QWORD PTR[64+rsp]
	lea	rsi,QWORD PTR[96+rsi]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_767x63
	xor	rsi,256+8*12
	mov	edi,62
	call	__ab_approximation_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[56+rsp],rdx
	mov	QWORD PTR[64+rsp],rcx

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383_n_shift_by_62
	mov	QWORD PTR[72+rsp],rdx
	mov	QWORD PTR[80+rsp],rcx

	mov	rdx,QWORD PTR[56+rsp]
	mov	rcx,QWORD PTR[64+rsp]
	lea	rsi,QWORD PTR[96+rsi]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_383x63

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_767x63

	xor	rsi,256+8*12
	mov	edi,62

	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[48+rsi]
	mov	r11,QWORD PTR[56+rsi]
	call	__inner_loop_62


	mov	QWORD PTR[72+rsp],r12
	mov	QWORD PTR[80+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	mov	QWORD PTR[rdi],r8
	mov	QWORD PTR[48+rdi],r10



	lea	rsi,QWORD PTR[96+rsi]
	lea	rdi,QWORD PTR[96+rdi]
	call	__smulq_383x63

	mov	rdx,QWORD PTR[72+rsp]
	mov	rcx,QWORD PTR[80+rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__smulq_767x63


	xor	rsi,256+8*12
	mov	edi,22

	mov	r8,QWORD PTR[rsi]
	xor	r9,r9
	mov	r10,QWORD PTR[48+rsi]
	xor	r11,r11
	call	__inner_loop_62







	lea	rsi,QWORD PTR[96+rsi]





	mov	rdx,r12
	mov	rcx,r13
	mov	rdi,QWORD PTR[32+rsp]
	call	__smulq_767x63

	mov	rsi,QWORD PTR[40+rsp]
	mov	rdx,rax
	sar	rax,63

	mov	r8,rax
	mov	r9,rax
	mov	r10,rax
	and	r8,QWORD PTR[rsi]
	and	r9,QWORD PTR[8+rsi]
	mov	r11,rax
	and	r10,QWORD PTR[16+rsi]
	and	r11,QWORD PTR[24+rsi]
	mov	r12,rax
	and	r12,QWORD PTR[32+rsi]
	and	rax,QWORD PTR[40+rsi]

	add	r14,r8
	adc	r15,r9
	adc	rbx,r10
	adc	rbp,r11
	adc	rcx,r12
	adc	rdx,rax

	mov	QWORD PTR[48+rdi],r14
	mov	QWORD PTR[56+rdi],r15
	mov	QWORD PTR[64+rdi],rbx
	mov	QWORD PTR[72+rdi],rbp
	mov	QWORD PTR[80+rdi],rcx
	mov	QWORD PTR[88+rdi],rdx

	lea	r8,QWORD PTR[1112+rsp]
	mov	r15,QWORD PTR[r8]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_ct_inverse_mod_383::
	mov	rdi,QWORD PTR[8+rsp]	;WIN64 epilogue
	mov	rsi,QWORD PTR[16+rsp]

	DB	0F3h,0C3h		;repret

$L$SEH_end_ct_inverse_mod_383::
ct_inverse_mod_383	ENDP

ALIGN	32
__smulq_767x63	PROC PRIVATE
	DB	243,15,30,250
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

	mov	rbp,rdx
	sar	rdx,63
	xor	rax,rax
	sub	rax,rdx

	mov	QWORD PTR[8+rsp],rdi
	mov	QWORD PTR[16+rsp],rsi
	lea	rsi,QWORD PTR[48+rsi]

	xor	rbp,rdx
	add	rbp,rax

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

	mul	rbp
	mov	QWORD PTR[rdi],rax
	mov	rax,r9
	mov	r9,rdx
	mul	rbp
	add	r9,rax
	mov	rax,r10
	adc	rdx,0
	mov	r10,rdx
	mov	QWORD PTR[8+rdi],r9
	mul	rbp
	add	r10,rax
	mov	rax,r11
	adc	rdx,0
	mov	r11,rdx
	mov	QWORD PTR[16+rdi],r10
	mul	rbp
	add	r11,rax
	mov	rax,r12
	adc	rdx,0
	mov	r12,rdx
	mov	QWORD PTR[24+rdi],r11
	mul	rbp
	add	r12,rax
	mov	rax,r13
	adc	rdx,0
	mov	r13,rdx
	mov	QWORD PTR[32+rdi],r12
	imul	rbp
	add	r13,rax
	adc	rdx,0

	mov	QWORD PTR[40+rdi],r13
	mov	QWORD PTR[48+rdi],rdx
	sar	rdx,63
	mov	QWORD PTR[56+rdi],rdx
	mov	rdx,rcx

	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]
	mov	r14,QWORD PTR[48+rsi]
	mov	r15,QWORD PTR[56+rsi]
	mov	rbx,QWORD PTR[64+rsi]
	mov	rbp,QWORD PTR[72+rsi]
	mov	rcx,QWORD PTR[80+rsi]
	mov	rdi,QWORD PTR[88+rsi]

	mov	rsi,rdx
	sar	rdx,63
	xor	rax,rax
	sub	rax,rdx

	xor	rsi,rdx
	add	rsi,rax

	xor	r8,rdx
	xor	r9,rdx
	xor	r10,rdx
	xor	r11,rdx
	xor	r12,rdx
	xor	r13,rdx
	xor	r14,rdx
	xor	r15,rdx
	xor	rbx,rdx
	xor	rbp,rdx
	xor	rcx,rdx
	xor	rdi,rdx
	add	rax,r8
	adc	r9,0
	adc	r10,0
	adc	r11,0
	adc	r12,0
	adc	r13,0
	adc	r14,0
	adc	r15,0
	adc	rbx,0
	adc	rbp,0
	adc	rcx,0
	adc	rdi,0

	mul	rsi
	mov	r8,rax
	mov	rax,r9
	mov	r9,rdx
	mul	rsi
	add	r9,rax
	mov	rax,r10
	adc	rdx,0
	mov	r10,rdx
	mul	rsi
	add	r10,rax
	mov	rax,r11
	adc	rdx,0
	mov	r11,rdx
	mul	rsi
	add	r11,rax
	mov	rax,r12
	adc	rdx,0
	mov	r12,rdx
	mul	rsi
	add	r12,rax
	mov	rax,r13
	adc	rdx,0
	mov	r13,rdx
	mul	rsi
	add	r13,rax
	mov	rax,r14
	adc	rdx,0
	mov	r14,rdx
	mul	rsi
	add	r14,rax
	mov	rax,r15
	adc	rdx,0
	mov	r15,rdx
	mul	rsi
	add	r15,rax
	mov	rax,rbx
	adc	rdx,0
	mov	rbx,rdx
	mul	rsi
	add	rbx,rax
	mov	rax,rbp
	adc	rdx,0
	mov	rbp,rdx
	mul	rsi
	add	rbp,rax
	mov	rax,rcx
	adc	rdx,0
	mov	rcx,rdx
	mul	rsi
	add	rcx,rax
	mov	rax,rdi
	adc	rdx,0
	mov	rdi,rdx
	mov	rdx,QWORD PTR[8+rsp]
	imul	rax,rsi
	mov	rsi,QWORD PTR[16+rsp]
	add	rax,rdi

	add	r8,QWORD PTR[rdx]
	adc	r9,QWORD PTR[8+rdx]
	adc	r10,QWORD PTR[16+rdx]
	adc	r11,QWORD PTR[24+rdx]
	adc	r12,QWORD PTR[32+rdx]
	adc	r13,QWORD PTR[40+rdx]
	adc	r14,QWORD PTR[48+rdx]
	mov	rdi,QWORD PTR[56+rdx]
	adc	r15,rdi
	adc	rbx,rdi
	adc	rbp,rdi
	adc	rcx,rdi
	adc	rax,rdi

	mov	rdi,rdx

	mov	QWORD PTR[rdx],r8
	mov	QWORD PTR[8+rdx],r9
	mov	QWORD PTR[16+rdx],r10
	mov	QWORD PTR[24+rdx],r11
	mov	QWORD PTR[32+rdx],r12
	mov	QWORD PTR[40+rdx],r13
	mov	QWORD PTR[48+rdx],r14
	mov	QWORD PTR[56+rdx],r15
	mov	QWORD PTR[64+rdx],rbx
	mov	QWORD PTR[72+rdx],rbp
	mov	QWORD PTR[80+rdx],rcx
	mov	QWORD PTR[88+rdx],rax

	DB	0F3h,0C3h		;repret
__smulq_767x63	ENDP

ALIGN	32
__smulq_383x63	PROC PRIVATE
	DB	243,15,30,250
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

	mov	rbp,rdx
	sar	rdx,63
	xor	rax,rax
	sub	rax,rdx

	xor	rbp,rdx
	add	rbp,rax

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

	mul	rbp
	mov	r8,rax
	mov	rax,r9
	mov	r9,rdx
	mul	rbp
	add	r9,rax
	mov	rax,r10
	adc	rdx,0
	mov	r10,rdx
	mul	rbp
	add	r10,rax
	mov	rax,r11
	adc	rdx,0
	mov	r11,rdx
	mul	rbp
	add	r11,rax
	mov	rax,r12
	adc	rdx,0
	mov	r12,rdx
	mul	rbp
	add	r12,rax
	mov	rax,r13
	adc	rdx,0
	mov	r13,rdx
	imul	rax,rbp
	add	r13,rax

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

	mov	rbp,rdx
	sar	rdx,63
	xor	rax,rax
	sub	rax,rdx

	xor	rbp,rdx
	add	rbp,rax

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

	mul	rbp
	mov	r8,rax
	mov	rax,r9
	mov	r9,rdx
	mul	rbp
	add	r9,rax
	mov	rax,r10
	adc	rdx,0
	mov	r10,rdx
	mul	rbp
	add	r10,rax
	mov	rax,r11
	adc	rdx,0
	mov	r11,rdx
	mul	rbp
	add	r11,rax
	mov	rax,r12
	adc	rdx,0
	mov	r12,rdx
	mul	rbp
	add	r12,rax
	mov	rax,r13
	adc	rdx,0
	mov	r13,rdx
	imul	rax,rbp
	add	r13,rax

	lea	rsi,QWORD PTR[((-48))+rsi]

	add	r8,QWORD PTR[rdi]
	adc	r9,QWORD PTR[8+rdi]
	adc	r10,QWORD PTR[16+rdi]
	adc	r11,QWORD PTR[24+rdi]
	adc	r12,QWORD PTR[32+rdi]
	adc	r13,QWORD PTR[40+rdi]

	mov	QWORD PTR[rdi],r8
	mov	QWORD PTR[8+rdi],r9
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11
	mov	QWORD PTR[32+rdi],r12
	mov	QWORD PTR[40+rdi],r13

	DB	0F3h,0C3h		;repret
__smulq_383x63	ENDP

ALIGN	32
__smulq_383_n_shift_by_62	PROC PRIVATE
	DB	243,15,30,250
	mov	rbx,rdx
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

	mov	rbp,rdx
	sar	rdx,63
	xor	rax,rax
	sub	rax,rdx

	xor	rbp,rdx
	add	rbp,rax

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

	mul	rbp
	mov	r8,rax
	mov	rax,r9
	mov	r9,rdx
	mul	rbp
	add	r9,rax
	mov	rax,r10
	adc	rdx,0
	mov	r10,rdx
	mul	rbp
	add	r10,rax
	mov	rax,r11
	adc	rdx,0
	mov	r11,rdx
	mul	rbp
	add	r11,rax
	mov	rax,r12
	adc	rdx,0
	mov	r12,rdx
	mul	rbp
	add	r12,rax
	mov	rax,r13
	adc	rdx,0
	mov	r13,rdx
	imul	rbp
	add	r13,rax
	adc	rdx,0

	lea	rsi,QWORD PTR[48+rsi]
	mov	r14,rdx
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

	mov	rbp,rdx
	sar	rdx,63
	xor	rax,rax
	sub	rax,rdx

	xor	rbp,rdx
	add	rbp,rax

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

	mul	rbp
	mov	r8,rax
	mov	rax,r9
	mov	r9,rdx
	mul	rbp
	add	r9,rax
	mov	rax,r10
	adc	rdx,0
	mov	r10,rdx
	mul	rbp
	add	r10,rax
	mov	rax,r11
	adc	rdx,0
	mov	r11,rdx
	mul	rbp
	add	r11,rax
	mov	rax,r12
	adc	rdx,0
	mov	r12,rdx
	mul	rbp
	add	r12,rax
	mov	rax,r13
	adc	rdx,0
	mov	r13,rdx
	imul	rbp
	add	r13,rax
	adc	rdx,0

	lea	rsi,QWORD PTR[((-48))+rsi]

	add	r8,QWORD PTR[rdi]
	adc	r9,QWORD PTR[8+rdi]
	adc	r10,QWORD PTR[16+rdi]
	adc	r11,QWORD PTR[24+rdi]
	adc	r12,QWORD PTR[32+rdi]
	adc	r13,QWORD PTR[40+rdi]
	adc	r14,rdx
	mov	rdx,rbx

	shrd	r8,r9,62
	shrd	r9,r10,62
	shrd	r10,r11,62
	shrd	r11,r12,62
	shrd	r12,r13,62
	shrd	r13,r14,62

	sar	r14,63
	xor	rbp,rbp
	sub	rbp,r14

	xor	r8,r14
	xor	r9,r14
	xor	r10,r14
	xor	r11,r14
	xor	r12,r14
	xor	r13,r14
	add	r8,rbp
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

	xor	rdx,r14
	xor	rcx,r14
	add	rdx,rbp
	add	rcx,rbp

	DB	0F3h,0C3h		;repret
__smulq_383_n_shift_by_62	ENDP

ALIGN	32
__ab_approximation_62	PROC PRIVATE
	DB	243,15,30,250
	mov	r9,QWORD PTR[40+rsi]
	mov	r11,QWORD PTR[88+rsi]
	mov	rbx,QWORD PTR[32+rsi]
	mov	rbp,QWORD PTR[80+rsi]
	mov	r8,QWORD PTR[24+rsi]
	mov	r10,QWORD PTR[72+rsi]

	mov	rax,r9
	or	rax,r11
	cmovz	r9,rbx
	cmovz	r11,rbp
	cmovz	rbx,r8
	cmovz	rbp,r10
	mov	r8,QWORD PTR[16+rsi]
	mov	r10,QWORD PTR[64+rsi]

	mov	rax,r9
	or	rax,r11
	cmovz	r9,rbx
	cmovz	r11,rbp
	cmovz	rbx,r8
	cmovz	rbp,r10
	mov	r8,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[56+rsi]

	mov	rax,r9
	or	rax,r11
	cmovz	r9,rbx
	cmovz	r11,rbp
	cmovz	rbx,r8
	cmovz	rbp,r10
	mov	r8,QWORD PTR[rsi]
	mov	r10,QWORD PTR[48+rsi]

	mov	rax,r9
	or	rax,r11
	bsr	rcx,rax
	lea	rcx,QWORD PTR[1+rcx]
	cmovz	r9,rbx
	cmovz	r11,rbp
	cmovz	rcx,rax
	neg	rcx


	shld	r9,rbx,cl
	shld	r11,rbp,cl

	jmp	__inner_loop_62

	DB	0F3h,0C3h		;repret
__ab_approximation_62	ENDP

ALIGN	8
	DD	0
__inner_loop_62	PROC PRIVATE
	DB	243,15,30,250
	mov	rdx,1
	xor	rcx,rcx
	xor	r12,r12
	mov	r13,1
	mov	QWORD PTR[8+rsp],rsi

$L$oop_62::
	xor	rax,rax
	xor	rbx,rbx
	test	r8,1
	mov	rbp,r10
	mov	r14,r11
	cmovnz	rax,r10
	cmovnz	rbx,r11
	sub	rbp,r8
	sbb	r14,r9
	mov	r15,r8
	mov	rsi,r9
	sub	r8,rax
	sbb	r9,rbx
	cmovc	r8,rbp
	cmovc	r9,r14
	cmovc	r10,r15
	cmovc	r11,rsi
	mov	rax,rdx
	cmovc	rdx,r12
	cmovc	r12,rax
	mov	rbx,rcx
	cmovc	rcx,r13
	cmovc	r13,rbx
	xor	rax,rax
	xor	rbx,rbx
	shrd	r8,r9,1
	shr	r9,1
	test	r15,1
	cmovnz	rax,r12
	cmovnz	rbx,r13
	add	r12,r12
	add	r13,r13
	sub	rdx,rax
	sub	rcx,rbx
	sub	edi,1
	jnz	$L$oop_62

	mov	rsi,QWORD PTR[8+rsp]
	DB	0F3h,0C3h		;repret
__inner_loop_62	ENDP
.text$	ENDS
.pdata	SEGMENT READONLY ALIGN(4)
ALIGN	4
	DD	imagerel $L$SEH_begin_ct_inverse_mod_383
	DD	imagerel $L$SEH_body_ct_inverse_mod_383
	DD	imagerel $L$SEH_info_ct_inverse_mod_383_prologue

	DD	imagerel $L$SEH_body_ct_inverse_mod_383
	DD	imagerel $L$SEH_epilogue_ct_inverse_mod_383
	DD	imagerel $L$SEH_info_ct_inverse_mod_383_body

	DD	imagerel $L$SEH_epilogue_ct_inverse_mod_383
	DD	imagerel $L$SEH_end_ct_inverse_mod_383
	DD	imagerel $L$SEH_info_ct_inverse_mod_383_epilogue

.pdata	ENDS
.xdata	SEGMENT READONLY ALIGN(8)
ALIGN	8
$L$SEH_info_ct_inverse_mod_383_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,003h
DB	0,0
$L$SEH_info_ct_inverse_mod_383_body::
DB	1,0,18,0
DB	000h,0f4h,08bh,000h
DB	000h,0e4h,08ch,000h
DB	000h,0d4h,08dh,000h
DB	000h,0c4h,08eh,000h
DB	000h,034h,08fh,000h
DB	000h,054h,090h,000h
DB	000h,074h,092h,000h
DB	000h,064h,093h,000h
DB	000h,001h,091h,000h
$L$SEH_info_ct_inverse_mod_383_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h


.xdata	ENDS
END
