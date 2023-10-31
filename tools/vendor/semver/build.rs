use std::env;
use std::process::Command;
use std::str;

fn main() {
    let compiler = match rustc_minor_version() {
        Some(compiler) => compiler,
        None => return,
    };

    if compiler < 33 {
        // Exhaustive integer patterns. On older compilers, a final `_` arm is
        // required even if every possible integer value is otherwise covered.
        // https://github.com/rust-lang/rust/issues/50907
        println!("cargo:rustc-cfg=no_exhaustive_int_match");
    }

    if compiler < 36 {
        // extern crate alloc.
        // https://blog.rust-lang.org/2019/07/04/Rust-1.36.0.html#the-alloc-crate-is-stable
        println!("cargo:rustc-cfg=no_alloc_crate");
    }

    if compiler < 39 {
        // const Vec::new.
        // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.new
        println!("cargo:rustc-cfg=no_const_vec_new");
    }

    if compiler < 40 {
        // #[non_exhaustive].
        // https://blog.rust-lang.org/2019/12/19/Rust-1.40.0.html#non_exhaustive-structs-enums-and-variants
        println!("cargo:rustc-cfg=no_non_exhaustive");
    }

    if compiler < 45 {
        // String::strip_prefix.
        // https://doc.rust-lang.org/std/primitive.str.html#method.repeat
        println!("cargo:rustc-cfg=no_str_strip_prefix");
    }

    if compiler < 46 {
        // #[track_caller].
        // https://blog.rust-lang.org/2020/08/27/Rust-1.46.0.html#track_caller
        println!("cargo:rustc-cfg=no_track_caller");
    }

    if compiler < 52 {
        // #![deny(unsafe_op_in_unsafe_fn)].
        // https://github.com/rust-lang/rust/issues/71668
        println!("cargo:rustc-cfg=no_unsafe_op_in_unsafe_fn_lint");
    }

    if compiler < 53 {
        // Efficient intrinsics for count-leading-zeros and count-trailing-zeros
        // on NonZero integers stabilized in 1.53.0. On many architectures these
        // are more efficient than counting zeros on ordinary zeroable integers.
        // https://doc.rust-lang.org/std/num/struct.NonZeroU64.html#method.leading_zeros
        // https://doc.rust-lang.org/std/num/struct.NonZeroU64.html#method.trailing_zeros
        println!("cargo:rustc-cfg=no_nonzero_bitscan");
    }
}

fn rustc_minor_version() -> Option<u32> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = str::from_utf8(&output.stdout).ok()?;
    let mut pieces = version.split('.');
    if pieces.next() != Some("rustc 1") {
        return None;
    }
    pieces.next()?.parse().ok()
}
