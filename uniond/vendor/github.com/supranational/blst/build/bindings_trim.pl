#!/usr/bin/env perl

# read whole file
while(<>) { push @file, $_; }

# traverse and remove auto-generated PartialEq for chosen types
for (my $i = 0; $i <= $#file; $i++) {
    if (@file[$i] =~ m/pub\s+(?:struct|enum)\s+(\w+)/) {
        push @structs, $1;
    }

    if (@file[$i] =~ m/struct\s+blst_p[12]/) {
        @file[$i-1] =~ s/,\s*PartialEq//;
    } elsif (@file[$i] =~ m/struct\s+blst_fp12/) {
        @file[$i-1] =~ s/,\s*(?:Default|PartialEq)//g;
    } elsif (@file[$i] =~ m/struct\s+(blst_pairing|blst_uniq)/) {
        @file[$i-1] =~ s/,\s*(?:Copy|Clone|Eq|PartialEq)//g;
    } elsif (@file[$i] =~ m/struct\s+blst_scalar/) {
        @file[$i-1] =~ s/,\s*Copy//;
        @file[$i-1] =~ s/\)/, Zeroize\)/;
        splice @file, $i, 0, "#[zeroize(drop)]\n"; $i++;
    } else {
        @file[$i] =~ s/::std::/::core::/g;
    }
}

print @file;

print << '___';
#[test]
fn bindgen_test_normal_types() {
    // from "Rust for Rustaceans" by Jon Gjengset
    fn is_normal<T: Sized + Send + Sync + Unpin>() {}
___
for (@structs) {
    print "    is_normal::<$_>();\n";
}
print "}\n";

close STDOUT;
