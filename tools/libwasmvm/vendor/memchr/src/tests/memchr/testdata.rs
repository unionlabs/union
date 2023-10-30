use std::iter::repeat;

/// Create a sequence of tests that should be run by memchr implementations.
pub fn memchr_tests() -> Vec<MemchrTest> {
    let mut tests = Vec::new();
    for statict in MEMCHR_TESTS {
        assert!(!statict.corpus.contains("%"), "% is not allowed in corpora");
        assert!(!statict.corpus.contains("#"), "# is not allowed in corpora");
        assert!(!statict.needles.contains(&b'%'), "% is an invalid needle");
        assert!(!statict.needles.contains(&b'#'), "# is an invalid needle");

        let t = MemchrTest {
            corpus: statict.corpus.to_string(),
            needles: statict.needles.to_vec(),
            positions: statict.positions.to_vec(),
        };
        tests.push(t.clone());
        tests.extend(t.expand());
    }
    tests
}

/// A set of tests for memchr-like functions.
///
/// These tests mostly try to cover the short string cases. We cover the longer
/// string cases via the benchmarks (which are tests themselves), via
/// quickcheck tests and via automatic expansion of each test case (by
/// increasing the corpus size). Finally, we cover different alignment cases
/// in the tests by varying the starting point of the slice.
const MEMCHR_TESTS: &[MemchrTestStatic] = &[
    // one needle (applied to memchr + memchr2 + memchr3)
    MemchrTestStatic { corpus: "a", needles: &[b'a'], positions: &[0] },
    MemchrTestStatic { corpus: "aa", needles: &[b'a'], positions: &[0, 1] },
    MemchrTestStatic {
        corpus: "aaa",
        needles: &[b'a'],
        positions: &[0, 1, 2],
    },
    MemchrTestStatic { corpus: "", needles: &[b'a'], positions: &[] },
    MemchrTestStatic { corpus: "z", needles: &[b'a'], positions: &[] },
    MemchrTestStatic { corpus: "zz", needles: &[b'a'], positions: &[] },
    MemchrTestStatic { corpus: "zza", needles: &[b'a'], positions: &[2] },
    MemchrTestStatic { corpus: "zaza", needles: &[b'a'], positions: &[1, 3] },
    MemchrTestStatic { corpus: "zzza", needles: &[b'a'], positions: &[3] },
    MemchrTestStatic { corpus: "\x00a", needles: &[b'a'], positions: &[1] },
    MemchrTestStatic { corpus: "\x00", needles: &[b'\x00'], positions: &[0] },
    MemchrTestStatic {
        corpus: "\x00\x00",
        needles: &[b'\x00'],
        positions: &[0, 1],
    },
    MemchrTestStatic {
        corpus: "\x00a\x00",
        needles: &[b'\x00'],
        positions: &[0, 2],
    },
    MemchrTestStatic {
        corpus: "zzzzzzzzzzzzzzzza",
        needles: &[b'a'],
        positions: &[16],
    },
    MemchrTestStatic {
        corpus: "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzza",
        needles: &[b'a'],
        positions: &[32],
    },
    // two needles (applied to memchr2 + memchr3)
    MemchrTestStatic {
        corpus: "az",
        needles: &[b'a', b'z'],
        positions: &[0, 1],
    },
    MemchrTestStatic {
        corpus: "az",
        needles: &[b'a', b'z'],
        positions: &[0, 1],
    },
    MemchrTestStatic { corpus: "az", needles: &[b'x', b'y'], positions: &[] },
    MemchrTestStatic { corpus: "az", needles: &[b'a', b'y'], positions: &[0] },
    MemchrTestStatic { corpus: "az", needles: &[b'x', b'z'], positions: &[1] },
    MemchrTestStatic {
        corpus: "yyyyaz",
        needles: &[b'a', b'z'],
        positions: &[4, 5],
    },
    MemchrTestStatic {
        corpus: "yyyyaz",
        needles: &[b'z', b'a'],
        positions: &[4, 5],
    },
    // three needles (applied to memchr3)
    MemchrTestStatic {
        corpus: "xyz",
        needles: &[b'x', b'y', b'z'],
        positions: &[0, 1, 2],
    },
    MemchrTestStatic {
        corpus: "zxy",
        needles: &[b'x', b'y', b'z'],
        positions: &[0, 1, 2],
    },
    MemchrTestStatic {
        corpus: "zxy",
        needles: &[b'x', b'a', b'z'],
        positions: &[0, 1],
    },
    MemchrTestStatic {
        corpus: "zxy",
        needles: &[b't', b'a', b'z'],
        positions: &[0],
    },
    MemchrTestStatic {
        corpus: "yxz",
        needles: &[b't', b'a', b'z'],
        positions: &[2],
    },
];

/// A description of a test on a memchr like function.
#[derive(Clone, Debug)]
pub struct MemchrTest {
    /// The thing to search. We use `&str` instead of `&[u8]` because they
    /// are nicer to write in tests, and we don't miss much since memchr
    /// doesn't care about UTF-8.
    ///
    /// Corpora cannot contain either '%' or '#'. We use these bytes when
    /// expanding test cases into many test cases, and we assume they are not
    /// used. If they are used, `memchr_tests` will panic.
    corpus: String,
    /// The needles to search for. This is intended to be an "alternation" of
    /// needles. The number of needles may cause this test to be skipped for
    /// some memchr variants. For example, a test with 2 needles cannot be used
    /// to test `memchr`, but can be used to test `memchr2` and `memchr3`.
    /// However, a test with only 1 needle can be used to test all of `memchr`,
    /// `memchr2` and `memchr3`. We achieve this by filling in the needles with
    /// bytes that we never used in the corpus (such as '#').
    needles: Vec<u8>,
    /// The positions expected to match for all of the needles.
    positions: Vec<usize>,
}

/// Like MemchrTest, but easier to define as a constant.
#[derive(Clone, Debug)]
pub struct MemchrTestStatic {
    corpus: &'static str,
    needles: &'static [u8],
    positions: &'static [usize],
}

impl MemchrTest {
    pub fn one<F: Fn(u8, &[u8]) -> Option<usize>>(&self, reverse: bool, f: F) {
        let needles = match self.needles(1) {
            None => return,
            Some(needles) => needles,
        };
        // We test different alignments here. Since some implementations use
        // AVX2, which can read 32 bytes at a time, we test at least that.
        // Moreover, with loop unrolling, we sometimes process 64 (sse2) or 128
        // (avx) bytes at a time, so we include that in our offsets as well.
        //
        // You might think this would cause most needles to not be found, but
        // we actually expand our tests to include corpus sizes all the way up
        // to >500 bytes, so we should exercise most branches.
        for align in 0..130 {
            let corpus = self.corpus(align);
            assert_eq!(
                self.positions(align, reverse).get(0).cloned(),
                f(needles[0], corpus.as_bytes()),
                "search for {:?} failed in: {:?} (len: {}, alignment: {})",
                needles[0] as char,
                corpus,
                corpus.len(),
                align
            );
        }
    }

    pub fn two<F: Fn(u8, u8, &[u8]) -> Option<usize>>(
        &self,
        reverse: bool,
        f: F,
    ) {
        let needles = match self.needles(2) {
            None => return,
            Some(needles) => needles,
        };
        for align in 0..130 {
            let corpus = self.corpus(align);
            assert_eq!(
                self.positions(align, reverse).get(0).cloned(),
                f(needles[0], needles[1], corpus.as_bytes()),
                "search for {:?}|{:?} failed in: {:?} \
                 (len: {}, alignment: {})",
                needles[0] as char,
                needles[1] as char,
                corpus,
                corpus.len(),
                align
            );
        }
    }

    pub fn three<F: Fn(u8, u8, u8, &[u8]) -> Option<usize>>(
        &self,
        reverse: bool,
        f: F,
    ) {
        let needles = match self.needles(3) {
            None => return,
            Some(needles) => needles,
        };
        for align in 0..130 {
            let corpus = self.corpus(align);
            assert_eq!(
                self.positions(align, reverse).get(0).cloned(),
                f(needles[0], needles[1], needles[2], corpus.as_bytes()),
                "search for {:?}|{:?}|{:?} failed in: {:?} \
                 (len: {}, alignment: {})",
                needles[0] as char,
                needles[1] as char,
                needles[2] as char,
                corpus,
                corpus.len(),
                align
            );
        }
    }

    pub fn iter_one<'a, I, F>(&'a self, reverse: bool, f: F)
    where
        F: FnOnce(u8, &'a [u8]) -> I,
        I: Iterator<Item = usize>,
    {
        if let Some(ns) = self.needles(1) {
            self.iter(reverse, f(ns[0], self.corpus.as_bytes()));
        }
    }

    pub fn iter_two<'a, I, F>(&'a self, reverse: bool, f: F)
    where
        F: FnOnce(u8, u8, &'a [u8]) -> I,
        I: Iterator<Item = usize>,
    {
        if let Some(ns) = self.needles(2) {
            self.iter(reverse, f(ns[0], ns[1], self.corpus.as_bytes()));
        }
    }

    pub fn iter_three<'a, I, F>(&'a self, reverse: bool, f: F)
    where
        F: FnOnce(u8, u8, u8, &'a [u8]) -> I,
        I: Iterator<Item = usize>,
    {
        if let Some(ns) = self.needles(3) {
            self.iter(reverse, f(ns[0], ns[1], ns[2], self.corpus.as_bytes()));
        }
    }

    /// Test that the positions yielded by the given iterator match the
    /// positions in this test. If reverse is true, then reverse the positions
    /// before comparing them.
    fn iter<I: Iterator<Item = usize>>(&self, reverse: bool, it: I) {
        assert_eq!(
            self.positions(0, reverse),
            it.collect::<Vec<usize>>(),
            r"search for {:?} failed in: {:?}",
            self.needles.iter().map(|&b| b as char).collect::<Vec<char>>(),
            self.corpus
        );
    }

    /// Expand this test into many variations of the same test.
    ///
    /// In particular, this will generate more tests with larger corpus sizes.
    /// The expected positions are updated to maintain the integrity of the
    /// test.
    ///
    /// This is important in testing a memchr implementation, because there are
    /// often different cases depending on the length of the corpus.
    ///
    /// Note that we extend the corpus by adding `%` bytes, which we
    /// don't otherwise use as a needle.
    fn expand(&self) -> Vec<MemchrTest> {
        let mut more = Vec::new();

        // Add bytes to the start of the corpus.
        for i in 1..515 {
            let mut t = self.clone();
            let mut new_corpus: String = repeat('%').take(i).collect();
            new_corpus.push_str(&t.corpus);
            t.corpus = new_corpus;
            t.positions = t.positions.into_iter().map(|p| p + i).collect();
            more.push(t);
        }
        // Add bytes to the end of the corpus.
        for i in 1..515 {
            let mut t = self.clone();
            let padding: String = repeat('%').take(i).collect();
            t.corpus.push_str(&padding);
            more.push(t);
        }

        more
    }

    /// Return the corpus at the given alignment.
    ///
    /// If the alignment exceeds the length of the corpus, then this returns
    /// an empty slice.
    fn corpus(&self, align: usize) -> &str {
        self.corpus.get(align..).unwrap_or("")
    }

    /// Return exactly `count` needles from this test. If this test has less
    /// than `count` needles, then add `#` until the number of needles
    /// matches `count`. If this test has more than `count` needles, then
    /// return `None` (because there is no way to use this test data for a
    /// search using fewer needles).
    fn needles(&self, count: usize) -> Option<Vec<u8>> {
        if self.needles.len() > count {
            return None;
        }

        let mut needles = self.needles.to_vec();
        for _ in needles.len()..count {
            // we assume # is never used in tests.
            needles.push(b'#');
        }
        Some(needles)
    }

    /// Return the positions in this test, reversed if `reverse` is true.
    ///
    /// If alignment is given, then all positions greater than or equal to that
    /// alignment are offset by the alignment. Positions less than the
    /// alignment are dropped.
    fn positions(&self, align: usize, reverse: bool) -> Vec<usize> {
        let positions = if reverse {
            let mut positions = self.positions.to_vec();
            positions.reverse();
            positions
        } else {
            self.positions.to_vec()
        };
        positions
            .into_iter()
            .filter(|&p| p >= align)
            .map(|p| p - align)
            .collect()
    }
}
