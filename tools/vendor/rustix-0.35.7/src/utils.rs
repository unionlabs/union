/// Convert a `&T` into a `*const T` without using an `as`.
#[inline]
#[allow(dead_code)]
pub(crate) const fn as_ptr<T>(t: &T) -> *const T {
    t
}

/// Convert a `&mut T` into a `*mut T` without using an `as`.
#[inline]
#[allow(dead_code)]
pub(crate) fn as_mut_ptr<T>(t: &mut T) -> *mut T {
    t
}
