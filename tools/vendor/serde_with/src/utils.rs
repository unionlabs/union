pub(crate) mod duration;

use crate::prelude::*;

/// Re-Implementation of `serde::private::de::size_hint::cautious`
#[cfg(feature = "alloc")]
#[inline]
pub(crate) fn size_hint_cautious(hint: Option<usize>) -> usize {
    core::cmp::min(hint.unwrap_or(0), 4096)
}

/// Re-Implementation of `serde::private::de::size_hint::from_bounds`
#[cfg(feature = "alloc")]
#[inline]
pub fn size_hint_from_bounds<I>(iter: &I) -> Option<usize>
where
    I: Iterator,
{
    fn _size_hint_from_bounds(bounds: (usize, Option<usize>)) -> Option<usize> {
        match bounds {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
    _size_hint_from_bounds(iter.size_hint())
}

pub(crate) const NANOS_PER_SEC: u32 = 1_000_000_000;
// pub(crate) const NANOS_PER_MILLI: u32 = 1_000_000;
// pub(crate) const NANOS_PER_MICRO: u32 = 1_000;
// pub(crate) const MILLIS_PER_SEC: u64 = 1_000;
// pub(crate) const MICROS_PER_SEC: u64 = 1_000_000;

pub(crate) struct MapIter<'de, A, K, V> {
    pub(crate) access: A,
    marker: PhantomData<(&'de (), K, V)>,
}

impl<'de, A, K, V> MapIter<'de, A, K, V> {
    pub(crate) fn new(access: A) -> Self
    where
        A: MapAccess<'de>,
    {
        Self {
            access,
            marker: PhantomData,
        }
    }
}

impl<'de, A, K, V> Iterator for MapIter<'de, A, K, V>
where
    A: MapAccess<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    type Item = Result<(K, V), A::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.access.next_entry().transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.access.size_hint() {
            Some(size) => (size, Some(size)),
            None => (0, None),
        }
    }
}

pub(crate) struct SeqIter<'de, A, T> {
    access: A,
    marker: PhantomData<(&'de (), T)>,
}

impl<'de, A, T> SeqIter<'de, A, T> {
    pub(crate) fn new(access: A) -> Self
    where
        A: SeqAccess<'de>,
    {
        Self {
            access,
            marker: PhantomData,
        }
    }
}

impl<'de, A, T> Iterator for SeqIter<'de, A, T>
where
    A: SeqAccess<'de>,
    T: Deserialize<'de>,
{
    type Item = Result<T, A::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.access.next_element().transpose()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.access.size_hint() {
            Some(size) => (size, Some(size)),
            None => (0, None),
        }
    }
}

pub(crate) fn duration_as_secs_f64(dur: &Duration) -> f64 {
    (dur.as_secs() as f64) + (dur.subsec_nanos() as f64) / (NANOS_PER_SEC as f64)
}

pub(crate) fn duration_signed_from_secs_f64(secs: f64) -> Result<DurationSigned, &'static str> {
    const MAX_NANOS_F64: f64 = ((u64::max_value() as u128 + 1) * (NANOS_PER_SEC as u128)) as f64;
    // TODO why are the seconds converted to nanoseconds first?
    // Does it make sense to just truncate the value?
    let mut nanos = secs * (NANOS_PER_SEC as f64);
    if !nanos.is_finite() {
        return Err("got non-finite value when converting float to duration");
    }
    if nanos >= MAX_NANOS_F64 {
        return Err("overflow when converting float to duration");
    }
    let mut sign = self::duration::Sign::Positive;
    if nanos < 0.0 {
        nanos = -nanos;
        sign = self::duration::Sign::Negative;
    }
    let nanos = nanos as u128;
    Ok(self::duration::DurationSigned::new(
        sign,
        (nanos / (NANOS_PER_SEC as u128)) as u64,
        (nanos % (NANOS_PER_SEC as u128)) as u32,
    ))
}

/// Collect an array of a fixed size from an iterator.
///
/// # Safety
/// The code follow exactly the pattern of initializing an array element-by-element from the standard library.
/// <https://doc.rust-lang.org/nightly/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element>
pub(crate) fn array_from_iterator<I, T, E, const N: usize>(
    mut iter: I,
    expected: &dyn Expected,
) -> Result<[T; N], E>
where
    I: Iterator<Item = Result<T, E>>,
    E: DeError,
{
    use core::mem::MaybeUninit;

    fn drop_array_elems<T, const N: usize>(num: usize, mut arr: [MaybeUninit<T>; N]) {
        arr[..num].iter_mut().for_each(|elem| {
            // TODO This would be better with assume_init_drop nightly function
            // https://github.com/rust-lang/rust/issues/63567
            unsafe { core::ptr::drop_in_place(elem.as_mut_ptr()) };
        });
    }

    // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
    // safe because the type we are claiming to have initialized here is a
    // bunch of `MaybeUninit`s, which do not require initialization.
    //
    // TODO could be simplified with nightly maybe_uninit_uninit_array feature
    // https://doc.rust-lang.org/nightly/std/mem/union.MaybeUninit.html#method.uninit_array

    // Clippy is broken and has a false positive here
    // https://github.com/rust-lang/rust-clippy/issues/10551
    #[allow(clippy::uninit_assumed_init)]
    let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };

    // Dropping a `MaybeUninit` does nothing. Thus using raw pointer
    // assignment instead of `ptr::write` does not cause the old
    // uninitialized value to be dropped. Also if there is a panic during
    // this loop, we have a memory leak, but there is no memory safety
    // issue.
    for (idx, elem) in arr[..].iter_mut().enumerate() {
        *elem = match iter.next() {
            Some(Ok(value)) => MaybeUninit::new(value),
            Some(Err(err)) => {
                drop_array_elems(idx, arr);
                return Err(err);
            }
            None => {
                drop_array_elems(idx, arr);
                return Err(DeError::invalid_length(idx, expected));
            }
        };
    }

    // Everything is initialized. Transmute the array to the
    // initialized type.
    // A normal transmute is not possible because of:
    // https://github.com/rust-lang/rust/issues/61956
    Ok(unsafe { core::mem::transmute_copy::<_, [T; N]>(&arr) })
}
