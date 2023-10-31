// Assembly code for making arm syscalls.
//
// arm syscall argument register ordering is the similar to the arm
// userspace argument register ordering except that the syscall number
// (nr) is passed in r7.
//
// nr_last.rs takes care of reordering the nr argument to the end for us,
// so we only need to move nr into r7 and take care of r4 and r5 if needed.

    .file       "arm.s"
    .arch       armv5t

    .section    .text.rustix_syscall0_nr_last,"ax",%progbits
    .p2align    4
    .weak       rustix_syscall0_nr_last
    .hidden     rustix_syscall0_nr_last
    .type       rustix_syscall0_nr_last, %function
rustix_syscall0_nr_last:
    .fnstart
    .cantunwind
    push    {r7, lr}
    mov     r7, r0
    svc     #0
    pop     {r7, pc}
    .fnend
    .size rustix_syscall0_nr_last, .-rustix_syscall0_nr_last

    .section    .text.rustix_syscall1_nr_last,"ax",%progbits
    .p2align 4
    .weak       rustix_syscall1_nr_last
    .hidden     rustix_syscall1_nr_last
    .type       rustix_syscall1_nr_last, %function
rustix_syscall1_nr_last:
    .fnstart
    .cantunwind
    push    {r7, lr}
    mov     r7, r1
    svc     #0
    pop     {r7, pc}
    .fnend
    .size rustix_syscall1_nr_last, .-rustix_syscall1_nr_last

    .section    .text.rustix_syscall1_noreturn_nr_last,"ax",%progbits
    .p2align    4
    .weak       rustix_syscall1_noreturn_nr_last
    .hidden     rustix_syscall1_noreturn_nr_last
    .type       rustix_syscall1_noreturn_nr_last, %function
rustix_syscall1_noreturn_nr_last:
    .fnstart
    .cantunwind
    // Don't save r7 and lr; this is noreturn, so we'll never restore them.
    mov     r7, r1
    svc     #0
    udf     #16 // Trap instruction
    .fnend
    .size rustix_syscall1_noreturn_nr_last, .-rustix_syscall1_noreturn_nr_last

    .section    .text.rustix_syscall2_nr_last,"ax",%progbits
    .p2align 4
    .weak       rustix_syscall2_nr_last
    .hidden     rustix_syscall2_nr_last
    .type       rustix_syscall2_nr_last, %function
rustix_syscall2_nr_last:
    .fnstart
    .cantunwind
    push    {r7, lr}
    mov     r7, r2
    svc     #0
    pop     {r7, pc}
    .fnend
    .size rustix_syscall2_nr_last, .-rustix_syscall2_nr_last

    .section    .text.rustix_syscall3_nr_last,"ax",%progbits
    .p2align 4
    .weak       rustix_syscall3_nr_last
    .hidden     rustix_syscall3_nr_last
    .type       rustix_syscall3_nr_last, %function
rustix_syscall3_nr_last:
    .fnstart
    .cantunwind
    push    {r7, lr}
    mov     r7, r3
    svc     #0
    pop     {r7, pc}
    .fnend
    .size rustix_syscall3_nr_last, .-rustix_syscall3_nr_last

    .section    .text.rustix_syscall4_nr_last,"ax",%progbits
    .p2align 4
    .weak       rustix_syscall4_nr_last
    .hidden     rustix_syscall4_nr_last
    .type       rustix_syscall4_nr_last, %function
rustix_syscall4_nr_last:
    .fnstart
    .cantunwind
    push    {r7, lr}
    ldr     r7, [sp, #8]
    svc     #0
    pop     {r7, pc}
    .fnend
    .size rustix_syscall4_nr_last, .-rustix_syscall4_nr_last

    .section    .text.rustix_syscall5_nr_last,"ax",%progbits
    .p2align 4
    .weak       rustix_syscall5_nr_last
    .hidden     rustix_syscall5_nr_last
    .type       rustix_syscall5_nr_last, %function
rustix_syscall5_nr_last:
    .fnstart
    .cantunwind
    push    {r4, r7, r11, lr}
    ldr     r7, [sp, #20]
    ldr     r4, [sp, #16]
    svc     #0
    pop     {r4, r7, r11, pc}
    .fnend
    .size rustix_syscall5_nr_last, .-rustix_syscall5_nr_last

    .section    .text.rustix_syscall6_nr_last,"ax",%progbits
    .p2align 4
    .weak       rustix_syscall6_nr_last
    .hidden     rustix_syscall6_nr_last
    .type       rustix_syscall6_nr_last, %function
rustix_syscall6_nr_last:
    .fnstart
    .cantunwind
    push    {r4, r5, r7, lr}
    add     r7, sp, #16
    ldm     r7, {r4, r5, r7}
    svc     #0
    pop     {r4, r5, r7, pc}
    .fnend
    .size rustix_syscall6_nr_last, .-rustix_syscall6_nr_last

    .section .note.GNU-stack,"",%progbits
