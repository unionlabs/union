/// Validates that the 2 given fields match.
/// Both fields are optionals
#[must_use]
pub fn validate_must_match<T: Eq>(a: T, b: T) -> bool {
    a == b
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::validate_must_match;

    #[test]
    fn test_validate_must_match_strings_valid() {
        assert!(validate_must_match("hey".to_string(), "hey".to_string()))
    }

    #[test]
    fn test_validate_must_match_cows_valid() {
        let left: Cow<'static, str> = "hey".into();
        let right: Cow<'static, str> = String::from("hey").into();
        assert!(validate_must_match(left, right))
    }

    #[test]
    fn test_validate_must_match_numbers() {
        assert!(validate_must_match(2, 2))
    }

    #[test]
    fn test_validate_must_match_numbers_false() {
        assert!(!validate_must_match(2, 3));
    }

    #[test]
    fn test_validate_must_match_numbers_option_false() {
        assert!(!validate_must_match(Some(2), Some(3)));
    }

    #[test]
    fn test_validate_must_match_numbers_option_true() {
        assert!(validate_must_match(Some(6), Some(6)));
    }

    #[test]
    fn test_validate_must_match_none_some_false() {
        assert!(!validate_must_match(None, Some(3)));
    }

    #[test]
    fn test_validate_must_match_some_none_false() {
        assert!(!validate_must_match(Some(3), None));
    }

    #[test]
    fn test_validate_must_match_none_none_true() {
        // We need to define one of the values here as rust
        // can not infer the generic type from None and None
        let a: Option<u64> = None;
        assert!(validate_must_match(a, None));
    }
}
