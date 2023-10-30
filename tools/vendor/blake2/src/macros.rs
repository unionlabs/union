macro_rules! blake2_impl {
    (
        $name:ident, $alg_name:expr, $word:ident, $vec:ident, $bytes:ident,
        $block_size:ident, $R1:expr, $R2:expr, $R3:expr, $R4:expr, $IV:expr,
        $vardoc:expr, $doc:expr,
    ) => {
        #[derive(Clone)]
        #[doc=$vardoc]
        pub struct $name {
            h: [$vec; 2],
            t: u64,
            #[cfg(feature = "reset")]
            h0: [$vec; 2],
        }

        impl $name {
            #[inline(always)]
            fn iv0() -> $vec {
                $vec::new($IV[0], $IV[1], $IV[2], $IV[3])
            }
            #[inline(always)]
            fn iv1() -> $vec {
                $vec::new($IV[4], $IV[5], $IV[6], $IV[7])
            }

            /// Creates a new context with the full set of sequential-mode parameters.
            pub fn new_with_params(
                salt: &[u8],
                persona: &[u8],
                key_size: usize,
                output_size: usize,
            ) -> Self {
                assert!(key_size <= $bytes::to_usize());
                assert!(output_size <= $bytes::to_usize());

                // The number of bytes needed to express two words.
                let length = $bytes::to_usize() / 4;
                assert!(salt.len() <= length);
                assert!(persona.len() <= length);

                // Build a parameter block
                let mut p = [0 as $word; 8];
                p[0] = 0x0101_0000 ^ ((key_size as $word) << 8) ^ (output_size as $word);

                // salt is two words long
                if salt.len() < length {
                    let mut padded_salt =
                        GenericArray::<u8, <$bytes as Div<U4>>::Output>::default();
                    for i in 0..salt.len() {
                        padded_salt[i] = salt[i];
                    }
                    p[4] = $word::from_le_bytes(padded_salt[0..length / 2].try_into().unwrap());
                    p[5] = $word::from_le_bytes(
                        padded_salt[length / 2..padded_salt.len()]
                            .try_into()
                            .unwrap(),
                    );
                } else {
                    p[4] = $word::from_le_bytes(salt[0..salt.len() / 2].try_into().unwrap());
                    p[5] =
                        $word::from_le_bytes(salt[salt.len() / 2..salt.len()].try_into().unwrap());
                }

                // persona is also two words long
                if persona.len() < length {
                    let mut padded_persona =
                        GenericArray::<u8, <$bytes as Div<U4>>::Output>::default();
                    for i in 0..persona.len() {
                        padded_persona[i] = persona[i];
                    }
                    p[6] = $word::from_le_bytes(padded_persona[0..length / 2].try_into().unwrap());
                    p[7] = $word::from_le_bytes(
                        padded_persona[length / 2..padded_persona.len()]
                            .try_into()
                            .unwrap(),
                    );
                } else {
                    p[6] = $word::from_le_bytes(persona[0..length / 2].try_into().unwrap());
                    p[7] = $word::from_le_bytes(
                        persona[length / 2..persona.len()].try_into().unwrap(),
                    );
                }

                let h = [
                    Self::iv0() ^ $vec::new(p[0], p[1], p[2], p[3]),
                    Self::iv1() ^ $vec::new(p[4], p[5], p[6], p[7]),
                ];
                $name {
                    #[cfg(feature = "reset")]
                    h0: h.clone(),
                    h,
                    t: 0,
                }
            }

            fn finalize_with_flag(
                &mut self,
                final_block: &GenericArray<u8, $block_size>,
                flag: $word,
                out: &mut Output<Self>,
            ) {
                self.compress(final_block, !0, flag);
                let buf = [self.h[0].to_le(), self.h[1].to_le()];
                out.copy_from_slice(buf.as_bytes())
            }

            fn compress(&mut self, block: &Block<Self>, f0: $word, f1: $word) {
                use $crate::consts::SIGMA;

                #[cfg_attr(not(feature = "size_opt"), inline(always))]
                fn quarter_round(v: &mut [$vec; 4], rd: u32, rb: u32, m: $vec) {
                    v[0] = v[0].wrapping_add(v[1]).wrapping_add(m.from_le());
                    v[3] = (v[3] ^ v[0]).rotate_right_const(rd);
                    v[2] = v[2].wrapping_add(v[3]);
                    v[1] = (v[1] ^ v[2]).rotate_right_const(rb);
                }

                #[cfg_attr(not(feature = "size_opt"), inline(always))]
                fn shuffle(v: &mut [$vec; 4]) {
                    v[1] = v[1].shuffle_left_1();
                    v[2] = v[2].shuffle_left_2();
                    v[3] = v[3].shuffle_left_3();
                }

                #[cfg_attr(not(feature = "size_opt"), inline(always))]
                fn unshuffle(v: &mut [$vec; 4]) {
                    v[1] = v[1].shuffle_right_1();
                    v[2] = v[2].shuffle_right_2();
                    v[3] = v[3].shuffle_right_3();
                }

                #[cfg_attr(not(feature = "size_opt"), inline(always))]
                fn round(v: &mut [$vec; 4], m: &[$word; 16], s: &[usize; 16]) {
                    quarter_round(v, $R1, $R2, $vec::gather(m, s[0], s[2], s[4], s[6]));
                    quarter_round(v, $R3, $R4, $vec::gather(m, s[1], s[3], s[5], s[7]));

                    shuffle(v);
                    quarter_round(v, $R1, $R2, $vec::gather(m, s[8], s[10], s[12], s[14]));
                    quarter_round(v, $R3, $R4, $vec::gather(m, s[9], s[11], s[13], s[15]));
                    unshuffle(v);
                }

                let mut m: [$word; 16] = Default::default();
                let n = core::mem::size_of::<$word>();
                for (v, chunk) in m.iter_mut().zip(block.chunks_exact(n)) {
                    *v = $word::from_ne_bytes(chunk.try_into().unwrap());
                }
                let h = &mut self.h;

                let t0 = self.t as $word;
                let t1 = match $bytes::to_u8() {
                    64 => 0,
                    32 => (self.t >> 32) as $word,
                    _ => unreachable!(),
                };

                let mut v = [
                    h[0],
                    h[1],
                    Self::iv0(),
                    Self::iv1() ^ $vec::new(t0, t1, f0, f1),
                ];

                round(&mut v, &m, &SIGMA[0]);
                round(&mut v, &m, &SIGMA[1]);
                round(&mut v, &m, &SIGMA[2]);
                round(&mut v, &m, &SIGMA[3]);
                round(&mut v, &m, &SIGMA[4]);
                round(&mut v, &m, &SIGMA[5]);
                round(&mut v, &m, &SIGMA[6]);
                round(&mut v, &m, &SIGMA[7]);
                round(&mut v, &m, &SIGMA[8]);
                round(&mut v, &m, &SIGMA[9]);
                if $bytes::to_u8() == 64 {
                    round(&mut v, &m, &SIGMA[0]);
                    round(&mut v, &m, &SIGMA[1]);
                }

                h[0] = h[0] ^ (v[0] ^ v[2]);
                h[1] = h[1] ^ (v[1] ^ v[3]);
            }
        }

        impl HashMarker for $name {}

        impl BlockSizeUser for $name {
            type BlockSize = $block_size;
        }

        impl BufferKindUser for $name {
            type BufferKind = Lazy;
        }

        impl UpdateCore for $name {
            #[inline]
            fn update_blocks(&mut self, blocks: &[Block<Self>]) {
                for block in blocks {
                    self.t += block.len() as u64;
                    self.compress(block, 0, 0);
                }
            }
        }

        impl OutputSizeUser for $name {
            type OutputSize = $bytes;
        }

        impl VariableOutputCore for $name {
            const TRUNC_SIDE: TruncSide = TruncSide::Left;

            #[inline]
            fn new(output_size: usize) -> Result<Self, InvalidOutputSize> {
                if output_size > Self::OutputSize::USIZE {
                    return Err(InvalidOutputSize);
                }
                Ok(Self::new_with_params(&[], &[], 0, output_size))
            }

            #[inline]
            fn finalize_variable_core(
                &mut self,
                buffer: &mut Buffer<Self>,
                out: &mut Output<Self>,
            ) {
                self.t += buffer.get_pos() as u64;
                let block = buffer.pad_with_zeros();
                self.finalize_with_flag(block, 0, out);
            }
        }

        #[cfg(feature = "reset")]
        impl Reset for $name {
            fn reset(&mut self) {
                self.h = self.h0;
                self.t = 0;
            }
        }

        impl AlgorithmName for $name {
            #[inline]
            fn write_alg_name(f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str($alg_name)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(concat!(stringify!($name), " { ... }"))
            }
        }
    };
}

macro_rules! blake2_mac_impl {
    (
        $name:ident, $hash:ty, $max_size:ty, $doc:expr
    ) => {
        #[derive(Clone)]
        #[doc=$doc]
        pub struct $name<OutSize>
        where
            OutSize: ArrayLength<u8> + IsLessOrEqual<$max_size>,
            LeEq<OutSize, $max_size>: NonZero,
        {
            core: $hash,
            buffer: LazyBuffer<<$hash as BlockSizeUser>::BlockSize>,
            #[cfg(feature = "reset")]
            key_block: Key<Self>,
            _out: PhantomData<OutSize>,
        }

        impl<OutSize> $name<OutSize>
        where
            OutSize: ArrayLength<u8> + IsLessOrEqual<$max_size>,
            LeEq<OutSize, $max_size>: NonZero,
        {
            /// Create new instance using provided key, salt, and persona.
            ///
            /// Key length should not be bigger than block size, salt and persona
            /// length should not be bigger than quarter of block size. If any
            /// of those conditions is false the method will return an error.
            #[inline]
            pub fn new_with_salt_and_personal(
                key: &[u8],
                salt: &[u8],
                persona: &[u8],
            ) -> Result<Self, InvalidLength> {
                let kl = key.len();
                let bs = <$hash as BlockSizeUser>::BlockSize::USIZE;
                let qbs = bs / 4;
                if kl > bs || salt.len() > qbs || persona.len() > qbs {
                    return Err(InvalidLength);
                }
                let mut padded_key = Block::<$hash>::default();
                padded_key[..kl].copy_from_slice(key);
                Ok(Self {
                    core: <$hash>::new_with_params(salt, persona, key.len(), OutSize::USIZE),
                    buffer: LazyBuffer::new(&padded_key),
                    #[cfg(feature = "reset")]
                    key_block: {
                        let mut t = Key::<Self>::default();
                        t[..kl].copy_from_slice(key);
                        t
                    },
                    _out: PhantomData,
                })
            }
        }

        impl<OutSize> KeySizeUser for $name<OutSize>
        where
            OutSize: ArrayLength<u8> + IsLessOrEqual<$max_size>,
            LeEq<OutSize, $max_size>: NonZero,
        {
            type KeySize = $max_size;
        }

        impl<OutSize> KeyInit for $name<OutSize>
        where
            OutSize: ArrayLength<u8> + IsLessOrEqual<$max_size>,
            LeEq<OutSize, $max_size>: NonZero,
        {
            #[inline]
            fn new(key: &Key<Self>) -> Self {
                Self::new_from_slice(key).expect("Key has correct length")
            }

            #[inline]
            fn new_from_slice(key: &[u8]) -> Result<Self, InvalidLength> {
                let kl = key.len();
                if kl > <Self as KeySizeUser>::KeySize::USIZE {
                    return Err(InvalidLength);
                }
                let mut padded_key = Block::<$hash>::default();
                padded_key[..kl].copy_from_slice(key);
                Ok(Self {
                    core: <$hash>::new_with_params(&[], &[], key.len(), OutSize::USIZE),
                    buffer: LazyBuffer::new(&padded_key),
                    #[cfg(feature = "reset")]
                    key_block: {
                        let mut t = Key::<Self>::default();
                        t[..kl].copy_from_slice(key);
                        t
                    },
                    _out: PhantomData,
                })
            }
        }

        impl<OutSize> Update for $name<OutSize>
        where
            OutSize: ArrayLength<u8> + IsLessOrEqual<$max_size>,
            LeEq<OutSize, $max_size>: NonZero,
        {
            #[inline]
            fn update(&mut self, input: &[u8]) {
                let Self { core, buffer, .. } = self;
                buffer.digest_blocks(input, |blocks| core.update_blocks(blocks));
            }
        }

        impl<OutSize> OutputSizeUser for $name<OutSize>
        where
            OutSize: ArrayLength<u8> + IsLessOrEqual<$max_size> + 'static,
            LeEq<OutSize, $max_size>: NonZero,
        {
            type OutputSize = OutSize;
        }

        impl<OutSize> FixedOutput for $name<OutSize>
        where
            OutSize: ArrayLength<u8> + IsLessOrEqual<$max_size> + 'static,
            LeEq<OutSize, $max_size>: NonZero,
        {
            #[inline]
            fn finalize_into(mut self, out: &mut Output<Self>) {
                let Self { core, buffer, .. } = &mut self;
                let mut full_res = Default::default();
                core.finalize_variable_core(buffer, &mut full_res);
                out.copy_from_slice(&full_res[..OutSize::USIZE]);
            }
        }

        #[cfg(feature = "reset")]
        impl<OutSize> Reset for $name<OutSize>
        where
            OutSize: ArrayLength<u8> + IsLessOrEqual<$max_size>,
            LeEq<OutSize, $max_size>: NonZero,
        {
            fn reset(&mut self) {
                self.core.reset();
                let kl = self.key_block.len();
                let mut padded_key = Block::<$hash>::default();
                padded_key[..kl].copy_from_slice(&self.key_block);
                self.buffer = LazyBuffer::new(&padded_key);
            }
        }

        #[cfg(feature = "reset")]
        impl<OutSize> FixedOutputReset for $name<OutSize>
        where
            OutSize: ArrayLength<u8> + IsLessOrEqual<$max_size>,
            LeEq<OutSize, $max_size>: NonZero,
        {
            #[inline]
            fn finalize_into_reset(&mut self, out: &mut Output<Self>) {
                let Self { core, buffer, .. } = self;
                let mut full_res = Default::default();
                core.finalize_variable_core(buffer, &mut full_res);
                out.copy_from_slice(&full_res[..OutSize::USIZE]);
                self.reset();
            }
        }

        impl<OutSize> MacMarker for $name<OutSize>
        where
            OutSize: ArrayLength<u8> + IsLessOrEqual<$max_size>,
            LeEq<OutSize, $max_size>: NonZero,
        {
        }

        impl<OutSize> fmt::Debug for $name<OutSize>
        where
            OutSize: ArrayLength<u8> + IsLessOrEqual<$max_size>,
            LeEq<OutSize, $max_size>: NonZero,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}{} {{ ... }}", stringify!($name), OutSize::USIZE)
            }
        }
    };
}
