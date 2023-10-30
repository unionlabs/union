pub type c_char = u8;
pub type c_long = i64;
pub type c_ulong = u64;
pub type wchar_t = u32;
pub type time_t = i64;
pub type suseconds_t = i64;
pub type register_t = i64;

s_no_extra_traits! {
    pub struct gpregs {
        pub gp_x: [::register_t; 30],
        pub gp_lr: ::register_t,
        pub gp_sp: ::register_t,
        pub gp_elr: ::register_t,
        pub gp_spsr: u32,
        pub gp_pad: ::c_int,
    }

    pub struct fpregs {
        pub fp_q: u128,
        pub fp_sr: u32,
        pub fp_cr: u32,
        pub fp_flags: ::c_int,
        pub fp_pad: ::c_int,
    }

    pub struct mcontext_t {
        pub mc_gpregs: gpregs,
        pub mc_fpregs: fpregs,
        pub mc_flags: ::c_int,
        pub mc_pad: ::c_int,
        pub mc_spare: [u64; 8],
    }
}

// should be pub(crate), but that requires Rust 1.18.0
cfg_if! {
    if #[cfg(libc_const_size_of)] {
        #[doc(hidden)]
        pub const _ALIGNBYTES: usize = ::mem::size_of::<::c_longlong>() - 1;
    } else {
        #[doc(hidden)]
        pub const _ALIGNBYTES: usize = 8 - 1;
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        impl PartialEq for gpregs {
            fn eq(&self, other: &gpregs) -> bool {
                self.gp_x.iter().zip(other.gp_x.iter()).all(|(a, b)| a == b) &&
                self.gp_lr == other.gp_lr &&
                self.gp_sp == other.gp_sp &&
                self.gp_elr == other.gp_elr &&
                self.gp_spsr == other.gp_spsr &&
                self.gp_pad == other.gp_pad
            }
        }
        impl Eq for gpregs {}
        impl ::fmt::Debug for gpregs {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("gpregs")
                    .field("gp_x", &self.gp_x)
                    .field("gp_lr", &self.gp_lr)
                    .field("gp_sp", &self.gp_sp)
                    .field("gp_elr", &self.gp_elr)
                    .field("gp_spsr", &self.gp_spsr)
                    .field("gp_pad", &self.gp_pad)
                    .finish()
            }
        }
        impl ::hash::Hash for gpregs {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.gp_x.hash(state);
                self.gp_lr.hash(state);
                self.gp_sp.hash(state);
                self.gp_elr.hash(state);
                self.gp_spsr.hash(state);
                self.gp_pad.hash(state);
            }
        }
        impl PartialEq for fpregs {
            fn eq(&self, other: &fpregs) -> bool {
                self.fp_q == other.fp_q &&
                self.fp_sr == other.fp_sr &&
                self.fp_cr == other.fp_cr &&
                self.fp_flags == other.fp_flags &&
                self.fp_pad == other.fp_pad
            }
        }
        impl Eq for fpregs {}
        impl ::fmt::Debug for fpregs {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("fpregs")
                    .field("fp_q", &self.fp_q)
                    .field("fp_sr", &self.fp_sr)
                    .field("fp_cr", &self.fp_cr)
                    .field("fp_flags", &self.fp_flags)
                    .field("fp_pad", &self.fp_pad)
                    .finish()
            }
        }
        impl ::hash::Hash for fpregs {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.fp_q.hash(state);
                self.fp_sr.hash(state);
                self.fp_cr.hash(state);
                self.fp_flags.hash(state);
                self.fp_pad.hash(state);
            }
        }
        impl PartialEq for mcontext_t {
            fn eq(&self, other: &mcontext_t) -> bool {
                self.mc_gpregs == other.mc_gpregs &&
                self.mc_fpregs == other.mc_fpregs &&
                self.mc_flags == other.mc_flags &&
                self.mc_pad == other.mc_pad &&
                self.mc_spare.iter().zip(other.mc_spare.iter()).all(|(a, b)| a == b)
            }
        }
        impl Eq for mcontext_t {}
        impl ::fmt::Debug for mcontext_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("mcontext_t")
                    .field("mc_gpregs", &self.mc_gpregs)
                    .field("mc_fpregs", &self.mc_fpregs)
                    .field("mc_flags", &self.mc_flags)
                    .field("mc_pad", &self.mc_pad)
                    .field("mc_spare", &self.mc_spare)
                    .finish()
            }
        }
        impl ::hash::Hash for mcontext_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.mc_gpregs.hash(state);
                self.mc_fpregs.hash(state);
                self.mc_flags.hash(state);
                self.mc_pad.hash(state);
                self.mc_spare.hash(state);
            }
        }
    }
}

pub const MAP_32BIT: ::c_int = 0x00080000;
pub const MINSIGSTKSZ: ::size_t = 4096; // 1024 * 4
