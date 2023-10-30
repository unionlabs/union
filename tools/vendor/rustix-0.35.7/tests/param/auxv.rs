#[cfg(any(
    all(target_os = "android", target_pointer_width = "64"),
    target_os = "linux",
))]
use rustix::param::linux_hwcap;
use rustix::param::{clock_ticks_per_second, page_size};

#[test]
fn test_page_size() {
    let size = page_size();
    assert_ne!(size, 0);
    assert!(size.is_power_of_two());
    assert_eq!(size, page_size());
    assert_eq!(size, unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize });
}

#[test]
fn test_clock_ticks_per_second() {
    let size = clock_ticks_per_second();
    assert_ne!(size, 0);
    assert_eq!(size, unsafe { libc::sysconf(libc::_SC_CLK_TCK) as u64 });
}

#[cfg(any(
    all(target_os = "android", target_pointer_width = "64"),
    target_os = "linux",
))]
#[test]
fn test_linux_hwcap() {
    weak!(fn getauxval(libc::c_ulong) -> libc::c_ulong);

    if let Some(libc_getauxval) = getauxval.get() {
        let (_hwcap, hwcap2) = linux_hwcap();

        // GLIBC seems to return a different value than `LD_SHOW_AUXV=1` reports.
        #[cfg(not(target_env = "gnu"))]
        assert_eq!(_hwcap, unsafe { libc_getauxval(libc::AT_HWCAP) } as usize);

        assert_eq!(hwcap2, unsafe { libc_getauxval(libc::AT_HWCAP2) } as usize);
    }
}
