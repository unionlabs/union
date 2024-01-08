#!/usr/bin/env perl

# read whole file
while(<>) { push @file, $_; }

# traverse and remove auto-generated PartialEq for chosen types
for (my $i = 0; $i <= $#file; $i++) {
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
    } elsif (@file[$i] =~ m/assert_eq!\($/) {
        @file[++$i] =~ s/unsafe\s*\{\s*&\(\*\(::std::ptr::null::<(\w+)>\(\)\)\)\.(\w+).*\}/offsetof!($1, $2)/;
    }
}

print << '___';
#[cfg(test)]
macro_rules! offsetof {
    ($type:ty, $field:tt) => {
        unsafe {
            let v = std::mem::MaybeUninit::<$type>::uninit().assume_init();
            (&v.$field as *const _ as usize) - (&v as *const _ as usize)
        }
    };
}
___
# print the file
print @file;

close STDOUT;
