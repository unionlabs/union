use core::num::ParseIntError;

use crate::{Proto, TypeUrl};

#[derive(Debug, Clone, PartialEq)]
pub struct FungibleTokenPacketData {
    /// the token denomination to be transferred
    pub denom: String,
    /// the token amount to be transferred
    pub amount: u64,
    /// the sender address
    pub sender: String,
    /// the recipient address on the destination chain
    pub receiver: String,
    /// optional memo
    pub memo: Option<String>,
}

impl Proto for FungibleTokenPacketData {
    type Proto = protos::ibc::applications::transfer::v2::FungibleTokenPacketData;
}

impl TypeUrl for protos::ibc::applications::transfer::v2::FungibleTokenPacketData {
    const TYPE_URL: &'static str = "/ibc.applications.transfer.v2.FungibleTokenPacketData";
}

#[derive(Debug)]
pub enum TryFromFungibleTokenPacketDataError {
    Amount(ParseIntError),
}

impl TryFrom<protos::ibc::applications::transfer::v2::FungibleTokenPacketData>
    for FungibleTokenPacketData
{
    type Error = TryFromFungibleTokenPacketDataError;

    fn try_from(
        value: protos::ibc::applications::transfer::v2::FungibleTokenPacketData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            denom: value.denom,
            amount: value
                .amount
                .parse()
                .map_err(TryFromFungibleTokenPacketDataError::Amount)?,
            sender: value.sender,
            receiver: value.receiver,
            memo: (!value.memo.is_empty()).then_some(value.memo),
        })
    }
}

impl From<FungibleTokenPacketData>
    for protos::ibc::applications::transfer::v2::FungibleTokenPacketData
{
    fn from(value: FungibleTokenPacketData) -> Self {
        Self {
            denom: value.denom,
            amount: value.amount.to_string(),
            sender: value.sender,
            receiver: value.receiver,
            memo: value.memo.unwrap_or_default(),
        }
    }
}
