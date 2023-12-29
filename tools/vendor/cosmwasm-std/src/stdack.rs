use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::binary::Binary;
use crate::to_json_binary;

/// This is a standard IBC acknowledgement type. IBC application are free
/// to use any acknowledgement format they want. However, for compatibility
/// purposes it is recommended to use this.
///
/// The original proto definition can be found at <https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147>
/// and <https://github.com/cosmos/ibc/tree/ed849c7bac/spec/core/ics-004-channel-and-packet-semantics#acknowledgement-envelope>.
///
/// In contrast to the original idea, [ICS-20](https://github.com/cosmos/ibc/tree/ed849c7bacf16204e9509f0f0df325391f3ce25c/spec/app/ics-020-fungible-token-transfer#technical-specification) and CosmWasm IBC protocols
/// use JSON instead of a protobuf serialization.
///
/// For compatibility, we use the field name "result" for the success case in JSON.
/// However, all Rust APIs use the term "success" for clarity and discriminability from [Result].
///
/// If ibc_receive_packet returns Err(), then x/wasm runtime will rollback the state and
/// return an error message in this format.
///
/// ## Examples
///
/// For your convenience, there are success and error constructors.
///
/// ```
/// use cosmwasm_std::StdAck;
///
/// let ack1 = StdAck::success(b"\x01"); // 0x01 is a FungibleTokenPacketSuccess from ICS-20.
/// assert!(ack1.is_success());
///
/// let ack2 = StdAck::error("kaputt"); // Some free text error message
/// assert!(ack2.is_error());
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StdAck {
    #[serde(rename = "result")]
    Success(Binary),
    Error(String),
}

impl StdAck {
    /// Creates a success ack with the given data
    pub fn success(data: impl Into<Binary>) -> Self {
        StdAck::Success(data.into())
    }

    /// Creates an error ack
    pub fn error(err: impl Into<String>) -> Self {
        StdAck::Error(err.into())
    }

    #[must_use = "if you intended to assert that this is a success, consider `.unwrap()` instead"]
    #[inline]
    pub const fn is_success(&self) -> bool {
        matches!(*self, StdAck::Success(_))
    }

    #[must_use = "if you intended to assert that this is an error, consider `.unwrap_err()` instead"]
    #[inline]
    pub const fn is_error(&self) -> bool {
        !self.is_success()
    }

    /// Serialized the ack to binary using JSON. This used for setting the acknowledgement
    /// field in IbcReceiveResponse.
    ///
    /// ## Examples
    ///
    /// Show how the acknowledgement looks on the write:
    ///
    /// ```
    /// # use cosmwasm_std::StdAck;
    /// let ack1 = StdAck::success(b"\x01"); // 0x01 is a FungibleTokenPacketSuccess from ICS-20.
    /// assert_eq!(ack1.to_binary(), br#"{"result":"AQ=="}"#);
    ///
    /// let ack2 = StdAck::error("kaputt"); // Some free text error message
    /// assert_eq!(ack2.to_binary(), br#"{"error":"kaputt"}"#);
    /// ```
    ///
    /// Set acknowledgement field in `IbcReceiveResponse`:
    ///
    /// ```
    /// use cosmwasm_std::{StdAck, IbcReceiveResponse};
    ///
    /// let ack = StdAck::success(b"\x01"); // 0x01 is a FungibleTokenPacketSuccess from ICS-20.
    ///
    /// let res: IbcReceiveResponse = IbcReceiveResponse::new().set_ack(ack.to_binary());
    /// let res: IbcReceiveResponse = IbcReceiveResponse::new().set_ack(ack); // Does the same but consumes the instance
    /// ```
    pub fn to_binary(&self) -> Binary {
        // We need a non-failing StdAck -> Binary conversion to allow using StdAck in
        // `impl Into<Binary>` arguments.
        // Pretty sure this cannot fail. If that changes we can create a non-failing implementation here.
        to_json_binary(&self).unwrap()
    }

    pub fn unwrap(self) -> Binary {
        match self {
            StdAck::Success(data) => data,
            StdAck::Error(err) => panic!("{}", err),
        }
    }

    pub fn unwrap_err(self) -> String {
        match self {
            StdAck::Success(_) => panic!("not an error"),
            StdAck::Error(err) => err,
        }
    }
}

impl From<StdAck> for Binary {
    fn from(original: StdAck) -> Binary {
        original.to_binary()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdack_success_works() {
        let success = StdAck::success(b"foo");
        match success {
            StdAck::Success(data) => assert_eq!(data, b"foo"),
            StdAck::Error(_err) => panic!("must not be an error"),
        }
    }

    #[test]
    fn stdack_error_works() {
        let err = StdAck::error("bar");
        match err {
            StdAck::Success(_data) => panic!("must not be a success"),
            StdAck::Error(err) => assert_eq!(err, "bar"),
        }
    }

    #[test]
    fn stdack_is_success_is_error_work() {
        let success = StdAck::success(b"foo");
        let err = StdAck::error("bar");
        // is_success
        assert!(success.is_success());
        assert!(!err.is_success());
        // is_eror
        assert!(!success.is_error());
        assert!(err.is_error());
    }

    #[test]
    fn stdack_to_binary_works() {
        let ack1 = StdAck::success(b"\x01");
        assert_eq!(ack1.to_binary(), br#"{"result":"AQ=="}"#);

        let ack2 = StdAck::error("kaputt");
        assert_eq!(ack2.to_binary(), br#"{"error":"kaputt"}"#);
    }
}
