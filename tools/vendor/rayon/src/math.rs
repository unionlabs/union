use std::ops::{Bound, Range, RangeBounds};

/// Divide `n` by `divisor`, and round up to the nearest integer
/// if not evenly divisible.
#[inline]
pub(super) fn div_round_up(n: usize, divisor: usize) -> usize {
    debug_assert!(divisor != 0, "Division by zero!");
    if n == 0 {
        0
    } else {
        (n - 1) / divisor + 1
    }
}

/// Normalize arbitrary `RangeBounds` to a `Range`
pub(super) fn simplify_range(range: impl RangeBounds<usize>, len: usize) -> Range<usize> {
    let start = match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Included(&i) if i <= len => i,
        Bound::Excluded(&i) if i < len => i + 1,
        bound => panic!("range start {:?} should be <= length {}", bound, len),
    };
    let end = match range.end_bound() {
        Bound::Unbounded => len,
        Bound::Excluded(&i) if i <= len => i,
        Bound::Included(&i) if i < len => i + 1,
        bound => panic!("range end {:?} should be <= length {}", bound, len),
    };
    if start > end {
        panic!(
            "range start {:?} should be <= range end {:?}",
            range.start_bound(),
            range.end_bound()
        );
    }
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_div_round_up() {
        assert_eq!(0, div_round_up(0, 5));
        assert_eq!(1, div_round_up(5, 5));
        assert_eq!(1, div_round_up(1, 5));
        assert_eq!(2, div_round_up(3, 2));
        assert_eq!(
            usize::max_value() / 2 + 1,
            div_round_up(usize::max_value(), 2)
        );
    }
}
