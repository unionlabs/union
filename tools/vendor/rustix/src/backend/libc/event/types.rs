#[cfg(any(linux_kernel, target_os = "freebsd", target_os = "illumos"))]
use {crate::backend::c, bitflags::bitflags};

#[cfg(any(linux_kernel, target_os = "freebsd", target_os = "illumos"))]
bitflags! {
    /// `EFD_*` flags for use with [`eventfd`].
    ///
    /// [`eventfd`]: crate::io::eventfd
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct EventfdFlags: u32 {
        /// `EFD_CLOEXEC`
        const CLOEXEC = bitcast!(c::EFD_CLOEXEC);
        /// `EFD_NONBLOCK`
        const NONBLOCK = bitcast!(c::EFD_NONBLOCK);
        /// `EFD_SEMAPHORE`
        const SEMAPHORE = bitcast!(c::EFD_SEMAPHORE);
    }
}
