#[inline]
pub const fn tuple_to_option<T: Copy>((int, overflow): (T, bool)) -> Option<T> {
    if overflow {
        None
    } else {
        Some(int)
    }
}
