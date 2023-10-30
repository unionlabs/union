// Assembly code for making x86-64 syscalls.
//
// x86-64 syscall argument register ordering is the same as the x86-64
// userspace argument register ordering except that a3 is passed in r10
// instead of rcx, and the syscall number (nr) is passed in eax.
//
// outline.rs takes care of reordering the nr argument to the end for us,
// so we only need to move nr into eax and move rcx into r10 as needed.
//
// x32 is not yet supported.

    .file       "x86_64.s"
    .intel_syntax noprefix

    .section    .text.rustix_syscall0_nr_last,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall0_nr_last
    .hidden     rustix_syscall0_nr_last
    .type       rustix_syscall0_nr_last, @function
rustix_syscall0_nr_last:
    .cfi_startproc
    mov    eax,edi
    syscall
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
    mov    eax,esi
    syscall
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
    mov    eax,esi
    syscall
    ud2
    .cfi_endproc
    .size rustix_syscall1_noreturn_nr_last, .-rustix_syscall1_noreturn_nr_last

    .section    .text.rustix_syscall2_nr_last,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall2_nr_last
    .hidden     rustix_syscall2_nr_last
    .type       rustix_syscall2_nr_last, @function
rustix_syscall2_nr_last:
    .cfi_startproc
    mov    eax,edx
    syscall
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
    mov    eax,ecx
    syscall
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
    mov    eax,r8d
    mov    r10,rcx
    syscall
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
    mov    eax,r9d
    mov    r10,rcx
    syscall
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
    mov    eax,DWORD PTR [rsp+0x8]
    mov    r10,rcx
    syscall
    ret
    .cfi_endproc
    .size rustix_syscall6_nr_last, .-rustix_syscall6_nr_last

    .section .note.GNU-stack,"",@progbits
