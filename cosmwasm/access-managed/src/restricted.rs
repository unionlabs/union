use std::marker::PhantomData;

use access_manager_types::{managed::error::AccessManagedError, CanCall, Selector};
use cosmwasm_std::{
    from_json, to_json_binary, Addr, DepsMut, Env, MessageInfo, StdError, SubMsg, WasmMsg,
};
use depolama::{StorageExt, Store};
use serde::{
    de::{self, DeserializeOwned},
    Deserialize, Deserializer,
};
use serde_json::value::RawValue;

use crate::{error::ContractError, state::ConsumingSchedule};

pub const ACCESS_MANAGED_CONSUME_SCHEDULED_OP_REPLY_ID: u64 = u64::MAX;

#[derive(Debug)]
pub struct Restricted<'a, T: DeserializeOwned> {
    selector: &'a Selector,
    raw: &'a RawValue,
    __marker: PhantomData<fn() -> T>,
}

impl<T: DeserializeOwned> Restricted<'_, T> {
    #[allow(clippy::needless_pass_by_value)]
    pub fn ensure_can_call<S: Store<Key = (), Value = Addr>>(
        self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
    ) -> Result<EnsureCanCallResult<T>, ContractError> {
        let authority = deps.storage.read_item::<S>()?;

        let CanCall {
            allowed: immediate,
            delay,
        } = deps.querier.query_wasm_smart::<CanCall>(
            &authority,
            &access_manager_types::manager::msg::QueryMsg::CanCall {
                selector: self.selector.to_owned(),
                target: env.contract.address.clone(),
                caller: info.sender.clone(),
            },
        )?;

        if immediate {
            Ok(EnsureCanCallResult::Msg(self.deserialize_inner()?))
        } else if delay > 0 {
            deps.storage.write_item::<ConsumingSchedule>(&true);

            Ok(EnsureCanCallResult::Scheduled(
                [
                    SubMsg::reply_never(WasmMsg::Execute {
                        contract_addr: authority.into(),
                        msg: to_json_binary(
                            &access_manager_types::manager::msg::ExecuteMsg::ConsumeScheduledOp {
                                caller: info.sender.clone(),
                                data: self.raw.get().to_owned(),
                            },
                        )?,
                        funds: vec![],
                    }),
                    SubMsg::reply_always(
                        WasmMsg::Execute {
                            contract_addr: env.contract.address.to_string(),
                            msg: self.raw.get().as_bytes().into(),
                            funds: vec![],
                        },
                        ACCESS_MANAGED_CONSUME_SCHEDULED_OP_REPLY_ID,
                    ),
                ]
                .to_vec(),
            ))
        } else {
            Err(AccessManagedError::AccessManagedUnauthorized {
                caller: info.sender.clone(),
            }
            .into())
        }
    }

    fn deserialize_inner(self) -> Result<T, StdError> {
        from_json(self.raw.get())
    }
}

pub enum EnsureCanCallResult<T> {
    Msg(T),
    Scheduled(Vec<SubMsg>),
}

impl<'de, T: DeserializeOwned> Deserialize<'de> for Restricted<'de, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = <&RawValue as Deserialize>::deserialize(deserializer)?;

        let selector = Selector::extract(raw.get()).map_err(de::Error::custom)?;

        Ok(Self {
            selector,
            raw,
            __marker: PhantomData,
        })
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
            serde_json::from_slice::<Restricted<ExecuteMsg>>(json)
                .unwrap_err()
                .to_string(),
            expect
        );
    }

    #[test]
    fn restricted_deser_ok() {
        let obj = br#"{"key":{}}"#;
        let restricted = serde_json::from_slice::<Restricted<ExecuteMsg>>(obj).unwrap();

        assert_eq!(restricted.selector, Selector::new("key"));
        assert_eq!(restricted.raw.get().as_bytes(), obj);

        assert_eq!(restricted.deserialize_inner().unwrap(), ExecuteMsg::Key {});
    }

    #[test]
    fn restricted_deser_value_not_object_ok() {
        let obj = br#"{"key2":1}"#;
        let restricted = serde_json::from_slice::<Restricted<ExecuteMsg>>(obj).unwrap();

        assert_eq!(restricted.selector, Selector::new("key2"));
        assert_eq!(restricted.raw.get().as_bytes(), obj);

        assert_eq!(restricted.deserialize_inner().unwrap(), ExecuteMsg::Key2(1));
    }

    #[test]
    fn restricted_deser_escaped_fails() {
        deser_expect_error(
            br#"{"key\n":{}}"#,
            r#"invalid type: string "key\n", expected a borrowed string at line 1 column 8"#,
        );
    }

    #[test]
    fn restricted_deser_multiple_keys_different_key_name_fails() {
        deser_expect_error(
            br#"{"key":{},"key2":{}}"#,
            "multiple keys found at line 1 column 16",
        );
    }

    #[test]
    fn restricted_deser_multiple_keys_same_key_name_fails() {
        deser_expect_error(
            br#"{"key":{},"key":{}}"#,
            "multiple keys found at line 1 column 15",
        );
    }

    #[test]
    fn restricted_deser_no_key() {
        deser_expect_error(br"{}", "no key found at line 1 column 2");
    }

    #[test]
    fn restricted_deser_not_object() {
        deser_expect_error(
            b"null",
            "invalid type: null, expected json object with single top level key at line 1 column 4",
        );
    }
}
