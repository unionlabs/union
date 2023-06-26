#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
#
# ====================================================================
# Written by Andy Polyakov, @dot-asm, initially for the OpenSSL
# project.
# ====================================================================
#
# sha256_block procedure for x86_64.
#
# This module is stripped of AVX and even scalar code paths, with
# raionale that
#
# a) AVX1 is [justifiably] faster than SSSE3 code path only on *one*
#    processor, venerable Sandy Bridge;
# b) AVX2 incurs costly power transitions, which would be justifiable
#    if AVX2 code was executing most of the time, which is not the
#    case in the context;
# c) all comtemporary processors support SSSE3, so that nobody would
#    actually use scalar code path anyway;
#
# See original module at CRYPTOGAMS for further details.

$flavour = shift;
$output  = pop;
if ($flavour =~ /\./) { $output = $flavour; undef $flavour; }

$win64=0; $win64=1 if ($flavour =~ /[nm]asm|mingw64/ || $output =~ /\.asm$/);

$0 =~ m/(.*[\/\\])[^\/\\]+$/; $dir=$1;
( $xlate="${dir}x86_64-xlate.pl" and -f $xlate ) or
( $xlate="${dir}../../perlasm/x86_64-xlate.pl" and -f $xlate) or
die "can't locate x86_64-xlate.pl";

open STDOUT,"| \"$^X\" \"$xlate\" $flavour \"$output\""
    or die "can't call $xlate: $!";

$pre="blst_";
$func="${pre}sha256_block_data_order";
$TABLE="K256";
$SZ=4;
@ROT=($A,$B,$C,$D,$E,$F,$G,$H)=("%eax","%ebx","%ecx","%edx",
				"%r8d","%r9d","%r10d","%r11d");
($T1,$a0,$a1,$a2,$a3)=("%r12d","%r13d","%r14d","%r15d","%edi");
@Sigma0=( 2,13,22);
@Sigma1=( 6,11,25);
@sigma0=( 7,18, 3);
@sigma1=(17,19,10);
$rounds=64;

$ctx="%rdi";	# 1st arg, zapped by $a3
$inp="%rsi";	# 2nd arg
$Tbl="%rbp";

$_ctx="16*$SZ+0*8(%rsp)";
$_inp="16*$SZ+1*8(%rsp)";
$_end="16*$SZ+2*8(%rsp)";
$framesz="16*$SZ+3*8";

$code=<<___;
.text

.align	64
.type	$TABLE,\@object
$TABLE:
	.long	0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5
	.long	0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5
	.long	0xd807aa98,0x12835b01,0x243185be,0x550c7dc3
	.long	0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174
	.long	0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc
	.long	0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da
	.long	0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7
	.long	0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967
	.long	0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13
	.long	0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85
	.long	0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3
	.long	0xd192e819,0xd6990624,0xf40e3585,0x106aa070
	.long	0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5
	.long	0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3
	.long	0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208
	.long	0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2

	.long	0x00010203,0x04050607,0x08090a0b,0x0c0d0e0f
	.long	0x03020100,0x0b0a0908,0xffffffff,0xffffffff
	.long	0xffffffff,0xffffffff,0x03020100,0x0b0a0908
	.asciz	"SHA256 block transform for x86_64, CRYPTOGAMS by \@dot-asm"
___

######################################################################
# SIMD code paths
#
{{{
######################################################################
# Intel SHA Extensions implementation of SHA256 update function.
#
my ($ctx,$inp,$num,$Tbl)=("%rdi","%rsi","%rdx","%rcx");

my ($Wi,$ABEF,$CDGH,$TMP,$BSWAP,$ABEF_SAVE,$CDGH_SAVE)=map("%xmm$_",(0..2,7..10));
my @MSG=map("%xmm$_",(3..6));

$code.=<<___;
.globl	${pre}sha256_block_data_order_shaext
.hidden	${pre}sha256_block_data_order_shaext
.type	${pre}sha256_block_data_order_shaext,\@function,3,"unwind"
.align	64
${pre}sha256_block_data_order_shaext:
.cfi_startproc
___
$code.=<<___ if ($win64);
	sub	\$0x58,%rsp
.cfi_adjust_cfa_offset	0x58
	movaps	%xmm6,-0x58(%r11)
.cfi_offset	%xmm6,-0x60
	movaps	%xmm7,-0x48(%r11)
.cfi_offset	%xmm7,-0x50
	movaps	%xmm8,-0x38(%r11)
.cfi_offset	%xmm8,-0x40
	movaps	%xmm9,-0x28(%r11)
.cfi_offset	%xmm9,-0x30
	movaps	%xmm10,-0x18(%r11)
.cfi_offset	%xmm10,-0x20
.cfi_end_prologue
___
$code.=<<___;
	lea		K256+0x80(%rip),$Tbl
	movdqu		($ctx),$ABEF		# DCBA
	movdqu		16($ctx),$CDGH		# HGFE
	movdqa		0x100-0x80($Tbl),$TMP	# byte swap mask

	pshufd		\$0x1b,$ABEF,$Wi	# ABCD
	pshufd		\$0xb1,$ABEF,$ABEF	# CDAB
	pshufd		\$0x1b,$CDGH,$CDGH	# EFGH
	movdqa		$TMP,$BSWAP		# offload
	palignr		\$8,$CDGH,$ABEF		# ABEF
	punpcklqdq	$Wi,$CDGH		# CDGH
	jmp		.Loop_shaext

.align	16
.Loop_shaext:
	movdqu		($inp),@MSG[0]
	movdqu		0x10($inp),@MSG[1]
	movdqu		0x20($inp),@MSG[2]
	pshufb		$TMP,@MSG[0]
	movdqu		0x30($inp),@MSG[3]

	movdqa		0*16-0x80($Tbl),$Wi
	paddd		@MSG[0],$Wi
	pshufb		$TMP,@MSG[1]
	movdqa		$CDGH,$CDGH_SAVE	# offload
	sha256rnds2	$ABEF,$CDGH		# 0-3
	pshufd		\$0x0e,$Wi,$Wi
	nop
	movdqa		$ABEF,$ABEF_SAVE	# offload
	sha256rnds2	$CDGH,$ABEF

	movdqa		1*16-0x80($Tbl),$Wi
	paddd		@MSG[1],$Wi
	pshufb		$TMP,@MSG[2]
	sha256rnds2	$ABEF,$CDGH		# 4-7
	pshufd		\$0x0e,$Wi,$Wi
	lea		0x40($inp),$inp
	sha256msg1	@MSG[1],@MSG[0]
	sha256rnds2	$CDGH,$ABEF

	movdqa		2*16-0x80($Tbl),$Wi
	paddd		@MSG[2],$Wi
	pshufb		$TMP,@MSG[3]
	sha256rnds2	$ABEF,$CDGH		# 8-11
	pshufd		\$0x0e,$Wi,$Wi
	movdqa		@MSG[3],$TMP
	palignr		\$4,@MSG[2],$TMP
	nop
	paddd		$TMP,@MSG[0]
	sha256msg1	@MSG[2],@MSG[1]
	sha256rnds2	$CDGH,$ABEF

	movdqa		3*16-0x80($Tbl),$Wi
	paddd		@MSG[3],$Wi
	sha256msg2	@MSG[3],@MSG[0]
	sha256rnds2	$ABEF,$CDGH		# 12-15
	pshufd		\$0x0e,$Wi,$Wi
	movdqa		@MSG[0],$TMP
	palignr		\$4,@MSG[3],$TMP
	nop
	paddd		$TMP,@MSG[1]
	sha256msg1	@MSG[3],@MSG[2]
	sha256rnds2	$CDGH,$ABEF
___
for($i=4;$i<16-3;$i++) {
$code.=<<___;
	movdqa		$i*16-0x80($Tbl),$Wi
	paddd		@MSG[0],$Wi
	sha256msg2	@MSG[0],@MSG[1]
	sha256rnds2	$ABEF,$CDGH		# 16-19...
	pshufd		\$0x0e,$Wi,$Wi
	movdqa		@MSG[1],$TMP
	palignr		\$4,@MSG[0],$TMP
	nop
	paddd		$TMP,@MSG[2]
	sha256msg1	@MSG[0],@MSG[3]
	sha256rnds2	$CDGH,$ABEF
___
	push(@MSG,shift(@MSG));
}
$code.=<<___;
	movdqa		13*16-0x80($Tbl),$Wi
	paddd		@MSG[0],$Wi
	sha256msg2	@MSG[0],@MSG[1]
	sha256rnds2	$ABEF,$CDGH		# 52-55
	pshufd		\$0x0e,$Wi,$Wi
	movdqa		@MSG[1],$TMP
	palignr		\$4,@MSG[0],$TMP
	sha256rnds2	$CDGH,$ABEF
	paddd		$TMP,@MSG[2]

	movdqa		14*16-0x80($Tbl),$Wi
	paddd		@MSG[1],$Wi
	sha256rnds2	$ABEF,$CDGH		# 56-59
	pshufd		\$0x0e,$Wi,$Wi
	sha256msg2	@MSG[1],@MSG[2]
	movdqa		$BSWAP,$TMP
	sha256rnds2	$CDGH,$ABEF

	movdqa		15*16-0x80($Tbl),$Wi
	paddd		@MSG[2],$Wi
	nop
	sha256rnds2	$ABEF,$CDGH		# 60-63
	pshufd		\$0x0e,$Wi,$Wi
	dec		$num
	nop
	sha256rnds2	$CDGH,$ABEF

	paddd		$CDGH_SAVE,$CDGH
	paddd		$ABEF_SAVE,$ABEF
	jnz		.Loop_shaext

	pshufd		\$0xb1,$CDGH,$CDGH	# DCHG
	pshufd		\$0x1b,$ABEF,$TMP	# FEBA
	pshufd		\$0xb1,$ABEF,$ABEF	# BAFE
	punpckhqdq	$CDGH,$ABEF		# DCBA
	palignr		\$8,$TMP,$CDGH		# HGFE

	movdqu	$ABEF,($ctx)
	movdqu	$CDGH,16($ctx)
___
$code.=<<___ if ($win64);
	movaps	-0x58(%r11),%xmm6
	movaps	-0x48(%r11),%xmm7
	movaps	-0x38(%r11),%xmm8
	movaps	-0x28(%r11),%xmm9
	movaps	-0x18(%r11),%xmm10
	mov	%r11,%rsp
.cfi_def_cfa	%r11,8
.cfi_epilogue
___
$code.=<<___;
	ret
.cfi_endproc
.size	${pre}sha256_block_data_order_shaext,.-${pre}sha256_block_data_order_shaext
___
}}}
{{{

my $a4=$T1;
my ($a,$b,$c,$d,$e,$f,$g,$h);

sub AUTOLOAD()		# thunk [simplified] 32-bit style perlasm
{ my $opcode = $AUTOLOAD; $opcode =~ s/.*:://;
  my $arg = pop;
    $arg = "\$$arg" if ($arg*1 eq $arg);
    $code .= "\t$opcode\t".join(',',$arg,reverse @_)."\n";
}

sub body_00_15 () {
	(
	'($a,$b,$c,$d,$e,$f,$g,$h)=@ROT;'.

	'&ror	($a0,$Sigma1[2]-$Sigma1[1])',
	'&mov	($a,$a1)',
	'&mov	($a4,$f)',

	'&ror	($a1,$Sigma0[2]-$Sigma0[1])',
	'&xor	($a0,$e)',
	'&xor	($a4,$g)',			# f^g

	'&ror	($a0,$Sigma1[1]-$Sigma1[0])',
	'&xor	($a1,$a)',
	'&and	($a4,$e)',			# (f^g)&e

	'&xor	($a0,$e)',
	'&add	($h,$SZ*($i&15)."(%rsp)")',	# h+=X[i]+K[i]
	'&mov	($a2,$a)',

	'&xor	($a4,$g)',			# Ch(e,f,g)=((f^g)&e)^g
	'&ror	($a1,$Sigma0[1]-$Sigma0[0])',
	'&xor	($a2,$b)',			# a^b, b^c in next round

	'&add	($h,$a4)',			# h+=Ch(e,f,g)
	'&ror	($a0,$Sigma1[0])',		# Sigma1(e)
	'&and	($a3,$a2)',			# (b^c)&(a^b)

	'&xor	($a1,$a)',
	'&add	($h,$a0)',			# h+=Sigma1(e)
	'&xor	($a3,$b)',			# Maj(a,b,c)=Ch(a^b,c,b)

	'&ror	($a1,$Sigma0[0])',		# Sigma0(a)
	'&add	($d,$h)',			# d+=h
	'&add	($h,$a3)',			# h+=Maj(a,b,c)

	'&mov	($a0,$d)',
	'&add	($a1,$h);'.			# h+=Sigma0(a)
	'($a2,$a3) = ($a3,$a2); unshift(@ROT,pop(@ROT)); $i++;'
	);
}

######################################################################
# SSSE3 code path
#
{
my $Tbl = $inp;
my $_ctx="0(%rbp)";
my $_inp="8(%rbp)";
my $_end="16(%rbp)";
my $framesz=4*8+$win64*16*4+8;

my @X = map("%xmm$_",(0..3));
my ($t0,$t1,$t2,$t3, $t4,$t5) = map("%xmm$_",(4..9));

$code.=<<___;
.globl	${func}
.hidden	${func}
.type	${func},\@function,3,"unwind"
.align	64
${func}:
.cfi_startproc
	push	%rbp
.cfi_push	%rbp
	push	%rbx
.cfi_push	%rbx
	push	%r12
.cfi_push	%r12
	push	%r13
.cfi_push	%r13
	push	%r14
.cfi_push	%r14
	push	%r15
.cfi_push	%r15
	shl	\$4,%rdx		# num*16
	sub	\$$framesz,%rsp
.cfi_adjust_cfa_offset	$framesz
	lea	($inp,%rdx,$SZ),%rdx	# inp+num*16*$SZ
	mov	$ctx,0(%rsp)		# save ctx, 1st arg
	#mov	$inp,8(%rsp)		# save inp, 2nd arg
	mov	%rdx,16(%rsp)		# save end pointer, "3rd" arg
___
$code.=<<___ if ($win64);
	movaps	%xmm6,0x20(%rsp)
.cfi_offset	%xmm6,-0x78
	movaps	%xmm7,0x30(%rsp)
.cfi_offset	%xmm7,-0x68
	movaps	%xmm8,0x40(%rsp)
.cfi_offset	%xmm8,-0x58
	movaps	%xmm9,0x50(%rsp)
.cfi_offset	%xmm9,-0x48
___
$code.=<<___;
	mov	%rsp,%rbp
.cfi_def_cfa_register	%rbp
.cfi_end_prologue

	lea	-16*$SZ(%rsp),%rsp
	mov	$SZ*0($ctx),$A
	and	\$-64,%rsp		# align stack
	mov	$SZ*1($ctx),$B
	mov	$SZ*2($ctx),$C
	mov	$SZ*3($ctx),$D
	mov	$SZ*4($ctx),$E
	mov	$SZ*5($ctx),$F
	mov	$SZ*6($ctx),$G
	mov	$SZ*7($ctx),$H
___

$code.=<<___;
	#movdqa	$TABLE+`$SZ*$rounds`+32(%rip),$t4
	#movdqa	$TABLE+`$SZ*$rounds`+64(%rip),$t5
	jmp	.Lloop_ssse3
.align	16
.Lloop_ssse3:
	movdqa	$TABLE+`$SZ*$rounds`(%rip),$t3
	mov	$inp,$_inp		# offload $inp
	movdqu	0x00($inp),@X[0]
	movdqu	0x10($inp),@X[1]
	movdqu	0x20($inp),@X[2]
	pshufb	$t3,@X[0]
	movdqu	0x30($inp),@X[3]
	lea	$TABLE(%rip),$Tbl
	pshufb	$t3,@X[1]
	movdqa	0x00($Tbl),$t0
	movdqa	0x10($Tbl),$t1
	pshufb	$t3,@X[2]
	paddd	@X[0],$t0
	movdqa	0x20($Tbl),$t2
	pshufb	$t3,@X[3]
	movdqa	0x30($Tbl),$t3
	paddd	@X[1],$t1
	paddd	@X[2],$t2
	paddd	@X[3],$t3
	movdqa	$t0,0x00(%rsp)
	mov	$A,$a1
	movdqa	$t1,0x10(%rsp)
	mov	$B,$a3
	movdqa	$t2,0x20(%rsp)
	xor	$C,$a3			# magic
	movdqa	$t3,0x30(%rsp)
	mov	$E,$a0
	jmp	.Lssse3_00_47

.align	16
.Lssse3_00_47:
	sub	\$`-16*$SZ`,$Tbl	# size optimization
___
sub Xupdate_256_SSSE3 () {
	(
	'&movdqa	($t0,@X[1]);',
	'&movdqa	($t3,@X[3])',
	'&palignr	($t0,@X[0],$SZ)',	# X[1..4]
	 '&palignr	($t3,@X[2],$SZ);',	# X[9..12]
	'&movdqa	($t1,$t0)',
	'&movdqa	($t2,$t0);',
	'&psrld		($t0,$sigma0[2])',
	 '&paddd	(@X[0],$t3);',		# X[0..3] += X[9..12]
	'&psrld		($t2,$sigma0[0])',
	 '&pshufd	($t3,@X[3],0b11111010)',# X[14..15]
	'&pslld		($t1,8*$SZ-$sigma0[1]);'.
	'&pxor		($t0,$t2)',
	'&psrld		($t2,$sigma0[1]-$sigma0[0]);'.
	'&pxor		($t0,$t1)',
	'&pslld		($t1,$sigma0[1]-$sigma0[0]);'.
	'&pxor		($t0,$t2);',
	 '&movdqa	($t2,$t3)',
	'&pxor		($t0,$t1);',		# sigma0(X[1..4])
	 '&psrld	($t3,$sigma1[2])',
	'&paddd		(@X[0],$t0);',		# X[0..3] += sigma0(X[1..4])
	 '&psrlq	($t2,$sigma1[0])',
	 '&pxor		($t3,$t2);',
	 '&psrlq	($t2,$sigma1[1]-$sigma1[0])',
	 '&pxor		($t3,$t2)',
	 '&pshufb	($t3,$t4)',		# sigma1(X[14..15])
	'&paddd		(@X[0],$t3)',		# X[0..1] += sigma1(X[14..15])
	 '&pshufd	($t3,@X[0],0b01010000)',# X[16..17]
	 '&movdqa	($t2,$t3);',
	 '&psrld	($t3,$sigma1[2])',
	 '&psrlq	($t2,$sigma1[0])',
	 '&pxor		($t3,$t2);',
	 '&psrlq	($t2,$sigma1[1]-$sigma1[0])',
	 '&pxor		($t3,$t2);',
	'&movdqa	($t2,16*$j."($Tbl)")',
	 '&pshufb	($t3,$t5)',
	'&paddd		(@X[0],$t3)'		# X[2..3] += sigma1(X[16..17])
	);
}

sub SSSE3_256_00_47 () {
my $j = shift;
my $body = shift;
my @X = @_;
my @insns = (&$body,&$body,&$body,&$body);	# 104 instructions

    if (0) {
	foreach (Xupdate_256_SSSE3()) {		# 36 instructions
	    eval;
	    eval(shift(@insns));
	    eval(shift(@insns));
	    eval(shift(@insns));
	}
    } else {			# squeeze extra 4% on Westmere and 19% on Atom
	  eval(shift(@insns));	#@
	&movdqa		($t0,@X[1]);
	  eval(shift(@insns));
	  eval(shift(@insns));
	&movdqa		($t3,@X[3]);
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	&palignr	($t0,@X[0],$SZ);	# X[1..4]
	  eval(shift(@insns));
	  eval(shift(@insns));
	 &palignr	($t3,@X[2],$SZ);	# X[9..12]
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	&movdqa		($t1,$t0);
	  eval(shift(@insns));
	  eval(shift(@insns));
	&movdqa		($t2,$t0);
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	&psrld		($t0,$sigma0[2]);
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));
	 &paddd		(@X[0],$t3);		# X[0..3] += X[9..12]
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	&psrld		($t2,$sigma0[0]);
	  eval(shift(@insns));
	  eval(shift(@insns));
	 &pshufd	($t3,@X[3],0b11111010);	# X[4..15]
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	&pslld		($t1,8*$SZ-$sigma0[1]);
	  eval(shift(@insns));
	  eval(shift(@insns));
	&pxor		($t0,$t2);
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	&psrld		($t2,$sigma0[1]-$sigma0[0]);
	  eval(shift(@insns));
	&pxor		($t0,$t1);
	  eval(shift(@insns));
	  eval(shift(@insns));
	&pslld		($t1,$sigma0[1]-$sigma0[0]);
	  eval(shift(@insns));
	  eval(shift(@insns));
	&pxor		($t0,$t2);
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	 &movdqa	($t2,$t3);
	  eval(shift(@insns));
	  eval(shift(@insns));
	&pxor		($t0,$t1);		# sigma0(X[1..4])
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	  eval(shift(@insns));
	 &psrld		($t3,$sigma1[2]);
	  eval(shift(@insns));
	  eval(shift(@insns));
	&paddd		(@X[0],$t0);		# X[0..3] += sigma0(X[1..4])
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	 &psrlq		($t2,$sigma1[0]);
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));
	 &pxor		($t3,$t2);
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	 &psrlq		($t2,$sigma1[1]-$sigma1[0]);
	  eval(shift(@insns));
	  eval(shift(@insns));
	 &pxor		($t3,$t2);
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	  eval(shift(@insns));
	 #&pshufb	($t3,$t4);		# sigma1(X[14..15])
	 &pshufd	($t3,$t3,0b10000000);
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));
	 &psrldq	($t3,8);
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	&paddd		(@X[0],$t3);		# X[0..1] += sigma1(X[14..15])
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));
	 &pshufd	($t3,@X[0],0b01010000);	# X[16..17]
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	 &movdqa	($t2,$t3);
	  eval(shift(@insns));
	  eval(shift(@insns));
	 &psrld		($t3,$sigma1[2]);
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	 &psrlq		($t2,$sigma1[0]);
	  eval(shift(@insns));
	  eval(shift(@insns));
	 &pxor		($t3,$t2);
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	 &psrlq		($t2,$sigma1[1]-$sigma1[0]);
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));
	 &pxor		($t3,$t2);
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));	#@
	 #&pshufb	($t3,$t5);
	 &pshufd	($t3,$t3,0b00001000);
	  eval(shift(@insns));
	  eval(shift(@insns));
	&movdqa		($t2,16*$j."($Tbl)");
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	 &pslldq	($t3,8);
	  eval(shift(@insns));
	  eval(shift(@insns));
	  eval(shift(@insns));
	&paddd		(@X[0],$t3);		# X[2..3] += sigma1(X[16..17])
	  eval(shift(@insns));	#@
	  eval(shift(@insns));
	  eval(shift(@insns));
    }
	&paddd		($t2,@X[0]);
	  foreach (@insns) { eval; }		# remaining instructions
	&movdqa		(16*$j."(%rsp)",$t2);
}

    for ($i=0,$j=0; $j<4; $j++) {
	&SSSE3_256_00_47($j,\&body_00_15,@X);
	push(@X,shift(@X));			# rotate(@X)
    }
	&cmpb	($SZ-1+16*$SZ."($Tbl)",0);
	&jne	(".Lssse3_00_47");

    for ($i=0; $i<16; ) {
	foreach(body_00_15()) { eval; }
    }
$code.=<<___;
	mov	$_ctx,$ctx
	mov	$a1,$A
	mov	$_inp,$inp

	add	$SZ*0($ctx),$A
	add	$SZ*1($ctx),$B
	add	$SZ*2($ctx),$C
	add	$SZ*3($ctx),$D
	add	$SZ*4($ctx),$E
	add	$SZ*5($ctx),$F
	add	$SZ*6($ctx),$G
	add	$SZ*7($ctx),$H

	lea	16*$SZ($inp),$inp
	cmp	$_end,$inp

	mov	$A,$SZ*0($ctx)
	mov	$B,$SZ*1($ctx)
	mov	$C,$SZ*2($ctx)
	mov	$D,$SZ*3($ctx)
	mov	$E,$SZ*4($ctx)
	mov	$F,$SZ*5($ctx)
	mov	$G,$SZ*6($ctx)
	mov	$H,$SZ*7($ctx)
	jb	.Lloop_ssse3

	xorps	%xmm0, %xmm0
	lea	$framesz+6*8(%rbp),%r11
.cfi_def_cfa	%r11,8
	movaps	%xmm0, 0x00(%rsp)	# scrub the stack
	movaps	%xmm0, 0x10(%rsp)
	movaps	%xmm0, 0x20(%rsp)
	movaps	%xmm0, 0x30(%rsp)
___
$code.=<<___ if ($win64);
	movaps	0x20(%rbp),%xmm6
	movaps	0x30(%rbp),%xmm7
	movaps	0x40(%rbp),%xmm8
	movaps	0x50(%rbp),%xmm9
___
$code.=<<___;
	mov	$framesz(%rbp),%r15
.cfi_restore	%r15
	mov	-40(%r11),%r14
.cfi_restore	%r14
	mov	-32(%r11),%r13
.cfi_restore	%r13
	mov	-24(%r11),%r12
.cfi_restore	%r12
	mov	-16(%r11),%rbx
.cfi_restore	%rbx
	mov	-8(%r11),%rbp
.cfi_restore	%rbp
.cfi_epilogue
	lea	(%r11),%rsp
	ret
.cfi_endproc
.size	${func},.-${func}
___
}
}}}
{
my ($out,$inp,$len) = $win64 ? ("%rcx","%rdx","%r8") :  # Win64 order
                               ("%rdi","%rsi","%rdx");  # Unix order
$code.=<<___;
.globl	${pre}sha256_emit
.hidden	${pre}sha256_emit
.type	${pre}sha256_emit,\@abi-omnipotent
.align	16
${pre}sha256_emit:
	mov	0($inp), %r8
	mov	8($inp), %r9
	mov	16($inp), %r10
	bswap	%r8
	mov	24($inp), %r11
	bswap	%r9
	mov	%r8d, 4($out)
	bswap	%r10
	mov	%r9d, 12($out)
	bswap	%r11
	mov	%r10d, 20($out)
	shr	\$32, %r8
	mov	%r11d, 28($out)
	shr	\$32, %r9
	mov	%r8d, 0($out)
	shr	\$32, %r10
	mov	%r9d, 8($out)
	shr	\$32, %r11
	mov	%r10d, 16($out)
	mov	%r11d, 24($out)
	ret
.size	${pre}sha256_emit,.-${pre}sha256_emit

.globl	${pre}sha256_bcopy
.hidden	${pre}sha256_bcopy
.type	${pre}sha256_bcopy,\@abi-omnipotent
.align	16
${pre}sha256_bcopy:
	sub	$inp, $out
.Loop_bcopy:
	movzb	($inp), %eax
	lea	1($inp), $inp
	mov	%al, -1($out,$inp)
	dec	$len
	jnz	.Loop_bcopy
	ret
.size	${pre}sha256_bcopy,.-${pre}sha256_bcopy

.globl	${pre}sha256_hcopy
.hidden	${pre}sha256_hcopy
.type	${pre}sha256_hcopy,\@abi-omnipotent
.align	16
${pre}sha256_hcopy:
	mov	0($inp), %r8
	mov	8($inp), %r9
	mov	16($inp), %r10
	mov	24($inp), %r11
	mov	%r8, 0($out)
	mov	%r9, 8($out)
	mov	%r10, 16($out)
	mov	%r11, 24($out)
	ret
.size	${pre}sha256_hcopy,.-${pre}sha256_hcopy
___
}

sub sha256op38 {
    my $instr = shift;
    my %opcodelet = (
		"sha256rnds2" => 0xcb,
  		"sha256msg1"  => 0xcc,
		"sha256msg2"  => 0xcd	);

    if (defined($opcodelet{$instr}) && @_[0] =~ /%xmm([0-7]),\s*%xmm([0-7])/) {
      my @opcode=(0x0f,0x38);
	push @opcode,$opcodelet{$instr};
	push @opcode,0xc0|($1&7)|(($2&7)<<3);		# ModR/M
	return ".byte\t".join(',',@opcode);
    } else {
	return $instr."\t".@_[0];
    }
}

foreach (split("\n",$code)) {
	s/\`([^\`]*)\`/eval $1/geo;

	s/\b(sha256[^\s]*)\s+(.*)/sha256op38($1,$2)/geo;

	print $_,"\n";
}
close STDOUT;
