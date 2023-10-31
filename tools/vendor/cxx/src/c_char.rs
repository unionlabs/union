#[allow(missing_docs)]
pub type c_char = c_char_definition::c_char;

// Validate that our definition is consistent with libstd's definition, without
// introducing a dependency on libstd in ordinary builds.
#[cfg(all(test, feature = "std"))]
const _: self::c_char = 0 as std::os::raw::c_char;

#[allow(dead_code)]
mod c_char_definition {
    // These are the targets on which c_char is unsigned.
    #[cfg(any(
        all(
            target_os = "linux",
            any(
                target_arch = "aarch64",
                target_arch = "arm",
                target_arch = "hexagon",
                target_arch = "powerpc",
                target_arch = "powerpc64",
                target_arch = "s390x",
                target_arch = "riscv64",
                target_arch = "riscv32"
            )
        ),
        all(
            target_os = "android",
            any(target_arch = "aarch64", target_arch = "arm")
        ),
        all(target_os = "l4re", target_arch = "x86_64"),
        all(
            target_os = "freebsd",
            any(
                target_arch = "aarch64",
                target_arch = "arm",
                target_arch = "powerpc",
                target_arch = "powerpc64",
                target_arch = "riscv64"
            )
        ),
        all(
            target_os = "netbsd",
            any(target_arch = "aarch64", target_arch = "arm", target_arch = "powerpc")
        ),
        all(target_os = "openbsd", target_arch = "aarch64"),
        all(
            target_os = "vxworks",
            any(
                target_arch = "aarch64",
                target_arch = "arm",
                target_arch = "powerpc64",
                target_arch = "powerpc"
            )
        ),
        all(target_os = "fuchsia", target_arch = "aarch64")
    ))]
    pub use self::unsigned::c_char;

    // On every other target, c_char is signed.
    pub use self::signed::*;

    mod unsigned {
        pub type c_char = u8;
    }

    mod signed {
        pub type c_char = i8;
    }
}
