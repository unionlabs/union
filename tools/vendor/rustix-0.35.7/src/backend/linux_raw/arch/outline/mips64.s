# Assembly code for making mips64 syscalls.
#
# mips64 syscall argument register ordering is the same as the mips64
# userspace argument register ordering except that the syscall number
# (nr) is passed in v0.
#
# outline.rs takes care of reordering the nr argument to the end for us,
# so we only need to move nr into v0.

    .file       "mips.s"
    .section    .mdebug.abi64
    .previous
    .abicalls

    .section    .text.rustix_syscall0_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall0_nr_last
    .hidden     rustix_syscall0_nr_last
    .type       rustix_syscall0_nr_last, @function
    .set        nomips16
    .set        nomicromips
    .ent        rustix_syscall0_nr_last
rustix_syscall0_nr_last:
    .frame      $sp,0,$31
    .mask       0x00000000,0
    .fmask      0x00000000,0
    .set        noreorder
    .set        nomacro
    move   $2, $4
    syscall
    dnegu  $12, $2
    jr     $31
    movn   $2, $12, $7
    .end        rustix_syscall0_nr_last
    .size       rustix_syscall0_nr_last, .-rustix_syscall0_nr_last

    .section    .text.rustix_syscall1_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall1_nr_last
    .hidden     rustix_syscall1_nr_last
    .type       rustix_syscall1_nr_last, @function
    .set        nomips16
    .set        nomicromips
    .ent        rustix_syscall1_nr_last
rustix_syscall1_nr_last:
    .frame      $sp,0,$31
    .mask       0x00000000,0
    .fmask      0x00000000,0
    .set        noreorder
    .set        nomacro
    move   $2, $5
    syscall
    dnegu  $12, $2
    jr     $31
    movn   $2, $12, $7
    .end        rustix_syscall1_nr_last
    .size       rustix_syscall1_nr_last, .-rustix_syscall1_nr_last

    .section    .text.rustix_syscall1_noreturn_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall1_noreturn_nr_last
    .hidden     rustix_syscall1_noreturn_nr_last
    .type       rustix_syscall1_noreturn_nr_last, @function
    .set        nomips16
    .set        nomicromips
    .ent        rustix_syscall1_noreturn_nr_last
rustix_syscall1_noreturn_nr_last:
    .frame      $sp,0,$31
    .mask       0x00000000,0
    .fmask      0x00000000,0
    .set        noreorder
    .set        nomacro
    move   $2, $5
    syscall
    teq    $0, $0
    .end        rustix_syscall1_noreturn_nr_last
    .size       rustix_syscall1_noreturn_nr_last, .-rustix_syscall1_noreturn_nr_last

    .section    .text.rustix_syscall2_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall2_nr_last
    .hidden     rustix_syscall2_nr_last
    .type       rustix_syscall2_nr_last, @function
    .set        nomips16
    .set        nomicromips
    .ent        rustix_syscall2_nr_last
rustix_syscall2_nr_last:
    .frame      $sp,0,$31
    .mask       0x00000000,0
    .fmask      0x00000000,0
    .set        noreorder
    .set        nomacro
    move   $2, $6
    syscall
    dnegu  $12, $2
    jr     $31
    movn   $2, $12, $7
    .end        rustix_syscall2_nr_last
    .size       rustix_syscall2_nr_last, .-rustix_syscall2_nr_last

    .section    .text.rustix_syscall3_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall3_nr_last
    .hidden     rustix_syscall3_nr_last
    .type       rustix_syscall3_nr_last, @function
    .set        nomips16
    .set        nomicromips
    .ent        rustix_syscall3_nr_last
rustix_syscall3_nr_last:
    .frame      $sp,0,$31
    .mask       0x00000000,0
    .fmask      0x00000000,0
    .set        noreorder
    .set        nomacro
    move   $2, $7
    syscall
    dnegu  $12, $2
    jr     $31
    movn   $2, $12, $7
    .end        rustix_syscall3_nr_last
    .size       rustix_syscall3_nr_last, .-rustix_syscall3_nr_last

    .section    .text.rustix_syscall4_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall4_nr_last
    .hidden     rustix_syscall4_nr_last
    .type       rustix_syscall4_nr_last, @function
    .set        nomips16
    .set        nomicromips
    .ent        rustix_syscall4_nr_last
rustix_syscall4_nr_last:
    .frame      $sp,0,$31
    .mask       0x00000000,0
    .fmask      0x00000000,0
    .set        noreorder
    .set        nomacro
    move   $2, $8
    syscall
    dnegu  $12, $2
    jr     $31
    movn   $2, $12, $7
    .end        rustix_syscall4_nr_last
    .size       rustix_syscall4_nr_last, .-rustix_syscall4_nr_last

    .section    .text.rustix_syscall5_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall5_nr_last
    .hidden     rustix_syscall5_nr_last
    .type       rustix_syscall5_nr_last, @function
    .set        nomips16
    .set        nomicromips
    .ent        rustix_syscall5_nr_last
rustix_syscall5_nr_last:
    .frame      $sp,0,$31
    .mask       0x00000000,0
    .fmask      0x00000000,0
    .set        noreorder
    .set        nomacro
    move   $2, $9
    syscall
    dnegu  $12, $2
    jr     $31
    movn   $2, $12, $7
    .end        rustix_syscall5_nr_last
    .size       rustix_syscall5_nr_last, .-rustix_syscall5_nr_last

    .section    .text.rustix_syscall6_nr_last,"ax",@progbits
    .p2align    2
    .weak       rustix_syscall6_nr_last
    .hidden     rustix_syscall6_nr_last
    .type       rustix_syscall6_nr_last, @function
    .set        nomips16
    .set        nomicromips
    .ent        rustix_syscall6_nr_last
rustix_syscall6_nr_last:
    .frame      $sp,0,$31
    .mask       0x00000000,0
    .fmask      0x00000000,0
    .set        noreorder
    .set        nomacro
    move   $2, $10
    syscall
    dnegu  $12, $2
    jr     $31
    movn   $2, $12, $7
    .end        rustix_syscall6_nr_last
    .size       rustix_syscall6_nr_last, .-rustix_syscall6_nr_last

    .section .note.GNU-stack,"",@progbits
