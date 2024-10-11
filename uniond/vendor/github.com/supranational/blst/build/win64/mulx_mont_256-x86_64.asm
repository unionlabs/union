OPTION	DOTNAME
PUBLIC	mul_mont_sparse_256$1
PUBLIC	sqr_mont_sparse_256$1
PUBLIC	from_mont_256$1
PUBLIC	redc_mont_256$1
.text$	SEGMENT ALIGN(256) 'CODE'

PUBLIC	mulx_mont_sparse_256


ALIGN	32
mulx_mont_sparse_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_mulx_mont_sparse_256::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	mov	r8,QWORD PTR[40+rsp]
mul_mont_sparse_256$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_mulx_mont_sparse_256::


	mov	rbx,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rdx,QWORD PTR[rdx]
	mov	r14,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	rbp,QWORD PTR[16+rsi]
	mov	r9,QWORD PTR[24+rsi]
	lea	rsi,QWORD PTR[((-128))+rsi]
	lea	rcx,QWORD PTR[((-128))+rcx]

	mulx	r11,rax,r14
	call	__mulx_mont_sparse_256

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_mulx_mont_sparse_256::
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

$L$SEH_end_mulx_mont_sparse_256::
mulx_mont_sparse_256	ENDP

PUBLIC	sqrx_mont_sparse_256


ALIGN	32
sqrx_mont_sparse_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sqrx_mont_sparse_256::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
sqr_mont_sparse_256$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_sqrx_mont_sparse_256::


	mov	rbx,rsi
	mov	r8,rcx
	mov	rcx,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rdx,QWORD PTR[rsi]
	mov	r15,QWORD PTR[8+rsi]
	mov	rbp,QWORD PTR[16+rsi]
	mov	r9,QWORD PTR[24+rsi]
	lea	rsi,QWORD PTR[((-128))+rbx]
	lea	rcx,QWORD PTR[((-128))+rcx]

	mulx	r11,rax,rdx
	call	__mulx_mont_sparse_256

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_sqrx_mont_sparse_256::
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

$L$SEH_end_sqrx_mont_sparse_256::
sqrx_mont_sparse_256	ENDP

ALIGN	32
__mulx_mont_sparse_256	PROC PRIVATE
	DB	243,15,30,250

	mulx	r12,r15,r15
	mulx	r13,rbp,rbp
	add	r11,r15
	mulx	r14,r9,r9
	mov	rdx,QWORD PTR[8+rbx]
	adc	r12,rbp
	adc	r13,r9
	adc	r14,0

	mov	r10,rax
	imul	rax,r8


	xor	r15,r15
	mulx	r9,rbp,QWORD PTR[((0+128))+rsi]
	adox	r11,rbp
	adcx	r12,r9

	mulx	r9,rbp,QWORD PTR[((8+128))+rsi]
	adox	r12,rbp
	adcx	r13,r9

	mulx	r9,rbp,QWORD PTR[((16+128))+rsi]
	adox	r13,rbp
	adcx	r14,r9

	mulx	r9,rbp,QWORD PTR[((24+128))+rsi]
	mov	rdx,rax
	adox	r14,rbp
	adcx	r9,r15
	adox	r15,r9


	mulx	rax,rbp,QWORD PTR[((0+128))+rcx]
	adcx	r10,rbp
	adox	rax,r11

	mulx	r9,rbp,QWORD PTR[((8+128))+rcx]
	adcx	rax,rbp
	adox	r12,r9

	mulx	r9,rbp,QWORD PTR[((16+128))+rcx]
	adcx	r12,rbp
	adox	r13,r9

	mulx	r9,rbp,QWORD PTR[((24+128))+rcx]
	mov	rdx,QWORD PTR[16+rbx]
	adcx	r13,rbp
	adox	r14,r9
	adcx	r14,r10
	adox	r15,r10
	adcx	r15,r10
	adox	r10,r10
	adc	r10,0
	mov	r11,rax
	imul	rax,r8


	xor	rbp,rbp
	mulx	r9,rbp,QWORD PTR[((0+128))+rsi]
	adox	r12,rbp
	adcx	r13,r9

	mulx	r9,rbp,QWORD PTR[((8+128))+rsi]
	adox	r13,rbp
	adcx	r14,r9

	mulx	r9,rbp,QWORD PTR[((16+128))+rsi]
	adox	r14,rbp
	adcx	r15,r9

	mulx	r9,rbp,QWORD PTR[((24+128))+rsi]
	mov	rdx,rax
	adox	r15,rbp
	adcx	r9,r10
	adox	r10,r9


	mulx	rax,rbp,QWORD PTR[((0+128))+rcx]
	adcx	r11,rbp
	adox	rax,r12

	mulx	r9,rbp,QWORD PTR[((8+128))+rcx]
	adcx	rax,rbp
	adox	r13,r9

	mulx	r9,rbp,QWORD PTR[((16+128))+rcx]
	adcx	r13,rbp
	adox	r14,r9

	mulx	r9,rbp,QWORD PTR[((24+128))+rcx]
	mov	rdx,QWORD PTR[24+rbx]
	adcx	r14,rbp
	adox	r15,r9
	adcx	r15,r11
	adox	r10,r11
	adcx	r10,r11
	adox	r11,r11
	adc	r11,0
	mov	r12,rax
	imul	rax,r8


	xor	rbp,rbp
	mulx	r9,rbp,QWORD PTR[((0+128))+rsi]
	adox	r13,rbp
	adcx	r14,r9

	mulx	r9,rbp,QWORD PTR[((8+128))+rsi]
	adox	r14,rbp
	adcx	r15,r9

	mulx	r9,rbp,QWORD PTR[((16+128))+rsi]
	adox	r15,rbp
	adcx	r10,r9

	mulx	r9,rbp,QWORD PTR[((24+128))+rsi]
	mov	rdx,rax
	adox	r10,rbp
	adcx	r9,r11
	adox	r11,r9


	mulx	rax,rbp,QWORD PTR[((0+128))+rcx]
	adcx	r12,rbp
	adox	rax,r13

	mulx	r9,rbp,QWORD PTR[((8+128))+rcx]
	adcx	rax,rbp
	adox	r14,r9

	mulx	r9,rbp,QWORD PTR[((16+128))+rcx]
	adcx	r14,rbp
	adox	r15,r9

	mulx	r9,rbp,QWORD PTR[((24+128))+rcx]
	mov	rdx,rax
	adcx	r15,rbp
	adox	r10,r9
	adcx	r10,r12
	adox	r11,r12
	adcx	r11,r12
	adox	r12,r12
	adc	r12,0
	imul	rdx,r8


	xor	rbp,rbp
	mulx	r9,r13,QWORD PTR[((0+128))+rcx]
	adcx	r13,rax
	adox	r14,r9

	mulx	r9,rbp,QWORD PTR[((8+128))+rcx]
	adcx	r14,rbp
	adox	r15,r9

	mulx	r9,rbp,QWORD PTR[((16+128))+rcx]
	adcx	r15,rbp
	adox	r10,r9

	mulx	r9,rbp,QWORD PTR[((24+128))+rcx]
	mov	rdx,r14
	lea	rcx,QWORD PTR[128+rcx]
	adcx	r10,rbp
	adox	r11,r9
	mov	rax,r15
	adcx	r11,r13
	adox	r12,r13
	adc	r12,0




	mov	rbp,r10
	sub	r14,QWORD PTR[rcx]
	sbb	r15,QWORD PTR[8+rcx]
	sbb	r10,QWORD PTR[16+rcx]
	mov	r9,r11
	sbb	r11,QWORD PTR[24+rcx]
	sbb	r12,0

	cmovc	r14,rdx
	cmovc	r15,rax
	cmovc	r10,rbp
	mov	QWORD PTR[rdi],r14
	cmovc	r11,r9
	mov	QWORD PTR[8+rdi],r15
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11

	
ifdef	__SGX_LVI_HARDENING__
	pop	rdx
	lfence
	jmp	rdx
	ud2
else
	DB	0F3h,0C3h
endif
__mulx_mont_sparse_256	ENDP
PUBLIC	fromx_mont_256


ALIGN	32
fromx_mont_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_fromx_mont_256::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
from_mont_256$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_fromx_mont_256::


	mov	rbx,rdx
	call	__mulx_by_1_mont_256





	mov	rdx,r15
	mov	r12,r10
	mov	r13,r11

	sub	r14,QWORD PTR[rbx]
	sbb	r15,QWORD PTR[8+rbx]
	sbb	r10,QWORD PTR[16+rbx]
	sbb	r11,QWORD PTR[24+rbx]

	cmovnc	rax,r14
	cmovnc	rdx,r15
	cmovnc	r12,r10
	mov	QWORD PTR[rdi],rax
	cmovnc	r13,r11
	mov	QWORD PTR[8+rdi],rdx
	mov	QWORD PTR[16+rdi],r12
	mov	QWORD PTR[24+rdi],r13

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_fromx_mont_256::
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

$L$SEH_end_fromx_mont_256::
fromx_mont_256	ENDP

PUBLIC	redcx_mont_256


ALIGN	32
redcx_mont_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_redcx_mont_256::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
redc_mont_256$1::
	push	rbp

	push	rbx

	push	r12

	push	r13

	push	r14

	push	r15

	sub	rsp,8

$L$SEH_body_redcx_mont_256::


	mov	rbx,rdx
	call	__mulx_by_1_mont_256

	add	r14,QWORD PTR[32+rsi]
	adc	r15,QWORD PTR[40+rsi]
	mov	rax,r14
	adc	r10,QWORD PTR[48+rsi]
	mov	rdx,r15
	adc	r11,QWORD PTR[56+rsi]
	sbb	rsi,rsi




	mov	r12,r10
	sub	r14,QWORD PTR[rbx]
	sbb	r15,QWORD PTR[8+rbx]
	sbb	r10,QWORD PTR[16+rbx]
	mov	r13,r11
	sbb	r11,QWORD PTR[24+rbx]
	sbb	rsi,0

	cmovnc	rax,r14
	cmovnc	rdx,r15
	cmovnc	r12,r10
	mov	QWORD PTR[rdi],rax
	cmovnc	r13,r11
	mov	QWORD PTR[8+rdi],rdx
	mov	QWORD PTR[16+rdi],r12
	mov	QWORD PTR[24+rdi],r13

	mov	r15,QWORD PTR[8+rsp]

	mov	r14,QWORD PTR[16+rsp]

	mov	r13,QWORD PTR[24+rsp]

	mov	r12,QWORD PTR[32+rsp]

	mov	rbx,QWORD PTR[40+rsp]

	mov	rbp,QWORD PTR[48+rsp]

	lea	rsp,QWORD PTR[56+rsp]

$L$SEH_epilogue_redcx_mont_256::
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

$L$SEH_end_redcx_mont_256::
redcx_mont_256	ENDP

ALIGN	32
__mulx_by_1_mont_256	PROC PRIVATE
	DB	243,15,30,250

ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rax,QWORD PTR[rsi]
	mov	r11,QWORD PTR[8+rsi]
	mov	r12,QWORD PTR[16+rsi]
	mov	r13,QWORD PTR[24+rsi]

	mov	r14,rax
	imul	rax,rcx
	mov	r10,rax

	mul	QWORD PTR[rbx]
	add	r14,rax
	mov	rax,r10
	adc	r14,rdx

	mul	QWORD PTR[8+rbx]
	add	r11,rax
	mov	rax,r10
	adc	rdx,0
	add	r11,r14
	adc	rdx,0
	mov	r14,rdx

	mul	QWORD PTR[16+rbx]
	mov	r15,r11
	imul	r11,rcx
	add	r12,rax
	mov	rax,r10
	adc	rdx,0
	add	r12,r14
	adc	rdx,0
	mov	r14,rdx

	mul	QWORD PTR[24+rbx]
	add	r13,rax
	mov	rax,r11
	adc	rdx,0
	add	r13,r14
	adc	rdx,0
	mov	r14,rdx

	mul	QWORD PTR[rbx]
	add	r15,rax
	mov	rax,r11
	adc	r15,rdx

	mul	QWORD PTR[8+rbx]
	add	r12,rax
	mov	rax,r11
	adc	rdx,0
	add	r12,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[16+rbx]
	mov	r10,r12
	imul	r12,rcx
	add	r13,rax
	mov	rax,r11
	adc	rdx,0
	add	r13,r15
	adc	rdx,0
	mov	r15,rdx

	mul	QWORD PTR[24+rbx]
	add	r14,rax
	mov	rax,r12
	adc	rdx,0
	add	r14,r15
	adc	rdx,0
	mov	r15,rdx

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
	mov	r11,r13
	imul	r13,rcx
	add	r14,rax
	mov	rax,r12
	adc	rdx,0
	add	r14,r10
	adc	rdx,0
	mov	r10,rdx

	mul	QWORD PTR[24+rbx]
	add	r15,rax
	mov	rax,r13
	adc	rdx,0
	add	r15,r10
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
__mulx_by_1_mont_256	ENDP
.text$	ENDS
.pdata	SEGMENT READONLY ALIGN(4)
ALIGN	4
	DD	imagerel $L$SEH_begin_mulx_mont_sparse_256
	DD	imagerel $L$SEH_body_mulx_mont_sparse_256
	DD	imagerel $L$SEH_info_mulx_mont_sparse_256_prologue

	DD	imagerel $L$SEH_body_mulx_mont_sparse_256
	DD	imagerel $L$SEH_epilogue_mulx_mont_sparse_256
	DD	imagerel $L$SEH_info_mulx_mont_sparse_256_body

	DD	imagerel $L$SEH_epilogue_mulx_mont_sparse_256
	DD	imagerel $L$SEH_end_mulx_mont_sparse_256
	DD	imagerel $L$SEH_info_mulx_mont_sparse_256_epilogue

	DD	imagerel $L$SEH_begin_sqrx_mont_sparse_256
	DD	imagerel $L$SEH_body_sqrx_mont_sparse_256
	DD	imagerel $L$SEH_info_sqrx_mont_sparse_256_prologue

	DD	imagerel $L$SEH_body_sqrx_mont_sparse_256
	DD	imagerel $L$SEH_epilogue_sqrx_mont_sparse_256
	DD	imagerel $L$SEH_info_sqrx_mont_sparse_256_body

	DD	imagerel $L$SEH_epilogue_sqrx_mont_sparse_256
	DD	imagerel $L$SEH_end_sqrx_mont_sparse_256
	DD	imagerel $L$SEH_info_sqrx_mont_sparse_256_epilogue

	DD	imagerel $L$SEH_begin_fromx_mont_256
	DD	imagerel $L$SEH_body_fromx_mont_256
	DD	imagerel $L$SEH_info_fromx_mont_256_prologue

	DD	imagerel $L$SEH_body_fromx_mont_256
	DD	imagerel $L$SEH_epilogue_fromx_mont_256
	DD	imagerel $L$SEH_info_fromx_mont_256_body

	DD	imagerel $L$SEH_epilogue_fromx_mont_256
	DD	imagerel $L$SEH_end_fromx_mont_256
	DD	imagerel $L$SEH_info_fromx_mont_256_epilogue

	DD	imagerel $L$SEH_begin_redcx_mont_256
	DD	imagerel $L$SEH_body_redcx_mont_256
	DD	imagerel $L$SEH_info_redcx_mont_256_prologue

	DD	imagerel $L$SEH_body_redcx_mont_256
	DD	imagerel $L$SEH_epilogue_redcx_mont_256
	DD	imagerel $L$SEH_info_redcx_mont_256_body

	DD	imagerel $L$SEH_epilogue_redcx_mont_256
	DD	imagerel $L$SEH_end_redcx_mont_256
	DD	imagerel $L$SEH_info_redcx_mont_256_epilogue

.pdata	ENDS
.xdata	SEGMENT READONLY ALIGN(8)
ALIGN	8
$L$SEH_info_mulx_mont_sparse_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_mulx_mont_sparse_256_body::
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
$L$SEH_info_mulx_mont_sparse_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sqrx_mont_sparse_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sqrx_mont_sparse_256_body::
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
$L$SEH_info_sqrx_mont_sparse_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_fromx_mont_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_fromx_mont_256_body::
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
$L$SEH_info_fromx_mont_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_redcx_mont_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_redcx_mont_256_body::
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
$L$SEH_info_redcx_mont_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h


.xdata	ENDS
END
