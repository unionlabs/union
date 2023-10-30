use crate::Result;
use crate::errno::Errno;

use libc::{self, c_int, c_ulong};

libc_bitflags! {
    /// Flags used and returned by [`get()`](fn.get.html) and
    /// [`set()`](fn.set.html).
    pub struct Persona: c_int {
        ADDR_COMPAT_LAYOUT;
        ADDR_NO_RANDOMIZE;
        ADDR_LIMIT_32BIT;
        ADDR_LIMIT_3GB;
        #[cfg(not(target_env = "musl"))]
        FDPIC_FUNCPTRS;
        MMAP_PAGE_ZERO;
        READ_IMPLIES_EXEC;
        SHORT_INODE;
        STICKY_TIMEOUTS;
        #[cfg(not(target_env = "musl"))]
        UNAME26;
        WHOLE_SECONDS;
    }
}

/// Retrieve the current process personality.
///
/// Returns a Result containing a Persona instance.
///
/// Example:
///
/// ```
/// # use nix::sys::personality::{self, Persona};
/// let pers = personality::get().unwrap();
/// assert!(!pers.contains(Persona::WHOLE_SECONDS));
/// ```
pub fn get() -> Result<Persona> {
    let res = unsafe {
        libc::personality(0xFFFFFFFF)
    };

    Errno::result(res).map(Persona::from_bits_truncate)
}

/// Set the current process personality.
///
/// Returns a Result containing the *previous* personality for the
/// process, as a Persona.
///
/// For more information, see [personality(2)](https://man7.org/linux/man-pages/man2/personality.2.html)
///
/// **NOTE**: This call **replaces** the current personality entirely.
/// To **update** the personality, first call `get()` and then `set()`
/// with the modified persona.
///
/// Example:
///
/// ```
/// # use nix::sys::personality::{self, Persona};
/// let mut pers = personality::get().unwrap();
/// assert!(!pers.contains(Persona::ADDR_NO_RANDOMIZE));
/// personality::set(pers | Persona::ADDR_NO_RANDOMIZE);
/// ```
pub fn set(persona: Persona) -> Result<Persona> {
    let res = unsafe {
        libc::personality(persona.bits() as c_ulong)
    };

    Errno::result(res).map(Persona::from_bits_truncate)
}
