#[cfg(no_str_strip_prefix)] // rustc <1.45
pub(crate) trait StripPrefixExt {
    fn strip_prefix(&self, ch: char) -> Option<&str>;
}

#[cfg(no_str_strip_prefix)]
impl StripPrefixExt for str {
    fn strip_prefix(&self, ch: char) -> Option<&str> {
        if self.starts_with(ch) {
            Some(&self[ch.len_utf8()..])
        } else {
            None
        }
    }
}

pub(crate) use crate::alloc::vec::Vec;

#[cfg(no_alloc_crate)] // rustc <1.36
pub(crate) mod alloc {
    pub use std::vec;

    pub mod alloc {
        use std::mem;

        pub struct Layout {
            size: usize,
        }

        impl Layout {
            pub unsafe fn from_size_align_unchecked(size: usize, align: usize) -> Self {
                assert_eq!(align, 2);
                Layout { size }
            }
        }

        pub unsafe fn alloc(layout: Layout) -> *mut u8 {
            let len_u16 = (layout.size + 1) / 2;
            let mut vec = Vec::new();
            vec.reserve_exact(len_u16);
            let ptr: *mut u16 = vec.as_mut_ptr();
            mem::forget(vec);
            ptr as *mut u8
        }

        pub unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
            let len_u16 = (layout.size + 1) / 2;
            unsafe { Vec::from_raw_parts(ptr as *mut u16, 0, len_u16) };
        }
    }
}
