// Assembly code for making aarch64 syscalls.
//
// aarch64 syscall argument register ordering is the same as the aarch64
// userspace argument register ordering except that the syscall number
// (nr) is passed in w8.
//
// outline.rs takes care of reordering the nr argument to the end for us,
// so we only need to move nr into w8.
//
// arm64-ilp32 is not yet supported.

    .file       "aarch64.s"
    .arch       armv8-a

    .section    .text.rustix_syscall0_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall0_nr_last
    .hidden     rustix_syscall0_nr_last
    .type       rustix_syscall0_nr_last, @function
rustix_syscall0_nr_last:
    .cfi_startproc
    mov    w8, w0
    svc    #0
    ret
    .cfi_endproc
    .size rustix_syscall0_nr_last, .-rustix_syscall0_nr_last

    .section    .text.rustix_syscall1_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall1_nr_last
    .hidden     rustix_syscall1_nr_last
    .type       rustix_syscall1_nr_last, @function
rustix_syscall1_nr_last:
    .cfi_startproc
    mov    w8, w1
    svc    #0
    ret
    .cfi_endproc
    .size rustix_syscall1_nr_last, .-rustix_syscall1_nr_last

    .section    .text.rustix_syscall1_noreturn_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall1_noreturn_nr_last
    .hidden     rustix_syscall1_noreturn_nr_last
    .type       rustix_syscall1_noreturn_nr_last, @function
rustix_syscall1_noreturn_nr_last:
    .cfi_startproc
    mov    w8, w1
    svc    #0
    brk    #0x1
    .cfi_endproc
    .size rustix_syscall1_noreturn_nr_last, .-rustix_syscall1_noreturn_nr_last

    .section    .text.rustix_syscall2_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall2_nr_last
    .hidden     rustix_syscall2_nr_last
    .type       rustix_syscall2_nr_last, @function
rustix_syscall2_nr_last:
    .cfi_startproc
    mov    w8, w2
    svc    #0
    ret
    .cfi_endproc
    .size rustix_syscall2_nr_last, .-rustix_syscall2_nr_last

    .section    .text.rustix_syscall3_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall3_nr_last
    .hidden     rustix_syscall3_nr_last
    .type       rustix_syscall3_nr_last, @function
rustix_syscall3_nr_last:
    .cfi_startproc
    mov    w8, w3
    svc    #0
    ret
    .cfi_endproc
    .size rustix_syscall3_nr_last, .-rustix_syscall3_nr_last

    .section    .text.rustix_syscall4_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall4_nr_last
    .hidden     rustix_syscall4_nr_last
    .type       rustix_syscall4_nr_last, @function
rustix_syscall4_nr_last:
    .cfi_startproc
    mov    w8, w4
    svc    #0
    ret
    .cfi_endproc
    .size rustix_syscall4_nr_last, .-rustix_syscall4_nr_last

    .section    .text.rustix_syscall5_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall5_nr_last
    .hidden     rustix_syscall5_nr_last
    .type       rustix_syscall5_nr_last, @function
rustix_syscall5_nr_last:
    .cfi_startproc
    mov    w8, w5
    svc    #0
    ret
    .cfi_endproc
    .size rustix_syscall5_nr_last, .-rustix_syscall5_nr_last

    .section    .text.rustix_syscall6_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall6_nr_last
    .hidden     rustix_syscall6_nr_last
    .type       rustix_syscall6_nr_last, @function
rustix_syscall6_nr_last:
    .cfi_startproc
    mov    w8, w6
    svc    #0
    ret
    .cfi_endproc
    .size rustix_syscall6_nr_last, .-rustix_syscall6_nr_last

    .section .note.GNU-stack,"",@progbits
