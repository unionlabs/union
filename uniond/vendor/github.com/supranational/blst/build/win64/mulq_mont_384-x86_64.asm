OPTION	DOTNAME
EXTERN	mul_mont_384x$1:NEAR
EXTERN	sqr_mont_384x$1:NEAR
EXTERN	mul_382x$1:NEAR
EXTERN	sqr_382x$1:NEAR
EXTERN	mul_384$1:NEAR
EXTERN	sqr_384$1:NEAR
EXTERN	redc_mont_384$1:NEAR
EXTERN	from_mont_384$1:NEAR
EXTERN	sgn0_pty_mont_384$1:NEAR
EXTERN	sgn0_pty_mont_384x$1:NEAR
EXTERN	mul_mont_384$1:NEAR
EXTERN	sqr_mont_384$1:NEAR
EXTERN	sqr_n_mul_mont_384$1:NEAR
EXTERN	sqr_n_mul_mont_383$1:NEAR
EXTERN	sqr_mont_382x$1:NEAR
_DATA	SEGMENT
COMM	__blst_platform_cap:DWORD:1
_DATA	ENDS
.text$	SEGMENT ALIGN(256) 'CODE'








ALIGN	32
__subq_mod_384x384	PROC PRIVATE
	DB	243,15,30,250

	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]
	mov	r14,QWORD PTR[48+rsi]

	sub	r8,QWORD PTR[rdx]
	mov	r15,QWORD PTR[56+rsi]
	sbb	r9,QWORD PTR[8+rdx]
	mov	rax,QWORD PTR[64+rsi]
	sbb	r10,QWORD PTR[16+rdx]
	mov	rbx,QWORD PTR[72+rsi]
	sbb	r11,QWORD PTR[24+rdx]
	mov	rbp,QWORD PTR[80+rsi]
	sbb	r12,QWORD PTR[32+rdx]
	mov	rsi,QWORD PTR[88+rsi]
	sbb	r13,QWORD PTR[40+rdx]
	mov	QWORD PTR[rdi],r8
	sbb	r14,QWORD PTR[48+rdx]
	mov	r8,QWORD PTR[rcx]
	mov	QWORD PTR[8+rdi],r9
	sbb	r15,QWORD PTR[56+rdx]
	mov	r9,QWORD PTR[8+rcx]
	mov	QWORD PTR[16+rdi],r10
	sbb	rax,QWORD PTR[64+rdx]
	mov	r10,QWORD PTR[16+rcx]
	mov	QWORD PTR[24+rdi],r11
	sbb	rbx,QWORD PTR[72+rdx]
	mov	r11,QWORD PTR[24+rcx]
	mov	QWORD PTR[32+rdi],r12
	sbb	rbp,QWORD PTR[80+rdx]
	mov	r12,QWORD PTR[32+rcx]
	mov	QWORD PTR[40+rdi],r13
	sbb	rsi,QWORD PTR[88+rdx]
	mov	r13,QWORD PTR[40+rcx]
	sbb	rdx,rdx

	and	r8,rdx
	and	r9,rdx
	and	r10,rdx
	and	r11,rdx
	and	r12,rdx
	and	r13,rdx

	add	r14,r8
	adc	r15,r9
	mov	QWORD PTR[48+rdi],r14
	adc	rax,r10
	mov	QWORD PTR[56+rdi],r15
	adc	rbx,r11
	mov	QWORD PTR[64+rdi],rax
	adc	rbp,r12
	mov	QWORD PTR[72+rdi],rbx
	adc	rsi,r13
	mov	QWORD PTR[80+rdi],rbp
	mov	QWORD PTR[88+rdi],rsi

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__subq_mod_384x384	ENDP


ALIGN	32
__addq_mod_384	PROC PRIVATE
	DB	243,15,30,250

	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

	add	r8,QWORD PTR[rdx]
	adc	r9,QWORD PTR[8+rdx]
	adc	r10,QWORD PTR[16+rdx]
	mov	r14,r8
	adc	r11,QWORD PTR[24+rdx]
	mov	r15,r9
	adc	r12,QWORD PTR[32+rdx]
	mov	rax,r10
	adc	r13,QWORD PTR[40+rdx]
	mov	rbx,r11
	sbb	rdx,rdx

	sub	r8,QWORD PTR[rcx]
	sbb	r9,QWORD PTR[8+rcx]
	mov	rbp,r12
	sbb	r10,QWORD PTR[16+rcx]
	sbb	r11,QWORD PTR[24+rcx]
	sbb	r12,QWORD PTR[32+rcx]
	mov	rsi,r13
	sbb	r13,QWORD PTR[40+rcx]
	sbb	rdx,0

	cmovc	r8,r14
	cmovc	r9,r15
	cmovc	r10,rax
	mov	QWORD PTR[rdi],r8
	cmovc	r11,rbx
	mov	QWORD PTR[8+rdi],r9
	cmovc	r12,rbp
	mov	QWORD PTR[16+rdi],r10
	cmovc	r13,rsi
	mov	QWORD PTR[24+rdi],r11
	mov	QWORD PTR[32+rdi],r12
	mov	QWORD PTR[40+rdi],r13

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__addq_mod_384	ENDP


ALIGN	32
__subq_mod_384	PROC PRIVATE
	DB	243,15,30,250

	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

__subq_mod_384_a_is_loaded::
	sub	r8,QWORD PTR[rdx]
	mov	r14,QWORD PTR[rcx]
	sbb	r9,QWORD PTR[8+rdx]
	mov	r15,QWORD PTR[8+rcx]
	sbb	r10,QWORD PTR[16+rdx]
	mov	rax,QWORD PTR[16+rcx]
	sbb	r11,QWORD PTR[24+rdx]
	mov	rbx,QWORD PTR[24+rcx]
	sbb	r12,QWORD PTR[32+rdx]
	mov	rbp,QWORD PTR[32+rcx]
	sbb	r13,QWORD PTR[40+rdx]
	mov	rsi,QWORD PTR[40+rcx]
	sbb	rdx,rdx

	and	r14,rdx
	and	r15,rdx
	and	rax,rdx
	and	rbx,rdx
	and	rbp,rdx
	and	rsi,rdx

	add	r8,r14
	adc	r9,r15
	mov	QWORD PTR[rdi],r8
	adc	r10,rax
	mov	QWORD PTR[8+rdi],r9
	adc	r11,rbx
	mov	QWORD PTR[16+rdi],r10
	adc	r12,rbp
	mov	QWORD PTR[24+rdi],r11
	adc	r13,rsi
	mov	QWORD PTR[32+rdi],r12
	mov	QWORD PTR[40+rdi],r13

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__subq_mod_384	ENDP
PUBLIC	mul_mont_384x


ALIGN	32
mul_mont_384x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_mul_mont_384x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	mov	r8,QWORD PTR[40+rsp]
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	mul_mont_384x$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,328

$L$SEH_body_mul_mont_384x::


	mov	rbx,rdx
	mov	QWORD PTR[32+rsp],rdi
	mov	QWORD PTR[24+rsp],rsi
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[8+rsp],rcx
	mov	QWORD PTR[rsp],r8




	lea	rdi,QWORD PTR[40+rsp]
	call	__mulq_384


	lea	rbx,QWORD PTR[48+rbx]
	lea	rsi,QWORD PTR[48+rsi]
	lea	rdi,QWORD PTR[((40+96))+rsp]
	call	__mulq_384


	mov	rcx,QWORD PTR[8+rsp]
	lea	rdx,QWORD PTR[((-48))+rsi]
	lea	rdi,QWORD PTR[((40+192+48))+rsp]
	call	__addq_mod_384

	mov	rsi,QWORD PTR[16+rsp]
	lea	rdx,QWORD PTR[48+rsi]
	lea	rdi,QWORD PTR[((-48))+rdi]
	call	__addq_mod_384

	lea	rbx,QWORD PTR[rdi]
	lea	rsi,QWORD PTR[48+rdi]
	call	__mulq_384


	lea	rsi,QWORD PTR[rdi]
	lea	rdx,QWORD PTR[40+rsp]
	mov	rcx,QWORD PTR[8+rsp]
	call	__subq_mod_384x384

	lea	rsi,QWORD PTR[rdi]
	lea	rdx,QWORD PTR[((-96))+rdi]
	call	__subq_mod_384x384


	lea	rsi,QWORD PTR[40+rsp]
	lea	rdx,QWORD PTR[((40+96))+rsp]
	lea	rdi,QWORD PTR[40+rsp]
	call	__subq_mod_384x384

	mov	rbx,rcx


	lea	rsi,QWORD PTR[40+rsp]
	mov	rcx,QWORD PTR[rsp]
	mov	rdi,QWORD PTR[32+rsp]
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384


	lea	rsi,QWORD PTR[((40+192))+rsp]
	mov	rcx,QWORD PTR[rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

	lea	r8,QWORD PTR[328+rsp]
	mov	r15,QWORD PTR[r8]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_mul_mont_384x::
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

$L$SEH_end_mul_mont_384x::
mul_mont_384x	ENDP
PUBLIC	sqr_mont_384x


ALIGN	32
sqr_mont_384x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqr_mont_384x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	sqr_mont_384x$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,136

$L$SEH_body_sqr_mont_384x::


	mov	QWORD PTR[rsp],rcx
	mov	rcx,rdx
	mov	QWORD PTR[8+rsp],rdi
	mov	QWORD PTR[16+rsp],rsi


	lea	rdx,QWORD PTR[48+rsi]
	lea	rdi,QWORD PTR[32+rsp]
	call	__addq_mod_384


	mov	rsi,QWORD PTR[16+rsp]
	lea	rdx,QWORD PTR[48+rsi]
	lea	rdi,QWORD PTR[((32+48))+rsp]
	call	__subq_mod_384


	mov	rsi,QWORD PTR[16+rsp]
	lea	rbx,QWORD PTR[48+rsi]

	mov	rax,QWORD PTR[48+rsi]
	mov	r14,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	r12,QWORD PTR[16+rsi]
	mov	r13,QWORD PTR[24+rsi]

	call	__mulq_mont_384
	add	r14,r14
	adc	r15,r15
	adc	r8,r8
	mov	r12,r14
	adc	r9,r9
	mov	r13,r15
	adc	r10,r10
	mov	rax,r8
	adc	r11,r11
	mov	rbx,r9
	sbb	rdx,rdx

	sub	r14,QWORD PTR[rcx]
	sbb	r15,QWORD PTR[8+rcx]
	mov	rbp,r10
	sbb	r8,QWORD PTR[16+rcx]
	sbb	r9,QWORD PTR[24+rcx]
	sbb	r10,QWORD PTR[32+rcx]
	mov	rsi,r11
	sbb	r11,QWORD PTR[40+rcx]
	sbb	rdx,0

	cmovc	r14,r12
	cmovc	r15,r13
	cmovc	r8,rax
	mov	QWORD PTR[48+rdi],r14
	cmovc	r9,rbx
	mov	QWORD PTR[56+rdi],r15
	cmovc	r10,rbp
	mov	QWORD PTR[64+rdi],r8
	cmovc	r11,rsi
	mov	QWORD PTR[72+rdi],r9
	mov	QWORD PTR[80+rdi],r10
	mov	QWORD PTR[88+rdi],r11

	lea	rsi,QWORD PTR[32+rsp]
	lea	rbx,QWORD PTR[((32+48))+rsp]

	mov	rax,QWORD PTR[((32+48))+rsp]
	mov	r14,QWORD PTR[((32+0))+rsp]
	mov	r15,QWORD PTR[((32+8))+rsp]
	mov	r12,QWORD PTR[((32+16))+rsp]
	mov	r13,QWORD PTR[((32+24))+rsp]

	call	__mulq_mont_384

	lea	r8,QWORD PTR[136+rsp]
	mov	r15,QWORD PTR[r8]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_sqr_mont_384x::
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

$L$SEH_end_sqr_mont_384x::
sqr_mont_384x	ENDP

PUBLIC	mul_382x


ALIGN	32
mul_382x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_mul_382x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	mul_382x$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,136

$L$SEH_body_mul_382x::


	lea	rdi,QWORD PTR[96+rdi]
	mov	QWORD PTR[rsp],rsi
	mov	QWORD PTR[8+rsp],rdx
	mov	QWORD PTR[16+rsp],rdi
	mov	QWORD PTR[24+rsp],rcx


	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

	add	r8,QWORD PTR[48+rsi]
	adc	r9,QWORD PTR[56+rsi]
	adc	r10,QWORD PTR[64+rsi]
	adc	r11,QWORD PTR[72+rsi]
	adc	r12,QWORD PTR[80+rsi]
	adc	r13,QWORD PTR[88+rsi]

	mov	QWORD PTR[((32+0))+rsp],r8
	mov	QWORD PTR[((32+8))+rsp],r9
	mov	QWORD PTR[((32+16))+rsp],r10
	mov	QWORD PTR[((32+24))+rsp],r11
	mov	QWORD PTR[((32+32))+rsp],r12
	mov	QWORD PTR[((32+40))+rsp],r13


	mov	r8,QWORD PTR[rdx]
	mov	r9,QWORD PTR[8+rdx]
	mov	r10,QWORD PTR[16+rdx]
	mov	r11,QWORD PTR[24+rdx]
	mov	r12,QWORD PTR[32+rdx]
	mov	r13,QWORD PTR[40+rdx]

	add	r8,QWORD PTR[48+rdx]
	adc	r9,QWORD PTR[56+rdx]
	adc	r10,QWORD PTR[64+rdx]
	adc	r11,QWORD PTR[72+rdx]
	adc	r12,QWORD PTR[80+rdx]
	adc	r13,QWORD PTR[88+rdx]

	mov	QWORD PTR[((32+48))+rsp],r8
	mov	QWORD PTR[((32+56))+rsp],r9
	mov	QWORD PTR[((32+64))+rsp],r10
	mov	QWORD PTR[((32+72))+rsp],r11
	mov	QWORD PTR[((32+80))+rsp],r12
	mov	QWORD PTR[((32+88))+rsp],r13


	lea	rsi,QWORD PTR[((32+0))+rsp]
	lea	rbx,QWORD PTR[((32+48))+rsp]
	call	__mulq_384


	mov	rsi,QWORD PTR[rsp]
	mov	rbx,QWORD PTR[8+rsp]
	lea	rdi,QWORD PTR[((-96))+rdi]
	call	__mulq_384


	lea	rsi,QWORD PTR[48+rsi]
	lea	rbx,QWORD PTR[48+rbx]
	lea	rdi,QWORD PTR[32+rsp]
	call	__mulq_384


	mov	rsi,QWORD PTR[16+rsp]
	lea	rdx,QWORD PTR[32+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	mov	rdi,rsi
	call	__subq_mod_384x384


	lea	rsi,QWORD PTR[rdi]
	lea	rdx,QWORD PTR[((-96))+rdi]
	call	__subq_mod_384x384


	lea	rsi,QWORD PTR[((-96))+rdi]
	lea	rdx,QWORD PTR[32+rsp]
	lea	rdi,QWORD PTR[((-96))+rdi]
	call	__subq_mod_384x384

	lea	r8,QWORD PTR[136+rsp]
	mov	r15,QWORD PTR[r8]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_mul_382x::
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

$L$SEH_end_mul_382x::
mul_382x	ENDP
PUBLIC	sqr_382x


ALIGN	32
sqr_382x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqr_382x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	sqr_382x$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	push	rsi

$L$SEH_body_sqr_382x::


	mov	rcx,rdx


	mov	r14,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	rax,QWORD PTR[16+rsi]
	mov	rbx,QWORD PTR[24+rsi]
	mov	rbp,QWORD PTR[32+rsi]
	mov	rdx,QWORD PTR[40+rsi]

	mov	r8,r14
	add	r14,QWORD PTR[48+rsi]
	mov	r9,r15
	adc	r15,QWORD PTR[56+rsi]
	mov	r10,rax
	adc	rax,QWORD PTR[64+rsi]
	mov	r11,rbx
	adc	rbx,QWORD PTR[72+rsi]
	mov	r12,rbp
	adc	rbp,QWORD PTR[80+rsi]
	mov	r13,rdx
	adc	rdx,QWORD PTR[88+rsi]

	mov	QWORD PTR[rdi],r14
	mov	QWORD PTR[8+rdi],r15
	mov	QWORD PTR[16+rdi],rax
	mov	QWORD PTR[24+rdi],rbx
	mov	QWORD PTR[32+rdi],rbp
	mov	QWORD PTR[40+rdi],rdx


	lea	rdx,QWORD PTR[48+rsi]
	lea	rdi,QWORD PTR[48+rdi]
	call	__subq_mod_384_a_is_loaded


	lea	rsi,QWORD PTR[rdi]
	lea	rbx,QWORD PTR[((-48))+rdi]
	lea	rdi,QWORD PTR[((-48))+rdi]
	call	__mulq_384


	mov	rsi,QWORD PTR[rsp]
	lea	rbx,QWORD PTR[48+rsi]
	lea	rdi,QWORD PTR[96+rdi]
	call	__mulq_384

	mov	r8,QWORD PTR[rdi]
	mov	r9,QWORD PTR[8+rdi]
	mov	r10,QWORD PTR[16+rdi]
	mov	r11,QWORD PTR[24+rdi]
	mov	r12,QWORD PTR[32+rdi]
	mov	r13,QWORD PTR[40+rdi]
	mov	r14,QWORD PTR[48+rdi]
	mov	r15,QWORD PTR[56+rdi]
	mov	rax,QWORD PTR[64+rdi]
	mov	rbx,QWORD PTR[72+rdi]
	mov	rbp,QWORD PTR[80+rdi]
	add	r8,r8
	mov	rdx,QWORD PTR[88+rdi]
	adc	r9,r9
	mov	QWORD PTR[rdi],r8
	adc	r10,r10
	mov	QWORD PTR[8+rdi],r9
	adc	r11,r11
	mov	QWORD PTR[16+rdi],r10
	adc	r12,r12
	mov	QWORD PTR[24+rdi],r11
	adc	r13,r13
	mov	QWORD PTR[32+rdi],r12
	adc	r14,r14
	mov	QWORD PTR[40+rdi],r13
	adc	r15,r15
	mov	QWORD PTR[48+rdi],r14
	adc	rax,rax
	mov	QWORD PTR[56+rdi],r15
	adc	rbx,rbx
	mov	QWORD PTR[64+rdi],rax
	adc	rbp,rbp
	mov	QWORD PTR[72+rdi],rbx
	adc	rdx,rdx
	mov	QWORD PTR[80+rdi],rbp
	mov	QWORD PTR[88+rdi],rdx

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_sqr_382x::
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

$L$SEH_end_sqr_382x::
sqr_382x	ENDP
PUBLIC	mul_384


ALIGN	32
mul_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_mul_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	mul_384$1
endif
	push	rbp

	push	rbx

	push	r12

$L$SEH_body_mul_384::


	mov	rbx,rdx
	call	__mulq_384

	mov	r12,QWORD PTR[rsp]

	mov	rbx,QWORD PTR[8+rsp]

	mov	rbp,QWORD PTR[16+rsp]

	lea	rsp,QWORD PTR[24+rsp]

$L$SEH_epilogue_mul_384::
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

$L$SEH_end_mul_384::
mul_384	ENDP


ALIGN	32
__mulq_384	PROC PRIVATE
	DB	243,15,30,250

	mov	rax,QWORD PTR[rbx]

	mov	rbp,rax
	mul	QWORD PTR[rsi]
	mov	QWORD PTR[rdi],rax
	mov	rax,rbp
	mov	rcx,rdx

	mul	QWORD PTR[8+rsi]
	add	rcx,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[16+rsi]
	add	r8,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[24+rsi]
	add	r9,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[32+rsi]
	add	r10,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[40+rsi]
	add	r11,rax
	mov	rax,QWORD PTR[8+rbx]
	adc	rdx,0
	mov	r12,rdx
	mov	rbp,rax
	mul	QWORD PTR[rsi]
	add	rcx,rax
	mov	rax,rbp
	adc	rdx,0
	mov	QWORD PTR[8+rdi],rcx
	mov	rcx,rdx

	mul	QWORD PTR[8+rsi]
	add	r8,rax
	mov	rax,rbp
	adc	rdx,0
	add	rcx,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[16+rsi]
	add	r9,rax
	mov	rax,rbp
	adc	rdx,0
	add	r8,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[24+rsi]
	add	r10,rax
	mov	rax,rbp
	adc	rdx,0
	add	r9,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[32+rsi]
	add	r11,rax
	mov	rax,rbp
	adc	rdx,0
	add	r10,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[40+rsi]
	add	r12,rax
	mov	rax,QWORD PTR[16+rbx]
	adc	rdx,0
	add	r11,r12
	adc	rdx,0
	mov	r12,rdx
	mov	rbp,rax
	mul	QWORD PTR[rsi]
	add	rcx,rax
	mov	rax,rbp
	adc	rdx,0
	mov	QWORD PTR[16+rdi],rcx
	mov	rcx,rdx

	mul	QWORD PTR[8+rsi]
	add	r8,rax
	mov	rax,rbp
	adc	rdx,0
	add	rcx,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[16+rsi]
	add	r9,rax
	mov	rax,rbp
	adc	rdx,0
	add	r8,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[24+rsi]
	add	r10,rax
	mov	rax,rbp
	adc	rdx,0
	add	r9,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[32+rsi]
	add	r11,rax
	mov	rax,rbp
	adc	rdx,0
	add	r10,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[40+rsi]
	add	r12,rax
	mov	rax,QWORD PTR[24+rbx]
	adc	rdx,0
	add	r11,r12
	adc	rdx,0
	mov	r12,rdx
	mov	rbp,rax
	mul	QWORD PTR[rsi]
	add	rcx,rax
	mov	rax,rbp
	adc	rdx,0
	mov	QWORD PTR[24+rdi],rcx
	mov	rcx,rdx

	mul	QWORD PTR[8+rsi]
	add	r8,rax
	mov	rax,rbp
	adc	rdx,0
	add	rcx,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[16+rsi]
	add	r9,rax
	mov	rax,rbp
	adc	rdx,0
	add	r8,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[24+rsi]
	add	r10,rax
	mov	rax,rbp
	adc	rdx,0
	add	r9,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[32+rsi]
	add	r11,rax
	mov	rax,rbp
	adc	rdx,0
	add	r10,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[40+rsi]
	add	r12,rax
	mov	rax,QWORD PTR[32+rbx]
	adc	rdx,0
	add	r11,r12
	adc	rdx,0
	mov	r12,rdx
	mov	rbp,rax
	mul	QWORD PTR[rsi]
	add	rcx,rax
	mov	rax,rbp
	adc	rdx,0
	mov	QWORD PTR[32+rdi],rcx
	mov	rcx,rdx

	mul	QWORD PTR[8+rsi]
	add	r8,rax
	mov	rax,rbp
	adc	rdx,0
	add	rcx,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[16+rsi]
	add	r9,rax
	mov	rax,rbp
	adc	rdx,0
	add	r8,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[24+rsi]
	add	r10,rax
	mov	rax,rbp
	adc	rdx,0
	add	r9,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[32+rsi]
	add	r11,rax
	mov	rax,rbp
	adc	rdx,0
	add	r10,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[40+rsi]
	add	r12,rax
	mov	rax,QWORD PTR[40+rbx]
	adc	rdx,0
	add	r11,r12
	adc	rdx,0
	mov	r12,rdx
	mov	rbp,rax
	mul	QWORD PTR[rsi]
	add	rcx,rax
	mov	rax,rbp
	adc	rdx,0
	mov	QWORD PTR[40+rdi],rcx
	mov	rcx,rdx

	mul	QWORD PTR[8+rsi]
	add	r8,rax
	mov	rax,rbp
	adc	rdx,0
	add	rcx,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[16+rsi]
	add	r9,rax
	mov	rax,rbp
	adc	rdx,0
	add	r8,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[24+rsi]
	add	r10,rax
	mov	rax,rbp
	adc	rdx,0
	add	r9,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[32+rsi]
	add	r11,rax
	mov	rax,rbp
	adc	rdx,0
	add	r10,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[40+rsi]
	add	r12,rax
	mov	rax,rax
	adc	rdx,0
	add	r11,r12
	adc	rdx,0
	mov	r12,rdx
	mov	QWORD PTR[48+rdi],rcx
	mov	QWORD PTR[56+rdi],r8
	mov	QWORD PTR[64+rdi],r9
	mov	QWORD PTR[72+rdi],r10
	mov	QWORD PTR[80+rdi],r11
	mov	QWORD PTR[88+rdi],r12

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__mulq_384	ENDP
PUBLIC	sqr_384


ALIGN	32
sqr_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqr_384::


	mov	rdi,rcx
	mov	rsi,rdx
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	sqr_384$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_sqr_384::


	call	__sqrq_384

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_sqr_384::
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

$L$SEH_end_sqr_384::
sqr_384	ENDP


ALIGN	32
__sqrq_384	PROC PRIVATE
	DB	243,15,30,250

	mov	rax,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	rcx,QWORD PTR[16+rsi]
	mov	rbx,QWORD PTR[24+rsi]


	mov	r14,rax
	mul	r15
	mov	r9,rax
	mov	rax,r14
	mov	rbp,QWORD PTR[32+rsi]
	mov	r10,rdx

	mul	rcx
	add	r10,rax
	mov	rax,r14
	adc	rdx,0
	mov	rsi,QWORD PTR[40+rsi]
	mov	r11,rdx

	mul	rbx
	add	r11,rax
	mov	rax,r14
	adc	rdx,0
	mov	r12,rdx

	mul	rbp
	add	r12,rax
	mov	rax,r14
	adc	rdx,0
	mov	r13,rdx

	mul	rsi
	add	r13,rax
	mov	rax,r14
	adc	rdx,0
	mov	r14,rdx

	mul	rax
	xor	r8,r8
	mov	QWORD PTR[rdi],rax
	mov	rax,r15
	add	r9,r9
	adc	r8,0
	add	r9,rdx
	adc	r8,0
	mov	QWORD PTR[8+rdi],r9

	mul	rcx
	add	r11,rax
	mov	rax,r15
	adc	rdx,0
	mov	r9,rdx

	mul	rbx
	add	r12,rax
	mov	rax,r15
	adc	rdx,0
	add	r12,r9
	adc	rdx,0
	mov	r9,rdx

	mul	rbp
	add	r13,rax
	mov	rax,r15
	adc	rdx,0
	add	r13,r9
	adc	rdx,0
	mov	r9,rdx

	mul	rsi
	add	r14,rax
	mov	rax,r15
	adc	rdx,0
	add	r14,r9
	adc	rdx,0
	mov	r15,rdx

	mul	rax
	xor	r9,r9
	add	r8,rax
	mov	rax,rcx
	add	r10,r10
	adc	r11,r11
	adc	r9,0
	add	r10,r8
	adc	r11,rdx
	adc	r9,0
	mov	QWORD PTR[16+rdi],r10

	mul	rbx
	add	r13,rax
	mov	rax,rcx
	adc	rdx,0
	mov	QWORD PTR[24+rdi],r11
	mov	r8,rdx

	mul	rbp
	add	r14,rax
	mov	rax,rcx
	adc	rdx,0
	add	r14,r8
	adc	rdx,0
	mov	r8,rdx

	mul	rsi
	add	r15,rax
	mov	rax,rcx
	adc	rdx,0
	add	r15,r8
	adc	rdx,0
	mov	rcx,rdx

	mul	rax
	xor	r11,r11
	add	r9,rax
	mov	rax,rbx
	add	r12,r12
	adc	r13,r13
	adc	r11,0
	add	r12,r9
	adc	r13,rdx
	adc	r11,0
	mov	QWORD PTR[32+rdi],r12


	mul	rbp
	add	r15,rax
	mov	rax,rbx
	adc	rdx,0
	mov	QWORD PTR[40+rdi],r13
	mov	r8,rdx

	mul	rsi
	add	rcx,rax
	mov	rax,rbx
	adc	rdx,0
	add	rcx,r8
	adc	rdx,0
	mov	rbx,rdx

	mul	rax
	xor	r12,r12
	add	r11,rax
	mov	rax,rbp
	add	r14,r14
	adc	r15,r15
	adc	r12,0
	add	r14,r11
	adc	r15,rdx
	mov	QWORD PTR[48+rdi],r14
	adc	r12,0
	mov	QWORD PTR[56+rdi],r15


	mul	rsi
	add	rbx,rax
	mov	rax,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	rax
	xor	r13,r13
	add	r12,rax
	mov	rax,rsi
	add	rcx,rcx
	adc	rbx,rbx
	adc	r13,0
	add	rcx,r12
	adc	rbx,rdx
	mov	QWORD PTR[64+rdi],rcx
	adc	r13,0
	mov	QWORD PTR[72+rdi],rbx


	mul	rax
	add	rax,r13
	add	rbp,rbp
	adc	rdx,0
	add	rax,rbp
	adc	rdx,0
	mov	QWORD PTR[80+rdi],rax
	mov	QWORD PTR[88+rdi],rdx

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__sqrq_384	ENDP

PUBLIC	sqr_mont_384


ALIGN	32
sqr_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqr_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	sqr_mont_384$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8*15

$L$SEH_body_sqr_mont_384::


	mov	QWORD PTR[96+rsp],rcx
	mov	QWORD PTR[104+rsp],rdx
	mov	QWORD PTR[112+rsp],rdi

	mov	rdi,rsp
	call	__sqrq_384

	lea	rsi,QWORD PTR[rsp]
	mov	rcx,QWORD PTR[96+rsp]
	mov	rbx,QWORD PTR[104+rsp]
	mov	rdi,QWORD PTR[112+rsp]
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

	lea	r8,QWORD PTR[120+rsp]
	mov	r15,QWORD PTR[120+rsp]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_sqr_mont_384::
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

$L$SEH_end_sqr_mont_384::
sqr_mont_384	ENDP



PUBLIC	redc_mont_384


ALIGN	32
redc_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_redc_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	redc_mont_384$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_redc_mont_384::


	mov	rbx,rdx
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_redc_mont_384::
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

$L$SEH_end_redc_mont_384::
redc_mont_384	ENDP




PUBLIC	from_mont_384


ALIGN	32
from_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_from_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	from_mont_384$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_from_mont_384::


	mov	rbx,rdx
	call	__mulq_by_1_mont_384





	mov	rcx,r15
	mov	rdx,r8
	mov	rbp,r9

	sub	r14,QWORD PTR[rbx]
	sbb	r15,QWORD PTR[8+rbx]
	mov	r13,r10
	sbb	r8,QWORD PTR[16+rbx]
	sbb	r9,QWORD PTR[24+rbx]
	sbb	r10,QWORD PTR[32+rbx]
	mov	rsi,r11
	sbb	r11,QWORD PTR[40+rbx]

	cmovc	r14,rax
	cmovc	r15,rcx
	cmovc	r8,rdx
	mov	QWORD PTR[rdi],r14
	cmovc	r9,rbp
	mov	QWORD PTR[8+rdi],r15
	cmovc	r10,r13
	mov	QWORD PTR[16+rdi],r8
	cmovc	r11,rsi
	mov	QWORD PTR[24+rdi],r9
	mov	QWORD PTR[32+rdi],r10
	mov	QWORD PTR[40+rdi],r11

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_from_mont_384::
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

$L$SEH_end_from_mont_384::
from_mont_384	ENDP

ALIGN	32
__mulq_by_1_mont_384	PROC PRIVATE
	DB	243,15,30,250

	mov	rax,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

	mov	r14,rax
	imul	rax,rcx
	mov	r8,rax

	mul	QWORD PTR[rbx]
	add	r14,rax
	mov	rax,r8
	adc	r14,rdx

	mul	QWORD PTR[8+rbx]
	add	r9,rax
	mov	rax,r8
	adc	rdx,0
	add	r9,r14
	adc	rdx,0
	mov	r14,rdx

	mul	QWORD PTR[16+rbx]
	add	r10,rax
	mov	rax,r8
	adc	rdx,0
	add	r10,r14
	adc	rdx,0
	mov	r14,rdx

	mul	QWORD PTR[24+rbx]
	add	r11,rax
	mov	rax,r8
	adc	rdx,0
	mov	r15,r9
	imul	r9,rcx
	add	r11,r14
	adc	rdx,0
	mov	r14,rdx

	mul	QWORD PTR[32+rbx]
	add	r12,rax
	mov	rax,r8
	adc	rdx,0
	add	r12,r14
	adc	rdx,0
	mov	r14,rdx

	mul	QWORD PTR[40+rbx]
	add	r13,rax
	mov	rax,r9
	adc	rdx,0
	add	r13,r14
	adc	rdx,0
	mov	r14,rdx

	mul	QWORD PTR[rbx]
	add	r15,rax
	mov	rax,r9
	adc	r15,rdx

	mul	QWORD PTR[8+rbx]
	add	r10,rax
	mov	rax,r9
	adc	rdx,0
	add	r10,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[16+rbx]
	add	r11,rax
	mov	rax,r9
	adc	rdx,0
	add	r11,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[24+rbx]
	add	r12,rax
	mov	rax,r9
	adc	rdx,0
	mov	r8,r10
	imul	r10,rcx
	add	r12,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[32+rbx]
	add	r13,rax
	mov	rax,r9
	adc	rdx,0
	add	r13,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[40+rbx]
	add	r14,rax
	mov	rax,r10
	adc	rdx,0
	add	r14,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[rbx]
	add	r8,rax
	mov	rax,r10
	adc	r8,rdx

	mul	QWORD PTR[8+rbx]
	add	r11,rax
	mov	rax,r10
	adc	rdx,0
	add	r11,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[16+rbx]
	add	r12,rax
	mov	rax,r10
	adc	rdx,0
	add	r12,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[24+rbx]
	add	r13,rax
	mov	rax,r10
	adc	rdx,0
	mov	r9,r11
	imul	r11,rcx
	add	r13,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[32+rbx]
	add	r14,rax
	mov	rax,r10
	adc	rdx,0
	add	r14,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[40+rbx]
	add	r15,rax
	mov	rax,r11
	adc	rdx,0
	add	r15,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[rbx]
	add	r9,rax
	mov	rax,r11
	adc	r9,rdx

	mul	QWORD PTR[8+rbx]
	add	r12,rax
	mov	rax,r11
	adc	rdx,0
	add	r12,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[16+rbx]
	add	r13,rax
	mov	rax,r11
	adc	rdx,0
	add	r13,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[24+rbx]
	add	r14,rax
	mov	rax,r11
	adc	rdx,0
	mov	r10,r12
	imul	r12,rcx
	add	r14,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[32+rbx]
	add	r15,rax
	mov	rax,r11
	adc	rdx,0
	add	r15,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[40+rbx]
	add	r8,rax
	mov	rax,r12
	adc	rdx,0
	add	r8,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[rbx]
	add	r10,rax
	mov	rax,r12
	adc	r10,rdx

	mul	QWORD PTR[8+rbx]
	add	r13,rax
	mov	rax,r12
	adc	rdx,0
	add	r13,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[16+rbx]
	add	r14,rax
	mov	rax,r12
	adc	rdx,0
	add	r14,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[24+rbx]
	add	r15,rax
	mov	rax,r12
	adc	rdx,0
	mov	r11,r13
	imul	r13,rcx
	add	r15,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[32+rbx]
	add	r8,rax
	mov	rax,r12
	adc	rdx,0
	add	r8,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[40+rbx]
	add	r9,rax
	mov	rax,r13
	adc	rdx,0
	add	r9,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[rbx]
	add	r11,rax
	mov	rax,r13
	adc	r11,rdx

	mul	QWORD PTR[8+rbx]
	add	r14,rax
	mov	rax,r13
	adc	rdx,0
	add	r14,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[16+rbx]
	add	r15,rax
	mov	rax,r13
	adc	rdx,0
	add	r15,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[24+rbx]
	add	r8,rax
	mov	rax,r13
	adc	rdx,0
	add	r8,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[32+rbx]
	add	r9,rax
	mov	rax,r13
	adc	rdx,0
	add	r9,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[40+rbx]
	add	r10,rax
	mov	rax,r14
	adc	rdx,0
	add	r10,r11
	adc	rdx,0
	mov	r11,rdx
	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__mulq_by_1_mont_384	ENDP


ALIGN	32
__redq_tail_mont_384	PROC PRIVATE
	DB	243,15,30,250

	add	r14,QWORD PTR[48+rsi]
	mov	rax,r14
	adc	r15,QWORD PTR[56+rsi]
	adc	r8,QWORD PTR[64+rsi]
	adc	r9,QWORD PTR[72+rsi]
	mov	rcx,r15
	adc	r10,QWORD PTR[80+rsi]
	adc	r11,QWORD PTR[88+rsi]
	sbb	r12,r12




	mov	rdx,r8
	mov	rbp,r9

	sub	r14,QWORD PTR[rbx]
	sbb	r15,QWORD PTR[8+rbx]
	mov	r13,r10
	sbb	r8,QWORD PTR[16+rbx]
	sbb	r9,QWORD PTR[24+rbx]
	sbb	r10,QWORD PTR[32+rbx]
	mov	rsi,r11
	sbb	r11,QWORD PTR[40+rbx]
	sbb	r12,0

	cmovc	r14,rax
	cmovc	r15,rcx
	cmovc	r8,rdx
	mov	QWORD PTR[rdi],r14
	cmovc	r9,rbp
	mov	QWORD PTR[8+rdi],r15
	cmovc	r10,r13
	mov	QWORD PTR[16+rdi],r8
	cmovc	r11,rsi
	mov	QWORD PTR[24+rdi],r9
	mov	QWORD PTR[32+rdi],r10
	mov	QWORD PTR[40+rdi],r11

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__redq_tail_mont_384	ENDP

PUBLIC	sgn0_pty_mont_384


ALIGN	32
sgn0_pty_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sgn0_pty_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	sgn0_pty_mont_384$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_sgn0_pty_mont_384::


	mov	rbx,rsi
	lea	rsi,QWORD PTR[rdi]
	mov	rcx,rdx
	call	__mulq_by_1_mont_384

	xor	rax,rax
	mov	r13,r14
	add	r14,r14
	adc	r15,r15
	adc	r8,r8
	adc	r9,r9
	adc	r10,r10
	adc	r11,r11
	adc	rax,0

	sub	r14,QWORD PTR[rbx]
	sbb	r15,QWORD PTR[8+rbx]
	sbb	r8,QWORD PTR[16+rbx]
	sbb	r9,QWORD PTR[24+rbx]
	sbb	r10,QWORD PTR[32+rbx]
	sbb	r11,QWORD PTR[40+rbx]
	sbb	rax,0

	not	rax
	and	r13,1
	and	rax,2
	or	rax,r13

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_sgn0_pty_mont_384::
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

$L$SEH_end_sgn0_pty_mont_384::
sgn0_pty_mont_384	ENDP

PUBLIC	sgn0_pty_mont_384x


ALIGN	32
sgn0_pty_mont_384x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sgn0_pty_mont_384x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	sgn0_pty_mont_384x$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_sgn0_pty_mont_384x::


	mov	rbx,rsi
	lea	rsi,QWORD PTR[48+rdi]
	mov	rcx,rdx
	call	__mulq_by_1_mont_384

	mov	r12,r14
	or	r14,r15
	or	r14,r8
	or	r14,r9
	or	r14,r10
	or	r14,r11

	lea	rsi,QWORD PTR[rdi]
	xor	rdi,rdi
	mov	r13,r12
	add	r12,r12
	adc	r15,r15
	adc	r8,r8
	adc	r9,r9
	adc	r10,r10
	adc	r11,r11
	adc	rdi,0

	sub	r12,QWORD PTR[rbx]
	sbb	r15,QWORD PTR[8+rbx]
	sbb	r8,QWORD PTR[16+rbx]
	sbb	r9,QWORD PTR[24+rbx]
	sbb	r10,QWORD PTR[32+rbx]
	sbb	r11,QWORD PTR[40+rbx]
	sbb	rdi,0

	mov	QWORD PTR[rsp],r14
	not	rdi
	and	r13,1
	and	rdi,2
	or	rdi,r13

	call	__mulq_by_1_mont_384

	mov	r12,r14
	or	r14,r15
	or	r14,r8
	or	r14,r9
	or	r14,r10
	or	r14,r11

	xor	rax,rax
	mov	r13,r12
	add	r12,r12
	adc	r15,r15
	adc	r8,r8
	adc	r9,r9
	adc	r10,r10
	adc	r11,r11
	adc	rax,0

	sub	r12,QWORD PTR[rbx]
	sbb	r15,QWORD PTR[8+rbx]
	sbb	r8,QWORD PTR[16+rbx]
	sbb	r9,QWORD PTR[24+rbx]
	sbb	r10,QWORD PTR[32+rbx]
	sbb	r11,QWORD PTR[40+rbx]
	sbb	rax,0

	mov	r12,QWORD PTR[rsp]

	not	rax

	test	r14,r14
	cmovz	r13,rdi

	test	r12,r12
	cmovnz	rax,rdi

	and	r13,1
	and	rax,2
	or	rax,r13

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_sgn0_pty_mont_384x::
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

$L$SEH_end_sgn0_pty_mont_384x::
sgn0_pty_mont_384x	ENDP
PUBLIC	mul_mont_384


ALIGN	32
mul_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_mul_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	mov	r8,QWORD PTR[40+rsp]
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	mul_mont_384$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8*3

$L$SEH_body_mul_mont_384::


	mov	rax,QWORD PTR[rdx]
	mov	r14,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	r12,QWORD PTR[16+rsi]
	mov	r13,QWORD PTR[24+rsi]
	mov	rbx,rdx
	mov	QWORD PTR[rsp],r8
	mov	QWORD PTR[8+rsp],rdi

	call	__mulq_mont_384

	mov	r15,QWORD PTR[24+rsp]

	mov	r14,QWORD PTR[32+rsp]

	mov	r13,QWORD PTR[40+rsp]

	mov	r12,QWORD PTR[48+rsp]

	mov	rbx,QWORD PTR[56+rsp]

	mov	rbp,QWORD PTR[64+rsp]

	lea	rsp,QWORD PTR[72+rsp]

$L$SEH_epilogue_mul_mont_384::
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

$L$SEH_end_mul_mont_384::
mul_mont_384	ENDP

ALIGN	32
__mulq_mont_384	PROC PRIVATE
	DB	243,15,30,250

	mov	rdi,rax
	mul	r14
	mov	r8,rax
	mov	rax,rdi
	mov	r9,rdx

	mul	r15
	add	r9,rax
	mov	rax,rdi
	adc	rdx,0
	mov	r10,rdx

	mul	r12
	add	r10,rax
	mov	rax,rdi
	adc	rdx,0
	mov	r11,rdx

	mov	rbp,r8
	imul	r8,QWORD PTR[8+rsp]

	mul	r13
	add	r11,rax
	mov	rax,rdi
	adc	rdx,0
	mov	r12,rdx

	mul	QWORD PTR[32+rsi]
	add	r12,rax
	mov	rax,rdi
	adc	rdx,0
	mov	r13,rdx

	mul	QWORD PTR[40+rsi]
	add	r13,rax
	mov	rax,r8
	adc	rdx,0
	xor	r15,r15
	mov	r14,rdx

	mul	QWORD PTR[rcx]
	add	rbp,rax
	mov	rax,r8
	adc	rbp,rdx

	mul	QWORD PTR[8+rcx]
	add	r9,rax
	mov	rax,r8
	adc	rdx,0
	add	r9,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[16+rcx]
	add	r10,rax
	mov	rax,r8
	adc	rdx,0
	add	r10,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[24+rcx]
	add	r11,rbp
	adc	rdx,0
	add	r11,rax
	mov	rax,r8
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[32+rcx]
	add	r12,rax
	mov	rax,r8
	adc	rdx,0
	add	r12,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[40+rcx]
	add	r13,rax
	mov	rax,QWORD PTR[8+rbx]
	adc	rdx,0
	add	r13,rbp
	adc	r14,rdx
	adc	r15,0

	mov	rdi,rax
	mul	QWORD PTR[rsi]
	add	r9,rax
	mov	rax,rdi
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[8+rsi]
	add	r10,rax
	mov	rax,rdi
	adc	rdx,0
	add	r10,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[16+rsi]
	add	r11,rax
	mov	rax,rdi
	adc	rdx,0
	add	r11,r8
	adc	rdx,0
	mov	r8,rdx

	mov	rbp,r9
	imul	r9,QWORD PTR[8+rsp]

	mul	QWORD PTR[24+rsi]
	add	r12,rax
	mov	rax,rdi
	adc	rdx,0
	add	r12,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[32+rsi]
	add	r13,rax
	mov	rax,rdi
	adc	rdx,0
	add	r13,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[40+rsi]
	add	r14,r8
	adc	rdx,0
	xor	r8,r8
	add	r14,rax
	mov	rax,r9
	adc	r15,rdx
	adc	r8,0

	mul	QWORD PTR[rcx]
	add	rbp,rax
	mov	rax,r9
	adc	rbp,rdx

	mul	QWORD PTR[8+rcx]
	add	r10,rax
	mov	rax,r9
	adc	rdx,0
	add	r10,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[16+rcx]
	add	r11,rax
	mov	rax,r9
	adc	rdx,0
	add	r11,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[24+rcx]
	add	r12,rbp
	adc	rdx,0
	add	r12,rax
	mov	rax,r9
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[32+rcx]
	add	r13,rax
	mov	rax,r9
	adc	rdx,0
	add	r13,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[40+rcx]
	add	r14,rax
	mov	rax,QWORD PTR[16+rbx]
	adc	rdx,0
	add	r14,rbp
	adc	r15,rdx
	adc	r8,0

	mov	rdi,rax
	mul	QWORD PTR[rsi]
	add	r10,rax
	mov	rax,rdi
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[8+rsi]
	add	r11,rax
	mov	rax,rdi
	adc	rdx,0
	add	r11,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[16+rsi]
	add	r12,rax
	mov	rax,rdi
	adc	rdx,0
	add	r12,r9
	adc	rdx,0
	mov	r9,rdx

	mov	rbp,r10
	imul	r10,QWORD PTR[8+rsp]

	mul	QWORD PTR[24+rsi]
	add	r13,rax
	mov	rax,rdi
	adc	rdx,0
	add	r13,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[32+rsi]
	add	r14,rax
	mov	rax,rdi
	adc	rdx,0
	add	r14,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[40+rsi]
	add	r15,r9
	adc	rdx,0
	xor	r9,r9
	add	r15,rax
	mov	rax,r10
	adc	r8,rdx
	adc	r9,0

	mul	QWORD PTR[rcx]
	add	rbp,rax
	mov	rax,r10
	adc	rbp,rdx

	mul	QWORD PTR[8+rcx]
	add	r11,rax
	mov	rax,r10
	adc	rdx,0
	add	r11,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[16+rcx]
	add	r12,rax
	mov	rax,r10
	adc	rdx,0
	add	r12,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[24+rcx]
	add	r13,rbp
	adc	rdx,0
	add	r13,rax
	mov	rax,r10
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[32+rcx]
	add	r14,rax
	mov	rax,r10
	adc	rdx,0
	add	r14,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[40+rcx]
	add	r15,rax
	mov	rax,QWORD PTR[24+rbx]
	adc	rdx,0
	add	r15,rbp
	adc	r8,rdx
	adc	r9,0

	mov	rdi,rax
	mul	QWORD PTR[rsi]
	add	r11,rax
	mov	rax,rdi
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[8+rsi]
	add	r12,rax
	mov	rax,rdi
	adc	rdx,0
	add	r12,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[16+rsi]
	add	r13,rax
	mov	rax,rdi
	adc	rdx,0
	add	r13,r10
	adc	rdx,0
	mov	r10,rdx

	mov	rbp,r11
	imul	r11,QWORD PTR[8+rsp]

	mul	QWORD PTR[24+rsi]
	add	r14,rax
	mov	rax,rdi
	adc	rdx,0
	add	r14,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[32+rsi]
	add	r15,rax
	mov	rax,rdi
	adc	rdx,0
	add	r15,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[40+rsi]
	add	r8,r10
	adc	rdx,0
	xor	r10,r10
	add	r8,rax
	mov	rax,r11
	adc	r9,rdx
	adc	r10,0

	mul	QWORD PTR[rcx]
	add	rbp,rax
	mov	rax,r11
	adc	rbp,rdx

	mul	QWORD PTR[8+rcx]
	add	r12,rax
	mov	rax,r11
	adc	rdx,0
	add	r12,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[16+rcx]
	add	r13,rax
	mov	rax,r11
	adc	rdx,0
	add	r13,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[24+rcx]
	add	r14,rbp
	adc	rdx,0
	add	r14,rax
	mov	rax,r11
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[32+rcx]
	add	r15,rax
	mov	rax,r11
	adc	rdx,0
	add	r15,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[40+rcx]
	add	r8,rax
	mov	rax,QWORD PTR[32+rbx]
	adc	rdx,0
	add	r8,rbp
	adc	r9,rdx
	adc	r10,0

	mov	rdi,rax
	mul	QWORD PTR[rsi]
	add	r12,rax
	mov	rax,rdi
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[8+rsi]
	add	r13,rax
	mov	rax,rdi
	adc	rdx,0
	add	r13,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[16+rsi]
	add	r14,rax
	mov	rax,rdi
	adc	rdx,0
	add	r14,r11
	adc	rdx,0
	mov	r11,rdx

	mov	rbp,r12
	imul	r12,QWORD PTR[8+rsp]

	mul	QWORD PTR[24+rsi]
	add	r15,rax
	mov	rax,rdi
	adc	rdx,0
	add	r15,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[32+rsi]
	add	r8,rax
	mov	rax,rdi
	adc	rdx,0
	add	r8,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[40+rsi]
	add	r9,r11
	adc	rdx,0
	xor	r11,r11
	add	r9,rax
	mov	rax,r12
	adc	r10,rdx
	adc	r11,0

	mul	QWORD PTR[rcx]
	add	rbp,rax
	mov	rax,r12
	adc	rbp,rdx

	mul	QWORD PTR[8+rcx]
	add	r13,rax
	mov	rax,r12
	adc	rdx,0
	add	r13,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[16+rcx]
	add	r14,rax
	mov	rax,r12
	adc	rdx,0
	add	r14,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[24+rcx]
	add	r15,rbp
	adc	rdx,0
	add	r15,rax
	mov	rax,r12
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[32+rcx]
	add	r8,rax
	mov	rax,r12
	adc	rdx,0
	add	r8,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[40+rcx]
	add	r9,rax
	mov	rax,QWORD PTR[40+rbx]
	adc	rdx,0
	add	r9,rbp
	adc	r10,rdx
	adc	r11,0

	mov	rdi,rax
	mul	QWORD PTR[rsi]
	add	r13,rax
	mov	rax,rdi
	adc	rdx,0
	mov	r12,rdx

	mul	QWORD PTR[8+rsi]
	add	r14,rax
	mov	rax,rdi
	adc	rdx,0
	add	r14,r12
	adc	rdx,0
	mov	r12,rdx

	mul	QWORD PTR[16+rsi]
	add	r15,rax
	mov	rax,rdi
	adc	rdx,0
	add	r15,r12
	adc	rdx,0
	mov	r12,rdx

	mov	rbp,r13
	imul	r13,QWORD PTR[8+rsp]

	mul	QWORD PTR[24+rsi]
	add	r8,rax
	mov	rax,rdi
	adc	rdx,0
	add	r8,r12
	adc	rdx,0
	mov	r12,rdx

	mul	QWORD PTR[32+rsi]
	add	r9,rax
	mov	rax,rdi
	adc	rdx,0
	add	r9,r12
	adc	rdx,0
	mov	r12,rdx

	mul	QWORD PTR[40+rsi]
	add	r10,r12
	adc	rdx,0
	xor	r12,r12
	add	r10,rax
	mov	rax,r13
	adc	r11,rdx
	adc	r12,0

	mul	QWORD PTR[rcx]
	add	rbp,rax
	mov	rax,r13
	adc	rbp,rdx

	mul	QWORD PTR[8+rcx]
	add	r14,rax
	mov	rax,r13
	adc	rdx,0
	add	r14,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[16+rcx]
	add	r15,rax
	mov	rax,r13
	adc	rdx,0
	add	r15,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[24+rcx]
	add	r8,rbp
	adc	rdx,0
	add	r8,rax
	mov	rax,r13
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[32+rcx]
	add	r9,rax
	mov	rax,r13
	adc	rdx,0
	add	r9,rbp
	adc	rdx,0
	mov	rbp,rdx

	mul	QWORD PTR[40+rcx]
	add	r10,rax
	mov	rax,r14
	adc	rdx,0
	add	r10,rbp
	adc	r11,rdx
	adc	r12,0




	mov	rdi,QWORD PTR[16+rsp]
	sub	r14,QWORD PTR[rcx]
	mov	rdx,r15
	sbb	r15,QWORD PTR[8+rcx]
	mov	rbx,r8
	sbb	r8,QWORD PTR[16+rcx]
	mov	rsi,r9
	sbb	r9,QWORD PTR[24+rcx]
	mov	rbp,r10
	sbb	r10,QWORD PTR[32+rcx]
	mov	r13,r11
	sbb	r11,QWORD PTR[40+rcx]
	sbb	r12,0

	cmovc	r14,rax
	cmovc	r15,rdx
	cmovc	r8,rbx
	mov	QWORD PTR[rdi],r14
	cmovc	r9,rsi
	mov	QWORD PTR[8+rdi],r15
	cmovc	r10,rbp
	mov	QWORD PTR[16+rdi],r8
	cmovc	r11,r13
	mov	QWORD PTR[24+rdi],r9
	mov	QWORD PTR[32+rdi],r10
	mov	QWORD PTR[40+rdi],r11

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__mulq_mont_384	ENDP
PUBLIC	sqr_n_mul_mont_384


ALIGN	32
sqr_n_mul_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqr_n_mul_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	mov	r8,QWORD PTR[40+rsp]
	mov	r9,QWORD PTR[48+rsp]
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	sqr_n_mul_mont_384$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8*17

$L$SEH_body_sqr_n_mul_mont_384::


	mov	QWORD PTR[rsp],r8
	mov	QWORD PTR[8+rsp],rdi
	mov	QWORD PTR[16+rsp],rcx
	lea	rdi,QWORD PTR[32+rsp]
	mov	QWORD PTR[24+rsp],r9
	movq	xmm2,QWORD PTR[r9]

$L$oop_sqr_384::
	movd	xmm1,edx

	call	__sqrq_384

	lea	rsi,QWORD PTR[rdi]
	mov	rcx,QWORD PTR[rsp]
	mov	rbx,QWORD PTR[16+rsp]
	call	__mulq_by_1_mont_384
	call	__redq_tail_mont_384

	movd	edx,xmm1
	lea	rsi,QWORD PTR[rdi]
	dec	edx
	jnz	$L$oop_sqr_384

DB	102,72,15,126,208
	mov	rcx,rbx
	mov	rbx,QWORD PTR[24+rsp]






	mov	r12,r8
	mov	r13,r9

	call	__mulq_mont_384

	lea	r8,QWORD PTR[136+rsp]
	mov	r15,QWORD PTR[136+rsp]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_sqr_n_mul_mont_384::
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

$L$SEH_end_sqr_n_mul_mont_384::
sqr_n_mul_mont_384	ENDP

PUBLIC	sqr_n_mul_mont_383


ALIGN	32
sqr_n_mul_mont_383	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqr_n_mul_mont_383::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	mov	r8,QWORD PTR[40+rsp]
	mov	r9,QWORD PTR[48+rsp]
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	sqr_n_mul_mont_383$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8*17

$L$SEH_body_sqr_n_mul_mont_383::


	mov	QWORD PTR[rsp],r8
	mov	QWORD PTR[8+rsp],rdi
	mov	QWORD PTR[16+rsp],rcx
	lea	rdi,QWORD PTR[32+rsp]
	mov	QWORD PTR[24+rsp],r9
	movq	xmm2,QWORD PTR[r9]

$L$oop_sqr_383::
	movd	xmm1,edx

	call	__sqrq_384

	lea	rsi,QWORD PTR[rdi]
	mov	rcx,QWORD PTR[rsp]
	mov	rbx,QWORD PTR[16+rsp]
	call	__mulq_by_1_mont_384

	movd	edx,xmm1
	add	r14,QWORD PTR[48+rsi]
	adc	r15,QWORD PTR[56+rsi]
	adc	r8,QWORD PTR[64+rsi]
	adc	r9,QWORD PTR[72+rsi]
	adc	r10,QWORD PTR[80+rsi]
	adc	r11,QWORD PTR[88+rsi]
	lea	rsi,QWORD PTR[rdi]

	mov	QWORD PTR[rdi],r14
	mov	QWORD PTR[8+rdi],r15
	mov	QWORD PTR[16+rdi],r8
	mov	QWORD PTR[24+rdi],r9
	mov	QWORD PTR[32+rdi],r10
	mov	QWORD PTR[40+rdi],r11

	dec	edx
	jnz	$L$oop_sqr_383

DB	102,72,15,126,208
	mov	rcx,rbx
	mov	rbx,QWORD PTR[24+rsp]






	mov	r12,r8
	mov	r13,r9

	call	__mulq_mont_384

	lea	r8,QWORD PTR[136+rsp]
	mov	r15,QWORD PTR[136+rsp]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_sqr_n_mul_mont_383::
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

$L$SEH_end_sqr_n_mul_mont_383::
sqr_n_mul_mont_383	ENDP

ALIGN	32
__mulq_mont_383_nonred	PROC PRIVATE
	DB	243,15,30,250

	mov	rbp,rax
	mul	r14
	mov	r8,rax
	mov	rax,rbp
	mov	r9,rdx

	mul	r15
	add	r9,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r10,rdx

	mul	r12
	add	r10,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r11,rdx

	mov	r15,r8
	imul	r8,QWORD PTR[8+rsp]

	mul	r13
	add	r11,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r12,rdx

	mul	QWORD PTR[32+rsi]
	add	r12,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r13,rdx

	mul	QWORD PTR[40+rsi]
	add	r13,rax
	mov	rax,r8
	adc	rdx,0
	mov	r14,rdx

	mul	QWORD PTR[rcx]
	add	r15,rax
	mov	rax,r8
	adc	r15,rdx

	mul	QWORD PTR[8+rcx]
	add	r9,rax
	mov	rax,r8
	adc	rdx,0
	add	r9,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[16+rcx]
	add	r10,rax
	mov	rax,r8
	adc	rdx,0
	add	r10,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[24+rcx]
	add	r11,r15
	adc	rdx,0
	add	r11,rax
	mov	rax,r8
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[32+rcx]
	add	r12,rax
	mov	rax,r8
	adc	rdx,0
	add	r12,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[40+rcx]
	add	r13,rax
	mov	rax,QWORD PTR[8+rbx]
	adc	rdx,0
	add	r13,r15
	adc	r14,rdx

	mov	rbp,rax
	mul	QWORD PTR[rsi]
	add	r9,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[8+rsi]
	add	r10,rax
	mov	rax,rbp
	adc	rdx,0
	add	r10,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[16+rsi]
	add	r11,rax
	mov	rax,rbp
	adc	rdx,0
	add	r11,r15
	adc	rdx,0
	mov	r15,rdx

	mov	r8,r9
	imul	r9,QWORD PTR[8+rsp]

	mul	QWORD PTR[24+rsi]
	add	r12,rax
	mov	rax,rbp
	adc	rdx,0
	add	r12,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[32+rsi]
	add	r13,rax
	mov	rax,rbp
	adc	rdx,0
	add	r13,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[40+rsi]
	add	r14,r15
	adc	rdx,0
	add	r14,rax
	mov	rax,r9
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[rcx]
	add	r8,rax
	mov	rax,r9
	adc	r8,rdx

	mul	QWORD PTR[8+rcx]
	add	r10,rax
	mov	rax,r9
	adc	rdx,0
	add	r10,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[16+rcx]
	add	r11,rax
	mov	rax,r9
	adc	rdx,0
	add	r11,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[24+rcx]
	add	r12,r8
	adc	rdx,0
	add	r12,rax
	mov	rax,r9
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[32+rcx]
	add	r13,rax
	mov	rax,r9
	adc	rdx,0
	add	r13,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[40+rcx]
	add	r14,rax
	mov	rax,QWORD PTR[16+rbx]
	adc	rdx,0
	add	r14,r8
	adc	r15,rdx

	mov	rbp,rax
	mul	QWORD PTR[rsi]
	add	r10,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[8+rsi]
	add	r11,rax
	mov	rax,rbp
	adc	rdx,0
	add	r11,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[16+rsi]
	add	r12,rax
	mov	rax,rbp
	adc	rdx,0
	add	r12,r8
	adc	rdx,0
	mov	r8,rdx

	mov	r9,r10
	imul	r10,QWORD PTR[8+rsp]

	mul	QWORD PTR[24+rsi]
	add	r13,rax
	mov	rax,rbp
	adc	rdx,0
	add	r13,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[32+rsi]
	add	r14,rax
	mov	rax,rbp
	adc	rdx,0
	add	r14,r8
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[40+rsi]
	add	r15,r8
	adc	rdx,0
	add	r15,rax
	mov	rax,r10
	adc	rdx,0
	mov	r8,rdx

	mul	QWORD PTR[rcx]
	add	r9,rax
	mov	rax,r10
	adc	r9,rdx

	mul	QWORD PTR[8+rcx]
	add	r11,rax
	mov	rax,r10
	adc	rdx,0
	add	r11,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[16+rcx]
	add	r12,rax
	mov	rax,r10
	adc	rdx,0
	add	r12,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[24+rcx]
	add	r13,r9
	adc	rdx,0
	add	r13,rax
	mov	rax,r10
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[32+rcx]
	add	r14,rax
	mov	rax,r10
	adc	rdx,0
	add	r14,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[40+rcx]
	add	r15,rax
	mov	rax,QWORD PTR[24+rbx]
	adc	rdx,0
	add	r15,r9
	adc	r8,rdx

	mov	rbp,rax
	mul	QWORD PTR[rsi]
	add	r11,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[8+rsi]
	add	r12,rax
	mov	rax,rbp
	adc	rdx,0
	add	r12,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[16+rsi]
	add	r13,rax
	mov	rax,rbp
	adc	rdx,0
	add	r13,r9
	adc	rdx,0
	mov	r9,rdx

	mov	r10,r11
	imul	r11,QWORD PTR[8+rsp]

	mul	QWORD PTR[24+rsi]
	add	r14,rax
	mov	rax,rbp
	adc	rdx,0
	add	r14,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[32+rsi]
	add	r15,rax
	mov	rax,rbp
	adc	rdx,0
	add	r15,r9
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[40+rsi]
	add	r8,r9
	adc	rdx,0
	add	r8,rax
	mov	rax,r11
	adc	rdx,0
	mov	r9,rdx

	mul	QWORD PTR[rcx]
	add	r10,rax
	mov	rax,r11
	adc	r10,rdx

	mul	QWORD PTR[8+rcx]
	add	r12,rax
	mov	rax,r11
	adc	rdx,0
	add	r12,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[16+rcx]
	add	r13,rax
	mov	rax,r11
	adc	rdx,0
	add	r13,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[24+rcx]
	add	r14,r10
	adc	rdx,0
	add	r14,rax
	mov	rax,r11
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[32+rcx]
	add	r15,rax
	mov	rax,r11
	adc	rdx,0
	add	r15,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[40+rcx]
	add	r8,rax
	mov	rax,QWORD PTR[32+rbx]
	adc	rdx,0
	add	r8,r10
	adc	r9,rdx

	mov	rbp,rax
	mul	QWORD PTR[rsi]
	add	r12,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[8+rsi]
	add	r13,rax
	mov	rax,rbp
	adc	rdx,0
	add	r13,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[16+rsi]
	add	r14,rax
	mov	rax,rbp
	adc	rdx,0
	add	r14,r10
	adc	rdx,0
	mov	r10,rdx

	mov	r11,r12
	imul	r12,QWORD PTR[8+rsp]

	mul	QWORD PTR[24+rsi]
	add	r15,rax
	mov	rax,rbp
	adc	rdx,0
	add	r15,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[32+rsi]
	add	r8,rax
	mov	rax,rbp
	adc	rdx,0
	add	r8,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[40+rsi]
	add	r9,r10
	adc	rdx,0
	add	r9,rax
	mov	rax,r12
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[rcx]
	add	r11,rax
	mov	rax,r12
	adc	r11,rdx

	mul	QWORD PTR[8+rcx]
	add	r13,rax
	mov	rax,r12
	adc	rdx,0
	add	r13,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[16+rcx]
	add	r14,rax
	mov	rax,r12
	adc	rdx,0
	add	r14,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[24+rcx]
	add	r15,r11
	adc	rdx,0
	add	r15,rax
	mov	rax,r12
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[32+rcx]
	add	r8,rax
	mov	rax,r12
	adc	rdx,0
	add	r8,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[40+rcx]
	add	r9,rax
	mov	rax,QWORD PTR[40+rbx]
	adc	rdx,0
	add	r9,r11
	adc	r10,rdx

	mov	rbp,rax
	mul	QWORD PTR[rsi]
	add	r13,rax
	mov	rax,rbp
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[8+rsi]
	add	r14,rax
	mov	rax,rbp
	adc	rdx,0
	add	r14,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[16+rsi]
	add	r15,rax
	mov	rax,rbp
	adc	rdx,0
	add	r15,r11
	adc	rdx,0
	mov	r11,rdx

	mov	r12,r13
	imul	r13,QWORD PTR[8+rsp]

	mul	QWORD PTR[24+rsi]
	add	r8,rax
	mov	rax,rbp
	adc	rdx,0
	add	r8,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[32+rsi]
	add	r9,rax
	mov	rax,rbp
	adc	rdx,0
	add	r9,r11
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[40+rsi]
	add	r10,r11
	adc	rdx,0
	add	r10,rax
	mov	rax,r13
	adc	rdx,0
	mov	r11,rdx

	mul	QWORD PTR[rcx]
	add	r12,rax
	mov	rax,r13
	adc	r12,rdx

	mul	QWORD PTR[8+rcx]
	add	r14,rax
	mov	rax,r13
	adc	rdx,0
	add	r14,r12
	adc	rdx,0
	mov	r12,rdx

	mul	QWORD PTR[16+rcx]
	add	r15,rax
	mov	rax,r13
	adc	rdx,0
	add	r15,r12
	adc	rdx,0
	mov	r12,rdx

	mul	QWORD PTR[24+rcx]
	add	r8,r12
	adc	rdx,0
	add	r8,rax
	mov	rax,r13
	adc	rdx,0
	mov	r12,rdx

	mul	QWORD PTR[32+rcx]
	add	r9,rax
	mov	rax,r13
	adc	rdx,0
	add	r9,r12
	adc	rdx,0
	mov	r12,rdx

	mul	QWORD PTR[40+rcx]
	add	r10,rax
	mov	rax,r14
	adc	rdx,0
	add	r10,r12
	adc	r11,rdx
	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__mulq_mont_383_nonred	ENDP
PUBLIC	sqr_mont_382x


ALIGN	32
sqr_mont_382x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqr_mont_382x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
ifdef __BLST_PORTABLE__
	test	DWORD PTR[__blst_platform_cap],1
	jnz	sqr_mont_382x$1
endif
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,136

$L$SEH_body_sqr_mont_382x::


	mov	QWORD PTR[rsp],rcx
	mov	rcx,rdx
	mov	QWORD PTR[16+rsp],rsi
	mov	QWORD PTR[24+rsp],rdi


	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

	mov	r14,r8
	add	r8,QWORD PTR[48+rsi]
	mov	r15,r9
	adc	r9,QWORD PTR[56+rsi]
	mov	rax,r10
	adc	r10,QWORD PTR[64+rsi]
	mov	rdx,r11
	adc	r11,QWORD PTR[72+rsi]
	mov	rbx,r12
	adc	r12,QWORD PTR[80+rsi]
	mov	rbp,r13
	adc	r13,QWORD PTR[88+rsi]

	sub	r14,QWORD PTR[48+rsi]
	sbb	r15,QWORD PTR[56+rsi]
	sbb	rax,QWORD PTR[64+rsi]
	sbb	rdx,QWORD PTR[72+rsi]
	sbb	rbx,QWORD PTR[80+rsi]
	sbb	rbp,QWORD PTR[88+rsi]
	sbb	rdi,rdi

	mov	QWORD PTR[((32+0))+rsp],r8
	mov	QWORD PTR[((32+8))+rsp],r9
	mov	QWORD PTR[((32+16))+rsp],r10
	mov	QWORD PTR[((32+24))+rsp],r11
	mov	QWORD PTR[((32+32))+rsp],r12
	mov	QWORD PTR[((32+40))+rsp],r13

	mov	QWORD PTR[((32+48))+rsp],r14
	mov	QWORD PTR[((32+56))+rsp],r15
	mov	QWORD PTR[((32+64))+rsp],rax
	mov	QWORD PTR[((32+72))+rsp],rdx
	mov	QWORD PTR[((32+80))+rsp],rbx
	mov	QWORD PTR[((32+88))+rsp],rbp
	mov	QWORD PTR[((32+96))+rsp],rdi



	lea	rbx,QWORD PTR[48+rsi]

	mov	rax,QWORD PTR[48+rsi]
	mov	r14,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	r12,QWORD PTR[16+rsi]
	mov	r13,QWORD PTR[24+rsi]

	mov	rdi,QWORD PTR[24+rsp]
	call	__mulq_mont_383_nonred
	add	r14,r14
	adc	r15,r15
	adc	r8,r8
	adc	r9,r9
	adc	r10,r10
	adc	r11,r11

	mov	QWORD PTR[48+rdi],r14
	mov	QWORD PTR[56+rdi],r15
	mov	QWORD PTR[64+rdi],r8
	mov	QWORD PTR[72+rdi],r9
	mov	QWORD PTR[80+rdi],r10
	mov	QWORD PTR[88+rdi],r11

	lea	rsi,QWORD PTR[32+rsp]
	lea	rbx,QWORD PTR[((32+48))+rsp]

	mov	rax,QWORD PTR[((32+48))+rsp]
	mov	r14,QWORD PTR[((32+0))+rsp]
	mov	r15,QWORD PTR[((32+8))+rsp]
	mov	r12,QWORD PTR[((32+16))+rsp]
	mov	r13,QWORD PTR[((32+24))+rsp]

	call	__mulq_mont_383_nonred
	mov	rsi,QWORD PTR[((32+96))+rsp]
	mov	r12,QWORD PTR[((32+0))+rsp]
	mov	r13,QWORD PTR[((32+8))+rsp]
	and	r12,rsi
	mov	rax,QWORD PTR[((32+16))+rsp]
	and	r13,rsi
	mov	rbx,QWORD PTR[((32+24))+rsp]
	and	rax,rsi
	mov	rbp,QWORD PTR[((32+32))+rsp]
	and	rbx,rsi
	and	rbp,rsi
	and	rsi,QWORD PTR[((32+40))+rsp]

	sub	r14,r12
	mov	r12,QWORD PTR[rcx]
	sbb	r15,r13
	mov	r13,QWORD PTR[8+rcx]
	sbb	r8,rax
	mov	rax,QWORD PTR[16+rcx]
	sbb	r9,rbx
	mov	rbx,QWORD PTR[24+rcx]
	sbb	r10,rbp
	mov	rbp,QWORD PTR[32+rcx]
	sbb	r11,rsi
	sbb	rsi,rsi

	and	r12,rsi
	and	r13,rsi
	and	rax,rsi
	and	rbx,rsi
	and	rbp,rsi
	and	rsi,QWORD PTR[40+rcx]

	add	r14,r12
	adc	r15,r13
	adc	r8,rax
	adc	r9,rbx
	adc	r10,rbp
	adc	r11,rsi

	mov	QWORD PTR[rdi],r14
	mov	QWORD PTR[8+rdi],r15
	mov	QWORD PTR[16+rdi],r8
	mov	QWORD PTR[24+rdi],r9
	mov	QWORD PTR[32+rdi],r10
	mov	QWORD PTR[40+rdi],r11
	lea	r8,QWORD PTR[136+rsp]
	mov	r15,QWORD PTR[r8]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_sqr_mont_382x::
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

$L$SEH_end_sqr_mont_382x::
sqr_mont_382x	ENDP
.text$	ENDS
.pdata	SEGMENT READONLY ALIGN(4)
ALIGN	4
	DD	imagerel $L$SEH_begin_mul_mont_384x
	DD	imagerel $L$SEH_body_mul_mont_384x
	DD	imagerel $L$SEH_info_mul_mont_384x_prologue

	DD	imagerel $L$SEH_body_mul_mont_384x
	DD	imagerel $L$SEH_epilogue_mul_mont_384x
	DD	imagerel $L$SEH_info_mul_mont_384x_body

	DD	imagerel $L$SEH_epilogue_mul_mont_384x
	DD	imagerel $L$SEH_end_mul_mont_384x
	DD	imagerel $L$SEH_info_mul_mont_384x_epilogue

	DD	imagerel $L$SEH_begin_sqr_mont_384x
	DD	imagerel $L$SEH_body_sqr_mont_384x
	DD	imagerel $L$SEH_info_sqr_mont_384x_prologue

	DD	imagerel $L$SEH_body_sqr_mont_384x
	DD	imagerel $L$SEH_epilogue_sqr_mont_384x
	DD	imagerel $L$SEH_info_sqr_mont_384x_body

	DD	imagerel $L$SEH_epilogue_sqr_mont_384x
	DD	imagerel $L$SEH_end_sqr_mont_384x
	DD	imagerel $L$SEH_info_sqr_mont_384x_epilogue

	DD	imagerel $L$SEH_begin_mul_382x
	DD	imagerel $L$SEH_body_mul_382x
	DD	imagerel $L$SEH_info_mul_382x_prologue

	DD	imagerel $L$SEH_body_mul_382x
	DD	imagerel $L$SEH_epilogue_mul_382x
	DD	imagerel $L$SEH_info_mul_382x_body

	DD	imagerel $L$SEH_epilogue_mul_382x
	DD	imagerel $L$SEH_end_mul_382x
	DD	imagerel $L$SEH_info_mul_382x_epilogue

	DD	imagerel $L$SEH_begin_sqr_382x
	DD	imagerel $L$SEH_body_sqr_382x
	DD	imagerel $L$SEH_info_sqr_382x_prologue

	DD	imagerel $L$SEH_body_sqr_382x
	DD	imagerel $L$SEH_epilogue_sqr_382x
	DD	imagerel $L$SEH_info_sqr_382x_body

	DD	imagerel $L$SEH_epilogue_sqr_382x
	DD	imagerel $L$SEH_end_sqr_382x
	DD	imagerel $L$SEH_info_sqr_382x_epilogue

	DD	imagerel $L$SEH_begin_mul_384
	DD	imagerel $L$SEH_body_mul_384
	DD	imagerel $L$SEH_info_mul_384_prologue

	DD	imagerel $L$SEH_body_mul_384
	DD	imagerel $L$SEH_epilogue_mul_384
	DD	imagerel $L$SEH_info_mul_384_body

	DD	imagerel $L$SEH_epilogue_mul_384
	DD	imagerel $L$SEH_end_mul_384
	DD	imagerel $L$SEH_info_mul_384_epilogue

	DD	imagerel $L$SEH_begin_sqr_384
	DD	imagerel $L$SEH_body_sqr_384
	DD	imagerel $L$SEH_info_sqr_384_prologue

	DD	imagerel $L$SEH_body_sqr_384
	DD	imagerel $L$SEH_epilogue_sqr_384
	DD	imagerel $L$SEH_info_sqr_384_body

	DD	imagerel $L$SEH_epilogue_sqr_384
	DD	imagerel $L$SEH_end_sqr_384
	DD	imagerel $L$SEH_info_sqr_384_epilogue

	DD	imagerel $L$SEH_begin_sqr_mont_384
	DD	imagerel $L$SEH_body_sqr_mont_384
	DD	imagerel $L$SEH_info_sqr_mont_384_prologue

	DD	imagerel $L$SEH_body_sqr_mont_384
	DD	imagerel $L$SEH_epilogue_sqr_mont_384
	DD	imagerel $L$SEH_info_sqr_mont_384_body

	DD	imagerel $L$SEH_epilogue_sqr_mont_384
	DD	imagerel $L$SEH_end_sqr_mont_384
	DD	imagerel $L$SEH_info_sqr_mont_384_epilogue

	DD	imagerel $L$SEH_begin_redc_mont_384
	DD	imagerel $L$SEH_body_redc_mont_384
	DD	imagerel $L$SEH_info_redc_mont_384_prologue

	DD	imagerel $L$SEH_body_redc_mont_384
	DD	imagerel $L$SEH_epilogue_redc_mont_384
	DD	imagerel $L$SEH_info_redc_mont_384_body

	DD	imagerel $L$SEH_epilogue_redc_mont_384
	DD	imagerel $L$SEH_end_redc_mont_384
	DD	imagerel $L$SEH_info_redc_mont_384_epilogue

	DD	imagerel $L$SEH_begin_from_mont_384
	DD	imagerel $L$SEH_body_from_mont_384
	DD	imagerel $L$SEH_info_from_mont_384_prologue

	DD	imagerel $L$SEH_body_from_mont_384
	DD	imagerel $L$SEH_epilogue_from_mont_384
	DD	imagerel $L$SEH_info_from_mont_384_body

	DD	imagerel $L$SEH_epilogue_from_mont_384
	DD	imagerel $L$SEH_end_from_mont_384
	DD	imagerel $L$SEH_info_from_mont_384_epilogue

	DD	imagerel $L$SEH_begin_sgn0_pty_mont_384
	DD	imagerel $L$SEH_body_sgn0_pty_mont_384
	DD	imagerel $L$SEH_info_sgn0_pty_mont_384_prologue

	DD	imagerel $L$SEH_body_sgn0_pty_mont_384
	DD	imagerel $L$SEH_epilogue_sgn0_pty_mont_384
	DD	imagerel $L$SEH_info_sgn0_pty_mont_384_body

	DD	imagerel $L$SEH_epilogue_sgn0_pty_mont_384
	DD	imagerel $L$SEH_end_sgn0_pty_mont_384
	DD	imagerel $L$SEH_info_sgn0_pty_mont_384_epilogue

	DD	imagerel $L$SEH_begin_sgn0_pty_mont_384x
	DD	imagerel $L$SEH_body_sgn0_pty_mont_384x
	DD	imagerel $L$SEH_info_sgn0_pty_mont_384x_prologue

	DD	imagerel $L$SEH_body_sgn0_pty_mont_384x
	DD	imagerel $L$SEH_epilogue_sgn0_pty_mont_384x
	DD	imagerel $L$SEH_info_sgn0_pty_mont_384x_body

	DD	imagerel $L$SEH_epilogue_sgn0_pty_mont_384x
	DD	imagerel $L$SEH_end_sgn0_pty_mont_384x
	DD	imagerel $L$SEH_info_sgn0_pty_mont_384x_epilogue

	DD	imagerel $L$SEH_begin_mul_mont_384
	DD	imagerel $L$SEH_body_mul_mont_384
	DD	imagerel $L$SEH_info_mul_mont_384_prologue

	DD	imagerel $L$SEH_body_mul_mont_384
	DD	imagerel $L$SEH_epilogue_mul_mont_384
	DD	imagerel $L$SEH_info_mul_mont_384_body

	DD	imagerel $L$SEH_epilogue_mul_mont_384
	DD	imagerel $L$SEH_end_mul_mont_384
	DD	imagerel $L$SEH_info_mul_mont_384_epilogue

	DD	imagerel $L$SEH_begin_sqr_n_mul_mont_384
	DD	imagerel $L$SEH_body_sqr_n_mul_mont_384
	DD	imagerel $L$SEH_info_sqr_n_mul_mont_384_prologue

	DD	imagerel $L$SEH_body_sqr_n_mul_mont_384
	DD	imagerel $L$SEH_epilogue_sqr_n_mul_mont_384
	DD	imagerel $L$SEH_info_sqr_n_mul_mont_384_body

	DD	imagerel $L$SEH_epilogue_sqr_n_mul_mont_384
	DD	imagerel $L$SEH_end_sqr_n_mul_mont_384
	DD	imagerel $L$SEH_info_sqr_n_mul_mont_384_epilogue

	DD	imagerel $L$SEH_begin_sqr_n_mul_mont_383
	DD	imagerel $L$SEH_body_sqr_n_mul_mont_383
	DD	imagerel $L$SEH_info_sqr_n_mul_mont_383_prologue

	DD	imagerel $L$SEH_body_sqr_n_mul_mont_383
	DD	imagerel $L$SEH_epilogue_sqr_n_mul_mont_383
	DD	imagerel $L$SEH_info_sqr_n_mul_mont_383_body

	DD	imagerel $L$SEH_epilogue_sqr_n_mul_mont_383
	DD	imagerel $L$SEH_end_sqr_n_mul_mont_383
	DD	imagerel $L$SEH_info_sqr_n_mul_mont_383_epilogue

	DD	imagerel $L$SEH_begin_sqr_mont_382x
	DD	imagerel $L$SEH_body_sqr_mont_382x
	DD	imagerel $L$SEH_info_sqr_mont_382x_prologue

	DD	imagerel $L$SEH_body_sqr_mont_382x
	DD	imagerel $L$SEH_epilogue_sqr_mont_382x
	DD	imagerel $L$SEH_info_sqr_mont_382x_body

	DD	imagerel $L$SEH_epilogue_sqr_mont_382x
	DD	imagerel $L$SEH_end_sqr_mont_382x
	DD	imagerel $L$SEH_info_sqr_mont_382x_epilogue

.pdata	ENDS
.xdata	SEGMENT READONLY ALIGN(8)
ALIGN	8
$L$SEH_info_mul_mont_384x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_mul_mont_384x_body::
DB	1,0,18,0
DB	000h,0f4h,029h,000h
DB	000h,0e4h,02ah,000h
DB	000h,0d4h,02bh,000h
DB	000h,0c4h,02ch,000h
DB	000h,034h,02dh,000h
DB	000h,054h,02eh,000h
DB	000h,074h,030h,000h
DB	000h,064h,031h,000h
DB	000h,001h,02fh,000h
DB	000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_mul_mont_384x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqr_mont_384x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqr_mont_384x_body::
DB	1,0,18,0
DB	000h,0f4h,011h,000h
DB	000h,0e4h,012h,000h
DB	000h,0d4h,013h,000h
DB	000h,0c4h,014h,000h
DB	000h,034h,015h,000h
DB	000h,054h,016h,000h
DB	000h,074h,018h,000h
DB	000h,064h,019h,000h
DB	000h,001h,017h,000h
DB	000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sqr_mont_384x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_mul_382x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_mul_382x_body::
DB	1,0,18,0
DB	000h,0f4h,011h,000h
DB	000h,0e4h,012h,000h
DB	000h,0d4h,013h,000h
DB	000h,0c4h,014h,000h
DB	000h,034h,015h,000h
DB	000h,054h,016h,000h
DB	000h,074h,018h,000h
DB	000h,064h,019h,000h
DB	000h,001h,017h,000h
DB	000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_mul_382x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqr_382x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqr_382x_body::
DB	1,0,17,0
DB	000h,0f4h,001h,000h
DB	000h,0e4h,002h,000h
DB	000h,0d4h,003h,000h
DB	000h,0c4h,004h,000h
DB	000h,034h,005h,000h
DB	000h,054h,006h,000h
DB	000h,074h,008h,000h
DB	000h,064h,009h,000h
DB	000h,062h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sqr_382x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_mul_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_mul_384_body::
DB	1,0,11,0
DB	000h,0c4h,000h,000h
DB	000h,034h,001h,000h
DB	000h,054h,002h,000h
DB	000h,074h,004h,000h
DB	000h,064h,005h,000h
DB	000h,022h
DB	000h,000h,000h,000h,000h,000h
$L$SEH_info_mul_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqr_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqr_384_body::
DB	1,0,17,0
DB	000h,0f4h,001h,000h
DB	000h,0e4h,002h,000h
DB	000h,0d4h,003h,000h
DB	000h,0c4h,004h,000h
DB	000h,034h,005h,000h
DB	000h,054h,006h,000h
DB	000h,074h,008h,000h
DB	000h,064h,009h,000h
DB	000h,062h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sqr_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqr_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqr_mont_384_body::
DB	1,0,18,0
DB	000h,0f4h,00fh,000h
DB	000h,0e4h,010h,000h
DB	000h,0d4h,011h,000h
DB	000h,0c4h,012h,000h
DB	000h,034h,013h,000h
DB	000h,054h,014h,000h
DB	000h,074h,016h,000h
DB	000h,064h,017h,000h
DB	000h,001h,015h,000h
DB	000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sqr_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_redc_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_redc_mont_384_body::
DB	1,0,17,0
DB	000h,0f4h,001h,000h
DB	000h,0e4h,002h,000h
DB	000h,0d4h,003h,000h
DB	000h,0c4h,004h,000h
DB	000h,034h,005h,000h
DB	000h,054h,006h,000h
DB	000h,074h,008h,000h
DB	000h,064h,009h,000h
DB	000h,062h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_redc_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_from_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_from_mont_384_body::
DB	1,0,17,0
DB	000h,0f4h,001h,000h
DB	000h,0e4h,002h,000h
DB	000h,0d4h,003h,000h
DB	000h,0c4h,004h,000h
DB	000h,034h,005h,000h
DB	000h,054h,006h,000h
DB	000h,074h,008h,000h
DB	000h,064h,009h,000h
DB	000h,062h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_from_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sgn0_pty_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sgn0_pty_mont_384_body::
DB	1,0,17,0
DB	000h,0f4h,001h,000h
DB	000h,0e4h,002h,000h
DB	000h,0d4h,003h,000h
DB	000h,0c4h,004h,000h
DB	000h,034h,005h,000h
DB	000h,054h,006h,000h
DB	000h,074h,008h,000h
DB	000h,064h,009h,000h
DB	000h,062h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sgn0_pty_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sgn0_pty_mont_384x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sgn0_pty_mont_384x_body::
DB	1,0,17,0
DB	000h,0f4h,001h,000h
DB	000h,0e4h,002h,000h
DB	000h,0d4h,003h,000h
DB	000h,0c4h,004h,000h
DB	000h,034h,005h,000h
DB	000h,054h,006h,000h
DB	000h,074h,008h,000h
DB	000h,064h,009h,000h
DB	000h,062h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sgn0_pty_mont_384x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_mul_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_mul_mont_384_body::
DB	1,0,17,0
DB	000h,0f4h,003h,000h
DB	000h,0e4h,004h,000h
DB	000h,0d4h,005h,000h
DB	000h,0c4h,006h,000h
DB	000h,034h,007h,000h
DB	000h,054h,008h,000h
DB	000h,074h,00ah,000h
DB	000h,064h,00bh,000h
DB	000h,082h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_mul_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqr_n_mul_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqr_n_mul_mont_384_body::
DB	1,0,18,0
DB	000h,0f4h,011h,000h
DB	000h,0e4h,012h,000h
DB	000h,0d4h,013h,000h
DB	000h,0c4h,014h,000h
DB	000h,034h,015h,000h
DB	000h,054h,016h,000h
DB	000h,074h,018h,000h
DB	000h,064h,019h,000h
DB	000h,001h,017h,000h
DB	000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sqr_n_mul_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqr_n_mul_mont_383_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqr_n_mul_mont_383_body::
DB	1,0,18,0
DB	000h,0f4h,011h,000h
DB	000h,0e4h,012h,000h
DB	000h,0d4h,013h,000h
DB	000h,0c4h,014h,000h
DB	000h,034h,015h,000h
DB	000h,054h,016h,000h
DB	000h,074h,018h,000h
DB	000h,064h,019h,000h
DB	000h,001h,017h,000h
DB	000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sqr_n_mul_mont_383_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqr_mont_382x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqr_mont_382x_body::
DB	1,0,18,0
DB	000h,0f4h,011h,000h
DB	000h,0e4h,012h,000h
DB	000h,0d4h,013h,000h
DB	000h,0c4h,014h,000h
DB	000h,034h,015h,000h
DB	000h,054h,016h,000h
DB	000h,074h,018h,000h
DB	000h,064h,019h,000h
DB	000h,001h,017h,000h
DB	000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sqr_mont_382x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h


.xdata	ENDS
END
