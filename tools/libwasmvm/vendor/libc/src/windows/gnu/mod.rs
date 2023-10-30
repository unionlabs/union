pub const L_tmpnam: ::c_uint = 14;
pub const TMP_MAX: ::c_uint = 0x7fff;

// stdio file descriptor numbers
pub const STDIN_FILENO: ::c_int = 0;
pub const STDOUT_FILENO: ::c_int = 1;
pub const STDERR_FILENO: ::c_int = 2;

extern "C" {
    pub fn strcasecmp(s1: *const ::c_char, s2: *const ::c_char) -> ::c_int;
    pub fn strncasecmp(s1: *const ::c_char, s2: *const ::c_char, n: ::size_t) -> ::c_int;

    // NOTE: For MSVC target, `wmemchr` is only a inline function in `<wchar.h>`
    //      header file. We cannot find a way to link to that symbol from Rust.
    pub fn wmemchr(cx: *const ::wchar_t, c: ::wchar_t, n: ::size_t) -> *mut ::wchar_t;
}

cfg_if! {
    if #[cfg(libc_align)] {
        mod align;
        pub use self::align::*;
    }
}
