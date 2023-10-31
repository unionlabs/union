use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(unix)] {
        mod unix;
        pub use unix::*;
        pub(crate) use std::os::unix::prelude::AsRawFd as AsRaw;
    } else if #[cfg(windows)] {
        mod windows;
        pub use windows::*;
        #[doc(no_inline)]
        pub(crate) use std::os::windows::prelude::AsRawHandle as AsRaw;
    } else {
        mod unsupported;
        pub use unsupported;
    }
}
