#! /usr/bin/env perl
# Copyright 2015-2016 The OpenSSL Project Authors. All Rights Reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions
# are met:
#
# 1. Redistributions of source code must retain the above copyright
#    notice, this list of conditions and the following disclaimer. 
#
# 2. Redistributions in binary form must reproduce the above copyright
#    notice, this list of conditions and the following disclaimer in
#    the documentation and/or other materials provided with the
#    distribution.
#
# 3. All advertising materials mentioning features or use of this
#    software must display the following acknowledgment:
#    "This product includes software developed by the OpenSSL Project
#    for use in the OpenSSL Toolkit. (http://www.openssl.org/)"
#
# 4. The names "OpenSSL Toolkit" and "OpenSSL Project" must not be used to
#    endorse or promote products derived from this software without
#    prior written permission. For written permission, please contact
#    openssl-core@openssl.org.
#
# 5. Products derived from this software may not be called "OpenSSL"
#    nor may "OpenSSL" appear in their names without prior written
#    permission of the OpenSSL Project.
#
# 6. Redistributions of any form whatsoever must retain the following
#    acknowledgment:
#    "This product includes software developed by the OpenSSL Project
#    for use in the OpenSSL Toolkit (http://www.openssl.org/)"
#
# THIS SOFTWARE IS PROVIDED BY THE OpenSSL PROJECT ``AS IS'' AND ANY
# EXPRESSED OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
# IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR
# PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL THE OpenSSL PROJECT OR
# ITS CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
# SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
# NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
# LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
# HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
# STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
# ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED
# OF THE POSSIBILITY OF SUCH DAMAGE.
# ====================================================================
#
# This product includes cryptographic software written by Eric Young
# (eay@cryptsoft.com).  This product includes software written by Tim
# Hudson (tjh@cryptsoft.com).


# ====================================================================
# Written by Andy Polyakov <appro@openssl.org> for the OpenSSL
# project. The module is, however, dual licensed under OpenSSL and
# CRYPTOGAMS licenses depending on where you obtain it. For further
# details see http://www.openssl.org/~appro/cryptogams/.
# ====================================================================
#
# ECP_NISTZ256 module for x86/SSE2.
#
# October 2014.
#
# Original ECP_NISTZ256 submission targeting x86_64 is detailed in
# http://eprint.iacr.org/2013/816. In the process of adaptation
# original .c module was made 32-bit savvy in order to make this
# implementation possible.
#
#		with/without -DECP_NISTZ256_ASM
# Pentium	+66-163%
# PIII		+72-172%
# P4		+65-132%
# Core2		+90-215%
# Sandy Bridge	+105-265% (contemporary i[57]-* are all close to this)
# Atom		+65-155%
# Opteron	+54-110%
# Bulldozer	+99-240%
# VIA Nano	+93-290%
#
# Ranges denote minimum and maximum improvement coefficients depending
# on benchmark. Lower coefficients are for ECDSA sign, server-side
# operation. Keep in mind that +200% means 3x improvement.

$0 =~ m/(.*[\/\\])[^\/\\]+$/; $dir=$1;
push(@INC,"${dir}","${dir}../../../perlasm");
require "x86asm.pl";

$output=pop;
open STDOUT,">$output";

&asm_init($ARGV[0],"ecp_nistz256-x86.pl",$ARGV[$#ARGV] eq "386");

$sse2=0;
for (@ARGV) { $sse2=1 if (/-DOPENSSL_IA32_SSE2/); }

&external_label("GFp_ia32cap_P") if ($sse2);


########################################################################
# Keep in mind that constants are stored least to most significant word
&static_label("ONE_mont");
&set_label("ONE_mont");
&data_word(1,0,0,-1,-1,-1,-2,0);


&function_begin_B("_ecp_nistz256_div_by_2");
	# tmp = a is odd ? a+mod : a
	#
	# note that because mod has special form, i.e. consists of
	# 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	# assigning least significant bit of input to one register,
	# %ebp, and its negative to another, %edx.

	&mov	("ebp",&DWP(0,"esi"));
	&xor	("edx","edx");
	&mov	("ebx",&DWP(4,"esi"));
	&mov	("eax","ebp");
	&and	("ebp",1);
	&mov	("ecx",&DWP(8,"esi"));
	&sub	("edx","ebp");

	&add	("eax","edx");
	&adc	("ebx","edx");
	&mov	(&DWP(0,"edi"),"eax");
	&adc	("ecx","edx");
	&mov	(&DWP(4,"edi"),"ebx");
	&mov	(&DWP(8,"edi"),"ecx");

	&mov	("eax",&DWP(12,"esi"));
	&mov	("ebx",&DWP(16,"esi"));
	&adc	("eax",0);
	&mov	("ecx",&DWP(20,"esi"));
	&adc	("ebx",0);
	&mov	(&DWP(12,"edi"),"eax");
	&adc	("ecx",0);
	&mov	(&DWP(16,"edi"),"ebx");
	&mov	(&DWP(20,"edi"),"ecx");

	&mov	("eax",&DWP(24,"esi"));
	&mov	("ebx",&DWP(28,"esi"));
	&adc	("eax","ebp");
	&adc	("ebx","edx");
	&mov	(&DWP(24,"edi"),"eax");
	&sbb	("esi","esi");			# broadcast carry bit
	&mov	(&DWP(28,"edi"),"ebx");

	# ret = tmp >> 1

	&mov	("eax",&DWP(0,"edi"));
	&mov	("ebx",&DWP(4,"edi"));
	&mov	("ecx",&DWP(8,"edi"));
	&mov	("edx",&DWP(12,"edi"));

	&shr	("eax",1);
	&mov	("ebp","ebx");
	&shl	("ebx",31);
	&or	("eax","ebx");

	&shr	("ebp",1);
	&mov	("ebx","ecx");
	&shl	("ecx",31);
	&mov	(&DWP(0,"edi"),"eax");
	&or	("ebp","ecx");
	&mov	("eax",&DWP(16,"edi"));

	&shr	("ebx",1);
	&mov	("ecx","edx");
	&shl	("edx",31);
	&mov	(&DWP(4,"edi"),"ebp");
	&or	("ebx","edx");
	&mov	("ebp",&DWP(20,"edi"));

	&shr	("ecx",1);
	&mov	("edx","eax");
	&shl	("eax",31);
	&mov	(&DWP(8,"edi"),"ebx");
	&or	("ecx","eax");
	&mov	("ebx",&DWP(24,"edi"));

	&shr	("edx",1);
	&mov	("eax","ebp");
	&shl	("ebp",31);
	&mov	(&DWP(12,"edi"),"ecx");
	&or	("edx","ebp");
	&mov	("ecx",&DWP(28,"edi"));

	&shr	("eax",1);
	&mov	("ebp","ebx");
	&shl	("ebx",31);
	&mov	(&DWP(16,"edi"),"edx");
	&or	("eax","ebx");

	&shr	("ebp",1);
	&mov	("ebx","ecx");
	&shl	("ecx",31);
	&mov	(&DWP(20,"edi"),"eax");
	&or	("ebp","ecx");

	&shr	("ebx",1);
	&shl	("esi",31);
	&mov	(&DWP(24,"edi"),"ebp");
	&or	("ebx","esi");			# handle top-most carry bit
	&mov	(&DWP(28,"edi"),"ebx");

	&ret	();
&function_end_B("_ecp_nistz256_div_by_2");

########################################################################
# void GFp_nistz256_add(BN_ULONG edi[8],const BN_ULONG esi[8],
#					const BN_ULONG ebp[8]);
&function_begin("GFp_nistz256_add");
	&mov	("esi",&wparam(1));
	&mov	("ebp",&wparam(2));
	&mov	("edi",&wparam(0));
	&call	("_ecp_nistz256_add");
&function_end("GFp_nistz256_add");

&function_begin_B("_ecp_nistz256_add");
	&mov	("eax",&DWP(0,"esi"));
	&mov	("ebx",&DWP(4,"esi"));
	&mov	("ecx",&DWP(8,"esi"));
	&add	("eax",&DWP(0,"ebp"));
	&mov	("edx",&DWP(12,"esi"));
	&adc	("ebx",&DWP(4,"ebp"));
	&mov	(&DWP(0,"edi"),"eax");
	&adc	("ecx",&DWP(8,"ebp"));
	&mov	(&DWP(4,"edi"),"ebx");
	&adc	("edx",&DWP(12,"ebp"));
	&mov	(&DWP(8,"edi"),"ecx");
	&mov	(&DWP(12,"edi"),"edx");

	&mov	("eax",&DWP(16,"esi"));
	&mov	("ebx",&DWP(20,"esi"));
	&mov	("ecx",&DWP(24,"esi"));
	&adc	("eax",&DWP(16,"ebp"));
	&mov	("edx",&DWP(28,"esi"));
	&adc	("ebx",&DWP(20,"ebp"));
	&mov	(&DWP(16,"edi"),"eax");
	&adc	("ecx",&DWP(24,"ebp"));
	&mov	(&DWP(20,"edi"),"ebx");
	&mov	("esi",0);
	&adc	("edx",&DWP(28,"ebp"));
	&mov	(&DWP(24,"edi"),"ecx");
	&adc	("esi",0);
	&mov	(&DWP(28,"edi"),"edx");

	# if a+b >= modulus, subtract modulus.
	#
	# But since comparison implies subtraction, we subtract modulus
	# to see if it borrows, and then subtract it for real if
	# subtraction didn't borrow.

	&mov	("eax",&DWP(0,"edi"));
	&mov	("ebx",&DWP(4,"edi"));
	&mov	("ecx",&DWP(8,"edi"));
	&sub	("eax",-1);
	&mov	("edx",&DWP(12,"edi"));
	&sbb	("ebx",-1);
	&mov	("eax",&DWP(16,"edi"));
	&sbb	("ecx",-1);
	&mov	("ebx",&DWP(20,"edi"));
	&sbb	("edx",0);
	&mov	("ecx",&DWP(24,"edi"));
	&sbb	("eax",0);
	&mov	("edx",&DWP(28,"edi"));
	&sbb	("ebx",0);
	&sbb	("ecx",1);
	&sbb	("edx",-1);
	&sbb	("esi",0);

	# Note that because mod has special form, i.e. consists of
	# 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	# by using borrow.

	&not	("esi");
	&mov	("eax",&DWP(0,"edi"));
	&mov	("ebp","esi");
	&mov	("ebx",&DWP(4,"edi"));
	&shr	("ebp",31);
	&mov	("ecx",&DWP(8,"edi"));
	&sub	("eax","esi");
	&mov	("edx",&DWP(12,"edi"));
	&sbb	("ebx","esi");
	&mov	(&DWP(0,"edi"),"eax");
	&sbb	("ecx","esi");
	&mov	(&DWP(4,"edi"),"ebx");
	&sbb	("edx",0);
	&mov	(&DWP(8,"edi"),"ecx");
	&mov	(&DWP(12,"edi"),"edx");

	&mov	("eax",&DWP(16,"edi"));
	&mov	("ebx",&DWP(20,"edi"));
	&mov	("ecx",&DWP(24,"edi"));
	&sbb	("eax",0);
	&mov	("edx",&DWP(28,"edi"));
	&sbb	("ebx",0);
	&mov	(&DWP(16,"edi"),"eax");
	&sbb	("ecx","ebp");
	&mov	(&DWP(20,"edi"),"ebx");
	&sbb	("edx","esi");
	&mov	(&DWP(24,"edi"),"ecx");
	&mov	(&DWP(28,"edi"),"edx");

	&ret	();
&function_end_B("_ecp_nistz256_add");

&function_begin_B("_ecp_nistz256_sub");
	&mov	("eax",&DWP(0,"esi"));
	&mov	("ebx",&DWP(4,"esi"));
	&mov	("ecx",&DWP(8,"esi"));
	&sub	("eax",&DWP(0,"ebp"));
	&mov	("edx",&DWP(12,"esi"));
	&sbb	("ebx",&DWP(4,"ebp"));
	&mov	(&DWP(0,"edi"),"eax");
	&sbb	("ecx",&DWP(8,"ebp"));
	&mov	(&DWP(4,"edi"),"ebx");
	&sbb	("edx",&DWP(12,"ebp"));
	&mov	(&DWP(8,"edi"),"ecx");
	&mov	(&DWP(12,"edi"),"edx");

	&mov	("eax",&DWP(16,"esi"));
	&mov	("ebx",&DWP(20,"esi"));
	&mov	("ecx",&DWP(24,"esi"));
	&sbb	("eax",&DWP(16,"ebp"));
	&mov	("edx",&DWP(28,"esi"));
	&sbb	("ebx",&DWP(20,"ebp"));
	&sbb	("ecx",&DWP(24,"ebp"));
	&mov	(&DWP(16,"edi"),"eax");
	&sbb	("edx",&DWP(28,"ebp"));
	&mov	(&DWP(20,"edi"),"ebx");
	&sbb	("esi","esi");			# broadcast borrow bit
	&mov	(&DWP(24,"edi"),"ecx");
	&mov	(&DWP(28,"edi"),"edx");

	# if a-b borrows, add modulus.
	#
	# Note that because mod has special form, i.e. consists of
	# 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	# assigning borrow bit to one register, %ebp, and its negative
	# to another, %esi. But we started by calculating %esi...

	&mov	("eax",&DWP(0,"edi"));
	&mov	("ebp","esi");
	&mov	("ebx",&DWP(4,"edi"));
	&shr	("ebp",31);
	&mov	("ecx",&DWP(8,"edi"));
	&add	("eax","esi");
	&mov	("edx",&DWP(12,"edi"));
	&adc	("ebx","esi");
	&mov	(&DWP(0,"edi"),"eax");
	&adc	("ecx","esi");
	&mov	(&DWP(4,"edi"),"ebx");
	&adc	("edx",0);
	&mov	(&DWP(8,"edi"),"ecx");
	&mov	(&DWP(12,"edi"),"edx");

	&mov	("eax",&DWP(16,"edi"));
	&mov	("ebx",&DWP(20,"edi"));
	&mov	("ecx",&DWP(24,"edi"));
	&adc	("eax",0);
	&mov	("edx",&DWP(28,"edi"));
	&adc	("ebx",0);
	&mov	(&DWP(16,"edi"),"eax");
	&adc	("ecx","ebp");
	&mov	(&DWP(20,"edi"),"ebx");
	&adc	("edx","esi");
	&mov	(&DWP(24,"edi"),"ecx");
	&mov	(&DWP(28,"edi"),"edx");

	&ret	();
&function_end_B("_ecp_nistz256_sub");

########################################################################
# void GFp_nistz256_neg(BN_ULONG edi[8],const BN_ULONG esi[8]);
&function_begin("GFp_nistz256_neg");
	&mov	("ebp",&wparam(1));
	&mov	("edi",&wparam(0));

	&xor	("eax","eax");
	&stack_push(8);
	&mov	(&DWP(0,"esp"),"eax");
	&mov	("esi","esp");
	&mov	(&DWP(4,"esp"),"eax");
	&mov	(&DWP(8,"esp"),"eax");
	&mov	(&DWP(12,"esp"),"eax");
	&mov	(&DWP(16,"esp"),"eax");
	&mov	(&DWP(20,"esp"),"eax");
	&mov	(&DWP(24,"esp"),"eax");
	&mov	(&DWP(28,"esp"),"eax");

	&call	("_ecp_nistz256_sub");

	&stack_pop(8);
&function_end("GFp_nistz256_neg");

&function_begin_B("_picup_eax");
	&mov	("eax",&DWP(0,"esp"));
	&ret	();
&function_end_B("_picup_eax");

########################################################################
# void GFp_nistz256_mul_mont(BN_ULONG edi[8],const BN_ULONG esi[8],
#					     const BN_ULONG ebp[8]);
&function_begin("GFp_nistz256_mul_mont");
	&mov	("esi",&wparam(1));
	&mov	("ebp",&wparam(2));
						if ($sse2) {
	&call	("_picup_eax");
    &set_label("pic");
	&picmeup("eax","GFp_ia32cap_P","eax",&label("pic"));
	&mov	("eax",&DWP(0,"eax"));		}
	&mov	("edi",&wparam(0));
	&call	("_ecp_nistz256_mul_mont");
&function_end("GFp_nistz256_mul_mont");

&function_begin_B("_ecp_nistz256_mul_mont");
						if ($sse2) {
	# We always use SSE2

	########################################
	# SSE2 code path featuring 32x16-bit
	# multiplications is ~2x faster than
	# IALU counterpart (except on Atom)...
	########################################
	# stack layout:
	# +------------------------------------+< %esp
	# | 7 16-byte temporary XMM words,     |
	# | "sliding" toward lower address     |
	# .                                    .
	# +------------------------------------+
	# | unused XMM word                    |
	# +------------------------------------+< +128,%ebx
	# | 8 16-byte XMM words holding copies |
	# | of a[i]<<64|a[i]                   |
	# .                                    .
	# .                                    .
	# +------------------------------------+< +256
	&mov	("edx","esp");
	&sub	("esp",0x100);

	&movd	("xmm7",&DWP(0,"ebp"));		# b[0] -> 0000.00xy
	&lea	("ebp",&DWP(4,"ebp"));
	&pcmpeqd("xmm6","xmm6");
	&psrlq	("xmm6",48);			# compose 0xffff<<64|0xffff

	&pshuflw("xmm7","xmm7",0b11011100);	# 0000.00xy -> 0000.0x0y
	&and	("esp",-64);
	&pshufd	("xmm7","xmm7",0b11011100);	# 0000.0x0y -> 000x.000y
	&lea	("ebx",&DWP(0x80,"esp"));

	&movd	("xmm0",&DWP(4*0,"esi"));	# a[0] -> 0000.00xy
	&pshufd	("xmm0","xmm0",0b11001100);	# 0000.00xy -> 00xy.00xy
	&movd	("xmm1",&DWP(4*1,"esi"));	# a[1] -> ...
	&movdqa	(&QWP(0x00,"ebx"),"xmm0");	# offload converted a[0]
	&pmuludq("xmm0","xmm7");		# a[0]*b[0]

	&movd	("xmm2",&DWP(4*2,"esi"));
	&pshufd	("xmm1","xmm1",0b11001100);
	&movdqa	(&QWP(0x10,"ebx"),"xmm1");
	&pmuludq("xmm1","xmm7");		# a[1]*b[0]

	 &movq	("xmm4","xmm0");		# clear upper 64 bits
	 &pslldq("xmm4",6);
	 &paddq	("xmm4","xmm0");
	 &movdqa("xmm5","xmm4");
	 &psrldq("xmm4",10);			# upper 32 bits of a[0]*b[0]
	 &pand	("xmm5","xmm6");		# lower 32 bits of a[0]*b[0]

	# Upper half of a[0]*b[i] is carried into next multiplication
	# iteration, while lower one "participates" in actual reduction.
	# Normally latter is done by accumulating result of multiplication
	# of modulus by "magic" digit, but thanks to special form of modulus
	# and "magic" digit it can be performed only with additions and
	# subtractions (see note in IALU section below). Note that we are
	# not bothered with carry bits, they are accumulated in "flatten"
	# phase after all multiplications and reductions.

	&movd	("xmm3",&DWP(4*3,"esi"));
	&pshufd	("xmm2","xmm2",0b11001100);
	&movdqa	(&QWP(0x20,"ebx"),"xmm2");
	&pmuludq("xmm2","xmm7");		# a[2]*b[0]
	 &paddq	("xmm1","xmm4");		# a[1]*b[0]+hw(a[0]*b[0]), carry
	&movdqa	(&QWP(0x00,"esp"),"xmm1");	# t[0]

	&movd	("xmm0",&DWP(4*4,"esi"));
	&pshufd	("xmm3","xmm3",0b11001100);
	&movdqa	(&QWP(0x30,"ebx"),"xmm3");
	&pmuludq("xmm3","xmm7");		# a[3]*b[0]
	&movdqa	(&QWP(0x10,"esp"),"xmm2");

	&movd	("xmm1",&DWP(4*5,"esi"));
	&pshufd	("xmm0","xmm0",0b11001100);
	&movdqa	(&QWP(0x40,"ebx"),"xmm0");
	&pmuludq("xmm0","xmm7");		# a[4]*b[0]
	 &paddq	("xmm3","xmm5");		# a[3]*b[0]+lw(a[0]*b[0]), reduction step
	&movdqa	(&QWP(0x20,"esp"),"xmm3");

	&movd	("xmm2",&DWP(4*6,"esi"));
	&pshufd	("xmm1","xmm1",0b11001100);
	&movdqa	(&QWP(0x50,"ebx"),"xmm1");
	&pmuludq("xmm1","xmm7");		# a[5]*b[0]
	&movdqa	(&QWP(0x30,"esp"),"xmm0");
	 &pshufd("xmm4","xmm5",0b10110001);	# xmm4 = xmm5<<32, reduction step

	&movd	("xmm3",&DWP(4*7,"esi"));
	&pshufd	("xmm2","xmm2",0b11001100);
	&movdqa	(&QWP(0x60,"ebx"),"xmm2");
	&pmuludq("xmm2","xmm7");		# a[6]*b[0]
	&movdqa	(&QWP(0x40,"esp"),"xmm1");
	 &psubq	("xmm4","xmm5");		# xmm4 = xmm5*0xffffffff, reduction step

	&movd	("xmm0",&DWP(0,"ebp"));		# b[1] -> 0000.00xy
	&pshufd	("xmm3","xmm3",0b11001100);
	&movdqa	(&QWP(0x70,"ebx"),"xmm3");
	&pmuludq("xmm3","xmm7");		# a[7]*b[0]

	&pshuflw("xmm7","xmm0",0b11011100);	# 0000.00xy -> 0000.0x0y
	&movdqa	("xmm0",&QWP(0x00,"ebx"));	# pre-load converted a[0]
	&pshufd	("xmm7","xmm7",0b11011100);	# 0000.0x0y -> 000x.000y

	&mov	("ecx",6);
	&lea	("ebp",&DWP(4,"ebp"));
	&jmp	(&label("madd_sse2"));

&set_label("madd_sse2",16);
	 &paddq	("xmm2","xmm5");		# a[6]*b[i-1]+lw(a[0]*b[i-1]), reduction step [modulo-scheduled]
	 &paddq	("xmm3","xmm4");		# a[7]*b[i-1]+lw(a[0]*b[i-1])*0xffffffff, reduction step [modulo-scheduled]
	&movdqa	("xmm1",&QWP(0x10,"ebx"));
	&pmuludq("xmm0","xmm7");		# a[0]*b[i]
	 &movdqa(&QWP(0x50,"esp"),"xmm2");

	&movdqa	("xmm2",&QWP(0x20,"ebx"));
	&pmuludq("xmm1","xmm7");		# a[1]*b[i]
	 &movdqa(&QWP(0x60,"esp"),"xmm3");
	&paddq	("xmm0",&QWP(0x00,"esp"));

	&movdqa	("xmm3",&QWP(0x30,"ebx"));
	&pmuludq("xmm2","xmm7");		# a[2]*b[i]
	 &movq	("xmm4","xmm0");		# clear upper 64 bits
	 &pslldq("xmm4",6);
	&paddq	("xmm1",&QWP(0x10,"esp"));
	 &paddq	("xmm4","xmm0");
	 &movdqa("xmm5","xmm4");
	 &psrldq("xmm4",10);			# upper 33 bits of a[0]*b[i]+t[0]

	&movdqa	("xmm0",&QWP(0x40,"ebx"));
	&pmuludq("xmm3","xmm7");		# a[3]*b[i]
	 &paddq	("xmm1","xmm4");		# a[1]*b[i]+hw(a[0]*b[i]), carry
	&paddq	("xmm2",&QWP(0x20,"esp"));
	&movdqa	(&QWP(0x00,"esp"),"xmm1");

	&movdqa	("xmm1",&QWP(0x50,"ebx"));
	&pmuludq("xmm0","xmm7");		# a[4]*b[i]
	&paddq	("xmm3",&QWP(0x30,"esp"));
	&movdqa	(&QWP(0x10,"esp"),"xmm2");
	 &pand	("xmm5","xmm6");		# lower 32 bits of a[0]*b[i]

	&movdqa	("xmm2",&QWP(0x60,"ebx"));
	&pmuludq("xmm1","xmm7");		# a[5]*b[i]
	 &paddq	("xmm3","xmm5");		# a[3]*b[i]+lw(a[0]*b[i]), reduction step
	&paddq	("xmm0",&QWP(0x40,"esp"));
	&movdqa	(&QWP(0x20,"esp"),"xmm3");
	 &pshufd("xmm4","xmm5",0b10110001);	# xmm4 = xmm5<<32, reduction step

	&movdqa	("xmm3","xmm7");
	&pmuludq("xmm2","xmm7");		# a[6]*b[i]
	 &movd	("xmm7",&DWP(0,"ebp"));		# b[i++] -> 0000.00xy
	 &lea	("ebp",&DWP(4,"ebp"));
	&paddq	("xmm1",&QWP(0x50,"esp"));
	 &psubq	("xmm4","xmm5");		# xmm4 = xmm5*0xffffffff, reduction step
	&movdqa	(&QWP(0x30,"esp"),"xmm0");
	 &pshuflw("xmm7","xmm7",0b11011100);	# 0000.00xy -> 0000.0x0y

	&pmuludq("xmm3",&QWP(0x70,"ebx"));	# a[7]*b[i]
	 &pshufd("xmm7","xmm7",0b11011100);	# 0000.0x0y -> 000x.000y
	 &movdqa("xmm0",&QWP(0x00,"ebx"));	# pre-load converted a[0]
	&movdqa	(&QWP(0x40,"esp"),"xmm1");
	&paddq	("xmm2",&QWP(0x60,"esp"));

	&dec	("ecx");
	&jnz	(&label("madd_sse2"));

	 &paddq	("xmm2","xmm5");		# a[6]*b[6]+lw(a[0]*b[6]), reduction step [modulo-scheduled]
	 &paddq	("xmm3","xmm4");		# a[7]*b[6]+lw(a[0]*b[6])*0xffffffff, reduction step [modulo-scheduled]
	&movdqa	("xmm1",&QWP(0x10,"ebx"));
	&pmuludq("xmm0","xmm7");		# a[0]*b[7]
	 &movdqa(&QWP(0x50,"esp"),"xmm2");

	&movdqa	("xmm2",&QWP(0x20,"ebx"));
	&pmuludq("xmm1","xmm7");		# a[1]*b[7]
	 &movdqa(&QWP(0x60,"esp"),"xmm3");
	&paddq	("xmm0",&QWP(0x00,"esp"));

	&movdqa	("xmm3",&QWP(0x30,"ebx"));
	&pmuludq("xmm2","xmm7");		# a[2]*b[7]
	 &movq	("xmm4","xmm0");		# clear upper 64 bits
	 &pslldq("xmm4",6);
	&paddq	("xmm1",&QWP(0x10,"esp"));
	 &paddq	("xmm4","xmm0");
	 &movdqa("xmm5","xmm4");
	 &psrldq("xmm4",10);			# upper 33 bits of a[0]*b[i]+t[0]

	&movdqa	("xmm0",&QWP(0x40,"ebx"));
	&pmuludq("xmm3","xmm7");		# a[3]*b[7]
	 &paddq	("xmm1","xmm4");		# a[1]*b[7]+hw(a[0]*b[7]), carry
	&paddq	("xmm2",&QWP(0x20,"esp"));
	&movdqa	(&QWP(0x00,"esp"),"xmm1");

	&movdqa	("xmm1",&QWP(0x50,"ebx"));
	&pmuludq("xmm0","xmm7");		# a[4]*b[7]
	&paddq	("xmm3",&QWP(0x30,"esp"));
	&movdqa	(&QWP(0x10,"esp"),"xmm2");
	 &pand	("xmm5","xmm6");		# lower 32 bits of a[0]*b[i]

	&movdqa	("xmm2",&QWP(0x60,"ebx"));
	&pmuludq("xmm1","xmm7");		# a[5]*b[7]
	 &paddq	("xmm3","xmm5");		# reduction step
	&paddq	("xmm0",&QWP(0x40,"esp"));
	&movdqa	(&QWP(0x20,"esp"),"xmm3");
	 &pshufd("xmm4","xmm5",0b10110001);	# xmm4 = xmm5<<32, reduction step

	&movdqa	("xmm3",&QWP(0x70,"ebx"));
	&pmuludq("xmm2","xmm7");		# a[6]*b[7]
	&paddq	("xmm1",&QWP(0x50,"esp"));
	 &psubq	("xmm4","xmm5");		# xmm4 = xmm5*0xffffffff, reduction step
	&movdqa	(&QWP(0x30,"esp"),"xmm0");

	&pmuludq("xmm3","xmm7");		# a[7]*b[7]
	&pcmpeqd("xmm7","xmm7");
	&movdqa	("xmm0",&QWP(0x00,"esp"));
	&pslldq	("xmm7",8);
	&movdqa	(&QWP(0x40,"esp"),"xmm1");
	&paddq	("xmm2",&QWP(0x60,"esp"));

	 &paddq	("xmm2","xmm5");		# a[6]*b[7]+lw(a[0]*b[7]), reduction step
	 &paddq	("xmm3","xmm4");		# a[6]*b[7]+lw(a[0]*b[7])*0xffffffff, reduction step
	 &movdqa(&QWP(0x50,"esp"),"xmm2");
	 &movdqa(&QWP(0x60,"esp"),"xmm3");

	&movdqa	("xmm1",&QWP(0x10,"esp"));
	&movdqa	("xmm2",&QWP(0x20,"esp"));
	&movdqa	("xmm3",&QWP(0x30,"esp"));

	&movq	("xmm4","xmm0");		# "flatten"
	&pand	("xmm0","xmm7");
	&xor	("ebp","ebp");
	&pslldq	("xmm4",6);
	 &movq	("xmm5","xmm1");
	&paddq	("xmm0","xmm4");
	 &pand	("xmm1","xmm7");
	&psrldq	("xmm0",6);
	&movd	("eax","xmm0");
	&psrldq	("xmm0",4);

	&paddq	("xmm5","xmm0");
	&movdqa	("xmm0",&QWP(0x40,"esp"));
	&sub	("eax",-1);			# start subtracting modulus,
						# this is used to determine
						# if result is larger/smaller
						# than modulus (see below)
	&pslldq	("xmm5",6);
	 &movq	("xmm4","xmm2");
	&paddq	("xmm1","xmm5");
	 &pand	("xmm2","xmm7");
	&psrldq	("xmm1",6);
	&mov	(&DWP(4*0,"edi"),"eax");
	&movd	("eax","xmm1");
	&psrldq	("xmm1",4);

	&paddq	("xmm4","xmm1");
	&movdqa	("xmm1",&QWP(0x50,"esp"));
	&sbb	("eax",-1);
	&pslldq	("xmm4",6);
	 &movq	("xmm5","xmm3");
	&paddq	("xmm2","xmm4");
	 &pand	("xmm3","xmm7");
	&psrldq	("xmm2",6);
	&mov	(&DWP(4*1,"edi"),"eax");
	&movd	("eax","xmm2");
	&psrldq	("xmm2",4);

	&paddq	("xmm5","xmm2");
	&movdqa	("xmm2",&QWP(0x60,"esp"));
	&sbb	("eax",-1);
	&pslldq	("xmm5",6);
	 &movq	("xmm4","xmm0");
	&paddq	("xmm3","xmm5");
	 &pand	("xmm0","xmm7");
	&psrldq	("xmm3",6);
	&mov	(&DWP(4*2,"edi"),"eax");
	&movd	("eax","xmm3");
	&psrldq	("xmm3",4);

	&paddq	("xmm4","xmm3");
	&sbb	("eax",0);
	&pslldq	("xmm4",6);
	 &movq	("xmm5","xmm1");
	&paddq	("xmm0","xmm4");
	 &pand	("xmm1","xmm7");
	&psrldq	("xmm0",6);
	&mov	(&DWP(4*3,"edi"),"eax");
	&movd	("eax","xmm0");
	&psrldq	("xmm0",4);

	&paddq	("xmm5","xmm0");
	&sbb	("eax",0);
	&pslldq	("xmm5",6);
	 &movq	("xmm4","xmm2");
	&paddq	("xmm1","xmm5");
	 &pand	("xmm2","xmm7");
	&psrldq	("xmm1",6);
	&movd	("ebx","xmm1");
	&psrldq	("xmm1",4);
	&mov	("esp","edx");

	&paddq	("xmm4","xmm1");
	&pslldq	("xmm4",6);
	&paddq	("xmm2","xmm4");
	&psrldq	("xmm2",6);
	&movd	("ecx","xmm2");
	&psrldq	("xmm2",4);
	&sbb	("ebx",0);
	&movd	("edx","xmm2");
	&pextrw	("esi","xmm2",2);		# top-most overflow bit
	&sbb	("ecx",1);
	&sbb	("edx",-1);
	&sbb	("esi",0);			# borrow from subtraction

	# Final step is "if result > mod, subtract mod", and at this point
	# we have result - mod written to output buffer, as well as borrow
	# bit from this subtraction, and if borrow bit is set, we add
	# modulus back.
	#
	# Note that because mod has special form, i.e. consists of
	# 0xffffffff, 1 and 0s, we can conditionally synthesize it by
	# assigning borrow bit to one register, %ebp, and its negative
	# to another, %esi. But we started by calculating %esi...

	&sub	("ebp","esi");
	&add	(&DWP(4*0,"edi"),"esi");	# add modulus or zero
	&adc	(&DWP(4*1,"edi"),"esi");
	&adc	(&DWP(4*2,"edi"),"esi");
	&adc	(&DWP(4*3,"edi"),0);
	&adc	("eax",0);
	&adc	("ebx",0);
	&mov	(&DWP(4*4,"edi"),"eax");
	&adc	("ecx","ebp");
	&mov	(&DWP(4*5,"edi"),"ebx");
	&adc	("edx","esi");
	&mov	(&DWP(4*6,"edi"),"ecx");
	&mov	(&DWP(4*7,"edi"),"edx");

	&ret	();

}	# Non-SSE2 code removed.

&function_end_B("_ecp_nistz256_mul_mont");

########################################################################
# following subroutines are "literal" implementation of those found in
# ecp_nistz256.c
#
########################################################################
# void GFp_nistz256_point_double(P256_POINT *out,const P256_POINT *inp);
#
&static_label("point_double_shortcut");
&function_begin("GFp_nistz256_point_double");
{   my ($S,$M,$Zsqr,$in_x,$tmp0)=map(32*$_,(0..4));

	&mov	("esi",&wparam(1));

	# above map() describes stack layout with 5 temporary
	# 256-bit vectors on top, then we take extra word for
	# GFp_ia32cap_P copy.
	&stack_push(8*5+1);
						if ($sse2) {
	&call	("_picup_eax");
    &set_label("pic");
	&picmeup("edx","GFp_ia32cap_P","eax",&label("pic"));
	&mov	("ebp",&DWP(0,"edx"));		}

&set_label("point_double_shortcut");
	&mov	("eax",&DWP(0,"esi"));		# copy in_x
	&mov	("ebx",&DWP(4,"esi"));
	&mov	("ecx",&DWP(8,"esi"));
	&mov	("edx",&DWP(12,"esi"));
	&mov	(&DWP($in_x+0,"esp"),"eax");
	&mov	(&DWP($in_x+4,"esp"),"ebx");
	&mov	(&DWP($in_x+8,"esp"),"ecx");
	&mov	(&DWP($in_x+12,"esp"),"edx");
	&mov	("eax",&DWP(16,"esi"));
	&mov	("ebx",&DWP(20,"esi"));
	&mov	("ecx",&DWP(24,"esi"));
	&mov	("edx",&DWP(28,"esi"));
	&mov	(&DWP($in_x+16,"esp"),"eax");
	&mov	(&DWP($in_x+20,"esp"),"ebx");
	&mov	(&DWP($in_x+24,"esp"),"ecx");
	&mov	(&DWP($in_x+28,"esp"),"edx");
	&mov	(&DWP(32*5,"esp"),"ebp");	# GFp_ia32cap_P copy

	&lea	("ebp",&DWP(32,"esi"));
	&lea	("esi",&DWP(32,"esi"));
	&lea	("edi",&DWP($S,"esp"));
	&call	("_ecp_nistz256_add");		# p256_mul_by_2(S, in_y);

	&mov	("eax",&DWP(32*5,"esp"));	# GFp_ia32cap_P copy
	&mov	("esi",64);
	&add	("esi",&wparam(1));
	&lea	("edi",&DWP($Zsqr,"esp"));
	&mov	("ebp","esi");
	&call	("_ecp_nistz256_mul_mont");	# p256_sqr_mont(Zsqr, in_z);

	&mov	("eax",&DWP(32*5,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($S,"esp"));
	&lea	("ebp",&DWP($S,"esp"));
	&lea	("edi",&DWP($S,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_sqr_mont(S, S);

	&mov	("eax",&DWP(32*5,"esp"));	# GFp_ia32cap_P copy
	&mov	("ebp",&wparam(1));
	&lea	("esi",&DWP(32,"ebp"));
	&lea	("ebp",&DWP(64,"ebp"));
	&lea	("edi",&DWP($tmp0,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(tmp0, in_z, in_y);

	&lea	("esi",&DWP($in_x,"esp"));
	&lea	("ebp",&DWP($Zsqr,"esp"));
	&lea	("edi",&DWP($M,"esp"));
	&call	("_ecp_nistz256_add");		# p256_add(M, in_x, Zsqr);

	&mov	("edi",64);
	&lea	("esi",&DWP($tmp0,"esp"));
	&lea	("ebp",&DWP($tmp0,"esp"));
	&add	("edi",&wparam(0));
	&call	("_ecp_nistz256_add");		# p256_mul_by_2(res_z, tmp0);

	&lea	("esi",&DWP($in_x,"esp"));
	&lea	("ebp",&DWP($Zsqr,"esp"));
	&lea	("edi",&DWP($Zsqr,"esp"));
	&call	("_ecp_nistz256_sub");		# p256_sub(Zsqr, in_x, Zsqr);

	&mov	("eax",&DWP(32*5,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($S,"esp"));
	&lea	("ebp",&DWP($S,"esp"));
	&lea	("edi",&DWP($tmp0,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_sqr_mont(tmp0, S);

	&mov	("eax",&DWP(32*5,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($M,"esp"));
	&lea	("ebp",&DWP($Zsqr,"esp"));
	&lea	("edi",&DWP($M,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(M, M, Zsqr);

	&mov	("edi",32);
	&lea	("esi",&DWP($tmp0,"esp"));
	&add	("edi",&wparam(0));
	&call	("_ecp_nistz256_div_by_2");	# p256_div_by_2(res_y, tmp0);

	&lea	("esi",&DWP($M,"esp"));
	&lea	("ebp",&DWP($M,"esp"));
	&lea	("edi",&DWP($tmp0,"esp"));
	&call	("_ecp_nistz256_add");		# 1/2 p256_mul_by_3(M, M);

	&mov	("eax",&DWP(32*5,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($in_x,"esp"));
	&lea	("ebp",&DWP($S,"esp"));
	&lea	("edi",&DWP($S,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(S, S, in_x);

	&lea	("esi",&DWP($tmp0,"esp"));
	&lea	("ebp",&DWP($M,"esp"));
	&lea	("edi",&DWP($M,"esp"));
	&call	("_ecp_nistz256_add");		# 2/2 p256_mul_by_3(M, M);

	&lea	("esi",&DWP($S,"esp"));
	&lea	("ebp",&DWP($S,"esp"));
	&lea	("edi",&DWP($tmp0,"esp"));
	&call	("_ecp_nistz256_add");		# p256_mul_by_2(tmp0, S);

	&mov	("eax",&DWP(32*5,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($M,"esp"));
	&lea	("ebp",&DWP($M,"esp"));
	&mov	("edi",&wparam(0));
	&call	("_ecp_nistz256_mul_mont");	# p256_sqr_mont(res_x, M);

	&mov	("esi","edi");			# %edi is still res_x here
	&lea	("ebp",&DWP($tmp0,"esp"));
	&call	("_ecp_nistz256_sub");		# p256_sub(res_x, res_x, tmp0);

	&lea	("esi",&DWP($S,"esp"));
	&mov	("ebp","edi");			# %edi is still res_x
	&lea	("edi",&DWP($S,"esp"));
	&call	("_ecp_nistz256_sub");		# p256_sub(S, S, res_x);

	&mov	("eax",&DWP(32*5,"esp"));	# GFp_ia32cap_P copy
	&mov	("esi","edi");			# %edi is still &S
	&lea	("ebp",&DWP($M,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(S, S, M);

	&mov	("ebp",32);
	&lea	("esi",&DWP($S,"esp"));
	&add	("ebp",&wparam(0));
	&mov	("edi","ebp");
	&call	("_ecp_nistz256_sub");		# p256_sub(res_y, S, res_y);

	&stack_pop(8*5+1);
} &function_end("GFp_nistz256_point_double");

########################################################################
# void GFp_nistz256_point_add_affine(P256_POINT *out,
#				     const P256_POINT *in1,
#				     const P256_POINT_AFFINE *in2);
&function_begin("GFp_nistz256_point_add_affine");
{
    my ($res_x,$res_y,$res_z,
	$in1_x,$in1_y,$in1_z,
	$in2_x,$in2_y,
	$U2,$S2,$H,$R,$Hsqr,$Hcub,$Rsqr)=map(32*$_,(0..14));
    my $Z1sqr = $S2;
    my @ONE_mont=(1,0,0,-1,-1,-1,-2,0);

	&mov	("esi",&wparam(1));

	# above map() describes stack layout with 15 temporary
	# 256-bit vectors on top, then we take extra words for
	# !in1infty, !in2infty, and GFp_ia32cap_P copy.
	&stack_push(8*15+3);
						if ($sse2) {
	&call	("_picup_eax");
    &set_label("pic");
	&picmeup("edx","GFp_ia32cap_P","eax",&label("pic"));
	&mov	("ebp",&DWP(0,"edx"));		}

	&lea	("edi",&DWP($in1_x,"esp"));
    for($i=0;$i<96;$i+=16) {
	&mov	("eax",&DWP($i+0,"esi"));	# copy in1
	&mov	("ebx",&DWP($i+4,"esi"));
	&mov	("ecx",&DWP($i+8,"esi"));
	&mov	("edx",&DWP($i+12,"esi"));
	&mov	(&DWP($i+0,"edi"),"eax");
	&mov	(&DWP(32*15+8,"esp"),"ebp")	if ($i==0);
	&mov	("ebp","eax")			if ($i==64);
	&or	("ebp","eax")			if ($i>64);
	&mov	(&DWP($i+4,"edi"),"ebx");
	&or	("ebp","ebx")			if ($i>=64);
	&mov	(&DWP($i+8,"edi"),"ecx");
	&or	("ebp","ecx")			if ($i>=64);
	&mov	(&DWP($i+12,"edi"),"edx");
	&or	("ebp","edx")			if ($i>=64);
    }
	&xor	("eax","eax");
	&mov	("esi",&wparam(2));
	&sub	("eax","ebp");
	&or	("ebp","eax");
	&sar	("ebp",31);
	&mov	(&DWP(32*15+0,"esp"),"ebp");	# !in1infty

	&lea	("edi",&DWP($in2_x,"esp"));
    for($i=0;$i<64;$i+=16) {
	&mov	("eax",&DWP($i+0,"esi"));	# copy in2
	&mov	("ebx",&DWP($i+4,"esi"));
	&mov	("ecx",&DWP($i+8,"esi"));
	&mov	("edx",&DWP($i+12,"esi"));
	&mov	(&DWP($i+0,"edi"),"eax");
	&mov	("ebp","eax")			if ($i==0);
	&or	("ebp","eax")			if ($i!=0);
	&mov	(&DWP($i+4,"edi"),"ebx");
	&or	("ebp","ebx");
	&mov	(&DWP($i+8,"edi"),"ecx");
	&or	("ebp","ecx");
	&mov	(&DWP($i+12,"edi"),"edx");
	&or	("ebp","edx");
    }
	&xor	("ebx","ebx");
	&mov	("eax",&DWP(32*15+8,"esp"));	# GFp_ia32cap_P copy
	&sub	("ebx","ebp");
	 &lea	("esi",&DWP($in1_z,"esp"));
	&or	("ebx","ebp");
	 &lea	("ebp",&DWP($in1_z,"esp"));
	&sar	("ebx",31);
	 &lea	("edi",&DWP($Z1sqr,"esp"));
	&mov	(&DWP(32*15+4,"esp"),"ebx");	# !in2infty

	&call	("_ecp_nistz256_mul_mont");	# p256_sqr_mont(Z1sqr, in1_z);

	&mov	("eax",&DWP(32*15+8,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($in2_x,"esp"));
	&mov	("ebp","edi");			# %esi is stull &Z1sqr
	&lea	("edi",&DWP($U2,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(U2, Z1sqr, in2_x);

	&mov	("eax",&DWP(32*15+8,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($in1_z,"esp"));
	&lea	("ebp",&DWP($Z1sqr,"esp"));
	&lea	("edi",&DWP($S2,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(S2, Z1sqr, in1_z);

	&lea	("esi",&DWP($U2,"esp"));
	&lea	("ebp",&DWP($in1_x,"esp"));
	&lea	("edi",&DWP($H,"esp"));
	&call	("_ecp_nistz256_sub");		# p256_sub(H, U2, in1_x);

	&mov	("eax",&DWP(32*15+8,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($in2_y,"esp"));
	&lea	("ebp",&DWP($S2,"esp"));
	&lea	("edi",&DWP($S2,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(S2, S2, in2_y);

	&mov	("eax",&DWP(32*15+8,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($in1_z,"esp"));
	&lea	("ebp",&DWP($H,"esp"));
	&lea	("edi",&DWP($res_z,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(res_z, H, in1_z);

	&lea	("esi",&DWP($S2,"esp"));
	&lea	("ebp",&DWP($in1_y,"esp"));
	&lea	("edi",&DWP($R,"esp"));
	&call	("_ecp_nistz256_sub");		# p256_sub(R, S2, in1_y);

	&mov	("eax",&DWP(32*15+8,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($H,"esp"));
	&lea	("ebp",&DWP($H,"esp"));
	&lea	("edi",&DWP($Hsqr,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_sqr_mont(Hsqr, H);

	&mov	("eax",&DWP(32*15+8,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($R,"esp"));
	&lea	("ebp",&DWP($R,"esp"));
	&lea	("edi",&DWP($Rsqr,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_sqr_mont(Rsqr, R);

	&mov	("eax",&DWP(32*15+8,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($in1_x,"esp"));
	&lea	("ebp",&DWP($Hsqr,"esp"));
	&lea	("edi",&DWP($U2,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(U2, in1_x, Hsqr);

	&mov	("eax",&DWP(32*15+8,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($H,"esp"));
	&lea	("ebp",&DWP($Hsqr,"esp"));
	&lea	("edi",&DWP($Hcub,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(Hcub, Hsqr, H);

	&lea	("esi",&DWP($U2,"esp"));
	&lea	("ebp",&DWP($U2,"esp"));
	&lea	("edi",&DWP($Hsqr,"esp"));
	&call	("_ecp_nistz256_add");		# p256_mul_by_2(Hsqr, U2);

	&lea	("esi",&DWP($Rsqr,"esp"));
	&lea	("ebp",&DWP($Hsqr,"esp"));
	&lea	("edi",&DWP($res_x,"esp"));
	&call	("_ecp_nistz256_sub");		# p256_sub(res_x, Rsqr, Hsqr);

	&lea	("esi",&DWP($res_x,"esp"));
	&lea	("ebp",&DWP($Hcub,"esp"));
	&lea	("edi",&DWP($res_x,"esp"));
	&call	("_ecp_nistz256_sub");		# p256_sub(res_x, res_x, Hcub);

	&lea	("esi",&DWP($U2,"esp"));
	&lea	("ebp",&DWP($res_x,"esp"));
	&lea	("edi",&DWP($res_y,"esp"));
	&call	("_ecp_nistz256_sub");		# p256_sub(res_y, U2, res_x);

	&mov	("eax",&DWP(32*15+8,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($Hcub,"esp"));
	&lea	("ebp",&DWP($in1_y,"esp"));
	&lea	("edi",&DWP($S2,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(S2, Hcub, in1_y);

	&mov	("eax",&DWP(32*15+8,"esp"));	# GFp_ia32cap_P copy
	&lea	("esi",&DWP($R,"esp"));
	&lea	("ebp",&DWP($res_y,"esp"));
	&lea	("edi",&DWP($res_y,"esp"));
	&call	("_ecp_nistz256_mul_mont");	# p256_mul_mont(res_y, res_y, R);

	&lea	("esi",&DWP($res_y,"esp"));
	&lea	("ebp",&DWP($S2,"esp"));
	&lea	("edi",&DWP($res_y,"esp"));
	&call	("_ecp_nistz256_sub");		# p256_sub(res_y, res_y, S2);

	&mov	("ebp",&DWP(32*15+0,"esp"));	# !in1infty
	&mov	("esi",&DWP(32*15+4,"esp"));	# !in2infty
	&mov	("edi",&wparam(0));
	&mov	("edx","ebp");
	&not	("ebp");
	&and	("edx","esi");
	&and	("ebp","esi");
	&not	("esi");

	########################################
	# conditional moves
    for($i=64;$i<96;$i+=4) {
	my $one=@ONE_mont[($i-64)/4];

	&mov	("eax","edx");
	&and	("eax",&DWP($res_x+$i,"esp"));
	&mov	("ebx","ebp")			if ($one && $one!=-1);
	&and	("ebx",$one)			if ($one && $one!=-1);
	&mov	("ecx","esi");
	&and	("ecx",&DWP($in1_x+$i,"esp"));
	&or	("eax",$one==-1?"ebp":"ebx")	if ($one);
	&or	("eax","ecx");
	&mov	(&DWP($i,"edi"),"eax");
    }
    for($i=0;$i<64;$i+=4) {
	&mov	("eax","edx");
	&and	("eax",&DWP($res_x+$i,"esp"));
	&mov	("ebx","ebp");
	&and	("ebx",&DWP($in2_x+$i,"esp"));
	&mov	("ecx","esi");
	&and	("ecx",&DWP($in1_x+$i,"esp"));
	&or	("eax","ebx");
	&or	("eax","ecx");
	&mov	(&DWP($i,"edi"),"eax");
    }
	&stack_pop(8*15+3);
} &function_end("GFp_nistz256_point_add_affine");

&asm_finish();

close STDOUT or die "error closing STDOUT";
