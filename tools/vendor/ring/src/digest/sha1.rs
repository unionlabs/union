// Copyright 2015-2016 Brian Smith.
// Copyright 2016 Simon Sapin.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

use super::sha2::{ch, maj, Word};
use crate::c;
use core::{convert::TryInto, num::Wrapping};

pub const BLOCK_LEN: usize = 512 / 8;
pub const CHAINING_LEN: usize = 160 / 8;
pub const OUTPUT_LEN: usize = 160 / 8;
const CHAINING_WORDS: usize = CHAINING_LEN / 4;

type W32 = Wrapping<u32>;

// FIPS 180-4 4.1.1
#[inline]
fn parity(x: W32, y: W32, z: W32) -> W32 {
    x ^ y ^ z
}

type State = [W32; CHAINING_WORDS];
const ROUNDS: usize = 80;

pub(super) extern "C" fn block_data_order(
    state: &mut super::State,
    data: *const u8,
    num: c::size_t,
) {
    let state = unsafe { &mut state.as32 };
    let state: &mut State = (&mut state[..CHAINING_WORDS]).try_into().unwrap();
    let data = data as *const [<W32 as Word>::InputBytes; 16];
    let blocks = unsafe { core::slice::from_raw_parts(data, num) };
    *state = block_data_order_(*state, blocks)
}

#[inline]
#[rustfmt::skip]
fn block_data_order_(mut H: State, M: &[[<W32 as Word>::InputBytes; 16]]) -> State {
    for M in M {
        // FIPS 180-4 6.1.2 Step 1
        let mut W: [W32; ROUNDS] = [W32::ZERO; ROUNDS];
        for t in 0..16 {
            W[t] = W32::from_be_bytes(M[t]);
        }
        for t in 16..ROUNDS {
            let wt = W[t - 3] ^ W[t - 8] ^ W[t - 14] ^ W[t - 16];
            W[t] = rotl(wt, 1);
        }

        // FIPS 180-4 6.1.2 Step 2
        let a = H[0];
        let b = H[1];
        let c = H[2];
        let d = H[3];
        let e = H[4];

        // FIPS 180-4 6.1.2 Step 3 with constants and functions from FIPS 180-4 {4.1.1, 4.2.1}
        let (a, b, c, d, e) = step3(a, b, c, d, e, W[ 0..20].try_into().unwrap(), Wrapping(0x5a827999), ch);
        let (a, b, c, d, e) = step3(a, b, c, d, e, W[20..40].try_into().unwrap(), Wrapping(0x6ed9eba1), parity);
        let (a, b, c, d, e) = step3(a, b, c, d, e, W[40..60].try_into().unwrap(), Wrapping(0x8f1bbcdc), maj);
        let (a, b, c, d, e) = step3(a, b, c, d, e, W[60..80].try_into().unwrap(), Wrapping(0xca62c1d6), parity);

        // FIPS 180-4 6.1.2 Step 4
        H[0] += a;
        H[1] += b;
        H[2] += c;
        H[3] += d;
        H[4] += e;
    }

    H
}

#[inline(always)]
fn step3(
    mut a: W32,
    mut b: W32,
    mut c: W32,
    mut d: W32,
    mut e: W32,
    W: [W32; 20],
    k: W32,
    f: impl Fn(W32, W32, W32) -> W32,
) -> (W32, W32, W32, W32, W32) {
    for W_t in W.iter() {
        let T = rotl(a, 5) + f(b, c, d) + e + k + W_t;
        e = d;
        d = c;
        c = rotl(b, 30);
        b = a;
        a = T;
    }
    (a, b, c, d, e)
}

#[inline(always)]
fn rotl(x: W32, n: u32) -> W32 {
    Wrapping(x.0.rotate_left(n))
}
