use std::borrow::Cow;

#[must_use]
pub fn validate_phone<'a, T>(phone_number: T) -> bool
where
    T: Into<Cow<'a, str>>,
{
    if let Ok(parsed) = phonenumber::parse(None, phone_number.into()) {
        phonenumber::is_valid(&parsed)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::validate_phone;

    #[test]
    fn test_phone() {
        let tests = vec![
            ("+1 (415) 237-0800", true),
            ("+14152370800", true),
            ("+33642926829", true),
            ("14152370800", false),
            ("0642926829", false),
            ("00642926829", false),
            ("A012", false),
            ("TEXT", false),
        ];

        for (input, expected) in tests {
            println!("{} - {}", input, expected);
            assert_eq!(validate_phone(input), expected);
        }
    }

    #[test]
    fn test_phone_cow() {
        let test: Cow<'static, str> = "+1 (415) 237-0800".into();
        assert!(validate_phone(test));
        let test: Cow<'static, str> = String::from("+1 (415) 237-0800").into();
        assert!(validate_phone(test));
        let test: Cow<'static, str> = "TEXT".into();
        assert!(!validate_phone(test));
        let test: Cow<'static, str> = String::from("TEXT").into();
        assert!(!validate_phone(test));
    }
}
