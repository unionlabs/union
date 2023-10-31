use std::alloc::{GlobalAlloc, Layout, System};
use std::{mem, ptr};

use bytes::{Buf, Bytes};

#[global_allocator]
static LEDGER: Ledger = Ledger;

struct Ledger;

const USIZE_SIZE: usize = mem::size_of::<usize>();

unsafe impl GlobalAlloc for Ledger {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() == 1 && layout.size() > 0 {
            // Allocate extra space to stash a record of
            // how much space there was.
            let orig_size = layout.size();
            let size = orig_size + USIZE_SIZE;
            let new_layout = match Layout::from_size_align(size, 1) {
                Ok(layout) => layout,
                Err(_err) => return ptr::null_mut(),
            };
            let ptr = System.alloc(new_layout);
            if !ptr.is_null() {
                (ptr as *mut usize).write(orig_size);
                let ptr = ptr.offset(USIZE_SIZE as isize);
                ptr
            } else {
                ptr
            }
        } else {
            System.alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.align() == 1 && layout.size() > 0 {
            let off_ptr = (ptr as *mut usize).offset(-1);
            let orig_size = off_ptr.read();
            if orig_size != layout.size() {
                panic!(
                    "bad dealloc: alloc size was {}, dealloc size is {}",
                    orig_size,
                    layout.size()
                );
            }

            let new_layout = match Layout::from_size_align(layout.size() + USIZE_SIZE, 1) {
                Ok(layout) => layout,
                Err(_err) => std::process::abort(),
            };
            System.dealloc(off_ptr as *mut u8, new_layout);
        } else {
            System.dealloc(ptr, layout);
        }
    }
}
#[test]
fn test_bytes_advance() {
    let mut bytes = Bytes::from(vec![10, 20, 30]);
    bytes.advance(1);
    drop(bytes);
}

#[test]
fn test_bytes_truncate() {
    let mut bytes = Bytes::from(vec![10, 20, 30]);
    bytes.truncate(2);
    drop(bytes);
}

#[test]
fn test_bytes_truncate_and_advance() {
    let mut bytes = Bytes::from(vec![10, 20, 30]);
    bytes.truncate(2);
    bytes.advance(1);
    drop(bytes);
}
