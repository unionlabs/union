/// Validates that the given `value` is inside the defined range. The `max` and `min` parameters are
/// optional and will only be validated if they are not `None`
///
#[must_use]
pub fn validate_range<T>(value: T, min: Option<T>, max: Option<T>) -> bool
where
    T: PartialOrd + PartialEq,
{
    if let Some(max) = max {
        if value > max {
            return false;
        }
    }

    if let Some(min) = min {
        if value < min {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::validate_range;

    #[test]
    fn test_validate_range_generic_ok() {
        // Unspecified generic type:
        assert!(validate_range(10, Some(-10), Some(10)));
        assert!(validate_range(0.0, Some(0.0), Some(10.0)));

        // Specified type:
        assert!(validate_range(5u8, Some(0), Some(255)));
        assert!(validate_range(4u16, Some(0), Some(16)));
        assert!(validate_range(6u32, Some(0), Some(23)));
    }

    #[test]
    fn test_validate_range_generic_fail() {
        assert!(!validate_range(5, Some(17), Some(19)));
        assert!(!validate_range(-1.0, Some(0.0), Some(10.0)));
    }

    #[test]
    fn test_validate_range_generic_min_only() {
        assert!(!validate_range(5, Some(10), None));
        assert!(validate_range(15, Some(10), None));
    }

    #[test]
    fn test_validate_range_generic_max_only() {
        assert!(validate_range(5, None, Some(10)));
        assert!(!validate_range(15, None, Some(10)));
    }
}
