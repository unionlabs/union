#![allow(clippy::many_single_char_names, clippy::unreadable_literal)]
use core::convert::TryInto;

const RC: [u32; 64] = [
    // round 1
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    // round 2
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    // round 3
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    // round 4
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];

#[inline(always)]
fn op_f(w: u32, x: u32, y: u32, z: u32, m: u32, c: u32, s: u32) -> u32 {
    ((x & y) | (!x & z))
        .wrapping_add(w)
        .wrapping_add(m)
        .wrapping_add(c)
        .rotate_left(s)
        .wrapping_add(x)
}
#[inline(always)]
fn op_g(w: u32, x: u32, y: u32, z: u32, m: u32, c: u32, s: u32) -> u32 {
    ((x & z) | (y & !z))
        .wrapping_add(w)
        .wrapping_add(m)
        .wrapping_add(c)
        .rotate_left(s)
        .wrapping_add(x)
}

#[inline(always)]
fn op_h(w: u32, x: u32, y: u32, z: u32, m: u32, c: u32, s: u32) -> u32 {
    (x ^ y ^ z)
        .wrapping_add(w)
        .wrapping_add(m)
        .wrapping_add(c)
        .rotate_left(s)
        .wrapping_add(x)
}

#[inline(always)]
fn op_i(w: u32, x: u32, y: u32, z: u32, m: u32, c: u32, s: u32) -> u32 {
    (y ^ (x | !z))
        .wrapping_add(w)
        .wrapping_add(m)
        .wrapping_add(c)
        .rotate_left(s)
        .wrapping_add(x)
}

#[inline]
pub fn compress_block(state: &mut [u32; 4], input: &[u8; 64]) {
    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];

    let mut data = [0u32; 16];
    for (o, chunk) in data.iter_mut().zip(input.chunks_exact(4)) {
        *o = u32::from_le_bytes(chunk.try_into().unwrap());
    }

    // round 1
    a = op_f(a, b, c, d, data[0], RC[0], 7);
    d = op_f(d, a, b, c, data[1], RC[1], 12);
    c = op_f(c, d, a, b, data[2], RC[2], 17);
    b = op_f(b, c, d, a, data[3], RC[3], 22);

    a = op_f(a, b, c, d, data[4], RC[4], 7);
    d = op_f(d, a, b, c, data[5], RC[5], 12);
    c = op_f(c, d, a, b, data[6], RC[6], 17);
    b = op_f(b, c, d, a, data[7], RC[7], 22);

    a = op_f(a, b, c, d, data[8], RC[8], 7);
    d = op_f(d, a, b, c, data[9], RC[9], 12);
    c = op_f(c, d, a, b, data[10], RC[10], 17);
    b = op_f(b, c, d, a, data[11], RC[11], 22);

    a = op_f(a, b, c, d, data[12], RC[12], 7);
    d = op_f(d, a, b, c, data[13], RC[13], 12);
    c = op_f(c, d, a, b, data[14], RC[14], 17);
    b = op_f(b, c, d, a, data[15], RC[15], 22);

    // round 2
    a = op_g(a, b, c, d, data[1], RC[16], 5);
    d = op_g(d, a, b, c, data[6], RC[17], 9);
    c = op_g(c, d, a, b, data[11], RC[18], 14);
    b = op_g(b, c, d, a, data[0], RC[19], 20);

    a = op_g(a, b, c, d, data[5], RC[20], 5);
    d = op_g(d, a, b, c, data[10], RC[21], 9);
    c = op_g(c, d, a, b, data[15], RC[22], 14);
    b = op_g(b, c, d, a, data[4], RC[23], 20);

    a = op_g(a, b, c, d, data[9], RC[24], 5);
    d = op_g(d, a, b, c, data[14], RC[25], 9);
    c = op_g(c, d, a, b, data[3], RC[26], 14);
    b = op_g(b, c, d, a, data[8], RC[27], 20);

    a = op_g(a, b, c, d, data[13], RC[28], 5);
    d = op_g(d, a, b, c, data[2], RC[29], 9);
    c = op_g(c, d, a, b, data[7], RC[30], 14);
    b = op_g(b, c, d, a, data[12], RC[31], 20);

    // round 3
    a = op_h(a, b, c, d, data[5], RC[32], 4);
    d = op_h(d, a, b, c, data[8], RC[33], 11);
    c = op_h(c, d, a, b, data[11], RC[34], 16);
    b = op_h(b, c, d, a, data[14], RC[35], 23);

    a = op_h(a, b, c, d, data[1], RC[36], 4);
    d = op_h(d, a, b, c, data[4], RC[37], 11);
    c = op_h(c, d, a, b, data[7], RC[38], 16);
    b = op_h(b, c, d, a, data[10], RC[39], 23);

    a = op_h(a, b, c, d, data[13], RC[40], 4);
    d = op_h(d, a, b, c, data[0], RC[41], 11);
    c = op_h(c, d, a, b, data[3], RC[42], 16);
    b = op_h(b, c, d, a, data[6], RC[43], 23);

    a = op_h(a, b, c, d, data[9], RC[44], 4);
    d = op_h(d, a, b, c, data[12], RC[45], 11);
    c = op_h(c, d, a, b, data[15], RC[46], 16);
    b = op_h(b, c, d, a, data[2], RC[47], 23);

    // round 4
    a = op_i(a, b, c, d, data[0], RC[48], 6);
    d = op_i(d, a, b, c, data[7], RC[49], 10);
    c = op_i(c, d, a, b, data[14], RC[50], 15);
    b = op_i(b, c, d, a, data[5], RC[51], 21);

    a = op_i(a, b, c, d, data[12], RC[52], 6);
    d = op_i(d, a, b, c, data[3], RC[53], 10);
    c = op_i(c, d, a, b, data[10], RC[54], 15);
    b = op_i(b, c, d, a, data[1], RC[55], 21);

    a = op_i(a, b, c, d, data[8], RC[56], 6);
    d = op_i(d, a, b, c, data[15], RC[57], 10);
    c = op_i(c, d, a, b, data[6], RC[58], 15);
    b = op_i(b, c, d, a, data[13], RC[59], 21);

    a = op_i(a, b, c, d, data[4], RC[60], 6);
    d = op_i(d, a, b, c, data[11], RC[61], 10);
    c = op_i(c, d, a, b, data[2], RC[62], 15);
    b = op_i(b, c, d, a, data[9], RC[63], 21);

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
}

#[inline]
pub fn compress(state: &mut [u32; 4], blocks: &[[u8; 64]]) {
    for block in blocks {
        compress_block(state, block)
    }
}
