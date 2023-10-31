pub type c_char = i8;
pub type wchar_t = u32;
pub type c_long = i64;
pub type c_ulong = u64;
pub type time_t = i64;

s! {
    #[repr(align(8))]
    pub struct x86_64_cpu_registers {
        pub rdi: u64,
        pub rsi: u64,
        pub rdx: u64,
        pub r10: u64,
        pub r8: u64,
        pub r9: u64,
        pub rax: u64,
        pub rbx: u64,
        pub rbp: u64,
        pub rcx: u64,
        pub r11: u64,
        pub r12: u64,
        pub r13: u64,
        pub r14: u64,
        pub r15: u64,
        pub rip: u64,
        pub cs: u32,
        rsvd1: u32,
        pub rflags: u64,
        pub rsp: u64,
        pub ss: u32,
        rsvd2: u32,
    }

    #[repr(align(8))]
    pub struct mcontext_t {
        pub cpu: x86_64_cpu_registers,
        #[cfg(libc_union)]
        pub fpu: x86_64_fpu_registers,
        #[cfg(not(libc_union))]
        __reserved: [u8; 1024],
    }

    pub struct stack_t {
        pub ss_sp: *mut ::c_void,
        pub ss_size: ::size_t,
        pub ss_flags: ::c_int,
    }

    pub struct fsave_area_64 {
        pub fpu_control_word: u32,
        pub fpu_status_word: u32,
        pub fpu_tag_word: u32,
        pub fpu_ip: u32,
        pub fpu_cs: u32,
        pub fpu_op: u32,
        pub fpu_ds: u32,
        pub st_regs: [u8; 80],
   }

    pub struct fxsave_area_64 {
        pub fpu_control_word: u16,
        pub fpu_status_word: u16,
        pub fpu_tag_word: u16,
        pub fpu_operand: u16,
        pub fpu_rip: u64,
        pub fpu_rdp: u64,
        pub mxcsr: u32,
        pub mxcsr_mask: u32,
        pub st_regs: [u8; 128],
        pub xmm_regs: [u8; 128],
        reserved2: [u8; 224],
    }

    pub struct fpu_extention_savearea_64 {
        pub other: [u8; 512],
        pub xstate_bv: u64,
        pub xstate_undef: [u64; 7],
        pub xstate_info: [u8; 224],
    }
}

s_no_extra_traits! {
    #[cfg(libc_union)]
    pub union x86_64_fpu_registers {
        pub fsave_area: fsave_area_64,
        pub fxsave_area: fxsave_area_64,
        pub xsave_area: fpu_extention_savearea_64,
        pub data: [u8; 1024],
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        #[cfg(libc_union)]
        impl Eq for x86_64_fpu_registers {}

        #[cfg(libc_union)]
        impl PartialEq for x86_64_fpu_registers {
            fn eq(&self, other: &x86_64_fpu_registers) -> bool {
                unsafe {
                    self.fsave_area == other.fsave_area
                        || self.fxsave_area == other.fxsave_area
                        || self.xsave_area == other.xsave_area
                }
            }
        }

        #[cfg(libc_union)]
        impl ::fmt::Debug for x86_64_fpu_registers {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                unsafe {
                    f.debug_struct("x86_64_fpu_registers")
                        .field("fsave_area", &self.fsave_area)
                        .field("fxsave_area", &self.fxsave_area)
                        .field("xsave_area", &self.xsave_area)
                        .finish()
                }
            }
        }

        #[cfg(libc_union)]
        impl ::hash::Hash for x86_64_fpu_registers {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                unsafe {
                    self.fsave_area.hash(state);
                    self.fxsave_area.hash(state);
                    self.xsave_area.hash(state);
                }
            }
        }
    }
}
