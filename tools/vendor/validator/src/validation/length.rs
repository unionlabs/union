use crate::traits::HasLen;

/// Validates the length of the value given.
/// If the validator has `equal` set, it will ignore any `min` and `max` value.
///
/// If you apply it on String, don't forget that the length can be different
/// from the number of visual characters for Unicode
#[must_use]
pub fn validate_length<T: HasLen>(
    value: T,
    min: Option<u64>,
    max: Option<u64>,
    equal: Option<u64>,
) -> bool {
    let val_length = value.length();

    if let Some(eq) = equal {
        return val_length == eq;
    } else {
        if let Some(m) = min {
            if val_length < m {
                return false;
            }
        }
        if let Some(m) = max {
            if val_length > m {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::validate_length;

    #[test]
    fn test_validate_length_equal_overrides_min_max() {
        assert!(validate_length("hello", Some(1), Some(2), Some(5)));
    }

    #[test]
    fn test_validate_length_string_min_max() {
        assert!(validate_length("hello", Some(1), Some(10), None));
    }

    #[test]
    fn test_validate_length_string_min_only() {
        assert!(!validate_length("hello", Some(10), None, None));
    }

    #[test]
    fn test_validate_length_string_max_only() {
        assert!(!validate_length("hello", None, Some(1), None));
    }

    #[test]
    fn test_validate_length_cow() {
        let test: Cow<'static, str> = "hello".into();
        assert!(validate_length(test, None, None, Some(5)));

        let test: Cow<'static, str> = String::from("hello").into();
        assert!(validate_length(test, None, None, Some(5)));
    }

    #[test]
    fn test_validate_length_vec() {
        assert!(validate_length(vec![1, 2, 3], None, None, Some(3)));
    }

    #[test]
    fn test_validate_length_unicode_chars() {
        assert!(validate_length("日本", None, None, Some(2)));
    }
}
