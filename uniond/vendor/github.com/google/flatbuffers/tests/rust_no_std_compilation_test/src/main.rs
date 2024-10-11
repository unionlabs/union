#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

// Include flatbuffers purely to check that it compiles in a no_std binary
#[allow(unused_imports)]
use flatbuffers;

// The rest is just no_std boilerplate

use core::alloc::{GlobalAlloc, Layout};

struct NullAllocator;
unsafe impl GlobalAlloc for NullAllocator {
    unsafe fn alloc(&self, _lt: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _lt: Layout) {
        panic!("won't deallocate: we never allocated!");
    }
}

#[global_allocator]
static A: NullAllocator = NullAllocator;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    0
}
