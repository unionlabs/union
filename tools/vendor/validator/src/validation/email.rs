use idna::domain_to_ascii;
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;

use crate::{validation::ip::validate_ip, HasLen};

lazy_static! {
    // Regex from the specs
    // https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address
    // It will mark esoteric email addresses like quoted string as invalid
    static ref EMAIL_USER_RE: Regex = Regex::new(r"^(?i)[a-z0-9.!#$%&'*+/=?^_`{|}~-]+\z").unwrap();
    static ref EMAIL_DOMAIN_RE: Regex = Regex::new(
        r"(?i)^[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*$"
    ).unwrap();
    // literal form, ipv4 or ipv6 address (SMTP 4.1.3)
    static ref EMAIL_LITERAL_RE: Regex = Regex::new(r"(?i)\[([A-f0-9:\.]+)\]\z").unwrap();
}

/// Validates whether the given string is an email based on the [HTML5 spec](https://html.spec.whatwg.org/multipage/forms.html#valid-e-mail-address).
/// [RFC 5322](https://tools.ietf.org/html/rfc5322) is not practical in most circumstances and allows email addresses
/// that are unfamiliar to most users.
#[must_use]
pub fn validate_email<'a, T>(val: T) -> bool
where
    T: Into<Cow<'a, str>>,
{
    let val = val.into();
    if val.is_empty() || !val.contains('@') {
        return false;
    }
    let parts: Vec<&str> = val.rsplitn(2, '@').collect();
    let user_part = parts[1];
    let domain_part = parts[0];

    // validate the length of each part of the email, BEFORE doing the regex
    // according to RFC5321 the max length of the local part is 64 characters
    // and the max length of the domain part is 255 characters
    // https://datatracker.ietf.org/doc/html/rfc5321#section-4.5.3.1.1
    if user_part.length() > 64 || domain_part.length() > 255 {
        return false;
    }

    if !EMAIL_USER_RE.is_match(user_part) {
        return false;
    }

    if !validate_domain_part(domain_part) {
        // Still the possibility of an [IDN](https://en.wikipedia.org/wiki/Internationalized_domain_name)
        return match domain_to_ascii(domain_part) {
            Ok(d) => validate_domain_part(&d),
            Err(_) => false,
        };
    }

    true
}

/// Checks if the domain is a valid domain and if not, check whether it's an IP
#[must_use]
fn validate_domain_part(domain_part: &str) -> bool {
    if EMAIL_DOMAIN_RE.is_match(domain_part) {
        return true;
    }

    // maybe we have an ip as a domain?
    match EMAIL_LITERAL_RE.captures(domain_part) {
        Some(caps) => match caps.get(1) {
            Some(c) => validate_ip(c.as_str()),
            None => false,
        },
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::validate_email;

    #[test]
    fn test_validate_email() {
        // Test cases taken from Django
        // https://github.com/django/django/blob/master/tests/validators/tests.py#L48
        let tests = vec![
            ("email@here.com", true),
            ("weirder-email@here.and.there.com", true),
            (r#"!def!xyz%abc@example.com"#, true),
            ("email@[127.0.0.1]", true),
            ("email@[2001:dB8::1]", true),
            ("email@[2001:dB8:0:0:0:0:0:1]", true),
            ("email@[::fffF:127.0.0.1]", true),
            ("example@valid-----hyphens.com", true),
            ("example@valid-with-hyphens.com", true),
            ("test@domain.with.idn.tld.उदाहरण.परीक्षा", true),
            (r#""test@test"@example.com"#, false),
            // max length for domain name labels is 63 characters per RFC 1034
            ("a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", true),
            ("a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.atm", true),
            (
                "a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.bbbbbbbbbb.atm",
                true,
            ),
            // 64 * a
            ("a@atm.aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", false),
            ("", false),
            ("abc", false),
            ("abc@", false),
            ("abc@bar", true),
            ("a @x.cz", false),
            ("abc@.com", false),
            ("something@@somewhere.com", false),
            ("email@127.0.0.1", true),
            ("email@[127.0.0.256]", false),
            ("email@[2001:db8::12345]", false),
            ("email@[2001:db8:0:0:0:0:1]", false),
            ("email@[::ffff:127.0.0.256]", false),
            ("example@invalid-.com", false),
            ("example@-invalid.com", false),
            ("example@invalid.com-", false),
            ("example@inv-.alid-.com", false),
            ("example@inv-.-alid.com", false),
            (r#"test@example.com\n\n<script src="x.js">"#, false),
            (r#""\\\011"@here.com"#, false),
            (r#""\\\012"@here.com"#, false),
            ("trailingdot@shouldfail.com.", false),
            // Trailing newlines in username or domain not allowed
            ("a@b.com\n", false),
            ("a\n@b.com", false),
            (r#""test@test"\n@example.com"#, false),
            ("a@[127.0.0.1]\n", false),
            // underscores are not allowed
            ("John.Doe@exam_ple.com", false),
        ];

        for (input, expected) in tests {
            // println!("{} - {}", input, expected);
            assert_eq!(
                validate_email(input),
                expected,
                "Email `{}` was not classified correctly",
                input
            );
        }
    }

    #[test]
    fn test_validate_email_cow() {
        let test: Cow<'static, str> = "email@here.com".into();
        assert!(validate_email(test));
        let test: Cow<'static, str> = String::from("email@here.com").into();
        assert!(validate_email(test));
        let test: Cow<'static, str> = "a@[127.0.0.1]\n".into();
        assert!(!validate_email(test));
        let test: Cow<'static, str> = String::from("a@[127.0.0.1]\n").into();
        assert!(!validate_email(test));
    }

    #[test]
    fn test_validate_email_rfc5321() {
        // 65 character local part
        let test = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa@mail.com";
        assert_eq!(validate_email(test), false);
        // 256 character domain part
        let test = "a@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.com";
        assert_eq!(validate_email(test), false);
    }
}
