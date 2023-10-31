use lib::*;

pub fn from_bounds<I>(iter: &I) -> Option<usize>
where
    I: Iterator,
{
    helper(iter.size_hint())
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[inline]
pub fn cautious(hint: Option<usize>) -> usize {
    cmp::min(hint.unwrap_or(0), 4096)
}

fn helper(bounds: (usize, Option<usize>)) -> Option<usize> {
    match bounds {
        (lower, Some(upper)) if lower == upper => Some(upper),
        _ => None,
    }
}
