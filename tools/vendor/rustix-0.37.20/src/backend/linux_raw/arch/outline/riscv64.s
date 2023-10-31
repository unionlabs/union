# Assembly code for making riscv64 syscalls.
#
# riscv64 syscall argument register ordering is the same as the riscv64
# userspace argument register ordering except that the syscall number
# (nr) is passed in a7.
#
# nr_last.rs takes care of reordering the nr argument to the end for us,
# so we only need to move nr into a7.

    .file       "riscv64.s"

    .section    .text.rustix_syscall0_nr_last,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall0_nr_last
    .hidden     rustix_syscall0_nr_last
    .type       rustix_syscall0_nr_last, @function
rustix_syscall0_nr_last:
    .cfi_startproc
    mv      a7, a0
    ecall
    ret
    .cfi_endproc
    .size rustix_syscall0_nr_last, .-rustix_syscall0_nr_last

    .section    .text.rustix_syscall1_nr_last,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall1_nr_last
    .hidden     rustix_syscall1_nr_last
    .type       rustix_syscall1_nr_last, @function
rustix_syscall1_nr_last:
    .cfi_startproc
    mv      a7, a1
    ecall
    ret
    .cfi_endproc
    .size rustix_syscall1_nr_last, .-rustix_syscall1_nr_last

    .section    .text.rustix_syscall1_noreturn_nr_last,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall1_noreturn_nr_last
    .hidden     rustix_syscall1_noreturn_nr_last
    .type       rustix_syscall1_noreturn_nr_last, @function
rustix_syscall1_noreturn_nr_last:
    .cfi_startproc
    mv      a7, a1
    ecall
    unimp
    .cfi_endproc
    .size rustix_syscall1_noreturn_nr_last, .-rustix_syscall1_noreturn_nr_last

    .section    .text.rustix_syscall2_nr_last,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall2_nr_last
    .hidden     rustix_syscall2_nr_last
    .type       rustix_syscall2_nr_last, @function
rustix_syscall2_nr_last:
    .cfi_startproc
    mv      a7, a2
    ecall
    ret
    .cfi_endproc
    .size rustix_syscall2_nr_last, .-rustix_syscall2_nr_last

    .section    .text.rustix_syscall3_nr_last,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall3_nr_last
    .hidden     rustix_syscall3_nr_last
    .type       rustix_syscall3_nr_last, @function
rustix_syscall3_nr_last:
    .cfi_startproc
    mv      a7, a3
    ecall
    ret
    .cfi_endproc
    .size rustix_syscall3_nr_last, .-rustix_syscall3_nr_last

    .section    .text.rustix_syscall4_nr_last,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall4_nr_last
    .hidden     rustix_syscall4_nr_last
    .type       rustix_syscall4_nr_last, @function
rustix_syscall4_nr_last:
    .cfi_startproc
    mv      a7, a4
    ecall
    ret
    .cfi_endproc
    .size rustix_syscall4_nr_last, .-rustix_syscall4_nr_last

    .section    .text.rustix_syscall5_nr_last,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall5_nr_last
    .hidden     rustix_syscall5_nr_last
    .type       rustix_syscall5_nr_last, @function
rustix_syscall5_nr_last:
    .cfi_startproc
    mv      a7, a5
    ecall
    ret
    .cfi_endproc
    .size rustix_syscall5_nr_last, .-rustix_syscall5_nr_last

    .section    .text.rustix_syscall6_nr_last,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall6_nr_last
    .hidden     rustix_syscall6_nr_last
    .type       rustix_syscall6_nr_last, @function
rustix_syscall6_nr_last:
    .cfi_startproc
    mv      a7, a6
    ecall
    ret
    .cfi_endproc
    .size rustix_syscall6_nr_last, .-rustix_syscall6_nr_last

    .section .note.GNU-stack,"",@progbits
