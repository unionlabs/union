s_no_extra_traits! {
    #[allow(missing_debug_implementations)]
    #[repr(align(8))]
    pub struct max_align_t {
        priv_: [i64; 2]
    }

    #[allow(missing_debug_implementations)]
    #[repr(align(8))]
    pub struct ucontext_t {
        pub uc_flags: ::c_ulong,
        pub uc_link: *mut ucontext_t,
        pub uc_stack: ::stack_t,
        pub uc_mcontext: ::mcontext_t,
        pub uc_sigmask: ::sigset_t,
        pub uc_regspace: [::c_ulong; 128],
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        impl PartialEq for ucontext_t {
            fn eq(&self, other: &ucontext_t) -> bool {
                self.uc_flags == other.uc_flags
                    && self.uc_link == other.uc_link
                    && self.uc_stack == other.uc_stack
                    && self.uc_mcontext == other.uc_mcontext
                    && self.uc_sigmask == other.uc_sigmask
            }
        }
        impl Eq for ucontext_t {}
        impl ::fmt::Debug for ucontext_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("ucontext_t")
                    .field("uc_flags", &self.uc_link)
                    .field("uc_link", &self.uc_link)
                    .field("uc_stack", &self.uc_stack)
                    .field("uc_mcontext", &self.uc_mcontext)
                    .field("uc_sigmask", &self.uc_sigmask)
                    .finish()
            }
        }
        impl ::hash::Hash for ucontext_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.uc_flags.hash(state);
                self.uc_link.hash(state);
                self.uc_stack.hash(state);
                self.uc_mcontext.hash(state);
                self.uc_sigmask.hash(state);
            }
        }
    }
}
