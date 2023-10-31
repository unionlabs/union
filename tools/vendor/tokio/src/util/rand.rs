use std::cell::Cell;

/// Fast random number generate.
///
/// Implement xorshift64+: 2 32-bit xorshift sequences added together.
/// Shift triplet `[17,7,16]` was calculated as indicated in Marsaglia's
/// Xorshift paper: <https://www.jstatsoft.org/article/view/v008i14/xorshift.pdf>
/// This generator passes the SmallCrush suite, part of TestU01 framework:
/// <http://simul.iro.umontreal.ca/testu01/tu01.html>
#[derive(Debug)]
pub(crate) struct FastRand {
    one: Cell<u32>,
    two: Cell<u32>,
}

impl FastRand {
    /// Initializes a new, thread-local, fast random number generator.
    pub(crate) fn new(seed: u64) -> FastRand {
        let one = (seed >> 32) as u32;
        let mut two = seed as u32;

        if two == 0 {
            // This value cannot be zero
            two = 1;
        }

        FastRand {
            one: Cell::new(one),
            two: Cell::new(two),
        }
    }

    pub(crate) fn fastrand_n(&self, n: u32) -> u32 {
        // This is similar to fastrand() % n, but faster.
        // See https://lemire.me/blog/2016/06/27/a-fast-alternative-to-the-modulo-reduction/
        let mul = (self.fastrand() as u64).wrapping_mul(n as u64);
        (mul >> 32) as u32
    }

    fn fastrand(&self) -> u32 {
        let mut s1 = self.one.get();
        let s0 = self.two.get();

        s1 ^= s1 << 17;
        s1 = s1 ^ s0 ^ s1 >> 7 ^ s0 >> 16;

        self.one.set(s0);
        self.two.set(s1);

        s0.wrapping_add(s1)
    }
}

// Used by the select macro and `StreamMap`
#[cfg(any(feature = "macros"))]
#[doc(hidden)]
#[cfg_attr(not(feature = "macros"), allow(unreachable_pub))]
pub fn thread_rng_n(n: u32) -> u32 {
    thread_local! {
        static THREAD_RNG: FastRand = FastRand::new(crate::loom::rand::seed());
    }

    THREAD_RNG.with(|rng| rng.fastrand_n(n))
}
