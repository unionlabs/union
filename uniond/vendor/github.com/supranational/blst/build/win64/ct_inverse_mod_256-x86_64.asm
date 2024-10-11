OPTION	DOTNAME
.text$	SEGMENT ALIGN(256) 'CODE'

PUBLIC	ct_inverse_mod_256


ALIGN	32
ct_inverse_mod_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_ct_inverse_mod_256::


	push	rbp

	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,1072

$L$SEH_body_ct_inverse_mod_256::


	lea	rax,QWORD PTR[((48+511))+rsp]
	and	rax,-512
	mov	QWORD PTR[32+rsp],rdi
	mov	QWORD PTR[40+rsp],rcx

ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]

	mov	r12,QWORD PTR[rdx]
	mov	r13,QWORD PTR[8+rdx]
	mov	r14,QWORD PTR[16+rdx]
	mov	r15,QWORD PTR[24+rdx]

	mov	QWORD PTR[rax],r8
	mov	QWORD PTR[8+rax],r9
	mov	QWORD PTR[16+rax],r10
	mov	QWORD PTR[24+rax],r11

	mov	QWORD PTR[32+rax],r12
	mov	QWORD PTR[40+rax],r13
	mov	QWORD PTR[48+rax],r14
	mov	QWORD PTR[56+rax],r15
	mov	rsi,rax


	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31


	mov	QWORD PTR[64+rdi],rdx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31


	mov	QWORD PTR[72+rdi],rdx


	xor	rsi,256
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31



	mov	r8,QWORD PTR[64+rsi]
	mov	r12,QWORD PTR[104+rsi]
	mov	r9,r8
	imul	r8,QWORD PTR[rsp]
	mov	r13,r12
	imul	r12,QWORD PTR[8+rsp]
	add	r8,r12
	mov	QWORD PTR[32+rdi],r8
	sar	r8,63
	mov	QWORD PTR[40+rdi],r8
	mov	QWORD PTR[48+rdi],r8
	mov	QWORD PTR[56+rdi],r8
	mov	QWORD PTR[64+rdi],r8
	lea	rsi,QWORD PTR[64+rsi]

	imul	r9,rdx
	imul	r13,rcx
	add	r9,r13
	mov	QWORD PTR[72+rdi],r9
	sar	r9,63
	mov	QWORD PTR[80+rdi],r9
	mov	QWORD PTR[88+rdi],r9
	mov	QWORD PTR[96+rdi],r9
	mov	QWORD PTR[104+rdi],r9
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_256x63
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_256x63
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_256x63
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_256x63
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_256x63
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_256x63
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_256x63
	sar	rbp,63
	mov	QWORD PTR[40+rdi],rbp
	mov	QWORD PTR[48+rdi],rbp
	mov	QWORD PTR[56+rdi],rbp
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_512x63
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_512x63
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_512x63
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_512x63
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_512x63
	xor	rsi,256+8*8
	mov	edx,31
	call	__ab_approximation_31_256


	mov	QWORD PTR[16+rsp],r12
	mov	QWORD PTR[24+rsp],r13

	mov	rdi,256
	xor	rdi,rsi
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[rsp],rdx
	mov	QWORD PTR[8+rsp],rcx

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256_n_shift_by_31
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[24+rsp],rcx

	mov	rdx,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[64+rsi]
	lea	rdi,QWORD PTR[32+rdi]
	call	__smulq_256x63

	mov	rdx,QWORD PTR[16+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	lea	rdi,QWORD PTR[40+rdi]
	call	__smulq_512x63

	xor	rsi,256+8*8
	mov	edx,47

	mov	r8,QWORD PTR[rsi]

	mov	r10,QWORD PTR[32+rsi]

	call	__inner_loop_62_256







	lea	rsi,QWORD PTR[64+rsi]





	mov	rdx,r12
	mov	rcx,r13
	mov	rdi,QWORD PTR[32+rsp]
	call	__smulq_512x63
	adc	rdx,rbp

	mov	rsi,QWORD PTR[40+rsp]
	mov	rax,rdx
	sar	rdx,63

	mov	r8,rdx
	mov	r9,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	and	r8,QWORD PTR[rsi]
	mov	r10,rdx
	and	r9,QWORD PTR[8+rsi]
	and	r10,QWORD PTR[16+rsi]
	and	rdx,QWORD PTR[24+rsi]

	add	r12,r8
	adc	r13,r9
	adc	r14,r10
	adc	r15,rdx
	adc	rax,0

	mov	rdx,rax
	neg	rax
	or	rdx,rax
	sar	rax,63

	mov	r8,rdx
	mov	r9,rdx
	and	r8,QWORD PTR[rsi]
	mov	r10,rdx
	and	r9,QWORD PTR[8+rsi]
	and	r10,QWORD PTR[16+rsi]
	and	rdx,QWORD PTR[24+rsi]

	xor	r8,rax
	xor	rcx,rcx
	xor	r9,rax
	sub	rcx,rax
	xor	r10,rax
	xor	rdx,rax
	add	r8,rcx
	adc	r9,0
	adc	r10,0
	adc	rdx,0

	add	r12,r8
	adc	r13,r9
	adc	r14,r10
	adc	r15,rdx

	mov	QWORD PTR[32+rdi],r12
	mov	QWORD PTR[40+rdi],r13
	mov	QWORD PTR[48+rdi],r14
	mov	QWORD PTR[56+rdi],r15

	lea	r8,QWORD PTR[1072+rsp]
	mov	r15,QWORD PTR[r8]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_ct_inverse_mod_256::
	mov	rdi,QWORD PTR[8+rsp]	;WIN64 epilogue
	mov	rsi,QWORD PTR[16+rsp]

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif

$L$SEH_end_ct_inverse_mod_256::
ct_inverse_mod_256	ENDP

ALIGN	32
__smulq_512x63	PROC PRIVATE
	DB	243,15,30,250

	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	rbp,QWORD PTR[32+rsi]

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
	xor	rbp,rdx
	add	rax,r8
	adc	r9,0
	adc	r10,0
	adc	r11,0
	adc	rbp,0

	mul	rbx
	mov	QWORD PTR[rdi],rax
	mov	rax,r9
	mov	r9,rdx
	mul	rbx
	add	r9,rax
	mov	rax,r10
	adc	rdx,0
	mov	QWORD PTR[8+rdi],r9
	mov	r10,rdx
	mul	rbx
	add	r10,rax
	mov	rax,r11
	adc	rdx,0
	mov	QWORD PTR[16+rdi],r10
	mov	r11,rdx
	and	rbp,rbx
	neg	rbp
	mul	rbx
	add	r11,rax
	adc	rbp,rdx
	mov	QWORD PTR[24+rdi],r11

	mov	r8,QWORD PTR[40+rsi]
	mov	r9,QWORD PTR[48+rsi]
	mov	r10,QWORD PTR[56+rsi]
	mov	r11,QWORD PTR[64+rsi]
	mov	r12,QWORD PTR[72+rsi]
	mov	r13,QWORD PTR[80+rsi]
	mov	r14,QWORD PTR[88+rsi]
	mov	r15,QWORD PTR[96+rsi]

	mov	rdx,rcx
	sar	rdx,63
	xor	rax,rax
	sub	rax,rdx

	xor	rcx,rdx
	add	rcx,rax

	xor	r8,rdx
	xor	r9,rdx
	xor	r10,rdx
	xor	r11,rdx
	xor	r12,rdx
	xor	r13,rdx
	xor	r14,rdx
	xor	r15,rdx
	add	rax,r8
	adc	r9,0
	adc	r10,0
	adc	r11,0
	adc	r12,0
	adc	r13,0
	adc	r14,0
	adc	r15,0

	mul	rcx
	mov	r8,rax
	mov	rax,r9
	mov	r9,rdx
	mul	rcx
	add	r9,rax
	mov	rax,r10
	adc	rdx,0
	mov	r10,rdx
	mul	rcx
	add	r10,rax
	mov	rax,r11
	adc	rdx,0
	mov	r11,rdx
	mul	rcx
	add	r11,rax
	mov	rax,r12
	adc	rdx,0
	mov	r12,rdx
	mul	rcx
	add	r12,rax
	mov	rax,r13
	adc	rdx,0
	mov	r13,rdx
	mul	rcx
	add	r13,rax
	mov	rax,r14
	adc	rdx,0
	mov	r14,rdx
	mul	rcx
	add	r14,rax
	mov	rax,r15
	adc	rdx,0
	mov	r15,rdx
	imul	rcx
	add	r15,rax
	adc	rdx,0

	mov	rbx,rbp
	sar	rbp,63

	add	r8,QWORD PTR[rdi]
	adc	r9,QWORD PTR[8+rdi]
	adc	r10,QWORD PTR[16+rdi]
	adc	r11,QWORD PTR[24+rdi]
	adc	r12,rbx
	adc	r13,rbp
	adc	r14,rbp
	adc	r15,rbp

	mov	QWORD PTR[rdi],r8
	mov	QWORD PTR[8+rdi],r9
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11
	mov	QWORD PTR[32+rdi],r12
	mov	QWORD PTR[40+rdi],r13
	mov	QWORD PTR[48+rdi],r14
	mov	QWORD PTR[56+rdi],r15

	
ifdef	__SGX_LVI_HARDENING__
	pop	r8
	lfence
	jmp	r8
	ud2
else
	DB	0F3h,0C3h
endif
__smulq_512x63	ENDP


ALIGN	32
__smulq_256x63	PROC PRIVATE
	DB	243,15,30,250

	mov	r8,QWORD PTR[((0+0))+rsi]
	mov	r9,QWORD PTR[((0+8))+rsi]
	mov	r10,QWORD PTR[((0+16))+rsi]
	mov	r11,QWORD PTR[((0+24))+rsi]
	mov	rbp,QWORD PTR[((0+32))+rsi]

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
	xor	rbp,rdx
	add	rax,r8
	adc	r9,0
	adc	r10,0
	adc	r11,0
	adc	rbp,0

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
	and	rbp,rbx
	neg	rbp
	mul	rbx
	add	r11,rax
	adc	rbp,rdx
	mov	rdx,rcx
	mov	r12,QWORD PTR[((40+0))+rsi]
	mov	r13,QWORD PTR[((40+8))+rsi]
	mov	r14,QWORD PTR[((40+16))+rsi]
	mov	r15,QWORD PTR[((40+24))+rsi]
	mov	rcx,QWORD PTR[((40+32))+rsi]

	mov	rbx,rdx
	sar	rdx,63
	xor	rax,rax
	sub	rax,rdx

	xor	rbx,rdx
	add	rbx,rax

	xor	r12,rdx
	xor	r13,rdx
	xor	r14,rdx
	xor	r15,rdx
	xor	rcx,rdx
	add	rax,r12
	adc	r13,0
	adc	r14,0
	adc	r15,0
	adc	rcx,0

	mul	rbx
	mov	r12,rax
	mov	rax,r13
	mov	r13,rdx
	mul	rbx
	add	r13,rax
	mov	rax,r14
	adc	rdx,0
	mov	r14,rdx
	mul	rbx
	add	r14,rax
	mov	rax,r15
	adc	rdx,0
	mov	r15,rdx
	and	rcx,rbx
	neg	rcx
	mul	rbx
	add	r15,rax
	adc	rcx,rdx
	add	r8,r12
	adc	r9,r13
	adc	r10,r14
	adc	r11,r15
	adc	rbp,rcx

	mov	QWORD PTR[rdi],r8
	mov	QWORD PTR[8+rdi],r9
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11
	mov	QWORD PTR[32+rdi],rbp

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__smulq_256x63	ENDP

ALIGN	32
__smulq_256_n_shift_by_31	PROC PRIVATE
	DB	243,15,30,250

	mov	QWORD PTR[rdi],rdx
	mov	QWORD PTR[8+rdi],rcx
	mov	rbp,rdx
	mov	r8,QWORD PTR[((0+0))+rsi]
	mov	r9,QWORD PTR[((0+8))+rsi]
	mov	r10,QWORD PTR[((0+16))+rsi]
	mov	r11,QWORD PTR[((0+24))+rsi]

	mov	rbx,rbp
	sar	rbp,63
	xor	rax,rax
	sub	rax,rbp

	xor	rbx,rbp
	add	rbx,rax

	xor	r8,rbp
	xor	r9,rbp
	xor	r10,rbp
	xor	r11,rbp
	add	rax,r8
	adc	r9,0
	adc	r10,0
	adc	r11,0

	mul	rbx
	mov	r8,rax
	mov	rax,r9
	and	rbp,rbx
	neg	rbp
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
	adc	rbp,rdx
	mov	r12,QWORD PTR[((32+0))+rsi]
	mov	r13,QWORD PTR[((32+8))+rsi]
	mov	r14,QWORD PTR[((32+16))+rsi]
	mov	r15,QWORD PTR[((32+24))+rsi]

	mov	rbx,rcx
	sar	rcx,63
	xor	rax,rax
	sub	rax,rcx

	xor	rbx,rcx
	add	rbx,rax

	xor	r12,rcx
	xor	r13,rcx
	xor	r14,rcx
	xor	r15,rcx
	add	rax,r12
	adc	r13,0
	adc	r14,0
	adc	r15,0

	mul	rbx
	mov	r12,rax
	mov	rax,r13
	and	rcx,rbx
	neg	rcx
	mov	r13,rdx
	mul	rbx
	add	r13,rax
	mov	rax,r14
	adc	rdx,0
	mov	r14,rdx
	mul	rbx
	add	r14,rax
	mov	rax,r15
	adc	rdx,0
	mov	r15,rdx
	mul	rbx
	add	r15,rax
	adc	rcx,rdx
	add	r8,r12
	adc	r9,r13
	adc	r10,r14
	adc	r11,r15
	adc	rbp,rcx

	mov	rdx,QWORD PTR[rdi]
	mov	rcx,QWORD PTR[8+rdi]

	shrd	r8,r9,31
	shrd	r9,r10,31
	shrd	r10,r11,31
	shrd	r11,rbp,31

	sar	rbp,63
	xor	rax,rax
	sub	rax,rbp

	xor	r8,rbp
	xor	r9,rbp
	xor	r10,rbp
	xor	r11,rbp
	add	r8,rax
	adc	r9,0
	adc	r10,0
	adc	r11,0

	mov	QWORD PTR[rdi],r8
	mov	QWORD PTR[8+rdi],r9
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11

	xor	rdx,rbp
	xor	rcx,rbp
	add	rdx,rax
	add	rcx,rax

	
ifdef	__SGX_LVI_HARDENING__
	pop	r8
	lfence
	jmp	r8
	ud2
else
	DB	0F3h,0C3h
endif
__smulq_256_n_shift_by_31	ENDP

ALIGN	32
__ab_approximation_31_256	PROC PRIVATE
	DB	243,15,30,250

	mov	r9,QWORD PTR[24+rsi]
	mov	r11,QWORD PTR[56+rsi]
	mov	rbx,QWORD PTR[16+rsi]
	mov	rbp,QWORD PTR[48+rsi]
	mov	r8,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[40+rsi]

	mov	rax,r9
	or	rax,r11
	cmovz	r9,rbx
	cmovz	r11,rbp
	cmovz	rbx,r8
	mov	r8,QWORD PTR[rsi]
	cmovz	rbp,r10
	mov	r10,QWORD PTR[32+rsi]

	mov	rax,r9
	or	rax,r11
	cmovz	r9,rbx
	cmovz	r11,rbp
	cmovz	rbx,r8
	cmovz	rbp,r10

	mov	rax,r9
	or	rax,r11
	bsr	rcx,rax
	lea	rcx,QWORD PTR[1+rcx]
	cmovz	r9,r8
	cmovz	r11,r10
	cmovz	rcx,rax
	neg	rcx


	shld	r9,rbx,cl
	shld	r11,rbp,cl

	mov	eax,07FFFFFFFh
	and	r8,rax
	and	r10,rax
	not	rax
	and	r9,rax
	and	r11,rax
	or	r8,r9
	or	r10,r11

	jmp	__inner_loop_31_256

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__ab_approximation_31_256	ENDP

ALIGN	32
__inner_loop_31_256	PROC PRIVATE
	DB	243,15,30,250

	mov	rcx,07FFFFFFF80000000h
	mov	r13,0800000007FFFFFFFh
	mov	r15,07FFFFFFF7FFFFFFFh

$L$oop_31_256::
	cmp	r8,r10
	mov	rax,r8
	mov	rbx,r10
	mov	rbp,rcx
	mov	r14,r13
	cmovb	r8,r10
	cmovb	r10,rax
	cmovb	rcx,r13
	cmovb	r13,rbp

	sub	r8,r10
	sub	rcx,r13
	add	rcx,r15

	test	rax,1
	cmovz	r8,rax
	cmovz	r10,rbx
	cmovz	rcx,rbp
	cmovz	r13,r14

	shr	r8,1
	add	r13,r13
	sub	r13,r15
	sub	edx,1
	jnz	$L$oop_31_256

	shr	r15,32
	mov	edx,ecx
	mov	r12d,r13d
	shr	rcx,32
	shr	r13,32
	sub	rdx,r15
	sub	rcx,r15
	sub	r12,r15
	sub	r13,r15

	
ifdef	__SGX_LVI_HARDENING__
	pop	r8
	lfence
	jmp	r8
	ud2
else
	DB	0F3h,0C3h
endif
__inner_loop_31_256	ENDP


ALIGN	32
__inner_loop_62_256	PROC PRIVATE
	DB	243,15,30,250

	mov	r15d,edx
	mov	rdx,1
	xor	rcx,rcx
	xor	r12,r12
	mov	r13,rdx
	mov	r14,rdx

$L$oop_62_256::
	xor	rax,rax
	test	r8,r14
	mov	rbx,r10
	cmovnz	rax,r10
	sub	rbx,r8
	mov	rbp,r8
	sub	r8,rax
	cmovc	r8,rbx
	cmovc	r10,rbp
	mov	rax,rdx
	cmovc	rdx,r12
	cmovc	r12,rax
	mov	rbx,rcx
	cmovc	rcx,r13
	cmovc	r13,rbx
	xor	rax,rax
	xor	rbx,rbx
	shr	r8,1
	test	rbp,r14
	cmovnz	rax,r12
	cmovnz	rbx,r13
	add	r12,r12
	add	r13,r13
	sub	rdx,rax
	sub	rcx,rbx
	sub	r15d,1
	jnz	$L$oop_62_256

	
ifdef	__SGX_LVI_HARDENING__
	pop	r8
	lfence
	jmp	r8
	ud2
else
	DB	0F3h,0C3h
endif
__inner_loop_62_256	ENDP
.text$	ENDS
.pdata	SEGMENT READONLY ALIGN(4)
ALIGN	4
	DD	imagerel $L$SEH_begin_ct_inverse_mod_256
	DD	imagerel $L$SEH_body_ct_inverse_mod_256
	DD	imagerel $L$SEH_info_ct_inverse_mod_256_prologue

	DD	imagerel $L$SEH_body_ct_inverse_mod_256
	DD	imagerel $L$SEH_epilogue_ct_inverse_mod_256
	DD	imagerel $L$SEH_info_ct_inverse_mod_256_body

	DD	imagerel $L$SEH_epilogue_ct_inverse_mod_256
	DD	imagerel $L$SEH_end_ct_inverse_mod_256
	DD	imagerel $L$SEH_info_ct_inverse_mod_256_epilogue

.pdata	ENDS
.xdata	SEGMENT READONLY ALIGN(8)
ALIGN	8
$L$SEH_info_ct_inverse_mod_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_ct_inverse_mod_256_body::
DB	1,0,18,0
DB	000h,0f4h,086h,000h
DB	000h,0e4h,087h,000h
DB	000h,0d4h,088h,000h
DB	000h,0c4h,089h,000h
DB	000h,034h,08ah,000h
DB	000h,054h,08bh,000h
DB	000h,074h,08dh,000h
DB	000h,064h,08eh,000h
DB	000h,001h,08ch,000h
DB	000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_ct_inverse_mod_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h


.xdata	ENDS
END
