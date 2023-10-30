# Assembly code for making powerpc64le syscalls.
#
# powerpc64le syscall argument register ordering is the same as the
# powerpc64le userspace argument register ordering except that the syscall
# number (nr) is passed in r0.
#
# outline.rs takes care of reordering the nr argument to the end for us,
# so we only need to move nr into r0.

    .file       "powerpc64le.s"
    .machine    power8
    .abiversion 2

    .section    .text.rustix_syscall0_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall0_nr_last
    .hidden     rustix_syscall0_nr_last
    .type       rustix_syscall0_nr_last, @function
rustix_syscall0_nr_last:
    .cfi_startproc
    mr     0, 3
    sc
    bnslr
    neg 3, 3
    blr
    .cfi_endproc
    .size rustix_syscall0_nr_last, .-rustix_syscall0_nr_last

    .section    .text.rustix_syscall1_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall1_nr_last
    .hidden     rustix_syscall1_nr_last
    .type       rustix_syscall1_nr_last, @function
rustix_syscall1_nr_last:
    .cfi_startproc
    mr     0, 4
    sc
    bnslr
    neg 3, 3
    blr
    .cfi_endproc
    .size rustix_syscall1_nr_last, .-rustix_syscall1_nr_last

    .section    .text.rustix_syscall1_noreturn_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall1_noreturn_nr_last
    .hidden     rustix_syscall1_noreturn_nr_last
    .type       rustix_syscall1_noreturn_nr_last, @function
rustix_syscall1_noreturn_nr_last:
    .cfi_startproc
    mr     0, 4
    sc
    trap
    .cfi_endproc
    .size rustix_syscall1_noreturn_nr_last, .-rustix_syscall1_noreturn_nr_last

    .section    .text.rustix_syscall2_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall2_nr_last
    .hidden     rustix_syscall2_nr_last
    .type       rustix_syscall2_nr_last, @function
rustix_syscall2_nr_last:
    .cfi_startproc
    mr     0, 5
    sc
    bnslr
    neg 3, 3
    blr
    .cfi_endproc
    .size rustix_syscall2_nr_last, .-rustix_syscall2_nr_last

    .section    .text.rustix_syscall3_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall3_nr_last
    .hidden     rustix_syscall3_nr_last
    .type       rustix_syscall3_nr_last, @function
rustix_syscall3_nr_last:
    .cfi_startproc
    mr     0, 6
    sc
    bnslr
    neg 3, 3
    blr
    .cfi_endproc
    .size rustix_syscall3_nr_last, .-rustix_syscall3_nr_last

    .section    .text.rustix_syscall4_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall4_nr_last
    .hidden     rustix_syscall4_nr_last
    .type       rustix_syscall4_nr_last, @function
rustix_syscall4_nr_last:
    .cfi_startproc
    mr     0, 7
    sc
    bnslr
    neg 3, 3
    blr
    .cfi_endproc
    .size rustix_syscall4_nr_last, .-rustix_syscall4_nr_last

    .section    .text.rustix_syscall5_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall5_nr_last
    .hidden     rustix_syscall5_nr_last
    .type       rustix_syscall5_nr_last, @function
rustix_syscall5_nr_last:
    .cfi_startproc
    mr     0, 8
    sc
    bnslr
    neg 3, 3
    blr
    .cfi_endproc
    .size rustix_syscall5_nr_last, .-rustix_syscall5_nr_last

    .section    .text.rustix_syscall6_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall6_nr_last
    .hidden     rustix_syscall6_nr_last
    .type       rustix_syscall6_nr_last, @function
rustix_syscall6_nr_last:
    .cfi_startproc
    mr     0, 9
    sc
    bnslr
    neg 3, 3
    blr
    .cfi_endproc
    .size rustix_syscall6_nr_last, .-rustix_syscall6_nr_last

    .section .note.GNU-stack,"",@progbits
