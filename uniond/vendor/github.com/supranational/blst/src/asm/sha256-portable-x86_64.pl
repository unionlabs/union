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
# Scalar-only version with minor twist minimizing 'lea' instructions.

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

sub ROUND_00_15()
{ my ($i,$a,$b,$c,$d,$e,$f,$g,$h) = @_;
  my $STRIDE=$SZ;
  #   $STRIDE += 16 if ($i%(16/$SZ)==(16/$SZ-1));

$code.=<<___;
	ror	\$`$Sigma1[2]-$Sigma1[1]`,$a0
	mov	$f,$a2

	xor	$e,$a0
	ror	\$`$Sigma0[2]-$Sigma0[1]`,$a1
	xor	$g,$a2			# f^g

	mov	$T1,`$SZ*($i&0xf)`(%rsp)
	xor	$a,$a1
	and	$e,$a2			# (f^g)&e

	ror	\$`$Sigma1[1]-$Sigma1[0]`,$a0
	add	$h,$T1			# T1+=h
	xor	$g,$a2			# Ch(e,f,g)=((f^g)&e)^g

	ror	\$`$Sigma0[1]-$Sigma0[0]`,$a1
	xor	$e,$a0
	add	$a2,$T1			# T1+=Ch(e,f,g)

	mov	$a,$a2
	add	`$SZ*$i`($Tbl),$T1	# T1+=K[round]
	xor	$a,$a1

	xor	$b,$a2			# a^b, b^c in next round
	ror	\$$Sigma1[0],$a0	# Sigma1(e)
	mov	$b,$h

	and	$a2,$a3
	ror	\$$Sigma0[0],$a1	# Sigma0(a)
	add	$a0,$T1			# T1+=Sigma1(e)

	xor	$a3,$h			# h=Maj(a,b,c)=Ch(a^b,c,b)
	add	$T1,$d			# d+=T1
	add	$T1,$h			# h+=T1
___
$code.=<<___ if ($i==31);
	lea	`16*$SZ`($Tbl),$Tbl	# round+=16
___
$code.=<<___ if ($i<15);
	add	$a1,$h			# h+=Sigma0(a)
___
	($a2,$a3) = ($a3,$a2);
}

sub ROUND_16_XX()
{ my ($i,$a,$b,$c,$d,$e,$f,$g,$h) = @_;

$code.=<<___;
	mov	`$SZ*(($i+1)&0xf)`(%rsp),$a0
	mov	`$SZ*(($i+14)&0xf)`(%rsp),$a2

	mov	$a0,$T1
	ror	\$`$sigma0[1]-$sigma0[0]`,$a0
	add	$a1,$a			# modulo-scheduled h+=Sigma0(a)
	mov	$a2,$a1
	ror	\$`$sigma1[1]-$sigma1[0]`,$a2

	xor	$T1,$a0
	shr	\$$sigma0[2],$T1
	ror	\$$sigma0[0],$a0
	xor	$a1,$a2
	shr	\$$sigma1[2],$a1

	ror	\$$sigma1[0],$a2
	xor	$a0,$T1			# sigma0(X[(i+1)&0xf])
	xor	$a1,$a2			# sigma1(X[(i+14)&0xf])
	add	`$SZ*(($i+9)&0xf)`(%rsp),$T1

	add	`$SZ*($i&0xf)`(%rsp),$T1
	mov	$e,$a0
	add	$a2,$T1
	mov	$a,$a1
___
	&ROUND_00_15(@_);
}

$code=<<___;
.text

.globl	$func
.type	$func,\@function,3,"unwind"
.align	16
$func:
.cfi_startproc
	push	%rbx
.cfi_push	%rbx
	push	%rbp
.cfi_push	%rbp
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
	mov	$ctx,$_ctx		# save ctx, 1st arg
	mov	$inp,$_inp		# save inp, 2nd arh
	mov	%rdx,$_end		# save end pointer, "3rd" arg
.cfi_end_prologue

	mov	$SZ*0($ctx),$A
	mov	$SZ*1($ctx),$B
	mov	$SZ*2($ctx),$C
	mov	$SZ*3($ctx),$D
	mov	$SZ*4($ctx),$E
	mov	$SZ*5($ctx),$F
	mov	$SZ*6($ctx),$G
	mov	$SZ*7($ctx),$H
	jmp	.Lloop

.align	16
.Lloop:
	mov	$B,$a3
	lea	$TABLE(%rip),$Tbl
	xor	$C,$a3			# magic
___
	for($i=0;$i<16;$i++) {
		$code.="	mov	$SZ*$i($inp),$T1\n";
		$code.="	mov	@ROT[4],$a0\n";
		$code.="	mov	@ROT[0],$a1\n";
		$code.="	bswap	$T1\n";
		&ROUND_00_15($i,@ROT);
		unshift(@ROT,pop(@ROT));
	}
$code.=<<___;
	jmp	.Lrounds_16_xx
.align	16
.Lrounds_16_xx:
___
	for(;$i<32;$i++) {
		&ROUND_16_XX($i,@ROT);
		unshift(@ROT,pop(@ROT));
	}

$code.=<<___;
	cmpb	\$0x19,`$SZ-1`($Tbl)
	jnz	.Lrounds_16_xx

	mov	$_ctx,$ctx
	add	$a1,$A			# modulo-scheduled h+=Sigma0(a)
	lea	16*$SZ($inp),$inp

	add	$SZ*0($ctx),$A
	add	$SZ*1($ctx),$B
	add	$SZ*2($ctx),$C
	add	$SZ*3($ctx),$D
	add	$SZ*4($ctx),$E
	add	$SZ*5($ctx),$F
	add	$SZ*6($ctx),$G
	add	$SZ*7($ctx),$H

	cmp	$_end,$inp

	mov	$A,$SZ*0($ctx)
	mov	$B,$SZ*1($ctx)
	mov	$C,$SZ*2($ctx)
	mov	$D,$SZ*3($ctx)
	mov	$E,$SZ*4($ctx)
	mov	$F,$SZ*5($ctx)
	mov	$G,$SZ*6($ctx)
	mov	$H,$SZ*7($ctx)
	jb	.Lloop

	lea	$framesz+6*8(%rsp),%r11
.cfi_def_cfa	%r11,8
	mov	$framesz(%rsp),%r15
.cfi_restore	%r15
	mov	-40(%r11),%r14
.cfi_restore	%r14
	mov	-32(%r11),%r13
.cfi_restore	%r13
	mov	-24(%r11),%r12
.cfi_restore	%r12
	mov	-16(%r11),%rbp
.cfi_restore	%rbp
	mov	-8(%r11),%rbx
.cfi_restore	%rbx
.cfi_epilogue
	lea	(%r11),%rsp
	ret
.cfi_endproc
.size	$func,.-$func

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

	.asciz	"SHA256 block transform for x86_64, CRYPTOGAMS by \@dot-asm"
___
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

foreach (split("\n",$code)) {
	s/\`([^\`]*)\`/eval $1/geo;
	print $_,"\n";
}
close STDOUT;
