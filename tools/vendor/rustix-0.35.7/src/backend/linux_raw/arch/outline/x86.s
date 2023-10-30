// Assembly code for making x86 syscalls.
//
// On x86 we use the "fastcall" convention which passes the first two
// arguments in ecx and edx. Outline.rs reorders the arguments to put
// a1 and a2 in those registers so they we don't have to move them to
// set up the kernel convention.
//
// "fastcall" expects callee to pop argument stack space, so we use
// `ret imm` instructions to clean up the stack. We don't need callee
// cleanup per se, it just comes along with using "fastcall".

    .file       "x86.s"
    .intel_syntax noprefix

    .section    .text.rustix_indirect_syscall0_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_indirect_syscall0_nr_last_fastcall
    .hidden     rustix_indirect_syscall0_nr_last_fastcall
    .type       rustix_indirect_syscall0_nr_last_fastcall, @function
rustix_indirect_syscall0_nr_last_fastcall:
    .cfi_startproc
    mov    eax,ecx
    call   edx
    ret
    .cfi_endproc
    .size rustix_indirect_syscall0_nr_last_fastcall, .-rustix_indirect_syscall0_nr_last_fastcall

    .section    .text.rustix_indirect_syscall1_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_indirect_syscall1_nr_last_fastcall
    .hidden     rustix_indirect_syscall1_nr_last_fastcall
    .type       rustix_indirect_syscall1_nr_last_fastcall, @function
rustix_indirect_syscall1_nr_last_fastcall:
    .cfi_startproc
    push   ebx
    .cfi_def_cfa_offset 8
    .cfi_offset ebx, -8
    mov    ebx,ecx
    mov    eax,edx
    call   DWORD PTR [esp+0x8]
    pop    ebx
    .cfi_def_cfa_offset 4
    ret    0x4
    .cfi_endproc
    .size rustix_indirect_syscall1_nr_last_fastcall, .-rustix_indirect_syscall1_nr_last_fastcall

    .section    .text.rustix_indirect_syscall1_noreturn_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_indirect_syscall1_noreturn_nr_last_fastcall
    .hidden     rustix_indirect_syscall1_noreturn_nr_last_fastcall
    .type       rustix_indirect_syscall1_noreturn_nr_last_fastcall, @function
rustix_indirect_syscall1_noreturn_nr_last_fastcall:
    .cfi_startproc
    mov    ebx,ecx
    mov    eax,edx
    call   DWORD PTR [esp+0x4]
    ud2
    .cfi_endproc
    .size rustix_indirect_syscall1_noreturn_nr_last_fastcall, .-rustix_indirect_syscall1_noreturn_nr_last_fastcall

    .section    .text.rustix_indirect_syscall2_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_indirect_syscall2_nr_last_fastcall
    .hidden     rustix_indirect_syscall2_nr_last_fastcall
    .type       rustix_indirect_syscall2_nr_last_fastcall, @function
rustix_indirect_syscall2_nr_last_fastcall:
    .cfi_startproc
    push   ebx
    .cfi_def_cfa_offset 8
    .cfi_offset ebx, -8
    mov    ebx,edx
    mov    eax,DWORD PTR [esp+0x8]
    call   DWORD PTR [esp+0xc]
    pop    ebx
    .cfi_def_cfa_offset 4
    ret    0x8
    .cfi_endproc
    .size rustix_indirect_syscall2_nr_last_fastcall, .-rustix_indirect_syscall2_nr_last_fastcall

    .section    .text.rustix_indirect_syscall3_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_indirect_syscall3_nr_last_fastcall
    .hidden     rustix_indirect_syscall3_nr_last_fastcall
    .type       rustix_indirect_syscall3_nr_last_fastcall, @function
rustix_indirect_syscall3_nr_last_fastcall:
    .cfi_startproc
    push   ebx
    .cfi_def_cfa_offset 8
    .cfi_offset ebx, -8
    mov    ebx,DWORD PTR [esp+0x8]
    mov    eax,DWORD PTR [esp+0xc]
    call   DWORD PTR [esp+0x10]
    pop    ebx
    .cfi_def_cfa_offset 4
    ret    0xc
    .cfi_endproc
    .size rustix_indirect_syscall3_nr_last_fastcall, .-rustix_indirect_syscall3_nr_last_fastcall

    .section    .text.rustix_indirect_syscall4_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_indirect_syscall4_nr_last_fastcall
    .hidden     rustix_indirect_syscall4_nr_last_fastcall
    .type       rustix_indirect_syscall4_nr_last_fastcall, @function
rustix_indirect_syscall4_nr_last_fastcall:
    .cfi_startproc
    push   ebx
    .cfi_def_cfa_offset 8
    push   esi
    .cfi_def_cfa_offset 12
    .cfi_offset esi, -12
    .cfi_offset ebx, -8
    mov    ebx,DWORD PTR [esp+0xc]
    mov    esi,DWORD PTR [esp+0x10]
    mov    eax,DWORD PTR [esp+0x14]
    call   DWORD PTR [esp+0x18]
    pop    esi
    .cfi_def_cfa_offset 8
    pop    ebx
    .cfi_def_cfa_offset 4
    ret    0x10
    .cfi_endproc
    .size rustix_indirect_syscall4_nr_last_fastcall, .-rustix_indirect_syscall4_nr_last_fastcall

    .section    .text.rustix_indirect_syscall5_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_indirect_syscall5_nr_last_fastcall
    .hidden     rustix_indirect_syscall5_nr_last_fastcall
    .type       rustix_indirect_syscall5_nr_last_fastcall, @function
rustix_indirect_syscall5_nr_last_fastcall:
    .cfi_startproc
    push   ebx
    .cfi_def_cfa_offset 8
    push   esi
    .cfi_def_cfa_offset 12
    push   edi
    .cfi_def_cfa_offset 16
    .cfi_offset edi, -16
    .cfi_offset esi, -12
    .cfi_offset ebx, -8
    mov    ebx,DWORD PTR [esp+0x10]
    mov    esi,DWORD PTR [esp+0x14]
    mov    edi,DWORD PTR [esp+0x18]
    mov    eax,DWORD PTR [esp+0x1c]
    call   DWORD PTR [esp+0x20]
    pop    edi
    .cfi_def_cfa_offset 12
    pop    esi
    .cfi_def_cfa_offset 8
    pop    ebx
    .cfi_def_cfa_offset 4
    ret    0x14
    .cfi_endproc
    .size rustix_indirect_syscall5_nr_last_fastcall, .-rustix_indirect_syscall5_nr_last_fastcall

    .section    .text.rustix_indirect_syscall6_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_indirect_syscall6_nr_last_fastcall
    .hidden     rustix_indirect_syscall6_nr_last_fastcall
    .type       rustix_indirect_syscall6_nr_last_fastcall, @function
rustix_indirect_syscall6_nr_last_fastcall:
    .cfi_startproc
    push   ebx
    .cfi_def_cfa_offset 8
    push   esi
    .cfi_def_cfa_offset 12
    push   edi
    .cfi_def_cfa_offset 16
    push   ebp
    .cfi_def_cfa_offset 20
    .cfi_offset ebp, -20
    .cfi_offset edi, -16
    .cfi_offset esi, -12
    .cfi_offset ebx, -8
    mov    ebx,DWORD PTR [esp+0x14]
    mov    esi,DWORD PTR [esp+0x18]
    mov    edi,DWORD PTR [esp+0x1c]
    mov    ebp,DWORD PTR [esp+0x20]
    mov    eax,DWORD PTR [esp+0x24]
    call   DWORD PTR [esp+0x28]
    pop    ebp
    .cfi_def_cfa_offset 16
    pop    edi
    .cfi_def_cfa_offset 12
    pop    esi
    .cfi_def_cfa_offset 8
    pop    ebx
    .cfi_def_cfa_offset 4
    ret    0x18
    .cfi_endproc
    .size rustix_indirect_syscall6_nr_last_fastcall, .-rustix_indirect_syscall6_nr_last_fastcall

    .section    .text.rustix_syscall0_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall0_nr_last_fastcall
    .hidden     rustix_syscall0_nr_last_fastcall
    .type       rustix_syscall0_nr_last_fastcall, @function
rustix_syscall0_nr_last_fastcall:
    .cfi_startproc
    mov    eax,ecx
    int    0x80
    ret
    .cfi_endproc
    .size rustix_syscall0_nr_last_fastcall, .-rustix_syscall0_nr_last_fastcall

    .section    .text.rustix_syscall1_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall1_nr_last_fastcall
    .hidden     rustix_syscall1_nr_last_fastcall
    .type       rustix_syscall1_nr_last_fastcall, @function
rustix_syscall1_nr_last_fastcall:
    .cfi_startproc
    push   ebx
    .cfi_def_cfa_offset 8
    .cfi_offset ebx, -8
    mov    eax,edx
    mov    ebx,ecx
    int    0x80
    pop    ebx
    .cfi_def_cfa_offset 4
    ret
    .cfi_endproc
    .size rustix_syscall1_nr_last_fastcall, .-rustix_syscall1_nr_last_fastcall

    .section    .text.rustix_syscall1_noreturn_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall1_noreturn_nr_last_fastcall
    .hidden     rustix_syscall1_noreturn_nr_last_fastcall
    .type       rustix_syscall1_noreturn_nr_last_fastcall, @function
rustix_syscall1_noreturn_nr_last_fastcall:
    .cfi_startproc
    mov    eax,edx
    mov    ebx,ecx
    int    0x80
    ud2
    .cfi_endproc
    .size rustix_syscall1_noreturn_nr_last_fastcall, .-rustix_syscall1_noreturn_nr_last_fastcall

    .section    .text.rustix_syscall2_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall2_nr_last_fastcall
    .hidden     rustix_syscall2_nr_last_fastcall
    .type       rustix_syscall2_nr_last_fastcall, @function
rustix_syscall2_nr_last_fastcall:
    .cfi_startproc
    push   ebx
    .cfi_def_cfa_offset 8
    .cfi_offset ebx, -8
    mov    ebx,edx
    mov    eax,DWORD PTR [esp+0x8]
    int    0x80
    pop    ebx
    .cfi_def_cfa_offset 4
    ret    0x4
    .cfi_endproc
    .size rustix_syscall2_nr_last_fastcall, .-rustix_syscall2_nr_last_fastcall

    .section    .text.rustix_syscall3_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall3_nr_last_fastcall
    .hidden     rustix_syscall3_nr_last_fastcall
    .type       rustix_syscall3_nr_last_fastcall, @function
rustix_syscall3_nr_last_fastcall:
    .cfi_startproc
    push   ebx
    .cfi_def_cfa_offset 8
    .cfi_offset ebx, -8
    mov    ebx,DWORD PTR [esp+0x8]
    mov    eax,DWORD PTR [esp+0xc]
    int    0x80
    pop    ebx
    .cfi_def_cfa_offset 4
    ret    0x8
    .cfi_endproc
    .size rustix_syscall3_nr_last_fastcall, .-rustix_syscall3_nr_last_fastcall

    .section    .text.rustix_syscall4_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall4_nr_last_fastcall
    .hidden     rustix_syscall4_nr_last_fastcall
    .type       rustix_syscall4_nr_last_fastcall, @function
rustix_syscall4_nr_last_fastcall:
    .cfi_startproc
    push   ebx
    .cfi_def_cfa_offset 8
    push   esi
    .cfi_def_cfa_offset 12
    .cfi_offset esi, -12
    .cfi_offset ebx, -8
    mov    ebx,DWORD PTR [esp+0xc]
    mov    esi,DWORD PTR [esp+0x10]
    mov    eax,DWORD PTR [esp+0x14]
    int    0x80
    pop    esi
    .cfi_def_cfa_offset 8
    pop    ebx
    .cfi_def_cfa_offset 4
    ret    0xc
    .cfi_endproc
    .size rustix_syscall4_nr_last_fastcall, .-rustix_syscall4_nr_last_fastcall

    .section    .text.rustix_syscall5_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall5_nr_last_fastcall
    .hidden     rustix_syscall5_nr_last_fastcall
    .type       rustix_syscall5_nr_last_fastcall, @function
rustix_syscall5_nr_last_fastcall:
    .cfi_startproc
    push   ebx
    .cfi_def_cfa_offset 8
    push   edi
    .cfi_def_cfa_offset 12
    push   esi
    .cfi_def_cfa_offset 16
    .cfi_offset esi, -16
    .cfi_offset edi, -12
    .cfi_offset ebx, -8
    mov    ebx,DWORD PTR [esp+0x10]
    mov    esi,DWORD PTR [esp+0x14]
    mov    edi,DWORD PTR [esp+0x18]
    mov    eax,DWORD PTR [esp+0x1c]
    int    0x80
    pop    esi
    .cfi_def_cfa_offset 12
    pop    edi
    .cfi_def_cfa_offset 8
    pop    ebx
    .cfi_def_cfa_offset 4
    ret    0x10
    .cfi_endproc
    .size rustix_syscall5_nr_last_fastcall, .-rustix_syscall5_nr_last_fastcall

    .section    .text.rustix_syscall6_nr_last_fastcall,"ax",@progbits
    .p2align    4
    .weak       rustix_syscall6_nr_last_fastcall
    .hidden     rustix_syscall6_nr_last_fastcall
    .type       rustix_syscall6_nr_last_fastcall, @function
rustix_syscall6_nr_last_fastcall:
    .cfi_startproc
    push   ebp
    .cfi_def_cfa_offset 8
    push   ebx
    .cfi_def_cfa_offset 12
    push   edi
    .cfi_def_cfa_offset 16
    push   esi
    .cfi_def_cfa_offset 20
    .cfi_offset esi, -20
    .cfi_offset edi, -16
    .cfi_offset ebx, -12
    .cfi_offset ebp, -8
    mov    ebx,DWORD PTR [esp+0x14]
    mov    esi,DWORD PTR [esp+0x18]
    mov    edi,DWORD PTR [esp+0x1c]
    mov    ebp,DWORD PTR [esp+0x20]
    mov    eax,DWORD PTR [esp+0x24]
    int    0x80
    pop    esi
    .cfi_def_cfa_offset 16
    pop    edi
    .cfi_def_cfa_offset 12
    pop    ebx
    .cfi_def_cfa_offset 8
    pop    ebp
    .cfi_def_cfa_offset 4
    ret    0x14
    .cfi_endproc
    .size rustix_syscall6_nr_last_fastcall, .-rustix_syscall6_nr_last_fastcall

    .section    .text.rustix_int_0x80,"ax",@progbits
    .p2align    4
    .weak       rustix_int_0x80
    .hidden     rustix_int_0x80
    .type       rustix_int_0x80, @function
rustix_int_0x80:
    .cfi_startproc
    int    0x80
    ret
    .cfi_endproc
    .size rustix_int_0x80, .-rustix_int_0x80

    .section .note.GNU-stack,"",@progbits
