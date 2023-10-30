use crate::traits::Contains;

/// Validates whether the value does not contain the needle
/// The value needs to implement the Contains trait, which is implement on String, str and Hashmap<String>
/// by default.
#[must_use]
pub fn validate_does_not_contain<T: Contains>(val: T, needle: &str) -> bool {
    !val.has_element(needle)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_validate_does_not_contain_string() {
        assert_eq!(validate_does_not_contain("hey", "e"), false);
    }

    #[test]
    fn test_validate_does_not_contain_string_can_fail() {
        assert!(validate_does_not_contain("hey", "o"));
    }

    #[test]
    fn test_validate_does_not_contain_hashmap_key() {
        let mut map = HashMap::new();
        map.insert("hey".to_string(), 1);
        assert_eq!(validate_does_not_contain(map, "hey"), false);
    }

    #[test]
    fn test_validate_does_not_contain_hashmap_key_can_fail() {
        let mut map = HashMap::new();
        map.insert("hey".to_string(), 1);
        assert!(validate_does_not_contain(map, "bob"));
    }

    #[test]
    fn test_validate_does_not_contain_cow() {
        let test: Cow<'static, str> = "hey".into();
        assert_eq!(validate_does_not_contain(test, "e"), false);
        let test: Cow<'static, str> = String::from("hey").into();
        assert_eq!(validate_does_not_contain(test, "e"), false);
    }

    #[test]
    fn test_validate_does_not_contain_cow_can_fail() {
        let test: Cow<'static, str> = "hey".into();
        assert!(validate_does_not_contain(test, "o"));
        let test: Cow<'static, str> = String::from("hey").into();
        assert!(validate_does_not_contain(test, "o"));
    }
}
