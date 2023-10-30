// See the README in this directory for an explanation of the Teddy algorithm.

use std::cmp;
use std::collections::BTreeMap;
use std::fmt;

use crate::packed::pattern::{PatternID, Patterns};
use crate::packed::teddy::Teddy;

/// A builder for constructing a Teddy matcher.
///
/// The builder primarily permits fine grained configuration of the Teddy
/// matcher. Most options are made only available for testing/benchmarking
/// purposes. In reality, options are automatically determined by the nature
/// and number of patterns given to the builder.
#[derive(Clone, Debug)]
pub struct Builder {
    /// When none, this is automatically determined. Otherwise, `false` means
    /// slim Teddy is used (8 buckets) and `true` means fat Teddy is used
    /// (16 buckets). Fat Teddy requires AVX2, so if that CPU feature isn't
    /// available and Fat Teddy was requested, no matcher will be built.
    fat: Option<bool>,
    /// When none, this is automatically determined. Otherwise, `false` means
    /// that 128-bit vectors will be used (up to SSSE3 instructions) where as
    /// `true` means that 256-bit vectors will be used. As with `fat`, if
    /// 256-bit vectors are requested and they aren't available, then a
    /// searcher will not be built.
    avx: Option<bool>,
}

impl Default for Builder {
    fn default() -> Builder {
        Builder::new()
    }
}

impl Builder {
    /// Create a new builder for configuring a Teddy matcher.
    pub fn new() -> Builder {
        Builder { fat: None, avx: None }
    }

    /// Build a matcher for the set of patterns given. If a matcher could not
    /// be built, then `None` is returned.
    ///
    /// Generally, a matcher isn't built if the necessary CPU features aren't
    /// available, an unsupported target or if the searcher is believed to be
    /// slower than standard techniques (i.e., if there are too many literals).
    pub fn build(&self, patterns: &Patterns) -> Option<Teddy> {
        self.build_imp(patterns)
    }

    /// Require the use of Fat (true) or Slim (false) Teddy. Fat Teddy uses
    /// 16 buckets where as Slim Teddy uses 8 buckets. More buckets are useful
    /// for a larger set of literals.
    ///
    /// `None` is the default, which results in an automatic selection based
    /// on the number of literals and available CPU features.
    pub fn fat(&mut self, yes: Option<bool>) -> &mut Builder {
        self.fat = yes;
        self
    }

    /// Request the use of 256-bit vectors (true) or 128-bit vectors (false).
    /// Generally, a larger vector size is better since it either permits
    /// matching more patterns or matching more bytes in the haystack at once.
    ///
    /// `None` is the default, which results in an automatic selection based on
    /// the number of literals and available CPU features.
    pub fn avx(&mut self, yes: Option<bool>) -> &mut Builder {
        self.avx = yes;
        self
    }

    fn build_imp(&self, patterns: &Patterns) -> Option<Teddy> {
        use crate::packed::teddy::runtime;

        // Most of the logic here is just about selecting the optimal settings,
        // or perhaps even rejecting construction altogether. The choices
        // we have are: fat (avx only) or not, ssse3 or avx2, and how many
        // patterns we allow ourselves to search. Additionally, for testing
        // and benchmarking, we permit callers to try to "force" a setting,
        // and if the setting isn't allowed (e.g., forcing AVX when AVX isn't
        // available), then we bail and return nothing.

        if patterns.len() > 64 {
            return None;
        }
        let has_ssse3 = is_x86_feature_detected!("ssse3");
        let has_avx = is_x86_feature_detected!("avx2");
        let avx = if self.avx == Some(true) {
            if !has_avx {
                return None;
            }
            true
        } else if self.avx == Some(false) {
            if !has_ssse3 {
                return None;
            }
            false
        } else if !has_ssse3 && !has_avx {
            return None;
        } else {
            has_avx
        };
        let fat = match self.fat {
            None => avx && patterns.len() > 32,
            Some(false) => false,
            Some(true) if !avx => return None,
            Some(true) => true,
        };

        let mut compiler = Compiler::new(patterns, fat);
        compiler.compile();
        let Compiler { buckets, masks, .. } = compiler;
        // SAFETY: It is required that the builder only produce Teddy matchers
        // that are allowed to run on the current CPU, since we later assume
        // that the presence of (for example) TeddySlim1Mask256 means it is
        // safe to call functions marked with the `avx2` target feature.
        match (masks.len(), avx, fat) {
            (1, false, _) => Some(Teddy {
                buckets,
                max_pattern_id: patterns.max_pattern_id(),
                exec: runtime::Exec::TeddySlim1Mask128(
                    runtime::TeddySlim1Mask128 {
                        mask1: runtime::Mask128::new(masks[0]),
                    },
                ),
            }),
            (1, true, false) => Some(Teddy {
                buckets,
                max_pattern_id: patterns.max_pattern_id(),
                exec: runtime::Exec::TeddySlim1Mask256(
                    runtime::TeddySlim1Mask256 {
                        mask1: runtime::Mask256::new(masks[0]),
                    },
                ),
            }),
            (1, true, true) => Some(Teddy {
                buckets,
                max_pattern_id: patterns.max_pattern_id(),
                exec: runtime::Exec::TeddyFat1Mask256(
                    runtime::TeddyFat1Mask256 {
                        mask1: runtime::Mask256::new(masks[0]),
                    },
                ),
            }),
            (2, false, _) => Some(Teddy {
                buckets,
                max_pattern_id: patterns.max_pattern_id(),
                exec: runtime::Exec::TeddySlim2Mask128(
                    runtime::TeddySlim2Mask128 {
                        mask1: runtime::Mask128::new(masks[0]),
                        mask2: runtime::Mask128::new(masks[1]),
                    },
                ),
            }),
            (2, true, false) => Some(Teddy {
                buckets,
                max_pattern_id: patterns.max_pattern_id(),
                exec: runtime::Exec::TeddySlim2Mask256(
                    runtime::TeddySlim2Mask256 {
                        mask1: runtime::Mask256::new(masks[0]),
                        mask2: runtime::Mask256::new(masks[1]),
                    },
                ),
            }),
            (2, true, true) => Some(Teddy {
                buckets,
                max_pattern_id: patterns.max_pattern_id(),
                exec: runtime::Exec::TeddyFat2Mask256(
                    runtime::TeddyFat2Mask256 {
                        mask1: runtime::Mask256::new(masks[0]),
                        mask2: runtime::Mask256::new(masks[1]),
                    },
                ),
            }),
            (3, false, _) => Some(Teddy {
                buckets,
                max_pattern_id: patterns.max_pattern_id(),
                exec: runtime::Exec::TeddySlim3Mask128(
                    runtime::TeddySlim3Mask128 {
                        mask1: runtime::Mask128::new(masks[0]),
                        mask2: runtime::Mask128::new(masks[1]),
                        mask3: runtime::Mask128::new(masks[2]),
                    },
                ),
            }),
            (3, true, false) => Some(Teddy {
                buckets,
                max_pattern_id: patterns.max_pattern_id(),
                exec: runtime::Exec::TeddySlim3Mask256(
                    runtime::TeddySlim3Mask256 {
                        mask1: runtime::Mask256::new(masks[0]),
                        mask2: runtime::Mask256::new(masks[1]),
                        mask3: runtime::Mask256::new(masks[2]),
                    },
                ),
            }),
            (3, true, true) => Some(Teddy {
                buckets,
                max_pattern_id: patterns.max_pattern_id(),
                exec: runtime::Exec::TeddyFat3Mask256(
                    runtime::TeddyFat3Mask256 {
                        mask1: runtime::Mask256::new(masks[0]),
                        mask2: runtime::Mask256::new(masks[1]),
                        mask3: runtime::Mask256::new(masks[2]),
                    },
                ),
            }),
            _ => unreachable!(),
        }
    }
}

/// A compiler is in charge of allocating patterns into buckets and generating
/// the masks necessary for searching.
#[derive(Clone)]
struct Compiler<'p> {
    patterns: &'p Patterns,
    buckets: Vec<Vec<PatternID>>,
    masks: Vec<Mask>,
}

impl<'p> Compiler<'p> {
    /// Create a new Teddy compiler for the given patterns. If `fat` is true,
    /// then 16 buckets will be used instead of 8.
    ///
    /// This panics if any of the patterns given are empty.
    fn new(patterns: &'p Patterns, fat: bool) -> Compiler<'p> {
        let mask_len = cmp::min(3, patterns.minimum_len());
        assert!(1 <= mask_len && mask_len <= 3);

        Compiler {
            patterns,
            buckets: vec![vec![]; if fat { 16 } else { 8 }],
            masks: vec![Mask::default(); mask_len],
        }
    }

    /// Compile the patterns in this compiler into buckets and masks.
    fn compile(&mut self) {
        let mut lonibble_to_bucket: BTreeMap<Vec<u8>, usize> = BTreeMap::new();
        for (id, pattern) in self.patterns.iter() {
            // We try to be slightly clever in how we assign patterns into
            // buckets. Generally speaking, we want patterns with the same
            // prefix to be in the same bucket, since it minimizes the amount
            // of time we spend churning through buckets in the verification
            // step.
            //
            // So we could assign patterns with the same N-prefix (where N
            // is the size of the mask, which is one of {1, 2, 3}) to the
            // same bucket. However, case insensitive searches are fairly
            // common, so we'd for example, ideally want to treat `abc` and
            // `ABC` as if they shared the same prefix. ASCII has the nice
            // property that the lower 4 bits of A and a are the same, so we
            // therefore group patterns with the same low-nybbe-N-prefix into
            // the same bucket.
            //
            // MOREOVER, this is actually necessary for correctness! In
            // particular, by grouping patterns with the same prefix into the
            // same bucket, we ensure that we preserve correct leftmost-first
            // and leftmost-longest match semantics. In addition to the fact
            // that `patterns.iter()` iterates in the correct order, this
            // guarantees that all possible ambiguous matches will occur in
            // the same bucket. The verification routine could be adjusted to
            // support correct leftmost match semantics regardless of bucket
            // allocation, but that results in a performance hit. It's much
            // nicer to be able to just stop as soon as a match is found.
            let lonybs = pattern.low_nybbles(self.masks.len());
            if let Some(&bucket) = lonibble_to_bucket.get(&lonybs) {
                self.buckets[bucket].push(id);
            } else {
                // N.B. We assign buckets in reverse because it shouldn't have
                // any influence on performance, but it does make it harder to
                // get leftmost match semantics accidentally correct.
                let bucket = (self.buckets.len() - 1)
                    - (id as usize % self.buckets.len());
                self.buckets[bucket].push(id);
                lonibble_to_bucket.insert(lonybs, bucket);
            }
        }
        for (bucket_index, bucket) in self.buckets.iter().enumerate() {
            for &pat_id in bucket {
                let pat = self.patterns.get(pat_id);
                for (i, mask) in self.masks.iter_mut().enumerate() {
                    if self.buckets.len() == 8 {
                        mask.add_slim(bucket_index as u8, pat.bytes()[i]);
                    } else {
                        mask.add_fat(bucket_index as u8, pat.bytes()[i]);
                    }
                }
            }
        }
    }
}

impl<'p> fmt::Debug for Compiler<'p> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buckets = vec![vec![]; self.buckets.len()];
        for (i, bucket) in self.buckets.iter().enumerate() {
            for &patid in bucket {
                buckets[i].push(self.patterns.get(patid));
            }
        }
        f.debug_struct("Compiler")
            .field("buckets", &buckets)
            .field("masks", &self.masks)
            .finish()
    }
}

/// Mask represents the low and high nybble masks that will be used during
/// search. Each mask is 32 bytes wide, although only the first 16 bytes are
/// used for the SSSE3 runtime.
///
/// Each byte in the mask corresponds to a 8-bit bitset, where bit `i` is set
/// if and only if the corresponding nybble is in the ith bucket. The index of
/// the byte (0-15, inclusive) corresponds to the nybble.
///
/// Each mask is used as the target of a shuffle, where the indices for the
/// shuffle are taken from the haystack. AND'ing the shuffles for both the
/// low and high masks together also results in 8-bit bitsets, but where bit
/// `i` is set if and only if the correspond *byte* is in the ith bucket.
///
/// During compilation, masks are just arrays. But during search, these masks
/// are represented as 128-bit or 256-bit vectors.
///
/// (See the README is this directory for more details.)
#[derive(Clone, Copy, Default)]
pub struct Mask {
    lo: [u8; 32],
    hi: [u8; 32],
}

impl Mask {
    /// Update this mask by adding the given byte to the given bucket. The
    /// given bucket must be in the range 0-7.
    ///
    /// This is for "slim" Teddy, where there are only 8 buckets.
    fn add_slim(&mut self, bucket: u8, byte: u8) {
        assert!(bucket < 8);

        let byte_lo = (byte & 0xF) as usize;
        let byte_hi = ((byte >> 4) & 0xF) as usize;
        // When using 256-bit vectors, we need to set this bucket assignment in
        // the low and high 128-bit portions of the mask. This allows us to
        // process 32 bytes at a time. Namely, AVX2 shuffles operate on each
        // of the 128-bit lanes, rather than the full 256-bit vector at once.
        self.lo[byte_lo] |= 1 << bucket;
        self.lo[byte_lo + 16] |= 1 << bucket;
        self.hi[byte_hi] |= 1 << bucket;
        self.hi[byte_hi + 16] |= 1 << bucket;
    }

    /// Update this mask by adding the given byte to the given bucket. The
    /// given bucket must be in the range 0-15.
    ///
    /// This is for "fat" Teddy, where there are 16 buckets.
    fn add_fat(&mut self, bucket: u8, byte: u8) {
        assert!(bucket < 16);

        let byte_lo = (byte & 0xF) as usize;
        let byte_hi = ((byte >> 4) & 0xF) as usize;
        // Unlike slim teddy, fat teddy only works with AVX2. For fat teddy,
        // the high 128 bits of our mask correspond to buckets 8-15, while the
        // low 128 bits correspond to buckets 0-7.
        if bucket < 8 {
            self.lo[byte_lo] |= 1 << bucket;
            self.hi[byte_hi] |= 1 << bucket;
        } else {
            self.lo[byte_lo + 16] |= 1 << (bucket % 8);
            self.hi[byte_hi + 16] |= 1 << (bucket % 8);
        }
    }

    /// Return the low 128 bits of the low-nybble mask.
    pub fn lo128(&self) -> [u8; 16] {
        let mut tmp = [0; 16];
        tmp.copy_from_slice(&self.lo[..16]);
        tmp
    }

    /// Return the full low-nybble mask.
    pub fn lo256(&self) -> [u8; 32] {
        self.lo
    }

    /// Return the low 128 bits of the high-nybble mask.
    pub fn hi128(&self) -> [u8; 16] {
        let mut tmp = [0; 16];
        tmp.copy_from_slice(&self.hi[..16]);
        tmp
    }

    /// Return the full high-nybble mask.
    pub fn hi256(&self) -> [u8; 32] {
        self.hi
    }
}

impl fmt::Debug for Mask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (mut parts_lo, mut parts_hi) = (vec![], vec![]);
        for i in 0..32 {
            parts_lo.push(format!("{:02}: {:08b}", i, self.lo[i]));
            parts_hi.push(format!("{:02}: {:08b}", i, self.hi[i]));
        }
        f.debug_struct("Mask")
            .field("lo", &parts_lo)
            .field("hi", &parts_hi)
            .finish()
    }
}
