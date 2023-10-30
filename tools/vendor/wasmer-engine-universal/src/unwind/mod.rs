cfg_if::cfg_if! {
    if #[cfg(all(windows, target_arch = "x86_64"))] {
        mod windows_x64;
        pub use self::windows_x64::*;
    } else if #[cfg(unix)] {
        mod systemv;
        pub use self::systemv::*;
    } else {
        // Otherwise, we provide a dummy fallback without unwinding
        mod dummy;
        pub use self::dummy::DummyUnwindRegistry as UnwindRegistry;
    }
}
