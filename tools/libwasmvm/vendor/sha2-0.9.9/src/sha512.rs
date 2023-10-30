//! SHA-512
use crate::consts::{H384, H512, H512_TRUNC_224, H512_TRUNC_256, STATE_LEN};
use block_buffer::BlockBuffer;
use core::slice::from_ref;
use digest::consts::{U128, U28, U32, U48, U64};
use digest::generic_array::GenericArray;
use digest::{BlockInput, FixedOutputDirty, Reset, Update};

type BlockSize = U128;

/// Structure that keeps state of the Sha-512 operation and
/// contains the logic necessary to perform the final calculations.
#[derive(Clone)]
struct Engine512 {
    len: u128,
    buffer: BlockBuffer<BlockSize>,
    state: [u64; 8],
}

impl Engine512 {
    fn new(h: &[u64; STATE_LEN]) -> Engine512 {
        Engine512 {
            len: 0,
            buffer: Default::default(),
            state: *h,
        }
    }

    fn update(&mut self, input: &[u8]) {
        self.len += (input.len() as u128) << 3;
        let s = &mut self.state;
        self.buffer.input_blocks(input, |b| compress512(s, b));
    }

    fn finish(&mut self) {
        let s = &mut self.state;
        self.buffer
            .len128_padding_be(self.len, |d| compress512(s, from_ref(d)));
    }

    fn reset(&mut self, h: &[u64; STATE_LEN]) {
        self.len = 0;
        self.buffer.reset();
        self.state = *h;
    }
}

/// The SHA-512 hash algorithm with the SHA-512 initial hash value.
#[derive(Clone)]
pub struct Sha512 {
    engine: Engine512,
}

impl Default for Sha512 {
    fn default() -> Self {
        Sha512 {
            engine: Engine512::new(&H512),
        }
    }
}

impl BlockInput for Sha512 {
    type BlockSize = BlockSize;
}

impl Update for Sha512 {
    fn update(&mut self, input: impl AsRef<[u8]>) {
        self.engine.update(input.as_ref());
    }
}

impl FixedOutputDirty for Sha512 {
    type OutputSize = U64;

    fn finalize_into_dirty(&mut self, out: &mut digest::Output<Self>) {
        self.engine.finish();
        let s = self.engine.state;
        for (chunk, v) in out.chunks_exact_mut(8).zip(s.iter()) {
            chunk.copy_from_slice(&v.to_be_bytes());
        }
    }
}

impl Reset for Sha512 {
    fn reset(&mut self) {
        self.engine.reset(&H512);
    }
}

/// The SHA-512 hash algorithm with the SHA-384 initial hash value. The result
/// is truncated to 384 bits.
#[derive(Clone)]
pub struct Sha384 {
    engine: Engine512,
}

impl Default for Sha384 {
    fn default() -> Self {
        Sha384 {
            engine: Engine512::new(&H384),
        }
    }
}

impl BlockInput for Sha384 {
    type BlockSize = BlockSize;
}

impl Update for Sha384 {
    fn update(&mut self, input: impl AsRef<[u8]>) {
        self.engine.update(input.as_ref());
    }
}

impl FixedOutputDirty for Sha384 {
    type OutputSize = U48;

    fn finalize_into_dirty(&mut self, out: &mut digest::Output<Self>) {
        self.engine.finish();
        let s = &self.engine.state[..6];
        for (chunk, v) in out.chunks_exact_mut(8).zip(s.iter()) {
            chunk.copy_from_slice(&v.to_be_bytes());
        }
    }
}

impl Reset for Sha384 {
    fn reset(&mut self) {
        self.engine.reset(&H384);
    }
}

/// The SHA-512 hash algorithm with the SHA-512/256 initial hash value. The
/// result is truncated to 256 bits.
#[derive(Clone)]
pub struct Sha512Trunc256 {
    engine: Engine512,
}

impl Default for Sha512Trunc256 {
    fn default() -> Self {
        Sha512Trunc256 {
            engine: Engine512::new(&H512_TRUNC_256),
        }
    }
}

impl BlockInput for Sha512Trunc256 {
    type BlockSize = BlockSize;
}

impl Update for Sha512Trunc256 {
    fn update(&mut self, input: impl AsRef<[u8]>) {
        self.engine.update(input.as_ref());
    }
}

impl FixedOutputDirty for Sha512Trunc256 {
    type OutputSize = U32;

    fn finalize_into_dirty(&mut self, out: &mut digest::Output<Self>) {
        self.engine.finish();
        let s = &self.engine.state[..4];
        for (chunk, v) in out.chunks_exact_mut(8).zip(s.iter()) {
            chunk.copy_from_slice(&v.to_be_bytes());
        }
    }
}

impl Reset for Sha512Trunc256 {
    fn reset(&mut self) {
        self.engine.reset(&H512_TRUNC_256);
    }
}

/// The SHA-512 hash algorithm with the SHA-512/224 initial hash value.
/// The result is truncated to 224 bits.
#[derive(Clone)]
pub struct Sha512Trunc224 {
    engine: Engine512,
}

impl Default for Sha512Trunc224 {
    fn default() -> Self {
        Sha512Trunc224 {
            engine: Engine512::new(&H512_TRUNC_224),
        }
    }
}

impl BlockInput for Sha512Trunc224 {
    type BlockSize = BlockSize;
}

impl Update for Sha512Trunc224 {
    fn update(&mut self, input: impl AsRef<[u8]>) {
        self.engine.update(input.as_ref());
    }
}

impl FixedOutputDirty for Sha512Trunc224 {
    type OutputSize = U28;

    fn finalize_into_dirty(&mut self, out: &mut digest::Output<Self>) {
        self.engine.finish();
        let s = &self.engine.state;
        for (chunk, v) in out.chunks_exact_mut(8).zip(s[..3].iter()) {
            chunk.copy_from_slice(&v.to_be_bytes());
        }
        out[24..28].copy_from_slice(&s[3].to_be_bytes()[..4]);
    }
}

impl Reset for Sha512Trunc224 {
    fn reset(&mut self) {
        self.engine.reset(&H512_TRUNC_224);
    }
}

opaque_debug::implement!(Sha384);
opaque_debug::implement!(Sha512);
opaque_debug::implement!(Sha512Trunc224);
opaque_debug::implement!(Sha512Trunc256);

digest::impl_write!(Sha384);
digest::impl_write!(Sha512);
digest::impl_write!(Sha512Trunc224);
digest::impl_write!(Sha512Trunc256);

cfg_if::cfg_if! {
    if #[cfg(feature = "force-soft")] {
        mod soft;
        use soft::compress;
    } else if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        #[cfg(not(feature = "asm"))]
        mod soft;
        #[cfg(feature = "asm")]
        mod soft {
            pub(crate) fn compress(state: &mut [u64; 8], blocks: &[[u8; 128]]) {
                sha2_asm::compress512(state, blocks);
            }
        }
        mod x86;
        use x86::compress;
    } else {
        mod soft;
        use soft::compress;
    }
}

/// Raw SHA-512 compression function.
///
/// This is a low-level "hazmat" API which provides direct access to the core
/// functionality of SHA-512.
#[cfg_attr(docsrs, doc(cfg(feature = "compress")))]
pub fn compress512(state: &mut [u64; 8], blocks: &[GenericArray<u8, U128>]) {
    // SAFETY: GenericArray<u8, U128> and [u8; 128] have
    // exactly the same memory layout
    #[allow(unsafe_code)]
    let blocks: &[[u8; 128]] = unsafe { &*(blocks as *const _ as *const [[u8; 128]]) };
    compress(state, blocks)
}
