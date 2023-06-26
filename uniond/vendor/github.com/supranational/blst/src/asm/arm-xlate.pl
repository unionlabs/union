#!/usr/bin/env perl
#
# Copyright Supranational LLC
# Licensed under the Apache License, Version 2.0, see LICENSE for details.
# SPDX-License-Identifier: Apache-2.0
#
# ARM assembler distiller/adapter by \@dot-asm.

use strict;

################################################################
# Recognized "flavour"-s are:
#
# linux[32|64]	GNU assembler, effectively pass-through
# ios[32|64]	global symbols' decorations, PIC tweaks, etc.
# win[32|64]	Visual Studio armasm-specific directives
# coff[32|64]	e.g. clang --target=arm-windows ...
#
my $flavour = shift;
   $flavour = "linux" if (!$flavour or $flavour eq "void");

my $output = shift;
open STDOUT,">$output" || die "can't open $output: $!";

my %GLOBALS;
my $dotinlocallabels = ($flavour !~ /ios/) ? 1 : 0;
my $in_proc;	# used with 'windows' flavour

################################################################
# directives which need special treatment on different platforms
################################################################
my $arch = sub { } if ($flavour !~ /linux|coff64/);# omit .arch
my $fpu  = sub { } if ($flavour !~ /linux/);       # omit .fpu

my $rodata = sub {
    SWITCH: for ($flavour) {
	/linux/		&& return ".section\t.rodata";
	/ios/		&& return ".section\t__TEXT,__const";
	/coff/		&& return ".section\t.rdata,\"dr\"";
	/win/		&& return "\tAREA\t|.rdata|,DATA,READONLY,ALIGN=8";
	last;
    }
};

my $hidden = sub {
    if ($flavour =~ /ios/)	{ ".private_extern\t".join(',',@_); }
} if ($flavour !~ /linux/);

my $comm = sub {
    my @args = split(/,\s*/,shift);
    my $name = @args[0];
    my $global = \$GLOBALS{$name};
    my $ret;

    if ($flavour =~ /ios32/)	{
	$ret = ".comm\t_$name,@args[1]\n";
	$ret .= ".non_lazy_symbol_pointer\n";
	$ret .= "$name:\n";
	$ret .= ".indirect_symbol\t_$name\n";
	$ret .= ".long\t0\n";
	$ret .= ".previous";
	$name = "_$name";
    } elsif ($flavour =~ /win/) {
	$ret = "\tCOMMON\t|$name|,@args[1]";
    } elsif ($flavour =~ /coff/) {
	$ret = ".comm\t$name,@args[1]";
    } else {
	$ret = ".comm\t".join(',',@args);
    }

    $$global = $name;
    $ret;
};

my $globl = sub {
    my $name = shift;
    my $global = \$GLOBALS{$name};
    my $ret;

    SWITCH: for ($flavour) {
	/ios/		&& do { $name = "_$name"; last; };
	/win/		&& do { $ret = ""; last; };
    }

    $ret = ".globl	$name" if (!defined($ret));
    $$global = $name;
    $ret;
};
my $global = $globl;

my $extern = sub {
    &$globl(@_);
    if ($flavour =~ /win/) {
	return "\tEXTERN\t@_";
    }
    return;	# return nothing
};

my $type = sub {
    my $arg = join(',',@_);
    my $ret;

    SWITCH: for ($flavour) {
	/ios32/		&& do { if ($arg =~ /(\w+),\s*%function/) {
				    $ret = "#ifdef __thumb2__\n" .
					   ".thumb_func	$1\n" .
					   "#endif";
				}
				last;
			      };
	/win/		&& do { if ($arg =~ /(\w+),\s*%(function|object)/) {
				    my $type = "[DATA]";
				    if ($2 eq "function") {
					$in_proc = $1;
					$type = "[FUNC]";
				    }
				    $ret = $GLOBALS{$1} ? "\tEXPORT\t|$1|$type"
							: "";
				}
				last;
			      };
	/coff/		&& do { if ($arg =~ /(\w+),\s*%function/) {
				    $ret = ".def	$1;\n".
					   ".type	32;\n".
					   ".endef";
				}
				last;
			      };
    }
    return $ret;
} if ($flavour !~ /linux/);

my $size = sub {
    if ($in_proc && $flavour =~ /win/) {
	$in_proc = undef;
	return "\tENDP";
    }
} if ($flavour !~ /linux/);

my $inst = sub {
    if ($flavour =~ /win/)	{ "\tDCDU\t".join(',',@_); }
    else			{ ".long\t".join(',',@_);  }
} if ($flavour !~ /linux/);

my $asciz = sub {
    my $line = join(",",@_);
    if ($line =~ /^"(.*)"$/)
    {	if ($flavour =~ /win/) {
	    "\tDCB\t$line,0\n\tALIGN\t4";
	} else {
	    ".byte	" . join(",",unpack("C*",$1),0) . "\n.align	2";
	}
    } else {	"";	}
};

my $align = sub {
    "\tALIGN\t".2**@_[0];
} if ($flavour =~ /win/);
   $align = sub {
    ".p2align\t".@_[0];
} if ($flavour =~ /coff/);

my $byte = sub {
    "\tDCB\t".join(',',@_);
} if ($flavour =~ /win/);

my $short = sub {
    "\tDCWU\t".join(',',@_);
} if ($flavour =~ /win/);

my $word = sub {
    "\tDCDU\t".join(',',@_);
} if ($flavour =~ /win/);

my $long = $word if ($flavour =~ /win/);

my $quad = sub {
    "\tDCQU\t".join(',',@_);
} if ($flavour =~ /win/);

my $skip = sub {
    "\tSPACE\t".shift;
} if ($flavour =~ /win/);

my $code = sub {
    "\tCODE@_[0]";
} if ($flavour =~ /win/);

my $thumb = sub {	# .thumb should appear prior .text in source
    "# define ARM THUMB\n" .
    "\tTHUMB";
} if ($flavour =~ /win/);

my $text = sub {
    "\tAREA\t|.text|,CODE,ALIGN=8,".($flavour =~ /64/ ? "ARM64" : "ARM");
} if ($flavour =~ /win/);

my $syntax = sub {} if ($flavour =~ /win/);	# omit .syntax

my $rva = sub {
    # .rva directive comes in handy only on 32-bit Windows, i.e. it can
    # be used only in '#if defined(_WIN32) && !defined(_WIN64)' sections.
    # However! Corresponding compilers don't seem to bet on PIC, which
    # raises the question why would assembler programmer have to jump
    # through the hoops? But just in case, it would go as following:
    #
    #	ldr	r1,.LOPENSSL_armcap
    #	ldr	r2,.LOPENSSL_armcap+4
    #	adr	r0,.LOPENSSL_armcap
    #	bic	r1,r1,#1		; de-thumb-ify link.exe's ideas
    #	sub	r0,r0,r1		; r0 is image base now
    #	ldr	r0,[r0,r2]
    #	...
    #.LOPENSSL_armcap:
    #	.rva	.LOPENSSL_armcap	; self-reference
    #	.rva	OPENSSL_armcap_P	; real target
    #
    # Non-position-independent [and ISA-neutral] alternative is so much
    # simpler:
    #
    #	ldr	r0,.LOPENSSL_armcap
    #	ldr	r0,[r0]
    #	...
    #.LOPENSSL_armcap:
    #	.long	OPENSSL_armcap_P
    #
    "\tDCDU\t@_[0]\n\tRELOC\t2"
} if ($flavour =~ /win(?!64)/);

################################################################
# some broken instructions in Visual Studio armasm[64]...

my $it = sub {} if ($flavour =~ /win32/);	# omit 'it'

my $ext = sub {
    "\text8\t".join(',',@_);
} if ($flavour =~ /win64/);

my $csel = sub {
    my ($args,$comment) = split(m|\s*//|,shift);
    my @regs = split(m|,\s*|,$args);
    my $cond = pop(@regs);

    "\tcsel$cond\t".join(',',@regs);
} if ($flavour =~ /win64/);

my $csetm = sub {
    my ($args,$comment) = split(m|\s*//|,shift);
    my @regs = split(m|,\s*|,$args);
    my $cond = pop(@regs);

    "\tcsetm$cond\t".join(',',@regs);
} if ($flavour =~ /win64/);

# ... then conditional branch instructions are also broken, but
# maintaining all the variants is tedious, so I kludge-fix it
# elsewhere...
################################################################
my $adrp = sub {
    my ($args,$comment) = split(m|\s*//|,shift);
    "\tadrp\t$args\@PAGE";
} if ($flavour =~ /ios64/);

my $paciasp = sub {
    ($flavour =~ /linux/) ? "\t.inst\t0xd503233f"
                          : &$inst(0xd503233f);
};

my $autiasp = sub {
    ($flavour =~ /linux/) ? "\t.inst\t0xd50323bf"
                          : &$inst(0xd50323bf);
};

sub range {
  my ($r,$sfx,$start,$end) = @_;

    join(",",map("$r$_$sfx",($start..$end)));
}

sub expand_line {
  my $line = shift;
  my @ret = ();

    pos($line)=0;

    while ($line =~ m/\G[^@\/\{\"]*/g) {
	if ($line =~ m/\G(@|\/\/|$)/gc) {
	    last;
	}
	elsif ($line =~ m/\G\{/gc) {
	    my $saved_pos = pos($line);
	    $line =~ s/\G([rdqv])([0-9]+)([^\-]*)\-\1([0-9]+)\3/range($1,$3,$2,$4)/e;
	    pos($line) = $saved_pos;
	    $line =~ m/\G[^\}]*\}/g;
	}
	elsif ($line =~ m/\G\"/gc) {
	    $line =~ m/\G[^\"]*\"/g;
	}
    }

    $line =~ s/\b(\w+)/$GLOBALS{$1} or $1/ge;

    if ($flavour =~ /win/) {
	# adjust alignment hints, "[rN,:32]" -> "[rN@32]"
	$line =~ s/(\[\s*(?:r[0-9]+|sp))\s*,?\s*:([0-9]+\s*\])/$1\@$2/;
	# adjust local labels, ".Lwhatever" -> "|$Lwhatever|"
	$line =~ s/\.(L\w{2,})/|\$$1|/g;
	# omit "#:lo12:" on win64
	$line =~ s/#:lo12://;
    } elsif ($flavour =~ /coff(?!64)/) {
	$line =~ s/\.L(\w{2,})/(\$ML$1)/g;
    } elsif ($flavour =~ /ios64/) {
	$line =~ s/#:lo12:(\w+)/$1\@PAGEOFF/;
    }

    return $line;
}

while(my $line=<>) {

    # fix up assembler-specific commentary delimiter
    $line =~ s/@(?=[\s@])/\;/g if ($flavour =~ /win|coff/);

    if ($line =~ m/^\s*(#|@|;|\/\/)/)	{ print $line; next; }

    $line =~ s|/\*.*\*/||;	# get rid of C-style comments...
    $line =~ s|^\s+||;		# ... and skip white spaces in beginning...
    $line =~ s|\s+$||;		# ... and at the end

    {
	$line =~ s|[\b\.]L(\w{2,})|L$1|g;	# common denominator for Locallabel
	$line =~ s|\bL(\w{2,})|\.L$1|g	if ($dotinlocallabels);
    }

    {
	$line =~ s|(^[\.\w]+)\:\s*||;
	my $label = $1;
	if ($label) {
	    $label = ($GLOBALS{$label} or $label);
	    if ($flavour =~ /win/) {
		$label =~ s|^\.L(?=\w)|\$L|;
		printf "|%s|%s", $label, ($label eq $in_proc ? " PROC" : "");
	    } else {
		$label =~ s|^\.L(?=\w)|\$ML| if ($flavour =~ /coff(?!64)/);
		printf "%s:", $label;
	    }
	}
    }

    if ($line !~ m/^[#@;]/) {
	$line =~ s|^\s*(\.?)(\S+)\s*||;
	my $c = $1; $c = "\t" if ($c eq "");
	my $mnemonic = $2;
	my $opcode;
	if ($mnemonic =~ m/([^\.]+)\.([^\.]+)/) {
	    $opcode = eval("\$$1_$2");
	} else {
	    $opcode = eval("\$$mnemonic");
	}

	my $arg=expand_line($line);

	if (ref($opcode) eq 'CODE') {
	    $line = &$opcode($arg);
	} elsif ($mnemonic)         {
	    if ($flavour =~ /win64/) {
		# "b.cond" -> "bcond", kludge-fix:-(
		$mnemonic =~ s/^b\.([a-z]{2}$)/b$1/;
	    }
	    $line = $c.$mnemonic;
	    $line.= "\t$arg" if ($arg ne "");
	}
    }

    print $line if ($line);
    print "\n";
}

print "\tEND\n" if ($flavour =~ /win/);

close STDOUT;
