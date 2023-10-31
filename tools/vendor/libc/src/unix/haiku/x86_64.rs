s_no_extra_traits! {
    pub struct fpu_state {
        pub control: ::c_ushort,
        pub status: ::c_ushort,
        pub tag: ::c_ushort,
        pub opcode: ::c_ushort,
        pub rip: ::c_ulong,
        pub rdp: ::c_ulong,
        pub mxcsr: ::c_uint,
        pub mscsr_mask: ::c_uint,
        pub _fpreg: [[::c_uchar; 8]; 16],
        pub _xmm: [[::c_uchar; 16]; 16],
        pub _reserved_416_511: [::c_uchar; 96],
    }

    pub struct xstate_hdr {
        pub bv: ::c_ulong,
        pub xcomp_bv: ::c_ulong,
        pub _reserved: [::c_uchar; 48],
    }

    pub struct savefpu {
        pub fp_fxsave: fpu_state,
        pub fp_xstate: xstate_hdr,
        pub _fp_ymm: [[::c_uchar; 16]; 16],
    }

    pub struct mcontext_t {
        pub rax: ::c_ulong,
        pub rbx: ::c_ulong,
        pub rcx: ::c_ulong,
        pub rdx: ::c_ulong,
        pub rdi: ::c_ulong,
        pub rsi: ::c_ulong,
        pub rbp: ::c_ulong,
        pub r8: ::c_ulong,
        pub r9: ::c_ulong,
        pub r10: ::c_ulong,
        pub r11: ::c_ulong,
        pub r12: ::c_ulong,
        pub r13: ::c_ulong,
        pub r14: ::c_ulong,
        pub r15: ::c_ulong,
        pub rsp: ::c_ulong,
        pub rip: ::c_ulong,
        pub rflags: ::c_ulong,
        pub fpu: savefpu,
    }

    pub struct ucontext_t {
        pub uc_link: *mut ucontext_t,
        pub uc_sigmask: ::sigset_t,
        pub uc_stack: ::stack_t,
        pub uc_mcontext: mcontext_t,
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        impl PartialEq for fpu_state {
            fn eq(&self, other: &fpu_state) -> bool {
                self.control == other.control
                    && self.status == other.status
                    && self.tag == other.tag
                    && self.opcode == other.opcode
                    && self.rip == other.rip
                    && self.rdp == other.rdp
                    && self.mxcsr == other.mxcsr
                    && self.mscsr_mask == other.mscsr_mask
                    && self._fpreg.iter().zip(other._fpreg.iter()).all(|(a, b)| a == b)
                    && self._xmm.iter().zip(other._xmm.iter()).all(|(a, b)| a == b)
                    && self._reserved_416_511.
                        iter().
                        zip(other._reserved_416_511.iter()).
                        all(|(a, b)| a == b)
            }
        }
        impl Eq for fpu_state {}
        impl ::fmt::Debug for fpu_state {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("fpu_state")
                    .field("control", &self.control)
                    .field("status", &self.status)
                    .field("tag", &self.tag)
                    .field("opcode", &self.opcode)
                    .field("rip", &self.rip)
                    .field("rdp", &self.rdp)
                    .field("mxcsr", &self.mxcsr)
                    .field("mscsr_mask", &self.mscsr_mask)
                    // FIXME: .field("_fpreg", &self._fpreg)
                    // FIXME: .field("_xmm", &self._xmm)
                    // FIXME: .field("_reserved_416_511", &self._reserved_416_511)
                    .finish()
            }
        }
        impl ::hash::Hash for fpu_state {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.control.hash(state);
                self.status.hash(state);
                self.tag.hash(state);
                self.opcode.hash(state);
                self.rip.hash(state);
                self.rdp.hash(state);
                self.mxcsr.hash(state);
                self.mscsr_mask.hash(state);
                self._fpreg.hash(state);
                self._xmm.hash(state);
                self._reserved_416_511.hash(state);
            }
        }

        impl PartialEq for xstate_hdr {
            fn eq(&self, other: &xstate_hdr) -> bool {
                self.bv == other.bv
                    && self.xcomp_bv == other.xcomp_bv
                    && self._reserved.iter().zip(other._reserved.iter()).all(|(a, b)| a == b)
            }
        }
        impl Eq for xstate_hdr {}
        impl ::fmt::Debug for xstate_hdr {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("xstate_hdr")
                    .field("bv", &self.bv)
                    .field("xcomp_bv", &self.xcomp_bv)
                    // FIXME: .field("_reserved", &field._reserved)
                    .finish()
            }
        }
        impl ::hash::Hash for xstate_hdr {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.bv.hash(state);
                self.xcomp_bv.hash(state);
                self._reserved.hash(state);
            }
        }

        impl PartialEq for savefpu {
            fn eq(&self, other: &savefpu) -> bool {
                self.fp_fxsave == other.fp_fxsave
                    && self.fp_xstate == other.fp_xstate
                    && self._fp_ymm.iter().zip(other._fp_ymm.iter()).all(|(a, b)| a == b)
            }
        }
        impl Eq for savefpu {}
        impl ::fmt::Debug for savefpu {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("savefpu")
                    .field("fp_fxsave", &self.fp_fxsave)
                    .field("fp_xstate", &self.fp_xstate)
                    // FIXME: .field("_fp_ymm", &field._fp_ymm)
                    .finish()
            }
        }
        impl ::hash::Hash for savefpu {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.fp_fxsave.hash(state);
                self.fp_xstate.hash(state);
                self._fp_ymm.hash(state);
            }
        }

        impl PartialEq for mcontext_t {
            fn eq(&self, other: &mcontext_t) -> bool {
                self.rax == other.rax
                    && self.rbx == other.rbx
                    && self.rbx == other.rbx
                    && self.rcx == other.rcx
                    && self.rdx == other.rdx
                    && self.rdi == other.rdi
                    && self.rsi == other.rsi
                    && self.r8 == other.r8
                    && self.r9 == other.r9
                    && self.r10 == other.r10
                    && self.r11 == other.r11
                    && self.r12 == other.r12
                    && self.r13 == other.r13
                    && self.r14 == other.r14
                    && self.r15 == other.r15
                    && self.rsp == other.rsp
                    && self.rip == other.rip
                    && self.rflags == other.rflags
                    && self.fpu == other.fpu
            }
        }
        impl Eq for mcontext_t {}
        impl ::fmt::Debug for mcontext_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("mcontext_t")
                    .field("rax", &self.rax)
                    .field("rbx", &self.rbx)
                    .field("rcx", &self.rcx)
                    .field("rdx", &self.rdx)
                    .field("rdi", &self.rdi)
                    .field("rsi", &self.rsi)
                    .field("rbp", &self.rbp)
                    .field("r8", &self.r8)
                    .field("r9", &self.r9)
                    .field("r10", &self.r10)
                    .field("r11", &self.r11)
                    .field("r12", &self.r12)
                    .field("r13", &self.r13)
                    .field("r14", &self.r14)
                    .field("r15", &self.r15)
                    .field("rsp", &self.rsp)
                    .field("rip", &self.rip)
                    .field("rflags", &self.rflags)
                    .field("fpu", &self.fpu)
                    .finish()

            }
        }
        impl ::hash::Hash for mcontext_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.rax.hash(state);
                self.rbx.hash(state);
                self.rcx.hash(state);
                self.rdx.hash(state);
                self.rdi.hash(state);
                self.rsi.hash(state);
                self.rbp.hash(state);
                self.r8.hash(state);
                self.r9.hash(state);
                self.r10.hash(state);
                self.r11.hash(state);
                self.r12.hash(state);
                self.r13.hash(state);
                self.r14.hash(state);
                self.r15.hash(state);
                self.rsp.hash(state);
                self.rip.hash(state);
                self.rflags.hash(state);
                self.fpu.hash(state);
            }
        }

        impl PartialEq for ucontext_t {
            fn eq(&self, other: &ucontext_t) -> bool {
                self.uc_link == other.uc_link
                    && self.uc_sigmask == other.uc_sigmask
                    && self.uc_stack == other.uc_stack
                    && self.uc_mcontext == other.uc_mcontext
            }
        }
        impl Eq for ucontext_t {}
        impl ::fmt::Debug for ucontext_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("ucontext_t")
                    .field("uc_link", &self.uc_link)
                    .field("uc_sigmask", &self.uc_sigmask)
                    .field("uc_stack", &self.uc_stack)
                    .field("uc_mcontext", &self.uc_mcontext)
                    .finish()
            }
        }
        impl ::hash::Hash for ucontext_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.uc_link.hash(state);
                self.uc_sigmask.hash(state);
                self.uc_stack.hash(state);
                self.uc_mcontext.hash(state);
            }
        }
    }
}
