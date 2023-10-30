//! Iterator utilities.

/// Extra methods for iterators.
pub trait IteratorExtras: Iterator {
    /// Create an iterator that produces adjacent pairs of elements from the iterator.
    fn adjacent_pairs(mut self) -> AdjacentPairs<Self>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        let elem = self.next();
        AdjacentPairs { iter: self, elem }
    }
}

impl<T> IteratorExtras for T where T: Iterator {}

/// Adjacent pairs iterator returned by `adjacent_pairs()`.
///
/// This wraps another iterator and produces a sequence of adjacent pairs of elements.
pub struct AdjacentPairs<I>
where
    I: Iterator,
    I::Item: Clone,
{
    iter: I,
    elem: Option<I::Item>,
}

impl<I> Iterator for AdjacentPairs<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = (I::Item, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.elem.take().and_then(|e| {
            self.elem = self.iter.next();
            self.elem.clone().map(|n| (e, n))
        })
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    #[test]
    fn adjpairs() {
        use super::IteratorExtras;

        assert_eq!(
            [1, 2, 3, 4]
                .iter()
                .cloned()
                .adjacent_pairs()
                .collect::<Vec<_>>(),
            vec![(1, 2), (2, 3), (3, 4)]
        );
        assert_eq!(
            [2, 3, 4]
                .iter()
                .cloned()
                .adjacent_pairs()
                .collect::<Vec<_>>(),
            vec![(2, 3), (3, 4)]
        );
        assert_eq!(
            [2, 3, 4]
                .iter()
                .cloned()
                .adjacent_pairs()
                .collect::<Vec<_>>(),
            vec![(2, 3), (3, 4)]
        );
        assert_eq!(
            [3, 4].iter().cloned().adjacent_pairs().collect::<Vec<_>>(),
            vec![(3, 4)]
        );
        assert_eq!(
            [4].iter().cloned().adjacent_pairs().collect::<Vec<_>>(),
            vec![]
        );
        assert_eq!(
            [].iter()
                .cloned()
                .adjacent_pairs()
                .collect::<Vec<(i32, i32)>>(),
            vec![]
        );
    }
}
