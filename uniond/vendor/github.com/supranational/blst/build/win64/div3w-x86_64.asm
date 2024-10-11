OPTION	DOTNAME
.text$	SEGMENT ALIGN(256) 'CODE'

PUBLIC	div_3_limbs


ALIGN	32
div_3_limbs	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_div_3_limbs::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
$L$SEH_body_div_3_limbs::

ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	r8,QWORD PTR[rdi]
	mov	r9,QWORD PTR[8+rdi]
	xor	rax,rax
	mov	ecx,64

$L$oop::
	mov	r10,r8
	sub	r8,rsi
	mov	r11,r9
	sbb	r9,rdx
	lea	rax,QWORD PTR[1+rax*1+rax]
	mov	rdi,rdx
	cmovc	r8,r10
	cmovc	r9,r11
	sbb	rax,0
	shl	rdi,63
	shr	rsi,1
	shr	rdx,1
	or	rsi,rdi
	sub	ecx,1
	jnz	$L$oop

	lea	rcx,QWORD PTR[1+rax*1+rax]
	sar	rax,63

	sub	r8,rsi
	sbb	r9,rdx
	sbb	rcx,0

	or	rax,rcx

$L$SEH_epilogue_div_3_limbs::
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

$L$SEH_end_div_3_limbs::
div_3_limbs	ENDP
PUBLIC	quot_rem_128


ALIGN	32
quot_rem_128	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_quot_rem_128::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
$L$SEH_body_quot_rem_128::

ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rax,rdx
	mov	rcx,rdx

	mul	QWORD PTR[rsi]
	mov	r8,rax
	mov	rax,rcx
	mov	r9,rdx

	mul	QWORD PTR[8+rsi]
	add	r9,rax
	adc	rdx,0

	mov	r10,QWORD PTR[rdi]
	mov	r11,QWORD PTR[8+rdi]
	mov	rax,QWORD PTR[16+rdi]

	sub	r10,r8
	sbb	r11,r9
	sbb	rax,rdx
	sbb	r8,r8

	add	rcx,r8
	mov	r9,r8
	and	r8,QWORD PTR[rsi]
	and	r9,QWORD PTR[8+rsi]
	add	r10,r8
	adc	r11,r9

	mov	QWORD PTR[rdi],r10
	mov	QWORD PTR[8+rdi],r11
	mov	QWORD PTR[16+rdi],rcx

	mov	rax,rcx

$L$SEH_epilogue_quot_rem_128::
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

$L$SEH_end_quot_rem_128::
quot_rem_128	ENDP





PUBLIC	quot_rem_64


ALIGN	32
quot_rem_64	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_quot_rem_64::


	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
$L$SEH_body_quot_rem_64::

ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rax,rdx
	imul	rdx,QWORD PTR[rsi]

	mov	r10,QWORD PTR[rdi]

	sub	r10,rdx

	mov	QWORD PTR[rdi],r10
	mov	QWORD PTR[8+rdi],rax

$L$SEH_epilogue_quot_rem_64::
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

$L$SEH_end_quot_rem_64::
quot_rem_64	ENDP
.text$	ENDS
.pdata	SEGMENT READONLY ALIGN(4)
ALIGN	4
	DD	imagerel $L$SEH_begin_div_3_limbs
	DD	imagerel $L$SEH_body_div_3_limbs
	DD	imagerel $L$SEH_info_div_3_limbs_prologue

	DD	imagerel $L$SEH_body_div_3_limbs
	DD	imagerel $L$SEH_epilogue_div_3_limbs
	DD	imagerel $L$SEH_info_div_3_limbs_body

	DD	imagerel $L$SEH_epilogue_div_3_limbs
	DD	imagerel $L$SEH_end_div_3_limbs
	DD	imagerel $L$SEH_info_div_3_limbs_epilogue

	DD	imagerel $L$SEH_begin_quot_rem_128
	DD	imagerel $L$SEH_body_quot_rem_128
	DD	imagerel $L$SEH_info_quot_rem_128_prologue

	DD	imagerel $L$SEH_body_quot_rem_128
	DD	imagerel $L$SEH_epilogue_quot_rem_128
	DD	imagerel $L$SEH_info_quot_rem_128_body

	DD	imagerel $L$SEH_epilogue_quot_rem_128
	DD	imagerel $L$SEH_end_quot_rem_128
	DD	imagerel $L$SEH_info_quot_rem_128_epilogue

	DD	imagerel $L$SEH_begin_quot_rem_64
	DD	imagerel $L$SEH_body_quot_rem_64
	DD	imagerel $L$SEH_info_quot_rem_64_prologue

	DD	imagerel $L$SEH_body_quot_rem_64
	DD	imagerel $L$SEH_epilogue_quot_rem_64
	DD	imagerel $L$SEH_info_quot_rem_64_body

	DD	imagerel $L$SEH_epilogue_quot_rem_64
	DD	imagerel $L$SEH_end_quot_rem_64
	DD	imagerel $L$SEH_info_quot_rem_64_epilogue

.pdata	ENDS
.xdata	SEGMENT READONLY ALIGN(8)
ALIGN	8
$L$SEH_info_div_3_limbs_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_div_3_limbs_body::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h
$L$SEH_info_div_3_limbs_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_quot_rem_128_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_quot_rem_128_body::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h
$L$SEH_info_quot_rem_128_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_quot_rem_64_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_quot_rem_64_body::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h
$L$SEH_info_quot_rem_64_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h


.xdata	ENDS
END
