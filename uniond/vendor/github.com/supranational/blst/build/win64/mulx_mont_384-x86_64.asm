OPTION	DOTNAME
PUBLIC	mul_mont_384x$1
PUBLIC	sqr_mont_384x$1
PUBLIC	mul_382x$1
PUBLIC	sqr_382x$1
PUBLIC	mul_384$1
PUBLIC	sqr_384$1
PUBLIC	redc_mont_384$1
PUBLIC	from_mont_384$1
PUBLIC	sgn0_pty_mont_384$1
PUBLIC	sgn0_pty_mont_384x$1
PUBLIC	mul_mont_384$1
PUBLIC	sqr_mont_384$1
PUBLIC	sqr_n_mul_mont_384$1
PUBLIC	sqr_n_mul_mont_383$1
PUBLIC	sqr_mont_382x$1
.text$	SEGMENT ALIGN(256) 'CODE'








ALIGN	32
__subx_mod_384x384	PROC PRIVATE
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
__subx_mod_384x384	ENDP


ALIGN	32
__addx_mod_384	PROC PRIVATE
	DB	243,15,30,250

ifdef	__SGX_LVI_HARDENING__
	lfence
endif
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
__addx_mod_384	ENDP


ALIGN	32
__subx_mod_384	PROC PRIVATE
	DB	243,15,30,250

ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]

__subx_mod_384_a_is_loaded::
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
__subx_mod_384	ENDP
PUBLIC	mulx_mont_384x


ALIGN	32
mulx_mont_384x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_mulx_mont_384x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	mov	r8,QWORD PTR[40+rsp]
mul_mont_384x$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,328

$L$SEH_body_mulx_mont_384x::


	mov	rbx,rdx
	mov	QWORD PTR[32+rsp],rdi
	mov	QWORD PTR[24+rsp],rsi
	mov	QWORD PTR[16+rsp],rdx
	mov	QWORD PTR[8+rsp],rcx
	mov	QWORD PTR[rsp],r8




	lea	rdi,QWORD PTR[40+rsp]
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__mulx_384


	lea	rbx,QWORD PTR[48+rbx]
	lea	rsi,QWORD PTR[((128+48))+rsi]
	lea	rdi,QWORD PTR[96+rdi]
	call	__mulx_384


	mov	rcx,QWORD PTR[8+rsp]
	lea	rsi,QWORD PTR[rbx]
	lea	rdx,QWORD PTR[((-48))+rbx]
	lea	rdi,QWORD PTR[((40+192+48))+rsp]
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__addx_mod_384

	mov	rsi,QWORD PTR[24+rsp]
	lea	rdx,QWORD PTR[48+rsi]
	lea	rdi,QWORD PTR[((-48))+rdi]
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__addx_mod_384

	lea	rbx,QWORD PTR[rdi]
	lea	rsi,QWORD PTR[48+rdi]
	call	__mulx_384


	lea	rsi,QWORD PTR[rdi]
	lea	rdx,QWORD PTR[40+rsp]
	mov	rcx,QWORD PTR[8+rsp]
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__subx_mod_384x384

	lea	rsi,QWORD PTR[rdi]
	lea	rdx,QWORD PTR[((-96))+rdi]
	call	__subx_mod_384x384


	lea	rsi,QWORD PTR[40+rsp]
	lea	rdx,QWORD PTR[((40+96))+rsp]
	lea	rdi,QWORD PTR[40+rsp]
	call	__subx_mod_384x384

	lea	rbx,QWORD PTR[rcx]


	lea	rsi,QWORD PTR[40+rsp]
	mov	rcx,QWORD PTR[rsp]
	mov	rdi,QWORD PTR[32+rsp]
	call	__mulx_by_1_mont_384
	call	__redx_tail_mont_384


	lea	rsi,QWORD PTR[((40+192))+rsp]
	mov	rcx,QWORD PTR[rsp]
	lea	rdi,QWORD PTR[48+rdi]
	call	__mulx_by_1_mont_384
	call	__redx_tail_mont_384

	lea	r8,QWORD PTR[328+rsp]
	mov	r15,QWORD PTR[r8]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_mulx_mont_384x::
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

$L$SEH_end_mulx_mont_384x::
mulx_mont_384x	ENDP
PUBLIC	sqrx_mont_384x


ALIGN	32
sqrx_mont_384x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqrx_mont_384x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
sqr_mont_384x$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,136

$L$SEH_body_sqrx_mont_384x::


	mov	QWORD PTR[rsp],rcx
	mov	rcx,rdx

	mov	QWORD PTR[16+rsp],rdi
	mov	QWORD PTR[24+rsp],rsi


	lea	rdx,QWORD PTR[48+rsi]
	lea	rdi,QWORD PTR[32+rsp]
	call	__addx_mod_384


	mov	rsi,QWORD PTR[24+rsp]
	lea	rdx,QWORD PTR[48+rsi]
	lea	rdi,QWORD PTR[((32+48))+rsp]
	call	__subx_mod_384


	mov	rsi,QWORD PTR[24+rsp]
	lea	rbx,QWORD PTR[48+rsi]

ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rdx,QWORD PTR[48+rsi]
	mov	r14,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	rax,QWORD PTR[16+rsi]
	mov	r12,QWORD PTR[24+rsi]
	mov	rdi,QWORD PTR[32+rsi]
	mov	rbp,QWORD PTR[40+rsi]
	lea	rsi,QWORD PTR[((-128))+rsi]
	lea	rcx,QWORD PTR[((-128))+rcx]

	mulx	r9,r8,r14
	call	__mulx_mont_384
	add	rdx,rdx
	adc	r15,r15
	adc	rax,rax
	mov	r8,rdx
	adc	r12,r12
	mov	r9,r15
	adc	rdi,rdi
	mov	r10,rax
	adc	rbp,rbp
	mov	r11,r12
	sbb	rsi,rsi

	sub	rdx,QWORD PTR[rcx]
	sbb	r15,QWORD PTR[8+rcx]
	mov	r13,rdi
	sbb	rax,QWORD PTR[16+rcx]
	sbb	r12,QWORD PTR[24+rcx]
	sbb	rdi,QWORD PTR[32+rcx]
	mov	r14,rbp
	sbb	rbp,QWORD PTR[40+rcx]
	sbb	rsi,0

	cmovc	rdx,r8
	cmovc	r15,r9
	cmovc	rax,r10
	mov	QWORD PTR[48+rbx],rdx
	cmovc	r12,r11
	mov	QWORD PTR[56+rbx],r15
	cmovc	rdi,r13
	mov	QWORD PTR[64+rbx],rax
	cmovc	rbp,r14
	mov	QWORD PTR[72+rbx],r12
	mov	QWORD PTR[80+rbx],rdi
	mov	QWORD PTR[88+rbx],rbp

	lea	rsi,QWORD PTR[32+rsp]
	lea	rbx,QWORD PTR[((32+48))+rsp]

	mov	rdx,QWORD PTR[((32+48))+rsp]
	mov	r14,QWORD PTR[((32+0))+rsp]
	mov	r15,QWORD PTR[((32+8))+rsp]
	mov	rax,QWORD PTR[((32+16))+rsp]
	mov	r12,QWORD PTR[((32+24))+rsp]
	mov	rdi,QWORD PTR[((32+32))+rsp]
	mov	rbp,QWORD PTR[((32+40))+rsp]
	lea	rsi,QWORD PTR[((-128))+rsi]
	lea	rcx,QWORD PTR[((-128))+rcx]

	mulx	r9,r8,r14
	call	__mulx_mont_384

	lea	r8,QWORD PTR[136+rsp]
	mov	r15,QWORD PTR[r8]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_sqrx_mont_384x::
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

$L$SEH_end_sqrx_mont_384x::
sqrx_mont_384x	ENDP

PUBLIC	mulx_382x


ALIGN	32
mulx_382x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_mulx_382x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
mul_382x$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,136

$L$SEH_body_mulx_382x::


	lea	rdi,QWORD PTR[96+rdi]
	mov	QWORD PTR[rsp],rsi
	mov	QWORD PTR[8+rsp],rdx
	mov	QWORD PTR[16+rsp],rdi
	mov	QWORD PTR[24+rsp],rcx


ifdef	__SGX_LVI_HARDENING__
	lfence
endif
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
	call	__mulx_384


	mov	rsi,QWORD PTR[rsp]
	mov	rbx,QWORD PTR[8+rsp]
	lea	rdi,QWORD PTR[((-96))+rdi]
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__mulx_384


	lea	rsi,QWORD PTR[((48+128))+rsi]
	lea	rbx,QWORD PTR[48+rbx]
	lea	rdi,QWORD PTR[32+rsp]
	call	__mulx_384


	mov	rsi,QWORD PTR[16+rsp]
	lea	rdx,QWORD PTR[32+rsp]
	mov	rcx,QWORD PTR[24+rsp]
	mov	rdi,rsi
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__subx_mod_384x384


	lea	rsi,QWORD PTR[rdi]
	lea	rdx,QWORD PTR[((-96))+rdi]
	call	__subx_mod_384x384


	lea	rsi,QWORD PTR[((-96))+rdi]
	lea	rdx,QWORD PTR[32+rsp]
	lea	rdi,QWORD PTR[((-96))+rdi]
	call	__subx_mod_384x384

	lea	r8,QWORD PTR[136+rsp]
	mov	r15,QWORD PTR[r8]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_mulx_382x::
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

$L$SEH_end_mulx_382x::
mulx_382x	ENDP
PUBLIC	sqrx_382x


ALIGN	32
sqrx_382x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqrx_382x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
sqr_382x$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	push	rsi

$L$SEH_body_sqrx_382x::


	mov	rcx,rdx


ifdef	__SGX_LVI_HARDENING__
	lfence
endif
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
	call	__subx_mod_384_a_is_loaded


	lea	rsi,QWORD PTR[rdi]
	lea	rbx,QWORD PTR[((-48))+rdi]
	lea	rdi,QWORD PTR[((-48))+rdi]
	call	__mulx_384


	mov	rsi,QWORD PTR[rsp]
	lea	rbx,QWORD PTR[48+rsi]
	lea	rdi,QWORD PTR[96+rdi]
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__mulx_384

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

$L$SEH_epilogue_sqrx_382x::
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

$L$SEH_end_sqrx_382x::
sqrx_382x	ENDP
PUBLIC	mulx_384


ALIGN	32
mulx_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_mulx_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
mul_384$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

$L$SEH_body_mulx_384::


	mov	rbx,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__mulx_384

	mov	r15,QWORD PTR[rsp]

	mov	r14,QWORD PTR[8+rsp]

	mov	r13,QWORD PTR[16+rsp]

	mov	r12,QWORD PTR[24+rsp]

	mov	rbx,QWORD PTR[32+rsp]

	mov	rbp,QWORD PTR[40+rsp]

	lea	rsp,QWORD PTR[48+rsp]

$L$SEH_epilogue_mulx_384::
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

$L$SEH_end_mulx_384::
mulx_384	ENDP


ALIGN	32
__mulx_384	PROC PRIVATE
	DB	243,15,30,250

	mov	rdx,QWORD PTR[rbx]
	mov	r14,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]
	lea	rsi,QWORD PTR[((-128))+rsi]

	mulx	rcx,r9,r14
	xor	rbp,rbp

	mulx	rax,r8,r15
	adcx	r8,rcx
	mov	QWORD PTR[rdi],r9

	mulx	rcx,r9,r10
	adcx	r9,rax

	mulx	rax,r10,r11
	adcx	r10,rcx

	mulx	rcx,r11,r12
	adcx	r11,rax

	mulx	r13,r12,r13
	mov	rdx,QWORD PTR[8+rbx]
	adcx	r12,rcx
	adcx	r13,rbp
	mulx	rcx,rax,r14
	adcx	rax,r8
	adox	r9,rcx
	mov	QWORD PTR[8+rdi],rax

	mulx	rcx,r8,r15
	adcx	r8,r9
	adox	r10,rcx

	mulx	rax,r9,QWORD PTR[((128+16))+rsi]
	adcx	r9,r10
	adox	r11,rax

	mulx	rcx,r10,QWORD PTR[((128+24))+rsi]
	adcx	r10,r11
	adox	r12,rcx

	mulx	rax,r11,QWORD PTR[((128+32))+rsi]
	adcx	r11,r12
	adox	rax,r13

	mulx	r13,r12,QWORD PTR[((128+40))+rsi]
	mov	rdx,QWORD PTR[16+rbx]
	adcx	r12,rax
	adox	r13,rbp
	adcx	r13,rbp
	mulx	rcx,rax,r14
	adcx	rax,r8
	adox	r9,rcx
	mov	QWORD PTR[16+rdi],rax

	mulx	rcx,r8,r15
	adcx	r8,r9
	adox	r10,rcx

	mulx	rax,r9,QWORD PTR[((128+16))+rsi]
	adcx	r9,r10
	adox	r11,rax

	mulx	rcx,r10,QWORD PTR[((128+24))+rsi]
	adcx	r10,r11
	adox	r12,rcx

	mulx	rax,r11,QWORD PTR[((128+32))+rsi]
	adcx	r11,r12
	adox	rax,r13

	mulx	r13,r12,QWORD PTR[((128+40))+rsi]
	mov	rdx,QWORD PTR[24+rbx]
	adcx	r12,rax
	adox	r13,rbp
	adcx	r13,rbp
	mulx	rcx,rax,r14
	adcx	rax,r8
	adox	r9,rcx
	mov	QWORD PTR[24+rdi],rax

	mulx	rcx,r8,r15
	adcx	r8,r9
	adox	r10,rcx

	mulx	rax,r9,QWORD PTR[((128+16))+rsi]
	adcx	r9,r10
	adox	r11,rax

	mulx	rcx,r10,QWORD PTR[((128+24))+rsi]
	adcx	r10,r11
	adox	r12,rcx

	mulx	rax,r11,QWORD PTR[((128+32))+rsi]
	adcx	r11,r12
	adox	rax,r13

	mulx	r13,r12,QWORD PTR[((128+40))+rsi]
	mov	rdx,QWORD PTR[32+rbx]
	adcx	r12,rax
	adox	r13,rbp
	adcx	r13,rbp
	mulx	rcx,rax,r14
	adcx	rax,r8
	adox	r9,rcx
	mov	QWORD PTR[32+rdi],rax

	mulx	rcx,r8,r15
	adcx	r8,r9
	adox	r10,rcx

	mulx	rax,r9,QWORD PTR[((128+16))+rsi]
	adcx	r9,r10
	adox	r11,rax

	mulx	rcx,r10,QWORD PTR[((128+24))+rsi]
	adcx	r10,r11
	adox	r12,rcx

	mulx	rax,r11,QWORD PTR[((128+32))+rsi]
	adcx	r11,r12
	adox	rax,r13

	mulx	r13,r12,QWORD PTR[((128+40))+rsi]
	mov	rdx,QWORD PTR[40+rbx]
	adcx	r12,rax
	adox	r13,rbp
	adcx	r13,rbp
	mulx	rcx,rax,r14
	adcx	rax,r8
	adox	r9,rcx
	mov	QWORD PTR[40+rdi],rax

	mulx	rcx,r8,r15
	adcx	r8,r9
	adox	r10,rcx

	mulx	rax,r9,QWORD PTR[((128+16))+rsi]
	adcx	r9,r10
	adox	r11,rax

	mulx	rcx,r10,QWORD PTR[((128+24))+rsi]
	adcx	r10,r11
	adox	r12,rcx

	mulx	rax,r11,QWORD PTR[((128+32))+rsi]
	adcx	r11,r12
	adox	rax,r13

	mulx	r13,r12,QWORD PTR[((128+40))+rsi]
	mov	rdx,rax
	adcx	r12,rax
	adox	r13,rbp
	adcx	r13,rbp
	mov	QWORD PTR[48+rdi],r8
	mov	QWORD PTR[56+rdi],r9
	mov	QWORD PTR[64+rdi],r10
	mov	QWORD PTR[72+rdi],r11
	mov	QWORD PTR[80+rdi],r12
	mov	QWORD PTR[88+rdi],r13

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__mulx_384	ENDP
PUBLIC	sqrx_384


ALIGN	32
sqrx_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqrx_384::


	mov	rdi,rcx
	mov	rsi,rdx
sqr_384$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	push	rdi

$L$SEH_body_sqrx_384::


ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__sqrx_384

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_sqrx_384::
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

$L$SEH_end_sqrx_384::
sqrx_384	ENDP

ALIGN	32
__sqrx_384	PROC PRIVATE
	DB	243,15,30,250

	mov	rdx,QWORD PTR[rsi]
	mov	r14,QWORD PTR[8+rsi]
	mov	r15,QWORD PTR[16+rsi]
	mov	rcx,QWORD PTR[24+rsi]
	mov	rbx,QWORD PTR[32+rsi]


	mulx	rdi,r8,r14
	mov	rbp,QWORD PTR[40+rsi]
	mulx	rax,r9,r15
	add	r9,rdi
	mulx	rdi,r10,rcx
	adc	r10,rax
	mulx	rax,r11,rbx
	adc	r11,rdi
	mulx	r13,r12,rbp
	mov	rdx,r14
	adc	r12,rax
	adc	r13,0


	xor	r14,r14
	mulx	rax,rdi,r15
	adcx	r10,rdi
	adox	r11,rax

	mulx	rax,rdi,rcx
	adcx	r11,rdi
	adox	r12,rax

	mulx	rax,rdi,rbx
	adcx	r12,rdi
	adox	r13,rax

	mulx	rax,rdi,rbp
	mov	rdx,r15
	adcx	r13,rdi
	adox	rax,r14
	adcx	r14,rax


	xor	r15,r15
	mulx	rax,rdi,rcx
	adcx	r12,rdi
	adox	r13,rax

	mulx	rax,rdi,rbx
	adcx	r13,rdi
	adox	r14,rax

	mulx	rax,rdi,rbp
	mov	rdx,rcx
	adcx	r14,rdi
	adox	rax,r15
	adcx	r15,rax


	xor	rcx,rcx
	mulx	rax,rdi,rbx
	adcx	r14,rdi
	adox	r15,rax

	mulx	rax,rdi,rbp
	mov	rdx,rbx
	adcx	r15,rdi
	adox	rax,rcx
	adcx	rcx,rax


	mulx	rbx,rdi,rbp
	mov	rdx,QWORD PTR[rsi]
	add	rcx,rdi
	mov	rdi,QWORD PTR[8+rsp]
	adc	rbx,0


	xor	rbp,rbp
	adcx	r8,r8
	adcx	r9,r9
	adcx	r10,r10
	adcx	r11,r11
	adcx	r12,r12


	mulx	rax,rdx,rdx
	mov	QWORD PTR[rdi],rdx
	mov	rdx,QWORD PTR[8+rsi]
	adox	r8,rax
	mov	QWORD PTR[8+rdi],r8

	mulx	rax,r8,rdx
	mov	rdx,QWORD PTR[16+rsi]
	adox	r9,r8
	adox	r10,rax
	mov	QWORD PTR[16+rdi],r9
	mov	QWORD PTR[24+rdi],r10

	mulx	r9,r8,rdx
	mov	rdx,QWORD PTR[24+rsi]
	adox	r11,r8
	adox	r12,r9
	adcx	r13,r13
	adcx	r14,r14
	mov	QWORD PTR[32+rdi],r11
	mov	QWORD PTR[40+rdi],r12

	mulx	r9,r8,rdx
	mov	rdx,QWORD PTR[32+rsi]
	adox	r13,r8
	adox	r14,r9
	adcx	r15,r15
	adcx	rcx,rcx
	mov	QWORD PTR[48+rdi],r13
	mov	QWORD PTR[56+rdi],r14

	mulx	r9,r8,rdx
	mov	rdx,QWORD PTR[40+rsi]
	adox	r15,r8
	adox	rcx,r9
	adcx	rbx,rbx
	adcx	rbp,rbp
	mov	QWORD PTR[64+rdi],r15
	mov	QWORD PTR[72+rdi],rcx

	mulx	r9,r8,rdx
	adox	rbx,r8
	adox	rbp,r9

	mov	QWORD PTR[80+rdi],rbx
	mov	QWORD PTR[88+rdi],rbp

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__sqrx_384	ENDP



PUBLIC	redcx_mont_384


ALIGN	32
redcx_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_redcx_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
redc_mont_384$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_redcx_mont_384::


	mov	rbx,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__mulx_by_1_mont_384
	call	__redx_tail_mont_384

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_redcx_mont_384::
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

$L$SEH_end_redcx_mont_384::
redcx_mont_384	ENDP




PUBLIC	fromx_mont_384


ALIGN	32
fromx_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_fromx_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
from_mont_384$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_fromx_mont_384::


	mov	rbx,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__mulx_by_1_mont_384




	mov	rax,r14
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

$L$SEH_epilogue_fromx_mont_384::
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

$L$SEH_end_fromx_mont_384::
fromx_mont_384	ENDP

ALIGN	32
__mulx_by_1_mont_384	PROC PRIVATE
	DB	243,15,30,250

	mov	r8,QWORD PTR[rsi]
	mov	rdx,rcx
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]
	mov	r12,QWORD PTR[32+rsi]
	mov	r13,QWORD PTR[40+rsi]
	imul	rdx,r8


	xor	r14,r14
	mulx	rbp,rax,QWORD PTR[rbx]
	adcx	r8,rax
	adox	r9,rbp

	mulx	rbp,rax,QWORD PTR[8+rbx]
	adcx	r9,rax
	adox	r10,rbp

	mulx	rbp,rax,QWORD PTR[16+rbx]
	adcx	r10,rax
	adox	r11,rbp

	mulx	rbp,rax,QWORD PTR[24+rbx]
	adcx	r11,rax
	adox	r12,rbp

	mulx	rbp,rax,QWORD PTR[32+rbx]
	adcx	r12,rax
	adox	r13,rbp

	mulx	rbp,rax,QWORD PTR[40+rbx]
	mov	rdx,rcx
	adcx	r13,rax
	adox	rbp,r14
	adcx	r14,rbp
	imul	rdx,r9


	xor	r15,r15
	mulx	rbp,rax,QWORD PTR[rbx]
	adcx	r9,rax
	adox	r10,rbp

	mulx	rbp,rax,QWORD PTR[8+rbx]
	adcx	r10,rax
	adox	r11,rbp

	mulx	rbp,rax,QWORD PTR[16+rbx]
	adcx	r11,rax
	adox	r12,rbp

	mulx	rbp,rax,QWORD PTR[24+rbx]
	adcx	r12,rax
	adox	r13,rbp

	mulx	rbp,rax,QWORD PTR[32+rbx]
	adcx	r13,rax
	adox	r14,rbp

	mulx	rbp,rax,QWORD PTR[40+rbx]
	mov	rdx,rcx
	adcx	r14,rax
	adox	rbp,r15
	adcx	r15,rbp
	imul	rdx,r10


	xor	r8,r8
	mulx	rbp,rax,QWORD PTR[rbx]
	adcx	r10,rax
	adox	r11,rbp

	mulx	rbp,rax,QWORD PTR[8+rbx]
	adcx	r11,rax
	adox	r12,rbp

	mulx	rbp,rax,QWORD PTR[16+rbx]
	adcx	r12,rax
	adox	r13,rbp

	mulx	rbp,rax,QWORD PTR[24+rbx]
	adcx	r13,rax
	adox	r14,rbp

	mulx	rbp,rax,QWORD PTR[32+rbx]
	adcx	r14,rax
	adox	r15,rbp

	mulx	rbp,rax,QWORD PTR[40+rbx]
	mov	rdx,rcx
	adcx	r15,rax
	adox	rbp,r8
	adcx	r8,rbp
	imul	rdx,r11


	xor	r9,r9
	mulx	rbp,rax,QWORD PTR[rbx]
	adcx	r11,rax
	adox	r12,rbp

	mulx	rbp,rax,QWORD PTR[8+rbx]
	adcx	r12,rax
	adox	r13,rbp

	mulx	rbp,rax,QWORD PTR[16+rbx]
	adcx	r13,rax
	adox	r14,rbp

	mulx	rbp,rax,QWORD PTR[24+rbx]
	adcx	r14,rax
	adox	r15,rbp

	mulx	rbp,rax,QWORD PTR[32+rbx]
	adcx	r15,rax
	adox	r8,rbp

	mulx	rbp,rax,QWORD PTR[40+rbx]
	mov	rdx,rcx
	adcx	r8,rax
	adox	rbp,r9
	adcx	r9,rbp
	imul	rdx,r12


	xor	r10,r10
	mulx	rbp,rax,QWORD PTR[rbx]
	adcx	r12,rax
	adox	r13,rbp

	mulx	rbp,rax,QWORD PTR[8+rbx]
	adcx	r13,rax
	adox	r14,rbp

	mulx	rbp,rax,QWORD PTR[16+rbx]
	adcx	r14,rax
	adox	r15,rbp

	mulx	rbp,rax,QWORD PTR[24+rbx]
	adcx	r15,rax
	adox	r8,rbp

	mulx	rbp,rax,QWORD PTR[32+rbx]
	adcx	r8,rax
	adox	r9,rbp

	mulx	rbp,rax,QWORD PTR[40+rbx]
	mov	rdx,rcx
	adcx	r9,rax
	adox	rbp,r10
	adcx	r10,rbp
	imul	rdx,r13


	xor	r11,r11
	mulx	rbp,rax,QWORD PTR[rbx]
	adcx	r13,rax
	adox	r14,rbp

	mulx	rbp,rax,QWORD PTR[8+rbx]
	adcx	r14,rax
	adox	r15,rbp

	mulx	rbp,rax,QWORD PTR[16+rbx]
	adcx	r15,rax
	adox	r8,rbp

	mulx	rbp,rax,QWORD PTR[24+rbx]
	adcx	r8,rax
	adox	r9,rbp

	mulx	rbp,rax,QWORD PTR[32+rbx]
	adcx	r9,rax
	adox	r10,rbp

	mulx	rbp,rax,QWORD PTR[40+rbx]
	mov	rdx,rcx
	adcx	r10,rax
	adox	rbp,r11
	adcx	r11,rbp
	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__mulx_by_1_mont_384	ENDP


ALIGN	32
__redx_tail_mont_384	PROC PRIVATE
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
__redx_tail_mont_384	ENDP

PUBLIC	sgn0x_pty_mont_384


ALIGN	32
sgn0x_pty_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sgn0x_pty_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
sgn0_pty_mont_384$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_sgn0x_pty_mont_384::


	mov	rbx,rsi
	lea	rsi,QWORD PTR[rdi]
	mov	rcx,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__mulx_by_1_mont_384

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

$L$SEH_epilogue_sgn0x_pty_mont_384::
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

$L$SEH_end_sgn0x_pty_mont_384::
sgn0x_pty_mont_384	ENDP

PUBLIC	sgn0x_pty_mont_384x


ALIGN	32
sgn0x_pty_mont_384x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sgn0x_pty_mont_384x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
sgn0_pty_mont_384x$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_sgn0x_pty_mont_384x::


	mov	rbx,rsi
	lea	rsi,QWORD PTR[48+rdi]
	mov	rcx,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	call	__mulx_by_1_mont_384

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

	call	__mulx_by_1_mont_384

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

$L$SEH_epilogue_sgn0x_pty_mont_384x::
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

$L$SEH_end_sgn0x_pty_mont_384x::
sgn0x_pty_mont_384x	ENDP
PUBLIC	mulx_mont_384


ALIGN	32
mulx_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_mulx_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	mov	r8,QWORD PTR[40+rsp]
mul_mont_384$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	lea	rsp,QWORD PTR[((-24))+rsp]

$L$SEH_body_mulx_mont_384::


	mov	rbx,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rdx,QWORD PTR[rdx]
	mov	r14,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	rax,QWORD PTR[16+rsi]
	mov	r12,QWORD PTR[24+rsi]
	mov	QWORD PTR[16+rsp],rdi
	mov	rdi,QWORD PTR[32+rsi]
	mov	rbp,QWORD PTR[40+rsi]
	lea	rsi,QWORD PTR[((-128))+rsi]
	lea	rcx,QWORD PTR[((-128))+rcx]
	mov	QWORD PTR[rsp],r8

	mulx	r9,r8,r14
	call	__mulx_mont_384

	mov	r15,QWORD PTR[24+rsp]

	mov	r14,QWORD PTR[32+rsp]

	mov	r13,QWORD PTR[40+rsp]

	mov	r12,QWORD PTR[48+rsp]

	mov	rbx,QWORD PTR[56+rsp]

	mov	rbp,QWORD PTR[64+rsp]

	lea	rsp,QWORD PTR[72+rsp]

$L$SEH_epilogue_mulx_mont_384::
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

$L$SEH_end_mulx_mont_384::
mulx_mont_384	ENDP

ALIGN	32
__mulx_mont_384	PROC PRIVATE
	DB	243,15,30,250


	mulx	r10,r14,r15
	mulx	r11,r15,rax
	add	r9,r14
	mulx	r12,rax,r12
	adc	r10,r15
	mulx	r13,rdi,rdi
	adc	r11,rax
	mulx	r14,rbp,rbp
	mov	rdx,QWORD PTR[8+rbx]
	adc	r12,rdi
	adc	r13,rbp
	adc	r14,0
	xor	r15,r15

	mov	QWORD PTR[16+rsp],r8
	imul	r8,QWORD PTR[8+rsp]


	xor	rax,rax
	mulx	rbp,rdi,QWORD PTR[((0+128))+rsi]
	adox	r9,rdi
	adcx	r10,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rsi]
	adox	r10,rdi
	adcx	r11,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rsi]
	adox	r11,rdi
	adcx	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rsi]
	adox	r12,rdi
	adcx	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rsi]
	adox	r13,rdi
	adcx	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rsi]
	mov	rdx,r8
	adox	r14,rdi
	adcx	r15,rbp
	adox	r15,rax
	adox	rax,rax


	xor	r8,r8
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	rdi,QWORD PTR[16+rsp]
	adox	r9,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r9,rdi
	adox	r10,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r10,rdi
	adox	r11,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	r11,rdi
	adox	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	r12,rdi
	adox	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	mov	rdx,QWORD PTR[16+rbx]
	adcx	r13,rdi
	adox	r14,rbp
	adcx	r14,r8
	adox	r15,r8
	adcx	r15,r8
	adox	rax,r8
	adcx	rax,r8
	mov	QWORD PTR[16+rsp],r9
	imul	r9,QWORD PTR[8+rsp]


	xor	r8,r8
	mulx	rbp,rdi,QWORD PTR[((0+128))+rsi]
	adox	r10,rdi
	adcx	r11,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rsi]
	adox	r11,rdi
	adcx	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rsi]
	adox	r12,rdi
	adcx	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rsi]
	adox	r13,rdi
	adcx	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rsi]
	adox	r14,rdi
	adcx	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rsi]
	mov	rdx,r9
	adox	r15,rdi
	adcx	rax,rbp
	adox	rax,r8
	adox	r8,r8


	xor	r9,r9
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	rdi,QWORD PTR[16+rsp]
	adox	r10,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r10,rdi
	adox	r11,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r11,rdi
	adox	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	r12,rdi
	adox	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	r13,rdi
	adox	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	mov	rdx,QWORD PTR[24+rbx]
	adcx	r14,rdi
	adox	r15,rbp
	adcx	r15,r9
	adox	rax,r9
	adcx	rax,r9
	adox	r8,r9
	adcx	r8,r9
	mov	QWORD PTR[16+rsp],r10
	imul	r10,QWORD PTR[8+rsp]


	xor	r9,r9
	mulx	rbp,rdi,QWORD PTR[((0+128))+rsi]
	adox	r11,rdi
	adcx	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rsi]
	adox	r12,rdi
	adcx	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rsi]
	adox	r13,rdi
	adcx	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rsi]
	adox	r14,rdi
	adcx	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rsi]
	adox	r15,rdi
	adcx	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rsi]
	mov	rdx,r10
	adox	rax,rdi
	adcx	r8,rbp
	adox	r8,r9
	adox	r9,r9


	xor	r10,r10
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	rdi,QWORD PTR[16+rsp]
	adox	r11,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r11,rdi
	adox	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r12,rdi
	adox	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	r13,rdi
	adox	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	r14,rdi
	adox	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	mov	rdx,QWORD PTR[32+rbx]
	adcx	r15,rdi
	adox	rax,rbp
	adcx	rax,r10
	adox	r8,r10
	adcx	r8,r10
	adox	r9,r10
	adcx	r9,r10
	mov	QWORD PTR[16+rsp],r11
	imul	r11,QWORD PTR[8+rsp]


	xor	r10,r10
	mulx	rbp,rdi,QWORD PTR[((0+128))+rsi]
	adox	r12,rdi
	adcx	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rsi]
	adox	r13,rdi
	adcx	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rsi]
	adox	r14,rdi
	adcx	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rsi]
	adox	r15,rdi
	adcx	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rsi]
	adox	rax,rdi
	adcx	r8,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rsi]
	mov	rdx,r11
	adox	r8,rdi
	adcx	r9,rbp
	adox	r9,r10
	adox	r10,r10


	xor	r11,r11
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	rdi,QWORD PTR[16+rsp]
	adox	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r12,rdi
	adox	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r13,rdi
	adox	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	r14,rdi
	adox	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	r15,rdi
	adox	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	mov	rdx,QWORD PTR[40+rbx]
	adcx	rax,rdi
	adox	r8,rbp
	adcx	r8,r11
	adox	r9,r11
	adcx	r9,r11
	adox	r10,r11
	adcx	r10,r11
	mov	QWORD PTR[16+rsp],r12
	imul	r12,QWORD PTR[8+rsp]


	xor	r11,r11
	mulx	rbp,rdi,QWORD PTR[((0+128))+rsi]
	adox	r13,rdi
	adcx	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rsi]
	adox	r14,rdi
	adcx	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rsi]
	adox	r15,rdi
	adcx	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rsi]
	adox	rax,rdi
	adcx	r8,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rsi]
	adox	r8,rdi
	adcx	r9,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rsi]
	mov	rdx,r12
	adox	r9,rdi
	adcx	r10,rbp
	adox	r10,r11
	adox	r11,r11


	xor	r12,r12
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	rdi,QWORD PTR[16+rsp]
	adox	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r13,rdi
	adox	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r14,rdi
	adox	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	r15,rdi
	adox	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	rax,rdi
	adox	r8,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	mov	rdx,r13
	adcx	r8,rdi
	adox	r9,rbp
	adcx	r9,r12
	adox	r10,r12
	adcx	r10,r12
	adox	r11,r12
	adcx	r11,r12
	imul	rdx,QWORD PTR[8+rsp]
	mov	rbx,QWORD PTR[24+rsp]


	xor	r12,r12
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	r13,rdi
	adox	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r14,rdi
	adox	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r15,rdi
	adox	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	rax,rdi
	adox	r8,rbp
	mov	r13,r15

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	r8,rdi
	adox	r9,rbp
	mov	rsi,rax

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	adcx	r9,rdi
	adox	r10,rbp
	mov	rdx,r14
	adcx	r10,r12
	adox	r11,r12
	lea	rcx,QWORD PTR[128+rcx]
	mov	r12,r8
	adc	r11,0




	sub	r14,QWORD PTR[rcx]
	sbb	r15,QWORD PTR[8+rcx]
	mov	rdi,r9
	sbb	rax,QWORD PTR[16+rcx]
	sbb	r8,QWORD PTR[24+rcx]
	sbb	r9,QWORD PTR[32+rcx]
	mov	rbp,r10
	sbb	r10,QWORD PTR[40+rcx]
	sbb	r11,0

	cmovnc	rdx,r14
	cmovc	r15,r13
	cmovc	rax,rsi
	cmovnc	r12,r8
	mov	QWORD PTR[rbx],rdx
	cmovnc	rdi,r9
	mov	QWORD PTR[8+rbx],r15
	cmovnc	rbp,r10
	mov	QWORD PTR[16+rbx],rax
	mov	QWORD PTR[24+rbx],r12
	mov	QWORD PTR[32+rbx],rdi
	mov	QWORD PTR[40+rbx],rbp

	
ifdef	__SGX_LVI_HARDENING__
	pop	rsi
	lfence
	jmp	rsi
	ud2
else
	DB	0F3h,0C3h
endif

__mulx_mont_384	ENDP
PUBLIC	sqrx_mont_384


ALIGN	32
sqrx_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqrx_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
sqr_mont_384$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	lea	rsp,QWORD PTR[((-24))+rsp]

$L$SEH_body_sqrx_mont_384::


	mov	r8,rcx
	lea	rcx,QWORD PTR[((-128))+rdx]
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rdx,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	rax,QWORD PTR[16+rsi]
	mov	r12,QWORD PTR[24+rsi]
	mov	QWORD PTR[16+rsp],rdi
	mov	rdi,QWORD PTR[32+rsi]
	mov	rbp,QWORD PTR[40+rsi]

	lea	rbx,QWORD PTR[rsi]
	mov	QWORD PTR[rsp],r8
	lea	rsi,QWORD PTR[((-128))+rsi]

	mulx	r9,r8,rdx
	call	__mulx_mont_384

	mov	r15,QWORD PTR[24+rsp]

	mov	r14,QWORD PTR[32+rsp]

	mov	r13,QWORD PTR[40+rsp]

	mov	r12,QWORD PTR[48+rsp]

	mov	rbx,QWORD PTR[56+rsp]

	mov	rbp,QWORD PTR[64+rsp]

	lea	rsp,QWORD PTR[72+rsp]

$L$SEH_epilogue_sqrx_mont_384::
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

$L$SEH_end_sqrx_mont_384::
sqrx_mont_384	ENDP

PUBLIC	sqrx_n_mul_mont_384


ALIGN	32
sqrx_n_mul_mont_384	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqrx_n_mul_mont_384::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	mov	r8,QWORD PTR[40+rsp]
	mov	r9,QWORD PTR[48+rsp]
sqr_n_mul_mont_384$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	lea	rsp,QWORD PTR[((-40))+rsp]

$L$SEH_body_sqrx_n_mul_mont_384::


	mov	r10,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rdx,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	rax,QWORD PTR[16+rsi]
	mov	rbx,rsi
	mov	r12,QWORD PTR[24+rsi]
	mov	QWORD PTR[16+rsp],rdi
	mov	rdi,QWORD PTR[32+rsi]
	mov	rbp,QWORD PTR[40+rsi]

	mov	QWORD PTR[rsp],r8
	mov	QWORD PTR[24+rsp],r9
	movq	xmm2,QWORD PTR[r9]

$L$oop_sqrx_384::
	movd	xmm1,r10d
	lea	rsi,QWORD PTR[((-128))+rbx]
	lea	rcx,QWORD PTR[((-128))+rcx]

	mulx	r9,r8,rdx
	call	__mulx_mont_384

	movd	r10d,xmm1
	dec	r10d
	jnz	$L$oop_sqrx_384

	mov	r14,rdx
DB	102,72,15,126,210
	lea	rsi,QWORD PTR[((-128))+rbx]
	mov	rbx,QWORD PTR[24+rsp]
	lea	rcx,QWORD PTR[((-128))+rcx]

	mulx	r9,r8,r14
	call	__mulx_mont_384

	mov	r15,QWORD PTR[40+rsp]

	mov	r14,QWORD PTR[48+rsp]

	mov	r13,QWORD PTR[56+rsp]

	mov	r12,QWORD PTR[64+rsp]

	mov	rbx,QWORD PTR[72+rsp]

	mov	rbp,QWORD PTR[80+rsp]

	lea	rsp,QWORD PTR[88+rsp]

$L$SEH_epilogue_sqrx_n_mul_mont_384::
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

$L$SEH_end_sqrx_n_mul_mont_384::
sqrx_n_mul_mont_384	ENDP

PUBLIC	sqrx_n_mul_mont_383


ALIGN	32
sqrx_n_mul_mont_383	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqrx_n_mul_mont_383::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	mov	r8,QWORD PTR[40+rsp]
	mov	r9,QWORD PTR[48+rsp]
sqr_n_mul_mont_383$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	lea	rsp,QWORD PTR[((-40))+rsp]

$L$SEH_body_sqrx_n_mul_mont_383::


	mov	r10,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rdx,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	rax,QWORD PTR[16+rsi]
	mov	rbx,rsi
	mov	r12,QWORD PTR[24+rsi]
	mov	QWORD PTR[16+rsp],rdi
	mov	rdi,QWORD PTR[32+rsi]
	mov	rbp,QWORD PTR[40+rsi]

	mov	QWORD PTR[rsp],r8
	mov	QWORD PTR[24+rsp],r9
	movq	xmm2,QWORD PTR[r9]
	lea	rcx,QWORD PTR[((-128))+rcx]

$L$oop_sqrx_383::
	movd	xmm1,r10d
	lea	rsi,QWORD PTR[((-128))+rbx]

	mulx	r9,r8,rdx
	call	__mulx_mont_383_nonred

	movd	r10d,xmm1
	dec	r10d
	jnz	$L$oop_sqrx_383

	mov	r14,rdx
DB	102,72,15,126,210
	lea	rsi,QWORD PTR[((-128))+rbx]
	mov	rbx,QWORD PTR[24+rsp]

	mulx	r9,r8,r14
	call	__mulx_mont_384

	mov	r15,QWORD PTR[40+rsp]

	mov	r14,QWORD PTR[48+rsp]

	mov	r13,QWORD PTR[56+rsp]

	mov	r12,QWORD PTR[64+rsp]

	mov	rbx,QWORD PTR[72+rsp]

	mov	rbp,QWORD PTR[80+rsp]

	lea	rsp,QWORD PTR[88+rsp]

$L$SEH_epilogue_sqrx_n_mul_mont_383::
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

$L$SEH_end_sqrx_n_mul_mont_383::
sqrx_n_mul_mont_383	ENDP

ALIGN	32
__mulx_mont_383_nonred	PROC PRIVATE
	DB	243,15,30,250


	mulx	r10,r14,r15
	mulx	r11,r15,rax
	add	r9,r14
	mulx	r12,rax,r12
	adc	r10,r15
	mulx	r13,rdi,rdi
	adc	r11,rax
	mulx	r14,rbp,rbp
	mov	rdx,QWORD PTR[8+rbx]
	adc	r12,rdi
	adc	r13,rbp
	adc	r14,0
	mov	rax,r8
	imul	r8,QWORD PTR[8+rsp]


	xor	r15,r15
	mulx	rbp,rdi,QWORD PTR[((0+128))+rsi]
	adox	r9,rdi
	adcx	r10,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rsi]
	adox	r10,rdi
	adcx	r11,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rsi]
	adox	r11,rdi
	adcx	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rsi]
	adox	r12,rdi
	adcx	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rsi]
	adox	r13,rdi
	adcx	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rsi]
	mov	rdx,r8
	adox	r14,rdi
	adcx	rbp,r15
	adox	r15,rbp


	xor	r8,r8
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	rax,rdi
	adox	r9,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r9,rdi
	adox	r10,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r10,rdi
	adox	r11,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	r11,rdi
	adox	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	r12,rdi
	adox	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	mov	rdx,QWORD PTR[16+rbx]
	adcx	r13,rdi
	adox	r14,rbp
	adcx	r14,rax
	adox	r15,rax
	adcx	r15,rax
	mov	r8,r9
	imul	r9,QWORD PTR[8+rsp]


	xor	rax,rax
	mulx	rbp,rdi,QWORD PTR[((0+128))+rsi]
	adox	r10,rdi
	adcx	r11,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rsi]
	adox	r11,rdi
	adcx	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rsi]
	adox	r12,rdi
	adcx	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rsi]
	adox	r13,rdi
	adcx	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rsi]
	adox	r14,rdi
	adcx	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rsi]
	mov	rdx,r9
	adox	r15,rdi
	adcx	rbp,rax
	adox	rax,rbp


	xor	r9,r9
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	r8,rdi
	adox	r10,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r10,rdi
	adox	r11,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r11,rdi
	adox	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	r12,rdi
	adox	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	r13,rdi
	adox	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	mov	rdx,QWORD PTR[24+rbx]
	adcx	r14,rdi
	adox	r15,rbp
	adcx	r15,r8
	adox	rax,r8
	adcx	rax,r8
	mov	r9,r10
	imul	r10,QWORD PTR[8+rsp]


	xor	r8,r8
	mulx	rbp,rdi,QWORD PTR[((0+128))+rsi]
	adox	r11,rdi
	adcx	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rsi]
	adox	r12,rdi
	adcx	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rsi]
	adox	r13,rdi
	adcx	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rsi]
	adox	r14,rdi
	adcx	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rsi]
	adox	r15,rdi
	adcx	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rsi]
	mov	rdx,r10
	adox	rax,rdi
	adcx	rbp,r8
	adox	r8,rbp


	xor	r10,r10
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	r9,rdi
	adox	r11,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r11,rdi
	adox	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r12,rdi
	adox	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	r13,rdi
	adox	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	r14,rdi
	adox	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	mov	rdx,QWORD PTR[32+rbx]
	adcx	r15,rdi
	adox	rax,rbp
	adcx	rax,r9
	adox	r8,r9
	adcx	r8,r9
	mov	r10,r11
	imul	r11,QWORD PTR[8+rsp]


	xor	r9,r9
	mulx	rbp,rdi,QWORD PTR[((0+128))+rsi]
	adox	r12,rdi
	adcx	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rsi]
	adox	r13,rdi
	adcx	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rsi]
	adox	r14,rdi
	adcx	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rsi]
	adox	r15,rdi
	adcx	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rsi]
	adox	rax,rdi
	adcx	r8,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rsi]
	mov	rdx,r11
	adox	r8,rdi
	adcx	rbp,r9
	adox	r9,rbp


	xor	r11,r11
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	r10,rdi
	adox	r12,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r12,rdi
	adox	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r13,rdi
	adox	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	r14,rdi
	adox	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	r15,rdi
	adox	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	mov	rdx,QWORD PTR[40+rbx]
	adcx	rax,rdi
	adox	r8,rbp
	adcx	r8,r10
	adox	r9,r10
	adcx	r9,r10
	mov	r11,r12
	imul	r12,QWORD PTR[8+rsp]


	xor	r10,r10
	mulx	rbp,rdi,QWORD PTR[((0+128))+rsi]
	adox	r13,rdi
	adcx	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rsi]
	adox	r14,rdi
	adcx	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rsi]
	adox	r15,rdi
	adcx	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rsi]
	adox	rax,rdi
	adcx	r8,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rsi]
	adox	r8,rdi
	adcx	r9,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rsi]
	mov	rdx,r12
	adox	r9,rdi
	adcx	rbp,r10
	adox	r10,rbp


	xor	r12,r12
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	r11,rdi
	adox	r13,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r13,rdi
	adox	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r14,rdi
	adox	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	r15,rdi
	adox	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	rax,rdi
	adox	r8,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	mov	rdx,r13
	adcx	r8,rdi
	adox	r9,rbp
	adcx	r9,r11
	adox	r10,r11
	adcx	r10,r11
	imul	rdx,QWORD PTR[8+rsp]
	mov	rbx,QWORD PTR[24+rsp]


	xor	r12,r12
	mulx	rbp,rdi,QWORD PTR[((0+128))+rcx]
	adcx	r13,rdi
	adox	r14,rbp

	mulx	rbp,rdi,QWORD PTR[((8+128))+rcx]
	adcx	r14,rdi
	adox	r15,rbp

	mulx	rbp,rdi,QWORD PTR[((16+128))+rcx]
	adcx	r15,rdi
	adox	rax,rbp

	mulx	rbp,rdi,QWORD PTR[((24+128))+rcx]
	adcx	rax,rdi
	adox	r8,rbp

	mulx	rbp,rdi,QWORD PTR[((32+128))+rcx]
	adcx	r8,rdi
	adox	r9,rbp

	mulx	rbp,rdi,QWORD PTR[((40+128))+rcx]
	mov	rdx,r14
	adcx	r9,rdi
	adox	r10,rbp
	adc	r10,0
	mov	r12,r8

	mov	QWORD PTR[rbx],r14
	mov	QWORD PTR[8+rbx],r15
	mov	QWORD PTR[16+rbx],rax
	mov	rdi,r9
	mov	QWORD PTR[24+rbx],r8
	mov	QWORD PTR[32+rbx],r9
	mov	QWORD PTR[40+rbx],r10
	mov	rbp,r10

	
ifdef	__SGX_LVI_HARDENING__
	pop	rsi
	lfence
	jmp	rsi
	ud2
else
	DB	0F3h,0C3h
endif

__mulx_mont_383_nonred	ENDP
PUBLIC	sqrx_mont_382x


ALIGN	32
sqrx_mont_382x	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqrx_mont_382x::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
sqr_mont_382x$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,136

$L$SEH_body_sqrx_mont_382x::


	mov	QWORD PTR[rsp],rcx
	mov	rcx,rdx
	mov	QWORD PTR[16+rsp],rdi
	mov	QWORD PTR[24+rsp],rsi


ifdef	__SGX_LVI_HARDENING__
	lfence
endif
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

	mov	rdx,QWORD PTR[48+rsi]
	mov	r14,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	rax,QWORD PTR[16+rsi]
	mov	r12,QWORD PTR[24+rsi]
	mov	rdi,QWORD PTR[32+rsi]
	mov	rbp,QWORD PTR[40+rsi]
	lea	rsi,QWORD PTR[((-128))+rsi]
	lea	rcx,QWORD PTR[((-128))+rcx]

	mulx	r9,r8,r14
	call	__mulx_mont_383_nonred
	add	rdx,rdx
	adc	r15,r15
	adc	rax,rax
	adc	r12,r12
	adc	rdi,rdi
	adc	rbp,rbp

	mov	QWORD PTR[48+rbx],rdx
	mov	QWORD PTR[56+rbx],r15
	mov	QWORD PTR[64+rbx],rax
	mov	QWORD PTR[72+rbx],r12
	mov	QWORD PTR[80+rbx],rdi
	mov	QWORD PTR[88+rbx],rbp

	lea	rsi,QWORD PTR[((32-128))+rsp]
	lea	rbx,QWORD PTR[((32+48))+rsp]

	mov	rdx,QWORD PTR[((32+48))+rsp]
	mov	r14,QWORD PTR[((32+0))+rsp]
	mov	r15,QWORD PTR[((32+8))+rsp]
	mov	rax,QWORD PTR[((32+16))+rsp]
	mov	r12,QWORD PTR[((32+24))+rsp]
	mov	rdi,QWORD PTR[((32+32))+rsp]
	mov	rbp,QWORD PTR[((32+40))+rsp]



	mulx	r9,r8,r14
	call	__mulx_mont_383_nonred
	mov	r14,QWORD PTR[((32+96))+rsp]
	lea	rcx,QWORD PTR[128+rcx]
	mov	r8,QWORD PTR[((32+0))+rsp]
	and	r8,r14
	mov	r9,QWORD PTR[((32+8))+rsp]
	and	r9,r14
	mov	r10,QWORD PTR[((32+16))+rsp]
	and	r10,r14
	mov	r11,QWORD PTR[((32+24))+rsp]
	and	r11,r14
	mov	r13,QWORD PTR[((32+32))+rsp]
	and	r13,r14
	and	r14,QWORD PTR[((32+40))+rsp]

	sub	rdx,r8
	mov	r8,QWORD PTR[rcx]
	sbb	r15,r9
	mov	r9,QWORD PTR[8+rcx]
	sbb	rax,r10
	mov	r10,QWORD PTR[16+rcx]
	sbb	r12,r11
	mov	r11,QWORD PTR[24+rcx]
	sbb	rdi,r13
	mov	r13,QWORD PTR[32+rcx]
	sbb	rbp,r14
	sbb	r14,r14

	and	r8,r14
	and	r9,r14
	and	r10,r14
	and	r11,r14
	and	r13,r14
	and	r14,QWORD PTR[40+rcx]

	add	rdx,r8
	adc	r15,r9
	adc	rax,r10
	adc	r12,r11
	adc	rdi,r13
	adc	rbp,r14

	mov	QWORD PTR[rbx],rdx
	mov	QWORD PTR[8+rbx],r15
	mov	QWORD PTR[16+rbx],rax
	mov	QWORD PTR[24+rbx],r12
	mov	QWORD PTR[32+rbx],rdi
	mov	QWORD PTR[40+rbx],rbp
	lea	r8,QWORD PTR[136+rsp]
	mov	r15,QWORD PTR[r8]

	mov	r14,QWORD PTR[8+r8]

	mov	r13,QWORD PTR[16+r8]

	mov	r12,QWORD PTR[24+r8]

	mov	rbx,QWORD PTR[32+r8]

	mov	rbp,QWORD PTR[40+r8]

	lea	rsp,QWORD PTR[48+r8]

$L$SEH_epilogue_sqrx_mont_382x::
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

$L$SEH_end_sqrx_mont_382x::
sqrx_mont_382x	ENDP
.text$	ENDS
.pdata	SEGMENT READONLY ALIGN(4)
ALIGN	4
	DD	imagerel $L$SEH_begin_mulx_mont_384x
	DD	imagerel $L$SEH_body_mulx_mont_384x
	DD	imagerel $L$SEH_info_mulx_mont_384x_prologue

	DD	imagerel $L$SEH_body_mulx_mont_384x
	DD	imagerel $L$SEH_epilogue_mulx_mont_384x
	DD	imagerel $L$SEH_info_mulx_mont_384x_body

	DD	imagerel $L$SEH_epilogue_mulx_mont_384x
	DD	imagerel $L$SEH_end_mulx_mont_384x
	DD	imagerel $L$SEH_info_mulx_mont_384x_epilogue

	DD	imagerel $L$SEH_begin_sqrx_mont_384x
	DD	imagerel $L$SEH_body_sqrx_mont_384x
	DD	imagerel $L$SEH_info_sqrx_mont_384x_prologue

	DD	imagerel $L$SEH_body_sqrx_mont_384x
	DD	imagerel $L$SEH_epilogue_sqrx_mont_384x
	DD	imagerel $L$SEH_info_sqrx_mont_384x_body

	DD	imagerel $L$SEH_epilogue_sqrx_mont_384x
	DD	imagerel $L$SEH_end_sqrx_mont_384x
	DD	imagerel $L$SEH_info_sqrx_mont_384x_epilogue

	DD	imagerel $L$SEH_begin_mulx_382x
	DD	imagerel $L$SEH_body_mulx_382x
	DD	imagerel $L$SEH_info_mulx_382x_prologue

	DD	imagerel $L$SEH_body_mulx_382x
	DD	imagerel $L$SEH_epilogue_mulx_382x
	DD	imagerel $L$SEH_info_mulx_382x_body

	DD	imagerel $L$SEH_epilogue_mulx_382x
	DD	imagerel $L$SEH_end_mulx_382x
	DD	imagerel $L$SEH_info_mulx_382x_epilogue

	DD	imagerel $L$SEH_begin_sqrx_382x
	DD	imagerel $L$SEH_body_sqrx_382x
	DD	imagerel $L$SEH_info_sqrx_382x_prologue

	DD	imagerel $L$SEH_body_sqrx_382x
	DD	imagerel $L$SEH_epilogue_sqrx_382x
	DD	imagerel $L$SEH_info_sqrx_382x_body

	DD	imagerel $L$SEH_epilogue_sqrx_382x
	DD	imagerel $L$SEH_end_sqrx_382x
	DD	imagerel $L$SEH_info_sqrx_382x_epilogue

	DD	imagerel $L$SEH_begin_mulx_384
	DD	imagerel $L$SEH_body_mulx_384
	DD	imagerel $L$SEH_info_mulx_384_prologue

	DD	imagerel $L$SEH_body_mulx_384
	DD	imagerel $L$SEH_epilogue_mulx_384
	DD	imagerel $L$SEH_info_mulx_384_body

	DD	imagerel $L$SEH_epilogue_mulx_384
	DD	imagerel $L$SEH_end_mulx_384
	DD	imagerel $L$SEH_info_mulx_384_epilogue

	DD	imagerel $L$SEH_begin_sqrx_384
	DD	imagerel $L$SEH_body_sqrx_384
	DD	imagerel $L$SEH_info_sqrx_384_prologue

	DD	imagerel $L$SEH_body_sqrx_384
	DD	imagerel $L$SEH_epilogue_sqrx_384
	DD	imagerel $L$SEH_info_sqrx_384_body

	DD	imagerel $L$SEH_epilogue_sqrx_384
	DD	imagerel $L$SEH_end_sqrx_384
	DD	imagerel $L$SEH_info_sqrx_384_epilogue

	DD	imagerel $L$SEH_begin_redcx_mont_384
	DD	imagerel $L$SEH_body_redcx_mont_384
	DD	imagerel $L$SEH_info_redcx_mont_384_prologue

	DD	imagerel $L$SEH_body_redcx_mont_384
	DD	imagerel $L$SEH_epilogue_redcx_mont_384
	DD	imagerel $L$SEH_info_redcx_mont_384_body

	DD	imagerel $L$SEH_epilogue_redcx_mont_384
	DD	imagerel $L$SEH_end_redcx_mont_384
	DD	imagerel $L$SEH_info_redcx_mont_384_epilogue

	DD	imagerel $L$SEH_begin_fromx_mont_384
	DD	imagerel $L$SEH_body_fromx_mont_384
	DD	imagerel $L$SEH_info_fromx_mont_384_prologue

	DD	imagerel $L$SEH_body_fromx_mont_384
	DD	imagerel $L$SEH_epilogue_fromx_mont_384
	DD	imagerel $L$SEH_info_fromx_mont_384_body

	DD	imagerel $L$SEH_epilogue_fromx_mont_384
	DD	imagerel $L$SEH_end_fromx_mont_384
	DD	imagerel $L$SEH_info_fromx_mont_384_epilogue

	DD	imagerel $L$SEH_begin_sgn0x_pty_mont_384
	DD	imagerel $L$SEH_body_sgn0x_pty_mont_384
	DD	imagerel $L$SEH_info_sgn0x_pty_mont_384_prologue

	DD	imagerel $L$SEH_body_sgn0x_pty_mont_384
	DD	imagerel $L$SEH_epilogue_sgn0x_pty_mont_384
	DD	imagerel $L$SEH_info_sgn0x_pty_mont_384_body

	DD	imagerel $L$SEH_epilogue_sgn0x_pty_mont_384
	DD	imagerel $L$SEH_end_sgn0x_pty_mont_384
	DD	imagerel $L$SEH_info_sgn0x_pty_mont_384_epilogue

	DD	imagerel $L$SEH_begin_sgn0x_pty_mont_384x
	DD	imagerel $L$SEH_body_sgn0x_pty_mont_384x
	DD	imagerel $L$SEH_info_sgn0x_pty_mont_384x_prologue

	DD	imagerel $L$SEH_body_sgn0x_pty_mont_384x
	DD	imagerel $L$SEH_epilogue_sgn0x_pty_mont_384x
	DD	imagerel $L$SEH_info_sgn0x_pty_mont_384x_body

	DD	imagerel $L$SEH_epilogue_sgn0x_pty_mont_384x
	DD	imagerel $L$SEH_end_sgn0x_pty_mont_384x
	DD	imagerel $L$SEH_info_sgn0x_pty_mont_384x_epilogue

	DD	imagerel $L$SEH_begin_mulx_mont_384
	DD	imagerel $L$SEH_body_mulx_mont_384
	DD	imagerel $L$SEH_info_mulx_mont_384_prologue

	DD	imagerel $L$SEH_body_mulx_mont_384
	DD	imagerel $L$SEH_epilogue_mulx_mont_384
	DD	imagerel $L$SEH_info_mulx_mont_384_body

	DD	imagerel $L$SEH_epilogue_mulx_mont_384
	DD	imagerel $L$SEH_end_mulx_mont_384
	DD	imagerel $L$SEH_info_mulx_mont_384_epilogue

	DD	imagerel $L$SEH_begin_sqrx_mont_384
	DD	imagerel $L$SEH_body_sqrx_mont_384
	DD	imagerel $L$SEH_info_sqrx_mont_384_prologue

	DD	imagerel $L$SEH_body_sqrx_mont_384
	DD	imagerel $L$SEH_epilogue_sqrx_mont_384
	DD	imagerel $L$SEH_info_sqrx_mont_384_body

	DD	imagerel $L$SEH_epilogue_sqrx_mont_384
	DD	imagerel $L$SEH_end_sqrx_mont_384
	DD	imagerel $L$SEH_info_sqrx_mont_384_epilogue

	DD	imagerel $L$SEH_begin_sqrx_n_mul_mont_384
	DD	imagerel $L$SEH_body_sqrx_n_mul_mont_384
	DD	imagerel $L$SEH_info_sqrx_n_mul_mont_384_prologue

	DD	imagerel $L$SEH_body_sqrx_n_mul_mont_384
	DD	imagerel $L$SEH_epilogue_sqrx_n_mul_mont_384
	DD	imagerel $L$SEH_info_sqrx_n_mul_mont_384_body

	DD	imagerel $L$SEH_epilogue_sqrx_n_mul_mont_384
	DD	imagerel $L$SEH_end_sqrx_n_mul_mont_384
	DD	imagerel $L$SEH_info_sqrx_n_mul_mont_384_epilogue

	DD	imagerel $L$SEH_begin_sqrx_n_mul_mont_383
	DD	imagerel $L$SEH_body_sqrx_n_mul_mont_383
	DD	imagerel $L$SEH_info_sqrx_n_mul_mont_383_prologue

	DD	imagerel $L$SEH_body_sqrx_n_mul_mont_383
	DD	imagerel $L$SEH_epilogue_sqrx_n_mul_mont_383
	DD	imagerel $L$SEH_info_sqrx_n_mul_mont_383_body

	DD	imagerel $L$SEH_epilogue_sqrx_n_mul_mont_383
	DD	imagerel $L$SEH_end_sqrx_n_mul_mont_383
	DD	imagerel $L$SEH_info_sqrx_n_mul_mont_383_epilogue

	DD	imagerel $L$SEH_begin_sqrx_mont_382x
	DD	imagerel $L$SEH_body_sqrx_mont_382x
	DD	imagerel $L$SEH_info_sqrx_mont_382x_prologue

	DD	imagerel $L$SEH_body_sqrx_mont_382x
	DD	imagerel $L$SEH_epilogue_sqrx_mont_382x
	DD	imagerel $L$SEH_info_sqrx_mont_382x_body

	DD	imagerel $L$SEH_epilogue_sqrx_mont_382x
	DD	imagerel $L$SEH_end_sqrx_mont_382x
	DD	imagerel $L$SEH_info_sqrx_mont_382x_epilogue

.pdata	ENDS
.xdata	SEGMENT READONLY ALIGN(8)
ALIGN	8
$L$SEH_info_mulx_mont_384x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_mulx_mont_384x_body::
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
$L$SEH_info_mulx_mont_384x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqrx_mont_384x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqrx_mont_384x_body::
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
$L$SEH_info_sqrx_mont_384x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_mulx_382x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_mulx_382x_body::
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
$L$SEH_info_mulx_382x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqrx_382x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqrx_382x_body::
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
$L$SEH_info_sqrx_382x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_mulx_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_mulx_384_body::
DB	1,0,17,0
DB	000h,0f4h,000h,000h
DB	000h,0e4h,001h,000h
DB	000h,0d4h,002h,000h
DB	000h,0c4h,003h,000h
DB	000h,034h,004h,000h
DB	000h,054h,005h,000h
DB	000h,074h,007h,000h
DB	000h,064h,008h,000h
DB	000h,052h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_mulx_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqrx_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqrx_384_body::
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
$L$SEH_info_sqrx_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_redcx_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_redcx_mont_384_body::
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
$L$SEH_info_redcx_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_fromx_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_fromx_mont_384_body::
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
$L$SEH_info_fromx_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sgn0x_pty_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sgn0x_pty_mont_384_body::
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
$L$SEH_info_sgn0x_pty_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sgn0x_pty_mont_384x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sgn0x_pty_mont_384x_body::
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
$L$SEH_info_sgn0x_pty_mont_384x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_mulx_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_mulx_mont_384_body::
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
$L$SEH_info_mulx_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqrx_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqrx_mont_384_body::
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
$L$SEH_info_sqrx_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqrx_n_mul_mont_384_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqrx_n_mul_mont_384_body::
DB	1,0,17,0
DB	000h,0f4h,005h,000h
DB	000h,0e4h,006h,000h
DB	000h,0d4h,007h,000h
DB	000h,0c4h,008h,000h
DB	000h,034h,009h,000h
DB	000h,054h,00ah,000h
DB	000h,074h,00ch,000h
DB	000h,064h,00dh,000h
DB	000h,0a2h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sqrx_n_mul_mont_384_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqrx_n_mul_mont_383_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqrx_n_mul_mont_383_body::
DB	1,0,17,0
DB	000h,0f4h,005h,000h
DB	000h,0e4h,006h,000h
DB	000h,0d4h,007h,000h
DB	000h,0c4h,008h,000h
DB	000h,034h,009h,000h
DB	000h,054h,00ah,000h
DB	000h,074h,00ch,000h
DB	000h,064h,00dh,000h
DB	000h,0a2h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sqrx_n_mul_mont_383_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqrx_mont_382x_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqrx_mont_382x_body::
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
$L$SEH_info_sqrx_mont_382x_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h


.xdata	ENDS
END
