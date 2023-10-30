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
global	_GFp_gcm_init_clmul
align	16
_GFp_gcm_init_clmul:
L$_GFp_gcm_init_clmul_begin:
	mov	edx,DWORD [4+esp]
	mov	eax,DWORD [8+esp]
	call	L$000pic
L$000pic:
	pop	ecx
	lea	ecx,[(L$bswap-L$000pic)+ecx]
	movdqu	xmm2,[eax]
	pshufd	xmm2,xmm2,78
	pshufd	xmm4,xmm2,255
	movdqa	xmm3,xmm2
	psllq	xmm2,1
	pxor	xmm5,xmm5
	psrlq	xmm3,63
	pcmpgtd	xmm5,xmm4
	pslldq	xmm3,8
	por	xmm2,xmm3
	pand	xmm5,[16+ecx]
	pxor	xmm2,xmm5
	movdqa	xmm0,xmm2
	movdqa	xmm1,xmm0
	pshufd	xmm3,xmm0,78
	pshufd	xmm4,xmm2,78
	pxor	xmm3,xmm0
	pxor	xmm4,xmm2
db	102,15,58,68,194,0
db	102,15,58,68,202,17
db	102,15,58,68,220,0
	xorps	xmm3,xmm0
	xorps	xmm3,xmm1
	movdqa	xmm4,xmm3
	psrldq	xmm3,8
	pslldq	xmm4,8
	pxor	xmm1,xmm3
	pxor	xmm0,xmm4
	movdqa	xmm4,xmm0
	movdqa	xmm3,xmm0
	psllq	xmm0,5
	pxor	xmm3,xmm0
	psllq	xmm0,1
	pxor	xmm0,xmm3
	psllq	xmm0,57
	movdqa	xmm3,xmm0
	pslldq	xmm0,8
	psrldq	xmm3,8
	pxor	xmm0,xmm4
	pxor	xmm1,xmm3
	movdqa	xmm4,xmm0
	psrlq	xmm0,1
	pxor	xmm1,xmm4
	pxor	xmm4,xmm0
	psrlq	xmm0,5
	pxor	xmm0,xmm4
	psrlq	xmm0,1
	pxor	xmm0,xmm1
	pshufd	xmm3,xmm2,78
	pshufd	xmm4,xmm0,78
	pxor	xmm3,xmm2
	movdqu	[edx],xmm2
	pxor	xmm4,xmm0
	movdqu	[16+edx],xmm0
db	102,15,58,15,227,8
	movdqu	[32+edx],xmm4
	ret
global	_GFp_gcm_gmult_clmul
align	16
_GFp_gcm_gmult_clmul:
L$_GFp_gcm_gmult_clmul_begin:
	mov	eax,DWORD [4+esp]
	mov	edx,DWORD [8+esp]
	call	L$001pic
L$001pic:
	pop	ecx
	lea	ecx,[(L$bswap-L$001pic)+ecx]
	movdqu	xmm0,[eax]
	movdqa	xmm5,[ecx]
	movups	xmm2,[edx]
db	102,15,56,0,197
	movups	xmm4,[32+edx]
	movdqa	xmm1,xmm0
	pshufd	xmm3,xmm0,78
	pxor	xmm3,xmm0
db	102,15,58,68,194,0
db	102,15,58,68,202,17
db	102,15,58,68,220,0
	xorps	xmm3,xmm0
	xorps	xmm3,xmm1
	movdqa	xmm4,xmm3
	psrldq	xmm3,8
	pslldq	xmm4,8
	pxor	xmm1,xmm3
	pxor	xmm0,xmm4
	movdqa	xmm4,xmm0
	movdqa	xmm3,xmm0
	psllq	xmm0,5
	pxor	xmm3,xmm0
	psllq	xmm0,1
	pxor	xmm0,xmm3
	psllq	xmm0,57
	movdqa	xmm3,xmm0
	pslldq	xmm0,8
	psrldq	xmm3,8
	pxor	xmm0,xmm4
	pxor	xmm1,xmm3
	movdqa	xmm4,xmm0
	psrlq	xmm0,1
	pxor	xmm1,xmm4
	pxor	xmm4,xmm0
	psrlq	xmm0,5
	pxor	xmm0,xmm4
	psrlq	xmm0,1
	pxor	xmm0,xmm1
db	102,15,56,0,197
	movdqu	[eax],xmm0
	ret
global	_GFp_gcm_ghash_clmul
align	16
_GFp_gcm_ghash_clmul:
L$_GFp_gcm_ghash_clmul_begin:
	push	ebp
	push	ebx
	push	esi
	push	edi
	mov	eax,DWORD [20+esp]
	mov	edx,DWORD [24+esp]
	mov	esi,DWORD [28+esp]
	mov	ebx,DWORD [32+esp]
	call	L$002pic
L$002pic:
	pop	ecx
	lea	ecx,[(L$bswap-L$002pic)+ecx]
	movdqu	xmm0,[eax]
	movdqa	xmm5,[ecx]
	movdqu	xmm2,[edx]
db	102,15,56,0,197
	sub	ebx,16
	jz	NEAR L$003odd_tail
	movdqu	xmm3,[esi]
	movdqu	xmm6,[16+esi]
db	102,15,56,0,221
db	102,15,56,0,245
	movdqu	xmm5,[32+edx]
	pxor	xmm0,xmm3
	pshufd	xmm3,xmm6,78
	movdqa	xmm7,xmm6
	pxor	xmm3,xmm6
	lea	esi,[32+esi]
db	102,15,58,68,242,0
db	102,15,58,68,250,17
db	102,15,58,68,221,0
	movups	xmm2,[16+edx]
	nop
	sub	ebx,32
	jbe	NEAR L$004even_tail
	jmp	NEAR L$005mod_loop
align	32
L$005mod_loop:
	pshufd	xmm4,xmm0,78
	movdqa	xmm1,xmm0
	pxor	xmm4,xmm0
	nop
db	102,15,58,68,194,0
db	102,15,58,68,202,17
db	102,15,58,68,229,16
	movups	xmm2,[edx]
	xorps	xmm0,xmm6
	movdqa	xmm5,[ecx]
	xorps	xmm1,xmm7
	movdqu	xmm7,[esi]
	pxor	xmm3,xmm0
	movdqu	xmm6,[16+esi]
	pxor	xmm3,xmm1
db	102,15,56,0,253
	pxor	xmm4,xmm3
	movdqa	xmm3,xmm4
	psrldq	xmm4,8
	pslldq	xmm3,8
	pxor	xmm1,xmm4
	pxor	xmm0,xmm3
db	102,15,56,0,245
	pxor	xmm1,xmm7
	movdqa	xmm7,xmm6
	movdqa	xmm4,xmm0
	movdqa	xmm3,xmm0
	psllq	xmm0,5
	pxor	xmm3,xmm0
	psllq	xmm0,1
	pxor	xmm0,xmm3
db	102,15,58,68,242,0
	movups	xmm5,[32+edx]
	psllq	xmm0,57
	movdqa	xmm3,xmm0
	pslldq	xmm0,8
	psrldq	xmm3,8
	pxor	xmm0,xmm4
	pxor	xmm1,xmm3
	pshufd	xmm3,xmm7,78
	movdqa	xmm4,xmm0
	psrlq	xmm0,1
	pxor	xmm3,xmm7
	pxor	xmm1,xmm4
db	102,15,58,68,250,17
	movups	xmm2,[16+edx]
	pxor	xmm4,xmm0
	psrlq	xmm0,5
	pxor	xmm0,xmm4
	psrlq	xmm0,1
	pxor	xmm0,xmm1
db	102,15,58,68,221,0
	lea	esi,[32+esi]
	sub	ebx,32
	ja	NEAR L$005mod_loop
L$004even_tail:
	pshufd	xmm4,xmm0,78
	movdqa	xmm1,xmm0
	pxor	xmm4,xmm0
db	102,15,58,68,194,0
db	102,15,58,68,202,17
db	102,15,58,68,229,16
	movdqa	xmm5,[ecx]
	xorps	xmm0,xmm6
	xorps	xmm1,xmm7
	pxor	xmm3,xmm0
	pxor	xmm3,xmm1
	pxor	xmm4,xmm3
	movdqa	xmm3,xmm4
	psrldq	xmm4,8
	pslldq	xmm3,8
	pxor	xmm1,xmm4
	pxor	xmm0,xmm3
	movdqa	xmm4,xmm0
	movdqa	xmm3,xmm0
	psllq	xmm0,5
	pxor	xmm3,xmm0
	psllq	xmm0,1
	pxor	xmm0,xmm3
	psllq	xmm0,57
	movdqa	xmm3,xmm0
	pslldq	xmm0,8
	psrldq	xmm3,8
	pxor	xmm0,xmm4
	pxor	xmm1,xmm3
	movdqa	xmm4,xmm0
	psrlq	xmm0,1
	pxor	xmm1,xmm4
	pxor	xmm4,xmm0
	psrlq	xmm0,5
	pxor	xmm0,xmm4
	psrlq	xmm0,1
	pxor	xmm0,xmm1
	test	ebx,ebx
	jnz	NEAR L$006done
	movups	xmm2,[edx]
L$003odd_tail:
	movdqu	xmm3,[esi]
db	102,15,56,0,221
	pxor	xmm0,xmm3
	movdqa	xmm1,xmm0
	pshufd	xmm3,xmm0,78
	pshufd	xmm4,xmm2,78
	pxor	xmm3,xmm0
	pxor	xmm4,xmm2
db	102,15,58,68,194,0
db	102,15,58,68,202,17
db	102,15,58,68,220,0
	xorps	xmm3,xmm0
	xorps	xmm3,xmm1
	movdqa	xmm4,xmm3
	psrldq	xmm3,8
	pslldq	xmm4,8
	pxor	xmm1,xmm3
	pxor	xmm0,xmm4
	movdqa	xmm4,xmm0
	movdqa	xmm3,xmm0
	psllq	xmm0,5
	pxor	xmm3,xmm0
	psllq	xmm0,1
	pxor	xmm0,xmm3
	psllq	xmm0,57
	movdqa	xmm3,xmm0
	pslldq	xmm0,8
	psrldq	xmm3,8
	pxor	xmm0,xmm4
	pxor	xmm1,xmm3
	movdqa	xmm4,xmm0
	psrlq	xmm0,1
	pxor	xmm1,xmm4
	pxor	xmm4,xmm0
	psrlq	xmm0,5
	pxor	xmm0,xmm4
	psrlq	xmm0,1
	pxor	xmm0,xmm1
L$006done:
db	102,15,56,0,197
	movdqu	[eax],xmm0
	pop	edi
	pop	esi
	pop	ebx
	pop	ebp
	ret
align	64
L$bswap:
db	15,14,13,12,11,10,9,8,7,6,5,4,3,2,1,0
db	1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,194
align	64
L$007rem_8bit:
dw	0,450,900,582,1800,1738,1164,1358
dw	3600,4050,3476,3158,2328,2266,2716,2910
dw	7200,7650,8100,7782,6952,6890,6316,6510
dw	4656,5106,4532,4214,5432,5370,5820,6014
dw	14400,14722,15300,14854,16200,16010,15564,15630
dw	13904,14226,13780,13334,12632,12442,13020,13086
dw	9312,9634,10212,9766,9064,8874,8428,8494
dw	10864,11186,10740,10294,11640,11450,12028,12094
dw	28800,28994,29444,29382,30600,30282,29708,30158
dw	32400,32594,32020,31958,31128,30810,31260,31710
dw	27808,28002,28452,28390,27560,27242,26668,27118
dw	25264,25458,24884,24822,26040,25722,26172,26622
dw	18624,18690,19268,19078,20424,19978,19532,19854
dw	18128,18194,17748,17558,16856,16410,16988,17310
dw	21728,21794,22372,22182,21480,21034,20588,20910
dw	23280,23346,22900,22710,24056,23610,24188,24510
dw	57600,57538,57988,58182,58888,59338,58764,58446
dw	61200,61138,60564,60758,59416,59866,60316,59998
dw	64800,64738,65188,65382,64040,64490,63916,63598
dw	62256,62194,61620,61814,62520,62970,63420,63102
dw	55616,55426,56004,56070,56904,57226,56780,56334
dw	55120,54930,54484,54550,53336,53658,54236,53790
dw	50528,50338,50916,50982,49768,50090,49644,49198
dw	52080,51890,51444,51510,52344,52666,53244,52798
dw	37248,36930,37380,37830,38536,38730,38156,38094
dw	40848,40530,39956,40406,39064,39258,39708,39646
dw	36256,35938,36388,36838,35496,35690,35116,35054
dw	33712,33394,32820,33270,33976,34170,34620,34558
dw	43456,43010,43588,43910,44744,44810,44364,44174
dw	42960,42514,42068,42390,41176,41242,41820,41630
dw	46560,46114,46692,47014,45800,45866,45420,45230
dw	48112,47666,47220,47542,48376,48442,49020,48830
db	71,72,65,83,72,32,102,111,114,32,120,56,54,44,32,67
db	82,89,80,84,79,71,65,77,83,32,98,121,32,60,97,112
db	112,114,111,64,111,112,101,110,115,115,108,46,111,114,103,62
db	0
