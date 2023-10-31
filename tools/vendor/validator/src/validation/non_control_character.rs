use std::borrow::Cow;
use unic_ucd_common::control;

#[must_use]
pub fn validate_non_control_character<'a, T>(alphabetic: T) -> bool
where
    T: Into<Cow<'a, str>> + Clone,
{
    alphabetic.into().chars().all(|code| !control::is_control(code))
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::validate_non_control_character;

    #[test]
    fn test_non_control_character() {
        let tests = vec![
            ("Himmel", true),
            ("आकाश", true),
            ("வானத்தில்", true),
            ("하늘", true),
            ("небо", true),
            ("2H₂ + O₂ ⇌ 2H₂O", true),
            ("\u{000c}", false),
            ("\u{009F}", false),
        ];

        for (input, expected) in tests {
            assert_eq!(validate_non_control_character(input), expected);
        }
    }

    #[test]
    fn test_non_control_character_cow() {
        let test: Cow<'static, str> = "आकाश".into();
        assert!(validate_non_control_character(test));
        let test: Cow<'static, str> = String::from("வானத்தில்").into();
        assert!(validate_non_control_character(test));
        let test: Cow<'static, str> = "\u{000c}".into();
        assert!(!validate_non_control_character(test));
        let test: Cow<'static, str> = String::from("\u{009F}").into();
        assert!(!validate_non_control_character(test));
    }
}
