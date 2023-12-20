use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Binary;

use super::{CosmosMsg, Empty, Event};

/// Use this to define when the contract gets a response callback.
/// If you only need it for errors or success you can select just those in order
/// to save gas.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReplyOn {
    /// Always perform a callback after SubMsg is processed
    Always,
    /// Only callback if SubMsg returned an error, no callback on success case
    Error,
    /// Only callback if SubMsg was successful, no callback on error case
    Success,
    /// Never make a callback - this is like the original CosmosMsg semantics
    Never,
}

/// A submessage that will guarantee a `reply` call on success or error, depending on
/// the `reply_on` setting. If you do not need to process the result, use regular messages instead.
///
/// Note: On error the submessage execution will revert any partial state changes due to this message,
/// but not revert any state changes in the calling contract. If this is required, it must be done
/// manually in the `reply` entry point.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SubMsg<T = Empty> {
    /// An arbitrary ID chosen by the contract.
    /// This is typically used to match `Reply`s in the `reply` entry point to the submessage.
    pub id: u64,
    pub msg: CosmosMsg<T>,
    /// Gas limit measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    pub gas_limit: Option<u64>,
    pub reply_on: ReplyOn,
}

/// This is used for cases when we use ReplyOn::Never and the id doesn't matter
pub const UNUSED_MSG_ID: u64 = 0;

impl<T> SubMsg<T> {
    /// new creates a "fire and forget" message with the pre-0.14 semantics
    pub fn new(msg: impl Into<CosmosMsg<T>>) -> Self {
        SubMsg {
            id: UNUSED_MSG_ID,
            msg: msg.into(),
            reply_on: ReplyOn::Never,
            gas_limit: None,
        }
    }

    /// create a `SubMsg` that will provide a `reply` with the given id if the message returns `Ok`
    pub fn reply_on_success(msg: impl Into<CosmosMsg<T>>, id: u64) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Success)
    }

    /// create a `SubMsg` that will provide a `reply` with the given id if the message returns `Err`
    pub fn reply_on_error(msg: impl Into<CosmosMsg<T>>, id: u64) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Error)
    }

    /// create a `SubMsg` that will always provide a `reply` with the given id
    pub fn reply_always(msg: impl Into<CosmosMsg<T>>, id: u64) -> Self {
        Self::reply_on(msg.into(), id, ReplyOn::Always)
    }

    /// Add a gas limit to the message.
    /// This gas limit measured in [Cosmos SDK gas](https://github.com/CosmWasm/cosmwasm/blob/main/docs/GAS.md).
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{coins, BankMsg, ReplyOn, SubMsg};
    /// # let msg = BankMsg::Send { to_address: String::from("you"), amount: coins(1015, "earth") };
    /// let sub_msg: SubMsg = SubMsg::reply_always(msg, 1234).with_gas_limit(60_000);
    /// assert_eq!(sub_msg.id, 1234);
    /// assert_eq!(sub_msg.gas_limit, Some(60_000));
    /// assert_eq!(sub_msg.reply_on, ReplyOn::Always);
    /// ```
    pub fn with_gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = Some(limit);
        self
    }

    fn reply_on(msg: CosmosMsg<T>, id: u64, reply_on: ReplyOn) -> Self {
        SubMsg {
            id,
            msg,
            reply_on,
            gas_limit: None,
        }
    }
}

/// The result object returned to `reply`. We always get the ID from the submessage
/// back and then must handle success and error cases ourselves.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Reply {
    /// The ID that the contract set when emitting the `SubMsg`.
    /// Use this to identify which submessage triggered the `reply`.
    pub id: u64,
    pub result: SubMsgResult,
}

/// This is the result type that is returned from a sub message execution.
///
/// We use a custom type here instead of Rust's Result because we want to be able to
/// define the serialization, which is a public interface. Every language that compiles
/// to Wasm and runs in the ComsWasm VM needs to create the same JSON representation.
///
/// Until version 1.0.0-beta5, `ContractResult<SubMsgResponse>` was used instead
/// of this type. Once serialized, the two types are the same. However, in the Rust type
/// system we want different types for clarity and documenation reasons.
///
/// # Examples
///
/// Success:
///
/// ```
/// # use cosmwasm_std::{to_vec, Binary, Event, SubMsgResponse, SubMsgResult};
/// let response = SubMsgResponse {
///     data: Some(Binary::from_base64("MTIzCg==").unwrap()),
///     events: vec![Event::new("wasm").add_attribute("fo", "ba")],
/// };
/// let result: SubMsgResult = SubMsgResult::Ok(response);
/// assert_eq!(to_vec(&result).unwrap(), br#"{"ok":{"events":[{"type":"wasm","attributes":[{"key":"fo","value":"ba"}]}],"data":"MTIzCg=="}}"#);
/// ```
///
/// Failure:
///
/// ```
/// # use cosmwasm_std::{to_vec, SubMsgResult, Response};
/// let error_msg = String::from("Something went wrong");
/// let result = SubMsgResult::Err(error_msg);
/// assert_eq!(to_vec(&result).unwrap(), br#"{"error":"Something went wrong"}"#);
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubMsgResult {
    Ok(SubMsgResponse),
    /// An error type that every custom error created by contract developers can be converted to.
    /// This could potientially have more structure, but String is the easiest.
    #[serde(rename = "error")]
    Err(String),
}

// Implementations here mimic the Result API and should be implemented via a conversion to Result
// to ensure API consistency
impl SubMsgResult {
    /// Converts a `SubMsgResult<S>` to a `Result<S, String>` as a convenient way
    /// to access the full Result API.
    pub fn into_result(self) -> Result<SubMsgResponse, String> {
        Result::<SubMsgResponse, String>::from(self)
    }

    pub fn unwrap(self) -> SubMsgResponse {
        self.into_result().unwrap()
    }

    pub fn unwrap_err(self) -> String {
        self.into_result().unwrap_err()
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, SubMsgResult::Ok(_))
    }

    pub fn is_err(&self) -> bool {
        matches!(self, SubMsgResult::Err(_))
    }
}

impl<E: ToString> From<Result<SubMsgResponse, E>> for SubMsgResult {
    fn from(original: Result<SubMsgResponse, E>) -> SubMsgResult {
        match original {
            Ok(value) => SubMsgResult::Ok(value),
            Err(err) => SubMsgResult::Err(err.to_string()),
        }
    }
}

impl From<SubMsgResult> for Result<SubMsgResponse, String> {
    fn from(original: SubMsgResult) -> Result<SubMsgResponse, String> {
        match original {
            SubMsgResult::Ok(value) => Ok(value),
            SubMsgResult::Err(err) => Err(err),
        }
    }
}

/// The information we get back from a successful sub message execution,
/// with full Cosmos SDK events.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SubMsgResponse {
    pub events: Vec<Event>,
    pub data: Option<Binary>,
}

#[deprecated(note = "Renamed to SubMsgResponse")]
pub type SubMsgExecutionResponse = SubMsgResponse;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_vec, StdError, StdResult};

    #[test]
    fn sub_msg_result_serialization_works() {
        let result = SubMsgResult::Ok(SubMsgResponse {
            data: None,
            events: vec![],
        });
        assert_eq!(
            &to_vec(&result).unwrap(),
            br#"{"ok":{"events":[],"data":null}}"#
        );

        let result = SubMsgResult::Ok(SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![Event::new("wasm").add_attribute("fo", "ba")],
        });
        assert_eq!(
            &to_vec(&result).unwrap(),
            br#"{"ok":{"events":[{"type":"wasm","attributes":[{"key":"fo","value":"ba"}]}],"data":"MTIzCg=="}}"#
        );

        let result: SubMsgResult = SubMsgResult::Err("broken".to_string());
        assert_eq!(&to_vec(&result).unwrap(), b"{\"error\":\"broken\"}");
    }

    #[test]
    fn sub_msg_result_deserialization_works() {
        let result: SubMsgResult = from_slice(br#"{"ok":{"events":[],"data":null}}"#).unwrap();
        assert_eq!(
            result,
            SubMsgResult::Ok(SubMsgResponse {
                events: vec![],
                data: None,
            })
        );

        let result: SubMsgResult = from_slice(
            br#"{"ok":{"events":[{"type":"wasm","attributes":[{"key":"fo","value":"ba"}]}],"data":"MTIzCg=="}}"#).unwrap();
        assert_eq!(
            result,
            SubMsgResult::Ok(SubMsgResponse {
                data: Some(Binary::from_base64("MTIzCg==").unwrap()),
                events: vec![Event::new("wasm").add_attribute("fo", "ba")],
            })
        );

        let result: SubMsgResult = from_slice(br#"{"error":"broken"}"#).unwrap();
        assert_eq!(result, SubMsgResult::Err("broken".to_string()));

        // fails for additional attributes
        let parse: StdResult<SubMsgResult> = from_slice(br#"{"unrelated":321,"error":"broken"}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        let parse: StdResult<SubMsgResult> = from_slice(br#"{"error":"broken","unrelated":321}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn sub_msg_result_unwrap_works() {
        let response = SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![Event::new("wasm").add_attribute("fo", "ba")],
        };
        let success = SubMsgResult::Ok(response.clone());
        assert_eq!(success.unwrap(), response);
    }

    #[test]
    #[should_panic]
    fn sub_msg_result_unwrap_panicks_for_err() {
        let failure = SubMsgResult::Err("broken".to_string());
        let _ = failure.unwrap();
    }

    #[test]
    fn sub_msg_result_unwrap_err_works() {
        let failure = SubMsgResult::Err("broken".to_string());
        assert_eq!(failure.unwrap_err(), "broken");
    }

    #[test]
    #[should_panic]
    fn sub_msg_result_unwrap_err_panics_for_ok() {
        let response = SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![Event::new("wasm").add_attribute("fo", "ba")],
        };
        let success = SubMsgResult::Ok(response);
        let _ = success.unwrap_err();
    }

    #[test]
    fn sub_msg_result_is_ok_works() {
        let success = SubMsgResult::Ok(SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![Event::new("wasm").add_attribute("fo", "ba")],
        });
        let failure = SubMsgResult::Err("broken".to_string());
        assert!(success.is_ok());
        assert!(!failure.is_ok());
    }

    #[test]
    fn sub_msg_result_is_err_works() {
        let success = SubMsgResult::Ok(SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![Event::new("wasm").add_attribute("fo", "ba")],
        });
        let failure = SubMsgResult::Err("broken".to_string());
        assert!(failure.is_err());
        assert!(!success.is_err());
    }

    #[test]
    fn sub_msg_result_can_convert_from_core_result() {
        let original: Result<SubMsgResponse, StdError> = Ok(SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![],
        });
        let converted: SubMsgResult = original.into();
        assert_eq!(
            converted,
            SubMsgResult::Ok(SubMsgResponse {
                data: Some(Binary::from_base64("MTIzCg==").unwrap()),
                events: vec![],
            })
        );

        let original: Result<SubMsgResponse, StdError> = Err(StdError::generic_err("broken"));
        let converted: SubMsgResult = original.into();
        assert_eq!(
            converted,
            SubMsgResult::Err("Generic error: broken".to_string())
        );
    }

    #[test]
    fn sub_msg_result_can_convert_to_core_result() {
        let original = SubMsgResult::Ok(SubMsgResponse {
            data: Some(Binary::from_base64("MTIzCg==").unwrap()),
            events: vec![],
        });
        let converted: Result<SubMsgResponse, String> = original.into();
        assert_eq!(
            converted,
            Ok(SubMsgResponse {
                data: Some(Binary::from_base64("MTIzCg==").unwrap()),
                events: vec![],
            })
        );

        let original = SubMsgResult::Err("went wrong".to_string());
        let converted: Result<SubMsgResponse, String> = original.into();
        assert_eq!(converted, Err("went wrong".to_string()));
    }
}
