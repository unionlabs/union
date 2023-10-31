/// Validates whether the given Option is Some
#[must_use]
pub fn validate_required<T>(val: &Option<T>) -> bool {
    val.is_some()
}
