; This file is generated from a similarly-named Perl script in the BoringSSL
; source tree. Do not edit by hand.

%ifdef BORINGSSL_PREFIX
%include "boringssl_prefix_symbols_nasm.inc"
%endif
%ifidn __OUTPUT_FORMAT__,obj
section	code	use32 class=code align=64
%elifidn __OUTPUT_FORMAT__,win32
$@feat.00 equ 1
section	.text	code align=64
%else
section	.text	code
%endif
;extern	_GFp_ia32cap_P
L$ONE_mont:
dd	1,0,0,-1,-1,-1,-2,0
align	16
__ecp_nistz256_div_by_2:
	mov	ebp,DWORD [esi]
	xor	edx,edx
	mov	ebx,DWORD [4+esi]
	mov	eax,ebp
	and	ebp,1
	mov	ecx,DWORD [8+esi]
	sub	edx,ebp
	add	eax,edx
	adc	ebx,edx
	mov	DWORD [edi],eax
	adc	ecx,edx
	mov	DWORD [4+edi],ebx
	mov	DWORD [8+edi],ecx
	mov	eax,DWORD [12+esi]
	mov	ebx,DWORD [16+esi]
	adc	eax,0
	mov	ecx,DWORD [20+esi]
	adc	ebx,0
	mov	DWORD [12+edi],eax
	adc	ecx,0
	mov	DWORD [16+edi],ebx
	mov	DWORD [20+edi],ecx
	mov	eax,DWORD [24+esi]
	mov	ebx,DWORD [28+esi]
	adc	eax,ebp
	adc	ebx,edx
	mov	DWORD [24+edi],eax
	sbb	esi,esi
	mov	DWORD [28+edi],ebx
	mov	eax,DWORD [edi]
	mov	ebx,DWORD [4+edi]
	mov	ecx,DWORD [8+edi]
	mov	edx,DWORD [12+edi]
	shr	eax,1
	mov	ebp,ebx
	shl	ebx,31
	or	eax,ebx
	shr	ebp,1
	mov	ebx,ecx
	shl	ecx,31
	mov	DWORD [edi],eax
	or	ebp,ecx
	mov	eax,DWORD [16+edi]
	shr	ebx,1
	mov	ecx,edx
	shl	edx,31
	mov	DWORD [4+edi],ebp
	or	ebx,edx
	mov	ebp,DWORD [20+edi]
	shr	ecx,1
	mov	edx,eax
	shl	eax,31
	mov	DWORD [8+edi],ebx
	or	ecx,eax
	mov	ebx,DWORD [24+edi]
	shr	edx,1
	mov	eax,ebp
	shl	ebp,31
	mov	DWORD [12+edi],ecx
	or	edx,ebp
	mov	ecx,DWORD [28+edi]
	shr	eax,1
	mov	ebp,ebx
	shl	ebx,31
	mov	DWORD [16+edi],edx
	or	eax,ebx
	shr	ebp,1
	mov	ebx,ecx
	shl	ecx,31
	mov	DWORD [20+edi],eax
	or	ebp,ecx
	shr	ebx,1
	shl	esi,31
	mov	DWORD [24+edi],ebp
	or	ebx,esi
	mov	DWORD [28+edi],ebx
	ret
global	_GFp_nistz256_add
align	16
_GFp_nistz256_add:
L$_GFp_nistz256_add_begin:
	push	ebp
	push	ebx
	push	esi
	push	edi
	mov	esi,DWORD [24+esp]
	mov	ebp,DWORD [28+esp]
	mov	edi,DWORD [20+esp]
	call	__ecp_nistz256_add
	pop	edi
	pop	esi
	pop	ebx
	pop	ebp
	ret
align	16
__ecp_nistz256_add:
	mov	eax,DWORD [esi]
	mov	ebx,DWORD [4+esi]
	mov	ecx,DWORD [8+esi]
	add	eax,DWORD [ebp]
	mov	edx,DWORD [12+esi]
	adc	ebx,DWORD [4+ebp]
	mov	DWORD [edi],eax
	adc	ecx,DWORD [8+ebp]
	mov	DWORD [4+edi],ebx
	adc	edx,DWORD [12+ebp]
	mov	DWORD [8+edi],ecx
	mov	DWORD [12+edi],edx
	mov	eax,DWORD [16+esi]
	mov	ebx,DWORD [20+esi]
	mov	ecx,DWORD [24+esi]
	adc	eax,DWORD [16+ebp]
	mov	edx,DWORD [28+esi]
	adc	ebx,DWORD [20+ebp]
	mov	DWORD [16+edi],eax
	adc	ecx,DWORD [24+ebp]
	mov	DWORD [20+edi],ebx
	mov	esi,0
	adc	edx,DWORD [28+ebp]
	mov	DWORD [24+edi],ecx
	adc	esi,0
	mov	DWORD [28+edi],edx
	mov	eax,DWORD [edi]
	mov	ebx,DWORD [4+edi]
	mov	ecx,DWORD [8+edi]
	sub	eax,-1
	mov	edx,DWORD [12+edi]
	sbb	ebx,-1
	mov	eax,DWORD [16+edi]
	sbb	ecx,-1
	mov	ebx,DWORD [20+edi]
	sbb	edx,0
	mov	ecx,DWORD [24+edi]
	sbb	eax,0
	mov	edx,DWORD [28+edi]
	sbb	ebx,0
	sbb	ecx,1
	sbb	edx,-1
	sbb	esi,0
	not	esi
	mov	eax,DWORD [edi]
	mov	ebp,esi
	mov	ebx,DWORD [4+edi]
	shr	ebp,31
	mov	ecx,DWORD [8+edi]
	sub	eax,esi
	mov	edx,DWORD [12+edi]
	sbb	ebx,esi
	mov	DWORD [edi],eax
	sbb	ecx,esi
	mov	DWORD [4+edi],ebx
	sbb	edx,0
	mov	DWORD [8+edi],ecx
	mov	DWORD [12+edi],edx
	mov	eax,DWORD [16+edi]
	mov	ebx,DWORD [20+edi]
	mov	ecx,DWORD [24+edi]
	sbb	eax,0
	mov	edx,DWORD [28+edi]
	sbb	ebx,0
	mov	DWORD [16+edi],eax
	sbb	ecx,ebp
	mov	DWORD [20+edi],ebx
	sbb	edx,esi
	mov	DWORD [24+edi],ecx
	mov	DWORD [28+edi],edx
	ret
align	16
__ecp_nistz256_sub:
	mov	eax,DWORD [esi]
	mov	ebx,DWORD [4+esi]
	mov	ecx,DWORD [8+esi]
	sub	eax,DWORD [ebp]
	mov	edx,DWORD [12+esi]
	sbb	ebx,DWORD [4+ebp]
	mov	DWORD [edi],eax
	sbb	ecx,DWORD [8+ebp]
	mov	DWORD [4+edi],ebx
	sbb	edx,DWORD [12+ebp]
	mov	DWORD [8+edi],ecx
	mov	DWORD [12+edi],edx
	mov	eax,DWORD [16+esi]
	mov	ebx,DWORD [20+esi]
	mov	ecx,DWORD [24+esi]
	sbb	eax,DWORD [16+ebp]
	mov	edx,DWORD [28+esi]
	sbb	ebx,DWORD [20+ebp]
	sbb	ecx,DWORD [24+ebp]
	mov	DWORD [16+edi],eax
	sbb	edx,DWORD [28+ebp]
	mov	DWORD [20+edi],ebx
	sbb	esi,esi
	mov	DWORD [24+edi],ecx
	mov	DWORD [28+edi],edx
	mov	eax,DWORD [edi]
	mov	ebp,esi
	mov	ebx,DWORD [4+edi]
	shr	ebp,31
	mov	ecx,DWORD [8+edi]
	add	eax,esi
	mov	edx,DWORD [12+edi]
	adc	ebx,esi
	mov	DWORD [edi],eax
	adc	ecx,esi
	mov	DWORD [4+edi],ebx
	adc	edx,0
	mov	DWORD [8+edi],ecx
	mov	DWORD [12+edi],edx
	mov	eax,DWORD [16+edi]
	mov	ebx,DWORD [20+edi]
	mov	ecx,DWORD [24+edi]
	adc	eax,0
	mov	edx,DWORD [28+edi]
	adc	ebx,0
	mov	DWORD [16+edi],eax
	adc	ecx,ebp
	mov	DWORD [20+edi],ebx
	adc	edx,esi
	mov	DWORD [24+edi],ecx
	mov	DWORD [28+edi],edx
	ret
global	_GFp_nistz256_neg
align	16
_GFp_nistz256_neg:
L$_GFp_nistz256_neg_begin:
	push	ebp
	push	ebx
	push	esi
	push	edi
	mov	ebp,DWORD [24+esp]
	mov	edi,DWORD [20+esp]
	xor	eax,eax
	sub	esp,32
	mov	DWORD [esp],eax
	mov	esi,esp
	mov	DWORD [4+esp],eax
	mov	DWORD [8+esp],eax
	mov	DWORD [12+esp],eax
	mov	DWORD [16+esp],eax
	mov	DWORD [20+esp],eax
	mov	DWORD [24+esp],eax
	mov	DWORD [28+esp],eax
	call	__ecp_nistz256_sub
	add	esp,32
	pop	edi
	pop	esi
	pop	ebx
	pop	ebp
	ret
align	16
__picup_eax:
	mov	eax,DWORD [esp]
	ret
global	_GFp_nistz256_mul_mont
align	16
_GFp_nistz256_mul_mont:
L$_GFp_nistz256_mul_mont_begin:
	push	ebp
	push	ebx
	push	esi
	push	edi
	mov	esi,DWORD [24+esp]
	mov	ebp,DWORD [28+esp]
	call	__picup_eax
L$000pic:
	lea	eax,[_GFp_ia32cap_P]
	mov	eax,DWORD [eax]
	mov	edi,DWORD [20+esp]
	call	__ecp_nistz256_mul_mont
	pop	edi
	pop	esi
	pop	ebx
	pop	ebp
	ret
align	16
__ecp_nistz256_mul_mont:
	mov	edx,esp
	sub	esp,256
	movd	xmm7,DWORD [ebp]
	lea	ebp,[4+ebp]
	pcmpeqd	xmm6,xmm6
	psrlq	xmm6,48
	pshuflw	xmm7,xmm7,220
	and	esp,-64
	pshufd	xmm7,xmm7,220
	lea	ebx,[128+esp]
	movd	xmm0,DWORD [esi]
	pshufd	xmm0,xmm0,204
	movd	xmm1,DWORD [4+esi]
	movdqa	[ebx],xmm0
	pmuludq	xmm0,xmm7
	movd	xmm2,DWORD [8+esi]
	pshufd	xmm1,xmm1,204
	movdqa	[16+ebx],xmm1
	pmuludq	xmm1,xmm7
	movq	xmm4,xmm0
	pslldq	xmm4,6
	paddq	xmm4,xmm0
	movdqa	xmm5,xmm4
	psrldq	xmm4,10
	pand	xmm5,xmm6
	movd	xmm3,DWORD [12+esi]
	pshufd	xmm2,xmm2,204
	movdqa	[32+ebx],xmm2
	pmuludq	xmm2,xmm7
	paddq	xmm1,xmm4
	movdqa	[esp],xmm1
	movd	xmm0,DWORD [16+esi]
	pshufd	xmm3,xmm3,204
	movdqa	[48+ebx],xmm3
	pmuludq	xmm3,xmm7
	movdqa	[16+esp],xmm2
	movd	xmm1,DWORD [20+esi]
	pshufd	xmm0,xmm0,204
	movdqa	[64+ebx],xmm0
	pmuludq	xmm0,xmm7
	paddq	xmm3,xmm5
	movdqa	[32+esp],xmm3
	movd	xmm2,DWORD [24+esi]
	pshufd	xmm1,xmm1,204
	movdqa	[80+ebx],xmm1
	pmuludq	xmm1,xmm7
	movdqa	[48+esp],xmm0
	pshufd	xmm4,xmm5,177
	movd	xmm3,DWORD [28+esi]
	pshufd	xmm2,xmm2,204
	movdqa	[96+ebx],xmm2
	pmuludq	xmm2,xmm7
	movdqa	[64+esp],xmm1
	psubq	xmm4,xmm5
	movd	xmm0,DWORD [ebp]
	pshufd	xmm3,xmm3,204
	movdqa	[112+ebx],xmm3
	pmuludq	xmm3,xmm7
	pshuflw	xmm7,xmm0,220
	movdqa	xmm0,[ebx]
	pshufd	xmm7,xmm7,220
	mov	ecx,6
	lea	ebp,[4+ebp]
	jmp	NEAR L$001madd_sse2
align	16
L$001madd_sse2:
	paddq	xmm2,xmm5
	paddq	xmm3,xmm4
	movdqa	xmm1,[16+ebx]
	pmuludq	xmm0,xmm7
	movdqa	[80+esp],xmm2
	movdqa	xmm2,[32+ebx]
	pmuludq	xmm1,xmm7
	movdqa	[96+esp],xmm3
	paddq	xmm0,[esp]
	movdqa	xmm3,[48+ebx]
	pmuludq	xmm2,xmm7
	movq	xmm4,xmm0
	pslldq	xmm4,6
	paddq	xmm1,[16+esp]
	paddq	xmm4,xmm0
	movdqa	xmm5,xmm4
	psrldq	xmm4,10
	movdqa	xmm0,[64+ebx]
	pmuludq	xmm3,xmm7
	paddq	xmm1,xmm4
	paddq	xmm2,[32+esp]
	movdqa	[esp],xmm1
	movdqa	xmm1,[80+ebx]
	pmuludq	xmm0,xmm7
	paddq	xmm3,[48+esp]
	movdqa	[16+esp],xmm2
	pand	xmm5,xmm6
	movdqa	xmm2,[96+ebx]
	pmuludq	xmm1,xmm7
	paddq	xmm3,xmm5
	paddq	xmm0,[64+esp]
	movdqa	[32+esp],xmm3
	pshufd	xmm4,xmm5,177
	movdqa	xmm3,xmm7
	pmuludq	xmm2,xmm7
	movd	xmm7,DWORD [ebp]
	lea	ebp,[4+ebp]
	paddq	xmm1,[80+esp]
	psubq	xmm4,xmm5
	movdqa	[48+esp],xmm0
	pshuflw	xmm7,xmm7,220
	pmuludq	xmm3,[112+ebx]
	pshufd	xmm7,xmm7,220
	movdqa	xmm0,[ebx]
	movdqa	[64+esp],xmm1
	paddq	xmm2,[96+esp]
	dec	ecx
	jnz	NEAR L$001madd_sse2
	paddq	xmm2,xmm5
	paddq	xmm3,xmm4
	movdqa	xmm1,[16+ebx]
	pmuludq	xmm0,xmm7
	movdqa	[80+esp],xmm2
	movdqa	xmm2,[32+ebx]
	pmuludq	xmm1,xmm7
	movdqa	[96+esp],xmm3
	paddq	xmm0,[esp]
	movdqa	xmm3,[48+ebx]
	pmuludq	xmm2,xmm7
	movq	xmm4,xmm0
	pslldq	xmm4,6
	paddq	xmm1,[16+esp]
	paddq	xmm4,xmm0
	movdqa	xmm5,xmm4
	psrldq	xmm4,10
	movdqa	xmm0,[64+ebx]
	pmuludq	xmm3,xmm7
	paddq	xmm1,xmm4
	paddq	xmm2,[32+esp]
	movdqa	[esp],xmm1
	movdqa	xmm1,[80+ebx]
	pmuludq	xmm0,xmm7
	paddq	xmm3,[48+esp]
	movdqa	[16+esp],xmm2
	pand	xmm5,xmm6
	movdqa	xmm2,[96+ebx]
	pmuludq	xmm1,xmm7
	paddq	xmm3,xmm5
	paddq	xmm0,[64+esp]
	movdqa	[32+esp],xmm3
	pshufd	xmm4,xmm5,177
	movdqa	xmm3,[112+ebx]
	pmuludq	xmm2,xmm7
	paddq	xmm1,[80+esp]
	psubq	xmm4,xmm5
	movdqa	[48+esp],xmm0
	pmuludq	xmm3,xmm7
	pcmpeqd	xmm7,xmm7
	movdqa	xmm0,[esp]
	pslldq	xmm7,8
	movdqa	[64+esp],xmm1
	paddq	xmm2,[96+esp]
	paddq	xmm2,xmm5
	paddq	xmm3,xmm4
	movdqa	[80+esp],xmm2
	movdqa	[96+esp],xmm3
	movdqa	xmm1,[16+esp]
	movdqa	xmm2,[32+esp]
	movdqa	xmm3,[48+esp]
	movq	xmm4,xmm0
	pand	xmm0,xmm7
	xor	ebp,ebp
	pslldq	xmm4,6
	movq	xmm5,xmm1
	paddq	xmm0,xmm4
	pand	xmm1,xmm7
	psrldq	xmm0,6
	movd	eax,xmm0
	psrldq	xmm0,4
	paddq	xmm5,xmm0
	movdqa	xmm0,[64+esp]
	sub	eax,-1
	pslldq	xmm5,6
	movq	xmm4,xmm2
	paddq	xmm1,xmm5
	pand	xmm2,xmm7
	psrldq	xmm1,6
	mov	DWORD [edi],eax
	movd	eax,xmm1
	psrldq	xmm1,4
	paddq	xmm4,xmm1
	movdqa	xmm1,[80+esp]
	sbb	eax,-1
	pslldq	xmm4,6
	movq	xmm5,xmm3
	paddq	xmm2,xmm4
	pand	xmm3,xmm7
	psrldq	xmm2,6
	mov	DWORD [4+edi],eax
	movd	eax,xmm2
	psrldq	xmm2,4
	paddq	xmm5,xmm2
	movdqa	xmm2,[96+esp]
	sbb	eax,-1
	pslldq	xmm5,6
	movq	xmm4,xmm0
	paddq	xmm3,xmm5
	pand	xmm0,xmm7
	psrldq	xmm3,6
	mov	DWORD [8+edi],eax
	movd	eax,xmm3
	psrldq	xmm3,4
	paddq	xmm4,xmm3
	sbb	eax,0
	pslldq	xmm4,6
	movq	xmm5,xmm1
	paddq	xmm0,xmm4
	pand	xmm1,xmm7
	psrldq	xmm0,6
	mov	DWORD [12+edi],eax
	movd	eax,xmm0
	psrldq	xmm0,4
	paddq	xmm5,xmm0
	sbb	eax,0
	pslldq	xmm5,6
	movq	xmm4,xmm2
	paddq	xmm1,xmm5
	pand	xmm2,xmm7
	psrldq	xmm1,6
	movd	ebx,xmm1
	psrldq	xmm1,4
	mov	esp,edx
	paddq	xmm4,xmm1
	pslldq	xmm4,6
	paddq	xmm2,xmm4
	psrldq	xmm2,6
	movd	ecx,xmm2
	psrldq	xmm2,4
	sbb	ebx,0
	movd	edx,xmm2
	pextrw	esi,xmm2,2
	sbb	ecx,1
	sbb	edx,-1
	sbb	esi,0
	sub	ebp,esi
	add	DWORD [edi],esi
	adc	DWORD [4+edi],esi
	adc	DWORD [8+edi],esi
	adc	DWORD [12+edi],0
	adc	eax,0
	adc	ebx,0
	mov	DWORD [16+edi],eax
	adc	ecx,ebp
	mov	DWORD [20+edi],ebx
	adc	edx,esi
	mov	DWORD [24+edi],ecx
	mov	DWORD [28+edi],edx
	ret
global	_GFp_nistz256_point_double
align	16
_GFp_nistz256_point_double:
L$_GFp_nistz256_point_double_begin:
	push	ebp
	push	ebx
	push	esi
	push	edi
	mov	esi,DWORD [24+esp]
	sub	esp,164
	call	__picup_eax
L$002pic:
	lea	edx,[_GFp_ia32cap_P]
	mov	ebp,DWORD [edx]
L$point_double_shortcut:
	mov	eax,DWORD [esi]
	mov	ebx,DWORD [4+esi]
	mov	ecx,DWORD [8+esi]
	mov	edx,DWORD [12+esi]
	mov	DWORD [96+esp],eax
	mov	DWORD [100+esp],ebx
	mov	DWORD [104+esp],ecx
	mov	DWORD [108+esp],edx
	mov	eax,DWORD [16+esi]
	mov	ebx,DWORD [20+esi]
	mov	ecx,DWORD [24+esi]
	mov	edx,DWORD [28+esi]
	mov	DWORD [112+esp],eax
	mov	DWORD [116+esp],ebx
	mov	DWORD [120+esp],ecx
	mov	DWORD [124+esp],edx
	mov	DWORD [160+esp],ebp
	lea	ebp,[32+esi]
	lea	esi,[32+esi]
	lea	edi,[esp]
	call	__ecp_nistz256_add
	mov	eax,DWORD [160+esp]
	mov	esi,64
	add	esi,DWORD [188+esp]
	lea	edi,[64+esp]
	mov	ebp,esi
	call	__ecp_nistz256_mul_mont
	mov	eax,DWORD [160+esp]
	lea	esi,[esp]
	lea	ebp,[esp]
	lea	edi,[esp]
	call	__ecp_nistz256_mul_mont
	mov	eax,DWORD [160+esp]
	mov	ebp,DWORD [188+esp]
	lea	esi,[32+ebp]
	lea	ebp,[64+ebp]
	lea	edi,[128+esp]
	call	__ecp_nistz256_mul_mont
	lea	esi,[96+esp]
	lea	ebp,[64+esp]
	lea	edi,[32+esp]
	call	__ecp_nistz256_add
	mov	edi,64
	lea	esi,[128+esp]
	lea	ebp,[128+esp]
	add	edi,DWORD [184+esp]
	call	__ecp_nistz256_add
	lea	esi,[96+esp]
	lea	ebp,[64+esp]
	lea	edi,[64+esp]
	call	__ecp_nistz256_sub
	mov	eax,DWORD [160+esp]
	lea	esi,[esp]
	lea	ebp,[esp]
	lea	edi,[128+esp]
	call	__ecp_nistz256_mul_mont
	mov	eax,DWORD [160+esp]
	lea	esi,[32+esp]
	lea	ebp,[64+esp]
	lea	edi,[32+esp]
	call	__ecp_nistz256_mul_mont
	mov	edi,32
	lea	esi,[128+esp]
	add	edi,DWORD [184+esp]
	call	__ecp_nistz256_div_by_2
	lea	esi,[32+esp]
	lea	ebp,[32+esp]
	lea	edi,[128+esp]
	call	__ecp_nistz256_add
	mov	eax,DWORD [160+esp]
	lea	esi,[96+esp]
	lea	ebp,[esp]
	lea	edi,[esp]
	call	__ecp_nistz256_mul_mont
	lea	esi,[128+esp]
	lea	ebp,[32+esp]
	lea	edi,[32+esp]
	call	__ecp_nistz256_add
	lea	esi,[esp]
	lea	ebp,[esp]
	lea	edi,[128+esp]
	call	__ecp_nistz256_add
	mov	eax,DWORD [160+esp]
	lea	esi,[32+esp]
	lea	ebp,[32+esp]
	mov	edi,DWORD [184+esp]
	call	__ecp_nistz256_mul_mont
	mov	esi,edi
	lea	ebp,[128+esp]
	call	__ecp_nistz256_sub
	lea	esi,[esp]
	mov	ebp,edi
	lea	edi,[esp]
	call	__ecp_nistz256_sub
	mov	eax,DWORD [160+esp]
	mov	esi,edi
	lea	ebp,[32+esp]
	call	__ecp_nistz256_mul_mont
	mov	ebp,32
	lea	esi,[esp]
	add	ebp,DWORD [184+esp]
	mov	edi,ebp
	call	__ecp_nistz256_sub
	add	esp,164
	pop	edi
	pop	esi
	pop	ebx
	pop	ebp
	ret
global	_GFp_nistz256_point_add_affine
align	16
_GFp_nistz256_point_add_affine:
L$_GFp_nistz256_point_add_affine_begin:
	push	ebp
	push	ebx
	push	esi
	push	edi
	mov	esi,DWORD [24+esp]
	sub	esp,492
	call	__picup_eax
L$003pic:
	lea	edx,[_GFp_ia32cap_P]
	mov	ebp,DWORD [edx]
	lea	edi,[96+esp]
	mov	eax,DWORD [esi]
	mov	ebx,DWORD [4+esi]
	mov	ecx,DWORD [8+esi]
	mov	edx,DWORD [12+esi]
	mov	DWORD [edi],eax
	mov	DWORD [488+esp],ebp
	mov	DWORD [4+edi],ebx
	mov	DWORD [8+edi],ecx
	mov	DWORD [12+edi],edx
	mov	eax,DWORD [16+esi]
	mov	ebx,DWORD [20+esi]
	mov	ecx,DWORD [24+esi]
	mov	edx,DWORD [28+esi]
	mov	DWORD [16+edi],eax
	mov	DWORD [20+edi],ebx
	mov	DWORD [24+edi],ecx
	mov	DWORD [28+edi],edx
	mov	eax,DWORD [32+esi]
	mov	ebx,DWORD [36+esi]
	mov	ecx,DWORD [40+esi]
	mov	edx,DWORD [44+esi]
	mov	DWORD [32+edi],eax
	mov	DWORD [36+edi],ebx
	mov	DWORD [40+edi],ecx
	mov	DWORD [44+edi],edx
	mov	eax,DWORD [48+esi]
	mov	ebx,DWORD [52+esi]
	mov	ecx,DWORD [56+esi]
	mov	edx,DWORD [60+esi]
	mov	DWORD [48+edi],eax
	mov	DWORD [52+edi],ebx
	mov	DWORD [56+edi],ecx
	mov	DWORD [60+edi],edx
	mov	eax,DWORD [64+esi]
	mov	ebx,DWORD [68+esi]
	mov	ecx,DWORD [72+esi]
	mov	edx,DWORD [76+esi]
	mov	DWORD [64+edi],eax
	mov	ebp,eax
	mov	DWORD [68+edi],ebx
	or	ebp,ebx
	mov	DWORD [72+edi],ecx
	or	ebp,ecx
	mov	DWORD [76+edi],edx
	or	ebp,edx
	mov	eax,DWORD [80+esi]
	mov	ebx,DWORD [84+esi]
	mov	ecx,DWORD [88+esi]
	mov	edx,DWORD [92+esi]
	mov	DWORD [80+edi],eax
	or	ebp,eax
	mov	DWORD [84+edi],ebx
	or	ebp,ebx
	mov	DWORD [88+edi],ecx
	or	ebp,ecx
	mov	DWORD [92+edi],edx
	or	ebp,edx
	xor	eax,eax
	mov	esi,DWORD [520+esp]
	sub	eax,ebp
	or	ebp,eax
	sar	ebp,31
	mov	DWORD [480+esp],ebp
	lea	edi,[192+esp]
	mov	eax,DWORD [esi]
	mov	ebx,DWORD [4+esi]
	mov	ecx,DWORD [8+esi]
	mov	edx,DWORD [12+esi]
	mov	DWORD [edi],eax
	mov	ebp,eax
	mov	DWORD [4+edi],ebx
	or	ebp,ebx
	mov	DWORD [8+edi],ecx
	or	ebp,ecx
	mov	DWORD [12+edi],edx
	or	ebp,edx
	mov	eax,DWORD [16+esi]
	mov	ebx,DWORD [20+esi]
	mov	ecx,DWORD [24+esi]
	mov	edx,DWORD [28+esi]
	mov	DWORD [16+edi],eax
	or	ebp,eax
	mov	DWORD [20+edi],ebx
	or	ebp,ebx
	mov	DWORD [24+edi],ecx
	or	ebp,ecx
	mov	DWORD [28+edi],edx
	or	ebp,edx
	mov	eax,DWORD [32+esi]
	mov	ebx,DWORD [36+esi]
	mov	ecx,DWORD [40+esi]
	mov	edx,DWORD [44+esi]
	mov	DWORD [32+edi],eax
	or	ebp,eax
	mov	DWORD [36+edi],ebx
	or	ebp,ebx
	mov	DWORD [40+edi],ecx
	or	ebp,ecx
	mov	DWORD [44+edi],edx
	or	ebp,edx
	mov	eax,DWORD [48+esi]
	mov	ebx,DWORD [52+esi]
	mov	ecx,DWORD [56+esi]
	mov	edx,DWORD [60+esi]
	mov	DWORD [48+edi],eax
	or	ebp,eax
	mov	DWORD [52+edi],ebx
	or	ebp,ebx
	mov	DWORD [56+edi],ecx
	or	ebp,ecx
	mov	DWORD [60+edi],edx
	or	ebp,edx
	xor	ebx,ebx
	mov	eax,DWORD [488+esp]
	sub	ebx,ebp
	lea	esi,[160+esp]
	or	ebx,ebp
	lea	ebp,[160+esp]
	sar	ebx,31
	lea	edi,[288+esp]
	mov	DWORD [484+esp],ebx
	call	__ecp_nistz256_mul_mont
	mov	eax,DWORD [488+esp]
	lea	esi,[192+esp]
	mov	ebp,edi
	lea	edi,[256+esp]
	call	__ecp_nistz256_mul_mont
	mov	eax,DWORD [488+esp]
	lea	esi,[160+esp]
	lea	ebp,[288+esp]
	lea	edi,[288+esp]
	call	__ecp_nistz256_mul_mont
	lea	esi,[256+esp]
	lea	ebp,[96+esp]
	lea	edi,[320+esp]
	call	__ecp_nistz256_sub
	mov	eax,DWORD [488+esp]
	lea	esi,[224+esp]
	lea	ebp,[288+esp]
	lea	edi,[288+esp]
	call	__ecp_nistz256_mul_mont
	mov	eax,DWORD [488+esp]
	lea	esi,[160+esp]
	lea	ebp,[320+esp]
	lea	edi,[64+esp]
	call	__ecp_nistz256_mul_mont
	lea	esi,[288+esp]
	lea	ebp,[128+esp]
	lea	edi,[352+esp]
	call	__ecp_nistz256_sub
	mov	eax,DWORD [488+esp]
	lea	esi,[320+esp]
	lea	ebp,[320+esp]
	lea	edi,[384+esp]
	call	__ecp_nistz256_mul_mont
	mov	eax,DWORD [488+esp]
	lea	esi,[352+esp]
	lea	ebp,[352+esp]
	lea	edi,[448+esp]
	call	__ecp_nistz256_mul_mont
	mov	eax,DWORD [488+esp]
	lea	esi,[96+esp]
	lea	ebp,[384+esp]
	lea	edi,[256+esp]
	call	__ecp_nistz256_mul_mont
	mov	eax,DWORD [488+esp]
	lea	esi,[320+esp]
	lea	ebp,[384+esp]
	lea	edi,[416+esp]
	call	__ecp_nistz256_mul_mont
	lea	esi,[256+esp]
	lea	ebp,[256+esp]
	lea	edi,[384+esp]
	call	__ecp_nistz256_add
	lea	esi,[448+esp]
	lea	ebp,[384+esp]
	lea	edi,[esp]
	call	__ecp_nistz256_sub
	lea	esi,[esp]
	lea	ebp,[416+esp]
	lea	edi,[esp]
	call	__ecp_nistz256_sub
	lea	esi,[256+esp]
	lea	ebp,[esp]
	lea	edi,[32+esp]
	call	__ecp_nistz256_sub
	mov	eax,DWORD [488+esp]
	lea	esi,[416+esp]
	lea	ebp,[128+esp]
	lea	edi,[288+esp]
	call	__ecp_nistz256_mul_mont
	mov	eax,DWORD [488+esp]
	lea	esi,[352+esp]
	lea	ebp,[32+esp]
	lea	edi,[32+esp]
	call	__ecp_nistz256_mul_mont
	lea	esi,[32+esp]
	lea	ebp,[288+esp]
	lea	edi,[32+esp]
	call	__ecp_nistz256_sub
	mov	ebp,DWORD [480+esp]
	mov	esi,DWORD [484+esp]
	mov	edi,DWORD [512+esp]
	mov	edx,ebp
	not	ebp
	and	edx,esi
	and	ebp,esi
	not	esi
	mov	eax,edx
	and	eax,DWORD [64+esp]
	mov	ebx,ebp
	and	ebx,1
	mov	ecx,esi
	and	ecx,DWORD [160+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [64+edi],eax
	mov	eax,edx
	and	eax,DWORD [68+esp]
	mov	ecx,esi
	and	ecx,DWORD [164+esp]
	or	eax,ecx
	mov	DWORD [68+edi],eax
	mov	eax,edx
	and	eax,DWORD [72+esp]
	mov	ecx,esi
	and	ecx,DWORD [168+esp]
	or	eax,ecx
	mov	DWORD [72+edi],eax
	mov	eax,edx
	and	eax,DWORD [76+esp]
	mov	ecx,esi
	and	ecx,DWORD [172+esp]
	or	eax,ebp
	or	eax,ecx
	mov	DWORD [76+edi],eax
	mov	eax,edx
	and	eax,DWORD [80+esp]
	mov	ecx,esi
	and	ecx,DWORD [176+esp]
	or	eax,ebp
	or	eax,ecx
	mov	DWORD [80+edi],eax
	mov	eax,edx
	and	eax,DWORD [84+esp]
	mov	ecx,esi
	and	ecx,DWORD [180+esp]
	or	eax,ebp
	or	eax,ecx
	mov	DWORD [84+edi],eax
	mov	eax,edx
	and	eax,DWORD [88+esp]
	mov	ebx,ebp
	and	ebx,-2
	mov	ecx,esi
	and	ecx,DWORD [184+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [88+edi],eax
	mov	eax,edx
	and	eax,DWORD [92+esp]
	mov	ecx,esi
	and	ecx,DWORD [188+esp]
	or	eax,ecx
	mov	DWORD [92+edi],eax
	mov	eax,edx
	and	eax,DWORD [esp]
	mov	ebx,ebp
	and	ebx,DWORD [192+esp]
	mov	ecx,esi
	and	ecx,DWORD [96+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [edi],eax
	mov	eax,edx
	and	eax,DWORD [4+esp]
	mov	ebx,ebp
	and	ebx,DWORD [196+esp]
	mov	ecx,esi
	and	ecx,DWORD [100+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [4+edi],eax
	mov	eax,edx
	and	eax,DWORD [8+esp]
	mov	ebx,ebp
	and	ebx,DWORD [200+esp]
	mov	ecx,esi
	and	ecx,DWORD [104+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [8+edi],eax
	mov	eax,edx
	and	eax,DWORD [12+esp]
	mov	ebx,ebp
	and	ebx,DWORD [204+esp]
	mov	ecx,esi
	and	ecx,DWORD [108+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [12+edi],eax
	mov	eax,edx
	and	eax,DWORD [16+esp]
	mov	ebx,ebp
	and	ebx,DWORD [208+esp]
	mov	ecx,esi
	and	ecx,DWORD [112+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [16+edi],eax
	mov	eax,edx
	and	eax,DWORD [20+esp]
	mov	ebx,ebp
	and	ebx,DWORD [212+esp]
	mov	ecx,esi
	and	ecx,DWORD [116+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [20+edi],eax
	mov	eax,edx
	and	eax,DWORD [24+esp]
	mov	ebx,ebp
	and	ebx,DWORD [216+esp]
	mov	ecx,esi
	and	ecx,DWORD [120+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [24+edi],eax
	mov	eax,edx
	and	eax,DWORD [28+esp]
	mov	ebx,ebp
	and	ebx,DWORD [220+esp]
	mov	ecx,esi
	and	ecx,DWORD [124+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [28+edi],eax
	mov	eax,edx
	and	eax,DWORD [32+esp]
	mov	ebx,ebp
	and	ebx,DWORD [224+esp]
	mov	ecx,esi
	and	ecx,DWORD [128+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [32+edi],eax
	mov	eax,edx
	and	eax,DWORD [36+esp]
	mov	ebx,ebp
	and	ebx,DWORD [228+esp]
	mov	ecx,esi
	and	ecx,DWORD [132+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [36+edi],eax
	mov	eax,edx
	and	eax,DWORD [40+esp]
	mov	ebx,ebp
	and	ebx,DWORD [232+esp]
	mov	ecx,esi
	and	ecx,DWORD [136+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [40+edi],eax
	mov	eax,edx
	and	eax,DWORD [44+esp]
	mov	ebx,ebp
	and	ebx,DWORD [236+esp]
	mov	ecx,esi
	and	ecx,DWORD [140+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [44+edi],eax
	mov	eax,edx
	and	eax,DWORD [48+esp]
	mov	ebx,ebp
	and	ebx,DWORD [240+esp]
	mov	ecx,esi
	and	ecx,DWORD [144+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [48+edi],eax
	mov	eax,edx
	and	eax,DWORD [52+esp]
	mov	ebx,ebp
	and	ebx,DWORD [244+esp]
	mov	ecx,esi
	and	ecx,DWORD [148+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [52+edi],eax
	mov	eax,edx
	and	eax,DWORD [56+esp]
	mov	ebx,ebp
	and	ebx,DWORD [248+esp]
	mov	ecx,esi
	and	ecx,DWORD [152+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [56+edi],eax
	mov	eax,edx
	and	eax,DWORD [60+esp]
	mov	ebx,ebp
	and	ebx,DWORD [252+esp]
	mov	ecx,esi
	and	ecx,DWORD [156+esp]
	or	eax,ebx
	or	eax,ecx
	mov	DWORD [60+edi],eax
	add	esp,492
	pop	edi
	pop	esi
	pop	ebx
	pop	ebp
	ret
segment	.bss
common	_GFp_ia32cap_P 16
