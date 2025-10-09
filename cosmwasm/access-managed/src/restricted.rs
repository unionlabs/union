use access_manager_types::{CanCall, Selector, managed::error::AccessManagedError};
use cosmwasm_std::{Addr, DepsMut, Env, MessageInfo, SubMsg, WasmMsg, to_json_binary};
use depolama::{StorageExt, Store};
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};

use crate::{error::ContractError, state::ConsumingSchedule};

pub const ACCESS_MANAGED_CONSUME_SCHEDULED_OP_REPLY_ID: u64 = u64::MAX;

#[derive(Debug)]
pub struct Restricted<T: DeserializeOwned + Serialize> {
    selector: &'static Selector,
    value: T,
}

impl<T: DeserializeOwned + Serialize> Restricted<T> {
    #[allow(clippy::needless_pass_by_value, clippy::missing_panics_doc)]
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
            Ok(EnsureCanCallResult::Msg(self.value))
        } else if delay > 0 {
            deps.storage.write_item::<ConsumingSchedule>(&true);

            Ok(EnsureCanCallResult::Scheduled(
                [
                    SubMsg::reply_never(WasmMsg::Execute {
                        contract_addr: authority.into(),
                        msg: to_json_binary(
                            &access_manager_types::manager::msg::ExecuteMsg::ConsumeScheduledOp {
                                caller: info.sender.clone(),
                                data: serde_json_wasm::to_string(&self.value).expect("infallible"),
                            },
                        )?,
                        funds: vec![],
                    }),
                    SubMsg::reply_on_success(
                        WasmMsg::Execute {
                            contract_addr: env.contract.address.to_string(),
                            msg: to_json_binary(&self.value).expect("infallible"),
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
}

pub enum EnsureCanCallResult<T> {
    Msg(T),
    Scheduled(Vec<SubMsg>),
}

impl<'de, T: DeserializeOwned + Serialize> Deserialize<'de> for Restricted<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = T::deserialize(deserializer)?;

        let selector = Selector::extract_from_serialize(&value);

        Ok(Self { selector, value })
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
            serde_json_wasm::from_slice::<Restricted<ExecuteMsg>>(json)
                .unwrap_err()
                .to_string(),
            expect
        );
    }

    #[test]
    fn restricted_deser_ok() {
        let obj = br#"{"key":{}}"#;
        let restricted = serde_json_wasm::from_slice::<Restricted<ExecuteMsg>>(obj).unwrap();

        assert_eq!(restricted.selector, Selector::new("key"));
        assert_eq!(restricted.value, ExecuteMsg::Key {});
    }

    #[test]
    fn restricted_deser_value_not_object_ok() {
        let obj = br#"{"key2":1}"#;
        let restricted = serde_json_wasm::from_slice::<Restricted<ExecuteMsg>>(obj).unwrap();

        assert_eq!(restricted.selector, Selector::new("key2"));
        assert_eq!(restricted.value, ExecuteMsg::Key2(1));
    }

    #[test]
    fn restricted_deser_unknown_variant_fails() {
        deser_expect_error(
            br#"{"key\n":{}}"#,
            "unknown variant `key\n`, expected `key` or `key2`",
        );
    }

    #[test]
    fn restricted_deser_multiple_keys_different_key_name_fails() {
        deser_expect_error(
            br#"{"key":{},"key2":{}}"#,
            "Expected this character to start a JSON value.",
        );
    }

    #[test]
    fn restricted_deser_multiple_keys_same_key_name_fails() {
        deser_expect_error(
            br#"{"key":{},"key":{}}"#,
            "Expected this character to start a JSON value.",
        );
    }

    #[test]
    fn restricted_deser_no_key() {
        deser_expect_error(br"{}", "Invalid type");
    }

    #[test]
    fn restricted_deser_not_object() {
        deser_expect_error(
            b"null",
            "Expected to parse either a `true`, `false`, or a `null`.",
        );
    }
}
