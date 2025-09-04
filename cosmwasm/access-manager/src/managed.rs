use std::{borrow::Cow, marker::PhantomData};

use cosmwasm_std::{from_json, Addr, Deps, Env, MessageInfo, StdError, StdResult};
use depolama::{StorageExt, Store};
use serde::{
    de::{self, DeserializeOwned, MapAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::value::RawValue;

use crate::{msg::QueryMsg, types::Selector};

#[derive(Debug)]
pub struct Managed<'a, T: DeserializeOwned> {
    selector: &'a str,
    value: &'a RawValue,
    __marker: PhantomData<fn() -> T>,
}

impl<T: DeserializeOwned> Managed<'_, T> {
    // TODO: Optimize this to return a ref (Selector will need to be made generic over a lifetime)
    pub(crate) fn selector(&self) -> Selector {
        Selector::new(self.selector)
    }
}

impl<T: DeserializeOwned> Managed<'_, T> {
    pub fn ensure_can_call<S: Store<Key = (), Value = Addr>>(
        self,
        deps: Deps,
        env: &Env,
        info: &MessageInfo,
    ) -> StdResult<T> {
        let can_call = deps.querier.query_wasm_smart::<bool>(
            deps.storage.read_item::<S>()?,
            &QueryMsg::CanCall {
                selector: Selector::new(self.selector),
                target: env.contract.address.clone(),
                caller: info.sender.clone(),
            },
        )?;

        if can_call {
            let t = self.deserialize_inner()?;

            Ok(t)
        } else {
            Err(StdError::generic_err("unauthorized"))
        }
    }

    fn deserialize_inner(self) -> Result<T, StdError> {
        // TODO: Optimize this to not construct a full string
        from_json(format!(r#"{{"{}":{}}}"#, self.selector, self.value.get()).as_bytes())
    }
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for Managed<'de, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ManagedVisitor<T>(PhantomData<fn() -> T>);

        impl<'de, T: DeserializeOwned> Visitor<'de> for ManagedVisitor<T> {
            type Value = Managed<'de, T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "json object with single top level key")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                match map.next_entry()? {
                    Some((selector, data)) => {
                        if map.next_key::<Cow<'de, str>>()?.is_some() {
                            Err(<A::Error as de::Error>::custom("multiple keys found"))
                        } else {
                            Ok(Managed {
                                selector,
                                value: data,
                                __marker: PhantomData,
                            })
                        }
                    }
                    None => Err(<A::Error as de::Error>::custom("no key found")),
                }
            }
        }

        deserializer.deserialize_map(ManagedVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::*;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum ExecuteMsg {
        Key {},
        Key2(u32),
    }

    #[track_caller]
    fn deser_expect_error(json: &[u8], expect: &str) {
        assert_eq!(
            serde_json::from_slice::<Managed<ExecuteMsg>>(json)
                .unwrap_err()
                .to_string(),
            expect
        );
    }

    #[test]
    fn managed_deser_ok() {
        let obj = br#"{"key":{}}"#;
        let managed = serde_json::from_slice::<Managed<ExecuteMsg>>(obj).unwrap();

        assert_eq!(managed.selector, "key");
        assert_eq!(managed.value.get(), "{}");

        assert_eq!(managed.deserialize_inner().unwrap(), ExecuteMsg::Key {});
    }

    #[test]
    fn managed_deser_value_not_object_ok() {
        let obj = br#"{"key2":1}"#;
        let managed = serde_json::from_slice::<Managed<ExecuteMsg>>(obj).unwrap();

        assert_eq!(managed.selector, "key2");
        assert_eq!(managed.value.get(), "1");

        assert_eq!(managed.deserialize_inner().unwrap(), ExecuteMsg::Key2(1));
    }

    #[test]
    fn managed_deser_escaped_fails() {
        deser_expect_error(
            br#"{"key\n":{}}"#,
            r#"invalid type: string "key\n", expected a borrowed string at line 1 column 8"#,
        );
    }

    #[test]
    fn managed_deser_multiple_keys_different_key_name_fails() {
        deser_expect_error(
            br#"{"key":{},"key2":{}}"#,
            "multiple keys found at line 1 column 16",
        );
    }

    #[test]
    fn managed_deser_multiple_keys_same_key_name_fails() {
        deser_expect_error(
            br#"{"key":{},"key":{}}"#,
            "multiple keys found at line 1 column 15",
        );
    }

    #[test]
    fn managed_deser_no_key() {
        deser_expect_error(br"{}", "no key found at line 1 column 2");
    }

    #[test]
    fn managed_deser_not_object() {
        deser_expect_error(
            b"null",
            "invalid type: null, expected json object with single top level key at line 1 column 4",
        );
    }
}
