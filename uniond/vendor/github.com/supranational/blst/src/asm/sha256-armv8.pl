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
# sha256_block procedure for ARMv8.
#
# This module is stripped of scalar code paths, with raionale that all
# known processors are NEON-capable.
#
# See original module at CRYPTOGAMS for further details.

$flavour = shift;
$output  = shift;

if ($flavour && $flavour ne "void") {
    $0 =~ m/(.*[\/\\])[^\/\\]+$/; $dir=$1;
    ( $xlate="${dir}arm-xlate.pl" and -f $xlate ) or
    ( $xlate="${dir}../../perlasm/arm-xlate.pl" and -f $xlate) or
    die "can't locate arm-xlate.pl";

    open STDOUT,"| \"$^X\" $xlate $flavour $output";
} else {
    open STDOUT,">$output";
}

$BITS=256;
$SZ=4;
@Sigma0=( 2,13,22);
@Sigma1=( 6,11,25);
@sigma0=( 7,18, 3);
@sigma1=(17,19,10);
$rounds=64;
$reg_t="w";
$pre="blst_";

($ctx,$inp,$num,$Ktbl)=map("x$_",(0..2,30));

$code.=<<___;
.text

.align	6
.type	.LK$BITS,%object
.LK$BITS:
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
	.long	0	//terminator
.size	.LK$BITS,.-.LK$BITS
.asciz	"SHA$BITS block transform for ARMv8, CRYPTOGAMS by \@dot-asm"
.align	2
___

if ($SZ==4) {
my $Ktbl="x3";

my ($ABCD,$EFGH,$abcd)=map("v$_.16b",(0..2));
my @MSG=map("v$_.16b",(4..7));
my ($W0,$W1)=("v16.4s","v17.4s");
my ($ABCD_SAVE,$EFGH_SAVE)=("v18.16b","v19.16b");

$code.=<<___;
.globl	${pre}sha256_block_armv8
.type	${pre}sha256_block_armv8,%function
.align	6
${pre}sha256_block_armv8:
.Lv8_entry:
	stp		x29,x30,[sp,#-16]!
	add		x29,sp,#0

	ld1.32		{$ABCD,$EFGH},[$ctx]
	adr		$Ktbl,.LK256

.Loop_hw:
	ld1		{@MSG[0]-@MSG[3]},[$inp],#64
	sub		$num,$num,#1
	ld1.32		{$W0},[$Ktbl],#16
	rev32		@MSG[0],@MSG[0]
	rev32		@MSG[1],@MSG[1]
	rev32		@MSG[2],@MSG[2]
	rev32		@MSG[3],@MSG[3]
	orr		$ABCD_SAVE,$ABCD,$ABCD		// offload
	orr		$EFGH_SAVE,$EFGH,$EFGH
___
for($i=0;$i<12;$i++) {
$code.=<<___;
	ld1.32		{$W1},[$Ktbl],#16
	add.i32		$W0,$W0,@MSG[0]
	sha256su0	@MSG[0],@MSG[1]
	orr		$abcd,$ABCD,$ABCD
	sha256h		$ABCD,$EFGH,$W0
	sha256h2	$EFGH,$abcd,$W0
	sha256su1	@MSG[0],@MSG[2],@MSG[3]
___
	($W0,$W1)=($W1,$W0);	push(@MSG,shift(@MSG));
}
$code.=<<___;
	ld1.32		{$W1},[$Ktbl],#16
	add.i32		$W0,$W0,@MSG[0]
	orr		$abcd,$ABCD,$ABCD
	sha256h		$ABCD,$EFGH,$W0
	sha256h2	$EFGH,$abcd,$W0

	ld1.32		{$W0},[$Ktbl],#16
	add.i32		$W1,$W1,@MSG[1]
	orr		$abcd,$ABCD,$ABCD
	sha256h		$ABCD,$EFGH,$W1
	sha256h2	$EFGH,$abcd,$W1

	ld1.32		{$W1},[$Ktbl]
	add.i32		$W0,$W0,@MSG[2]
	sub		$Ktbl,$Ktbl,#$rounds*$SZ-16	// rewind
	orr		$abcd,$ABCD,$ABCD
	sha256h		$ABCD,$EFGH,$W0
	sha256h2	$EFGH,$abcd,$W0

	add.i32		$W1,$W1,@MSG[3]
	orr		$abcd,$ABCD,$ABCD
	sha256h		$ABCD,$EFGH,$W1
	sha256h2	$EFGH,$abcd,$W1

	add.i32		$ABCD,$ABCD,$ABCD_SAVE
	add.i32		$EFGH,$EFGH,$EFGH_SAVE

	cbnz		$num,.Loop_hw

	st1.32		{$ABCD,$EFGH},[$ctx]

	ldr		x29,[sp],#16
	ret
.size	${pre}sha256_block_armv8,.-${pre}sha256_block_armv8
___
}

if ($SZ==4) {	######################################### NEON stuff #
# You'll surely note a lot of similarities with sha256-armv4 module,
# and of course it's not a coincidence. sha256-armv4 was used as
# initial template, but was adapted for ARMv8 instruction set and
# extensively re-tuned for all-round performance.

my @V = ($A,$B,$C,$D,$E,$F,$G,$H) = map("w$_",(3..10));
my ($t0,$t1,$t2,$t3,$t4) = map("w$_",(11..15));
my $Ktbl="x16";
my $Xfer="x17";
my @X = map("q$_",(0..3));
my ($T0,$T1,$T2,$T3,$T4,$T5,$T6,$T7) = map("q$_",(4..7,16..19));
my $j=0;

sub AUTOLOAD()          # thunk [simplified] x86-style perlasm
{ my $opcode = $AUTOLOAD; $opcode =~ s/.*:://; $opcode =~ s/_/\./;
  my $arg = pop;
    $arg = "#$arg" if ($arg*1 eq $arg);
    $code .= "\t$opcode\t".join(',',@_,$arg)."\n";
}

sub Dscalar { shift =~ m|[qv]([0-9]+)|?"d$1":""; }
sub Dlo     { shift =~ m|[qv]([0-9]+)|?"v$1.d[0]":""; }
sub Dhi     { shift =~ m|[qv]([0-9]+)|?"v$1.d[1]":""; }

sub Xupdate()
{ use integer;
  my $body = shift;
  my @insns = (&$body,&$body,&$body,&$body);
  my ($a,$b,$c,$d,$e,$f,$g,$h);

	&ext_8		($T0,@X[0],@X[1],4);	# X[1..4]
	 eval(shift(@insns));
	 eval(shift(@insns));
	 eval(shift(@insns));
	&ext_8		($T3,@X[2],@X[3],4);	# X[9..12]
	 eval(shift(@insns));
	 eval(shift(@insns));
	&mov		(&Dscalar($T7),&Dhi(@X[3]));	# X[14..15]
	 eval(shift(@insns));
	 eval(shift(@insns));
	&ushr_32	($T2,$T0,$sigma0[0]);
	 eval(shift(@insns));
	&ushr_32	($T1,$T0,$sigma0[2]);
	 eval(shift(@insns));
	&add_32 	(@X[0],@X[0],$T3);	# X[0..3] += X[9..12]
	 eval(shift(@insns));
	&sli_32		($T2,$T0,32-$sigma0[0]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	&ushr_32	($T3,$T0,$sigma0[1]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	&eor_8		($T1,$T1,$T2);
	 eval(shift(@insns));
	 eval(shift(@insns));
	&sli_32		($T3,$T0,32-$sigma0[1]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	  &ushr_32	($T4,$T7,$sigma1[0]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	&eor_8		($T1,$T1,$T3);		# sigma0(X[1..4])
	 eval(shift(@insns));
	 eval(shift(@insns));
	  &sli_32	($T4,$T7,32-$sigma1[0]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	  &ushr_32	($T5,$T7,$sigma1[2]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	  &ushr_32	($T3,$T7,$sigma1[1]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	&add_32		(@X[0],@X[0],$T1);	# X[0..3] += sigma0(X[1..4])
	 eval(shift(@insns));
	 eval(shift(@insns));
	  &sli_u32	($T3,$T7,32-$sigma1[1]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	  &eor_8	($T5,$T5,$T4);
	 eval(shift(@insns));
	 eval(shift(@insns));
	 eval(shift(@insns));
	  &eor_8	($T5,$T5,$T3);		# sigma1(X[14..15])
	 eval(shift(@insns));
	 eval(shift(@insns));
	 eval(shift(@insns));
	&add_32		(@X[0],@X[0],$T5);	# X[0..1] += sigma1(X[14..15])
	 eval(shift(@insns));
	 eval(shift(@insns));
	 eval(shift(@insns));
	  &ushr_32	($T6,@X[0],$sigma1[0]);
	 eval(shift(@insns));
	  &ushr_32	($T7,@X[0],$sigma1[2]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	  &sli_32	($T6,@X[0],32-$sigma1[0]);
	 eval(shift(@insns));
	  &ushr_32	($T5,@X[0],$sigma1[1]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	  &eor_8	($T7,$T7,$T6);
	 eval(shift(@insns));
	 eval(shift(@insns));
	  &sli_32	($T5,@X[0],32-$sigma1[1]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	&ld1_32		("{$T0}","[$Ktbl], #16");
	 eval(shift(@insns));
	  &eor_8	($T7,$T7,$T5);		# sigma1(X[16..17])
	 eval(shift(@insns));
	 eval(shift(@insns));
	&eor_8		($T5,$T5,$T5);
	 eval(shift(@insns));
	 eval(shift(@insns));
	&mov		(&Dhi($T5), &Dlo($T7));
	 eval(shift(@insns));
	 eval(shift(@insns));
	 eval(shift(@insns));
	&add_32		(@X[0],@X[0],$T5);	# X[2..3] += sigma1(X[16..17])
	 eval(shift(@insns));
	 eval(shift(@insns));
	 eval(shift(@insns));
	&add_32		($T0,$T0,@X[0]);
	 while($#insns>=1) { eval(shift(@insns)); }
	&st1_32		("{$T0}","[$Xfer], #16");
	 eval(shift(@insns));

	push(@X,shift(@X));		# "rotate" X[]
}

sub Xpreload()
{ use integer;
  my $body = shift;
  my @insns = (&$body,&$body,&$body,&$body);
  my ($a,$b,$c,$d,$e,$f,$g,$h);

	 eval(shift(@insns));
	 eval(shift(@insns));
	&ld1_8		("{@X[0]}","[$inp],#16");
	 eval(shift(@insns));
	 eval(shift(@insns));
	&ld1_32		("{$T0}","[$Ktbl],#16");
	 eval(shift(@insns));
	 eval(shift(@insns));
	 eval(shift(@insns));
	 eval(shift(@insns));
	&rev32		(@X[0],@X[0]);
	 eval(shift(@insns));
	 eval(shift(@insns));
	 eval(shift(@insns));
	 eval(shift(@insns));
	&add_32		($T0,$T0,@X[0]);
	 foreach (@insns) { eval; }	# remaining instructions
	&st1_32		("{$T0}","[$Xfer], #16");

	push(@X,shift(@X));		# "rotate" X[]
}

sub body_00_15 () {
	(
	'($a,$b,$c,$d,$e,$f,$g,$h)=@V;'.
	'&add	($h,$h,$t1)',			# h+=X[i]+K[i]
	'&add	($a,$a,$t4);'.			# h+=Sigma0(a) from the past
	'&and	($t1,$f,$e)',
	'&bic	($t4,$g,$e)',
	'&eor	($t0,$e,$e,"ror#".($Sigma1[1]-$Sigma1[0]))',
	'&add	($a,$a,$t2)',			# h+=Maj(a,b,c) from the past
	'&orr	($t1,$t1,$t4)',			# Ch(e,f,g)
	'&eor	($t0,$t0,$e,"ror#".($Sigma1[2]-$Sigma1[0]))',	# Sigma1(e)
	'&eor	($t4,$a,$a,"ror#".($Sigma0[1]-$Sigma0[0]))',
	'&add	($h,$h,$t1)',			# h+=Ch(e,f,g)
	'&ror	($t0,$t0,"#$Sigma1[0]")',
	'&eor	($t2,$a,$b)',			# a^b, b^c in next round
	'&eor	($t4,$t4,$a,"ror#".($Sigma0[2]-$Sigma0[0]))',	# Sigma0(a)
	'&add	($h,$h,$t0)',			# h+=Sigma1(e)
	'&ldr	($t1,sprintf "[sp,#%d]",4*(($j+1)&15))	if (($j&15)!=15);'.
	'&ldr	($t1,"[$Ktbl]")				if ($j==15);'.
	'&and	($t3,$t3,$t2)',			# (b^c)&=(a^b)
	'&ror	($t4,$t4,"#$Sigma0[0]")',
	'&add	($d,$d,$h)',			# d+=h
	'&eor	($t3,$t3,$b)',			# Maj(a,b,c)
	'$j++;	unshift(@V,pop(@V)); ($t2,$t3)=($t3,$t2);'
	)
}

$code.=<<___;
.globl	${pre}sha256_block_data_order
.type	${pre}sha256_block_data_order,%function
.align	4
${pre}sha256_block_data_order:
	stp	x29, x30, [sp, #-16]!
	mov	x29, sp
	sub	sp,sp,#16*4

	adr	$Ktbl,.LK256
	add	$num,$inp,$num,lsl#6	// len to point at the end of inp

	ld1.8	{@X[0]},[$inp], #16
	ld1.8	{@X[1]},[$inp], #16
	ld1.8	{@X[2]},[$inp], #16
	ld1.8	{@X[3]},[$inp], #16
	ld1.32	{$T0},[$Ktbl], #16
	ld1.32	{$T1},[$Ktbl], #16
	ld1.32	{$T2},[$Ktbl], #16
	ld1.32	{$T3},[$Ktbl], #16
	rev32	@X[0],@X[0]		// yes, even on
	rev32	@X[1],@X[1]		// big-endian
	rev32	@X[2],@X[2]
	rev32	@X[3],@X[3]
	mov	$Xfer,sp
	add.32	$T0,$T0,@X[0]
	add.32	$T1,$T1,@X[1]
	add.32	$T2,$T2,@X[2]
	st1.32	{$T0-$T1},[$Xfer], #32
	add.32	$T3,$T3,@X[3]
	st1.32	{$T2-$T3},[$Xfer]
	sub	$Xfer,$Xfer,#32

	ldp	$A,$B,[$ctx]
	ldp	$C,$D,[$ctx,#8]
	ldp	$E,$F,[$ctx,#16]
	ldp	$G,$H,[$ctx,#24]
	ldr	$t1,[sp,#0]
	mov	$t2,wzr
	eor	$t3,$B,$C
	mov	$t4,wzr
	b	.L_00_48

.align	4
.L_00_48:
___
	&Xupdate(\&body_00_15);
	&Xupdate(\&body_00_15);
	&Xupdate(\&body_00_15);
	&Xupdate(\&body_00_15);
$code.=<<___;
	cmp	$t1,#0				// check for K256 terminator
	ldr	$t1,[sp,#0]
	sub	$Xfer,$Xfer,#64
	bne	.L_00_48

	sub	$Ktbl,$Ktbl,#256		// rewind $Ktbl
	cmp	$inp,$num
	mov	$Xfer, #64
	csel	$Xfer, $Xfer, xzr, eq
	sub	$inp,$inp,$Xfer			// avoid SEGV
	mov	$Xfer,sp
___
	&Xpreload(\&body_00_15);
	&Xpreload(\&body_00_15);
	&Xpreload(\&body_00_15);
	&Xpreload(\&body_00_15);
$code.=<<___;
	add	$A,$A,$t4			// h+=Sigma0(a) from the past
	ldp	$t0,$t1,[$ctx,#0]
	add	$A,$A,$t2			// h+=Maj(a,b,c) from the past
	ldp	$t2,$t3,[$ctx,#8]
	add	$A,$A,$t0			// accumulate
	add	$B,$B,$t1
	ldp	$t0,$t1,[$ctx,#16]
	add	$C,$C,$t2
	add	$D,$D,$t3
	ldp	$t2,$t3,[$ctx,#24]
	add	$E,$E,$t0
	add	$F,$F,$t1
	 ldr	$t1,[sp,#0]
	stp	$A,$B,[$ctx,#0]
	add	$G,$G,$t2
	 mov	$t2,wzr
	stp	$C,$D,[$ctx,#8]
	add	$H,$H,$t3
	stp	$E,$F,[$ctx,#16]
	 eor	$t3,$B,$C
	stp	$G,$H,[$ctx,#24]
	 mov	$t4,wzr
	 mov	$Xfer,sp
	b.ne	.L_00_48

	ldr	x29,[x29]
	add	sp,sp,#16*4+16
	ret
.size	${pre}sha256_block_data_order,.-${pre}sha256_block_data_order
___
}

{
my ($out,$inp,$len) = map("x$_",(0..2));

$code.=<<___;
.globl	${pre}sha256_emit
.hidden	${pre}sha256_emit
.type	${pre}sha256_emit,%function
.align	4
${pre}sha256_emit:
	ldp	x4,x5,[$inp]
	ldp	x6,x7,[$inp,#16]
#ifndef	__AARCH64EB__
	rev	x4,x4
	rev	x5,x5
	rev	x6,x6
	rev	x7,x7
#endif
	str	w4,[$out,#4]
	lsr	x4,x4,#32
	str	w5,[$out,#12]
	lsr	x5,x5,#32
	str	w6,[$out,#20]
	lsr	x6,x6,#32
	str	w7,[$out,#28]
	lsr	x7,x7,#32
	str	w4,[$out,#0]
	str	w5,[$out,#8]
	str	w6,[$out,#16]
	str	w7,[$out,#24]
	ret
.size	${pre}sha256_emit,.-${pre}sha256_emit

.globl	${pre}sha256_bcopy
.hidden	${pre}sha256_bcopy
.type	${pre}sha256_bcopy,%function
.align	4
${pre}sha256_bcopy:
.Loop_bcopy:
	ldrb	w3,[$inp],#1
	sub	$len,$len,#1
	strb	w3,[$out],#1
	cbnz	$len,.Loop_bcopy
	ret
.size	${pre}sha256_bcopy,.-${pre}sha256_bcopy

.globl	${pre}sha256_hcopy
.hidden	${pre}sha256_hcopy
.type	${pre}sha256_hcopy,%function
.align	4
${pre}sha256_hcopy:
	ldp	x4,x5,[$inp]
	ldp	x6,x7,[$inp,#16]
	stp	x4,x5,[$out]
	stp	x6,x7,[$out,#16]
	ret
.size	${pre}sha256_hcopy,.-${pre}sha256_hcopy
___
}

{   my  %opcode = (
	"sha256h"	=> 0x5e004000,	"sha256h2"	=> 0x5e005000,
	"sha256su0"	=> 0x5e282800,	"sha256su1"	=> 0x5e006000	);

    sub unsha256 {
	my ($mnemonic,$arg)=@_;

	$arg =~ m/[qv]([0-9]+)[^,]*,\s*[qv]([0-9]+)[^,]*(?:,\s*[qv]([0-9]+))?/o
	&&
	sprintf ".inst\t0x%08x\t//%s %s",
			$opcode{$mnemonic}|$1|($2<<5)|($3<<16),
			$mnemonic,$arg;
    }
}

open SELF,$0;
while(<SELF>) {
        next if (/^#!/);
        last if (!s/^#/\/\// and !/^$/);
        print;
}
close SELF;

foreach(split("\n",$code)) {

	s/\`([^\`]*)\`/eval($1)/ge;

	s/\b(sha512\w+)\s+([qv].*)/unsha512($1,$2)/ge	or
	s/\b(sha256\w+)\s+([qv].*)/unsha256($1,$2)/ge;

	s/\bq([0-9]+)\b/v$1.16b/g;		# old->new registers

	s/\.[ui]?8(\s)/$1/;
	s/\.\w?64\b//		and s/\.16b/\.2d/g	or
	s/\.\w?32\b//		and s/\.16b/\.4s/g;
	m/\bext\b/		and s/\.2d/\.16b/g	or
	m/(ld|st)1[^\[]+\[0\]/	and s/\.4s/\.s/g;

	print $_,"\n";
}

close STDOUT;
