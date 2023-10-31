pub use self::r#impl::Diff;

pub enum Render<'a> {
    Common(&'a str),
    Unique(&'a str),
}

#[cfg(all(feature = "diff", not(windows)))]
mod r#impl {
    use super::Render;
    use dissimilar::Chunk;
    use std::cmp;
    use std::panic;

    pub struct Diff<'a> {
        expected: &'a str,
        actual: &'a str,
        diff: Vec<Chunk<'a>>,
    }

    impl<'a> Diff<'a> {
        pub fn compute(expected: &'a str, actual: &'a str) -> Option<Self> {
            if expected.len() + actual.len() > 2048 {
                // We don't yet trust the dissimilar crate to work well on large
                // inputs.
                return None;
            }

            // Nor on non-ascii inputs.
            let diff = panic::catch_unwind(|| dissimilar::diff(expected, actual)).ok()?;

            let mut common_len = 0;
            for chunk in &diff {
                if let Chunk::Equal(common) = chunk {
                    common_len += common.len();
                }
            }

            let bigger_len = cmp::max(expected.len(), actual.len());
            let worth_printing = 5 * common_len >= 4 * bigger_len;
            if !worth_printing {
                return None;
            }

            Some(Diff {
                expected,
                actual,
                diff,
            })
        }

        pub fn iter<'i>(&'i self, input: &str) -> impl Iterator<Item = Render<'a>> + 'i {
            let expected = input == self.expected;
            let actual = input == self.actual;
            self.diff.iter().filter_map(move |chunk| match chunk {
                Chunk::Equal(common) => Some(Render::Common(common)),
                Chunk::Delete(unique) if expected => Some(Render::Unique(unique)),
                Chunk::Insert(unique) if actual => Some(Render::Unique(unique)),
                _ => None,
            })
        }
    }
}

#[cfg(any(not(feature = "diff"), windows))]
mod r#impl {
    use super::Render;

    pub enum Diff {}

    impl Diff {
        pub fn compute(_expected: &str, _actual: &str) -> Option<Self> {
            None
        }

        pub fn iter(&self, _input: &str) -> Box<dyn Iterator<Item = Render>> {
            let _ = Render::Common;
            let _ = Render::Unique;
            match *self {}
        }
    }
}
