use core::mem;

// Helper macros to deal with platform-specific differences in assembly code
// between ELF, Mach-O and COFF file formats.
cfg_if::cfg_if! {
    if #[cfg(any(
            target_vendor = "apple",
            all(windows, target_arch = "x86"),
        ))] {
        macro_rules! asm_mangle {
            ($name:literal) => { concat!("_", $name) };
        }
    } else {
        macro_rules! asm_mangle {
            ($name:literal) => { $name };
        }
    }
}
cfg_if::cfg_if! {
    if #[cfg(windows)] {
        // COFF
        macro_rules! asm_function_begin {
            ($name:literal) => {
                concat!(
                    ".globl ", asm_mangle!($name), "\n",
                    ".def ", asm_mangle!($name), "\n",
                    ".scl 2\n",
                    ".type 32\n",
                    ".endef ", asm_mangle!($name), "\n",
                    asm_mangle!($name), ":\n",
                )
            };
        }
        macro_rules! asm_function_alt_entry {
            ($name:literal) => {
                asm_function_begin!($name)
            };
        }
        macro_rules! asm_function_end {
            ($name:literal) => {
                ""
            };
        }
    } else if #[cfg(target_vendor = "apple")] {
        // Mach-O
        macro_rules! asm_function_begin {
            ($name:literal) => {
                concat!(
                    ".globl ", asm_mangle!($name), "\n",
                    ".private_extern ", asm_mangle!($name), "\n",
                    asm_mangle!($name), ":\n",
                )
            };
        }
        // Mach-O normally splits each symbol into its own section unless we
        // explicitly mark it as an alternate entry point.
        macro_rules! asm_function_alt_entry {
            ($name:literal) => {
                concat!(
                    ".alt_entry ", asm_mangle!($name), "\n",
                    asm_function_begin!($name),
                )
            };
        }
        macro_rules! asm_function_end {
            ($name:literal) => {
                ""
            };
        }
        // Darwin refuses to unwind through signal frames, but we don't need it
        // there anyways since those platforms don't use GDB (which was the
        // whole point of using .cfi_signal_frame in the first place).
        macro_rules! cfi_signal_frame {
            () => { "" }
        }
    } else {
        // Everything else uses ELF
        #[cfg(not(target_arch = "arm"))]
        macro_rules! asm_function_type {
            () => { "@function" }
        }
        #[cfg(target_arch = "arm")]
        macro_rules! asm_function_type {
            () => { "%function" }
        }
        macro_rules! asm_function_begin {
            ($name:literal) => {
                concat!(
                    ".globl ", asm_mangle!($name), "\n",
                    ".hidden ", asm_mangle!($name), "\n",
                    ".type ", asm_mangle!($name), ", ", asm_function_type!(), "\n",
                    asm_mangle!($name), ":\n",
                )
            };
        }
        macro_rules! asm_function_alt_entry {
            ($name:literal) => {
                asm_function_begin!($name)
            };
        }
        macro_rules! asm_function_end {
            ($name:literal) => {
                concat!(".size ", asm_mangle!($name), ", . - ", asm_mangle!($name), "\n")
            };
        }
        macro_rules! cfi_signal_frame {
            () => { ".cfi_signal_frame" }
        }
    }
}

/// Helper function to allocate an object on the stack with proper alignment.
///
/// This function is written such that the stack pointer alignment can be
/// constant-folded away when the object doesn't need an alignment greater than
/// `STACK_ALIGNMENT`.
#[inline]
unsafe fn allocate_obj_on_stack<T>(sp: &mut usize, sp_offset: usize, obj: T) {
    // Sanity check to avoid stack overflows.
    assert!(mem::size_of::<T>() <= 1024, "type is too big to transfer");

    if mem::align_of::<T>() > STACK_ALIGNMENT {
        *sp -= mem::size_of::<T>();
        *sp &= !(mem::align_of::<T>() - 1);
    } else {
        // We know that sp + sp_offset is aligned to STACK_ALIGNMENT. Calculate
        // how much padding we need to add so that sp_offset + padding +
        // sizeof(T) is aligned to STACK_ALIGNMENT.
        let total_size = sp_offset + mem::size_of::<T>();
        let align_offset = total_size % STACK_ALIGNMENT;
        if align_offset != 0 {
            *sp -= STACK_ALIGNMENT - align_offset;
        }
        *sp -= mem::size_of::<T>();
    }
    (*sp as *mut T).write(obj);

    // The stack is aligned to STACK_ALIGNMENT at this point.
    debug_assert_eq!(*sp % STACK_ALIGNMENT, 0);
}

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "x86_64", not(windows)))] {
        mod x86_64;
        pub use self::x86_64::*;
    } else if #[cfg(all(target_arch = "x86_64", windows))] {
        mod x86_64_windows;
        pub use self::x86_64_windows::*;
    } else if #[cfg(all(target_arch = "x86", not(windows)))] {
        mod x86;
        pub use self::x86::*;
    } else if #[cfg(all(target_arch = "x86", windows))] {
        mod x86_windows;
        pub use self::x86_windows::*;
    } else if #[cfg(all(target_arch = "aarch64", not(windows)))] {
        mod aarch64;
        pub use self::aarch64::*;
    } else if #[cfg(all(target_arch = "arm", not(any(windows, target_vendor = "apple"))))] {
        // Apple on ARM uses SJLJ unwinding which we don't support.
        mod arm;
        pub use self::arm::*;
    } else if #[cfg(all(any(target_arch = "riscv64", target_arch = "riscv32"), not(windows)))] {
        mod riscv;
        pub use self::riscv::*;
    } else {
        compile_error!("Unsupported target");
    }
}

/// Helper function to push a value onto a stack.
#[inline]
unsafe fn push(sp: &mut usize, val: Option<StackWord>) {
    *sp -= mem::size_of::<StackWord>();
    if let Some(val) = val {
        *(*sp as *mut StackWord) = val;
    }
}
