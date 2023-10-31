pub type boolean_t = ::c_int;

s! {
    pub struct malloc_zone_t {
        __private: [::uintptr_t; 18], // FIXME: needs arm64 auth pointers support
    }
}

cfg_if! {
    if #[cfg(libc_align)] {
        mod align;
        pub use self::align::*;
    }
}
