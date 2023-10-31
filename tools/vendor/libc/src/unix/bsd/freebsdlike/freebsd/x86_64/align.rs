use {c_long, register_t};

s_no_extra_traits! {
    #[allow(missing_debug_implementations)]
    #[repr(align(16))]
    pub struct max_align_t {
        priv_: [f64; 4]
    }

    #[repr(align(16))]
    pub struct mcontext_t {
        pub mc_onstack: register_t,
        pub mc_rdi: register_t,
        pub mc_rsi: register_t,
        pub mc_rdx: register_t,
        pub mc_rcx: register_t,
        pub mc_r8: register_t,
        pub mc_r9: register_t,
        pub mc_rax: register_t,
        pub mc_rbx: register_t,
        pub mc_rbp: register_t,
        pub mc_r10: register_t,
        pub mc_r11: register_t,
        pub mc_r12: register_t,
        pub mc_r13: register_t,
        pub mc_r14: register_t,
        pub mc_r15: register_t,
        pub mc_trapno: u32,
        pub mc_fs: u16,
        pub mc_gs: u16,
        pub mc_addr: register_t,
        pub mc_flags: u32,
        pub mc_es: u16,
        pub mc_ds: u16,
        pub mc_err: register_t,
        pub mc_rip: register_t,
        pub mc_cs: register_t,
        pub mc_rflags: register_t,
        pub mc_rsp: register_t,
        pub mc_ss: register_t,
        pub mc_len: c_long,
        pub mc_fpformat: c_long,
        pub mc_ownedfp: c_long,
        pub mc_fpstate: [c_long; 64],
        pub mc_fsbase: register_t,
        pub mc_gsbase: register_t,
        pub mc_xfpustate: register_t,
        pub mc_xfpustate_len: register_t,
        pub mc_spare: [c_long; 4],
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        impl PartialEq for mcontext_t {
            fn eq(&self, other: &mcontext_t) -> bool {
                self.mc_onstack == other.mc_onstack &&
                self.mc_rdi == other.mc_rdi &&
                self.mc_rsi == other.mc_rsi &&
                self.mc_rdx == other.mc_rdx &&
                self.mc_rcx == other.mc_rcx &&
                self.mc_r8 == other.mc_r8 &&
                self.mc_r9 == other.mc_r9 &&
                self.mc_rax == other.mc_rax &&
                self.mc_rbx == other.mc_rbx &&
                self.mc_rbp == other.mc_rbp &&
                self.mc_r10 == other.mc_r10 &&
                self.mc_r11 == other.mc_r11 &&
                self.mc_r12 == other.mc_r12 &&
                self.mc_r13 == other.mc_r13 &&
                self.mc_r14 == other.mc_r14 &&
                self.mc_r15 == other.mc_r15 &&
                self.mc_trapno == other.mc_trapno &&
                self.mc_fs == other.mc_fs &&
                self.mc_gs == other.mc_gs &&
                self.mc_addr == other.mc_addr &&
                self.mc_flags == other.mc_flags &&
                self.mc_es == other.mc_es &&
                self.mc_ds == other.mc_ds &&
                self.mc_err == other.mc_err &&
                self.mc_rip == other.mc_rip &&
                self.mc_cs == other.mc_cs &&
                self.mc_rflags == other.mc_rflags &&
                self.mc_rsp == other.mc_rsp &&
                self.mc_ss == other.mc_ss &&
                self.mc_len == other.mc_len &&
                self.mc_fpformat == other.mc_fpformat &&
                self.mc_ownedfp == other.mc_ownedfp &&
                self.mc_fpstate.iter().zip(other.mc_fpstate.iter())
                .all(|(a, b)| a == b) &&
                self.mc_fsbase == other.mc_fsbase &&
                self.mc_gsbase == other.mc_gsbase &&
                self.mc_xfpustate == other.mc_xfpustate &&
                self.mc_xfpustate_len == other.mc_xfpustate_len &&
                self.mc_spare == other.mc_spare
            }
        }
        impl Eq for mcontext_t {}
        impl ::fmt::Debug for mcontext_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("mcontext_t")
                    .field("mc_onstack", &self.mc_onstack)
                    .field("mc_rdi", &self.mc_rdi)
                    .field("mc_rsi", &self.mc_rsi)
                    .field("mc_rdx", &self.mc_rdx)
                    .field("mc_rcx", &self.mc_rcx)
                    .field("mc_r8", &self.mc_r8)
                    .field("mc_r9", &self.mc_r9)
                    .field("mc_rax", &self.mc_rax)
                    .field("mc_rbx", &self.mc_rbx)
                    .field("mc_rbp", &self.mc_rbp)
                    .field("mc_r10", &self.mc_r10)
                    .field("mc_r11", &self.mc_r11)
                    .field("mc_r12", &self.mc_r12)
                    .field("mc_r13", &self.mc_r13)
                    .field("mc_r14", &self.mc_r14)
                    .field("mc_r15", &self.mc_r15)
                    .field("mc_trapno", &self.mc_trapno)
                    .field("mc_fs", &self.mc_fs)
                    .field("mc_gs", &self.mc_gs)
                    .field("mc_addr", &self.mc_addr)
                    .field("mc_flags", &self.mc_flags)
                    .field("mc_es", &self.mc_es)
                    .field("mc_ds", &self.mc_ds)
                    .field("mc_err", &self.mc_err)
                    .field("mc_rip", &self.mc_rip)
                    .field("mc_cs", &self.mc_cs)
                    .field("mc_rflags", &self.mc_rflags)
                    .field("mc_rsp", &self.mc_rsp)
                    .field("mc_ss", &self.mc_ss)
                    .field("mc_len", &self.mc_len)
                    .field("mc_fpformat", &self.mc_fpformat)
                    .field("mc_ownedfp", &self.mc_ownedfp)
                    // FIXME: .field("mc_fpstate", &self.mc_fpstate)
                    .field("mc_fsbase", &self.mc_fsbase)
                    .field("mc_gsbase", &self.mc_gsbase)
                    .field("mc_xfpustate", &self.mc_xfpustate)
                    .field("mc_xfpustate_len", &self.mc_xfpustate_len)
                    .field("mc_spare", &self.mc_spare)
                    .finish()
            }
        }
        impl ::hash::Hash for mcontext_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.mc_onstack.hash(state);
                self.mc_rdi.hash(state);
                self.mc_rsi.hash(state);
                self.mc_rdx.hash(state);
                self.mc_rcx.hash(state);
                self.mc_r8.hash(state);
                self.mc_r9.hash(state);
                self.mc_rax.hash(state);
                self.mc_rbx.hash(state);
                self.mc_rbp.hash(state);
                self.mc_r10.hash(state);
                self.mc_r11.hash(state);
                self.mc_r12.hash(state);
                self.mc_r13.hash(state);
                self.mc_r14.hash(state);
                self.mc_r15.hash(state);
                self.mc_trapno.hash(state);
                self.mc_fs.hash(state);
                self.mc_gs.hash(state);
                self.mc_addr.hash(state);
                self.mc_flags.hash(state);
                self.mc_es.hash(state);
                self.mc_ds.hash(state);
                self.mc_err.hash(state);
                self.mc_rip.hash(state);
                self.mc_cs.hash(state);
                self.mc_rflags.hash(state);
                self.mc_rsp.hash(state);
                self.mc_ss.hash(state);
                self.mc_len.hash(state);
                self.mc_fpformat.hash(state);
                self.mc_ownedfp.hash(state);
                self.mc_fpstate.hash(state);
                self.mc_fsbase.hash(state);
                self.mc_gsbase.hash(state);
                self.mc_xfpustate.hash(state);
                self.mc_xfpustate_len.hash(state);
                self.mc_spare.hash(state);
            }
        }
    }
}

s! {
    pub struct ucontext_t {
        pub uc_sigmask: ::sigset_t,
        pub uc_mcontext: ::mcontext_t,
        pub uc_link: *mut ::ucontext_t,
        pub uc_stack: ::stack_t,
        pub uc_flags: ::c_int,
        __spare__: [::c_int; 4],
    }
}
