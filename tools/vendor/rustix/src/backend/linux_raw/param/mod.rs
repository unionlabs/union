// On Mustang, origin is in control of program startup and can access the
// incoming aux values on the stack.
//
// With "use-libc-auxv" enabled, use libc's `getauxval`.
//
// Otherwise, we read aux values from /proc/self/auxv.
#[cfg_attr(target_vendor = "mustang", path = "mustang_auxv.rs")]
#[cfg_attr(
    all(not(target_vendor = "mustang"), feature = "use-libc-auxv"),
    path = "libc_auxv.rs"
)]
pub(crate) mod auxv;
