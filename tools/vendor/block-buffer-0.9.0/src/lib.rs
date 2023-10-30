#![no_std]
pub use generic_array;
#[cfg(feature = "block-padding")]
pub use block_padding;

use core::{slice, convert::TryInto};
use generic_array::{GenericArray, ArrayLength};
#[cfg(feature = "block-padding")]
use block_padding::{Padding, PadError};

/// Buffer for block processing of data
#[derive(Clone, Default)]
pub struct BlockBuffer<BlockSize: ArrayLength<u8>>  {
    buffer: GenericArray<u8, BlockSize>,
    pos: usize,
}

impl<BlockSize: ArrayLength<u8>> BlockBuffer<BlockSize> {
    /// Process data in `input` in blocks of size `BlockSize` using function `f`.
    #[inline]
    pub fn input_block(
        &mut self, mut input: &[u8], mut f: impl FnMut(&GenericArray<u8, BlockSize>),
    ) {
        let r = self.remaining();
        if input.len() < r {
            let n = input.len();
            self.buffer[self.pos..self.pos + n].copy_from_slice(input);
            self.pos += n;
            return;
        }
        if self.pos != 0 && input.len() >= r {
            let (l, r) = input.split_at(r);
            input = r;
            self.buffer[self.pos..].copy_from_slice(l);
            f(&self.buffer);
        }

        let mut chunks_iter = input.chunks_exact(self.size());
        for chunk in &mut chunks_iter {
            f(chunk.try_into().unwrap());
        }
        let rem = chunks_iter.remainder();

        // Copy any remaining data into the buffer.
        self.buffer[..rem.len()].copy_from_slice(rem);
        self.pos = rem.len();
    }

    /// Process data in `input` in blocks of size `BlockSize` using function `f`, which accepts
    /// slice of blocks.
    #[inline]
    pub fn input_blocks(
        &mut self, mut input: &[u8], mut f: impl FnMut(&[GenericArray<u8, BlockSize>]),
    ) {
        let r = self.remaining();
        if input.len() < r {
            let n = input.len();
            self.buffer[self.pos..self.pos + n].copy_from_slice(input);
            self.pos += n;
            return;
        }
        if self.pos != 0 && input.len() >= r {
            let (l, r) = input.split_at(r);
            input = r;
            self.buffer[self.pos..].copy_from_slice(l);
            self.pos = 0;
            f(slice::from_ref(&self.buffer));
        }

        // While we have at least a full buffer size chunks's worth of data,
        // process its data without copying into the buffer
        let n_blocks = input.len()/self.size();
        let (left, right) = input.split_at(n_blocks*self.size());
        // SAFETY: we guarantee that `blocks` does not point outside of `input` 
        let blocks = unsafe {
            slice::from_raw_parts(
                left.as_ptr() as *const GenericArray<u8, BlockSize>,
                n_blocks,
            )
        };
        f(blocks);

        // Copy remaining data into the buffer.
        let n = right.len();
        self.buffer[..n].copy_from_slice(right);
        self.pos = n;
    }

    /// Variant that doesn't flush the buffer until there's additional
    /// data to be processed. Suitable for tweakable block ciphers
    /// like Threefish that need to know whether a block is the *last*
    /// data block before processing it.
    #[inline]
    pub fn input_lazy(
        &mut self, mut input: &[u8], mut f: impl FnMut(&GenericArray<u8, BlockSize>),
    ) {
        let r = self.remaining();
        if input.len() <= r {
            let n = input.len();
            self.buffer[self.pos..self.pos + n].copy_from_slice(input);
            self.pos += n;
            return;
        }
        if self.pos != 0 && input.len() > r {
            let (l, r) = input.split_at(r);
            input = r;
            self.buffer[self.pos..].copy_from_slice(l);
            f(&self.buffer);
        }

        while input.len() > self.size() {
            let (block, r) = input.split_at(self.size());
            input = r;
            f(block.try_into().unwrap());
        }

        self.buffer[..input.len()].copy_from_slice(input);
        self.pos = input.len();
    }

    /// Pad buffer with `prefix` and make sure that internall buffer
    /// has at least `up_to` free bytes. All remaining bytes get
    /// zeroed-out.
    #[inline]
    fn digest_pad(
        &mut self, up_to: usize, mut f: impl FnMut(&GenericArray<u8, BlockSize>),
    ) {
        if self.pos == self.size() {
            f(&self.buffer);
            self.pos = 0;
        }
        self.buffer[self.pos] = 0x80;
        self.pos += 1;

        set_zero(&mut self.buffer[self.pos..]);

        if self.remaining() < up_to {
            f(&self.buffer);
            set_zero(&mut self.buffer[..self.pos]);
        }
    }

    /// Pad message with 0x80, zeros and 64-bit message length
    /// using big-endian byte order
    #[inline]
    pub fn len64_padding_be(
        &mut self, data_len: u64, mut f: impl FnMut(&GenericArray<u8, BlockSize>),
    ) {
        self.digest_pad(8, &mut f);
        let b = data_len.to_be_bytes();
        let n = self.buffer.len() - b.len();
        self.buffer[n..].copy_from_slice(&b);
        f(&self.buffer);
        self.pos = 0;
    }

    /// Pad message with 0x80, zeros and 64-bit message length
    /// using little-endian byte order
    #[inline]
    pub fn len64_padding_le(
        &mut self, data_len: u64, mut f: impl FnMut(&GenericArray<u8, BlockSize>),
    ) {
        self.digest_pad(8, &mut f);
        let b = data_len.to_le_bytes();
        let n = self.buffer.len() - b.len();
        self.buffer[n..].copy_from_slice(&b);
        f(&self.buffer);
        self.pos = 0;
    }

    /// Pad message with 0x80, zeros and 128-bit message length
    /// using big-endian byte order
    #[inline]
    pub fn len128_padding_be(
        &mut self, data_len: u128, mut f: impl FnMut(&GenericArray<u8, BlockSize>),
    ) {
        self.digest_pad(16, &mut f);
        let b = data_len.to_be_bytes();
        let n = self.buffer.len() - b.len();
        self.buffer[n..].copy_from_slice(&b);
        f(&self.buffer);
        self.pos = 0;
    }

    /// Pad message with a given padding `P`
    ///
    /// Returns `PadError` if internall buffer is full, which can only happen if
    /// `input_lazy` was used.
    #[cfg(feature = "block-padding")]
    #[inline]
    pub fn pad_with<P: Padding>(&mut self)
        -> Result<&mut GenericArray<u8, BlockSize>, PadError>
    {
        P::pad_block(&mut self.buffer[..], self.pos)?;
        self.pos = 0;
        Ok(&mut self.buffer)
    }

    /// Return size of the internall buffer in bytes
    #[inline]
    pub fn size(&self) -> usize {
        BlockSize::to_usize()
    }

    /// Return current cursor position
    #[inline]
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Return number of remaining bytes in the internall buffer
    #[inline]
    pub fn remaining(&self) -> usize {
        self.size() - self.pos
    }

    /// Reset buffer by setting cursor position to zero
    #[inline]
    pub fn reset(&mut self)  {
        self.pos = 0
    }
}

/// Sets all bytes in `dst` to zero
#[inline(always)]
fn set_zero(dst: &mut [u8]) {
    // SAFETY: we overwrite valid memory behind `dst`
    // note: loop is not used here because it produces
    // unnecessary branch which tests for zero-length slices
    unsafe {
        core::ptr::write_bytes(dst.as_mut_ptr(), 0, dst.len());
    }
}
