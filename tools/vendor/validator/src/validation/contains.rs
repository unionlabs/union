use crate::traits::Contains;

/// Validates whether the value contains the needle
/// The value needs to implement the Contains trait, which is implement on String, str and Hashmap<String>
/// by default.
#[must_use]
pub fn validate_contains<T: Contains>(val: T, needle: &str) -> bool {
    val.has_element(needle)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_validate_contains_string() {
        assert!(validate_contains("hey", "e"));
    }

    #[test]
    fn test_validate_contains_string_can_fail() {
        assert!(!validate_contains("hey", "o"));
    }

    #[test]
    fn test_validate_contains_hashmap_key() {
        let mut map = HashMap::new();
        map.insert("hey".to_string(), 1);
        assert!(validate_contains(map, "hey"));
    }

    #[test]
    fn test_validate_contains_hashmap_key_can_fail() {
        let mut map = HashMap::new();
        map.insert("hey".to_string(), 1);
        assert!(!validate_contains(map, "bob"));
    }

    #[test]
    fn test_validate_contains_cow() {
        let test: Cow<'static, str> = "hey".into();
        assert!(validate_contains(test, "e"));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(validate_contains(test, "e"));
    }

    #[test]
    fn test_validate_contains_cow_can_fail() {
        let test: Cow<'static, str> = "hey".into();
        assert!(!validate_contains(test, "o"));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(!validate_contains(test, "o"));
    }
}
