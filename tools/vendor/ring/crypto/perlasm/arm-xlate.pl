#! /usr/bin/env perl
# Copyright 2015-2016 The OpenSSL Project Authors. All Rights Reserved.
#
# Licensed under the OpenSSL license (the "License").  You may not use
# this file except in compliance with the License.  You can obtain a copy
# in the file LICENSE in the source distribution or at
# https://www.openssl.org/source/license.html

use strict;

my $flavour = shift;
my $output = shift;
open STDOUT,">$output" || die "can't open $output: $!";

$flavour = "linux32" if (!$flavour or $flavour eq "void");

my %GLOBALS;
my $dotinlocallabels=($flavour=~/linux/)?1:0;

################################################################
# directives which need special treatment on different platforms
################################################################
my $arch = sub {
    if ($flavour =~ /linux/)	{ ".arch\t".join(',',@_); }
    else			{ ""; }
};
my $fpu = sub {
    if ($flavour =~ /linux/)	{ ".fpu\t".join(',',@_); }
    else			{ ""; }
};
my $hidden = sub {
    if ($flavour =~ /ios/)	{ ".private_extern\t".join(',',@_); }
    else			{ ".hidden\t".join(',',@_); }
};
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
	$ret .= ".long\t0";
	$name = "_$name";
    } else			{ $ret = ".comm\t".join(',',@args); }

    $$global = $name;
    $ret;
};
my $globl = sub {
    my $name = shift;
    my $global = \$GLOBALS{$name};
    my $ret;

    SWITCH: for ($flavour) {
	/ios/		&& do { $name = "_$name";
				last;
			      };
    }

    $ret = ".globl	$name\n";
    # All symbols in assembly files are hidden.
    $ret .= &$hidden($name);
    $$global = $name;
    $ret;
};
my $global = $globl;
my $extern = sub {
    &$globl(@_);
    return;	# return nothing
};
my $type = sub {
    if ($flavour =~ /linux/)	{ ".type\t".join(',',@_); }
    elsif ($flavour =~ /ios32/)	{ if (join(',',@_) =~ /(\w+),%function/) {
					"#ifdef __thumb2__\n".
					".thumb_func	$1\n".
					"#endif";
				  }
			        }
    else			{ ""; }
};
my $size = sub {
    if ($flavour =~ /linux/)	{ ".size\t".join(',',@_); }
    else			{ ""; }
};
my $inst = sub {
    if ($flavour =~ /linux/)    { ".inst\t".join(',',@_); }
    else                        { ".long\t".join(',',@_); }
};
my $asciz = sub {
    my $line = join(",",@_);
    if ($line =~ /^"(.*)"$/)
    {	".byte	" . join(",",unpack("C*",$1),0) . "\n.align	2";	}
    else
    {	"";	}
};
my $section = sub {
    if ($flavour =~ /ios/) {
        if ($_[0] eq ".rodata") {
            return ".section\t__TEXT,__const";
        }
        die "Unknown section name $_[0]";
    } else {
        return ".section\t" . join(",", @_);
    }
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

    return $line;
}

print <<___;
// This file is generated from a similarly-named Perl script in the BoringSSL
// source tree. Do not edit by hand.

#if !defined(__has_feature)
#define __has_feature(x) 0
#endif
#if __has_feature(memory_sanitizer) && !defined(OPENSSL_NO_ASM)
#define OPENSSL_NO_ASM
#endif

#if !defined(OPENSSL_NO_ASM)
___

print "#if defined(__arm__)\n" if ($flavour eq "linux32");
print "#if defined(__aarch64__)\n" if ($flavour eq "linux64");

while(my $line=<>) {

    if ($line =~ m/^\s*(#|@|\/\/)/)	{ print $line; next; }

    $line =~ s|/\*.*\*/||;	# get rid of C-style comments...
    $line =~ s|^\s+||;		# ... and skip white spaces in beginning...
    $line =~ s|\s+$||;		# ... and at the end

    if ($flavour =~ /64/) {
	my $copy = $line;
	# Also remove line comments.
	$copy =~ s|//.*||;
	if ($copy =~ /\b[wx]18\b/) {
	    die "r18 is reserved by the platform and may not be used.";
	}
    }

    {
	$line =~ s|[\b\.]L(\w{2,})|L$1|g;	# common denominator for Locallabel
	$line =~ s|\bL(\w{2,})|\.L$1|g	if ($dotinlocallabels);
    }

    {
	$line =~ s|(^[\.\w]+)\:\s*||;
	my $label = $1;
	if ($label) {
	    printf "%s:",($GLOBALS{$label} or $label);
	}
    }

    if ($line !~ m/^[#@]/) {
	$line =~ s|^\s*(\.?)(\S+)\s*||;
	my $c = $1; $c = "\t" if ($c eq "");
	my $mnemonic = $2;
	my $opcode;
	if ($mnemonic =~ m/([^\.]+)\.([^\.]+)/) {
	    $opcode = eval("\$$1_$2");
	} else {
	    $opcode = eval("\$$mnemonic");
	}

	if ($flavour =~ /ios/) {
	    # Mach-O and ELF use different syntax for these relocations. Note
	    # that we require :pg_hi21: to be explicitly listed. It is normally
	    # optional with adrp instructions.
	    $line =~ s|:pg_hi21:(\w+)|\1\@PAGE|;
	    $line =~ s|:lo12:(\w+)|\1\@PAGEOFF|;
	} else {
	    # Clang's integrated assembly does not support the optional
	    # :pg_hi21: markers, so erase them.
	    $line =~ s|:pg_hi21:||;
	}

	my $arg=expand_line($line);

	if (ref($opcode) eq 'CODE') {
		$line = &$opcode($arg);
	} elsif ($mnemonic)         {
		$line = $c.$mnemonic;
		$line.= "\t$arg" if ($arg ne "");
	}
    }

    print $line if ($line);
    print "\n";
}

print "#endif\n" if ($flavour eq "linux32" || $flavour eq "linux64");
print "#endif  // !OPENSSL_NO_ASM\n";

# See https://www.airs.com/blog/archives/518.
print ".section\t.note.GNU-stack,\"\",\%progbits\n" if ($flavour =~ /linux/);

close STDOUT;
