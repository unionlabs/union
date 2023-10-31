// Vendored from libstd:
// https://github.com/rust-lang/rust/blob/1.57.0/library/core/src/hash/sip.rs
//
// TODO: maybe depend on a hasher from crates.io if this becomes annoying to
// maintain, or change this to a simpler one.

#![cfg(not(feature = "std"))]

use core::cmp;
use core::hash::Hasher;
use core::mem;
use core::ptr;

/// An implementation of SipHash 1-3.
///
/// This is currently the default hashing function used by standard library
/// (e.g., `collections::HashMap` uses it by default).
///
/// See: <https://131002.net/siphash>
pub struct SipHasher13 {
    k0: u64,
    k1: u64,
    length: usize, // how many bytes we've processed
    state: State,  // hash State
    tail: u64,     // unprocessed bytes le
    ntail: usize,  // how many bytes in tail are valid
}

#[derive(Clone, Copy)]
#[repr(C)]
struct State {
    // v0, v2 and v1, v3 show up in pairs in the algorithm,
    // and simd implementations of SipHash will use vectors
    // of v02 and v13. By placing them in this order in the struct,
    // the compiler can pick up on just a few simd optimizations by itself.
    v0: u64,
    v2: u64,
    v1: u64,
    v3: u64,
}

macro_rules! compress {
    ($state:expr) => {
        compress!($state.v0, $state.v1, $state.v2, $state.v3)
    };
    ($v0:expr, $v1:expr, $v2:expr, $v3:expr) => {
        $v0 = $v0.wrapping_add($v1);
        $v1 = $v1.rotate_left(13);
        $v1 ^= $v0;
        $v0 = $v0.rotate_left(32);
        $v2 = $v2.wrapping_add($v3);
        $v3 = $v3.rotate_left(16);
        $v3 ^= $v2;
        $v0 = $v0.wrapping_add($v3);
        $v3 = $v3.rotate_left(21);
        $v3 ^= $v0;
        $v2 = $v2.wrapping_add($v1);
        $v1 = $v1.rotate_left(17);
        $v1 ^= $v2;
        $v2 = $v2.rotate_left(32);
    };
}

/// Loads an integer of the desired type from a byte stream, in LE order. Uses
/// `copy_nonoverlapping` to let the compiler generate the most efficient way
/// to load it from a possibly unaligned address.
///
/// Unsafe because: unchecked indexing at i..i+size_of(int_ty)
macro_rules! load_int_le {
    ($buf:expr, $i:expr, $int_ty:ident) => {{
        debug_assert!($i + mem::size_of::<$int_ty>() <= $buf.len());
        let mut data = 0 as $int_ty;
        ptr::copy_nonoverlapping(
            $buf.as_ptr().add($i),
            &mut data as *mut _ as *mut u8,
            mem::size_of::<$int_ty>(),
        );
        data.to_le()
    }};
}

/// Loads a u64 using up to 7 bytes of a byte slice. It looks clumsy but the
/// `copy_nonoverlapping` calls that occur (via `load_int_le!`) all have fixed
/// sizes and avoid calling `memcpy`, which is good for speed.
///
/// Unsafe because: unchecked indexing at start..start+len
unsafe fn u8to64_le(buf: &[u8], start: usize, len: usize) -> u64 {
    debug_assert!(len < 8);
    let mut i = 0; // current byte index (from LSB) in the output u64
    let mut out = 0;
    if i + 3 < len {
        // SAFETY: `i` cannot be greater than `len`, and the caller must guarantee
        // that the index start..start+len is in bounds.
        out = unsafe { load_int_le!(buf, start + i, u32) } as u64;
        i += 4;
    }
    if i + 1 < len {
        // SAFETY: same as above.
        out |= (unsafe { load_int_le!(buf, start + i, u16) } as u64) << (i * 8);
        i += 2
    }
    if i < len {
        // SAFETY: same as above.
        out |= (unsafe { *buf.get_unchecked(start + i) } as u64) << (i * 8);
        i += 1;
    }
    debug_assert_eq!(i, len);
    out
}

impl SipHasher13 {
    /// Creates a new `SipHasher13` with the two initial keys set to 0.
    pub fn new() -> Self {
        Self::new_with_keys(0, 0)
    }

    /// Creates a `SipHasher13` that is keyed off the provided keys.
    fn new_with_keys(key0: u64, key1: u64) -> Self {
        let mut state = SipHasher13 {
            k0: key0,
            k1: key1,
            length: 0,
            state: State {
                v0: 0,
                v1: 0,
                v2: 0,
                v3: 0,
            },
            tail: 0,
            ntail: 0,
        };
        state.reset();
        state
    }

    fn reset(&mut self) {
        self.length = 0;
        self.state.v0 = self.k0 ^ 0x736f6d6570736575;
        self.state.v1 = self.k1 ^ 0x646f72616e646f6d;
        self.state.v2 = self.k0 ^ 0x6c7967656e657261;
        self.state.v3 = self.k1 ^ 0x7465646279746573;
        self.ntail = 0;
    }
}

impl Hasher for SipHasher13 {
    // Note: no integer hashing methods (`write_u*`, `write_i*`) are defined
    // for this type. We could add them, copy the `short_write` implementation
    // in librustc_data_structures/sip128.rs, and add `write_u*`/`write_i*`
    // methods to `SipHasher`, `SipHasher13`, and `DefaultHasher`. This would
    // greatly speed up integer hashing by those hashers, at the cost of
    // slightly slowing down compile speeds on some benchmarks. See #69152 for
    // details.
    fn write(&mut self, msg: &[u8]) {
        let length = msg.len();
        self.length += length;

        let mut needed = 0;

        if self.ntail != 0 {
            needed = 8 - self.ntail;
            // SAFETY: `cmp::min(length, needed)` is guaranteed to not be over `length`
            self.tail |= unsafe { u8to64_le(msg, 0, cmp::min(length, needed)) } << (8 * self.ntail);
            if length < needed {
                self.ntail += length;
                return;
            } else {
                self.state.v3 ^= self.tail;
                Sip13Rounds::c_rounds(&mut self.state);
                self.state.v0 ^= self.tail;
                self.ntail = 0;
            }
        }

        // Buffered tail is now flushed, process new input.
        let len = length - needed;
        let left = len & 0x7; // len % 8

        let mut i = needed;
        while i < len - left {
            // SAFETY: because `len - left` is the biggest multiple of 8 under
            // `len`, and because `i` starts at `needed` where `len` is `length - needed`,
            // `i + 8` is guaranteed to be less than or equal to `length`.
            let mi = unsafe { load_int_le!(msg, i, u64) };

            self.state.v3 ^= mi;
            Sip13Rounds::c_rounds(&mut self.state);
            self.state.v0 ^= mi;

            i += 8;
        }

        // SAFETY: `i` is now `needed + len.div_euclid(8) * 8`,
        // so `i + left` = `needed + len` = `length`, which is by
        // definition equal to `msg.len()`.
        self.tail = unsafe { u8to64_le(msg, i, left) };
        self.ntail = left;
    }

    fn finish(&self) -> u64 {
        let mut state = self.state;

        let b: u64 = ((self.length as u64 & 0xff) << 56) | self.tail;

        state.v3 ^= b;
        Sip13Rounds::c_rounds(&mut state);
        state.v0 ^= b;

        state.v2 ^= 0xff;
        Sip13Rounds::d_rounds(&mut state);

        state.v0 ^ state.v1 ^ state.v2 ^ state.v3
    }
}

struct Sip13Rounds;

impl Sip13Rounds {
    fn c_rounds(state: &mut State) {
        compress!(state);
    }

    fn d_rounds(state: &mut State) {
        compress!(state);
        compress!(state);
        compress!(state);
    }
}
