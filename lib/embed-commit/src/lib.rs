use bytemuck::CheckedBitPattern;

/// The git rev of the code, as supplied at build time. On `wasm32` targets, this is available via the `commit_hash` export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, CheckedBitPattern)]
#[repr(C, u64)]
#[rustfmt::skip]
pub enum Rev {
    /// The state of the build is unknown (i.e. `GIT_REV` was not set).
    //                  U  N  K  N  O  W  N
    Unknown        = 0x_75_6E_6B_6E_6F_77_6E,
    /// The build is dirty.
    ///
    /// `GIT_REV=dirty`
    //                  D  I  R  T  Y
    Dirty          = 0x_64_69_72_74_79,
    /// The build was done on the specified commit hash.
    ///
    /// `GIT_REV=6e65766572676f6e6e6167697665796f75757020`
    //                  H  A  S  H
    Hash([u8; 20]) = 0x_68_61_73_68,
}

impl core::fmt::Display for Rev {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rev::Unknown => f.write_str("unknown"),
            Rev::Dirty => f.write_str("dirty"),
            Rev::Hash(hash) => const_hex::Buffer::<20, false>::new().format(hash).fmt(f),
        }
    }
}

#[cfg_attr(
    not(target_arch = "wasm32"),
    no_mangle,
    used,
    link_section = ".note.embed_commit.GIT_REV"
)]
pub static GIT_REV: Rev = match option_env!("GIT_REV") {
    None => Rev::Unknown,
    Some(hash) => match hash.as_bytes() {
        b"dirty" => Rev::Dirty,
        hash => Rev::Hash(match const_hex::const_decode_to_array(hash) {
            Ok(ok) => ok,
            Err(_) => panic!(
                "invalid GIT_REV env var, value must be either \
                unset, \"dirty\" or a 20-byte hex string (commit)"
            ),
        }),
    },
};

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn commit_hash() -> Rev {
    unsafe { core::ptr::read_volatile(&GIT_REV as *const _) }
}
