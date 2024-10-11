OPTION	DOTNAME
.text$	SEGMENT ALIGN(256) 'CODE'

PUBLIC	add_mod_256


ALIGN	32
add_mod_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_add_mod_256::


	push	rbp

	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	push	rbx

	sub	rsp,8

$L$SEH_body_add_mod_256::


ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]

$L$oaded_a_add_mod_256::
	add	r8,QWORD PTR[rdx]
	adc	r9,QWORD PTR[8+rdx]
	mov	rax,r8
	adc	r10,QWORD PTR[16+rdx]
	mov	rsi,r9
	adc	r11,QWORD PTR[24+rdx]
	sbb	rdx,rdx

	mov	rbx,r10
	sub	r8,QWORD PTR[rcx]
	sbb	r9,QWORD PTR[8+rcx]
	sbb	r10,QWORD PTR[16+rcx]
	mov	rbp,r11
	sbb	r11,QWORD PTR[24+rcx]
	sbb	rdx,0

	cmovc	r8,rax
	cmovc	r9,rsi
	mov	QWORD PTR[rdi],r8
	cmovc	r10,rbx
	mov	QWORD PTR[8+rdi],r9
	cmovc	r11,rbp
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11

	mov	rbx,QWORD PTR[8+rsp]

	mov	rbp,QWORD PTR[16+rsp]

	lea	rsp,QWORD PTR[24+rsp]

$L$SEH_epilogue_add_mod_256::
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

$L$SEH_end_add_mod_256::
add_mod_256	ENDP


PUBLIC	mul_by_3_mod_256


ALIGN	32
mul_by_3_mod_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_mul_by_3_mod_256::


	push	rbp

	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	push	rbx

	push	r12

$L$SEH_body_mul_by_3_mod_256::


	mov	rcx,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	rdx,rsi
	mov	r11,QWORD PTR[24+rsi]

	call	__lshift_mod_256
	mov	r12,QWORD PTR[rsp]

	jmp	$L$oaded_a_add_mod_256

	mov	rbx,QWORD PTR[8+rsp]

	mov	rbp,QWORD PTR[16+rsp]

	lea	rsp,QWORD PTR[24+rsp]

$L$SEH_epilogue_mul_by_3_mod_256::
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

$L$SEH_end_mul_by_3_mod_256::
mul_by_3_mod_256	ENDP


ALIGN	32
__lshift_mod_256	PROC PRIVATE
	DB	243,15,30,250

	add	r8,r8
	adc	r9,r9
	mov	rax,r8
	adc	r10,r10
	mov	rsi,r9
	adc	r11,r11
	sbb	r12,r12

	mov	rbx,r10
	sub	r8,QWORD PTR[rcx]
	sbb	r9,QWORD PTR[8+rcx]
	sbb	r10,QWORD PTR[16+rcx]
	mov	rbp,r11
	sbb	r11,QWORD PTR[24+rcx]
	sbb	r12,0

	cmovc	r8,rax
	cmovc	r9,rsi
	cmovc	r10,rbx
	cmovc	r11,rbp

	
ifdef	__SGX_LVI_HARDENING__
	pop	rax
	lfence
	jmp	rax
	ud2
else
	DB	0F3h,0C3h
endif
__lshift_mod_256	ENDP


PUBLIC	lshift_mod_256


ALIGN	32
lshift_mod_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_lshift_mod_256::


	push	rbp

	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	push	rbx

	push	r12

$L$SEH_body_lshift_mod_256::


ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]

$L$oop_lshift_mod_256::
	call	__lshift_mod_256
	dec	edx
	jnz	$L$oop_lshift_mod_256

	mov	QWORD PTR[rdi],r8
	mov	QWORD PTR[8+rdi],r9
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11

	mov	r12,QWORD PTR[rsp]

	mov	rbx,QWORD PTR[8+rsp]

	mov	rbp,QWORD PTR[16+rsp]

	lea	rsp,QWORD PTR[24+rsp]

$L$SEH_epilogue_lshift_mod_256::
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

$L$SEH_end_lshift_mod_256::
lshift_mod_256	ENDP


PUBLIC	rshift_mod_256


ALIGN	32
rshift_mod_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_rshift_mod_256::


	push	rbp

	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	push	rbx

	sub	rsp,8

$L$SEH_body_rshift_mod_256::


ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rbp,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]

$L$oop_rshift_mod_256::
	mov	r8,rbp
	and	rbp,1
	mov	rax,QWORD PTR[rcx]
	neg	rbp
	mov	rsi,QWORD PTR[8+rcx]
	mov	rbx,QWORD PTR[16+rcx]

	and	rax,rbp
	and	rsi,rbp
	and	rbx,rbp
	and	rbp,QWORD PTR[24+rcx]

	add	r8,rax
	adc	r9,rsi
	adc	r10,rbx
	adc	r11,rbp
	sbb	rax,rax

	shr	r8,1
	mov	rbp,r9
	shr	r9,1
	mov	rbx,r10
	shr	r10,1
	mov	rsi,r11
	shr	r11,1

	shl	rbp,63
	shl	rbx,63
	or	rbp,r8
	shl	rsi,63
	or	r9,rbx
	shl	rax,63
	or	r10,rsi
	or	r11,rax

	dec	edx
	jnz	$L$oop_rshift_mod_256

	mov	QWORD PTR[rdi],rbp
	mov	QWORD PTR[8+rdi],r9
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11

	mov	rbx,QWORD PTR[8+rsp]

	mov	rbp,QWORD PTR[16+rsp]

	lea	rsp,QWORD PTR[24+rsp]

$L$SEH_epilogue_rshift_mod_256::
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

$L$SEH_end_rshift_mod_256::
rshift_mod_256	ENDP


PUBLIC	cneg_mod_256


ALIGN	32
cneg_mod_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_cneg_mod_256::


	push	rbp

	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	push	rbx

	push	r12

$L$SEH_body_cneg_mod_256::


ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	r12,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r8,r12
	mov	r11,QWORD PTR[24+rsi]
	or	r12,r9
	or	r12,r10
	or	r12,r11
	mov	rbp,-1

	mov	rax,QWORD PTR[rcx]
	cmovnz	r12,rbp
	mov	rsi,QWORD PTR[8+rcx]
	mov	rbx,QWORD PTR[16+rcx]
	and	rax,r12
	mov	rbp,QWORD PTR[24+rcx]
	and	rsi,r12
	and	rbx,r12
	and	rbp,r12

	sub	rax,r8
	sbb	rsi,r9
	sbb	rbx,r10
	sbb	rbp,r11

	or	rdx,rdx

	cmovz	rax,r8
	cmovz	rsi,r9
	mov	QWORD PTR[rdi],rax
	cmovz	rbx,r10
	mov	QWORD PTR[8+rdi],rsi
	cmovz	rbp,r11
	mov	QWORD PTR[16+rdi],rbx
	mov	QWORD PTR[24+rdi],rbp

	mov	r12,QWORD PTR[rsp]

	mov	rbx,QWORD PTR[8+rsp]

	mov	rbp,QWORD PTR[16+rsp]

	lea	rsp,QWORD PTR[24+rsp]

$L$SEH_epilogue_cneg_mod_256::
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

$L$SEH_end_cneg_mod_256::
cneg_mod_256	ENDP


PUBLIC	sub_mod_256


ALIGN	32
sub_mod_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sub_mod_256::


	push	rbp

	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	push	rbx

	sub	rsp,8

$L$SEH_body_sub_mod_256::


ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]

	sub	r8,QWORD PTR[rdx]
	mov	rax,QWORD PTR[rcx]
	sbb	r9,QWORD PTR[8+rdx]
	mov	rsi,QWORD PTR[8+rcx]
	sbb	r10,QWORD PTR[16+rdx]
	mov	rbx,QWORD PTR[16+rcx]
	sbb	r11,QWORD PTR[24+rdx]
	mov	rbp,QWORD PTR[24+rcx]
	sbb	rdx,rdx

	and	rax,rdx
	and	rsi,rdx
	and	rbx,rdx
	and	rbp,rdx

	add	r8,rax
	adc	r9,rsi
	mov	QWORD PTR[rdi],r8
	adc	r10,rbx
	mov	QWORD PTR[8+rdi],r9
	adc	r11,rbp
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11

	mov	rbx,QWORD PTR[8+rsp]

	mov	rbp,QWORD PTR[16+rsp]

	lea	rsp,QWORD PTR[24+rsp]

$L$SEH_epilogue_sub_mod_256::
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

$L$SEH_end_sub_mod_256::
sub_mod_256	ENDP


PUBLIC	check_mod_256


ALIGN	32
check_mod_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_check_mod_256::


	mov	rdi,rcx
	mov	rsi,rdx
ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	rax,QWORD PTR[rdi]
	mov	r9,QWORD PTR[8+rdi]
	mov	r10,QWORD PTR[16+rdi]
	mov	r11,QWORD PTR[24+rdi]

	mov	r8,rax
	or	rax,r9
	or	rax,r10
	or	rax,r11

	sub	r8,QWORD PTR[rsi]
	sbb	r9,QWORD PTR[8+rsi]
	sbb	r10,QWORD PTR[16+rsi]
	sbb	r11,QWORD PTR[24+rsi]
	sbb	rsi,rsi

	mov	rdx,1
	cmp	rax,0
	cmovne	rax,rdx
	and	rax,rsi
$L$SEH_epilogue_check_mod_256::
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

$L$SEH_end_check_mod_256::
check_mod_256	ENDP


PUBLIC	add_n_check_mod_256


ALIGN	32
add_n_check_mod_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_add_n_check_mod_256::


	push	rbp

	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	push	rbx

	sub	rsp,8

$L$SEH_body_add_n_check_mod_256::


ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]

	add	r8,QWORD PTR[rdx]
	adc	r9,QWORD PTR[8+rdx]
	mov	rax,r8
	adc	r10,QWORD PTR[16+rdx]
	mov	rsi,r9
	adc	r11,QWORD PTR[24+rdx]
	sbb	rdx,rdx

	mov	rbx,r10
	sub	r8,QWORD PTR[rcx]
	sbb	r9,QWORD PTR[8+rcx]
	sbb	r10,QWORD PTR[16+rcx]
	mov	rbp,r11
	sbb	r11,QWORD PTR[24+rcx]
	sbb	rdx,0

	cmovc	r8,rax
	cmovc	r9,rsi
	mov	QWORD PTR[rdi],r8
	cmovc	r10,rbx
	mov	QWORD PTR[8+rdi],r9
	cmovc	r11,rbp
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11

	or	r8,r9
	or	r10,r11
	or	r8,r10
	mov	rax,1
	cmovz	rax,r8

	mov	rbx,QWORD PTR[8+rsp]

	mov	rbp,QWORD PTR[16+rsp]

	lea	rsp,QWORD PTR[24+rsp]

$L$SEH_epilogue_add_n_check_mod_256::
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

$L$SEH_end_add_n_check_mod_256::
add_n_check_mod_256	ENDP


PUBLIC	sub_n_check_mod_256


ALIGN	32
sub_n_check_mod_256	PROC PUBLIC
	DB	243,15,30,250
	mov	QWORD PTR[8+rsp],rdi	;WIN64 prologue
	mov	QWORD PTR[16+rsp],rsi
	mov	r11,rsp
$L$SEH_begin_sub_n_check_mod_256::


	push	rbp

	mov	rdi,rcx
	mov	rsi,rdx
	mov	rdx,r8
	mov	rcx,r9
	push	rbx

	sub	rsp,8

$L$SEH_body_sub_n_check_mod_256::


ifdef	__SGX_LVI_HARDENING__
	lfence
endif
	mov	r8,QWORD PTR[rsi]
	mov	r9,QWORD PTR[8+rsi]
	mov	r10,QWORD PTR[16+rsi]
	mov	r11,QWORD PTR[24+rsi]

	sub	r8,QWORD PTR[rdx]
	mov	rax,QWORD PTR[rcx]
	sbb	r9,QWORD PTR[8+rdx]
	mov	rsi,QWORD PTR[8+rcx]
	sbb	r10,QWORD PTR[16+rdx]
	mov	rbx,QWORD PTR[16+rcx]
	sbb	r11,QWORD PTR[24+rdx]
	mov	rbp,QWORD PTR[24+rcx]
	sbb	rdx,rdx

	and	rax,rdx
	and	rsi,rdx
	and	rbx,rdx
	and	rbp,rdx

	add	r8,rax
	adc	r9,rsi
	mov	QWORD PTR[rdi],r8
	adc	r10,rbx
	mov	QWORD PTR[8+rdi],r9
	adc	r11,rbp
	mov	QWORD PTR[16+rdi],r10
	mov	QWORD PTR[24+rdi],r11

	or	r8,r9
	or	r10,r11
	or	r8,r10
	mov	rax,1
	cmovz	rax,r8

	mov	rbx,QWORD PTR[8+rsp]

	mov	rbp,QWORD PTR[16+rsp]

	lea	rsp,QWORD PTR[24+rsp]

$L$SEH_epilogue_sub_n_check_mod_256::
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

$L$SEH_end_sub_n_check_mod_256::
sub_n_check_mod_256	ENDP
.text$	ENDS
.pdata	SEGMENT READONLY ALIGN(4)
ALIGN	4
	DD	imagerel $L$SEH_begin_add_mod_256
	DD	imagerel $L$SEH_body_add_mod_256
	DD	imagerel $L$SEH_info_add_mod_256_prologue

	DD	imagerel $L$SEH_body_add_mod_256
	DD	imagerel $L$SEH_epilogue_add_mod_256
	DD	imagerel $L$SEH_info_add_mod_256_body

	DD	imagerel $L$SEH_epilogue_add_mod_256
	DD	imagerel $L$SEH_end_add_mod_256
	DD	imagerel $L$SEH_info_add_mod_256_epilogue

	DD	imagerel $L$SEH_begin_mul_by_3_mod_256
	DD	imagerel $L$SEH_body_mul_by_3_mod_256
	DD	imagerel $L$SEH_info_mul_by_3_mod_256_prologue

	DD	imagerel $L$SEH_body_mul_by_3_mod_256
	DD	imagerel $L$SEH_epilogue_mul_by_3_mod_256
	DD	imagerel $L$SEH_info_mul_by_3_mod_256_body

	DD	imagerel $L$SEH_epilogue_mul_by_3_mod_256
	DD	imagerel $L$SEH_end_mul_by_3_mod_256
	DD	imagerel $L$SEH_info_mul_by_3_mod_256_epilogue

	DD	imagerel $L$SEH_begin_lshift_mod_256
	DD	imagerel $L$SEH_body_lshift_mod_256
	DD	imagerel $L$SEH_info_lshift_mod_256_prologue

	DD	imagerel $L$SEH_body_lshift_mod_256
	DD	imagerel $L$SEH_epilogue_lshift_mod_256
	DD	imagerel $L$SEH_info_lshift_mod_256_body

	DD	imagerel $L$SEH_epilogue_lshift_mod_256
	DD	imagerel $L$SEH_end_lshift_mod_256
	DD	imagerel $L$SEH_info_lshift_mod_256_epilogue

	DD	imagerel $L$SEH_begin_rshift_mod_256
	DD	imagerel $L$SEH_body_rshift_mod_256
	DD	imagerel $L$SEH_info_rshift_mod_256_prologue

	DD	imagerel $L$SEH_body_rshift_mod_256
	DD	imagerel $L$SEH_epilogue_rshift_mod_256
	DD	imagerel $L$SEH_info_rshift_mod_256_body

	DD	imagerel $L$SEH_epilogue_rshift_mod_256
	DD	imagerel $L$SEH_end_rshift_mod_256
	DD	imagerel $L$SEH_info_rshift_mod_256_epilogue

	DD	imagerel $L$SEH_begin_cneg_mod_256
	DD	imagerel $L$SEH_body_cneg_mod_256
	DD	imagerel $L$SEH_info_cneg_mod_256_prologue

	DD	imagerel $L$SEH_body_cneg_mod_256
	DD	imagerel $L$SEH_epilogue_cneg_mod_256
	DD	imagerel $L$SEH_info_cneg_mod_256_body

	DD	imagerel $L$SEH_epilogue_cneg_mod_256
	DD	imagerel $L$SEH_end_cneg_mod_256
	DD	imagerel $L$SEH_info_cneg_mod_256_epilogue

	DD	imagerel $L$SEH_begin_sub_mod_256
	DD	imagerel $L$SEH_body_sub_mod_256
	DD	imagerel $L$SEH_info_sub_mod_256_prologue

	DD	imagerel $L$SEH_body_sub_mod_256
	DD	imagerel $L$SEH_epilogue_sub_mod_256
	DD	imagerel $L$SEH_info_sub_mod_256_body

	DD	imagerel $L$SEH_epilogue_sub_mod_256
	DD	imagerel $L$SEH_end_sub_mod_256
	DD	imagerel $L$SEH_info_sub_mod_256_epilogue

	DD	imagerel $L$SEH_epilogue_check_mod_256
	DD	imagerel $L$SEH_end_check_mod_256
	DD	imagerel $L$SEH_info_check_mod_256_epilogue

	DD	imagerel $L$SEH_begin_add_n_check_mod_256
	DD	imagerel $L$SEH_body_add_n_check_mod_256
	DD	imagerel $L$SEH_info_add_n_check_mod_256_prologue

	DD	imagerel $L$SEH_body_add_n_check_mod_256
	DD	imagerel $L$SEH_epilogue_add_n_check_mod_256
	DD	imagerel $L$SEH_info_add_n_check_mod_256_body

	DD	imagerel $L$SEH_epilogue_add_n_check_mod_256
	DD	imagerel $L$SEH_end_add_n_check_mod_256
	DD	imagerel $L$SEH_info_add_n_check_mod_256_epilogue

	DD	imagerel $L$SEH_begin_sub_n_check_mod_256
	DD	imagerel $L$SEH_body_sub_n_check_mod_256
	DD	imagerel $L$SEH_info_sub_n_check_mod_256_prologue

	DD	imagerel $L$SEH_body_sub_n_check_mod_256
	DD	imagerel $L$SEH_epilogue_sub_n_check_mod_256
	DD	imagerel $L$SEH_info_sub_n_check_mod_256_body

	DD	imagerel $L$SEH_epilogue_sub_n_check_mod_256
	DD	imagerel $L$SEH_end_sub_n_check_mod_256
	DD	imagerel $L$SEH_info_sub_n_check_mod_256_epilogue

.pdata	ENDS
.xdata	SEGMENT READONLY ALIGN(8)
ALIGN	8
$L$SEH_info_add_mod_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_add_mod_256_body::
DB	1,0,9,0
DB	000h,034h,001h,000h
DB	000h,054h,002h,000h
DB	000h,074h,004h,000h
DB	000h,064h,005h,000h
DB	000h,022h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_add_mod_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_mul_by_3_mod_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_mul_by_3_mod_256_body::
DB	1,0,11,0
DB	000h,0c4h,000h,000h
DB	000h,034h,001h,000h
DB	000h,054h,002h,000h
DB	000h,074h,004h,000h
DB	000h,064h,005h,000h
DB	000h,022h
DB	000h,000h,000h,000h,000h,000h
$L$SEH_info_mul_by_3_mod_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_lshift_mod_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_lshift_mod_256_body::
DB	1,0,11,0
DB	000h,0c4h,000h,000h
DB	000h,034h,001h,000h
DB	000h,054h,002h,000h
DB	000h,074h,004h,000h
DB	000h,064h,005h,000h
DB	000h,022h
DB	000h,000h,000h,000h,000h,000h
$L$SEH_info_lshift_mod_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_rshift_mod_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_rshift_mod_256_body::
DB	1,0,9,0
DB	000h,034h,001h,000h
DB	000h,054h,002h,000h
DB	000h,074h,004h,000h
DB	000h,064h,005h,000h
DB	000h,022h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_rshift_mod_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_cneg_mod_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_cneg_mod_256_body::
DB	1,0,11,0
DB	000h,0c4h,000h,000h
DB	000h,034h,001h,000h
DB	000h,054h,002h,000h
DB	000h,074h,004h,000h
DB	000h,064h,005h,000h
DB	000h,022h
DB	000h,000h,000h,000h,000h,000h
$L$SEH_info_cneg_mod_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sub_mod_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sub_mod_256_body::
DB	1,0,9,0
DB	000h,034h,001h,000h
DB	000h,054h,002h,000h
DB	000h,074h,004h,000h
DB	000h,064h,005h,000h
DB	000h,022h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sub_mod_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_check_mod_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_add_n_check_mod_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_add_n_check_mod_256_body::
DB	1,0,9,0
DB	000h,034h,001h,000h
DB	000h,054h,002h,000h
DB	000h,074h,004h,000h
DB	000h,064h,005h,000h
DB	000h,022h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_add_n_check_mod_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h

$L$SEH_info_sub_n_check_mod_256_prologue::
DB	1,0,5,00bh
DB	0,074h,1,0
DB	0,064h,2,0
DB	0,0b3h
DB	0,0
	DD	0,0
$L$SEH_info_sub_n_check_mod_256_body::
DB	1,0,9,0
DB	000h,034h,001h,000h
DB	000h,054h,002h,000h
DB	000h,074h,004h,000h
DB	000h,064h,005h,000h
DB	000h,022h
DB	000h,000h,000h,000h,000h,000h
DB	000h,000h,000h,000h
$L$SEH_info_sub_n_check_mod_256_epilogue::
DB	1,0,4,0
DB	000h,074h,001h,000h
DB	000h,064h,002h,000h
DB	000h,000h,000h,000h


.xdata	ENDS
END
