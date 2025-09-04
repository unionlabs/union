use std::borrow::Cow;

use cosmwasm_std::Addr;
use serde::{
    de::{Error, MapAccess, Visitor},
    Deserialize, Deserializer,
};

#[derive(Debug, PartialEq)]
pub struct Method<'a>(Cow<'a, str>);

impl<'de> Deserialize<'de> for Method<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MethodVisitor;

        #[derive(Deserialize)]
        struct Empty {}

        impl<'de> Visitor<'de> for MethodVisitor {
            type Value = Method<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "single top level key")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                match map.next_entry()? {
                    Some((key, Empty {})) => {
                        if dbg!(map.next_key::<Cow<'de, str>>())?.is_some() {
                            Err(<A::Error as Error>::custom("multiple keys found"))
                        } else {
                            Ok(Method(key))
                        }
                    }
                    None => Err(<A::Error as Error>::custom("no key found")),
                }
            }
        }

        deserializer.deserialize_map(MethodVisitor)
    }
}

pub enum QueryMsg {
    CanCall {
        method: Method<'static>,
        target: Addr,
        caller: Addr,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn method_deser_ok() {
        let obj = br#"{"key":{}}"#;
        let method = serde_json::from_slice::<Method>(obj).unwrap();

        assert_eq!(method, Method("key".into()));
    }

    #[test]
    fn method_deser_escaped_ok() {
        let obj = br#"{"key\n":{}}"#;
        let method = serde_json::from_slice::<Method>(obj).unwrap();

        assert_eq!(method, Method("key\n".into()));
    }

    #[track_caller]
    fn deser_expect_error(json: &[u8], expect: &str) {
        assert_eq!(
            serde_json::from_slice::<Method>(json)
                .unwrap_err()
                .to_string(),
            expect
        )
    }

    #[test]
    fn method_deser_multiple_keys_different_key_name_fails() {
        deser_expect_error(
            br#"{"key":{},"key2":{}}"#,
            "multiple keys found at line 1 column 16",
        );
    }

    #[test]
    fn method_deser_multiple_keys_same_key_name_fails() {
        deser_expect_error(
            br#"{"key":{},"key":{}}"#,
            "multiple keys found at line 1 column 15",
        );
    }

    #[test]
    fn method_deser_no_key() {
        deser_expect_error(br#"{}"#, "no key found at line 1 column 2");
    }

    #[test]
    fn method_deser_not_object() {
        deser_expect_error(
            b"null",
            "invalid type: null, expected single top level key at line 1 column 4",
        );
    }

    #[test]
    fn method_deser_value_not_object() {
        deser_expect_error(
            br#"{"key":null}"#,
            "invalid type: null, expected struct Empty at line 1 column 11",
        );
    }
}
