macro_rules! expand_align {
    () => {
        s! {
            #[cfg_attr(any(target_pointer_width = "32",
                           target_arch = "x86_64",
                           target_arch = "powerpc64",
                           target_arch = "mips64",
                           target_arch = "s390x",
                           target_arch = "sparc64",
                           target_arch = "aarch64",
                           target_arch = "riscv64",
                           target_arch = "riscv32",
                           target_arch = "loongarch64"),
                       repr(align(4)))]
            #[cfg_attr(not(any(target_pointer_width = "32",
                               target_arch = "x86_64",
                               target_arch = "powerpc64",
                               target_arch = "mips64",
                               target_arch = "s390x",
                               target_arch = "sparc64",
                               target_arch = "aarch64",
                               target_arch = "riscv64",
                               target_arch = "riscv32",
                               target_arch = "loongarch64")),
                       repr(align(8)))]
            pub struct pthread_mutexattr_t {
                #[doc(hidden)]
                size: [u8; ::__SIZEOF_PTHREAD_MUTEXATTR_T],
            }

            #[cfg_attr(any(target_env = "musl", target_env = "ohos", target_pointer_width = "32"),
                       repr(align(4)))]
            #[cfg_attr(all(not(target_env = "musl"),
                           not(target_env = "ohos"),
                           target_pointer_width = "64"),
                       repr(align(8)))]
            pub struct pthread_rwlockattr_t {
                #[doc(hidden)]
                size: [u8; ::__SIZEOF_PTHREAD_RWLOCKATTR_T],
            }

            #[repr(align(4))]
            pub struct pthread_condattr_t {
                #[doc(hidden)]
                size: [u8; ::__SIZEOF_PTHREAD_CONDATTR_T],
            }

            #[repr(align(4))]
            pub struct pthread_barrierattr_t {
                #[doc(hidden)]
                size: [u8; ::__SIZEOF_PTHREAD_BARRIERATTR_T],
            }

            #[repr(align(8))]
            pub struct fanotify_event_metadata {
                pub event_len: __u32,
                pub vers: __u8,
                pub reserved: __u8,
                pub metadata_len: __u16,
                pub mask: __u64,
                pub fd: ::c_int,
                pub pid: ::c_int,
            }
        }

        s_no_extra_traits! {
            #[cfg_attr(all(any(target_env = "musl", target_env = "ohos"),
                           target_pointer_width = "32"),
                       repr(align(4)))]
            #[cfg_attr(all(any(target_env = "musl", target_env = "ohos"),
                           target_pointer_width = "64"),
                       repr(align(8)))]
            #[cfg_attr(all(not(any(target_env = "musl", target_env = "ohos")),
                           target_arch = "x86"),
                       repr(align(4)))]
            #[cfg_attr(all(not(any(target_env = "musl", target_env = "ohos")),
                           not(target_arch = "x86")),
                       repr(align(8)))]
            pub struct pthread_cond_t {
                #[doc(hidden)]
                size: [u8; ::__SIZEOF_PTHREAD_COND_T],
            }

            #[cfg_attr(all(target_pointer_width = "32",
                           any(target_arch = "mips",
                               target_arch = "arm",
                               target_arch = "hexagon",
                               target_arch = "m68k",
                               target_arch = "powerpc",
                               target_arch = "sparc",
                               target_arch = "x86_64",
                               target_arch = "x86")),
                       repr(align(4)))]
            #[cfg_attr(any(target_pointer_width = "64",
                           not(any(target_arch = "mips",
                                   target_arch = "arm",
                                   target_arch = "hexagon",
                                   target_arch = "m68k",
                                   target_arch = "powerpc",
                                   target_arch = "sparc",
                                   target_arch = "x86_64",
                                   target_arch = "x86"))),
                       repr(align(8)))]
            pub struct pthread_mutex_t {
                #[doc(hidden)]
                size: [u8; ::__SIZEOF_PTHREAD_MUTEX_T],
            }

            #[cfg_attr(all(target_pointer_width = "32",
                           any(target_arch = "mips",
                               target_arch = "arm",
                               target_arch = "hexagon",
                               target_arch = "m68k",
                               target_arch = "powerpc",
                               target_arch = "sparc",
                               target_arch = "x86_64",
                               target_arch = "x86")),
                       repr(align(4)))]
            #[cfg_attr(any(target_pointer_width = "64",
                           not(any(target_arch = "mips",
                                   target_arch = "arm",
                                   target_arch = "hexagon",
                                   target_arch = "m68k",
                                   target_arch = "powerpc",
                                   target_arch = "sparc",
                                   target_arch = "x86_64",
                                   target_arch = "x86"))),
                       repr(align(8)))]
            pub struct pthread_rwlock_t {
                size: [u8; ::__SIZEOF_PTHREAD_RWLOCK_T],
            }

            #[cfg_attr(all(target_pointer_width = "32",
                           any(target_arch = "mips",
                               target_arch = "arm",
                               target_arch = "hexagon",
                               target_arch = "m68k",
                               target_arch = "powerpc",
                               target_arch = "sparc",
                               target_arch = "x86_64",
                               target_arch = "x86")),
                       repr(align(4)))]
            #[cfg_attr(any(target_pointer_width = "64",
                           not(any(target_arch = "mips",
                                   target_arch = "arm",
                                   target_arch = "hexagon",
                                   target_arch = "m68k",
                                   target_arch = "powerpc",
                                   target_arch = "sparc",
                                   target_arch = "x86_64",
                                   target_arch = "x86"))),
                       repr(align(8)))]
            pub struct pthread_barrier_t {
                size: [u8; ::__SIZEOF_PTHREAD_BARRIER_T],
            }

            // linux/can.h
            #[repr(align(8))]
            #[allow(missing_debug_implementations)]
            pub struct can_frame {
                pub can_id: canid_t,
                pub can_dlc: u8,
                __pad: u8,
                __res0: u8,
                __res1: u8,
                pub data: [u8; CAN_MAX_DLEN],
            }

            #[repr(align(8))]
            #[allow(missing_debug_implementations)]
            pub struct canfd_frame {
                pub can_id: canid_t,
                pub len: u8,
                pub flags: u8,
                __res0: u8,
                __res1: u8,
                pub data: [u8; CANFD_MAX_DLEN],
            }

            #[repr(align(8))]
            #[allow(missing_debug_implementations)]
            pub struct canxl_frame {
                pub prio: canid_t,
                pub flags: u8,
                pub sdt: u8,
                pub len: u16,
                pub af: u32,
                pub data: [u8; CANXL_MAX_DLEN],
            }
        }
    };
}
