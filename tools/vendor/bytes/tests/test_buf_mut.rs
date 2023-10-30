#![warn(rust_2018_idioms)]

use bytes::buf::UninitSlice;
use bytes::{BufMut, BytesMut};
use core::fmt::Write;
use core::usize;

#[test]
fn test_vec_as_mut_buf() {
    let mut buf = Vec::with_capacity(64);

    assert_eq!(buf.remaining_mut(), isize::MAX as usize);

    assert!(buf.chunk_mut().len() >= 64);

    buf.put(&b"zomg"[..]);

    assert_eq!(&buf, b"zomg");

    assert_eq!(buf.remaining_mut(), isize::MAX as usize - 4);
    assert_eq!(buf.capacity(), 64);

    for _ in 0..16 {
        buf.put(&b"zomg"[..]);
    }

    assert_eq!(buf.len(), 68);
}

#[test]
fn test_vec_put_bytes() {
    let mut buf = Vec::new();
    buf.push(17);
    buf.put_bytes(19, 2);
    assert_eq!([17, 19, 19], &buf[..]);
}

#[test]
fn test_put_u8() {
    let mut buf = Vec::with_capacity(8);
    buf.put_u8(33);
    assert_eq!(b"\x21", &buf[..]);
}

#[test]
fn test_put_u16() {
    let mut buf = Vec::with_capacity(8);
    buf.put_u16(8532);
    assert_eq!(b"\x21\x54", &buf[..]);

    buf.clear();
    buf.put_u16_le(8532);
    assert_eq!(b"\x54\x21", &buf[..]);
}

#[test]
fn test_put_int() {
    let mut buf = Vec::with_capacity(8);
    buf.put_int(0x1020304050607080, 3);
    assert_eq!(b"\x60\x70\x80", &buf[..]);
}

#[test]
#[should_panic]
fn test_put_int_nbytes_overflow() {
    let mut buf = Vec::with_capacity(8);
    buf.put_int(0x1020304050607080, 9);
}

#[test]
fn test_put_int_le() {
    let mut buf = Vec::with_capacity(8);
    buf.put_int_le(0x1020304050607080, 3);
    assert_eq!(b"\x80\x70\x60", &buf[..]);
}

#[test]
#[should_panic]
fn test_put_int_le_nbytes_overflow() {
    let mut buf = Vec::with_capacity(8);
    buf.put_int_le(0x1020304050607080, 9);
}

#[test]
#[should_panic(expected = "cannot advance")]
fn test_vec_advance_mut() {
    // Verify fix for #354
    let mut buf = Vec::with_capacity(8);
    unsafe {
        buf.advance_mut(12);
    }
}

#[test]
fn test_clone() {
    let mut buf = BytesMut::with_capacity(100);
    buf.write_str("this is a test").unwrap();
    let buf2 = buf.clone();

    buf.write_str(" of our emergency broadcast system").unwrap();
    assert!(buf != buf2);
}

#[test]
fn test_mut_slice() {
    let mut v = vec![0, 0, 0, 0];
    let mut s = &mut v[..];
    s.put_u32(42);

    assert_eq!(s.len(), 0);
    assert_eq!(&v, &[0, 0, 0, 42]);
}

#[test]
fn test_slice_put_bytes() {
    let mut v = [0, 0, 0, 0];
    let mut s = &mut v[..];
    s.put_u8(17);
    s.put_bytes(19, 2);
    assert_eq!(1, s.remaining_mut());
    assert_eq!(&[17, 19, 19, 0], &v[..]);
}

#[test]
fn test_deref_bufmut_forwards() {
    struct Special;

    unsafe impl BufMut for Special {
        fn remaining_mut(&self) -> usize {
            unreachable!("remaining_mut");
        }

        fn chunk_mut(&mut self) -> &mut UninitSlice {
            unreachable!("chunk_mut");
        }

        unsafe fn advance_mut(&mut self, _: usize) {
            unreachable!("advance");
        }

        fn put_u8(&mut self, _: u8) {
            // specialized!
        }
    }

    // these should all use the specialized method
    Special.put_u8(b'x');
    (&mut Special as &mut dyn BufMut).put_u8(b'x');
    (Box::new(Special) as Box<dyn BufMut>).put_u8(b'x');
    Box::new(Special).put_u8(b'x');
}

#[test]
#[should_panic]
fn write_byte_panics_if_out_of_bounds() {
    let mut data = [b'b', b'a', b'r'];

    let slice = unsafe { UninitSlice::from_raw_parts_mut(data.as_mut_ptr(), 3) };
    slice.write_byte(4, b'f');
}

#[test]
#[should_panic]
fn copy_from_slice_panics_if_different_length_1() {
    let mut data = [b'b', b'a', b'r'];

    let slice = unsafe { UninitSlice::from_raw_parts_mut(data.as_mut_ptr(), 3) };
    slice.copy_from_slice(b"a");
}

#[test]
#[should_panic]
fn copy_from_slice_panics_if_different_length_2() {
    let mut data = [b'b', b'a', b'r'];

    let slice = unsafe { UninitSlice::from_raw_parts_mut(data.as_mut_ptr(), 3) };
    slice.copy_from_slice(b"abcd");
}
