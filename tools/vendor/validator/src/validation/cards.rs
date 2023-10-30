use std::borrow::Cow;

use card_validate::Validate as CardValidate;

#[must_use]
pub fn validate_credit_card<'a, T>(card: T) -> bool
where
    T: Into<Cow<'a, str>>,
{
    CardValidate::from(&card.into()).is_ok()
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::validate_credit_card;

    #[test]
    fn test_credit_card() {
        let tests = vec![
            ("4539571147647251", true),
            ("343380440754432", true),
            ("zduhefljsdfKJKJZHUI", false),
            ("5236313877109141", false),
        ];

        for (input, expected) in tests {
            assert_eq!(validate_credit_card(input), expected);
        }
    }

    #[test]
    fn test_credit_card_cow() {
        let test: Cow<'static, str> = "4539571147647251".into();
        assert!(validate_credit_card(test));
        let test: Cow<'static, str> = String::from("4539571147647251").into();
        assert!(validate_credit_card(test));
        let test: Cow<'static, str> = "5236313877109141".into();
        assert!(!validate_credit_card(test));
        let test: Cow<'static, str> = String::from("5236313877109141").into();
        assert!(!validate_credit_card(test));
    }
}
